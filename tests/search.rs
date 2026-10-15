//! A search in a game the narrator tells: what it finds reaches the
//! narrator as the room's own lines, with no sentence of the engine's for
//! the search itself (that wording waits on the owner), and the engine's
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

#[test]
fn a_search_tells_the_narrator_what_lies_here_and_no_sentence_of_its_own() {
    let (mut engine, playthrough) = study();
    let mut replay = Replay::new(vec![Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You ease the drawers open.",
        "prompt_includes": [
            "Lying here, and takeable: ward stamp, potted fern, lump of coal, sheaf of papers, letter.",
        ],
        "prompt_excludes": [
            "What has ALREADY happened",
            "The physical result above",
            "You searched the desk",
        ],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(playthrough, "/search desk", "s", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    let (words, fact, action, fallback) = scene(&engine, submitted.turned.scene.unwrap());
    assert_eq!(words, "You ease the drawers open.");
    assert_eq!(fact, None);
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
    assert_eq!(fact, None);
    assert_eq!(action, "search");
    assert!(fallback);
}
