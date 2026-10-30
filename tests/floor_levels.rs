//! A floor that raises a level, as a caller holds it: Three Floors That
//! Raise You walked into on the live path with a scripted model and by the
//! engine alone, and the rows read back.
//!
//! Each of its three floors is two rooms and says `levels_up`. Odile is
//! level 3 on a d8 in the world: 18 hit points.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Live, Route, Secret};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::records::Records;
use serde_json::{json, Value};
use std::path::Path;

fn world() -> String {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/three-floors-that-raise-you.sql");
    std::fs::read_to_string(path).expect("a world fixture")
}

const TITLE: &str = "Three Floors That Raise You";

struct Scripted {
    answers: Vec<(u16, Value)>,
    sent: Vec<Value>,
}

impl renderedstep_engine::model::http::Transport for Scripted {
    fn post(
        &mut self,
        _endpoint: &renderedstep_engine::model::route::Endpoint,
        body: &Value,
        _options: &renderedstep_engine::model::http::Options,
        _on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<renderedstep_engine::model::http::Posted, renderedstep_engine::model::http::Unreached>
    {
        self.sent.push(body.clone());
        let (status, answer) = self.answers.remove(0);
        Ok(renderedstep_engine::model::http::Posted {
            status,
            body: answer.to_string(),
        })
    }
}

fn answered(content: Value) -> (u16, Value) {
    (
        200,
        json!({"choices": [{"message": {"content": content.to_string()}, "finish_reason": "stop"}]}),
    )
}

fn place() -> Value {
    json!({
        "description": "Black stone passages lit by nothing you can see.",
        "lore": "Cut by people nobody remembers.",
        "place_kind": "warren",
    })
}

fn room() -> Value {
    json!({
        "description": "A bare stone chamber.",
        "lore": "Old.",
        "items": [],
        "people": [],
    })
}

fn arrival() -> Value {
    json!({"description": "You arrive.", "summary": "Arrived."})
}

/// `line` submitted on the live path, the model answering `answers` in
/// turn; every request it was sent.
fn submitted(engine: &mut Engine, playthrough: i64, line: &str, answers: Vec<Value>) -> Vec<Value> {
    let transport = Scripted {
        answers: answers.into_iter().map(answered).collect(),
        sent: Vec::new(),
    };
    let route = Route::Direct {
        key: Secret::new("own"),
    };
    let mut live = Live::with_transport(route, transport);
    engine
        .submit(playthrough, line, line, &mut live, &mut |_| {})
        .unwrap();
    live.transport().sent.clone()
}

/// `line` played by the engine alone: what it says it did, and its notes.
fn played(engine: &mut Engine, playthrough: i64, line: &str) -> (String, Vec<String>) {
    let report = engine.play(playthrough, line, &mut |_| {}).unwrap().report;
    assert_eq!(report.refusal, None, "{line} was refused");
    (report.change.unwrap_or_default(), report.note)
}

fn opened() -> (Engine, i64) {
    let engine = open_world(&world()).unwrap();
    let story = engine
        .story_titled(&format!("{TITLE}{TITLE_SUFFIX}"))
        .unwrap();
    (engine, story)
}

fn int(engine: &Engine, sql: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

fn execute(engine: &Engine, sql: &str) {
    engine.store().connection().execute_batch(sql).unwrap();
}

fn location(engine: &Engine, name: &str) -> i64 {
    int(
        engine,
        &format!("SELECT id FROM locations WHERE name = '{name}'"),
    )
}

/// The floors a game has reached, in order: the floor, the level it raised
/// the player to (`None` for none), and whether a scene claimed it.
fn reached(engine: &Engine, playthrough: i64) -> Vec<(i64, Option<i64>, bool)> {
    let connection = engine.store().connection();
    let mut statement = connection
        .prepare(
            "SELECT location_id, level, scene_id IS NOT NULL FROM playthrough_floors \
             WHERE playthrough_id = ?1 ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([playthrough], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

/// The player's hit points in this game's own row, none before it has one.
fn hp(engine: &Engine, playthrough: i64) -> Option<i64> {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT hp_current FROM playthrough_vitals WHERE playthrough_id = ?1 \
             AND character_id = (SELECT character_id FROM playthroughs WHERE id = ?1)",
            [playthrough],
            |row| row.get(0),
        )
        .ok()
}

/// The first line of the game's stat sheet, as a front end shows it.
fn sheet(engine: &Engine, playthrough: i64) -> String {
    engine.glance(playthrough).unwrap().sheet[0].clone()
}

#[test]
fn reaching_a_floor_raises_the_games_level_once_and_heals_the_player_to_full() {
    let (mut engine, story) = opened();
    let (first, second) = (
        location(&engine, "The First Floor"),
        location(&engine, "The Second Floor"),
    );
    let game = engine.start(story).unwrap();
    let (change, _) = played(&mut engine, game, "harm 5");
    assert_eq!(change, "harmed: Odile unhurt -> hurt (13 of 18)");

    submitted(
        &mut engine,
        game,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(reached(&engine, game), [(first, Some(4), true)]);
    assert_eq!(hp(&engine, game), Some(23), "healed to level 4's full");
    assert_eq!(sheet(&engine, game), "level 4, d8");
    assert_eq!(
        int(
            &engine,
            "SELECT level FROM characters WHERE is_protagonist = 1"
        ),
        3,
        "the world's row is never raised"
    );
    let (change, _) = played(&mut engine, game, "harm 3");
    assert_eq!(change, "harmed: Odile unhurt -> hurt (20 of 23)");

    // Another room of a floor already reached raises nothing.
    submitted(
        &mut engine,
        game,
        "/move The First Floor room 2",
        vec![room(), arrival()],
    );
    assert_eq!(reached(&engine, game), [(first, Some(4), true)]);
    assert_eq!(hp(&engine, game), Some(20));

    submitted(
        &mut engine,
        game,
        "/move The Second Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(
        reached(&engine, game),
        [(first, Some(4), true), (second, Some(5), true)]
    );
    assert_eq!(hp(&engine, game), Some(28));

    // Back up and down again: neither floor is reached a second time.
    played(&mut engine, game, "harm 4");
    let (_, notes) = played(&mut engine, game, "/move The First Floor room 2");
    assert!(notes.is_empty(), "{notes:?}");
    played(&mut engine, game, "/move The Second Floor room 1");
    assert_eq!(reached(&engine, game).len(), 2);
    assert_eq!(hp(&engine, game), Some(24));
    assert_eq!(sheet(&engine, game), "level 5, d8");
    let (change, _) = played(&mut engine, game, "mend 99");
    assert_eq!(change, "mended: Odile hurt (24 of 28) -> unhurt");
}

#[test]
fn the_engine_says_what_a_floor_raised_and_no_model_is_told() {
    let (mut engine, story) = opened();
    let first = engine.start(story).unwrap();
    let sent = submitted(
        &mut engine,
        first,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    for request in &sent {
        let text = request.to_string();
        assert!(!text.contains("level 4"), "{text}");
        assert!(!text.contains("healed"), "{text}");
    }

    // A second game reaches the floor laid out, with no model at all.
    let second = engine.start(story).unwrap();
    let (_, notes) = played(&mut engine, second, "/move The First Floor");
    assert_eq!(
        notes,
        [
            "the game: Reaching The First Floor raised Odile to level 4: 23 hit points, and \
          healed to full."
        ]
    );
    let (_, notes) = played(&mut engine, second, "/move The First Floor room 2");
    assert!(notes.is_empty(), "{notes:?}");
}

#[test]
fn the_level_is_the_games_and_every_game_starts_from_the_worlds() {
    let (mut engine, story) = opened();
    let first = engine.start(story).unwrap();
    submitted(
        &mut engine,
        first,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    submitted(
        &mut engine,
        first,
        "/move The First Floor room 2",
        vec![room(), arrival()],
    );
    submitted(
        &mut engine,
        first,
        "/move The Second Floor",
        vec![place(), room(), arrival()],
    );
    let second = engine.start(story).unwrap();
    played(&mut engine, second, "/move The First Floor");
    assert_eq!(sheet(&engine, first), "level 5, d8");
    assert_eq!(sheet(&engine, second), "level 4, d8");
    assert_eq!(hp(&engine, second), Some(23));
}

#[test]
fn a_floor_that_raises_nothing_still_records_the_arrival_and_heals_nobody() {
    let (mut engine, story) = opened();
    let first = location(&engine, "The First Floor");
    execute(&engine, "UPDATE floors SET levels_up = 0;");
    let game = engine.start(story).unwrap();
    played(&mut engine, game, "harm 5");
    submitted(
        &mut engine,
        game,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(reached(&engine, game), [(first, None, true)]);
    assert_eq!(hp(&engine, game), Some(13));
    assert_eq!(sheet(&engine, game), "level 3, d8");
}

#[test]
fn no_floor_raises_a_body_past_the_highest_level() {
    let (mut engine, story) = opened();
    let first = location(&engine, "The First Floor");
    execute(
        &engine,
        "UPDATE characters SET level = 20 WHERE is_protagonist = 1;",
    );
    let game = engine.start(story).unwrap();
    submitted(
        &mut engine,
        game,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(reached(&engine, game), [(first, Some(20), true)]);
    assert_eq!(hp(&engine, game), Some(8 + 19 * 5));
}

#[test]
fn the_way_onto_a_floor_takes_its_toll_after_the_floor_has_healed_the_player() {
    let (mut engine, story) = opened();
    let (landing, first) = (
        location(&engine, "The Landing"),
        location(&engine, "The First Floor"),
    );
    execute(
        &engine,
        &format!(
            "UPDATE location_connections SET hazard = 'undertow', hazard_die = 6 \
             WHERE location_id = {landing} AND connected_location_id = {first};"
        ),
    );
    let game = engine.start(story).unwrap();
    played(&mut engine, game, "harm 5");
    submitted(
        &mut engine,
        game,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    let damage = int(
        &engine,
        &format!("SELECT damage FROM playthrough_tolls WHERE playthrough_id = {game}"),
    );
    assert!(damage > 0, "the toll was saved, so it shows nothing");
    assert_eq!(hp(&engine, game), Some(23 - damage));
    assert_eq!(
        int(
            &engine,
            &format!("SELECT hp_after FROM playthrough_tolls WHERE playthrough_id = {game}"),
        ),
        23 - damage,
        "paid out of level 4's full 23"
    );
}

#[test]
fn a_database_from_before_the_levels_plays_and_raises_nobody() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world()).unwrap();
    conn.execute_batch(
        r#"DROP TABLE "playthrough_floors";
           ALTER TABLE "floors" DROP COLUMN "levels_up";
           DELETE FROM schema_migrations WHERE version = '20261031090000';"#,
    )
    .unwrap();
    let mut engine = Engine::from_connection(conn).expect("the older shape is opened");
    let story = engine
        .story_titled(&format!("{TITLE}{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    played(&mut engine, game, "harm 5");
    submitted(
        &mut engine,
        game,
        "/move The First Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(hp(&engine, game), Some(13));
    assert_eq!(sheet(&engine, game), "level 3, d8");
}

#[test]
fn a_toll_is_said_against_what_the_body_could_take_when_it_was_paid() {
    let records = Records::from_json(&json!({
        "stories": [{"id": 1}],
        "playthroughs": [{"id": 1, "story_id": 1, "character_id": 1}],
        "characters": [
            {"id": 1, "fullname": "Odile", "level": 3, "hit_die": 8, "is_protagonist": true},
            {"id": 2, "fullname": "Wenna Cole", "level": 2, "hit_die": 6},
        ],
        "locations": [
            {"id": 1, "name": "The Landing"},
            {"id": 2, "name": "The First Floor"},
        ],
        "playthrough_floors": [
            {"id": 1, "playthrough_id": 1, "location_id": 2, "arrived_at": 1000, "level": 4},
        ],
        "playthrough_tolls": [
            {"id": 1, "playthrough_id": 1, "character_id": 1, "location_id": 1,
             "hazard": "unlit", "saved": false, "damage": 3, "hp_after": 15,
             "story_timestamp": 900},
            {"id": 2, "playthrough_id": 1, "character_id": 1, "location_id": 2,
             "hazard": "unlit", "saved": false, "damage": 3, "hp_after": 20,
             "story_timestamp": 1000},
        ],
    }));
    let game = Game::new(&records, 1);
    let odile = game.character(1);
    assert_eq!(game.max_hp_at(odile, Some(999)), Some(18));
    assert_eq!(game.max_hp_at(odile, Some(1000)), Some(23));
    assert_eq!(game.max_hp(odile), Some(23));
    assert_eq!(
        game.level(game.character(2)),
        Some(2),
        "a floor raises only the player"
    );
    let tolls = game.own("playthrough_tolls");
    assert!(
        game.toll_to_s(tolls[0])
            .ends_with("Odile is hurt (15 of 18)"),
        "{}",
        game.toll_to_s(tolls[0])
    );
    assert!(
        game.toll_to_s(tolls[1])
            .ends_with("Odile is hurt (20 of 23)"),
        "{}",
        game.toll_to_s(tolls[1])
    );
    assert_eq!(
        game.floor_to_s(game.floors_reached()[0]).as_deref(),
        Some(
            "Reaching The First Floor raised Odile to level 4: 23 hit points, and healed to full."
        )
    );
}
