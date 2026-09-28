//! `Location::Generator#realize!`: a room nobody has written, written out
//! in two calls, and what the answers are allowed to put in the world.
//!
//! The first call writes the room: its description and lore, the things
//! lying in it and the people standing in it, whom the engine has already
//! drawn (race, age and sex). The second names its ways out. Between the
//! two the answer is kept on the room as a checkpoint, so a room whose
//! second call failed is picked up there the next time somebody walks in,
//! and the first call is never paid for twice.
//!
//! What a model proposes is admitted, never obeyed: `Item::Registry` and
//! `Character::Registry` refuse a thing or a person with no name, one the
//! room or the story has no room for, or one whose name another thing,
//! person or place already has; a way out may reuse a known place or open
//! a new one, but never a new door into a room already written. A building
//! is laid out into rooms as it is written, and walked into at its entry.

use super::{model, Turn};
use crate::cast;
use crate::danger;
use crate::dialogue::sanitize;
use crate::engine::Error;
use crate::interior::{self, Place};
use crate::kind;
use crate::model::{Agent, Book, Call, Failure, Filed};
use crate::parameters::Parameters;
use crate::plan::box_of;
use crate::realization::{self, placeholder_name, Realization, Slot};
use crate::records::{flag, id, int, string, text, Row};
use crate::roll::{self, Seed};
use crate::room::USE_KINDS;
use crate::spot;
use crate::stat_block;
use crate::text::{is_blank, natural_key, ruby_downcase};
use arc::Bound;
use serde_json::{json, Map, Value};

mod arc;

const DETAIL_PENDING: &str = "detail_pending";
const EXITS_PENDING: &str = "exits_pending";

/// `Location::ExitsSchema::MAX_EXITS`.
const MAX_EXITS: i64 = 4;

/// `Item::INSCRIPTION_LIMIT`.
const INSCRIPTION_LIMIT: usize = 400;

/// `Location::RoomName::LIMIT`.
const ROOM_NAME_LIMIT: usize = 60;

/// `Location::Population::BANDS`' words.
const POPULATION_WORDS: [&str; 3] = ["nobody", "a person or two", "a crowd"];

/// `LocationConnection::DISTANCES` and `TRAVEL_METHODS`' words.
const DISTANCES: [&str; 5] = [
    "adjacent",
    "a short walk",
    "across the district",
    "a long journey",
    "days away",
];

/// `Character::Registry::PERSON_LIMITS`, in the order the sheet is read.
const PERSON_LIMITS: [(&str, usize); 12] = [
    ("fullname", 60),
    ("nickname", 30),
    ("appearance", 300),
    ("personality", 300),
    ("backstory", 450),
    ("likes", 160),
    ("dislikes", 160),
    ("fears", 160),
    ("conscious_desire", 180),
    ("unconscious_desire", 180),
    ("recognized_need", 180),
    ("unrecognized_need", 180),
];

/// `Character::Registry::SHEET`.
const SHEET: [&str; 6] = [
    "appearance",
    "personality",
    "backstory",
    "likes",
    "dislikes",
    "fears",
];

/// `Character::Registry::SENTENCE_FIELDS`.
const SENTENCE_FIELDS: [&str; 3] = ["appearance", "personality", "backstory"];

/// `Character::DESIRES`.
const DESIRES: [&str; 4] = [
    "conscious_desire",
    "unconscious_desire",
    "recognized_need",
    "unrecognized_need",
];

/// What sort of place a stub is and how cluttered (`Location::Kind`'s
/// words), when somebody picked them.
#[derive(Clone, Copy, Default)]
struct Words<'w> {
    kind: Option<&'w str>,
    density: Option<&'w str>,
}

/// A field that arrived at its cap: cut off rather than finished
/// (`SanitizesGeneratedText::TruncatedTextError`).
struct Truncated;

/// `sanitize_string(text, max_length:)`.
fn sanitize_capped(text: &str, cap: usize) -> Result<String, Truncated> {
    if text.chars().count() >= cap {
        return Err(Truncated);
    }
    Ok(sanitize(text))
}

/// The end of the last whole sentence (`Character::Registry::SENTENCE_END`):
/// `.`, `!`, `?` or `…`, any closing marks after it, and then a space or
/// the end.
fn complete_sentence_prefix(text: &str) -> Option<String> {
    let clean = sanitize(text);
    let chars: Vec<(usize, char)> = clean.char_indices().collect();
    let mut boundary = None;
    let mut i = 0;
    while i < chars.len() {
        if matches!(chars[i].1, '.' | '!' | '?' | '…') {
            let mut j = i + 1;
            while j < chars.len() && "\"'”’*_)]»›".contains(chars[j].1) {
                j += 1;
            }
            let end = chars.get(j).map_or(clean.len(), |(at, _)| *at);
            let followed = chars.get(j).is_none_or(|(_, c)| c.is_whitespace());
            if followed {
                boundary = Some(end);
                i = j;
                continue;
            }
        }
        i += 1;
    }
    let prefix = clean[..boundary?].trim().to_string();
    (!prefix.is_empty()).then_some(prefix)
}

fn checkpoint(row: &Row) -> Map<String, Value> {
    row.get("generation_checkpoint")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

fn str_of<'v>(value: &'v Value, key: &str) -> &'v str {
    value[key].as_str().unwrap_or_default()
}

impl Turn<'_, '_> {
    fn location(&self, location: i64) -> Result<Row, Error> {
        self.m.row("locations", location)
    }

    fn interior(row: &Row) -> bool {
        int(row, "width").is_some() && int(row, "depth").is_some()
    }

    fn is_place(row: &Row) -> bool {
        Self::interior(row) && box_of(row).is_none()
    }

    fn laid_out(&self, row: &Row) -> bool {
        Self::is_place(row)
            && self
                .m
                .records
                .first("locations", |room| {
                    int(room, "parent_location_id") == Some(id(row))
                })
                .is_some()
    }

    fn interior_room(row: &Row) -> bool {
        box_of(row).is_some() && int(row, "parent_location_id").is_some()
    }

    fn exits_from(&self, location: i64) -> i64 {
        self.m
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id") == Some(location)
            })
            .len() as i64
    }

    fn room_for_exits(&self, location: i64) -> i64 {
        (MAX_EXITS - self.exits_from(location)).max(0)
    }

    fn no_exits_call(&self, row: &Row) -> bool {
        Self::interior_room(row) || self.laid_out(row) || self.room_for_exits(id(row)) == 0
    }

    fn connected(&self, one: i64, other: i64) -> bool {
        self.m
            .records
            .first("location_connections", |edge| {
                let (from, to) = (int(edge, "location_id"), int(edge, "connected_location_id"));
                (from == Some(one) && to == Some(other)) || (from == Some(other) && to == Some(one))
            })
            .is_some()
    }

    fn set_checkpoint(
        &mut self,
        location: i64,
        checkpoint: Option<Map<String, Value>>,
    ) -> Result<(), Error> {
        self.m.update(
            "locations",
            location,
            vec![(
                "generation_checkpoint",
                checkpoint.map_or(Value::Null, Value::Object),
            )],
        )
    }

    /// The people the engine draws for a room (`Character::Registry#slots`),
    /// or the ones its checkpoint kept.
    fn slots(&self, row: &Row) -> Vec<(i64, i64, String)> {
        let kept = checkpoint(row);
        if let Some(Value::Array(slots)) = kept.get("slots") {
            return slots
                .iter()
                .map(|slot| {
                    (
                        slot["race_id"].as_i64().unwrap_or_default(),
                        slot["age"].as_i64().unwrap_or_default(),
                        slot["sex"].as_str().unwrap_or_default().to_string(),
                    )
                })
                .collect();
        }
        let story = self.m.game().story();
        let universe = int(story, "universe_id");
        let races: Vec<&Row> = self
            .m
            .records
            .select("races", |race| int(race, "universe_id") == universe);
        let named: Vec<cast::Race> = races
            .iter()
            .map(|race| cast::Race {
                name: string(race, "name"),
                monstrous: flag(race, "monstrous"),
            })
            .collect();
        let here = Some(id(row));
        let room = cast::Room {
            story_id: self.m.story_id(),
            location_id: id(row),
            name: string(row, "name"),
            danger: text(row, "danger").unwrap_or("safe"),
            population: text(row, "population"),
            present: self
                .m
                .records
                .select("characters", |who| int(who, "location_id") == here)
                .len() as i64,
            in_story: self
                .m
                .records
                .select("characters", |who| {
                    int(who, "story_id") == Some(self.m.story_id())
                })
                .len() as i64,
        };
        cast::slots(&room, &named)
            .into_iter()
            .map(|slot| {
                let race = races
                    .iter()
                    .find(|race| string(race, "name") == slot.race)
                    .map_or(0, |race| id(race));
                (race, slot.age, slot.sex.to_string())
            })
            .collect()
    }

    fn request(
        &self,
        location: i64,
        slots: &[(i64, i64, String)],
        chat: Option<i64>,
    ) -> Result<Value, Error> {
        let row = self
            .m
            .records
            .find("locations", location)
            .ok_or_else(|| Error::Database(format!("locations {location} is not there")))?;
        let slots = slots
            .iter()
            .map(|(race, age, sex)| Slot {
                race: self
                    .m
                    .records
                    .find("races", *race)
                    .map_or(String::new(), |race| string(race, "name").to_string()),
                age: *age,
                sex: sex.clone(),
            })
            .collect();
        let chat = chat.and_then(|chat| self.m.records.find("chats", chat));
        Ok(Realization {
            records: &self.m.records,
            location: row,
            slots,
        }
        .request(chat))
    }

    fn ask_location(&mut self, agent: &mut Agent, request: &Value) -> Result<Value, Error> {
        let call = Call::from_request(request);
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        let answer = self
            .models
            .ask(&mut book, agent, &call, None, None)
            .map_err(model)?;
        Ok(answer.content)
    }

    /// Writes a room out in full, if nobody has, and hands back the agent
    /// that paid for it, for the turn's scene to claim.
    pub(super) fn realize(&mut self, location: i64) -> Result<Option<Agent>, Error> {
        let row = self.location(location)?;
        let pending = !checkpoint(&row).is_empty();
        if text(&row, "detail_level") == Some("realized") && !pending {
            return Ok(None);
        }
        let filed = Filed {
            purpose: "location".into(),
            ..self.filed("location")
        };
        let kept_chat = checkpoint(&row)
            .get("chat_id")
            .and_then(Value::as_i64)
            .filter(|chat| self.m.records.find("chats", *chat).is_some());
        let mut agent = Agent::continuing(filed, kept_chat);
        self.restore_detail(location, &mut agent)?;
        self.write_detail(location, &mut agent)?;
        self.write_exits(location, &mut agent)?;
        Ok(Some(agent))
    }

    /// A conversation picked up from a checkpoint replays the room's written
    /// detail before its exits are asked for, whether or not the chat that
    /// asked for it is still there.
    fn restore_detail(&mut self, location: i64, agent: &mut Agent) -> Result<(), Error> {
        let kept = checkpoint(&self.location(location)?);
        let (Some(prompt), Some(detail)) = (
            kept.get("prompt").and_then(Value::as_str),
            kept.get("detail"),
        ) else {
            return Ok(());
        };
        let has = agent.chat().is_some_and(|chat| {
            self.m
                .records
                .select("messages", |m| {
                    int(m, "chat_id") == Some(chat) && text(m, "role") == Some("assistant")
                })
                .iter()
                .any(|m| {
                    m.get("content_raw").filter(|raw| !raw.is_null()) == Some(detail)
                        || text(m, "content")
                            .and_then(|c| serde_json::from_str::<Value>(c).ok())
                            .as_ref()
                            == Some(detail)
                })
        });
        if has {
            return Ok(());
        }
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        let restored = self.models.restore(
            &mut book,
            agent,
            Some(crate::data::location_generator("system_prompt")),
            prompt,
            detail,
        )?;
        if let Some(chat) = restored {
            let mut kept = kept;
            kept.insert("chat_id".into(), Value::from(chat));
            self.set_checkpoint(location, Some(kept))?;
        }
        Ok(())
    }

    /// `#write_detail_serially!`.
    fn write_detail(&mut self, location: i64, agent: &mut Agent) -> Result<(), Error> {
        let row = self.location(location)?;
        if checkpoint(&row).is_empty() {
            let slots = if Self::is_place(&row) {
                Vec::new()
            } else {
                self.slots(&row)
            };
            let request = self.request(location, &slots, agent.chat())?;
            let detail = self.ask_location(agent, &request)?;
            if is_blank(&sanitize(str_of(&detail, "description")))
                || is_blank(&sanitize(str_of(&detail, "lore")))
            {
                return Err(model(Failure::Rejected(format!(
                    "{} was written with no description or no lore",
                    string(&row, "name")
                ))));
            }
            let mut kept = Map::new();
            kept.insert("phase".into(), Value::from(DETAIL_PENDING));
            kept.insert("detail".into(), detail);
            kept.insert("prompt".into(), request["user"].clone());
            kept.insert(
                "slots".into(),
                slots
                    .iter()
                    .map(|(race, age, sex)| json!({ "race_id": race, "age": age, "sex": sex }))
                    .collect(),
            );
            if let Some(chat) = agent.chat() {
                kept.insert("chat_id".into(), Value::from(chat));
            }
            self.set_checkpoint(location, Some(kept))?;
        }
        let row = self.location(location)?;
        let kept = checkpoint(&row);
        if kept.get("phase").and_then(Value::as_str) == Some(EXITS_PENDING) {
            return Ok(());
        }
        let detail = kept.get("detail").cloned().unwrap_or(Value::Null);
        let slots = self.slots(&row);
        self.m.store.savepoint()?;
        let written = (|| {
            let mut values = vec![
                (
                    "description",
                    Value::from(sanitize(str_of(&detail, "description"))),
                ),
                ("lore", Value::from(sanitize(str_of(&detail, "lore")))),
            ];
            let named = self.room_name(&row, detail.get("name"));
            if let Some(name) = &named {
                values.push(("name", Value::from(name.as_str())));
            }
            self.m.update("locations", location, values)?;
            if named.is_some() {
                self.bind(Bound::Place, location)?;
            }
            if Self::is_place(&row) {
                let building = kind::word(
                    &sanitize(&value_text(&detail["place_kind"])),
                    &kind::buildings(),
                )
                .map(str::to_string);
                self.lay_out_interior(location, detail.get("parameters"), building.as_deref())?;
            }
            let row = self.location(location)?;
            if !self.laid_out(&row) {
                self.admit_items(location, detail.get("items"))?;
                self.admit_people(location, detail.get("people"), &slots)?;
            }
            let mut next = Map::new();
            for key in ["chat_id", "prompt", "detail"] {
                if let Some(value) = kept.get(key) {
                    next.insert(key.into(), value.clone());
                }
            }
            next.insert("phase".into(), Value::from(EXITS_PENDING));
            self.set_checkpoint(location, Some(next))?;
            let row = self.location(location)?;
            if self.no_exits_call(&row) {
                self.finish(location)?;
            }
            Ok(())
        })();
        match written {
            Ok(()) => self.m.store.release(),
            Err(error) => {
                self.m.store.rollback_to();
                Err(error)
            }
        }
    }

    /// `#finish_realization!`.
    fn finish(&mut self, location: i64) -> Result<(), Error> {
        let row = self.location(location)?;
        if checkpoint(&row).get("phase").and_then(Value::as_str) != Some(EXITS_PENDING) {
            return Ok(());
        }
        self.m.update(
            "locations",
            location,
            vec![
                ("detail_level", Value::from("realized")),
                ("generation_checkpoint", Value::Null),
            ],
        )?;
        self.after_realizing()
    }

    /// `#write_exits_serially!`.
    fn write_exits(&mut self, location: i64, agent: &mut Agent) -> Result<(), Error> {
        let row = self.location(location)?;
        if checkpoint(&row).is_empty() {
            return Ok(());
        }
        if self.no_exits_call(&row) {
            return self.finish(location);
        }
        let kept = checkpoint(&row);
        let exits = match kept.get("exits") {
            Some(exits) => exits.clone(),
            None => {
                let request = self.request(location, &[], agent.chat())?;
                let answer = self.ask_location(agent, &request)?;
                let exits = answer
                    .get("exits")
                    .filter(|e| e.is_array())
                    .cloned()
                    .unwrap_or(json!([]));
                if kept.get("phase").and_then(Value::as_str) == Some(EXITS_PENDING) {
                    let mut kept = kept.clone();
                    kept.insert("exits".into(), exits.clone());
                    self.set_checkpoint(location, Some(kept))?;
                }
                exits
            }
        };
        let exits = exits.as_array().cloned().unwrap_or_default();
        self.m.store.savepoint()?;
        let connected = (|| {
            for exit in &exits {
                if self.room_for_exits(location) > 0 {
                    self.connect_exit(location, exit, false)?;
                }
            }
            if self.exits_from(location) == 0 {
                for exit in &exits {
                    self.connect_exit(location, exit, true)?;
                }
            }
            self.finish(location)
        })();
        match connected {
            Ok(()) => self.m.store.release(),
            Err(error) => {
                self.m.store.rollback_to();
                if matches!(&error, Error::Model(Failure::Rejected(_))) {
                    let mut kept = checkpoint(&self.location(location)?);
                    if kept.remove("exits").is_some() {
                        self.set_checkpoint(location, Some(kept))?;
                    }
                }
                Err(error)
            }
        }
    }

    /// `Location::RoomName#accept`: the name a room of a building is given,
    /// if it is one the engine will keep.
    fn room_name(&self, row: &Row, proposed: Option<&Value>) -> Option<String> {
        let place =
            int(row, "parent_location_id").and_then(|p| self.m.records.find("locations", p))?;
        if !placeholder_name(place, string(row, "name")) {
            return None;
        }
        let name = sanitize_capped(proposed?.as_str()?, ROOM_NAME_LIMIT).ok()?;
        if is_blank(&name) || name.contains(',') {
            return None;
        }
        let key = natural_key(&name);
        if key == natural_key(string(row, "name")) || placeholder_name(place, &name) {
            return None;
        }
        let place_key = natural_key(string(place, "name"));
        if !place_key.is_empty() && contains_word(&key, &place_key) {
            return None;
        }
        let story = self.m.story_id();
        let taken = self
            .m
            .records
            .select("locations", |l| {
                int(l, "story_id") == Some(story) && id(l) != id(row)
            })
            .iter()
            .map(|l| natural_key(string(l, "name")))
            .chain(
                self.m
                    .records
                    .select("characters", |c| int(c, "story_id") == Some(story))
                    .iter()
                    .flat_map(|c| {
                        [
                            natural_key(string(c, "fullname")),
                            natural_key(string(c, "nickname")),
                        ]
                    }),
            )
            .chain(
                self.story_items()
                    .iter()
                    .map(|i| natural_key(string(i, "name"))),
            )
            .any(|taken| !taken.is_empty() && taken == key);
        (!taken).then_some(name)
    }

    /// `Item.in_story(story)`: every thing lying in or held in the story's
    /// world, whoever's copy it is.
    fn story_items(&self) -> Vec<Row> {
        let story = Some(self.m.story_id());
        self.m
            .records
            .table("items")
            .iter()
            .filter(|item| {
                let in_room = int(item, "location_id")
                    .and_then(|room| self.m.records.find("locations", room))
                    .is_some_and(|room| int(room, "story_id") == story);
                let held = int(item, "character_id")
                    .and_then(|who| self.m.records.find("characters", who))
                    .is_some_and(|who| int(who, "story_id") == story);
                in_room || held
            })
            .cloned()
            .collect()
    }

    fn templates(&self) -> Vec<Row> {
        self.story_items()
            .into_iter()
            .filter(|item| int(item, "playthrough_id").is_none())
            .collect()
    }

    fn named_by_a_person(&self, name: &str) -> bool {
        let story = Some(self.m.story_id());
        let lowered = ruby_downcase(name);
        self.m
            .records
            .select("characters", |c| int(c, "story_id") == story)
            .iter()
            .any(|c| {
                ruby_downcase(string(c, "fullname")) == lowered
                    || text(c, "nickname").is_some_and(|n| ruby_downcase(n) == lowered)
            })
    }

    fn named_by_a_place(&self, name: &str) -> bool {
        let story = Some(self.m.story_id());
        let lowered = ruby_downcase(name);
        self.m
            .records
            .select("locations", |l| int(l, "story_id") == story)
            .iter()
            .any(|l| ruby_downcase(string(l, "name")) == lowered)
    }

    /// `Location::Placement.in_the_world`.
    fn placement(&self, location: i64, record: spot::Record) -> Vec<(&'static str, Value)> {
        let room = self.m.records.find("locations", location).and_then(box_of);
        match spot::place(self.m.story_id(), room.as_ref(), record, None) {
            Some(at) => vec![("x", Value::from(at.x)), ("y", Value::from(at.y))],
            None => Vec::new(),
        }
    }

    /// `Item::Registry#admit!`.
    fn admit_items(&mut self, location: i64, candidates: Option<&Value>) -> Result<(), Error> {
        let mut created: Vec<String> = Vec::new();
        for attributes in candidates.and_then(Value::as_array).into_iter().flatten() {
            let name = sanitize(&value_text(&attributes["name"]));
            let description = sanitize(&value_text(&attributes["description"]));
            let lying = self
                .m
                .records
                .select("items", |item| {
                    int(item, "location_id") == Some(location)
                        && int(item, "character_id").is_none()
                        && int(item, "playthrough_id").is_none()
                        && text(item, "disposition") == Some("intact")
                })
                .len() as i64;
            let templates = self.templates();
            let mut distinct: Vec<&str> =
                templates.iter().map(|item| string(item, "name")).collect();
            distinct.sort();
            distinct.dedup();
            let lowered = ruby_downcase(&name);
            let refused = is_blank(&name)
                || is_blank(&description)
                || lying >= realization::ITEMS_PER_ROOM
                || distinct.len() as i64 >= realization::ITEMS_PER_STORY
                || created.iter().any(|made| ruby_downcase(made) == lowered)
                || templates
                    .iter()
                    .any(|item| ruby_downcase(string(item, "name")) == lowered)
                || self.named_by_a_person(&name)
                || self.named_by_a_place(&name);
            if refused {
                continue;
            }
            let kind = attributes["use_kind"]
                .as_str()
                .filter(|kind| USE_KINDS.contains(kind))
                .unwrap_or("ordinary");
            let readable = attributes["readable"] == Value::Bool(true);
            let inscription = if readable {
                sanitize_capped(&value_text(&attributes["inscription"]), INSCRIPTION_LIMIT)
                    .ok()
                    .filter(|words| !is_blank(words))
            } else {
                None
            };
            let row = self.m.insert(
                "items",
                vec![
                    ("location_id", Value::from(location)),
                    ("name", Value::from(name.as_str())),
                    ("description", Value::from(description.as_str())),
                    ("use_kind", Value::from(kind)),
                    (
                        "combustible",
                        Value::Bool(attributes["combustible"] == Value::Bool(true)),
                    ),
                    ("readable", Value::Bool(readable)),
                    ("inscription", inscription.map_or(Value::Null, Value::from)),
                ],
            )?;
            let placed = self.placement(location, spot::Record::Item(id(&row)));
            if !placed.is_empty() {
                self.m.update("items", id(&row), placed)?;
            }
            self.bind(Bound::Thing, id(&row))?;
            created.push(name);
        }
        Ok(())
    }

    /// `Character::Registry#admit!`.
    fn admit_people(
        &mut self,
        location: i64,
        candidates: Option<&Value>,
        slots: &[(i64, i64, String)],
    ) -> Result<(), Error> {
        for (slot, candidate) in candidates
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let admitted = match candidate {
                Value::String(name) => self.admit_known(location, name)?,
                Value::Object(_) => self.admit_new(location, candidate, slot, slots)?,
                _ => None,
            };
            if let Some(person) = admitted {
                self.bind(Bound::Person, person)?;
            }
        }
        Ok(())
    }

    fn present_here(&self, location: i64) -> i64 {
        self.m
            .records
            .select("characters", |who| {
                int(who, "location_id") == Some(location)
            })
            .len() as i64
    }

    /// A proposal naming somebody the story already has: they may be here
    /// already, or brought here from nowhere, and nobody is moved.
    fn admit_known(&mut self, location: i64, name: &str) -> Result<Option<i64>, Error> {
        if is_blank(name) {
            return Ok(None);
        }
        let story = Some(self.m.story_id());
        let lowered = ruby_downcase(name);
        let Some(person) = self
            .m
            .records
            .first("characters", |c| {
                int(c, "story_id") == story
                    && (ruby_downcase(string(c, "fullname")) == lowered
                        || text(c, "nickname").is_some_and(|n| ruby_downcase(n) == lowered))
            })
            .cloned()
        else {
            return Ok(None);
        };
        if flag(&person, "deliberately_absent") || int(&person, "location_id").is_some() {
            return Ok(None);
        }
        if flag(&person, "is_protagonist") || flag(&person, "is_companion") {
            return Ok(None);
        }
        if self.present_here(location) >= cast::MAX_PER_ROOM {
            return Ok(None);
        }
        let mut values = vec![
            ("location_id", Value::from(location)),
            ("deliberately_absent", Value::Bool(false)),
        ];
        values.extend(self.placement(location, spot::Record::Character(id(&person))));
        self.m.update("characters", id(&person), values)?;
        Ok(Some(id(&person)))
    }

    fn admit_new(
        &mut self,
        location: i64,
        candidate: &Value,
        slot: usize,
        slots: &[(i64, i64, String)],
    ) -> Result<Option<i64>, Error> {
        let mut fields: Vec<(&str, Option<String>)> = Vec::new();
        for (name, cap) in PERSON_LIMITS {
            let raw = value_text(&candidate[name]);
            let kept = match sanitize_capped(&raw, cap) {
                Ok(text) => Some(text),
                Err(Truncated) if DESIRES.contains(&name) => None,
                Err(Truncated) if SENTENCE_FIELDS.contains(&name) => {
                    match complete_sentence_prefix(&raw) {
                        Some(prefix) => Some(prefix),
                        None => return Ok(None),
                    }
                }
                Err(Truncated) => return Ok(None),
            };
            fields.push((name, kept));
        }
        let field = |name: &str| {
            fields
                .iter()
                .find(|(key, _)| *key == name)
                .and_then(|(_, value)| value.clone())
                .unwrap_or_default()
        };
        let fullname = field("fullname");
        let story = self.m.story_id();
        let universe = int(self.m.game().story(), "universe_id");
        let has_races = self
            .m
            .records
            .first("races", |race| int(race, "universe_id") == universe)
            .is_some();
        let in_story = self
            .m
            .records
            .select("characters", |c| int(c, "story_id") == Some(story))
            .len() as i64;
        let lowered = ruby_downcase(&fullname);
        let refused = is_blank(&fullname)
            || SHEET.iter().any(|name| is_blank(&field(name)))
            || !has_races
            || self.present_here(location) >= cast::MAX_PER_ROOM
            || in_story >= cast::MAX_PER_STORY
            || self.named_by_a_person(&fullname)
            || self.named_by_a_place(&fullname)
            || self
                .story_items()
                .iter()
                .any(|item| ruby_downcase(string(item, "name")) == lowered);
        if refused {
            return Ok(None);
        }
        let Some((race, age, sex)) = slots.get(slot) else {
            return Ok(None);
        };
        let monstrous = self
            .m
            .records
            .find("races", *race)
            .is_some_and(|row| flag(row, "monstrous"));
        let block = stat_block::roll(story, self.m.clock(), slot as i64);
        let mut values: Vec<(&str, Value)> = vec![
            ("story_id", Value::from(story)),
            ("fullname", Value::from(fullname.as_str())),
            (
                "nickname",
                Some(field("nickname"))
                    .filter(|n| !is_blank(n))
                    .map_or(Value::Null, Value::from),
            ),
            ("location_id", Value::from(location)),
            ("race_id", Value::from(*race)),
            ("age", Value::from(*age)),
            ("sex", Value::from(sex.as_str())),
            ("hostile", Value::Bool(monstrous)),
            // `Character`'s own attribute default: nobody is born a companion.
            ("is_companion", Value::Bool(false)),
            ("level", Value::from(block.level)),
            ("hit_die", Value::from(block.hit_die)),
            ("strength", Value::from(block.strength)),
            ("dexterity", Value::from(block.dexterity)),
            ("will", Value::from(block.will)),
        ];
        for name in SHEET {
            values.push((name, Value::from(field(name))));
        }
        for name in DESIRES {
            let value = fields
                .iter()
                .find(|(key, _)| *key == name)
                .and_then(|(_, v)| v.clone());
            values.push((name, value.map_or(Value::Null, Value::from)));
        }
        for name in ["desire_pursuit", "need_pursuit"] {
            let value = sanitize(&value_text(&candidate[name]));
            values.push((
                name,
                if value.is_empty() {
                    Value::Null
                } else {
                    Value::from(value)
                },
            ));
        }
        let person = self.m.insert("characters", values)?;
        let placed = self.placement(location, spot::Record::Character(id(&person)));
        if !placed.is_empty() {
            self.m.update("characters", id(&person), placed)?;
        }
        Ok(Some(id(&person)))
    }

    /// `Location::Generator.create_stub!`: a room with a name and a teaser
    /// and nothing written, its danger rolled, and a footprint when it has
    /// an inside.
    fn create_stub(
        &mut self,
        name: &str,
        teaser: &str,
        inside: Option<&str>,
        population: Option<&str>,
        words: Words,
    ) -> Result<i64, Error> {
        let story = self.m.story_id();
        let rooms = self
            .m
            .records
            .select("locations", |l| int(l, "story_id") == Some(story))
            .len() as i64;
        let danger = danger::for_a_new_room(story, self.m.clock(), rooms);
        let row = self.m.insert(
            "locations",
            vec![
                ("story_id", Value::from(story)),
                ("name", Value::from(name)),
                ("teaser", Value::from(teaser)),
                ("detail_level", Value::from("stub")),
                ("danger", Value::from(danger)),
                ("population", population.map_or(Value::Null, Value::from)),
                ("kind", words.kind.map_or(Value::Null, Value::from)),
                ("density", words.density.map_or(Value::Null, Value::from)),
            ],
        )?;
        let room = id(&row);
        let parameters = Parameters {
            inside: inside.map(str::to_string),
            ..Parameters::default()
        };
        let mut rng = Seed {
            story: story.into(),
            sequence: room.into(),
            kind: roll::FOOTPRINT,
            ..Seed::default()
        }
        .generator();
        if let Some((width, depth)) = parameters.footprint(&mut rng) {
            self.m.update(
                "locations",
                room,
                vec![("width", Value::from(width)), ("depth", Value::from(depth))],
            )?;
        }
        self.bind(Bound::Place, room)?;
        Ok(room)
    }

    /// `#connect!`: one directed doorway, refused whole when its labels are
    /// not words the engine has (`UnusableExitLabel`).
    fn connect(&mut self, from: i64, to: i64, distance: &str, method: &str) -> Result<(), Error> {
        let exists = self
            .m
            .records
            .first("location_connections", |edge| {
                int(edge, "location_id") == Some(from)
                    && int(edge, "connected_location_id") == Some(to)
            })
            .is_some();
        if exists {
            return Ok(());
        }
        if !DISTANCES.contains(&distance)
            || crate::arrival::travel_minutes(distance, method).is_none()
        {
            return Err(model(Failure::Rejected(format!(
                "an exit labelled {distance:?} by {method:?} is not one the engine can walk"
            ))));
        }
        let mut values = vec![
            ("location_id", Value::from(from)),
            ("connected_location_id", Value::from(to)),
            ("distance", Value::from(distance)),
            ("travel_method", Value::from(method)),
        ];
        let time = super::super::travel_time(&values);
        values.push(("time_to_travel", time));
        self.m.insert("location_connections", values)?;
        Ok(())
    }

    /// `#connect_exit!`.
    fn connect_exit(
        &mut self,
        location: i64,
        exit: &Value,
        into_written: bool,
    ) -> Result<(), Error> {
        let name = sanitize(&value_text(&exit["name"]));
        let here = self.location(location)?;
        if is_blank(&name) || natural_key(&name) == natural_key(string(&here, "name")) {
            return Ok(());
        }
        let existing = self.find_location(&name);
        if let Some(other) = &existing {
            let elsewhere = box_of(other).is_some()
                && int(other, "parent_location_id").is_some()
                && int(other, "parent_location_id") != int(&here, "parent_location_id");
            if elsewhere {
                return Ok(());
            }
        }
        let existing = existing.map(|other| self.m.way_in(id(&other)));
        if let Some(other) = existing {
            let row = self.location(other)?;
            if text(&row, "detail_level") == Some("realized")
                && !into_written
                && !self.connected(location, other)
            {
                return Ok(());
            }
        }
        let room_for_door = match existing {
            Some(other) if self.connected(location, other) => true,
            _ => {
                self.room_for_exits(location) > 0
                    && existing.is_none_or(|other| self.exits_from(other) < MAX_EXITS)
            }
        };
        if !room_for_door {
            return Ok(());
        }
        let neighbour = match existing {
            Some(other) => other,
            None => {
                let teaser = sanitize(&value_text(&exit["teaser"]));
                let inside = sanitize(&value_text(&exit["inside"]));
                let population = sanitize(&value_text(&exit["population"]));
                let population = POPULATION_WORDS
                    .contains(&population.as_str())
                    .then_some(population);
                let words = Words {
                    kind: kind::word(&sanitize(&value_text(&exit["kind"])), kind::kinds()),
                    density: kind::word(
                        &sanitize(&value_text(&exit["density"])),
                        kind::densities(),
                    ),
                };
                self.create_stub(
                    &name,
                    &teaser,
                    Some(inside.as_str()),
                    population.as_deref(),
                    words,
                )?
            }
        };
        let distance = sanitize(&value_text(&exit["distance"]));
        let method = sanitize(&value_text(&exit["travel_method"]));
        self.connect(location, neighbour, &distance, &method)?;
        self.connect(neighbour, location, &distance, &method)
    }

    /// `WorldSeed.find_location`: the story's place of this name, exactly
    /// or by its natural key.
    fn find_location(&self, name: &str) -> Option<Row> {
        let story = Some(self.m.story_id());
        let rows = self
            .m
            .records
            .select("locations", |l| int(l, "story_id") == story);
        let lowered = ruby_downcase(name);
        if let Some(exact) = rows
            .iter()
            .find(|l| ruby_downcase(string(l, "name")) == lowered)
        {
            return Some((*exact).clone());
        }
        let key = natural_key(name);
        rows.iter()
            .find(|l| natural_key(string(l, "name")) == key)
            .map(|l| (*l).clone())
    }

    /// `#lay_out_interior!`: the building's rooms and doorways written,
    /// and its ways in moved onto its doorstep rooms.
    fn lay_out_interior(
        &mut self,
        location: i64,
        picks: Option<&Value>,
        building: Option<&str>,
    ) -> Result<(), Error> {
        let row = self.location(location)?;
        if !Self::is_place(&row) || self.laid_out(&row) {
            return Ok(());
        }
        self.lay_out(location, parameters_from(picks).as_ref(), building)
    }

    /// `Location::Interior.lay_out!` and `#open_the_way_in!`: a place's
    /// rooms and doorways, its footprint rolled first when it has none, and
    /// each room dealt its sort from `building`'s.
    fn lay_out(
        &mut self,
        location: i64,
        parameters: Option<&Parameters>,
        building: Option<&str>,
    ) -> Result<(), Error> {
        let row = self.location(location)?;
        let story = self.m.story_id();
        let existing = self
            .m
            .records
            .select("locations", |l| int(l, "story_id") == Some(story))
            .len() as i64;
        let name = string(&row, "name").to_string();
        let layout = interior::lay_out(
            &Place {
                story_id: story,
                id: location,
                name: &name,
                clock: self.m.clock(),
                existing_locations: existing,
                footprint: int(&row, "width").zip(int(&row, "depth")),
                kind: building,
                density: text(&row, "density"),
            },
            None,
            parameters,
        );
        self.m.update(
            "locations",
            location,
            vec![
                ("width", Value::from(layout.footprint.0)),
                ("depth", Value::from(layout.footprint.1)),
            ],
        )?;
        let mut rooms = Vec::new();
        for room in &layout.rooms {
            let bounds = room.bounds;
            let teaser = format!(
                "A room inside {name}, {} by {} paces on storey {}.",
                bounds.width, bounds.depth, bounds.z
            );
            let mut values = vec![
                ("story_id", Value::from(story)),
                ("name", Value::from(room.name.as_str())),
                ("teaser", Value::from(teaser)),
                ("detail_level", Value::from("stub")),
                ("danger", Value::from(room.danger)),
                ("parent_location_id", Value::from(location)),
                ("x", Value::from(bounds.x)),
                ("y", Value::from(bounds.y)),
                ("z", Value::from(bounds.z)),
                ("width", Value::from(bounds.width)),
                ("depth", Value::from(bounds.depth)),
                ("kind", room.kind.map_or(Value::Null, Value::from)),
                (
                    "density",
                    room.density.as_deref().map_or(Value::Null, Value::from),
                ),
            ];
            if let Some(hazard) = &room.hazard {
                values.push(("hazard", Value::from(hazard.hazard)));
                values.push(("hazard_die", Value::from(hazard.hazard_die)));
            }
            let room = id(&self.m.insert("locations", values)?);
            self.bind(Bound::Place, room)?;
            rooms.push(room);
        }
        for edge in &layout.edges {
            let mut values = vec![
                ("location_id", Value::from(rooms[edge.from])),
                ("connected_location_id", Value::from(rooms[edge.to])),
                ("distance", Value::from(edge.distance)),
                ("travel_method", Value::from(edge.travel_method)),
            ];
            let time = super::super::travel_time(&values);
            values.push(("time_to_travel", time));
            self.m.insert("location_connections", values)?;
        }
        self.open_the_way_in(location, &rooms)
    }

    /// `#open_the_way_in!`: every doorway that led to the building now leads
    /// to the first of its doorstep rooms with a doorway to spare.
    fn open_the_way_in(&mut self, location: i64, rooms: &[i64]) -> Result<(), Error> {
        let doorstep: Vec<i64> = rooms
            .iter()
            .enumerate()
            .filter(|(index, room)| {
                *index == 0
                    || self
                        .m
                        .records
                        .find("locations", **room)
                        .and_then(|r| int(r, "z"))
                        == Some(0)
            })
            .map(|(_, room)| *room)
            .collect();
        if doorstep.is_empty() {
            return Ok(());
        }
        let mut ways: Vec<(i64, String, String)> = Vec::new();
        for edge in self.m.records.select("location_connections", |edge| {
            int(edge, "location_id") == Some(location)
                || int(edge, "connected_location_id") == Some(location)
        }) {
            let far = if int(edge, "location_id") == Some(location) {
                int(edge, "connected_location_id")
            } else {
                int(edge, "location_id")
            };
            let Some(far) = far else { continue };
            if !ways.iter().any(|(known, _, _)| *known == far) {
                ways.push((
                    far,
                    string(edge, "distance").to_string(),
                    string(edge, "travel_method").to_string(),
                ));
            }
        }
        for (neighbour, distance, method) in ways {
            let doomed: Vec<i64> = self
                .m
                .records
                .select("location_connections", |edge| {
                    let (from, to) = (int(edge, "location_id"), int(edge, "connected_location_id"));
                    (from == Some(location) && to == Some(neighbour))
                        || (from == Some(neighbour) && to == Some(location))
                })
                .iter()
                .map(|edge| id(edge))
                .collect();
            for edge in doomed {
                self.m.store.delete("location_connections", edge)?;
                self.m.records.remove("location_connections", edge);
            }
            let Some(room) = doorstep
                .iter()
                .copied()
                .find(|room| self.exits_from(*room) < MAX_EXITS)
            else {
                continue;
            };
            self.connect(room, neighbour, &distance, &method)?;
            self.connect(neighbour, room, &distance, &method)?;
        }
        Ok(())
    }
}

/// A value as Ruby's `to_s` writes it: a string itself, null empty.
fn value_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// Whether `word` appears in `text` with no letter or digit either side.
fn contains_word(text: &str, word: &str) -> bool {
    let mut from = 0;
    while let Some(at) = text[from..].find(word) {
        let start = from + at;
        let end = start + word.len();
        let before = text[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        let after = text[end..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
        if !before && !after {
            return true;
        }
        from = start + word.chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// `Location::Parameters.from`: a building's picks as its answer named
/// them, or none when there was no answer to read.
fn parameters_from(picks: Option<&Value>) -> Option<Parameters> {
    let pick = |key: &str| picks.and_then(|p| p[key].as_str()).map(str::to_string);
    picks.filter(|p| p.is_object()).map(|_| Parameters {
        inside: pick("inside"),
        storeys_above: pick("storeys_above"),
        storeys_below: pick("storeys_below"),
        danger: pick("danger"),
        gradient: pick("gradient"),
        hazard: pick("hazard"),
    })
}
