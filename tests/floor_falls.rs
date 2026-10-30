//! A floor with a countdown, as a caller holds it: A Floor That Falls walked
//! into on the live path with replayed replies and by the engine alone, and
//! the rows and the glance read back.
//!
//! The First Floor is two rooms and says `minutes: 30`; Wenna Cole is posted
//! to it, so she stands in the room with the stair down. The Second Floor is
//! two rooms with no countdown. The story starts at 8:00 am, so a game that
//! walks onto the first floor at once sees it fall at 8:30.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::glance::Floor;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::{Countdown, Game};
use renderedstep_engine::records::id;
use serde_json::{json, Value};
use std::path::Path;

fn world() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-floor-that-falls.sql");
    std::fs::read_to_string(path).expect("a world fixture")
}

const TITLE: &str = "A Floor That Falls";

/// 2026-10-31 08:30:00 UTC: when The First Floor falls for a game that
/// walked onto it at the story's start.
const HALF_PAST_EIGHT: i64 = 1_793_435_400;

const DEATH: &str = "Odile is dead, and this playthrough is over. Nothing you type can \
                     change it. Start a new playthrough to play this world again.";

fn reply(value: Value) -> Reply {
    Reply::from_value(&value).unwrap()
}

fn place() -> Reply {
    reply(json!({"purpose": "location", "content": {
        "description": "Black stone passages, cracked across every few paces.",
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

fn no_arrival() -> Reply {
    reply(json!({"purpose": "arrival", "unavailable": true}))
}

fn narration() -> Reply {
    reply(json!({"purpose": "narration", "content": "You wait."}))
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

/// `line` played by the engine alone: its refusal, if any, and its notes.
fn played(engine: &mut Engine, playthrough: i64, line: &str) -> (Option<String>, Vec<String>) {
    let report = engine.play(playthrough, line, &mut |_| {}).unwrap().report;
    (report.refusal, report.note)
}

fn opened() -> (Engine, i64) {
    let engine = open_world(&world()).unwrap();
    let story = engine
        .story_titled(&format!("{TITLE}{TITLE_SUFFIX}"))
        .unwrap();
    (engine, story)
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

fn execute(engine: &Engine, sql: &str) {
    engine.store().connection().execute_batch(sql).unwrap();
}

fn location(engine: &Engine, name: &str) -> i64 {
    query(
        engine,
        &format!("SELECT id FROM locations WHERE name = '{name}'"),
    )
    .unwrap()
}

/// When a game's floor fell, in seconds, none while it stands.
fn fell_at(engine: &Engine, playthrough: i64, floor: i64) -> Option<i64> {
    query(
        engine,
        &format!(
            "SELECT CAST(strftime('%s', fell_at) AS INTEGER) FROM playthrough_floors \
             WHERE playthrough_id = {playthrough} AND location_id = {floor}"
        ),
    )
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

/// Every world row the fall of The First Floor could have touched, as text:
/// its own rows and its rooms', every doorway, every person, every floor.
fn world_rows(engine: &Engine) -> String {
    let connection = engine.store().connection();
    let first = location(engine, "The First Floor");
    [
        format!("locations WHERE id = {first} OR parent_location_id = {first}"),
        "location_connections".into(),
        "characters".into(),
        "floors".into(),
    ]
    .iter()
    .map(|rows| {
        let mut statement = connection
            .prepare(&format!("SELECT * FROM {rows} ORDER BY id"))
            .unwrap();
        let width = statement.column_count();
        let rows: Vec<String> = statement
            .query_map([], |row| {
                Ok((0..width)
                    .map(|at| match row.get_ref(at).unwrap() {
                        rusqlite::types::ValueRef::Text(text) => {
                            String::from_utf8_lossy(text).into_owned()
                        }
                        other => format!("{other:?}"),
                    })
                    .collect::<Vec<_>>()
                    .join(","))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows.join("\n")
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// A game walked onto The First Floor on the live path.
fn on_the_first_floor(engine: &mut Engine, story: i64) -> i64 {
    let game = engine.start(story).unwrap();
    submitted(
        engine,
        game,
        "/move The First Floor",
        vec![place(), room(), no_arrival()],
    );
    game
}

/// A game that has gone on down to The Second Floor.
fn on_the_second_floor(engine: &mut Engine, story: i64) -> i64 {
    let game = on_the_first_floor(engine, story);
    submitted(
        engine,
        game,
        "/move The First Floor room 2",
        vec![room(), no_arrival()],
    );
    submitted(
        engine,
        game,
        "/move The Second Floor",
        vec![place(), room(), no_arrival()],
    );
    game
}

#[test]
fn waiting_past_the_countdown_on_the_floor_is_fatal_and_never_refused() {
    let (mut engine, story) = opened();
    let first = location(&engine, "The First Floor");
    let game = on_the_first_floor(&mut engine, story);
    assert_eq!(
        engine.glance(game).unwrap().floor,
        Some(Floor {
            name: "The First Floor".into(),
            falls_at: HALF_PAST_EIGHT,
            minutes_left: 29,
            countdown: "The First Floor falls in 29 minutes.".into(),
        }),
        "the walk in took a minute and a half of the thirty"
    );

    let mut sent = Vec::new();
    for _ in 0..5 {
        let (refusal, requests) = submitted(&mut engine, game, "/wait", vec![narration()]);
        assert_eq!(refusal, None);
        sent.extend(requests);
    }
    assert_eq!(
        engine
            .glance(game)
            .unwrap()
            .floor
            .map(|floor| floor.countdown),
        Some("The First Floor falls in 4 minutes.".into())
    );
    assert_eq!(fell_at(&engine, game, first), None);

    let (refusal, requests) = submitted(&mut engine, game, "/wait", vec![narration()]);
    sent.extend(requests);
    assert_eq!(refusal, None, "the wait across the countdown is played");
    assert_eq!(fell_at(&engine, game, first), Some(HALF_PAST_EIGHT));
    assert_eq!(hp(&engine, game), Some(0));
    let glance = engine.glance(game).unwrap();
    assert!(glance.over);
    assert_eq!(glance.floor, None);
    assert_eq!(
        glance.collapse.as_deref(),
        Some("The First Floor gave way at 8:30 am, and Odile fell with it.")
    );
    assert_eq!(glance.ended.as_deref(), Some(DEATH));

    for request in &sent {
        let text = request.to_string();
        for words in ["falls in", "gave way", "has fallen"] {
            assert!(!text.contains(words), "a request tells {words:?}: {text}");
        }
    }
    let (refusal, _) = submitted(&mut engine, game, "/wait", Vec::new());
    assert_eq!(refusal.as_deref(), Some(DEATH));
}

#[test]
fn a_floor_falls_behind_a_player_who_went_down_in_their_game_only() {
    let (mut engine, story) = opened();
    let first = location(&engine, "The First Floor");
    let game = on_the_second_floor(&mut engine, story);
    let room_2 = location(&engine, "The First Floor room 2");
    let before = world_rows(&engine);
    for _ in 0..6 {
        submitted(&mut engine, game, "/wait", vec![narration()]);
    }
    assert_eq!(fell_at(&engine, game, first), Some(HALF_PAST_EIGHT));
    assert_eq!(hp(&engine, game), Some(18), "she was not on it");
    let glance = engine.glance(game).unwrap();
    assert!(!glance.over);
    assert_eq!(glance.collapse, None);
    assert_eq!(glance.floor, None, "The Second Floor has no countdown");
    let exits: Vec<String> = glance.exits.iter().map(|exit| exit.name.clone()).collect();
    assert_eq!(exits, ["The Second Floor room 2"]);

    for line in ["/move The First Floor room 2", "/move The First Floor"] {
        let (refusal, _) = played(&mut engine, game, line);
        assert_eq!(
            refusal.as_deref(),
            Some("The First Floor has fallen. There is no way back onto it."),
            "{line}"
        );
        let (refusal, sent) = submitted(&mut engine, game, line, Vec::new());
        assert!(sent.is_empty(), "no reader is asked");
        assert_eq!(
            refusal.as_deref(),
            Some("The First Floor has fallen. There is no way back onto it. Nothing has changed."),
            "{line}"
        );
    }

    // Wenna is dead in this game, and alive in the world and in the next.
    let records = engine.store().load().unwrap();
    let wenna_room = records.find("locations", room_2).unwrap();
    assert!(Game::new(&records, game)
        .cast_in(Some(wenna_room))
        .is_empty());
    assert!(Game::new(&records, game).fallen(wenna_room));
    assert_eq!(world_rows(&engine), before, "no world row is touched");

    let second = engine.start(story).unwrap();
    played(&mut engine, second, "/move The First Floor");
    played(&mut engine, second, "/move The First Floor room 2");
    let records = engine.store().load().unwrap();
    let here = Game::new(&records, second);
    let present: Vec<i64> = here
        .cast_in(here.current_location())
        .into_iter()
        .filter(|who| id(who) != id(here.protagonist().unwrap()))
        .map(id)
        .collect();
    assert_eq!(
        present,
        [query(
            &engine,
            "SELECT id FROM characters WHERE fullname = 'Wenna Cole'"
        )
        .unwrap()]
    );
    assert_eq!(fell_at(&engine, second, first), None);
    assert_eq!(
        engine.glance(second).unwrap().floor.map(|floor| floor.name),
        Some("The First Floor".into()),
        "the second game's countdown is its own"
    );
}

#[test]
fn a_countdown_already_run_out_takes_the_player_before_the_line_is_read() {
    let (mut engine, story) = opened();
    let first = location(&engine, "The First Floor");
    let game = on_the_first_floor(&mut engine, story);
    execute(
        &engine,
        "UPDATE playthrough_floors SET arrived_at = '2026-10-31 07:00:00';",
    );
    let (refusal, notes) = played(&mut engine, game, "harm 1");
    assert_eq!(refusal.as_deref(), Some(DEATH));
    assert_eq!(
        notes,
        ["the game: The First Floor gave way at 7:30 am, and Odile fell with it."]
    );
    assert_eq!(fell_at(&engine, game, first), Some(HALF_PAST_EIGHT - 3600));
    assert_eq!(hp(&engine, game), Some(0));

    // The same on the live path: refused before any reader is asked.
    let (mut engine, story) = opened();
    let game = on_the_first_floor(&mut engine, story);
    execute(
        &engine,
        "UPDATE playthrough_floors SET arrived_at = '2026-10-31 07:00:00';",
    );
    let (refusal, sent) = submitted(&mut engine, game, "/wait", Vec::new());
    assert!(sent.is_empty());
    assert_eq!(refusal.as_deref(), Some(DEATH));
    assert_eq!(
        engine.glance(game).unwrap().collapse.as_deref(),
        Some("The First Floor gave way at 7:30 am, and Odile fell with it.")
    );
}

#[test]
fn a_floor_with_no_countdown_never_falls() {
    let (mut engine, story) = opened();
    execute(&engine, "UPDATE floors SET minutes = NULL;");
    let game = on_the_first_floor(&mut engine, story);
    execute(
        &engine,
        "UPDATE playthrough_floors SET arrived_at = '2026-10-01 07:00:00';",
    );
    let (refusal, notes) = played(&mut engine, game, "harm 1");
    assert_eq!(refusal, None);
    assert!(notes.is_empty(), "{notes:?}");
    let glance = engine.glance(game).unwrap();
    assert!(!glance.over);
    assert_eq!(glance.floor, None);
}

#[test]
fn a_database_from_before_the_countdown_plays_and_nothing_falls() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world()).unwrap();
    conn.execute_batch(
        r#"ALTER TABLE "floors" DROP COLUMN "minutes";
           ALTER TABLE "playthrough_floors" DROP COLUMN "fell_at";
           DELETE FROM schema_migrations WHERE version = '20261031120000';"#,
    )
    .unwrap();
    let mut engine = Engine::from_connection(conn).expect("the older shape is opened");
    let story = engine
        .story_titled(&format!("{TITLE}{TITLE_SUFFIX}"))
        .unwrap();
    let game = on_the_first_floor(&mut engine, story);
    for _ in 0..7 {
        let (refusal, _) = submitted(&mut engine, game, "/wait", vec![narration()]);
        assert_eq!(refusal, None);
    }
    let glance = engine.glance(game).unwrap();
    assert!(!glance.over);
    assert_eq!(glance.floor, None);
    assert_eq!(hp(&engine, game), Some(18));
}

#[test]
fn the_countdown_is_said_in_hours_and_minutes() {
    let said = |minutes_left| {
        Countdown {
            name: "The First Floor".into(),
            falls_at: 0,
            minutes_left,
        }
        .in_words()
    };
    assert_eq!(said(25), "The First Floor falls in 25 minutes.");
    assert_eq!(said(1), "The First Floor falls in 1 minute.");
    assert_eq!(said(60), "The First Floor falls in 1 hour 0 minutes.");
    assert_eq!(said(65), "The First Floor falls in 1 hour 5 minutes.");
    assert_eq!(said(1152), "The First Floor falls in 19 hours 12 minutes.");
    assert_eq!(said(121), "The First Floor falls in 2 hours 1 minute.");
}

#[test]
fn a_collapse_is_said_at_the_minute_it_came() {
    use renderedstep_engine::moment::clock_time;
    assert_eq!(clock_time(HALF_PAST_EIGHT), "8:30 am");
    assert_eq!(clock_time(HALF_PAST_EIGHT + 4 * 3600 + 35 * 60), "1:05 pm");
    assert_eq!(clock_time(HALF_PAST_EIGHT - 8 * 3600 - 30 * 60), "12:00 am");
}
