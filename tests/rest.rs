//! A rest, as a caller holds it: A Place To Rest walked on the live path with
//! replayed replies and by the engine alone, and the rows read back.
//!
//! The Landing and The Shrine are safe rooms the world file names; Brother
//! Amal keeps The Shrine. The Resting Floor is three rooms, its room 3 a safe
//! room when it is laid out, and falls 30 minutes after a game first stands
//! on it. Odile is level 3 on a d8: 18 hit points. The story starts at
//! 8:00 am.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::records::Records;
use renderedstep_engine::turn::{RESTED, WAITED};
use serde_json::{json, Value};
use std::path::Path;

fn world() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-place-to-rest.sql");
    std::fs::read_to_string(path).expect("a world fixture")
}

/// 2026-10-31 08:00:00 UTC: the story's start.
const EIGHT: i64 = 1_793_433_600;

const HOUR: i64 = 60 * 60;

const ON_THE_LANDING: &str =
    "Odile rested for an hour in The Landing and recovered 8 hit points (14 of 18).";

fn reply(value: Value) -> Reply {
    Reply::from_value(&value).unwrap()
}

fn narration() -> Reply {
    reply(json!({"purpose": "narration", "content": "You sit and let the lantern burn."}))
}

fn place() -> Reply {
    reply(json!({"purpose": "location", "content": {
        "description": "Black stone passages.",
        "lore": "Cut by people nobody remembers.",
        "place_kind": "warren",
    }}))
}

fn room() -> Reply {
    reply(json!({"purpose": "location", "content": {
        "description": "A bare stone chamber.",
        "lore": "Old.",
        "items": [],
        "people": [],
    }}))
}

fn safe_room() -> Reply {
    reply(json!({"purpose": "location", "content": {
        "description": "A quiet chamber with a dry floor.",
        "lore": "Nothing is born here.",
        "items": [],
    }}))
}

fn no_arrival() -> Reply {
    reply(json!({"purpose": "arrival", "unavailable": true}))
}

/// `line` submitted on the live path, answered by `replies` in turn: the
/// refusal's words, if it was refused, and every request it sent.
fn submitted(
    engine: &mut Engine,
    playthrough: i64,
    line: &str,
    replies: Vec<Reply>,
) -> (Option<String>, Vec<Value>) {
    let mut replay = Replay::new(replies);
    let token = format!("{line}-{}", count(engine, "playthrough_commands"));
    let submitted = engine
        .submit(playthrough, line, &token, &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    (
        submitted.turned.refusal.map(|refusal| refusal.text()),
        replay.sent().to_vec(),
    )
}

/// A game started in A Place To Rest, Odile unhurt on the landing.
fn started() -> (Engine, i64) {
    let mut engine = open_world(&world()).unwrap();
    let story = engine
        .story_titled(&format!("A Place To Rest{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    (engine, game)
}

/// [`started`], with Odile hurt to 6 of 18.
fn hurt() -> (Engine, i64) {
    let (mut engine, game) = started();
    engine.play(game, "harm 12", &mut |_| {}).unwrap();
    assert_eq!(hp(&engine, game), Some(6));
    (engine, game)
}

fn query(engine: &Engine, sql: &str) -> Option<i64> {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

fn count(engine: &Engine, table: &str) -> i64 {
    query(engine, &format!("SELECT COUNT(*) FROM {table}")).unwrap()
}

fn hp(engine: &Engine, playthrough: i64) -> Option<i64> {
    query(
        engine,
        &format!(
            "SELECT hp_current FROM playthrough_vitals WHERE playthrough_id = {playthrough} \
             AND character_id = (SELECT character_id FROM playthroughs WHERE id = {playthrough})"
        ),
    )
}

fn now(engine: &Engine, playthrough: i64) -> i64 {
    engine.glance(playthrough).unwrap().story_time.unwrap()
}

/// Every rest the game took, as the engine says it.
fn notices(engine: &Engine, playthrough: i64) -> Vec<String> {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    game.rests()
        .into_iter()
        .filter_map(|rest| game.rest_to_s(rest))
        .collect()
}

fn says_nothing_of_the_rest(requests: &[Value]) {
    for request in requests {
        let text = request.to_string();
        for words in ["rested", "recovered", "an hour", "safe room", "sanctuary"] {
            assert!(!text.contains(words), "a request says {words:?}: {text}");
        }
    }
}

#[test]
fn a_wait_in_a_safe_room_with_no_foe_is_a_rest_of_an_hour_that_mends_eight() {
    let (mut engine, game) = hurt();
    let (refusal, requests) = submitted(&mut engine, game, "/wait", vec![narration()]);
    assert_eq!(refusal, None);
    assert_eq!(hp(&engine, game), Some(14));
    assert_eq!(now(&engine, game), EIGHT + HOUR);
    says_nothing_of_the_rest(&requests);

    // One row, claimed by the turn's scene, and the notice said from it.
    assert_eq!(count(&engine, "playthrough_rests"), 1);
    let scene = query(
        &engine,
        &format!("SELECT current_scene_id FROM playthroughs WHERE id = {game}"),
    );
    assert_eq!(
        query(&engine, "SELECT scene_id FROM playthrough_rests"),
        scene,
        "the scene claims its rest"
    );
    assert_eq!(
        query(
            &engine,
            "SELECT recovered * 100 + hp_after FROM playthrough_rests"
        ),
        Some(814)
    );
    assert_eq!(
        query(
            &engine,
            "SELECT CAST(strftime('%s', story_timestamp) AS INTEGER) FROM playthrough_rests"
        ),
        Some(EIGHT),
        "a rest is kept at the time it began"
    );
    assert_eq!(notices(&engine, game), [ON_THE_LANDING]);
}

#[test]
fn a_rest_mends_only_up_to_the_players_maximum() {
    let (mut engine, game) = hurt();
    submitted(&mut engine, game, "/wait", vec![narration()]);
    submitted(&mut engine, game, "/wait", vec![narration()]);
    assert_eq!(hp(&engine, game), Some(18));
    assert_eq!(now(&engine, game), EIGHT + 2 * HOUR);
    submitted(&mut engine, game, "/wait", vec![narration()]);
    assert_eq!(hp(&engine, game), Some(18));
    assert_eq!(
        now(&engine, game),
        EIGHT + 3 * HOUR,
        "a rest at full is still an hour"
    );
    assert_eq!(
        notices(&engine, game),
        [
            ON_THE_LANDING,
            "Odile rested for an hour in The Landing and recovered 4 hit points (18 of 18).",
            "Odile rested for an hour in The Landing and recovered 0 hit points (18 of 18).",
        ]
    );
}

#[test]
fn a_wait_with_a_foe_in_the_safe_room_is_five_minutes_and_mends_nothing() {
    let (mut engine, game) = started();
    engine.play(game, "/move The Shrine", &mut |_| {}).unwrap();
    let report = engine
        .play(game, "/attack Brother Amal", &mut |_| {})
        .unwrap();
    assert_eq!(report.state.foes.len(), 1, "struck, the keeper is a foe");
    let before = hp(&engine, game).unwrap();
    let at = now(&engine, game);
    let (refusal, _) = submitted(&mut engine, game, "/wait", vec![narration()]);
    assert_eq!(refusal, None);
    assert_eq!(now(&engine, game), at + 5 * 60);
    assert!(
        hp(&engine, game).unwrap() <= before,
        "nothing mended: {before} -> {:?}",
        hp(&engine, game)
    );
    assert_eq!(count(&engine, "playthrough_rests"), 0);
}

#[test]
fn a_wait_in_a_room_that_is_not_safe_is_five_minutes_and_mends_nothing() {
    let (mut engine, game) = hurt();
    engine
        .store()
        .connection()
        .execute_batch("UPDATE locations SET sanctuary = FALSE")
        .unwrap();
    submitted(&mut engine, game, "/wait", vec![narration()]);
    assert_eq!(hp(&engine, game), Some(6));
    assert_eq!(now(&engine, game), EIGHT + 5 * 60);
    assert_eq!(count(&engine, "playthrough_rests"), 0);
}

#[test]
fn a_rest_past_the_countdown_is_played_and_fatal() {
    let (mut engine, game) = hurt();
    submitted(
        &mut engine,
        game,
        "/move The Resting Floor",
        vec![place(), room(), no_arrival()],
    );
    submitted(
        &mut engine,
        game,
        "/move The Resting Floor room 2",
        vec![room(), no_arrival()],
    );
    submitted(
        &mut engine,
        game,
        "/move The Resting Floor room 3",
        vec![safe_room(), no_arrival()],
    );
    let (refusal, _) = submitted(&mut engine, game, "/wait", vec![narration()]);
    assert_eq!(refusal, None, "the rest across the countdown is played");
    let glance = engine.glance(game).unwrap();
    assert!(glance.over);
    assert!(glance.collapse.is_some(), "the floor gave way under her");
    assert_eq!(hp(&engine, game), Some(0));
    assert_eq!(count(&engine, "playthrough_rests"), 1, "she rested first");
}

#[test]
fn a_rest_resumed_after_its_effect_spends_its_hour_and_mends_once() {
    let (mut engine, game) = hurt();
    let mut nobody = Replay::new(Vec::new());
    let stopped = engine.submit_stopping(
        game,
        "/wait",
        "w",
        &mut nobody,
        &mut |_| {},
        Some("physical_effect"),
    );
    assert!(stopped.is_err(), "the turn stops after its effect");
    let mut replay = Replay::new(vec![narration()]);
    engine
        .submit(game, "/wait", "w", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(hp(&engine, game), Some(14));
    assert_eq!(now(&engine, game), EIGHT + HOUR);
    assert_eq!(notices(&engine, game), [ON_THE_LANDING]);
}

#[test]
fn the_engine_alone_rests_and_notes_it_in_its_own_words_with_no_time_spent() {
    let (mut engine, game) = hurt();
    let outcome = engine.play(game, "/wait", &mut |_| {}).unwrap();
    assert_eq!(outcome.report.change.as_deref(), Some(RESTED));
    assert_eq!(outcome.report.note, [format!("the game: {ON_THE_LANDING}")]);
    assert_eq!(outcome.state.hp, Some(14));
    assert_eq!(
        now(&engine, game),
        EIGHT,
        "the engine alone spends no time on a wait"
    );

    engine
        .store()
        .connection()
        .execute_batch("UPDATE locations SET sanctuary = FALSE")
        .unwrap();
    let outcome = engine.play(game, "/wait", &mut |_| {}).unwrap();
    assert_eq!(outcome.report.change.as_deref(), Some(WAITED));
    assert!(outcome.report.note.is_empty());
    assert_eq!(outcome.state.hp, Some(14));
}

#[test]
fn a_rest_is_said_against_the_maximum_when_it_began_and_in_the_singular_for_one() {
    let records = Records::from_json(&json!({
        "stories": [{"id": 1}],
        "playthroughs": [{"id": 1, "story_id": 1, "character_id": 1}],
        "characters": [
            {"id": 1, "fullname": "Odile", "level": 3, "hit_die": 8, "is_protagonist": true},
        ],
        "locations": [{"id": 1, "name": "The Landing"}, {"id": 2, "name": "The First Floor room 4"}],
        "playthrough_floors": [
            {"id": 1, "playthrough_id": 1, "location_id": 2, "arrived_at": 1000, "level": 4},
        ],
        "playthrough_rests": [
            {"id": 1, "playthrough_id": 1, "location_id": 1, "recovered": 1,
             "hp_after": 18, "story_timestamp": 900},
            {"id": 2, "playthrough_id": 1, "location_id": 2, "recovered": 8,
             "hp_after": 20, "story_timestamp": 1000},
        ],
    }));
    let game = Game::new(&records, 1);
    let said: Vec<String> = game
        .rests()
        .into_iter()
        .filter_map(|rest| game.rest_to_s(rest))
        .collect();
    assert_eq!(
        said,
        [
            "Odile rested for an hour in The Landing and recovered 1 hit point (18 of 18).",
            "Odile rested for an hour in The First Floor room 4 and recovered 8 hit points (20 of 23).",
        ]
    );
}
