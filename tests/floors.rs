//! A floor laid out by the engine, and a doorway moved onto a place's rooms,
//! as a caller holds them: Two Floors and a Key walked into on the live path
//! with a scripted model, and the rows read back.
//!
//! The Upper Floor's row names four rooms and a way down to The Lower Floor
//! behind a stair locked with the stair key. The Old Cellar is an ordinary
//! place behind a stair locked with the cellar key that drops you on the way
//! down only.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Live, Route, Secret};
use renderedstep_engine::parameters::Parameters;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::{json, Value};
use std::path::Path;

fn world() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/two-floors-and-a-key.sql");
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

/// The place call's answer: its picks ask for something no floor is.
fn place() -> Value {
    json!({
        "description": "Black stone passages lit by nothing you can see.",
        "lore": "Cut by people nobody remembers.",
        "place_kind": "mansion",
        "parameters": {
            "inside": "a few rooms",
            "storeys_above": "two storeys up",
            "storeys_below": "deep",
            "danger": "safe",
            "gradient": "worse the deeper you go",
            "hazard": "flooded",
        },
    })
}

/// A room's answer. The live path reads an empty list as a field left out,
/// so it names one person, whom the room may or may not have a place for.
fn room() -> Value {
    json!({
        "description": "A bare stone chamber.",
        "lore": "Old.",
        "items": [],
        "people": [{
            "fullname": "Brask Holloway", "nickname": "Brask", "appearance": "A grey cloak.",
            "personality": "Quiet.", "backstory": "A delver who stayed.",
            "likes": "Silence.", "dislikes": "Lanterns.", "fears": "The stair.",
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

fn opened() -> (Engine, i64) {
    let engine = open_world(&world()).unwrap();
    let story = engine
        .story_titled(&format!("Two Floors and a Key{TITLE_SUFFIX}"))
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

/// The Upper Floor's rooms and every doorway off them, as they are written.
fn upper_floor(engine: &Engine) -> Vec<Vec<String>> {
    let floor = location(engine, "The Upper Floor");
    let mut written = rows(
        engine,
        &format!(
            "SELECT id, name, x, y, z, width, depth, danger, hazard, kind FROM locations \
             WHERE parent_location_id = {floor} ORDER BY id"
        ),
    );
    written.extend(rows(
        engine,
        &format!(
            "SELECT id, location_id, connected_location_id, barrier, key_template_id, hazard \
             FROM location_connections WHERE location_id IN \
             (SELECT id FROM locations WHERE parent_location_id = {floor}) ORDER BY id"
        ),
    ));
    written
}

#[test]
fn a_floor_is_laid_out_to_its_own_count_with_its_locked_way_down_off_the_entry() {
    let (mut engine, story) = opened();
    let game = engine.start(story).unwrap();
    assert_eq!(played(&mut engine, game, "/take stair key"), None);
    let calls = submitted(
        &mut engine,
        game,
        "/move The Upper Floor",
        vec![place(), room(), arrival()],
    );
    assert_eq!(calls, 3, "the place, its entry room and the arrival");

    let floor = location(&engine, "The Upper Floor");
    let lower = location(&engine, "The Lower Floor");
    let key = int(
        &engine,
        "SELECT id FROM items WHERE name = 'stair key' AND playthrough_id IS NULL",
    );
    let rooms = rows(
        &engine,
        &format!(
            "SELECT id, z, kind, danger, hazard FROM locations \
             WHERE parent_location_id = {floor} ORDER BY id"
        ),
    );
    assert_eq!(
        rooms.len(),
        4,
        "the floor's own count, not the picks' few rooms"
    );
    let uneasy = Parameters {
        danger: Some("uneasy".into()),
        ..Parameters::default()
    }
    .danger_for(0);
    for room in &rooms {
        assert_eq!(room[1], "0", "one storey");
        assert_eq!(room[2], "cave", "a warren's rooms");
        assert!(uneasy.contains(&room[3].as_str()), "{}", room[3]);
        assert_eq!(
            room[4], "null",
            "the floor's hazard is none, not the picks' flood"
        );
    }
    assert_eq!(
        rows(
            &engine,
            &format!("SELECT width, depth FROM locations WHERE id = {floor}")
        ),
        [["12", "12"]]
    );
    assert_eq!(
        int(
            &engine,
            &format!(
                "SELECT COUNT(*) FROM location_connections \
                 WHERE location_id = {floor} OR connected_location_id = {floor}"
            )
        ),
        0,
        "no doorway is left on the floor itself"
    );
    let entry = &rooms[0][0];
    let landing = location(&engine, "The Landing");
    assert_eq!(
        rows(
            &engine,
            &format!(
                "SELECT location_id FROM location_connections WHERE connected_location_id = {landing} \
                 AND location_id IN (SELECT id FROM locations WHERE parent_location_id = {floor})"
            )
        ),
        [[entry.clone()]],
        "the way in lands on the entry"
    );
    let down = rows(
        &engine,
        &format!(
            "SELECT location_id, connected_location_id, barrier, key_template_id \
             FROM location_connections \
             WHERE connected_location_id = {lower} OR location_id = {lower} ORDER BY id"
        ),
    );
    assert_eq!(down.len(), 2, "{down:?}");
    let way_down = &down[0][0];
    assert_ne!(way_down, entry, "the way down is never the entry");
    assert!(rooms.iter().any(|room| &room[0] == way_down));
    assert_eq!(down[1][1], *way_down);
    for side in &down {
        assert_eq!(side[2], "keyed");
        assert_eq!(side[3], key.to_string());
    }
    assert_eq!(
        played(&mut engine, game, "/move The Upper Floor room 2"),
        None
    );
}

#[test]
fn a_floor_is_laid_out_once_and_every_later_game_walks_the_stored_rooms() {
    let (mut engine, story) = opened();
    let first = engine.start(story).unwrap();
    submitted(
        &mut engine,
        first,
        "/move The Upper Floor",
        vec![place(), room(), arrival()],
    );
    let laid = upper_floor(&engine);
    let locations = int(&engine, "SELECT COUNT(*) FROM locations");
    // The world file re-seeded after the layout with other numbers: the
    // stored floor is what every game reads.
    engine
        .store()
        .connection()
        .execute_batch("UPDATE floors SET rooms = 9, danger = 'dangerous', hazard = 'flooded';")
        .unwrap();

    let second = engine.start(story).unwrap();
    let calls = submitted(
        &mut engine,
        second,
        "/move The Upper Floor room 1",
        vec![arrival()],
    );
    assert_eq!(calls, 1, "the arrival only: nothing is written again");
    assert_eq!(
        played(&mut engine, second, "/move The Upper Floor room 2"),
        None
    );
    let third = engine.start(story).unwrap();
    assert_eq!(
        played(&mut engine, third, "/move The Upper Floor room 1"),
        None
    );
    assert_eq!(played(&mut engine, first, "/move The Landing"), None);
    submitted(
        &mut engine,
        first,
        "/move The Upper Floor room 1",
        vec![arrival()],
    );

    assert_eq!(upper_floor(&engine), laid);
    assert_eq!(int(&engine, "SELECT COUNT(*) FROM locations"), locations);
}

#[test]
fn a_locked_stair_onto_a_place_keeps_its_lock_its_drop_and_every_game_s_opening() {
    let (mut engine, story) = opened();
    // A game with no model opens the stair and walks down before the cellar
    // is laid out, paying the drop on the stair as it was then.
    let early = engine.start(story).unwrap();
    assert_eq!(played(&mut engine, early, "/take cellar key"), None);
    assert_eq!(
        played(&mut engine, early, "/unlock The Old Cellar with cellar key"),
        None
    );
    assert_eq!(played(&mut engine, early, "/move The Old Cellar"), None);
    let landing = location(&engine, "The Landing");
    let cellar = location(&engine, "The Old Cellar");
    let old = rows(
        &engine,
        &format!(
            "SELECT id FROM location_connections \
             WHERE location_id = {cellar} OR connected_location_id = {cellar} ORDER BY id"
        ),
    );
    assert_eq!(old.len(), 2);
    assert_eq!(
        int(
            &engine,
            &format!("SELECT COUNT(*) FROM playthrough_tolls WHERE playthrough_id = {early}")
        ),
        1
    );

    let game = engine.start(story).unwrap();
    assert_eq!(played(&mut engine, game, "/take cellar key"), None);
    assert_eq!(
        played(&mut engine, game, "/unlock The Old Cellar with cellar key"),
        None
    );
    submitted(
        &mut engine,
        game,
        "/move The Old Cellar",
        vec![place(), room(), arrival()],
    );

    let key = int(
        &engine,
        "SELECT id FROM items WHERE name = 'cellar key' AND playthrough_id IS NULL",
    );
    let entry = int(
        &engine,
        &format!("SELECT MIN(id) FROM locations WHERE parent_location_id = {cellar}"),
    );
    let stair = |from: i64, to: i64| {
        rows(
            &engine,
            &format!(
                "SELECT id, barrier, key_template_id, hazard, hazard_die FROM location_connections \
                 WHERE location_id = {from} AND connected_location_id = {to}"
            ),
        )
        .pop()
        .expect("the stair is there")
    };
    let down = stair(landing, entry);
    let up = stair(entry, landing);
    assert_eq!(down[1..], ["keyed", &key.to_string(), "drop", "4"]);
    assert_eq!(up[1..], ["keyed", &key.to_string(), "null", "null"]);
    assert_eq!(
        int(
            &engine,
            &format!(
                "SELECT COUNT(*) FROM location_connections \
                 WHERE location_id = {cellar} OR connected_location_id = {cellar}"
            )
        ),
        0
    );
    for opener in [early, game] {
        assert_eq!(
            rows(
                &engine,
                &format!(
                    "SELECT location_connection_id FROM playthrough_passages \
                     WHERE playthrough_id = {opener} ORDER BY location_connection_id"
                )
            ),
            [[up[0].clone()], [down[0].clone()]],
            "game {opener}'s opening moved with the stair, both ways"
        );
    }
    assert_eq!(
        rows(
            &engine,
            "SELECT playthrough_id, hazard, location_connection_id FROM playthrough_tolls \
             WHERE hazard = 'drop' ORDER BY id"
        ),
        [
            [early.to_string(), "drop".into(), down[0].clone()],
            [game.to_string(), "drop".into(), down[0].clone()]
        ],
        "the drops paid before and after the layout both name the stair down"
    );

    assert_eq!(played(&mut engine, game, "/move The Landing"), None);
    assert_eq!(
        played(&mut engine, game, "/move The Old Cellar room 1"),
        None
    );
    let stranger = engine.start(story).unwrap();
    let refused = played(&mut engine, stranger, "/move The Old Cellar room 1")
        .expect("a game that never opened the stair is refused");
    assert!(refused.contains("locked"), "{refused}");
}
