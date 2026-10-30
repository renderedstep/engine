//! A floor's supplies, as a caller holds them: A Stocked Floor walked into on
//! the live path with replayed replies, and the rows, the requests and the
//! glance read back.
//!
//! The Stocked Floor is four rooms and says `safe_rooms: 1` and two red
//! draughts. Its layout makes room 4 the safe room and leaves a draught in
//! room 2 and in room 4; the entry, room 1, gets neither. Rooms 1, 2, 3 and
//! 4 are walked in that order, each a doorway from the last.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::glance::SAFE_ROOM;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::{json, Value};
use std::path::Path;

fn world() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-stocked-floor.sql");
    std::fs::read_to_string(path).expect("a world fixture")
}

const TITLE: &str = "A Stocked Floor";

fn reply(value: Value) -> Reply {
    Reply::from_value(&value).unwrap()
}

fn place() -> Reply {
    reply(json!({"purpose": "location", "content": {
        "description": "Black stone passages.",
        "lore": "Cut by people nobody remembers.",
        "place_kind": "warren",
    }}))
}

fn room() -> Reply {
    room_with(json!([]), json!([]))
}

fn room_with(items: Value, people: Value) -> Reply {
    reply(json!({"purpose": "location", "content": {
        "description": "A bare stone chamber.",
        "lore": "Old.",
        "items": items,
        "people": people,
    }}))
}

fn no_arrival() -> Reply {
    reply(json!({"purpose": "arrival", "unavailable": true}))
}

/// `line` submitted on the live path, answered by `replies` in turn: every
/// request it sent.
fn submitted(engine: &mut Engine, playthrough: i64, line: &str, replies: Vec<Reply>) -> Vec<Value> {
    let mut replay = Replay::new(replies);
    let token = format!("{line}-{}", count(engine, "playthrough_commands"));
    let submitted = engine
        .submit(playthrough, line, &token, &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(
        submitted.turned.refusal.map(|refusal| refusal.text()),
        None,
        "{line}"
    );
    replay.sent().to_vec()
}

fn opened_from(sql: &str) -> (Engine, i64) {
    let engine = open_world(sql).unwrap();
    let story = engine
        .story_titled(&format!("{TITLE}{TITLE_SUFFIX}"))
        .unwrap();
    (engine, story)
}

fn opened() -> (Engine, i64) {
    opened_from(&world())
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

fn room_id(engine: &Engine, number: i64) -> i64 {
    location(engine, &format!("The Stocked Floor room {number}"))
}

/// Rows of `sql`'s single column, as text.
fn column(engine: &Engine, sql: &str) -> Vec<String> {
    let connection = engine.store().connection();
    let mut statement = connection.prepare(sql).unwrap();
    let rows = statement
        .query_map([], |row| {
            Ok(match row.get_ref(0).unwrap() {
                rusqlite::types::ValueRef::Text(text) => String::from_utf8_lossy(text).into_owned(),
                other => format!("{other:?}"),
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    rows
}

/// The world's rows the layout wrote for the floor: its rooms and the things
/// lying in them, every column but the timestamps.
fn floor_rows(engine: &Engine) -> Vec<String> {
    let floor = location(engine, "The Stocked Floor");
    let mut rows = column(
        engine,
        &format!(
            "SELECT json_array(id, name, danger, hazard, population, sanctuary) FROM locations \
             WHERE parent_location_id = {floor} ORDER BY id"
        ),
    );
    rows.extend(column(
        engine,
        "SELECT json_array(id, location_id, name, description, use_kind, bulk, kit_key, supply, x, y) \
         FROM items WHERE playthrough_id IS NULL ORDER BY id",
    ));
    rows
}

/// A game walked onto The Stocked Floor on the live path: its requests.
fn onto_the_floor(engine: &mut Engine, game: i64) -> Vec<Value> {
    submitted(
        engine,
        game,
        "/move The Stocked Floor",
        vec![place(), room(), no_arrival()],
    )
}

/// The room-writing request among `requests`.
fn room_request(requests: &[Value]) -> String {
    requests
        .iter()
        .find(|request| request["purpose"] == "location")
        .map(|request| request["user"].as_str().unwrap_or_default().to_string())
        .expect("a room was written")
}

#[test]
fn a_floor_is_stocked_once_when_it_is_laid_out() {
    let (mut engine, story) = opened();
    let game = engine.start(story).unwrap();
    onto_the_floor(&mut engine, game);
    let floor = location(&engine, "The Stocked Floor");
    let entry = room_id(&engine, 1);

    let safe = column(
        &engine,
        &format!(
            "SELECT json_array(name, danger, hazard, population) FROM locations \
             WHERE parent_location_id = {floor} AND sanctuary ORDER BY id"
        ),
    );
    assert_eq!(
        safe,
        vec![r#"["The Stocked Floor room 4","safe",null,"nobody"]"#],
        "one safe room, never the entry"
    );
    assert_eq!(
        query(
            &engine,
            &format!("SELECT COUNT(*) FROM locations WHERE parent_location_id = {floor}")
        ),
        Some(4)
    );

    let supplies = column(
        &engine,
        "SELECT json_array(location_id, name, use_kind, bulk, kit_key, tier) FROM items \
         WHERE supply = 'floor' AND playthrough_id IS NULL ORDER BY location_id",
    );
    let description = "A stoppered glass vial of thick red liquid, warm to the touch. \
                       Drinking it closes wounds.";
    assert_eq!(
        supplies,
        [2, 4]
            .map(|room| format!(
                r#"[{},"red draught","healing","handy",null,"portable"]"#,
                room_id(&engine, room)
            ))
            .to_vec(),
        "two draughts, each in a room of its own, neither in the entry"
    );
    assert_eq!(
        column(
            &engine,
            &format!(
                "SELECT DISTINCT description FROM items WHERE supply = 'floor' AND location_id <> {entry}"
            )
        ),
        vec![description]
    );
    assert!(
        query(
            &engine,
            "SELECT COUNT(*) FROM items WHERE supply = 'floor' AND (x IS NULL OR y IS NULL)"
        ) == Some(0),
        "each draught is placed in its room"
    );

    let laid = floor_rows(&engine);
    let second = engine.start(story).unwrap();
    let sent = submitted(
        &mut engine,
        second,
        "/move The Stocked Floor",
        vec![no_arrival()],
    );
    assert!(
        sent.iter().all(|request| request["purpose"] != "location"),
        "the floor is not written again"
    );
    submitted(
        &mut engine,
        second,
        "/move The Stocked Floor room 2",
        vec![room(), no_arrival()],
    );
    let after = floor_rows(&engine);
    assert_eq!(
        query(
            &engine,
            "SELECT COUNT(*) FROM items WHERE supply = 'floor' AND playthrough_id IS NULL"
        ),
        Some(2),
        "no later game stocks the floor again"
    );
    assert!(
        laid.iter().all(|row| after.contains(row)),
        "every row the layout wrote is as it was"
    );
}

#[test]
fn the_room_panel_says_a_room_is_safe_and_no_request_does() {
    let (mut engine, story) = opened();
    let game = engine.start(story).unwrap();
    let mut sent = onto_the_floor(&mut engine, game);
    assert_eq!(engine.glance(game).unwrap().here.unwrap().safe, None);
    let mut safe_room = String::new();
    for number in 2..=4 {
        let requests = submitted(
            &mut engine,
            game,
            &format!("/move The Stocked Floor room {number}"),
            vec![room(), no_arrival()],
        );
        if number == 4 {
            safe_room = room_request(&requests);
        }
        sent.extend(requests);
        let here = engine.glance(game).unwrap().here.unwrap();
        assert_eq!(here.name, format!("The Stocked Floor room {number}"));
        assert_eq!(
            here.safe.as_deref(),
            (number == 4).then_some(SAFE_ROOM),
            "room {number}"
        );
    }
    assert_eq!(
        SAFE_ROOM,
        "A safe room: nothing hostile is born here, and resting here heals."
    );
    assert!(
        safe_room.contains("Write NOBODY into this place."),
        "{safe_room}"
    );
    for request in &sent {
        let text = request.to_string();
        for words in ["safe room", "sanctuary", "resting here heals"] {
            assert!(!text.contains(words), "a request tells {words:?}: {text}");
        }
    }
}

/// The story's thing limit is set to one: the two draughts are not among
/// what it counts, so the room writer is still offered one thing, and it is
/// admitted. The room with a draught in it is still furnished from its sort.
#[test]
fn a_draught_counts_against_no_limit_and_its_room_is_still_furnished() {
    let (mut engine, story) = opened();
    execute(
        &engine,
        &format!("UPDATE stories SET things_limit = 1 WHERE id = {story}"),
    );
    let game = engine.start(story).unwrap();
    onto_the_floor(&mut engine, game);
    let things = json!([
        {"name": "tallow candle", "description": "A stub of tallow candle.", "use_kind": "ordinary"},
        {"name": "chalk stick", "description": "A stick of white chalk.", "use_kind": "ordinary"},
    ]);
    let sent = submitted(
        &mut engine,
        game,
        "/move The Stocked Floor room 2",
        vec![room_with(things, json!([])), no_arrival()],
    );
    let request = room_request(&sent);
    assert!(
        request.contains("Loose, and could be picked up: red draught."),
        "{request}"
    );
    assert!(
        request.contains("List AT MOST 1 portable thing a player could pick up"),
        "{request}"
    );
    let room = room_id(&engine, 2);
    assert_eq!(
        column(
            &engine,
            &format!(
                "SELECT name FROM items WHERE location_id = {room} AND playthrough_id IS NULL \
                 AND kit_key IS NULL ORDER BY id"
            )
        ),
        vec!["red draught", "tallow candle"],
        "the draught, and the one thing the limit leaves room for"
    );
    assert!(
        query(
            &engine,
            &format!(
                "SELECT COUNT(*) FROM items WHERE location_id = {room} AND playthrough_id IS NULL \
                 AND kit_key IS NOT NULL"
            )
        )
        .unwrap()
            > 0,
        "the room holding a draught is still furnished"
    );
}

/// However its population reads, nobody is born in a safe room.
#[test]
fn a_safe_room_births_nobody_whatever_its_population_says() {
    let (mut engine, story) = opened();
    let game = engine.start(story).unwrap();
    onto_the_floor(&mut engine, game);
    execute(
        &engine,
        "UPDATE locations SET population = 'a crowd' WHERE sanctuary",
    );
    for number in 2..=3 {
        submitted(
            &mut engine,
            game,
            &format!("/move The Stocked Floor room {number}"),
            vec![room(), no_arrival()],
        );
    }
    let person = json!([{
        "fullname": "Mott Grey", "nickname": "Mott", "appearance": "A grey cloak.",
        "personality": "Quiet.", "backstory": "A delver.", "likes": "Quiet.",
        "dislikes": "Noise.", "fears": "The dark.",
    }]);
    let sent = submitted(
        &mut engine,
        game,
        "/move The Stocked Floor room 4",
        vec![room_with(json!([]), person), no_arrival()],
    );
    assert!(room_request(&sent).contains("Write NOBODY into this place."));
    let safe = room_id(&engine, 4);
    assert_eq!(
        query(
            &engine,
            &format!("SELECT COUNT(*) FROM characters WHERE location_id = {safe}")
        ),
        Some(0)
    );
}

/// A database from before the supplies' migration lays the floor out bare:
/// no safe room and nothing left about it, and each room's things are
/// copied into the game as before.
#[test]
fn a_database_from_before_the_supplies_lays_the_floor_out_bare() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world()).unwrap();
    conn.execute_batch(
        r#"ALTER TABLE "floors" DROP COLUMN "safe_rooms";
           ALTER TABLE "floors" DROP COLUMN "healing_count";
           ALTER TABLE "floors" DROP COLUMN "healing_name";
           ALTER TABLE "floors" DROP COLUMN "healing_description";
           ALTER TABLE "locations" DROP COLUMN "sanctuary";
           ALTER TABLE "items" DROP COLUMN "supply";
           DELETE FROM schema_migrations WHERE version = '20261031150000';"#,
    )
    .unwrap();
    let mut engine = Engine::from_connection(conn).expect("the older shape is opened");
    let story = engine
        .story_titled(&format!("{TITLE}{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    onto_the_floor(&mut engine, game);
    for number in 2..=4 {
        submitted(
            &mut engine,
            game,
            &format!("/move The Stocked Floor room {number}"),
            vec![room(), no_arrival()],
        );
        assert_eq!(engine.glance(game).unwrap().here.unwrap().safe, None);
    }
    assert_eq!(
        query(
            &engine,
            "SELECT COUNT(*) FROM items WHERE name = 'red draught'"
        ),
        Some(0)
    );
    assert!(!engine.glance(game).unwrap().lying_here.is_empty());
}
