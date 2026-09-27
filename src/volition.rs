//! `Playthrough::Volition`'s closed set of acts and the System One request
//! built over it (`Volition::State`, `Volition::SystemOne#request`). It builds
//! the request and sends nothing.

use crate::ledger;
use crate::playthrough::Game;
use crate::records::{flag, id, int, string, text, Row};
use serde_json::{json, Map, Value};

pub const WAIT: &str = "wait";

/// What an act can be said to serve.
pub const SERVES: [&str; 5] = [
    "conscious",
    "unconscious",
    "recognized",
    "unrecognized",
    "none",
];

/// How pressured a person must read before their typed act replaces the die.
pub const PRESSURE_THRESHOLD: f64 = 0.5;

const PRESSURE: &str = "How strongly is this person pressured toward what they will not face?";
const PRESSURE_TRUE: &str = "This person is pressured toward what they will not face.";
const PRESSURE_FALSE: &str = "Nothing here presses this person toward what they will not face.";

/// The desire columns each person is sent with, in order.
pub const DESIRES: [&str; 6] = [
    "conscious_desire",
    "unconscious_desire",
    "recognized_need",
    "unrecognized_need",
    "desire_pursuit",
    "need_pursuit",
];

/// `Item#throwable?`: whether a thing's bulk lets it leave a pair of hands.
pub fn throwable(item: &Row) -> bool {
    matches!(text(item, "bulk"), Some("light" | "handy" | "heavy"))
}

/// The templates the open arcs ask the player to hold, which nobody else
/// may pick up.
fn arc_item_ids(game: &Game) -> Vec<i64> {
    let story = Some(game.story_id());
    let open: Vec<i64> = game
        .records
        .select("quests", |quest| {
            int(quest, "story_id") == story && text(quest, "status") == Some("open")
        })
        .iter()
        .map(|quest| id(quest))
        .collect();
    game.records
        .select("quest_steps", |step| {
            text(step, "trigger_kind") == Some("hold_item")
                && text(step, "target_type") == Some("Item")
                && int(step, "quest_id").is_some_and(|quest| open.contains(&quest))
        })
        .iter()
        .filter_map(|step| int(step, "target_id"))
        .collect()
}

/// `Volition#choices`: token => sentence, in the order they are offered.
pub fn choices(game: &Game, character: &Row, location: &Row) -> Vec<(String, String)> {
    let mut available = vec![(
        WAIT.to_string(),
        "Stay where you are and change nothing.".to_string(),
    )];
    let player = game.protagonist();
    let is_player = player.is_some_and(|who| id(who) == id(character));
    let present = int(game.row, "ended_at").is_none()
        && !is_player
        && int(character, "story_id") == Some(game.story_id())
        && game
            .cast_in(Some(location))
            .iter()
            .any(|who| id(who) == id(character));
    if !present {
        return available;
    }
    let here = string(location, "name");
    let mut exits: Vec<&Row> = game
        .records
        .select("location_connections", |edge| {
            int(edge, "location_id") == Some(id(location))
        })
        .iter()
        .map(|edge| game.location(int(edge, "connected_location_id").unwrap()))
        .collect();
    exits.sort_by_key(|room| id(room));
    for way in exits {
        available.push((
            format!("move:{}", id(way)),
            format!("Walk out of {here} to {}.", string(way, "name")),
        ));
    }
    let arc_items = arc_item_ids(game);
    for item in game.items_lying_in(Some(location)) {
        if !throwable(item) || int(item, "template_id").is_some_and(|t| arc_items.contains(&t)) {
            continue;
        }
        available.push((
            format!("take:{}", id(item)),
            format!("Pick up {} from {here}.", string(item, "name")),
        ));
    }
    let player_present =
        player.is_some() && int(game.row, "current_location_id") == Some(id(location));
    let Some(player) = player.filter(|_| player_present) else {
        return available;
    };
    let player_name = string(player, "fullname");
    for item in game.items_held_by(character) {
        available.push((
            format!("give:{}", id(item)),
            format!("Give {} to {player_name}.", string(item, "name")),
        ));
    }
    let state = game
        .own("playthrough_npc_states")
        .into_iter()
        .find(|row| int(row, "character_id") == Some(id(character)));
    let following = match state {
        Some(row) => flag(row, "following"),
        None => flag(character, "is_companion"),
    };
    if following {
        available.push((
            "stop_following".into(),
            format!("Stay in {here} when the player leaves."),
        ));
    } else {
        available.push((
            "follow".into(),
            format!("Accompany {player_name} when they leave this room."),
        ));
    }
    available
}

fn choice(criteria: Map<String, Value>) -> Value {
    json!({ "type": "choice", "instructions": "Choose one option.", "criteria": criteria })
}

/// `Volition::SystemOne#request`: `{state, questions}` for these people, in
/// the order they are handed in.
pub fn request(game: &Game, characters: &[&Row], location: &Row, line: Option<&str>) -> Value {
    let mut people = Map::new();
    let mut questions = Map::new();
    for (offset, character) in characters.iter().enumerate() {
        let key = format!("person_{}", offset + 1);
        let mut entry = Map::new();
        entry.insert("name".into(), json!(string(character, "fullname")));
        for column in DESIRES {
            entry.insert(column.into(), json!(string(character, column)));
        }
        entry.insert(
            "witnessed".into(),
            json!(ledger::recall(game, character, Some(location))),
        );
        people.insert(key.clone(), Value::Object(entry));

        let acts: Map<String, Value> = choices(game, character, location)
            .into_iter()
            .enumerate()
            .map(|(index, (_, sentence))| (format!("act_{}", index + 1), json!(sentence)))
            .collect();
        questions.insert(format!("{key}:act"), choice(acts));
        let serves: Map<String, Value> = SERVES.iter().map(|s| (s.to_string(), json!(s))).collect();
        questions.insert(format!("{key}:serves"), choice(serves));
        questions.insert(
            format!("{key}:pressure"),
            json!({
                "type": "noul",
                "instructions": PRESSURE,
                "criteria": { "true": PRESSURE_TRUE, "false": PRESSURE_FALSE },
            }),
        );
    }
    json!({
        "state": {
            "location": string(location, "name"),
            "characters": people,
            "player_action": line.unwrap_or(""),
        },
        "questions": questions,
    })
}
