//! Where on a body a thing is worn (`items.wear`, one of the places in
//! `data/item/wear.yml`), and whether it is worn in this game
//! (`items.worn`), and what the game's doctor reports about the two.
//!
//! WORN IS A KIND OF CARRIED. A worn thing is in somebody's hands as far as
//! every other rule reads it: the party's (a game's copy with no room and no
//! person) or a person's (`character_id`). A world sets both columns by
//! hand, and a kit writes `wear` on what it furnishes
//! (`data/item/kits.yml`). A game's copy of a thing brings both along.
//!
//! PUT ON AND TAKEN OFF BY THE PARTY. The `wear` and `remove` attempts
//! (`Room::physical_actions`) set and clear `worn` on a thing the party
//! carries; a `wear` of a thing lying here picks it up in the same act. A
//! worn thing comes off before a hand act lets go of it
//! (`Intent::lets_go_of_the_worn`), and `worn` is cleared wherever a thing
//! leaves a body: laid on a floor (a dead person's things spill there),
//! spent, or handed between the party and a person. No reader is handed
//! either attempt.
//!
//! TOLD FROM THE RECORDS. What the party wears (`Game::worn`) is told to the
//! narrator on a line of its own after what it carries, and to a person
//! spoken to beside what they can see of the player; neither line is there
//! while the party wears nothing. What a person wears is told to nobody yet.
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

/// Where a thing worn on a place sits, as an engine sentence says it: "on
/// your head", "over your clothes".
pub fn on_a_body(place: &str) -> String {
    match place {
        "neck" => "around your neck".into(),
        "outer" => "over your clothes".into(),
        "waist" => "at your waist".into(),
        place => format!("on your {place}"),
    }
}

/// Whether an engine sentence calls a thing "them" rather than "it": a
/// pair is one thing with a plural name ("leather gloves", "boots"), and a
/// name ending in a double s ("dress") is one.
pub fn plural(name: &str) -> bool {
    name.ends_with('s') && !name.ends_with("ss")
}

/// "it" or "them", for a thing by its name.
pub fn it(name: &str) -> &'static str {
    if plural(name) {
        "them"
    } else {
        "it"
    }
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

/// Words that make a name more than one thing, or a thing beside another
/// ("box of gloves", "hat with a feather"): a name holding one is never
/// read as a garment.
const JOINING: &[&str] = &["of", "with", "for", "and", "or", "in", "on", "from"];

/// A word as one of it: "boots" is "boot", "sashes" is "sash", "dress"
/// stays "dress".
fn one_of(word: &str) -> &str {
    for ending in ["sses", "shes", "ches", "xes"] {
        if word.ends_with(ending) {
            return &word[..word.len() - 2];
        }
    }
    match word.strip_suffix('s') {
        Some(stem) if !stem.ends_with('s') && !stem.is_empty() => stem,
        _ => word,
    }
}

/// The place on a body a thing of this name is worn on, read off the kits'
/// `wear` table and nothing else: the whole name as the table lists it, or,
/// for a single-word entry, the name's last word, either one or several of
/// it ("a tattered wool coat" and "gloves" both match; "oil lamp" does not
/// match "helmet lamp"). A name that joins things ("box of gloves") is
/// refused rather than guessed at.
pub fn place_for(name: &str) -> Option<&'static str> {
    let lower = name.trim().to_lowercase();
    let mut words: Vec<&str> = lower.split_whitespace().collect();
    while matches!(
        words.first(),
        Some(&("a" | "an" | "the" | "some" | "pair" | "pairs" | "of"))
    ) {
        words.remove(0);
    }
    if words.is_empty() || words.iter().any(|word| JOINING.contains(word)) {
        return None;
    }
    let table = &crate::data::kits().wear;
    let whole = words.join(" ");
    let last = *words.last()?;
    let listed = |wanted: &str| {
        table
            .iter()
            .find(|(name, _)| name == wanted)
            .map(|(_, place)| place.as_str())
    };
    let single = |wanted: &str| {
        table
            .iter()
            .find(|(name, _)| !name.contains(' ') && name == wanted)
            .map(|(_, place)| place.as_str())
    };
    let plural_whole = words[..words.len() - 1]
        .iter()
        .copied()
        .chain([one_of(last)])
        .collect::<Vec<_>>()
        .join(" ");
    listed(&whole)
        .or_else(|| listed(&plural_whole))
        .or_else(|| single(last))
        .or_else(|| single(one_of(last)))
        .filter(|place| is_place(place))
}

/// A thing of a story with no place on a body that [`place_for`] gives one,
/// by id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wearable {
    pub item: i64,
    pub name: String,
    pub place: &'static str,
}

/// Every thing of `story`, in the world and in each of its games, that
/// names no place it is worn on and that [`place_for`] reads as a garment,
/// in id order. Only a thing light or handy enough to carry is one.
pub fn unmarked(records: &Records, story: i64) -> Vec<Wearable> {
    let mut found: Vec<Wearable> = records
        .select("items", |item| {
            place_of(item).is_none() && story_of(records, item) == Some(story)
        })
        .into_iter()
        .filter(|item| {
            matches!(text(item, "bulk").unwrap_or("handy"), "light" | "handy")
                && text(item, "tier") != Some(crate::kit::FIXTURE)
        })
        .filter_map(|item| {
            let name = string(item, "name");
            place_for(name).map(|place| Wearable {
                item: id(item),
                name: name.to_string(),
                place,
            })
        })
        .collect();
    found.sort_by_key(|wearable| wearable.item);
    found
}
