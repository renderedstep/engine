//! What the narrator is told of a furnished room: the pieces fixed in it on a
//! line of their own, never among the things a player could pick up, and no
//! such line at all in a room with nothing fixed in it.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::{json, Value};
use std::path::Path;

fn open(world: &str, title: &str) -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("parity/worlds/{world}.sql")),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("{title}{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    (engine, playthrough)
}

fn narrate(engine: &mut Engine, playthrough: i64, line: &str, reply: Value) {
    let mut replay = Replay::new(vec![Reply::from_value(&reply).unwrap()]);
    engine
        .submit(playthrough, line, "told", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
}

#[test]
fn a_furnished_room_tells_its_fixed_pieces_apart_from_what_lies_in_it() {
    let (mut engine, playthrough) = open("the-furnished-rooms", "The Furnished Rooms");
    narrate(
        &mut engine,
        playthrough,
        "/take ward stamp",
        json!({
            "purpose": "narration",
            "content": "You lift the stamp off the desk.",
            "prompt_includes": [
                "Fixed here, and not takeable: desk, windowsill, coal scuttle.",
                "Lying here, and takeable: potted fern, lump of coal.",
            ],
            "prompt_excludes": ["Lying here, and takeable: desk"],
        }),
    );
}

#[test]
fn a_room_with_nothing_fixed_in_it_has_no_line_for_it() {
    let (mut engine, playthrough) = open(
        "a-clerk-with-somewhere-to-be",
        "A Clerk With Somewhere To Be",
    );
    narrate(
        &mut engine,
        playthrough,
        "/examine quarter receipt",
        json!({
            "purpose": "narration",
            "content": "You look.",
            "prompt_includes": ["Lying here, and takeable: "],
            "prompt_excludes": ["Fixed here"],
        }),
    );
}
