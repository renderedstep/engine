//! A search in a game the narrator tells: the narrator is told what it
//! found, as already done, beside the room's own lines, and the engine's
//! sentence stands only where no narrator answers.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Live, Replay, Reply, Route};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Mode;
use std::path::Path;

/// The Furnished Rooms, and a game started in its Clerk's Study, where the
/// desk is shut and nobody has searched it.
fn study() -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-furnished-rooms.sql"),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("The Furnished Rooms{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start_in(story, Mode::Narrated).unwrap();
    (engine, playthrough)
}

fn scene(engine: &Engine, scene: i64) -> (String, Option<String>, String, bool) {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT description, engine_fact, resolved_action, engine_fallback \
             FROM scenes WHERE id = ?1",
            [scene],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap()
}

/// What the narrator is told the search of the desk found.
fn found(verb: &str) -> String {
    format!(
        "ON THIS TURN, and not before it, Mira Vell {verb} the desk. Inside it they found the \
         sheaf of papers and the letter, and nothing else: those were not there to be seen \
         before this turn, and now they are lying here and takeable. The desk now stands open. \
         Nothing was picked up: finding them is what has just happened and it is what to \
         narrate. Do not write anything else being found, taken out or carried."
    )
}

#[test]
fn a_search_tells_the_narrator_what_it_found() {
    let (mut engine, playthrough) = study();
    let mut replay = Replay::new(vec![Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You ease the drawers open.",
        "prompt_includes": [
            "Lying here, and takeable: ward stamp, potted fern, lump of coal, sheaf of papers, letter.",
            format!("What has ALREADY happened, recorded by the game: {}\nNarrate it as done.", found("searched")),
            "The physical result above belongs to this exact typed command.",
        ],
        "prompt_excludes": ["You searched the desk"],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(playthrough, "/search desk", "s", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    let (words, fact, action, fallback) = scene(&engine, submitted.turned.scene.unwrap());
    assert_eq!(words, "You ease the drawers open.");
    assert_eq!(fact, Some(found("searched")));
    assert_eq!(action, "search");
    assert!(!fallback);
}

#[test]
fn with_no_model_access_a_search_is_told_in_the_engines_own_words() {
    let (mut engine, playthrough) = study();
    let mut live = Live::new(Route::None);
    let submitted = engine
        .submit(playthrough, "/open desk", "o", &mut live, &mut |_| {})
        .unwrap();
    assert!(submitted.turned.setup);
    let (words, fact, action, fallback) = scene(&engine, submitted.turned.scene.unwrap());
    assert_eq!(
        words,
        "You opened the desk. In it you found the sheaf of papers and the letter, which lie \
         here now; you have not picked anything up."
    );
    assert_eq!(fact, Some(found("opened")));
    assert_eq!(action, "search");
    assert!(fallback);
}
