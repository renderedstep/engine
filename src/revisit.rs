//! What a game saw of a room when it last left it, and what has changed
//! there since, among what it saw: the arrival diff.
//!
//! THE RECORD IS A RECEIPT OF THE MOVE. A submitted turn that walks out of a
//! room keeps, in its journal, what the game had noticed of that room as it
//! left ([`Seen`], step [`LEFT`]): the things lying there it had noticed,
//! and where each lay; the people there, and whether each was alive; the
//! ways out; and the story time. A turn that walks back into a room this
//! game left before compares that record with the room as it is now and
//! keeps the changes it found ([`Since`], step [`SINCE`]). Both are kept in
//! every game, whoever tells it; the record is the engine's, and no prompt
//! reads either.
//!
//! WHAT WAS NOT NOTICED CANNOT HAVE CHANGED. In a game the player narrates,
//! the things the record holds are the ones this game had stamped
//! ([`crate::noticed`]); in a game told any other way, the player was shown
//! everything lying there, so the record holds all of it. A thing that came
//! into the room while the game was away is not a change: the record cannot
//! say it was not there before, unnoticed.
//!
//! THE CHANGES ARE A CLOSED LIST ([`Kind`]), each the engine's own sentence
//! about one row:
//!
//! - a thing the game saw is gone, is held by somebody here now, lies
//!   somewhere else in the room, or lies broken;
//! - a person the game saw has left, or died, or their body is gone; a
//!   person is here now who was not;
//! - a way out is gone, or new;
//! - an event of the world touched the room. A rearrangement of doorways
//!   (`world_mechanic`) is said by the ways out it changed, not again here.
//!
//! A turn played with no submission (`Engine::play`) keeps no journal, so it
//! leaves no record: a return after it is compared with the record the last
//! submitted departure kept.

use crate::playthrough::Game;
use crate::records::{flag, id, int, string, text, Records, Row};
use serde_json::{json, Value};

/// The journal step a move keeps what the game saw of the room it left in.
pub const LEFT: &str = "left";

/// The journal step a move into a room this game left before keeps what has
/// changed there since.
pub const SINCE: &str = "since";

/// What the card says on a return that found nothing changed.
pub const UNCHANGED: &str = "Nothing you saw here has changed since you were last here.";

/// A thing as the game saw it lying in a room: where in the room it lay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeenThing {
    pub id: i64,
    pub name: String,
    /// The piece it lay on or in, and how; none on the floor.
    pub within: Option<i64>,
    pub how: Option<String>,
}

/// A person as the game saw them in a room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeenPerson {
    pub id: i64,
    pub name: String,
    pub dead: bool,
}

/// A way out as the game saw it: the room it led to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeenWay {
    pub room: i64,
    pub name: String,
}

/// What the game had noticed of a room when it left it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Seen {
    pub room: i64,
    /// Story time when it left.
    pub at: i64,
    pub things: Vec<SeenThing>,
    pub people: Vec<SeenPerson>,
    pub ways_out: Vec<SeenWay>,
}

/// What sort of change one is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A thing seen here is not here any more, and nobody here holds it.
    Gone,
    /// A thing seen here is held by somebody here now.
    Held,
    /// A thing seen here lies somewhere else in the room.
    Moved,
    /// A thing seen here lies broken.
    Broken,
    /// A person seen here alive is not here any more.
    Left,
    /// A person is here alive who was not.
    Came,
    /// A person seen here alive lies dead, or a body lies here that did not.
    Died,
    /// A body seen here is not here any more.
    BodyGone,
    /// A way out seen here is not there any more.
    WayGone,
    /// A way out is here that was not.
    WayNew,
    /// An event of the world touched the room.
    Event,
}

impl Kind {
    pub const ALL: [Kind; 11] = [
        Kind::Gone,
        Kind::Held,
        Kind::Moved,
        Kind::Broken,
        Kind::Left,
        Kind::Came,
        Kind::Died,
        Kind::BodyGone,
        Kind::WayGone,
        Kind::WayNew,
        Kind::Event,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Gone => "gone",
            Kind::Held => "held",
            Kind::Moved => "moved",
            Kind::Broken => "broken",
            Kind::Left => "left",
            Kind::Came => "came",
            Kind::Died => "died",
            Kind::BodyGone => "body_gone",
            Kind::WayGone => "way_gone",
            Kind::WayNew => "way_new",
            Kind::Event => "event",
        }
    }

    pub fn parse(word: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|kind| kind.as_str() == word)
    }
}

/// One change, about one row, in the engine's words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub kind: Kind,
    /// The table and id of the row it is about.
    pub subject: (String, i64),
    pub fact: String,
}

/// What a move back into a room found changed since the game last left it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Since {
    /// The record it was compared with.
    pub seen: Seen,
    /// In the order: things, people, ways out, events. Empty when nothing
    /// the game saw has changed.
    pub changes: Vec<Change>,
}

impl Since {
    /// The card's lines: each change's fact, or [`UNCHANGED`].
    pub fn facts(&self) -> Vec<String> {
        if self.changes.is_empty() {
            return vec![UNCHANGED.to_string()];
        }
        self.changes
            .iter()
            .map(|change| change.fact.clone())
            .collect()
    }
}

// --- the record --------------------------------------------------------------

fn optional(value: Option<i64>) -> Value {
    value.map_or(Value::Null, Value::from)
}

impl Seen {
    /// What `game` has noticed of the room it stands in, now.
    pub fn of(game: &Game) -> Option<Seen> {
        let room = game.current_location()?;
        let things = game
            .items_noticed_in(Some(room))
            .into_iter()
            .map(|item| SeenThing {
                id: id(item),
                name: string(item, "name").to_string(),
                within: int(item, "within_id"),
                how: text(item, "how").map(str::to_string),
            })
            .collect();
        let people = people_in(game, room)
            .into_iter()
            .map(|who| SeenPerson {
                id: id(who),
                name: string(who, "fullname").to_string(),
                dead: dead(game, who),
            })
            .collect();
        Some(Seen {
            room: id(room),
            at: game.story_now(),
            things,
            people,
            ways_out: ways_out(game.records, id(room)),
        })
    }

    pub fn to_json(&self) -> Value {
        json!({
            "room": self.room,
            "at": self.at,
            "things": self.things.iter().map(|thing| json!({
                "id": thing.id,
                "name": thing.name,
                "within": optional(thing.within),
                "how": thing.how,
            })).collect::<Vec<_>>(),
            "people": self.people.iter().map(|who| json!({
                "id": who.id,
                "name": who.name,
                "dead": who.dead,
            })).collect::<Vec<_>>(),
            "ways_out": self.ways_out.iter().map(|way| json!({
                "room": way.room,
                "name": way.name,
            })).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(value: &Value) -> Option<Seen> {
        let list = |key: &str| value[key].as_array().cloned().unwrap_or_default();
        Some(Seen {
            room: value["room"].as_i64()?,
            at: value["at"].as_i64()?,
            things: list("things")
                .iter()
                .map(|thing| {
                    Some(SeenThing {
                        id: thing["id"].as_i64()?,
                        name: thing["name"].as_str()?.to_string(),
                        within: thing["within"].as_i64(),
                        how: thing["how"].as_str().map(str::to_string),
                    })
                })
                .collect::<Option<_>>()?,
            people: list("people")
                .iter()
                .map(|who| {
                    Some(SeenPerson {
                        id: who["id"].as_i64()?,
                        name: who["name"].as_str()?.to_string(),
                        dead: who["dead"].as_bool()?,
                    })
                })
                .collect::<Option<_>>()?,
            ways_out: list("ways_out")
                .iter()
                .map(|way| {
                    Some(SeenWay {
                        room: way["room"].as_i64()?,
                        name: way["name"].as_str()?.to_string(),
                    })
                })
                .collect::<Option<_>>()?,
        })
    }
}

impl Change {
    pub fn to_json(&self) -> Value {
        json!({
            "kind": self.kind.as_str(),
            "subject": [self.subject.0, self.subject.1],
            "fact": self.fact,
        })
    }

    pub fn from_json(value: &Value) -> Option<Change> {
        Some(Change {
            kind: Kind::parse(value["kind"].as_str()?)?,
            subject: (
                value["subject"][0].as_str()?.to_string(),
                value["subject"][1].as_i64()?,
            ),
            fact: value["fact"].as_str()?.to_string(),
        })
    }
}

impl Since {
    pub fn to_json(&self) -> Value {
        json!({
            "seen": self.seen.to_json(),
            "changes": self.changes.iter().map(Change::to_json).collect::<Vec<_>>(),
        })
    }

    pub fn from_json(value: &Value) -> Option<Since> {
        Some(Since {
            seen: Seen::from_json(&value["seen"])?,
            changes: value["changes"]
                .as_array()?
                .iter()
                .map(Change::from_json)
                .collect::<Option<_>>()?,
        })
    }
}

/// A record as a journal step keeps it: its JSON text, or null for none.
pub fn to_journal(value: Option<Value>) -> Value {
    value.map_or(Value::Null, |value| Value::String(value.to_string()))
}

/// The JSON a journal step kept, parsed; none for a step that kept none.
pub fn from_journal(value: &Value) -> Option<Value> {
    serde_json::from_str(value.as_str()?).ok()
}

// --- the rooms -----------------------------------------------------------------

/// The people in a room the game would see there, alive or dead, in id
/// order: never the player, and never anybody walking with the party.
fn people_in<'a>(game: &Game<'a>, room: &Row) -> Vec<&'a Row> {
    let player = game.protagonist().map(id);
    let walking: Vec<i64> = game.followers().iter().map(|who| id(who)).collect();
    let mut people: Vec<&Row> = game
        .characters_located_in(room)
        .into_iter()
        .filter(|who| Some(id(who)) != player && !flag(who, "is_protagonist"))
        .filter(|who| !walking.contains(&id(who)))
        .collect();
    people.sort_by_key(|who| id(who));
    people
}

fn dead(game: &Game, who: &Row) -> bool {
    game.vitals_for(who)
        .is_some_and(|condition| condition.dead())
}

/// The rooms a room's doorways lead to, in the order they were written, each
/// once.
fn ways_out(records: &Records, room: i64) -> Vec<SeenWay> {
    let mut ways: Vec<SeenWay> = Vec::new();
    for edge in records.select("location_connections", |edge| {
        int(edge, "location_id") == Some(room)
    }) {
        let Some(to) = int(edge, "connected_location_id") else {
            continue;
        };
        if ways.iter().any(|way| way.room == to) {
            continue;
        }
        let name = records
            .find("locations", to)
            .map_or("", |far| string(far, "name"));
        ways.push(SeenWay {
            room: to,
            name: name.to_string(),
        });
    }
    ways
}

/// The departure the game's journal kept last for `room`: the latest
/// completed move that walked out of it.
pub fn last_left(records: &Records, playthrough: i64, room: i64) -> Option<Seen> {
    let mut moves: Vec<&Row> = records.select("playthrough_commands", |row| {
        int(row, "playthrough_id") == Some(playthrough)
            && text(row, "status") == Some("completed")
            && row["journal"]["steps"].get("moved").is_some()
    });
    moves.sort_by_key(|row| id(row));
    moves
        .into_iter()
        .rev()
        .filter_map(|row| from_journal(&row["journal"]["steps"][LEFT]))
        .filter_map(|seen| Seen::from_json(&seen))
        .find(|seen| seen.room == room)
}

/// What a move into `room` finds changed since the game last left it, as the
/// records stand now; none when this game has no record of leaving it.
pub fn since(game: &Game, room: &Row) -> Option<Since> {
    let seen = last_left(game.records, game.id(), id(room))?;
    let changes = changes(game, room, &seen);
    Some(Since { seen, changes })
}

/// A sentence about a thing or a person, starting with its name.
fn sentence(name: &str, rest: &str) -> String {
    crate::text::upcase_first(&format!("{name} {rest}"))
}

/// Where in a room a thing lies, in words: on or in a piece, or on the floor.
fn place(records: &Records, within: Option<i64>, how: Option<&str>) -> String {
    let piece = within.and_then(|piece| records.find("items", piece));
    match (piece, how) {
        (Some(piece), Some(how)) => format!(
            "{how} {}",
            crate::moment::definite_name(string(piece, "name"))
        ),
        _ => "on the floor".to_string(),
    }
}

fn changes(game: &Game, room: &Row, seen: &Seen) -> Vec<Change> {
    let records = game.records;
    let here = id(room);
    let mut found = Vec::new();
    let mut add = |kind: Kind, table: &str, row: i64, fact: String| {
        found.push(Change {
            kind,
            subject: (table.to_string(), row),
            fact,
        });
    };
    let people_now = people_in(game, room);
    let present_alive = |who: i64| {
        people_now
            .iter()
            .any(|row| id(row) == who && !dead(game, row))
    };

    for thing in &seen.things {
        let name = crate::moment::definite_name(&thing.name);
        let Some(row) = records.find("items", thing.id) else {
            add(Kind::Gone, "items", thing.id, sentence(&name, "is gone."));
            continue;
        };
        let lies_here = int(row, "location_id") == Some(here) && int(row, "character_id").is_none();
        let holder = int(row, "character_id");
        match text(row, "disposition") {
            Some("intact") => {}
            Some(crate::physics::BROKEN) if lies_here => {
                add(
                    Kind::Broken,
                    "items",
                    thing.id,
                    sentence(&name, "lies broken."),
                );
                continue;
            }
            _ => {
                add(Kind::Gone, "items", thing.id, sentence(&name, "is gone."));
                continue;
            }
        }
        if lies_here {
            let now = (int(row, "within_id"), text(row, "how"));
            if now != (thing.within, thing.how.as_deref()) {
                let then = place(records, thing.within, thing.how.as_deref());
                let now = place(records, now.0, now.1);
                add(
                    Kind::Moved,
                    "items",
                    thing.id,
                    sentence(&name, &format!("is {now} now, not {then}.")),
                );
            }
            continue;
        }
        match holder {
            Some(who) if present_alive(who) => {
                let who = game.character(who);
                add(
                    Kind::Held,
                    "items",
                    thing.id,
                    format!("{} has {name} now.", string(who, "fullname")),
                );
            }
            // In the party's own hands: the player knows where it went.
            None if int(row, "location_id").is_none() => {}
            _ => add(Kind::Gone, "items", thing.id, sentence(&name, "is gone.")),
        }
    }

    for who in &seen.people {
        let now = people_now.iter().find(|row| id(row) == who.id);
        match (who.dead, now) {
            (false, None) => add(
                Kind::Left,
                "characters",
                who.id,
                format!("{} is no longer here.", who.name),
            ),
            (false, Some(row)) if dead(game, row) => add(
                Kind::Died,
                "characters",
                who.id,
                format!("{} lies dead.", who.name),
            ),
            (true, None) => add(
                Kind::BodyGone,
                "characters",
                who.id,
                format!("The body of {} is gone.", who.name),
            ),
            _ => {}
        }
    }
    for row in &people_now {
        if seen.people.iter().any(|who| who.id == id(row)) {
            continue;
        }
        let name = string(row, "fullname");
        if dead(game, row) {
            add(
                Kind::Died,
                "characters",
                id(row),
                format!("{name} lies dead here."),
            );
        } else {
            add(
                Kind::Came,
                "characters",
                id(row),
                format!("{name} is here now, and was not before."),
            );
        }
    }

    let now = ways_out(records, here);
    for way in &seen.ways_out {
        if !now.iter().any(|other| other.room == way.room) {
            add(
                Kind::WayGone,
                "locations",
                way.room,
                format!("The way out to {} is gone.", way.name),
            );
        }
    }
    for way in &now {
        if !seen.ways_out.iter().any(|other| other.room == way.room) {
            add(
                Kind::WayNew,
                "locations",
                way.room,
                format!(
                    "There is a way out to {} that was not there before.",
                    way.name
                ),
            );
        }
    }

    for event in events_since(game, here, seen.at) {
        add(
            Kind::Event,
            "world_events",
            id(event),
            string(event, "summary").to_string(),
        );
    }
    found
}

/// The events of this game's world that touched the room after `after`, in
/// the order they happened: a scheduled one once it fired, any other when it
/// occurred. Never a rearrangement of doorways, which the ways out say.
fn events_since<'a>(game: &Game<'a>, room: i64, after: i64) -> Vec<&'a Row> {
    let records = game.records;
    let touched: Vec<i64> = records
        .select("locations_world_events", |row| {
            int(row, "location_id") == Some(room)
        })
        .iter()
        .filter_map(|row| int(row, "world_event_id"))
        .collect();
    let story = Some(game.story_id());
    let own = game.id();
    let mut events: Vec<(i64, &Row)> = records
        .select("world_events", |event| {
            int(event, "story_id") == story
                && int(event, "playthrough_id").is_none_or(|of| of == own)
                && touched.contains(&id(event))
                && text(event, "source") != Some("world_mechanic")
        })
        .into_iter()
        .filter_map(|event| {
            let at = if int(event, "scheduled_for").is_some() {
                int(event, "fired_at")?
            } else {
                int(event, "occurred_at")?
            };
            (at > after).then_some((at, event))
        })
        .collect();
    events.sort_by_key(|(at, event)| (*at, id(event)));
    events.into_iter().map(|(_, event)| event).collect()
}

/// What the move answered with `scene` found changed since the game last
/// left the room it walked into; none for any other scene, and for a move
/// into a room this game had no record of leaving.
pub fn since_on(records: &Records, playthrough: i64, scene: i64) -> Option<Since> {
    let command = records.first("playthrough_commands", |row| {
        int(row, "playthrough_id") == Some(playthrough)
            && int(row, "result_scene_id") == Some(scene)
    })?;
    Since::from_json(&from_journal(&command["journal"]["steps"][SINCE])?)
}
