//! What an overdue arc places, and where: always in ground nobody has
//! written yet, never as a doorway or a thing added to a room the world has
//! already written.
//!
//! A Yard Before the Winter is walked as
//! `a-deadline-leaves-written-rooms-as-they-were` walks it: three rooms
//! written past the arc's grace, each placing one of the arc's three steps,
//! and then a walk to each of them. Two of the tests edit the world's rows
//! first: one leaves nothing unwritten to grow from, one puts the thing
//! first.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::parity::{load_script, open_world, play_one, rows, Script};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;

const YARD: i64 = 1_000_000_001;
const KITCHEN: i64 = 1_000_000_002;
const LOFT: i64 = 1_000_000_003;
const COLD_STORE: i64 = 1_000_000_004;
const WASH_HOUSE: i64 = 1_000_000_005;
const BYRE: i64 = 1_000_000_006;

/// The arc's first two steps, Warden Hesk and the Salt Stair, taken out so
/// the iron seal is the first step due.
const THE_SEAL_FIRST: &str = "DELETE FROM quest_steps WHERE id IN (1000000001, 1000000002);";

/// The wash house and the byre taken out, so that once the cold store is
/// written nothing the party can reach is unwritten.
const NOTHING_LEFT_UNWRITTEN: &str = "
    DELETE FROM location_connections
     WHERE location_id IN (1000000005, 1000000006)
        OR connected_location_id IN (1000000005, 1000000006);
    DELETE FROM locations WHERE id IN (1000000005, 1000000006);
";

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn the_walk() -> Script {
    load_script(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("parity/scripts/a-deadline-leaves-written-rooms-as-they-were.yml"),
    )
    .unwrap()
}

/// The yard's world with `sql` run over it.
fn the_yard(sql: &str) -> Engine {
    open_world(&format!(
        "{}{sql}",
        read("parity/worlds/a-yard-before-the-winter.sql")
    ))
    .unwrap()
}

/// Plays the walk's steps `from` to `to`, counting from 1.
fn walk(engine: &mut Engine, script: &Script, from: usize, to: usize) {
    for step in &script.steps[from - 1..to] {
        play_one(engine, script, step.index, &step.player)
            .unwrap_or_else(|error| panic!("{}: {error}", step.label()));
    }
}

/// Each written room as the world holds it: its row, less the
/// protagonist's last visit, which every walk in writes; every doorway out
/// of it and into it; and every thing of the world's own lying in it.
fn written_rooms(engine: &Engine) -> BTreeMap<i64, Value> {
    let rows = rows(engine).unwrap();
    let table = |name: &str| rows[name].as_array().unwrap().clone();
    let mut rooms = BTreeMap::new();
    for room in table("locations")
        .into_iter()
        .filter(|room| room["detail_level"] == "realized")
    {
        let id = room["id"].as_i64().unwrap();
        let mut row = room.clone();
        row.as_object_mut()
            .unwrap()
            .remove("last_protagonist_visit");
        let doorways: Vec<Value> = table("location_connections")
            .into_iter()
            .filter(|way| way["location_id"] == id || way["connected_location_id"] == id)
            .collect();
        let things: Vec<Value> = table("items")
            .into_iter()
            .filter(|thing| thing["location_id"] == id && thing["playthrough_id"].is_null())
            .collect();
        rooms.insert(
            id,
            json!({ "room": row, "doorways": doorways, "things": things }),
        );
    }
    rooms
}

fn query<T: rusqlite::types::FromSql>(engine: &Engine, sql: &str) -> T {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

/// Where the thing or person a step of the arc is bound to stands.
fn where_step(engine: &Engine, step: i64) -> i64 {
    query(
        engine,
        &format!(
            "SELECT COALESCE(i.location_id, c.location_id, l.id) FROM quest_steps s
               LEFT JOIN items i ON s.target_type = 'Item' AND i.id = s.target_id
               LEFT JOIN characters c ON s.target_type = 'Character' AND c.id = s.target_id
               LEFT JOIN locations l ON s.target_type = 'Location' AND l.id = s.target_id
              WHERE s.id = {step}"
        ),
    )
}

fn detail_level(engine: &Engine, room: i64) -> String {
    query(
        engine,
        &format!("SELECT detail_level FROM locations WHERE id = {room}"),
    )
}

/// Every step of the walk leaves each room written before it exactly as it
/// was, though each of the three rooms it writes places one of the arc's
/// steps; and the walk reaches all three.
#[test]
fn a_deadline_lands_without_changing_a_room_already_written() {
    let script = the_walk();
    let mut engine = the_yard("");
    for step in &script.steps {
        let before = written_rooms(&engine);
        walk(&mut engine, &script, step.index, step.index);
        let after = written_rooms(&engine);
        for (room, was) in &before {
            assert_eq!(
                after.get(room),
                Some(was),
                "{} changed room {room}, written before it",
                step.label()
            );
        }
    }
    let unbound: i64 = query(
        &engine,
        "SELECT COUNT(*) FROM quest_steps WHERE target_id IS NULL",
    );
    assert_eq!(unbound, 0, "the arc's three steps are placed");
    for step in 1_000_000_001..=1_000_000_003 {
        let room = where_step(&engine, step);
        assert_eq!(
            detail_level(&engine, room),
            "stub",
            "step {step} stands in ground nobody has written"
        );
    }
    let reached: i64 = query(
        &engine,
        "SELECT COUNT(*) FROM playthrough_beats WHERE quest_step_id IN (1000000002, 1000000003)",
    );
    assert_eq!(
        reached, 2,
        "the walk reaches the Salt Stair and takes up the iron seal"
    );
}

/// With nothing the party can reach left unwritten, the room whose writing
/// placed the step is the next room written, and the step grows off it as
/// part of that writing; the rooms written before it keep their doorways.
#[test]
fn with_nothing_left_unwritten_the_step_grows_off_the_room_being_written() {
    let script = the_walk();
    let mut engine = the_yard(NOTHING_LEFT_UNWRITTEN);
    let before = written_rooms(&engine);
    walk(&mut engine, &script, 1, 1);
    let after = written_rooms(&engine);
    for room in [YARD, KITCHEN, LOFT] {
        assert_eq!(after.get(&room), before.get(&room), "room {room} changed");
    }
    let off_the_cold_store: String = query(
        &engine,
        &format!(
            "SELECT l.name FROM location_connections c JOIN locations l ON l.id = c.connected_location_id
              WHERE c.location_id = {COLD_STORE} AND l.id <> {YARD}"
        ),
    );
    assert_eq!(off_the_cold_store, "where Warden Hesk is room 1");
    let warden = where_step(&engine, 1_000_000_001);
    assert_eq!(detail_level(&engine, warden), "stub");
}

/// A thing lies in the way on, a room nobody has written, given the box its
/// writing would give it so the thing has a spot; writing the room later
/// keeps both.
#[test]
fn a_thing_lies_in_a_room_nobody_has_written_and_its_writing_keeps_it() {
    let script = the_walk();
    let mut engine = the_yard(THE_SEAL_FIRST);
    walk(&mut engine, &script, 1, 1);
    assert_eq!(where_step(&engine, 1_000_000_003), WASH_HOUSE);
    assert_eq!(detail_level(&engine, WASH_HOUSE), "stub");
    let placed = |engine: &Engine| -> (i64, i64, i64, i64) {
        let conn = engine.store().connection();
        let (width, depth): (i64, i64) = conn
            .query_row(
                &format!("SELECT width, depth FROM locations WHERE id = {WASH_HOUSE}"),
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        let (x, y): (i64, i64) = conn
            .query_row(
                "SELECT x, y FROM items WHERE name = 'iron seal' AND playthrough_id IS NULL",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        (width, depth, x, y)
    };
    let laid = placed(&engine);
    walk(&mut engine, &script, 2, 3);
    assert_eq!(detail_level(&engine, WASH_HOUSE), "realized");
    assert_eq!(
        placed(&engine),
        laid,
        "the writing keeps the box and the spot"
    );
    assert_eq!(where_step(&engine, 1_000_000_003), WASH_HOUSE);
    let byre_untouched: i64 = query(
        &engine,
        &format!("SELECT COUNT(*) FROM items WHERE location_id = {BYRE}"),
    );
    assert_eq!(byre_untouched, 0);
}
