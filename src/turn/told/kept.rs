//! What each journal step keeps, and how a resumed turn reads it back
//! (`Playthrough::Command::Journal#encode` and `#decode`): a value a step
//! saved is the value the step answers the second time, without running.

use super::{Played, Told, Turn};
use crate::command::encode;
use crate::engine::Error;
use crate::intent::Intent;
use crate::physics::{self, Break, Reach};
use crate::records::{id, int, string, text, Row};
use crate::refusal::Refusal;
use crate::room::{Choice, Exit, Place, Record};
use crate::turn::Concluded;
use crate::turn::{person_of, thing_of, Report};
use serde_json::{json, Map, Value};

/// A value one journal step keeps.
pub(super) trait Kept: Sized {
    fn encode(&self) -> Value;
    fn decode(turn: &Turn, value: &Value) -> Result<Self, Error>;
}

fn unreadable(what: &str, value: &Value) -> Error {
    Error::Database(format!("a journal holds {value} where {what} belongs"))
}

/// The pairs of an encoded Hash, keys decoded as strings.
fn pairs(value: &Value) -> Result<Map<String, Value>, Error> {
    let rows = value["hash"]
        .as_array()
        .ok_or_else(|| unreadable("a hash", value))?;
    let mut map = Map::new();
    for row in rows {
        let key = match &row[0] {
            Value::String(key) => key.clone(),
            other => other["symbol"]
                .as_str()
                .ok_or_else(|| unreadable("a key", other))?
                .to_string(),
        };
        map.insert(key, row[1].clone());
    }
    Ok(map)
}

/// A data value's fields.
fn fields(value: &Value) -> Result<Map<String, Value>, Error> {
    pairs(&value["fields"])
}

fn symbol_or_string(value: &Value) -> Option<String> {
    value["symbol"]
        .as_str()
        .or_else(|| value.as_str())
        .map(str::to_string)
}

/// The inverse of the Ruby encoding for plain values.
pub(super) fn plain(value: &Value) -> Value {
    if let Some(items) = value.get("array").and_then(Value::as_array) {
        return items.iter().map(plain).collect();
    }
    if value.get("hash").is_some() {
        return pairs(value)
            .map(|map| Value::Object(map.into_iter().map(|(k, v)| (k, plain(&v))).collect()))
            .unwrap_or(Value::Null);
    }
    if let Some(symbol) = value.get("symbol").and_then(Value::as_str) {
        return Value::from(symbol);
    }
    value.clone()
}

impl Kept for () {
    fn encode(&self) -> Value {
        Value::Null
    }
    fn decode(_: &Turn, _: &Value) -> Result<(), Error> {
        Ok(())
    }
}

impl Kept for bool {
    fn encode(&self) -> Value {
        Value::Bool(*self)
    }
    fn decode(_: &Turn, value: &Value) -> Result<bool, Error> {
        Ok(value.as_bool().unwrap_or(false))
    }
}

/// Ids, as the journal keeps a list.
impl Kept for Vec<i64> {
    fn encode(&self) -> Value {
        json!({ "array": self })
    }
    fn decode(_: &Turn, value: &Value) -> Result<Vec<i64>, Error> {
        value["array"]
            .as_array()
            .map(|ids| ids.iter().filter_map(Value::as_i64).collect())
            .ok_or_else(|| unreadable("a list", value))
    }
}

impl Kept for i64 {
    fn encode(&self) -> Value {
        Value::from(*self)
    }
    fn decode(_: &Turn, value: &Value) -> Result<i64, Error> {
        value.as_i64().ok_or_else(|| unreadable("a number", value))
    }
}

impl Kept for Option<Refusal> {
    fn encode(&self) -> Value {
        self.as_ref().map_or(Value::Null, encode::refusal)
    }
    fn decode(_: &Turn, value: &Value) -> Result<Option<Refusal>, Error> {
        if value.is_null() {
            return Ok(None);
        }
        let map = pairs(&value["refusal"])?;
        let field = |key: &str| map.get(key).and_then(symbol_or_string).unwrap_or_default();
        let offer = map.get("offer").and_then(Value::as_str).map(str::to_string);
        Refusal::new(&field("kind"), &field("typed"), &field("fact"), offer)
            .map(Some)
            .map_err(|_| unreadable("a refusal", value))
    }
}

/// The room a turn began in.
impl Kept for Option<Row> {
    fn encode(&self) -> Value {
        self.as_ref()
            .map_or(Value::Null, |room| encode::record("locations", id(room)))
    }
    fn decode(turn: &Turn, value: &Value) -> Result<Option<Row>, Error> {
        match value["id"].as_i64() {
            None => Ok(None),
            Some(room) => turn.m.row("locations", room).map(Some),
        }
    }
}

impl Kept for Told {
    fn encode(&self) -> Value {
        encode::scene(
            self.id,
            self.tolls.as_deref(),
            self.volitions.as_deref(),
            self.safety,
            self.setup,
        )
    }
    fn decode(_: &Turn, value: &Value) -> Result<Told, Error> {
        let id = value["id"]
            .as_i64()
            .ok_or_else(|| unreadable("a scene", value))?;
        Ok(Told {
            id,
            tolls: value["tolls"]
                .as_array()
                .map(|tolls| tolls.iter().filter_map(Value::as_i64).collect()),
            volitions: match &value["volitions"] {
                // Written before a scene named the volitions it stated: a
                // prompt that stated none.
                Value::Bool(false) => Some(Vec::new()),
                Value::Array(told) => Some(told.iter().filter_map(Value::as_i64).collect()),
                _ => None,
            },
            safety: value["safety"].as_bool().unwrap_or(false),
            setup: value["setup"].as_bool().unwrap_or(false),
        })
    }
}

impl Kept for Option<Told> {
    fn encode(&self) -> Value {
        self.as_ref().map_or(Value::Null, Kept::encode)
    }
    fn decode(turn: &Turn, value: &Value) -> Result<Option<Told>, Error> {
        if value.is_null() {
            return Ok(None);
        }
        Told::decode(turn, value).map(Some)
    }
}

/// `Playthrough::Arc::Concluded`, or nothing on a line that ended no arc.
impl Kept for Option<Concluded> {
    fn encode(&self) -> Value {
        self.map_or(Value::Null, |concluded| {
            encode::data(
                "Playthrough::Arc::Concluded",
                vec![
                    (
                        "ending",
                        encode::record("playthrough_endings", concluded.ending),
                    ),
                    (
                        "outcome",
                        encode::record("quest_outcomes", concluded.outcome),
                    ),
                    ("scene", encode::record("scenes", concluded.scene)),
                ],
            )
        })
    }
    fn decode(_: &Turn, value: &Value) -> Result<Option<Concluded>, Error> {
        if value.is_null() {
            return Ok(None);
        }
        let map = fields(value)?;
        let id_of = |key: &str| {
            map.get(key)
                .and_then(|record| record["id"].as_i64())
                .ok_or_else(|| unreadable("an ending", value))
        };
        Ok(Some(Concluded {
            ending: id_of("ending")?,
            outcome: id_of("outcome")?,
            scene: id_of("scene")?,
        }))
    }
}

/// `Playthrough::PhysicalAction::Result`: a status and the engine's fact.
#[derive(Clone, Debug)]
pub(super) struct Effect {
    pub status: String,
    pub fact: String,
}

impl Kept for Effect {
    fn encode(&self) -> Value {
        encode::data(
            "Playthrough::PhysicalAction::Result",
            vec![
                ("status", Value::from(self.status.as_str())),
                ("fact", Value::from(self.fact.as_str())),
            ],
        )
    }
    fn decode(_: &Turn, value: &Value) -> Result<Effect, Error> {
        let map = fields(value)?;
        let field = |key: &str| {
            map.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        Ok(Effect {
            status: field("status"),
            fact: field("fact"),
        })
    }
}

/// `Playthrough::NpcAction::Result`: the token chosen, its status and the
/// engine's receipt.
#[derive(Clone, Debug)]
pub(super) struct NpcEffect {
    pub action: String,
    pub status: String,
    pub fact: String,
}

impl Kept for NpcEffect {
    fn encode(&self) -> Value {
        encode::data(
            "Playthrough::NpcAction::Result",
            vec![
                ("action", Value::from(self.action.as_str())),
                ("status", Value::from(self.status.as_str())),
                ("fact", Value::from(self.fact.as_str())),
            ],
        )
    }
    fn decode(_: &Turn, value: &Value) -> Result<NpcEffect, Error> {
        let map = fields(value)?;
        let field = |key: &str| {
            map.get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string()
        };
        Ok(NpcEffect {
            action: field("action"),
            status: field("status"),
            fact: field("fact"),
        })
    }
}

/// A character's parsed answer and the fields the turn keeps of it.
impl Kept for (Value, Map<String, Value>) {
    fn encode(&self) -> Value {
        encode::array(vec![
            ruby_hash(&self.0),
            ruby_hash(&Value::Object(self.1.clone())),
        ])
    }
    fn decode(_: &Turn, value: &Value) -> Result<(Value, Map<String, Value>), Error> {
        let items = value["array"]
            .as_array()
            .ok_or_else(|| unreadable("an answer", value))?;
        let answer = plain(&items[0]);
        let kept = plain(&items[1]).as_object().cloned().unwrap_or_default();
        Ok((answer, kept))
    }
}

/// A parsed answer as the Ruby engine keeps a Hash with string keys.
fn ruby_hash(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let rows: Vec<Value> = map
                .iter()
                .map(|(key, value)| json!([key, ruby_hash(value)]))
                .collect();
            json!({ "hash": rows })
        }
        Value::Array(items) => encode::array(items.iter().map(ruby_hash).collect()),
        other => other.clone(),
    }
}

/// The break die thrown for a thing that came down on a floor, or nothing
/// where none was: its sides, the face it came up, the share it broke on and
/// whether it broke. The one record of a break that held.
impl Kept for Option<Break> {
    fn encode(&self) -> Value {
        self.map_or(Value::Null, |rolled| {
            encode::data(
                "Physics::Break",
                vec![
                    ("sides", Value::from(physics::break_die())),
                    ("die", Value::from(rolled.die)),
                    ("share", Value::from(rolled.share)),
                    ("broke", Value::from(rolled.broke)),
                ],
            )
        })
    }
    fn decode(_: &Turn, value: &Value) -> Result<Option<Break>, Error> {
        if value.is_null() {
            return Ok(None);
        }
        break_of(value).map(Some)
    }
}

fn break_of(value: &Value) -> Result<Break, Error> {
    let map = fields(value)?;
    let number = |key: &str| {
        map.get(key)
            .and_then(Value::as_i64)
            .ok_or_else(|| unreadable("a break", value))
    };
    Ok(Break {
        share: number("share")?,
        die: number("die")?,
        broke: map
            .get("broke")
            .and_then(Value::as_bool)
            .ok_or_else(|| unreadable("a break", value))?,
    })
}

/// A throw, as the report the offline writer gives of it.
impl Kept for Report {
    fn encode(&self) -> Value {
        encode::data(
            "Playthrough::Turn::Throw",
            vec![
                (
                    "change",
                    self.change.as_deref().map_or(Value::Null, Value::from),
                ),
                (
                    "refusal",
                    self.refusal.as_deref().map_or(Value::Null, Value::from),
                ),
                (
                    "note",
                    encode::array(self.note.iter().map(|n| Value::from(n.as_str())).collect()),
                ),
            ]
            .into_iter()
            .chain(
                self.break_roll
                    .map(|rolled| ("break", Some(rolled).encode())),
            )
            .chain(self.reach.map(|reach| {
                (
                    "reach",
                    encode::hash(vec![
                        ("range", Value::from(reach.range)),
                        ("distance", reach.distance.map_or(Value::Null, Value::from)),
                    ]),
                )
            }))
            .collect(),
        )
    }
    fn decode(_: &Turn, value: &Value) -> Result<Report, Error> {
        let map = fields(value)?;
        let text = |key: &str| map.get(key).and_then(Value::as_str).map(str::to_string);
        let note = map
            .get("note")
            .map(plain)
            .and_then(|note| note.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(|n| n.as_str().map(str::to_string))
            .collect();
        let reach = map.get("reach").map(plain).and_then(|reach| {
            Some(Reach {
                range: reach["range"].as_i64()?,
                distance: reach["distance"].as_i64(),
            })
        });
        let break_roll = map.get("break").map(break_of).transpose()?;
        Ok(Report {
            change: text("change"),
            refusal: text("refusal"),
            note,
            reach,
            break_roll,
            ..Report::default()
        })
    }
}

impl Kept for Played {
    fn encode(&self) -> Value {
        match self {
            Played::Scene(told) => told.encode(),
            Played::Refused(refusal) => encode::refusal(refusal),
            Played::Nothing => Value::Null,
        }
    }
    fn decode(turn: &Turn, value: &Value) -> Result<Played, Error> {
        if value.is_null() {
            return Ok(Played::Nothing);
        }
        if value.get("refusal").is_some() {
            return Ok(
                Option::<Refusal>::decode(turn, value)?.map_or(Played::Nothing, Played::Refused)
            );
        }
        Told::decode(turn, value).map(Played::Scene)
    }
}

// --- the reading -------------------------------------------------------------

fn record_encoded(record: &Option<Record>) -> Value {
    match record {
        None => Value::Null,
        Some(Record::Place(place)) => encode::record("locations", place.id),
        Some(Record::Person(person)) => encode::record("characters", person.id),
        Some(Record::Thing(thing)) => encode::record("items", thing.id),
        Some(Record::Attempt(choice)) => choice_encoded(choice),
    }
}

fn choice_encoded(choice: &Choice) -> Value {
    let thing = |thing: &Option<crate::room::Thing>| {
        thing
            .as_ref()
            .map_or(Value::Null, |t| encode::record("items", t.id))
    };
    encode::data(
        "Playthrough::PhysicalAction::Choice",
        vec![
            ("kind", Value::from(choice.kind.as_str())),
            ("item", thing(&choice.item)),
            (
                "recipient",
                choice
                    .recipient
                    .as_ref()
                    .map_or(Value::Null, |p| encode::record("characters", p.id)),
            ),
            (
                "connection",
                choice.connection.as_ref().map_or(Value::Null, |exit| {
                    encode::record("location_connections", exit.edge)
                }),
            ),
            ("tool", thing(&choice.tool)),
        ],
    )
}

impl Turn<'_, '_> {
    fn thing(&self, item: i64) -> Result<crate::room::Thing, Error> {
        let row = self.m.row("items", item)?;
        let carried = int(&row, "playthrough_id") == Some(self.m.playthrough)
            && int(&row, "location_id").is_none()
            && int(&row, "character_id").is_none();
        Ok(thing_of(&row, carried))
    }

    fn record_of(&self, value: &Value) -> Result<Option<Record>, Error> {
        if value.is_null() {
            return Ok(None);
        }
        if value.get("data").is_some() {
            return self
                .choice_of(value)
                .map(|choice| Some(Record::Attempt(Box::new(choice))));
        }
        let row = value["id"]
            .as_i64()
            .ok_or_else(|| unreadable("a record", value))?;
        Ok(Some(match value["record"].as_str() {
            Some("Location") => {
                let room = self.m.row("locations", row)?;
                Record::Place(Place {
                    id: row,
                    name: string(&room, "name").to_string(),
                })
            }
            Some("Character") => Record::Person(person_of(&self.m.row("characters", row)?)),
            Some("Item") => Record::Thing(self.thing(row)?),
            _ => return Err(unreadable("a record", value)),
        }))
    }

    fn choice_of(&self, value: &Value) -> Result<Choice, Error> {
        let map = fields(value)?;
        let id_of = |key: &str| map.get(key).and_then(|v| v["id"].as_i64());
        let connection = match id_of("connection") {
            None => None,
            Some(edge) => {
                let row = self.m.row("location_connections", edge)?;
                let far = self.m.row(
                    "locations",
                    int(&row, "connected_location_id").unwrap_or_default(),
                )?;
                Some(Exit {
                    place: Place {
                        id: id(&far),
                        name: string(&far, "name").to_string(),
                    },
                    barrier: if self.m.open_for(&row) {
                        "open".into()
                    } else {
                        text(&row, "barrier").unwrap_or("open").to_string()
                    },
                    edge,
                    key: int(&row, "key_template_id"),
                })
            }
        };
        Ok(Choice {
            kind: map
                .get("kind")
                .and_then(symbol_or_string)
                .unwrap_or_default(),
            item: id_of("item").map(|item| self.thing(item)).transpose()?,
            recipient: id_of("recipient")
                .map(|who| self.m.row("characters", who).map(|row| person_of(&row)))
                .transpose()?,
            connection,
            tool: id_of("tool").map(|item| self.thing(item)).transpose()?,
        })
    }
}

/// The act the game picked in a game the player narrates, and the line that
/// plays it; none when there was nothing to pick.
impl Kept for Option<(Intent, String)> {
    fn encode(&self) -> Value {
        self.as_ref().map_or(Value::Null, Kept::encode)
    }
    fn decode(turn: &Turn, value: &Value) -> Result<Option<(Intent, String)>, Error> {
        if value.is_null() {
            return Ok(None);
        }
        <(Intent, String)>::decode(turn, value).map(Some)
    }
}

impl Kept for (Intent, String) {
    fn encode(&self) -> Value {
        let (intent, resolved_by) = self;
        encode::array(vec![
            encode::data(
                "Playthrough::Classifier::Intent",
                vec![
                    ("action", encode::symbol(&intent.action)),
                    ("destination", record_encoded(&intent.destination)),
                    ("speaker", record_encoded(&intent.speaker)),
                    ("item", record_encoded(&intent.item)),
                    ("at", record_encoded(&intent.at)),
                    ("also_named", record_encoded(&intent.also_named)),
                    (
                        "unknown_action",
                        intent
                            .unknown_action
                            .as_deref()
                            .map_or(Value::Null, Value::from),
                    ),
                    (
                        "physical",
                        intent.physical.as_ref().map_or(Value::Null, choice_encoded),
                    ),
                ],
            ),
            Value::from(resolved_by.as_str()),
        ])
    }

    fn decode(turn: &Turn, value: &Value) -> Result<(Intent, String), Error> {
        let items = value["array"]
            .as_array()
            .ok_or_else(|| unreadable("a reading", value))?;
        let map = fields(&items[0])?;
        let record = |key: &str| turn.record_of(map.get(key).unwrap_or(&Value::Null));
        let intent = Intent {
            action: map
                .get("action")
                .and_then(symbol_or_string)
                .unwrap_or_default(),
            destination: record("destination")?,
            speaker: record("speaker")?,
            item: record("item")?,
            at: record("at")?,
            also_named: record("also_named")?,
            unknown_action: map
                .get("unknown_action")
                .and_then(Value::as_str)
                .map(str::to_string),
            physical: match map.get("physical").filter(|v| !v.is_null()) {
                Some(choice) => Some(turn.choice_of(choice)?),
                None => None,
            },
        };
        Ok((intent, items[1].as_str().unwrap_or_default().to_string()))
    }
}
