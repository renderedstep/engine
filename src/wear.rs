//! Where on a body a thing is worn (`items.wear`, one of the places in
//! `data/item/wear.yml`), and whether it is worn in this game
//! (`items.worn`), and what the game's doctor reports about the two.
//!
//! WORN IS A KIND OF CARRIED. A worn thing is in somebody's hands as far as
//! every other rule reads it: the party's (a game's copy with no room and no
//! person) or a person's (`character_id`). Nothing reads `worn` but the
//! doctor yet, and no line puts a thing on or takes it off: a world sets
//! both columns by hand, and a kit writes `wear` on what it furnishes
//! (`data/item/kits.yml`). A game's copy of a thing brings both along.
//!
//! ONE THING ON EACH PLACE. A body wears one thing on each place, and the
//! one layering rule (`under`) says what lies beneath what: a coat and a
//! shirt are worn together, on `outer` and on `body`, and are not two
//! things on one place.

use crate::noticed::Finding;
use crate::records::{flag, id, int, string, text, Records, Row};
use std::collections::BTreeMap;

/// The doctor's code for a worn thing nobody holds.
pub const NOT_HELD: &str = "worn_not_held";

/// The doctor's code for two things worn on one place by one body.
pub const PLACE_TAKEN: &str = "worn_place_taken";

/// The doctor's code for a worn thing that names no place it is worn on.
pub const UNWEARABLE: &str = "worn_unwearable";

/// The doctor's code for a `wear` that is no place on a body.
pub const NO_SUCH_PLACE: &str = "wear_no_such_place";

/// Every place on a body, head to foot.
pub fn places() -> &'static [String] {
    &crate::data::wear().places
}

/// Whether a word is a place on a body.
pub fn is_place(place: &str) -> bool {
    places().iter().any(|known| known == place)
}

/// The place above one that lies under another (`body` under `outer`):
/// what is worn there is put on or taken off only while nothing is worn
/// above it.
pub fn under(place: &str) -> Option<&'static str> {
    crate::data::wear()
        .under
        .iter()
        .find(|(below, _)| below == place)
        .map(|(_, above)| above.as_str())
}

/// The place a thing is worn on, or `None` for a thing nobody wears.
pub fn place_of(item: &Row) -> Option<&str> {
    text(item, "wear")
}

/// Whether the row says the thing is worn.
pub fn worn(item: &Row) -> bool {
    flag(item, "worn")
}

/// Who holds a thing: `None` for a thing nobody holds (lying in a room, the
/// world's own row in no person's hands, or no longer whole); for a held
/// thing, its game (`None` on the world's own row) and the person holding it
/// (`None` for the party's hands).
fn holder(item: &Row) -> Option<(Option<i64>, Option<i64>)> {
    if text(item, "disposition") != Some("intact") {
        return None;
    }
    let game = int(item, "playthrough_id");
    match int(item, "character_id") {
        Some(person) => Some((game, Some(person))),
        None if game.is_some() && int(item, "location_id").is_none() => Some((game, None)),
        None => None,
    }
}

/// The story a thing belongs to: its game's, else its room's, else its
/// holder's.
fn story_of(records: &Records, item: &Row) -> Option<i64> {
    [
        ("playthroughs", "playthrough_id"),
        ("locations", "location_id"),
        ("characters", "character_id"),
    ]
    .iter()
    .find_map(|(table, column)| {
        int(item, column)
            .and_then(|other| records.find(table, other))
            .and_then(|other| int(other, "story_id"))
    })
}

fn finding(code: &'static str, message: String, remedy: &'static str, item: &Row) -> Finding {
    Finding {
        code,
        severity: "warning",
        message,
        remedy,
        subject: ("items", id(item)),
    }
}

/// Every worn or wearable record of a story out of line:
///
/// - a `wear` that is no place on a body;
/// - a thing worn that names no place it is worn on;
/// - a thing worn that nobody holds: lying in a room, or no longer whole;
/// - a second thing worn on one place by one body, in one game or in the
///   world itself, after the first by id.
pub fn findings(records: &Records, story: i64) -> Vec<Finding> {
    let mut found = Vec::new();
    let mut on: BTreeMap<(Option<i64>, Option<i64>, &str), &Row> = BTreeMap::new();
    let items = records.select("items", |item| {
        (worn(item) || place_of(item).is_some()) && story_of(records, item) == Some(story)
    });
    for item in items {
        let name = string(item, "name");
        let place = place_of(item);
        if let Some(place) = place.filter(|place| !is_place(place)) {
            found.push(finding(
                NO_SUCH_PLACE,
                format!(
                    "{name:?} names {place:?} as where it is worn, which is no place on a body"
                ),
                "manual",
                item,
            ));
            continue;
        }
        if !worn(item) {
            continue;
        }
        let Some(place) = place else {
            found.push(finding(
                UNWEARABLE,
                format!("{name:?} is worn, but it names no place on a body it is worn on"),
                "safe",
                item,
            ));
            continue;
        };
        let Some((game, person)) = holder(item) else {
            found.push(finding(
                NOT_HELD,
                format!(
                    "{name:?} is worn, but nobody holds it{}",
                    in_game(int(item, "playthrough_id"))
                ),
                "safe",
                item,
            ));
            continue;
        };
        match on.get(&(game, person, place)) {
            Some(first) => found.push(finding(
                PLACE_TAKEN,
                format!(
                    "{name:?} and {:?} are both worn on the {place} by {}{}, and a body wears \
                     one thing on each place",
                    string(first, "name"),
                    body(records, person),
                    in_game(game)
                ),
                "manual",
                item,
            )),
            None => {
                on.insert((game, person, place), item);
            }
        }
    }
    found
}

/// Who holds a thing, in words.
fn body(records: &Records, person: Option<i64>) -> String {
    match person {
        None => "the party".to_string(),
        Some(person) => records
            .find("characters", person)
            .map_or(format!("character #{person}"), |who| {
                string(who, "fullname").to_string()
            }),
    }
}

/// Which game a row is in, in words: nothing for the world's own row.
fn in_game(game: Option<i64>) -> String {
    game.map_or(String::new(), |game| format!(" in playthrough #{game}"))
}
