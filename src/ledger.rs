//! `Playthrough::Ledger`: what one person saw happen in one game, oldest
//! first, bounded by a count and by a budget of characters.
//!
//! Every sentence is a row the engine wrote for its own reasons: their own
//! acts (`playthrough_volitions`), the blows they threw or took
//! (`playthrough_blows`) and what the room they stood in took
//! (`playthrough_tolls`).

use crate::playthrough::Game;
use crate::records::{id, int, string, Row};

/// How many events are looked at at all.
pub const ROWS: usize = 8;

/// How many characters they may spend.
pub const BUDGET: usize = 800;

struct Event {
    at: i64,
    id: i64,
    sentence: String,
}

/// The newest `ROWS` rows of one table that `keep`, newest first.
fn newest<'a>(game: &Game<'a>, table: &str, keep: impl Fn(&Row) -> bool) -> Vec<&'a Row> {
    let mut rows: Vec<&'a Row> = game
        .own(table)
        .into_iter()
        .filter(|row| keep(row))
        .collect();
    rows.reverse();
    rows.truncate(ROWS);
    rows
}

fn event(row: &Row, sentence: String) -> Event {
    Event {
        at: int(row, "created_at").expect("created_at"),
        id: id(row),
        sentence,
    }
}

/// `Ledger.new(playthrough, character).recall(location:)`.
pub fn recall(game: &Game, character: &Row, location: Option<&Row>) -> Vec<String> {
    let who = Some(id(character));
    let mut events: Vec<Event> = Vec::new();
    for act in newest(game, "playthrough_volitions", |row| {
        int(row, "character_id") == who
    }) {
        events.push(event(act, format!("You: {}", string(act, "fact"))));
    }
    let blows = newest(game, "playthrough_blows", |row| {
        int(row, "attacker_id") == who || int(row, "target_id") == who
    });
    for blow in blows {
        let name = |column: &str| {
            string(game.character(int(blow, column).unwrap()), "fullname").to_string()
        };
        let sentence = format!(
            "You experienced this recorded blow: {} struck {} for {} hit points in {}. \
             This is a past event; your condition above is current.",
            name("attacker_id"),
            name("target_id"),
            int(blow, "damage").expect("damage"),
            string(game.location(int(blow, "location_id").unwrap()), "name"),
        );
        events.push(event(blow, sentence));
    }
    if let Some(location) = location {
        let here = Some(id(location));
        for toll in newest(game, "playthrough_tolls", |row| {
            int(row, "location_id") == here
        }) {
            events.push(event(
                toll,
                format!("You saw this happen here: {}", game.toll_to_s(toll)),
            ));
        }
    }
    events.sort_by_key(|event| (event.at, event.id));
    let recent = &events[events.len().saturating_sub(ROWS)..];
    let mut spent = 0;
    let mut kept: Vec<String> = Vec::new();
    for event in recent.iter().rev() {
        let length = event.sentence.chars().count();
        if spent + length > BUDGET {
            break;
        }
        spent += length;
        kept.insert(0, event.sentence.clone());
    }
    kept
}
