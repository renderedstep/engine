//! A game nobody is playing, or one standing nowhere: a line that needs a
//! pair of hands is refused before any model is asked, and leaves no scene
//! and no story time behind; a line that named nothing is still answered for
//! its reading first; and a foe has nobody to strike back at.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::Replay;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use std::path::Path;

/// A Turn at the Gate with nobody marked as the player, and a game started
/// in it.
fn castless() -> (Engine, i64) {
    gate("UPDATE characters SET is_protagonist = 0;")
}

/// A Turn at the Gate with `sql` run over it, and a game started in it.
fn gate(sql: &str) -> (Engine, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-turn-at-the-gate.sql"),
    )
    .expect("the gate's world");
    let mut engine = open_world(&format!("{world}{sql}")).unwrap();
    let story = engine
        .story_titled(&format!("A Turn at the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    (engine, game)
}

fn count(engine: &Engine, sql: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

/// The story's clock: its latest scene's moment.
fn clock(engine: &Engine) -> Option<String> {
    engine
        .store()
        .connection()
        .query_row("SELECT MAX(story_timestamp) FROM scenes", [], |row| {
            row.get(0)
        })
        .unwrap()
}

#[test]
fn a_take_with_nobody_to_take_it_is_refused_and_leaves_no_scene_and_no_story_time() {
    let (mut engine, game) = castless();
    let scenes = count(&engine, "SELECT COUNT(*) FROM scenes");
    let before = clock(&engine);

    let mut nobody_asked = Replay::new(Vec::new());
    let submitted = engine
        .submit(
            game,
            "/take red coin",
            "one",
            &mut nobody_asked,
            &mut |_| {},
        )
        .unwrap();
    nobody_asked.finish().unwrap();

    let refusal = submitted.turned.refusal.expect("the take is refused");
    assert_eq!(refusal.kind, "unplayable");
    assert!(
        refusal.fact.contains("no player character yet"),
        "{}",
        refusal.fact
    );
    assert_eq!(submitted.turned.scene, None);
    assert!(submitted.state.carrying.is_empty());
    assert!(submitted
        .state
        .here
        .iter()
        .any(|thing| thing.name == "red coin"));
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM scenes"), scenes);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM scenes WHERE resolved_action = 'take'"
        ),
        0,
        "no turn was recorded as a take that acted on nothing"
    );
    assert_eq!(clock(&engine), before);
}

#[test]
fn a_take_that_named_nothing_is_refused_for_its_reading_before_the_missing_player() {
    let (mut engine, game) = castless();

    let outcome = engine.play(game, "take the sky", &mut |_| {}).unwrap();

    let refusal = outcome.report.refusal.expect("the take is refused");
    assert!(refusal.contains("no thing lying here called"), "{refusal}");
    assert!(!refusal.contains("no player character"), "{refusal}");
}

/// Maren, hostile, strikes the player who stays beside her; beside a game with
/// no player, and beside a game standing nowhere, she strikes nobody.
#[test]
fn a_foe_has_nobody_to_strike_in_a_game_with_no_player_or_no_room() {
    let (mut engine, game) = gate("UPDATE characters SET hostile = 1 WHERE fullname = 'Maren';");
    engine.play(game, "/take red coin", &mut |_| {}).unwrap();
    assert!(
        count(&engine, "SELECT COUNT(*) FROM playthrough_blows") > 0,
        "a foe beside a player answers the line"
    );

    let (mut engine, game) = gate(
        "UPDATE characters SET is_protagonist = 0; \
         UPDATE characters SET hostile = 1 WHERE fullname = 'Maren';",
    );
    for line in ["/take red coin", "look", "take the sky"] {
        engine.play(game, line, &mut |_| {}).unwrap();
    }
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_blows"), 0);

    let (mut engine, game) = gate("UPDATE characters SET hostile = 1 WHERE fullname = 'Maren';");
    engine
        .store()
        .connection()
        .execute(
            "UPDATE playthroughs SET current_location_id = NULL WHERE id = ?1",
            [game],
        )
        .unwrap();
    for line in ["/take red coin", "look"] {
        engine.play(game, line, &mut |_| {}).unwrap();
    }
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_blows"), 0);
}
