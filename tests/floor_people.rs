//! Who stands on a floor, as a caller holds it: the guard a world posts to
//! a floor, stood on its way down when the floor is laid out, and the level
//! everybody born on a floor is given. A Guarded Stair and A Deeper Floor
//! walked into on the live path with a scripted model, and the rows read
//! back.
//!
//! In A Guarded Stair, Brask the Stairwarden is posted to The Upper Floor
//! (three rooms, a way down to The Lower Floor) and kept absent by the file.
//! In A Deeper Floor, The Bottom Floor's row names monster level 4 and The
//! Upper Floor's names none; each is one room.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Live, Route, Secret};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::{json, Value};
use std::path::Path;

fn world(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("parity/worlds/{name}.sql"));
    std::fs::read_to_string(path).expect("a world fixture")
}

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

/// A room's answer, naming one person, whom the room may or may not have
/// a place for.
fn room() -> Value {
    json!({
        "description": "A bare stone chamber.",
        "lore": "Old.",
        "items": [],
        "people": [{
            "fullname": "Hollin Graves", "nickname": "Hollin", "appearance": "A grey coat.",
            "personality": "Quiet.", "backstory": "A delver who stayed.",
            "likes": "Silence.", "dislikes": "Lanterns.", "fears": "The surface.",
        }],
    })
}

fn arrival() -> Value {
    json!({"description": "You arrive.", "summary": "Arrived."})
}

/// `line` submitted on the live path, the model answering `answers` in
/// turn; how many calls it made.
fn submitted(engine: &mut Engine, playthrough: i64, line: &str, answers: Vec<Value>) -> usize {
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
    live.transport().sent.len()
}

fn played(engine: &mut Engine, playthrough: i64, line: &str) -> Option<String> {
    engine
        .play(playthrough, line, &mut |_| {})
        .unwrap()
        .report
        .refusal
}

fn opened(name: &str, title: &str) -> (Engine, i64) {
    let engine = open_world(&world(name)).unwrap();
    let story = engine
        .story_titled(&format!("{title}{TITLE_SUFFIX}"))
        .unwrap();
    (engine, story)
}

fn guarded() -> (Engine, i64) {
    opened("a-guarded-stair", "A Guarded Stair")
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

/// Every row `sql` selects, each column as text.
fn rows(engine: &Engine, sql: &str) -> Vec<Vec<String>> {
    let connection = engine.store().connection();
    let mut statement = connection.prepare(sql).unwrap();
    let width = statement.column_count();
    statement
        .query_map([], |row| {
            (0..width)
                .map(|column| {
                    row.get::<_, rusqlite::types::Value>(column)
                        .map(|value| match value {
                            rusqlite::types::Value::Null => "null".to_string(),
                            rusqlite::types::Value::Integer(n) => n.to_string(),
                            rusqlite::types::Value::Text(text) => text,
                            other => format!("{other:?}"),
                        })
                })
                .collect()
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn location(engine: &Engine, name: &str) -> i64 {
    int(
        engine,
        &format!("SELECT id FROM locations WHERE name = '{name}'"),
    )
}

fn brask(engine: &Engine) -> Vec<String> {
    rows(
        engine,
        "SELECT location_id, deliberately_absent, posted_to_id, x, y FROM characters \
         WHERE fullname = 'Brask the Stairwarden'",
    )
    .pop()
    .expect("Brask is in the world")
}

/// The room of The Upper Floor its stair down to The Lower Floor leaves.
fn way_down(engine: &Engine) -> i64 {
    let lower = location(engine, "The Lower Floor");
    int(
        engine,
        &format!(
            "SELECT location_id FROM location_connections WHERE connected_location_id = {lower} \
             OR connected_location_id IN (SELECT id FROM locations WHERE parent_location_id = {lower})"
        ),
    )
}

fn entry(engine: &Engine, floor: i64) -> i64 {
    int(
        engine,
        &format!("SELECT MIN(id) FROM locations WHERE parent_location_id = {floor}"),
    )
}

#[test]
fn a_posted_guard_is_stood_on_the_way_down_when_the_floor_is_laid_out() {
    let (mut engine, story) = guarded();
    let floor = location(&engine, "The Upper Floor");
    assert_eq!(
        brask(&engine)[..3],
        ["null", "1", &floor.to_string()],
        "absent, and posted to the floor, until it is laid out"
    );
    let game = engine.start(story).unwrap();
    submitted(
        &mut engine,
        game,
        "/move The Upper Floor",
        vec![place(), room(), arrival()],
    );

    let stood = brask(&engine);
    let down = way_down(&engine);
    assert_ne!(down, entry(&engine, floor));
    assert_eq!(
        stood[..3],
        [down.to_string(), "0".into(), floor.to_string()],
        "in the way-down room, no longer absent, and still posted"
    );
    let (x, y): (i64, i64) = (stood[3].parse().unwrap(), stood[4].parse().unwrap());
    let bounds: Vec<i64> = rows(
        &engine,
        &format!("SELECT x, y, width, depth FROM locations WHERE id = {down}"),
    )
    .pop()
    .unwrap()
    .iter()
    .map(|side| side.parse().unwrap())
    .collect();
    assert!(
        (bounds[0]..bounds[0] + bounds[2]).contains(&x),
        "{stood:?} in {bounds:?}"
    );
    assert!(
        (bounds[1]..bounds[1] + bounds[3]).contains(&y),
        "{stood:?} in {bounds:?}"
    );
}

#[test]
fn a_guard_is_stood_once_for_the_world_and_never_again() {
    let (mut engine, story) = guarded();
    let first = engine.start(story).unwrap();
    submitted(
        &mut engine,
        first,
        "/move The Upper Floor",
        vec![place(), room(), arrival()],
    );
    let stood = brask(&engine);

    // Another game walks the floor as it was laid out: he is where he was.
    let second = engine.start(story).unwrap();
    submitted(
        &mut engine,
        second,
        "/move The Upper Floor room 1",
        vec![arrival()],
    );
    assert_eq!(brask(&engine), stood);

    // The file writes him absent again after the layout: no game's walk
    // onto the floor stands him anywhere a second time.
    execute(
        &engine,
        "UPDATE characters SET location_id = NULL, deliberately_absent = 1, x = NULL, y = NULL \
         WHERE fullname = 'Brask the Stairwarden';",
    );
    let third = engine.start(story).unwrap();
    submitted(
        &mut engine,
        third,
        "/move The Upper Floor room 1",
        vec![arrival()],
    );
    assert_eq!(
        played(&mut engine, first, "/move The Upper Floor room 2"),
        None
    );
    assert_eq!(brask(&engine)[..3], ["null", "1", &stood[2]]);
}

#[test]
fn a_posted_guard_who_already_stands_somewhere_is_left_there() {
    let (mut engine, story) = guarded();
    let landing = location(&engine, "The Landing");
    execute(
        &engine,
        &format!(
            "UPDATE characters SET location_id = {landing}, deliberately_absent = 0 \
             WHERE fullname = 'Brask the Stairwarden';"
        ),
    );
    let game = engine.start(story).unwrap();
    submitted(
        &mut engine,
        game,
        "/move The Upper Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(brask(&engine)[..2], [landing.to_string(), "0".into()]);
}

#[test]
fn on_a_floor_with_no_way_down_a_guard_stands_in_a_room_off_the_entry() {
    let (mut engine, story) = guarded();
    let floor = location(&engine, "The Upper Floor");
    execute(
        &engine,
        &format!("UPDATE floors SET down_location_id = NULL WHERE location_id = {floor};"),
    );
    let game = engine.start(story).unwrap();
    submitted(
        &mut engine,
        game,
        "/move The Upper Floor",
        vec![place(), room(), arrival()],
    );
    let rooms = rows(
        &engine,
        &format!("SELECT id FROM locations WHERE parent_location_id = {floor} ORDER BY id"),
    );
    assert_eq!(rooms.len(), 3);
    let stood = brask(&engine);
    assert_ne!(stood[0], rooms[0][0], "never the entry");
    assert!(rooms.iter().any(|room| room[0] == stood[0]), "{stood:?}");
}

/// The level, hit die and abilities of the person born on `floor`'s room
/// of A Deeper Floor, with the floors' monster levels set as `levels`.
fn born_on(floor: &str, levels: &str) -> Vec<String> {
    let (mut engine, story) = opened("a-deeper-floor", "A Deeper Floor");
    execute(&engine, levels);
    let game = engine.start(story).unwrap();
    submitted(
        &mut engine,
        game,
        &format!("/move {floor}"),
        vec![place(), room(), arrival()],
    );
    let room = entry(&engine, location(&engine, floor));
    rows(
        &engine,
        &format!(
            "SELECT level, hit_die, strength, dexterity, will FROM characters \
             WHERE location_id = {room}"
        ),
    )
    .pop()
    .expect("one person is born there")
}

#[test]
fn everybody_born_on_a_floor_takes_its_monster_level_and_nothing_else_of_it() {
    let as_loaded = "";
    let deep = born_on("The Bottom Floor", as_loaded);
    assert_eq!(deep[0], "4");
    let unleveled = born_on(
        "The Bottom Floor",
        "UPDATE floors SET monster_level = NULL;",
    );
    assert_eq!(unleveled[0], "1", "no monster level is the starting level");
    assert_eq!(
        deep[1..],
        unleveled[1..],
        "the hit die and abilities are rolled as they would have been"
    );
    assert_eq!(born_on("The Upper Floor", as_loaded)[0], "1");
    assert_eq!(
        born_on(
            "The Upper Floor",
            "UPDATE floors SET monster_level = 7 WHERE location_id = \
             (SELECT id FROM locations WHERE name = 'The Upper Floor');"
        )[0],
        "7"
    );
}
