//! The game's SQLite database, on a connection of the engine's own: the same
//! schema the Rails app migrates, read into [`Records`] and written back one
//! statement at a time.
//!
//! A column is read the way the Ruby engine's records hold it: a boolean as
//! true or false, a time as whole seconds since the Unix epoch, a JSON column
//! as its parsed value. Writing turns each back into what Rails stores: 1 or
//! 0, `YYYY-MM-DD HH:MM:SS`, JSON text.

use crate::clock;
use crate::engine::Error;
use crate::records::{Records, Row};
use rusqlite::types::{Value as Sql, ValueRef};
use rusqlite::{params_from_iter, Connection, OpenFlags};
use serde_json::{Map, Number, Value};
use std::collections::HashMap;
use std::path::Path;

/// The last migration the engine is written against (`db/schema.rb`'s
/// `version:`). A database at an older version is refused. One at a newer
/// version is opened only when every table the engine touches still has the
/// shape it had at this version ([`SHAPE`]): a later migration that adds a
/// table the engine never reads is harmless, and one that changes a table
/// the engine reads or writes is refused.
pub const SCHEMA_VERSION: &str = "20260927152056";

/// The shape, at [`SCHEMA_VERSION`], of every table the engine touches, as
/// [`shape`] describes it, one fact per line.
pub const SHAPE: &str = include_str!("store/shape.txt");

/// Every table the turn loop reads or writes, loaded whole at the start of a
/// turn.
pub const TABLES: &[&str] = &[
    "universes",
    "races",
    "stories",
    "characters",
    "locations",
    "location_connections",
    "items",
    "scenes",
    "characters_scenes",
    "interactions",
    "quests",
    "quest_steps",
    "quest_outcomes",
    "world_mechanics",
    "world_events",
    "locations_world_events",
    "playthroughs",
    "playthrough_beats",
    "playthrough_blows",
    "playthrough_commands",
    "playthrough_drifts",
    "playthrough_endings",
    "playthrough_npc_states",
    "playthrough_overreaches",
    "playthrough_passages",
    "playthrough_tolls",
    "playthrough_turn_events",
    "playthrough_vitals",
    "playthrough_volitions",
    "chats",
    "messages",
];

/// Tables the engine writes and never loads: the receipts a model call
/// leaves (`ruby_llm_usages` for a chat call, `system_one_receipts` for a
/// System One request), which no rule reads back.
pub const WRITTEN: &[&str] = &["ruby_llm_usages", "system_one_receipts"];

/// Tables the engine queries directly and never loads: the model registry a
/// receipt is priced from.
pub const QUERIED: &[&str] = &["ruby_llm_models"];

/// What the engine relies on in a database, one fact per line, sorted: for
/// every table it touches, each column's declared type (in lower case, as
/// SQLite compares it), nullability, default
/// and key position, each index's uniqueness, columns and condition, each
/// foreign key and its actions, and each trigger; and every foreign key in
/// any other table that points into one of them, since a row there can stop
/// the engine deleting or rewriting one of its own. A table the engine does
/// not touch, with no foreign key into one it does, is left out, so a
/// migration that only adds such a table leaves the shape as it was.
pub fn shape(conn: &Connection) -> Result<Vec<String>, Error> {
    let touched: Vec<&str> = TABLES
        .iter()
        .chain(WRITTEN)
        .chain(QUERIED)
        .copied()
        .collect();
    let mut facts = Vec::new();
    for table in &touched {
        let mut statement = conn
            .prepare("SELECT name, type, \"notnull\", dflt_value, pk FROM pragma_table_info(?1)")?;
        let columns = statement
            .query_map([table], |row| {
                Ok(format!(
                    "column {table}.{} {}{}{} pk={}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?.to_ascii_lowercase(),
                    if row.get::<_, bool>(2)? {
                        " not null"
                    } else {
                        ""
                    },
                    row.get::<_, Option<String>>(3)?
                        .map_or(String::new(), |default| format!(" default {default}")),
                    row.get::<_, i64>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        if columns.is_empty() {
            facts.push(format!("missing {table}"));
        }
        facts.extend(columns);
        let mut statement = conn.prepare(
            "SELECT l.name, l.\"unique\", l.origin, l.partial, m.sql FROM pragma_index_list(?1) l \
             LEFT JOIN sqlite_master m ON m.type = 'index' AND m.name = l.name",
        )?;
        let indexes = statement
            .query_map([table], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, bool>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (name, unique, origin, partial, sql) in indexes {
            let mut statement =
                conn.prepare("SELECT name FROM pragma_index_info(?1) ORDER BY seqno")?;
            let columns = statement
                .query_map([&name], |row| row.get::<_, Option<String>>(0))?
                .map(|column| column.map(|column| column.unwrap_or_else(|| "(expression)".into())))
                .collect::<Result<Vec<_>, _>>()?;
            let condition = match (partial, sql) {
                (true, Some(sql)) => sql
                    .find(" WHERE ")
                    .map_or(String::new(), |at| sql[at..].to_string()),
                _ => String::new(),
            };
            facts.push(format!(
                "index {table} ({}){}{}{condition}",
                columns.join(", "),
                if unique { " unique" } else { "" },
                if origin == "pk" { " primary key" } else { "" },
            ));
        }
        let mut statement = conn
            .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' AND tbl_name = ?1")?;
        let triggers = statement
            .query_map([table], |row| {
                Ok(format!("trigger {table} {}", row.get::<_, String>(0)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        facts.extend(triggers);
    }
    let mut statement = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let every: Vec<String> = statement
        .query_map([], |row| row.get(0))?
        .collect::<Result<_, _>>()?;
    for table in &every {
        let mut statement = conn.prepare(
            "SELECT \"table\", \"from\", \"to\", on_update, on_delete FROM pragma_foreign_key_list(?1)",
        )?;
        let keys = statement
            .query_map([table], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    format!(
                        "foreign key {table}.{} -> {}.{} on update {} on delete {}",
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(0)?,
                        row.get::<_, Option<String>>(2)?
                            .unwrap_or_else(|| "id".into()),
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                    ),
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        for (target, fact) in keys {
            if touched.contains(&table.as_str()) || touched.contains(&target.as_str()) {
                facts.push(fact);
            }
        }
    }
    facts.sort();
    Ok(facts)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Integer,
    Real,
    Text,
    Boolean,
    Time,
    Json,
}

impl Kind {
    fn of(declared: &str) -> Kind {
        let declared = declared.to_ascii_lowercase();
        if declared == "boolean" {
            Kind::Boolean
        } else if declared.starts_with("datetime") {
            Kind::Time
        } else if declared == "json" {
            Kind::Json
        } else if declared == "integer" || declared == "bigint" {
            Kind::Integer
        } else if declared.starts_with("decimal")
            || declared.starts_with("float")
            || declared == "real"
        {
            Kind::Real
        } else {
            Kind::Text
        }
    }
}

#[derive(Clone, Debug)]
struct Column {
    name: String,
    kind: Kind,
}

/// One database, open.
pub struct Store {
    conn: Connection,
    columns: HashMap<String, Vec<Column>>,
}

impl Store {
    /// Opens the database at `path` for reading and writing, and refuses it
    /// unless its schema is the one this engine is written against.
    pub fn open(path: &Path) -> Result<Store, Error> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        Store::from_connection(conn)
    }

    /// Takes over a connection already open, checking its schema.
    pub fn from_connection(conn: Connection) -> Result<Store, Error> {
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let found: Option<String> = conn
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .map_err(|_| Error::SchemaMismatch {
                found: None,
                expected: SCHEMA_VERSION.to_string(),
            })?;
        match found.as_deref() {
            Some(version) if version == SCHEMA_VERSION => {}
            Some(version) if newer(version) => {
                let expected: Vec<&str> = SHAPE.lines().collect();
                let actual = shape(&conn)?;
                let mut differences: Vec<String> = expected
                    .iter()
                    .filter(|fact| !actual.iter().any(|found| found == *fact))
                    .map(|fact| format!("expected {fact}"))
                    .collect();
                differences.extend(
                    actual
                        .iter()
                        .filter(|fact| !expected.contains(&fact.as_str()))
                        .map(|fact| format!("found {fact}")),
                );
                if !differences.is_empty() {
                    return Err(Error::SchemaChanged {
                        found: version.to_string(),
                        differences,
                    });
                }
            }
            _ => {
                return Err(Error::SchemaMismatch {
                    found,
                    expected: SCHEMA_VERSION.to_string(),
                })
            }
        }
        let mut columns = HashMap::new();
        for table in TABLES.iter().chain(WRITTEN) {
            let mut statement =
                conn.prepare("SELECT name, type FROM pragma_table_info(?1) ORDER BY name")?;
            let found: Vec<Column> = statement
                .query_map([table], |row| {
                    Ok(Column {
                        name: row.get(0)?,
                        kind: Kind::of(&row.get::<_, String>(1)?),
                    })
                })?
                .collect::<Result<_, _>>()?;
            if found.is_empty() {
                return Err(Error::Database(format!(
                    "the database has no {table} table"
                )));
            }
            columns.insert(table.to_string(), found);
        }
        Ok(Store { conn, columns })
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    fn columns(&self, table: &str) -> Result<&[Column], Error> {
        self.columns
            .get(table)
            .map(Vec::as_slice)
            .ok_or_else(|| Error::Database(format!("{table} is not a table the engine reads")))
    }

    fn has_id(&self, table: &str) -> Result<bool, Error> {
        Ok(self
            .columns(table)?
            .iter()
            .any(|column| column.name == "id"))
    }

    /// Every row of every table the loop reads, in id order.
    pub fn load(&self) -> Result<Records, Error> {
        let mut records = Records::default();
        for table in TABLES {
            let columns = self.columns(table)?;
            let order = if self.has_id(table)? {
                "\"id\"".to_string()
            } else {
                columns
                    .iter()
                    .map(|column| format!("\"{}\"", column.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let rows = self.select(table, &format!("ORDER BY {order}"), &[])?;
            records.set_table(table, rows);
        }
        Ok(records)
    }

    /// The rows of one table that `condition` (SQL after `WHERE`, or an
    /// `ORDER BY`) selects.
    fn select(&self, table: &str, condition: &str, params: &[Sql]) -> Result<Vec<Row>, Error> {
        let columns = self.columns(table)?;
        let names = columns
            .iter()
            .map(|column| format!("\"{}\"", column.name))
            .collect::<Vec<_>>()
            .join(", ");
        let mut statement = self
            .conn
            .prepare(&format!("SELECT {names} FROM \"{table}\" {condition}"))?;
        let rows = statement
            .query_map(params_from_iter(params.iter()), |row| {
                let mut record = Map::new();
                for (index, column) in columns.iter().enumerate() {
                    record.insert(column.name.clone(), read(column.kind, row.get_ref(index)?));
                }
                Ok(record)
            })?
            .collect::<Result<_, _>>()?;
        Ok(rows)
    }

    /// Inserts a row, stamping `created_at` and `updated_at` where the table
    /// has them, and returns it as it now stands, id and defaults included.
    pub fn insert(&self, table: &str, values: &[(&str, Value)]) -> Result<Row, Error> {
        let columns = self.columns(table)?;
        let (_, stamp) = clock::now();
        let mut names = Vec::new();
        let mut params = Vec::new();
        for (name, value) in values {
            let column = find(columns, table, name)?;
            names.push(format!("\"{name}\""));
            params.push(write(column.kind, value));
        }
        for stamped in ["created_at", "updated_at"] {
            if columns.iter().any(|column| column.name == stamped)
                && !values.iter().any(|(name, _)| *name == stamped)
            {
                names.push(format!("\"{stamped}\""));
                params.push(Sql::Text(stamp.clone()));
            }
        }
        let marks = vec!["?"; names.len()].join(", ");
        self.conn.execute(
            &format!(
                "INSERT INTO \"{table}\" ({}) VALUES ({marks})",
                names.join(", ")
            ),
            params_from_iter(params.iter()),
        )?;
        if !self.has_id(table)? {
            let mut row = Map::new();
            for (name, value) in values {
                row.insert(name.to_string(), value.clone());
            }
            return Ok(row);
        }
        let id = self.conn.last_insert_rowid();
        self.select(table, "WHERE \"id\" = ?", &[Sql::Integer(id)])?
            .pop()
            .ok_or_else(|| Error::Database(format!("{table} {id} was not written")))
    }

    /// Writes columns of one row, and its `updated_at`. Returns the values
    /// written, `updated_at` among them, as a row holds them.
    pub fn update(&self, table: &str, id: i64, values: &[(&str, Value)]) -> Result<Row, Error> {
        let columns = self.columns(table)?;
        let (seconds, stamp) = clock::now();
        let mut sets = Vec::new();
        let mut params = Vec::new();
        let mut written = Map::new();
        for (name, value) in values {
            let column = find(columns, table, name)?;
            sets.push(format!("\"{name}\" = ?"));
            params.push(write(column.kind, value));
            written.insert(name.to_string(), value.clone());
        }
        if columns.iter().any(|column| column.name == "updated_at") {
            sets.push("\"updated_at\" = ?".to_string());
            params.push(Sql::Text(stamp));
            written.insert("updated_at".into(), Value::from(seconds));
        }
        params.push(Sql::Integer(id));
        let changed = self.conn.execute(
            &format!(
                "UPDATE \"{table}\" SET {} WHERE \"id\" = ?",
                sets.join(", ")
            ),
            params_from_iter(params.iter()),
        )?;
        if changed != 1 {
            return Err(Error::Database(format!(
                "{table} {id} is not there to update"
            )));
        }
        Ok(written)
    }

    /// Deletes one row.
    pub fn delete(&self, table: &str, id: i64) -> Result<(), Error> {
        self.columns(table)?;
        self.conn
            .execute(&format!("DELETE FROM \"{table}\" WHERE \"id\" = ?"), [id])?;
        Ok(())
    }

    pub fn begin(&self) -> Result<(), Error> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        Ok(())
    }

    pub fn commit(&self) -> Result<(), Error> {
        self.conn.execute_batch("COMMIT")?;
        Ok(())
    }

    /// Opens a savepoint: a transaction of its own when none is open, a
    /// nested one inside one.
    pub fn savepoint(&self) -> Result<(), Error> {
        self.conn.execute_batch("SAVEPOINT journal")?;
        Ok(())
    }

    /// Keeps what the innermost savepoint wrote.
    pub fn release(&self) -> Result<(), Error> {
        self.conn.execute_batch("RELEASE journal")?;
        Ok(())
    }

    /// Takes back what the innermost savepoint wrote, and closes it.
    pub fn rollback_to(&self) {
        let _ = self
            .conn
            .execute_batch("ROLLBACK TO journal; RELEASE journal");
    }

    pub fn rollback(&self) {
        let _ = self.conn.execute_batch("ROLLBACK");
    }
}

/// Whether a migration version comes after [`SCHEMA_VERSION`]. Rails'
/// versions are fourteen-digit timestamps, so they order as numbers.
fn newer(version: &str) -> bool {
    match (version.parse::<u64>(), SCHEMA_VERSION.parse::<u64>()) {
        (Ok(found), Ok(pinned)) => found > pinned,
        _ => false,
    }
}

fn find<'a>(columns: &'a [Column], table: &str, name: &str) -> Result<&'a Column, Error> {
    columns
        .iter()
        .find(|column| column.name == name)
        .ok_or_else(|| Error::Database(format!("{table} has no column {name}")))
}

fn read(kind: Kind, value: ValueRef) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(n) => match kind {
            Kind::Boolean => Value::Bool(n != 0),
            Kind::Real => Number::from_f64(n as f64).map_or(Value::Null, Value::Number),
            _ => Value::from(n),
        },
        ValueRef::Real(f) => Number::from_f64(f).map_or(Value::Null, Value::Number),
        ValueRef::Text(bytes) => {
            let text = String::from_utf8_lossy(bytes);
            match kind {
                Kind::Boolean => Value::Bool(matches!(&*text, "t" | "true" | "1")),
                Kind::Time => clock::parse(&text).map_or(Value::Null, Value::from),
                Kind::Json => serde_json::from_str(&text).unwrap_or(Value::String(text.into())),
                Kind::Integer => text
                    .parse::<i64>()
                    .map_or(Value::String(text.to_string()), Value::from),
                _ => Value::String(text.into_owned()),
            }
        }
        ValueRef::Blob(bytes) => Value::String(String::from_utf8_lossy(bytes).into_owned()),
    }
}

fn write(kind: Kind, value: &Value) -> Sql {
    match value {
        Value::Null => Sql::Null,
        Value::Bool(flag) => Sql::Integer(i64::from(*flag)),
        Value::Number(n) => match (kind, n.as_i64()) {
            (Kind::Time, Some(seconds)) => Sql::Text(clock::format(seconds)),
            (_, Some(n)) => Sql::Integer(n),
            _ => Sql::Real(n.as_f64().unwrap_or_default()),
        },
        Value::String(text) => Sql::Text(text.clone()),
        other => Sql::Text(other.to_string()),
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Error {
        Error::Database(error.to_string())
    }
}
