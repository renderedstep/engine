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

/// The last migration the engine is written against
/// (`db/schema.rb`'s `version:`). A database at any other version is refused:
/// a column this engine does not know about, or one it expects and the
/// database lacks, would be read or written wrongly.
pub const SCHEMA_VERSION: &str = "20260927031841";

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
        if found.as_deref() != Some(SCHEMA_VERSION) {
            return Err(Error::SchemaMismatch {
                found,
                expected: SCHEMA_VERSION.to_string(),
            });
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

    pub fn rollback(&self) {
        let _ = self.conn.execute_batch("ROLLBACK");
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
