//! `Playthrough::Volition`'s closed set of acts and the System One request
//! built over it (`Volition::State`, `Volition::SystemOne#request`). It builds
//! the request and sends nothing.
//!
//! Beside the acts is the closed set of things a person may say unasked
//! ([`speech_choices`]): greet, warn, ask for, demand and dismiss. Each names
//! a record the engine holds (the player, a hazard, a foe, a thing the player
//! carries) and moves nothing, and its fact says so. The narrator is told the
//! fact and never why, so no speech act carries a desire.

use crate::data;
use crate::dialogue;
use crate::ledger;
use crate::playthrough::Game;
use crate::random::Random;
use crate::records::{flag, id, int, string, text, Row};
use crate::roll;
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

/// What every speech token starts with.
pub const SPEAK: &str = "speak:";

/// The shape of a speech token (`speak:warn:way:12` is `warn`), or none for
/// an act.
pub fn speech_shape(token: &str) -> Option<&str> {
    let rest = token.strip_prefix(SPEAK)?;
    Some(rest.split(':').next().unwrap_or(rest))
}

/// A pursuit's weight per speech shape, or none for a pursuit the speech
/// table has no row for, who never speaks up.
pub fn speech_weights(pursuit: Option<&str>) -> Option<&'static [(String, i64)]> {
    let pursuit = pursuit?;
    data::speech()
        .table
        .iter()
        .find(|(name, _)| name == pursuit)
        .map(|(_, row)| row.as_slice())
}

/// What somebody has said unasked in this game that went through, oldest
/// first.
fn spoken<'a>(game: &Game<'a>, character: &Row) -> Vec<&'a Row> {
    game.own("playthrough_volitions")
        .into_iter()
        .filter(|row| {
            int(row, "character_id") == Some(id(character))
                && text(row, "status") == Some("applied")
                && string(row, "chosen").starts_with(SPEAK)
        })
        .collect()
}

/// Whether somebody who spoke up is still quiet: what they last said is not
/// told yet, or the scene that told it is fewer than `cooldown_turns` scenes
/// back along this game's chain.
pub fn cooling(game: &Game, character: &Row) -> bool {
    let spoken = spoken(game, character);
    let Some(last) = spoken.last() else {
        return false;
    };
    let Some(scene) = int(last, "scene_id") else {
        return true;
    };
    let chain = game.scene_chain();
    chain
        .iter()
        .position(|row| id(row) == scene)
        .is_some_and(|at| ((chain.len() - 1 - at) as i64) < data::speech().cooldown_turns)
}

/// What a person may say unasked to the player: token => the fact the
/// narrator is told, in the order the shapes are offered. Nothing when the
/// game is over, when they or the player are not in `location`, when their
/// pursuit has no speech row, or while they are cooling. Somebody fighting
/// the party may only demand or dismiss.
pub fn speech_choices(game: &Game, character: &Row, location: &Row) -> Vec<(String, String)> {
    let mut offered = Vec::new();
    let Some(player) = game.protagonist() else {
        return offered;
    };
    let present = int(game.row, "ended_at").is_none()
        && id(player) != id(character)
        && !flag(character, "is_protagonist")
        && int(character, "story_id") == Some(game.story_id())
        && int(game.row, "current_location_id") == Some(id(location))
        && game
            .cast_in(Some(location))
            .iter()
            .any(|who| id(who) == id(character));
    if !present
        || speech_weights(text(character, "desire_pursuit")).is_none()
        || cooling(game, character)
    {
        return offered;
    }
    let who = string(character, "fullname");
    let name = string(player, "fullname");
    let here = string(location, "name");
    let spoken = spoken(game, character);
    let said = |token: &str, room: Option<i64>| {
        spoken.iter().any(|row| {
            string(row, "chosen") == token
                && room.is_none_or(|room| int(row, "location_id") == Some(room))
        })
    };
    let mut foes = game.foes_in(Some(location));
    foes.sort_by_key(|foe| id(foe));
    let hostile = foes.iter().any(|foe| id(foe) == id(character));
    let carried = game.carried();
    if !hostile {
        if !said("speak:greet", None) && !dialogue::spoken_with(game, character) {
            offered.push((
                "speak:greet".to_string(),
                format!("{who} spoke up unasked and greeted {name}."),
            ));
        }
        if let Some(hazard) = text(location, "hazard") {
            if !said("speak:warn:here", Some(id(location))) {
                offered.push((
                    "speak:warn:here".to_string(),
                    format!("{who} spoke up unasked and warned {name} that {here} is {hazard}."),
                ));
            }
        }
        let mut ways: Vec<(&Row, &str)> = game
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id") == Some(id(location))
            })
            .into_iter()
            .filter_map(|edge| {
                let hazard = text(edge, "hazard")?;
                Some((game.location(int(edge, "connected_location_id")?), hazard))
            })
            .collect();
        ways.sort_by_key(|(way, _)| id(way));
        for (way, hazard) in ways {
            let token = format!("speak:warn:way:{}", id(way));
            if !said(&token, None) {
                offered.push((
                    token,
                    format!(
                        "{who} spoke up unasked and warned {name} about the {hazard} on the way to {}.",
                        string(way, "name")
                    ),
                ));
            }
        }
        for foe in &foes {
            let token = format!("speak:warn:foe:{}", id(foe));
            if !said(&token, None) {
                offered.push((
                    token,
                    format!(
                        "{who} spoke up unasked and warned {name} to beware of {}.",
                        string(foe, "fullname")
                    ),
                ));
            }
        }
        for item in &carried {
            offered.push((
                format!("speak:ask:{}", id(item)),
                format!(
                    "{who} spoke up unasked and asked {name} for {}. Nothing changed hands.",
                    string(item, "name")
                ),
            ));
        }
    }
    for item in &carried {
        offered.push((
            format!("speak:demand:{}", id(item)),
            format!(
                "{who} spoke up unasked and demanded {} from {name}. Nothing changed hands.",
                string(item, "name")
            ),
        ));
    }
    offered.push((
        "speak:dismiss".to_string(),
        format!("{who} spoke up unasked and told {name} to leave {here}. Nobody moved."),
    ));
    offered
}

/// The speech die: silence or a shape first, from one draw weighted by the
/// person's pursuit over `silent` and every shape with something on offer,
/// then one of that shape's targets, evenly. So what the player carries
/// makes asking no likelier, only what is asked for. None is silence.
pub fn throw_speech(
    game: &Game,
    character: &Row,
    location: &Row,
    silent: i64,
    rng: &mut Random,
) -> Option<String> {
    let row = speech_weights(text(character, "desire_pursuit"))?;
    let offered = speech_choices(game, character, location);
    let of = |shape: &str| {
        offered
            .iter()
            .map(|(token, _)| token)
            .filter(|token| speech_shape(token) == Some(shape))
            .collect::<Vec<_>>()
    };
    let shapes: Vec<&(String, i64)> = row
        .iter()
        .filter(|(shape, _)| !of(shape).is_empty())
        .collect();
    if shapes.is_empty() {
        return None;
    }
    let weights: Vec<i64> = std::iter::once(silent)
        .chain(shapes.iter().map(|(_, weight)| *weight))
        .collect();
    let pick = roll::weighted_one_of(weights.len(), &weights, rng);
    let (shape, _) = shapes.get(pick.checked_sub(1)?)?;
    let targets = of(shape);
    let target = if targets.len() > 1 {
        roll::one_of(targets.len(), rng)
    } else {
        0
    };
    Some(targets[target].clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::Records;
    use crate::roll::Seed;

    const GAME: i64 = 10;
    const WICK: i64 = 100;
    const BELL: i64 = 101;
    const ROOK: i64 = 102;
    const OFFICE: i64 = 20;
    const LANDING: i64 = 21;

    /// A records office with Wick (the player) and Bell in it, a landing
    /// beyond, and whatever `carried` names in Wick's hands.
    fn office(carried: &[&str]) -> Records {
        let items: Vec<Value> = carried
            .iter()
            .enumerate()
            .map(|(at, name)| {
                json!({ "id": 30 + at as i64, "playthrough_id": GAME, "name": name,
                        "disposition": "intact" })
            })
            .collect();
        Records::from_json(&json!({
            "stories": [{ "id": 1, "title": "A Clerk", "start_time": 0 }],
            "playthroughs": [{ "id": GAME, "story_id": 1, "character_id": WICK,
                               "current_location_id": OFFICE, "current_scene_id": 40 }],
            "scenes": [{ "id": 40, "story_id": 1, "story_timestamp": 0 }],
            "characters": [
                { "id": WICK, "story_id": 1, "fullname": "Wick", "is_protagonist": true },
                { "id": BELL, "story_id": 1, "fullname": "Bell", "location_id": OFFICE,
                  "desire_pursuit": "keep" },
            ],
            "locations": [
                { "id": OFFICE, "story_id": 1, "name": "The Records Office" },
                { "id": LANDING, "story_id": 1, "name": "The Landing" },
            ],
            "location_connections": [
                { "id": 1, "location_id": OFFICE, "connected_location_id": LANDING },
                { "id": 2, "location_id": LANDING, "connected_location_id": OFFICE },
            ],
            "items": items,
        }))
    }

    fn offered(records: &Records) -> Vec<String> {
        let game = Game::new(records, GAME);
        speech_choices(&game, game.character(BELL), game.location(OFFICE))
            .into_iter()
            .map(|(token, _)| token)
            .collect()
    }

    fn said(records: &mut Records, id: i64, chosen: &str, scene: Option<i64>) {
        records.push(
            "playthrough_volitions",
            json!({ "id": id, "playthrough_id": GAME, "character_id": BELL,
                    "location_id": OFFICE, "chosen": chosen, "status": "applied",
                    "fact": "Bell spoke up unasked.", "scene_id": scene })
            .as_object()
            .unwrap()
            .clone(),
        );
    }

    /// Scenes after the current one, each the next of the last, and the game
    /// standing on the newest.
    fn scenes_after(records: &mut Records, count: i64) {
        for next in 0..count {
            let id = 41 + next;
            records.push(
                "scenes",
                json!({ "id": id, "story_id": 1, "previous_scene_id": id - 1,
                        "story_timestamp": 300 * (next + 1) })
                .as_object()
                .unwrap()
                .clone(),
            );
            records.set("playthroughs", GAME, "current_scene_id", json!(id));
        }
    }

    #[test]
    fn speech_offers_only_what_the_records_hold() {
        assert_eq!(offered(&office(&[])), ["speak:greet", "speak:dismiss"]);

        let game_records = office(&["quarter receipt", "ward stamp"]);
        let game = Game::new(&game_records, GAME);
        let bell = game.character(BELL);
        let facts: Vec<String> = speech_choices(&game, bell, game.location(OFFICE))
            .into_iter()
            .map(|(_, fact)| fact)
            .collect();
        assert_eq!(
            facts,
            [
                "Bell spoke up unasked and greeted Wick.",
                "Bell spoke up unasked and asked Wick for quarter receipt. Nothing changed hands.",
                "Bell spoke up unasked and asked Wick for ward stamp. Nothing changed hands.",
                "Bell spoke up unasked and demanded quarter receipt from Wick. Nothing changed hands.",
                "Bell spoke up unasked and demanded ward stamp from Wick. Nothing changed hands.",
                "Bell spoke up unasked and told Wick to leave The Records Office. Nobody moved.",
            ],
            "ask and demand list exactly what the player carries"
        );

        let mut records = office(&[]);
        records.set("locations", OFFICE, "hazard", json!("unlit"));
        records.set("location_connections", 1, "hazard", json!("drop"));
        records.push(
            "characters",
            json!({ "id": ROOK, "story_id": 1, "fullname": "Rook", "location_id": OFFICE,
                    "hostile": true })
            .as_object()
            .unwrap()
            .clone(),
        );
        let game = Game::new(&records, GAME);
        let facts: Vec<(String, String)> =
            speech_choices(&game, game.character(BELL), game.location(OFFICE));
        assert_eq!(
            facts,
            [
                ("speak:greet", "Bell spoke up unasked and greeted Wick."),
                (
                    "speak:warn:here",
                    "Bell spoke up unasked and warned Wick that The Records Office is unlit."
                ),
                (
                    "speak:warn:way:21",
                    "Bell spoke up unasked and warned Wick about the drop on the way to The Landing."
                ),
                (
                    "speak:warn:foe:102",
                    "Bell spoke up unasked and warned Wick to beware of Rook."
                ),
                (
                    "speak:dismiss",
                    "Bell spoke up unasked and told Wick to leave The Records Office. Nobody moved."
                ),
            ]
            .map(|(token, fact)| (token.to_string(), fact.to_string())),
            "a warning names a hazard or a foe the records hold"
        );

        let mut records = office(&["quarter receipt"]);
        records.set("characters", BELL, "hostile", json!(true));
        assert_eq!(
            offered(&records),
            ["speak:demand:30", "speak:dismiss"],
            "a foe may only demand or dismiss"
        );

        let mut records = office(&[]);
        records.set("playthroughs", GAME, "current_location_id", json!(LANDING));
        assert!(
            offered(&records).is_empty(),
            "nobody speaks to an empty room"
        );

        let mut records = office(&[]);
        records.set("characters", BELL, "desire_pursuit", Value::Null);
        assert!(offered(&records).is_empty(), "no pursuit, no speech");

        let mut records = office(&[]);
        records.set("playthroughs", GAME, "ended_at", json!(1));
        assert!(
            offered(&records).is_empty(),
            "nobody speaks once the game is over"
        );

        let mut records = office(&[]);
        records.push(
            "chats",
            json!({ "id": 1, "character_id": BELL, "playthrough_id": GAME })
                .as_object()
                .unwrap()
                .clone(),
        );
        records.push(
            "messages",
            json!({ "id": 1, "chat_id": 1, "role": "user", "content": "Hello." })
                .as_object()
                .unwrap()
                .clone(),
        );
        assert_eq!(
            offered(&records),
            ["speak:dismiss"],
            "somebody already spoken with is past greeting"
        );

        let mut records = office(&[]);
        records.set("locations", OFFICE, "hazard", json!("unlit"));
        said(&mut records, 1, "speak:greet", Some(40));
        said(&mut records, 2, "speak:warn:here", Some(40));
        scenes_after(&mut records, 3);
        assert_eq!(
            offered(&records),
            ["speak:dismiss"],
            "a greeting and a warning are said once in a game"
        );
    }

    #[test]
    fn a_speaker_is_quiet_for_the_cooldown_and_offered_again_after() {
        let cooldown = data::speech().cooldown_turns;
        let mut records = office(&[]);
        said(&mut records, 1, "speak:dismiss", None);
        assert!(
            offered(&records).is_empty(),
            "what was said is not told yet"
        );

        records.set("playthrough_volitions", 1, "scene_id", json!(40));
        for scene in 0..cooldown {
            assert!(
                offered(&records).is_empty(),
                "{scene} scene(s) after the one that told it"
            );
            scenes_after_current(&mut records);
        }
        assert_eq!(
            offered(&records),
            ["speak:greet", "speak:dismiss"],
            "{cooldown} scenes on, they may speak again"
        );
    }

    fn scenes_after_current(records: &mut Records) {
        let current = int(
            records.find("playthroughs", GAME).unwrap(),
            "current_scene_id",
        )
        .unwrap();
        let id = current + 1;
        records.push(
            "scenes",
            json!({ "id": id, "story_id": 1, "previous_scene_id": current,
                    "story_timestamp": 300 * (id - 40) })
            .as_object()
            .unwrap()
            .clone(),
        );
        records.set("playthroughs", GAME, "current_scene_id", json!(id));
    }

    fn throw_at(records: &Records, at: i64) -> Option<String> {
        let game = Game::new(records, GAME);
        let mut rng = Seed {
            story: 1,
            playthrough: GAME.into(),
            at: at.into(),
            sequence: BELL.into(),
            kind: roll::SPEECH,
        }
        .generator();
        throw_speech(
            &game,
            game.character(BELL),
            game.location(OFFICE),
            data::speech().silent.turn,
            &mut rng,
        )
    }

    /// The same seed throws the same, and carrying six things picks asking
    /// no more often than carrying one: the shape is drawn before its target.
    #[test]
    fn the_speech_die_is_deterministic_and_shape_first() {
        let mut one = office(&["a"]);
        let mut six = office(&["a", "b", "c", "d", "e", "f"]);
        for records in [&mut one, &mut six] {
            records.set("characters", BELL, "desire_pursuit", json!("obtain"));
        }
        let mut spoke = 0;
        for at in (0..400).map(|turn| turn * 300) {
            let (a, b) = (throw_at(&one, at), throw_at(&six, at));
            assert_eq!(a, throw_at(&one, at), "the same seed throws the same");
            assert_eq!(
                a.as_deref().and_then(speech_shape),
                b.as_deref().and_then(speech_shape),
                "at {at}"
            );
            spoke += usize::from(a.is_some());
        }
        assert!(spoke > 0 && spoke < 400, "{spoke} of 400 spoke");
        let table: Vec<Option<String>> = (0..16).map(|turn| throw_at(&six, turn * 300)).collect();
        let expected = [
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some("speak:demand:31"),
            Some("speak:ask:35"),
            Some("speak:ask:33"),
            None,
            None,
            None,
        ];
        assert_eq!(
            table,
            expected.map(|token| token.map(str::to_string)),
            "the fixed seed table"
        );
    }
}
