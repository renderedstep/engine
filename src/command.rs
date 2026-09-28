//! `Playthrough::Command` and its journal: one submitted line, however often
//! it is delivered, and the restart points of the turn that plays it.
//!
//! A submission is its token and its text. A resend of one submit (the same
//! token and the same line) is one turn: once it has completed, a delivery
//! hands back what it produced and plays nothing. The row's id is the order
//! lines were accepted in, and a turn plays every line accepted before its
//! own that is still owed a finish.
//!
//! The journal is the turn's receipts: each engine effect is written in the
//! same short transaction as the receipt that says it happened, and a model
//! answer is remembered only after the call returns. No transaction is ever
//! held across a model call. Values are written the way the Ruby engine
//! writes them (`Playthrough::Command::Journal#encode`), so either engine
//! reads the other's receipts.

use crate::engine::Error;
use crate::records::{int, text, Records, Row};
use crate::refusal::Refusal;
use crate::store::Store;
use serde_json::{json, Map, Value};

/// The steps whose effect a later line must wait for
/// (`Playthrough::Command#blocks_later?`).
pub const EFFECTS: [&str; 10] = [
    "take",
    "drop",
    "throw",
    "attack",
    "arrival_cost",
    "physical_effect",
    "character_effect",
    "narrated",
    "talked",
    "outcome",
];

fn steps(row: &Row) -> Map<String, Value> {
    row.get("journal")
        .and_then(|journal| journal.get("steps"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

fn status(row: &Row) -> &str {
    text(row, "status").unwrap_or("pending")
}

/// `#recoverable?`: a journalled turn that stopped part way, or failed after
/// it had written something.
pub fn recoverable(row: &Row) -> bool {
    let versioned = row.get("journal").and_then(|j| j.get("version")) == Some(&json!(1));
    versioned
        && (status(row) == "running"
            || (status(row) == "failed"
                && text(row, "error_kind") != Some("crisis")
                && !steps(row).is_empty()))
}

/// `#blocks_later?`.
pub fn blocks_later(row: &Row) -> bool {
    matches!(status(row), "pending" | "running")
        || (recoverable(row) && steps(row).keys().any(|key| EFFECTS.contains(&key.as_str())))
}

/// `#overtaken?`: a newer submission of this game has been started or
/// finished, so this one's own outcome is stale.
pub fn overtaken(records: &Records, row: &Row) -> bool {
    if blocks_later(row) {
        return false;
    }
    let (game, mine) = (
        int(row, "playthrough_id"),
        int(row, "id").unwrap_or_default(),
    );
    records
        .select("playthrough_commands", |other| {
            int(other, "playthrough_id") == game
                && int(other, "id").is_some_and(|id| id > mine)
                && status(other) != "pending"
        })
        .into_iter()
        .next()
        .is_some()
}

/// A command's row, found by its token and its text, or accepted now
/// (`.accept!`).
pub fn accept(
    store: &Store,
    records: &mut Records,
    playthrough: i64,
    line: &str,
    token: &str,
) -> Result<Row, Error> {
    if let Some(row) = records.first("playthrough_commands", |row| {
        int(row, "playthrough_id") == Some(playthrough)
            && text(row, "request_token") == Some(token)
            && text(row, "command") == Some(line)
    }) {
        return Ok(row.clone());
    }
    let row = store.insert(
        "playthrough_commands",
        &[
            ("playthrough_id", Value::from(playthrough)),
            ("request_token", Value::from(token)),
            ("command", Value::from(line)),
        ],
    )?;
    records.push("playthrough_commands", row.clone());
    Ok(row)
}

/// The lines this game accepted and has not finished, oldest first, up to
/// and including `mine` (`Playthrough::Turn#accepted_up_to`).
pub fn accepted_up_to(records: &Records, mine: &Row) -> Vec<Row> {
    if status(mine) == "completed" {
        return vec![mine.clone()];
    }
    let (game, id) = (
        int(mine, "playthrough_id"),
        int(mine, "id").unwrap_or_default(),
    );
    records
        .select("playthrough_commands", |row| {
            int(row, "playthrough_id") == game
                && status(row) != "completed"
                && int(row, "id").is_some_and(|other| other <= id)
                && (blocks_later(row) || int(row, "id") == Some(id))
        })
        .into_iter()
        .cloned()
        .collect()
}

/// What a finished submission produced, rebuilt from its own columns.
#[derive(Clone, Debug, PartialEq)]
pub enum Produced {
    Scene { id: i64, crisis: bool, setup: bool },
    Refused(Refusal),
    Nothing,
}

/// `#outcome`.
pub fn produced(row: &Row) -> Produced {
    let refusal = row
        .get("refusal")
        .and_then(Value::as_object)
        .filter(|r| !r.is_empty());
    if let Some(refusal) = refusal {
        let field = |key: &str| {
            refusal
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        let offer = refusal
            .get("offer")
            .and_then(Value::as_str)
            .map(str::to_string);
        return Refusal::new(&field("kind"), &field("typed"), &field("fact"), offer)
            .map(Produced::Refused)
            .unwrap_or(Produced::Nothing);
    }
    match int(row, "result_scene_id") {
        Some(id) => Produced::Scene {
            id,
            crisis: text(row, "error_kind") == Some("crisis"),
            setup: text(row, "error_kind") == Some("setup"),
        },
        None => Produced::Nothing,
    }
}

/// A command's receipts, open for one turn.
#[derive(Clone, Debug)]
pub struct Journal {
    pub command: i64,
    steps: Map<String, Value>,
}

impl Journal {
    /// The journal of a command that is about to run: its receipts so far,
    /// or a fresh version-1 journal.
    pub fn open(row: &Row) -> Journal {
        Journal {
            command: int(row, "id").expect("a command has an id"),
            steps: steps(row),
        }
    }

    pub fn saved(&self, key: &str) -> bool {
        self.steps.contains_key(key)
    }

    pub fn read(&self, key: &str) -> Option<&Value> {
        self.steps.get(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.steps.keys()
    }

    /// Writes one receipt onto the command's row.
    pub fn save(
        &mut self,
        store: &Store,
        records: &mut Records,
        key: &str,
        value: Value,
    ) -> Result<(), Error> {
        self.steps.insert(key.to_string(), value);
        let journal = json!({ "version": 1, "steps": Value::Object(self.steps.clone()) });
        let written = store.update(
            "playthrough_commands",
            self.command,
            &[("journal", journal)],
        )?;
        for (column, value) in written {
            records.set("playthrough_commands", self.command, &column, value);
        }
        Ok(())
    }
}

/// `Journal#encode` for the values a turn keeps.
pub mod encode {
    use crate::refusal::Refusal;
    use serde_json::{json, Value};

    pub fn symbol(name: &str) -> Value {
        json!({ "symbol": name })
    }

    pub fn array(values: Vec<Value>) -> Value {
        json!({ "array": values })
    }

    /// A hash with symbol keys, in order.
    pub fn hash(pairs: Vec<(&str, Value)>) -> Value {
        let pairs: Vec<Value> = pairs
            .into_iter()
            .map(|(key, value)| json!([symbol(key), value]))
            .collect();
        json!({ "hash": pairs })
    }

    pub fn text(value: Option<&str>) -> Value {
        value.map_or(Value::Null, Value::from)
    }

    /// A row, by the Ruby class that holds it.
    pub fn record(table: &str, id: i64) -> Value {
        let class = match table {
            "scenes" => "Scene",
            "locations" => "Location",
            "characters" => "Character",
            "items" => "Item",
            "playthrough_blows" => "Playthrough::Blow",
            "playthrough_tolls" => "Playthrough::Toll",
            "playthrough_endings" => "Playthrough::Ending",
            "quest_outcomes" => "Quest::Outcome",
            "location_connections" => "LocationConnection",
            other => panic!("{other} is not a table a journal names"),
        };
        json!({ "record": class, "id": id })
    }

    /// A scene, with what its turn knows about it beside the row.
    pub fn scene(
        id: i64,
        tolls: Option<&[i64]>,
        volitions: Option<&[i64]>,
        safety: bool,
        setup: bool,
    ) -> Value {
        json!({
            "record": "Scene",
            "id": id,
            "tolls": tolls,
            "volitions": volitions,
            "safety": safety,
            "setup": setup,
        })
    }

    pub fn refusal(refusal: &Refusal) -> Value {
        json!({ "refusal": hash(vec![
            ("kind", symbol(&refusal.kind)),
            ("typed", Value::from(refusal.typed.as_str())),
            ("fact", Value::from(refusal.fact.as_str())),
            ("offer", text(refusal.offer.as_deref())),
        ]) })
    }

    /// A value object, by its Ruby class.
    pub fn data(class: &str, fields: Vec<(&str, Value)>) -> Value {
        json!({ "data": class, "fields": hash(fields) })
    }
}

/// The refusal columns a completed command keeps.
pub fn refusal_columns(refusal: &Refusal) -> Value {
    json!({ "kind": refusal.kind, "typed": refusal.typed, "fact": refusal.fact, "offer": refusal.offer })
}
