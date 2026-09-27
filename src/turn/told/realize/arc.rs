//! The story's arc, as a room's writing meets it: `Quest::Binder` and
//! `Quest::Deadline`.
//!
//! A step of the arc names a place, a person or a thing the world may not
//! hold yet. Binding is a side effect of admission: the moment one of the
//! world's own writers writes a row (a room born, a room of a building
//! named, a thing or a person admitted), every open step waiting for that
//! name takes it. Nothing asks the arc whether a row may exist.
//!
//! Once the story has written more rooms than its grace, a room finishing
//! its writing places the arc's first waiting step itself, off the deepest
//! room the party can already reach: a place laid out two levels down, a
//! person in the deepest room of a place they can reach (a place is built
//! to keep them in when there is none), or a thing on the floor.

use super::{Parameters, Turn};
use crate::cast;
use crate::deadline::{self, Room, GRACE_ROOMS};
use crate::engine::Error;
use crate::records::{id, int, string, text, Row};
use crate::text::{is_blank, natural_key};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The kind of row a writer admitted, and so the kind of step it can bind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::turn::told) enum Bound {
    Place,
    Person,
    Thing,
}

impl Bound {
    fn table(self) -> &'static str {
        match self {
            Bound::Place => "locations",
            Bound::Person => "characters",
            Bound::Thing => "items",
        }
    }

    /// `Quest::Step::TARGET_CLASSES`, read the other way.
    fn trigger(self) -> &'static str {
        match self {
            Bound::Place => "reach_location",
            Bound::Person => "speak_to",
            Bound::Thing => "hold_item",
        }
    }

    fn class(self) -> &'static str {
        match self {
            Bound::Place => "Location",
            Bound::Person => "Character",
            Bound::Thing => "Item",
        }
    }
}

/// `Quest::Deadline::PICKS`: what a placed place is laid out as.
fn picks() -> Parameters {
    Parameters {
        storeys_below: Some("two levels down".to_string()),
        ..Parameters::default()
    }
}

/// `Quest::Deadline#teaser_for`.
fn teaser_for(step: &Row) -> String {
    match text(step, "teaser").filter(|teaser| !is_blank(teaser)) {
        Some(teaser) => teaser.to_string(),
        None => string(step, "summary").to_string(),
    }
}

impl Turn<'_, '_> {
    /// `Quest::Binder.bind!`: every open step of the row's story that was
    /// waiting for its name takes it, at the story's clock. Only a template
    /// binds an arc; a game's own copy of a thing never does.
    pub(super) fn bind(&mut self, kind: Bound, row: i64) -> Result<(), Error> {
        let record = self.m.row(kind.table(), row)?;
        let story = match kind {
            Bound::Thing if int(&record, "playthrough_id").is_some() => None,
            Bound::Thing => int(&record, "location_id")
                .and_then(|room| self.m.records.find("locations", room))
                .and_then(|room| int(room, "story_id"))
                .or_else(|| {
                    int(&record, "character_id")
                        .and_then(|who| self.m.records.find("characters", who))
                        .and_then(|who| int(who, "story_id"))
                }),
            _ => int(&record, "story_id"),
        };
        if story != Some(self.m.story_id()) {
            return Ok(());
        }
        let name = natural_key(string(
            &record,
            if kind == Bound::Person {
                "fullname"
            } else {
                "name"
            },
        ));
        let open: Vec<i64> = self
            .m
            .records
            .select("quests", |quest| {
                int(quest, "story_id") == story && text(quest, "status") == Some("open")
            })
            .iter()
            .map(|quest| id(quest))
            .collect();
        let mut steps: Vec<(i64, i64, i64)> = self
            .m
            .records
            .select("quest_steps", |step| {
                int(step, "target_id").is_none()
                    && int(step, "quest_id").is_some_and(|quest| open.contains(&quest))
                    && text(step, "trigger_kind") == Some(kind.trigger())
                    && natural_key(text(step, "target_name").unwrap_or_default()) == name
            })
            .iter()
            .map(|step| {
                (
                    int(step, "quest_id").unwrap_or_default(),
                    int(step, "position").unwrap_or_default(),
                    id(step),
                )
            })
            .collect();
        if steps.is_empty() {
            return Ok(());
        }
        steps.sort();
        let at = self.m.clock();
        for (_, _, step) in steps {
            self.m.update(
                "quest_steps",
                step,
                vec![
                    ("target_type", Value::from(kind.class())),
                    ("target_id", Value::from(row)),
                    ("bound_at", Value::from(at)),
                ],
            )?;
        }
        Ok(())
    }

    /// `Quest::Deadline.after_realizing!`: once the story is past its grace,
    /// the arc's first step still waiting for a row is placed.
    pub(super) fn after_realizing(&mut self) -> Result<(), Error> {
        let story = Some(self.m.story_id());
        let realized = self
            .m
            .records
            .select("locations", |room| {
                int(room, "story_id") == story && text(room, "detail_level") == Some("realized")
            })
            .len() as i64;
        if realized <= GRACE_ROOMS {
            return Ok(());
        }
        let Some(step) = self.due_step() else {
            return Ok(());
        };
        let (rooms, connections) = self.map_of_the_story();
        let hops = deadline::hops(&rooms, &connections);
        let Some(anchor) = deadline::anchor(&rooms, &connections) else {
            return Ok(());
        };
        match text(&step, "trigger_kind") {
            Some("reach_location") => self.place_a_place(&step, anchor),
            Some("speak_to") => self.place_a_person(&step, anchor, &hops),
            Some("hold_item") => self.place_a_thing(&step, anchor),
            _ => Ok(()),
        }
    }

    /// `#due_step`: the main arc's first step, by position, that wants a
    /// row it has not got. None for a doomed arc.
    fn due_step(&self) -> Option<Row> {
        let story = Some(self.m.story_id());
        let arc = self
            .m
            .records
            .select("quests", |quest| {
                int(quest, "story_id") == story && int(quest, "parent_quest_id").is_none()
            })
            .into_iter()
            .min_by_key(|quest| id(quest))?;
        if text(arc, "status") == Some("doomed") {
            return None;
        }
        let mut steps = self
            .m
            .records
            .select("quest_steps", |step| int(step, "quest_id") == Some(id(arc)));
        steps.sort_by_key(|step| int(step, "position"));
        steps
            .into_iter()
            .find(|step| {
                int(step, "target_id").is_none()
                    && text(step, "trigger_kind") != Some("time_passed")
                    && !is_blank(text(step, "target_name").unwrap_or_default())
            })
            .cloned()
    }

    /// The story's rooms and doorways as `Quest::Deadline`'s walk reads
    /// them: every doorway out of one of the story's rooms.
    fn map_of_the_story(&self) -> (Vec<Room>, Vec<(i64, i64)>) {
        let story = Some(self.m.story_id());
        let rows: Vec<&Row> = self
            .m
            .records
            .select("locations", |room| int(room, "story_id") == story);
        let ids: Vec<i64> = rows.iter().map(|room| id(room)).collect();
        let rooms = rows
            .iter()
            .map(|room| Room {
                id: id(room),
                realized: text(room, "detail_level") == Some("realized"),
                z: int(room, "z"),
                laid_out: self.laid_out(room),
            })
            .collect();
        let connections = self
            .m
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id").is_some_and(|room| ids.contains(&room))
            })
            .iter()
            .filter_map(|edge| {
                Some((
                    int(edge, "location_id")?,
                    int(edge, "connected_location_id")?,
                ))
            })
            .collect();
        (rooms, connections)
    }

    /// `#place_a_place!`.
    fn place_a_place(&mut self, step: &Row, anchor: i64) -> Result<(), Error> {
        let name = string(step, "target_name").to_string();
        let place = self.create_stub(&name, &teaser_for(step), None, None)?;
        self.open_the_door(anchor, place)?;
        self.lay_out(place, Some(&picks()))
    }

    /// `#place_a_person!`: the deepest room of a place the party can reach,
    /// or one built for them, and the person admitted to it from a sheet
    /// the step itself writes.
    fn place_a_person(
        &mut self,
        step: &Row,
        anchor: i64,
        hops: &BTreeMap<i64, i64>,
    ) -> Result<(), Error> {
        let cell = match self.deepest_room_we_can_reach(hops) {
            Some(cell) => cell,
            None => self.somewhere_to_keep_them(step, anchor)?,
        };
        let target = string(step, "target_name").to_string();
        let unwritten = format!(
            "Nobody has written this down yet; the world placed {target} because the story needed them."
        );
        let premise = int(step, "quest_id")
            .and_then(|quest| self.m.records.find("quests", quest))
            .and_then(|quest| text(quest, "premise"))
            .filter(|premise| !is_blank(premise))
            .map(str::to_string);
        let teaser = text(step, "teaser")
            .filter(|teaser| !is_blank(teaser))
            .map(str::to_string);
        let sheet = json!({
            "fullname": target,
            "appearance": teaser.unwrap_or_else(|| unwritten.clone()),
            "personality": unwritten,
            "backstory": premise.unwrap_or_else(|| string(step, "summary").to_string()),
            "likes": unwritten,
            "dislikes": unwritten,
            "fears": unwritten,
        });
        let row = self.location(cell)?;
        let slots = self.slots(&row);
        self.admit_people(cell, Some(&json!([sheet])), &slots)
    }

    /// `#deepest_room_of_a_place_we_can_reach`: a room of a building the
    /// walk reaches with room for somebody more, lowest storey first, then
    /// the latest written.
    fn deepest_room_we_can_reach(&self, hops: &BTreeMap<i64, i64>) -> Option<i64> {
        let story = Some(self.m.story_id());
        self.m
            .records
            .select("locations", |room| {
                int(room, "story_id") == story
                    && hops.contains_key(&id(room))
                    && int(room, "parent_location_id").is_some()
            })
            .into_iter()
            .filter(|room| self.present_here(id(room)) < cast::MAX_PER_ROOM)
            .min_by_key(|room| (int(room, "z").unwrap_or(0), -id(room)))
            .map(id)
    }

    /// `#build_somewhere_to_keep_them!`: a place named for the person, off
    /// the anchor room, laid out, and its deepest room.
    fn somewhere_to_keep_them(&mut self, step: &Row, anchor: i64) -> Result<i64, Error> {
        let name = format!("where {} is", string(step, "target_name"));
        let place = self.create_stub(&name, &teaser_for(step), None, None)?;
        self.open_the_door(anchor, place)?;
        self.lay_out(place, Some(&picks()))?;
        Ok(self
            .m
            .records
            .select("locations", |room| {
                int(room, "parent_location_id") == Some(place)
            })
            .into_iter()
            .min_by_key(|room| (int(room, "z").unwrap_or(0), -id(room)))
            .map_or(place, id))
    }

    /// `#place_a_thing!`: admitted to the anchor room's floor as any
    /// thing a room's writing names is.
    fn place_a_thing(&mut self, step: &Row, anchor: i64) -> Result<(), Error> {
        let name = string(step, "target_name").to_string();
        let description = match text(step, "teaser").filter(|teaser| !is_blank(teaser)) {
            Some(teaser) => teaser.to_string(),
            None => format!("{name}. {}", string(step, "summary")),
        };
        self.admit_items(
            anchor,
            Some(&json!([{ "name": name, "description": description }])),
        )
    }

    /// `#open_the_door!`: a doorway each way between the anchor and the new
    /// place, adjacent and on foot, where there is none already.
    fn open_the_door(&mut self, room: i64, place: i64) -> Result<(), Error> {
        self.connect(room, place, "adjacent", "walking")?;
        self.connect(place, room, "adjacent", "walking")
    }
}
