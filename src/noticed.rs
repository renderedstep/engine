//! What a game the player narrates has noticed of the rooms it stands in
//! (`items.noticed_at` on this game's copy of each thing, and
//! `playthrough_npc_states.noticed_at` on its state for a person), and the
//! closed sets, panels and card that follow it.
//!
//! THE RECORD IS THE ENGINE'S, AND EVERY TIER IS OPENED BY THE ENGINE:
//!
//! - **On arrival** (the move, and the opening room when the game starts):
//!   the people and the ways out, always, since they react to the player
//!   and are where the player can go; every fixed piece, the room's shape;
//!   and a rolled few loose things, larger bulk first. The count is one die
//!   of [`ON_ARRIVAL`] sides, `Roll::NOTICE`, seeded on the game, story time
//!   and the room.
//! - **A look** (`examine` with no target, [`is_look_line`]): everything
//!   else in plain sight, loose or on a top, up to [`MAX_VISIBLE_PER_ROOM`]
//!   noticed in the room. No die: a look shows all of it.
//! - **Time in the room**: one more thing in plain sight on each played turn
//!   that did not leave the room, picked by `Roll::NOTICE`, seeded on the
//!   game, story time and the submission.
//! - **A return visit**: whatever was noticed stays noticed.
//!
//! THE SEARCH IS NOT HERE. A thing lying *in* a hollow or a closed fixture is
//! [`concealed`]: no tier above notices it. Dense stage 2's `search` and
//! `open` are the tier that will, by stamping what they find; until then a
//! game the player narrates never lists one.
//!
//! WHAT IS NOT NOTICED IS NOT THERE FOR THE PLAYER. The room a line reads
//! against (`turn::room_of`) holds only noticed things in a game the player
//! narrates, so the grammar's candidates, the classifier's lists, the
//! glance's verbs and panels, and the chooser's picks all follow the record.
//! A line naming a thing that is lying here unnoticed is refused in the
//! engine's words ([`refusal_for_line`]), which say what is noticed and
//! suggest a look. Carried things and people are always noticed.
//!
//! A NARRATED GAME IS NEVER STAMPED, and every list in it is whole.

use crate::playthrough::Game;
use crate::records::{id, int, string, text, Records, Row};
use crate::refusal::Refusal;
use crate::room::Record;
use serde_json::Value;

/// `Item::Kit::MAX_VISIBLE_PER_ROOM`: the most one room shows, fixtures and
/// loose things together. A look and time stop at it; the surplus waits for
/// the search (the owner's decision D6 of the dense world).
pub const MAX_VISIBLE_PER_ROOM: usize = 24;

/// The sides of the die that says how many loose things catch the player's
/// eye on the way in.
pub const ON_ARRIVAL: i64 = 3;

/// The line that looks around the room, as the game picks it and a player
/// may type it.
pub const LOOK_LINE: &str = "/look around";

/// The word a glance target for a look carries (`Target.kind`).
pub const LOOK: &str = "look";

/// The journal steps that stamp what a turn noticed, in the order a turn
/// reaches them.
pub const STEPS: [&str; 3] = ["noticed_on_arrival", "looked", "noticed"];

/// The doctor's code for a noticed record out of line with the game.
pub const FINDING: &str = "noticed_record_inconsistent";

/// Said when a line names nothing the player has noticed here.
pub const UNNOTICED: &str = "You have not noticed anything here by that name. \
     Look around the room first: a look shows everything in plain sight.";

/// Whether this game has stamped the row noticed.
pub fn stamped(row: &Row) -> bool {
    int(row, "noticed_at").is_some()
}

/// Whether a thing lies inside a hollow or a closed fixture, where only a
/// search would find it.
pub fn concealed(item: &Row) -> bool {
    int(item, "within_id").is_some() && text(item, "how") == Some("in")
}

/// Where a bulk stands in `data/physics.yml`'s table, lightest first; a bulk
/// the table lacks stands past all of them.
pub fn bulk_rank(bulk: &str) -> usize {
    let table = &crate::data::physics().bulk;
    table
        .iter()
        .position(|(name, _)| name == bulk)
        .unwrap_or(table.len())
}

/// Whether a line looks around the room: `look`, `l` or `look around`, with
/// or without its slash.
pub fn is_look_line(typed: &str) -> bool {
    let words: Vec<String> = crate::grammar::unslashed(typed)
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    matches!(
        words.iter().map(String::as_str).collect::<Vec<_>>()[..],
        ["look"] | ["l"] | ["look", "around"]
    )
}

/// The things lying in plain sight in a room that this game has not
/// noticed, in id order. None in a game the player does not narrate.
pub fn unnoticed_in<'a>(game: &Game<'a>, location: Option<&Row>) -> Vec<&'a Row> {
    if !game.player_narrates() {
        return Vec::new();
    }
    game.items_lying_in(location)
        .into_iter()
        .filter(|item| !stamped(item) && !concealed(item))
        .collect()
}

/// How many more things a room may show this game before it reaches
/// [`MAX_VISIBLE_PER_ROOM`].
pub fn room_left(game: &Game, location: Option<&Row>) -> usize {
    let shown = game
        .items_lying_in(location)
        .into_iter()
        .filter(|item| stamped(item))
        .count();
    MAX_VISIBLE_PER_ROOM.saturating_sub(shown)
}

/// Whether a look would show anything: something in plain sight is
/// unnoticed, and the room has room to show it.
pub fn worth_a_look(game: &Game, location: Option<&Row>) -> bool {
    !unnoticed_in(game, location).is_empty() && room_left(game, location) > 0
}

/// The engine's words for a look: the room, and what it showed.
pub fn look_words(room: &str, seen: &[String]) -> String {
    let found = if seen.is_empty() {
        "You notice nothing new.".to_string()
    } else {
        format!("You notice: {}.", seen.join(", "))
    };
    format!("You look around {room}. {found}")
}

/// The names of things, by id, in the order given.
pub fn names(records: &Records, ids: &[i64]) -> Vec<String> {
    ids.iter()
        .filter_map(|item| records.find("items", *item))
        .map(|item| string(item, "name").to_string())
        .collect()
}

/// The refusal of a line that names a thing lying here which this game has
/// not noticed: in the engine's words, with what is noticed and a look
/// suggested. None in a game the player does not narrate, and for any line
/// that names nothing unnoticed.
pub fn refusal_for_line(records: &Records, playthrough: i64, typed: &str) -> Option<Refusal> {
    let game = Game::new(records, playthrough);
    if !game.player_narrates() {
        return None;
    }
    let room = crate::turn::room_of(records, playthrough);
    let whole = crate::turn::whole_room_of(records, playthrough);
    let noticed: Vec<i64> = room.lying.iter().map(|thing| thing.id).collect();
    let hidden: Vec<i64> = whole
        .lying
        .iter()
        .map(|thing| thing.id)
        .filter(|thing| !noticed.contains(thing))
        .collect();
    if hidden.is_empty() {
        return None;
    }
    let intent = crate::grammar::Grammar::new(&whole).parse(typed).intent?;
    let mut named: Vec<Record> = [intent.subject(), intent.item.clone(), intent.at.clone()]
        .into_iter()
        .flatten()
        .collect();
    if let Some(choice) = &intent.physical {
        named.extend(choice.records());
    }
    let unnoticed = named.iter().any(|record| {
        record
            .thing()
            .is_some_and(|thing| hidden.contains(&thing.id))
    });
    if !unnoticed {
        return None;
    }
    let seen: Vec<String> = room.lying.iter().map(|thing| thing.name.clone()).collect();
    let offer = if seen.is_empty() {
        "You have noticed nothing lying here yet.".to_string()
    } else {
        format!("You have noticed: {}.", seen.join(", "))
    };
    Refusal::new("unresolved", typed, UNNOTICED, Some(offer)).ok()
}

/// The ids a turn's journal says it noticed, step by step: what the turn
/// answered with `scene` stamped. Empty for a scene no submission of this
/// game was answered with.
pub fn noticed_on(records: &Records, playthrough: i64, scene: i64) -> Vec<i64> {
    let Some(command) = records.first("playthrough_commands", |row| {
        int(row, "playthrough_id") == Some(playthrough)
            && int(row, "result_scene_id") == Some(scene)
    }) else {
        return Vec::new();
    };
    let steps = &command["journal"]["steps"];
    STEPS
        .iter()
        .filter_map(|step| steps[*step]["array"].as_array())
        .flatten()
        .filter_map(Value::as_i64)
        .collect()
}

/// One thing the doctor reports about a noticed record (`Story::Doctor`'s
/// `Finding`): its code, how bad, in words, whether clearing it is safe,
/// and the row it is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub code: &'static str,
    pub severity: &'static str,
    pub message: String,
    /// `safe` where the doctor may repair it unasked, `manual` otherwise.
    pub remedy: &'static str,
    /// The table and id of the row it is about.
    pub subject: (&'static str, i64),
}

fn finding(message: String, remedy: &'static str, subject: (&'static str, i64)) -> Finding {
    Finding {
        code: FINDING,
        severity: "warning",
        message,
        remedy,
        subject,
    }
}

/// Every noticed record of a story out of line with its game:
///
/// - a stamp on the world's own row, which no game noticed;
/// - a stamp in a game the player does not narrate;
/// - a stamp later than the game's own clock;
/// - in a game the player narrates, standing still between turns, a fixed
///   piece of the room it stands in, or a thing it carries, left unnoticed,
///   which arrival and every turn notice.
pub fn findings(records: &Records, story: i64) -> Vec<Finding> {
    let mut found = Vec::new();
    let in_story = |table: &str, row: &Row, column: &str| {
        int(row, column)
            .and_then(|other| records.find(table, other))
            .is_some_and(|other| int(other, "story_id") == Some(story))
    };
    for item in records.select("items", |item| {
        stamped(item) && int(item, "playthrough_id").is_none()
    }) {
        if in_story("locations", item, "location_id")
            || in_story("characters", item, "character_id")
        {
            found.push(finding(
                format!(
                    "{:?} is the world's own row and carries a noticed stamp, but only a game's \
                     copy of a thing is ever noticed",
                    string(item, "name")
                ),
                "safe",
                ("items", id(item)),
            ));
        }
    }
    for playthrough in records.select("playthroughs", |row| int(row, "story_id") == Some(story)) {
        let game = Game::new(records, id(playthrough));
        let name = |row: &Row, table: &str| match table {
            "items" => format!("{:?}", string(row, "name")),
            _ => format!(
                "the state of {:?}",
                int(row, "character_id")
                    .and_then(|who| records.find("characters", who))
                    .map_or("", |who| string(who, "fullname"))
            ),
        };
        for table in ["items", "playthrough_npc_states"] {
            for row in game.own(table).into_iter().filter(|row| stamped(row)) {
                if !game.player_narrates() {
                    found.push(finding(
                        format!(
                            "{} carries a noticed stamp in playthrough #{}, which is narrated, \
                             and only a game the player narrates notices anything",
                            name(row, table),
                            game.id()
                        ),
                        "safe",
                        (table, id(row)),
                    ));
                } else if game
                    .story_time()
                    .is_some_and(|now| int(row, "noticed_at").is_some_and(|at| at > now))
                {
                    found.push(finding(
                        format!(
                            "{} was noticed later than playthrough #{}'s own clock",
                            name(row, table),
                            game.id()
                        ),
                        "manual",
                        (table, id(row)),
                    ));
                }
            }
        }
        if !game.player_narrates() || int(game.row, "ended_at").is_some() || mid_turn(&game) {
            continue;
        }
        let here = game.current_location();
        let fixtures = game
            .items_lying_in(here)
            .into_iter()
            .filter(|item| text(item, "tier") == Some(crate::kit::FIXTURE) && !concealed(item));
        for item in fixtures.filter(|item| !stamped(item)) {
            found.push(finding(
                format!(
                    "{:?} is fixed in the room playthrough #{} stands in and was never noticed, \
                     though arriving notices every fixed piece",
                    string(item, "name"),
                    game.id()
                ),
                "manual",
                ("items", id(item)),
            ));
        }
        for item in game.carried().into_iter().filter(|item| !stamped(item)) {
            found.push(finding(
                format!(
                    "{:?} is in playthrough #{}'s hands and was never noticed, though every \
                     turn notices what the player carries",
                    string(item, "name"),
                    game.id()
                ),
                "manual",
                ("items", id(item)),
            ));
        }
    }
    found
}

/// Whether a submission of this game is still owed a finish.
fn mid_turn(game: &Game) -> bool {
    game.own("playthrough_commands").iter().any(|row| {
        text(row, "status") != Some("completed") && text(row, "status") != Some("failed")
    })
}
