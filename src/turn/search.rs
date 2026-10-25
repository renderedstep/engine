//! A search, or the opening of a shut piece (`Item::Contents#reveal!` and
//! `Playthrough::PhysicalAction`'s `search` and `open`): what lies inside a
//! fixed piece, found and stamped as found.
//!
//! WHAT IS INSIDE IS DRAWN ONCE, BY THE FIRST GAME TO LOOK. A closed piece
//! holds nothing until somebody searches or opens it, unless its world
//! wrote what is in it, which is then all it holds. The first search in
//! any game draws its inside from the kit's table (`kit::contents`, seeded on
//! the piece's own row, `Roll::CONTENTS`) and writes it into the world, on
//! the piece itself, stamping the world's piece `searched_at` so it is never
//! drawn again; every game then copies it, the first one included, the way a
//! game copies anything lying in a room. A game that has not searched a
//! piece holds no copy of its inside (`Mechanics::snapshot_room` leaves it
//! out), so nothing in it can be named, taken or listed until it does.
//!
//! A SEARCH IS A RECORD OF THE GAME'S. It stamps `searched_at` on this game's
//! copy of the piece, which is what `Room::physical_actions` reads, so a piece is
//! searched once a game. A hollow piece's inside lies in plain view and is
//! already copied: searching it draws nothing. In a game the player narrates
//! the search is the tier that notices what lies inside a piece
//! (`crate::noticed`): everything it finds is stamped noticed.

use super::{Into, Mechanics};
use crate::engine::Error;
use crate::kit;
use crate::records::{id, int, string, text, Row};
use crate::room::Choice;
use serde_json::Value;

impl Mechanics<'_> {
    /// `search` and `open`: the piece's inside drawn if nobody has, copied
    /// into this game, the piece stamped searched, and what it held.
    pub(super) fn search(&mut self, choice: &Choice) -> Result<(&'static str, String), Error> {
        let Some(piece) = &choice.item else {
            return Err(Error::Database("a search with no piece".into()));
        };
        let row = self.row("items", piece.id)?;
        let at = self.story_now();
        if text(&row, "holds") == Some("closed") {
            if let (Some(template), Some(room)) =
                (int(&row, "template_id"), int(&row, "location_id"))
            {
                self.reveal(template, at)?;
                let inside: Vec<(i64, Into)> = self
                    .records
                    .select("items", |item| {
                        int(item, "within_id") == Some(template)
                            && text(item, "how") == Some("in")
                            && int(item, "playthrough_id").is_none()
                            && int(item, "character_id").is_none()
                            && text(item, "disposition") == Some("intact")
                    })
                    .iter()
                    .map(|item| (id(item), Into::Room(room)))
                    .collect();
                self.copy_all(inside)?;
            }
        }
        self.update("items", piece.id, vec![("searched_at", Value::from(at))])?;
        let found = self.found_in(piece.id);
        if self.game().player_narrates() {
            for item in found.iter().filter(|item| !crate::noticed::stamped(item)) {
                self.update("items", id(item), vec![("noticed_at", Value::from(at))])?;
            }
        }
        let verb = if choice.kind == "open" {
            "opened"
        } else {
            "searched"
        };
        let piece = piece.definite_name();
        if found.is_empty() {
            return Ok((
                "applied",
                format!("You {verb} {piece} and found nothing in it."),
            ));
        }
        let names: Vec<String> = found.iter().map(crate::facts::definite_name).collect();
        Ok((
            "applied",
            format!(
                "You {verb} {piece}. In it you found {}, which {} here now; you have not \
                 picked anything up.",
                crate::text::to_sentence(&names),
                if found.len() == 1 { "lies" } else { "lie" }
            ),
        ))
    }

    /// What lies inside this game's piece, whole and in nobody's hands: what
    /// a search of it finds.
    pub(super) fn found_in(&self, piece: i64) -> Vec<Row> {
        self.records
            .select("items", |item| {
                int(item, "within_id") == Some(piece)
                    && text(item, "how") == Some("in")
                    && int(item, "character_id").is_none()
                    && text(item, "disposition") == Some("intact")
            })
            .into_iter()
            .cloned()
            .collect()
    }

    /// `Item::Contents#reveal!`: a closed piece's inside, drawn and written
    /// into the world the first time any game searches it, and the world's
    /// piece stamped so it is never drawn again. A piece the world already
    /// put something inside (a hand-written world's drawer) holds that and
    /// nothing drawn.
    fn reveal(&mut self, template: i64, at: i64) -> Result<(), Error> {
        let fixture = self.row("items", template)?;
        if int(&fixture, "searched_at").is_some() {
            return Ok(());
        }
        let room = int(&fixture, "location_id");
        let used: Vec<String> = self
            .records
            .select("items", |item| {
                int(item, "location_id") == room
                    && room.is_some()
                    && int(item, "playthrough_id").is_none()
                    && text(item, "disposition") == Some("intact")
            })
            .iter()
            .map(|item| string(item, "name").to_string())
            .collect();
        let written = self
            .records
            .first("items", |item| {
                int(item, "within_id") == Some(template)
                    && text(item, "how") == Some("in")
                    && int(item, "playthrough_id").is_none()
            })
            .is_some();
        let name = string(&fixture, "name").to_string();
        let names = if written {
            Vec::new()
        } else {
            kit::contents(&name, self.story_id(), template, &used)
        };
        let prefix = text(&fixture, "kit_key").unwrap_or(&name).to_string();
        for drawn in names {
            let thing = kit::inside(&drawn, &prefix);
            self.insert(
                "items",
                vec![
                    ("location_id", room.map_or(Value::Null, Value::from)),
                    ("name", Value::from(thing.name.as_str())),
                    (
                        "description",
                        Value::from(kit::description(&thing, Some(&name))),
                    ),
                    ("tier", Value::from(thing.tier)),
                    ("bulk", Value::from(thing.bulk.as_str())),
                    ("within_id", Value::from(template)),
                    ("how", thing.how.map_or(Value::Null, Value::from)),
                    ("kit_key", Value::from(thing.kit_key.as_str())),
                    ("use_kind", Value::from(thing.use_kind())),
                    ("combustible", Value::Bool(thing.combustible())),
                    ("readable", Value::Bool(thing.readable())),
                    (
                        "material",
                        thing.material().map_or(Value::Null, Value::from),
                    ),
                ],
            )?;
        }
        self.update("items", template, vec![("searched_at", Value::from(at))])
    }
}
