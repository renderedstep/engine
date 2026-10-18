//! Breaking a part off a fixed piece: the old beech in The Furnished Rooms'
//! clearing, given two branches and a strip of bark as a kit gives a tree
//! its parts. A part is in no list a line is read against but the break
//! attempts, a failed check moves nothing, and a part that comes away is in
//! the hand -- or, brittle, may come away in pieces.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::outcome::Named;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Mode;
use std::path::Path;

/// The old beech's row in the world file, its clearing's, and the
/// protagonist's.
const BEECH: i64 = 1_000_000_007;
const CLEARING: i64 = 1_000_000_002;
const MIRA: i64 = 1_000_000_001;

/// The Furnished Rooms with the old beech's parts written into the world and
/// its protagonist given `strength`, and a game started in it and walked into
/// the clearing. At 23 no d20 fails a firm hold's check (23 less 3); at 3
/// there is nothing for a die to come up under.
fn clearing(strength: i64) -> (Engine, i64) {
    let mut sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-furnished-rooms.sql"),
    )
    .expect("the world fixture");
    for (name, hold, bulk, fragility) in [
        ("branch", "firm", "handy", "sturdy"),
        ("branch", "firm", "handy", "sturdy"),
        ("bark", "loose", "light", "brittle"),
    ] {
        sql.push_str(&format!(
            "INSERT INTO items (name, description, tier, location_id, within_id, how, hold, \
             bulk, fragility, kit_key, created_at, updated_at) VALUES ('{name}', \
             'The {name}, attached to the old beech.', 'part', {CLEARING}, {BEECH}, \
             'attached', '{hold}', '{bulk}', '{fragility}', 'forest/old beech/attached/{name}', \
             '2026-10-02 13:25:45', '2026-10-02 13:25:45');\n"
        ));
    }
    sql.push_str(&format!(
        "UPDATE items SET kit_key = 'forest/old beech' WHERE id = {BEECH};\n\
         UPDATE characters SET strength = {strength} WHERE id = {MIRA};\n"
    ));
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("The Furnished Rooms{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start_in(story, Mode::Narrated).unwrap();
    engine
        .play(playthrough, "go to The Beech Clearing", &mut |_| {})
        .unwrap();
    (engine, playthrough)
}

fn names(named: &[Named]) -> Vec<String> {
    named.iter().map(|thing| thing.name.clone()).collect()
}

/// This game's copy of each part on the beech, in the order written: its
/// name, its tier, what it is attached to, its disposition and whether it is
/// in the hand.
fn parts(engine: &Engine, playthrough: i64) -> Vec<(String, String, Option<i64>, String, bool)> {
    let connection = engine.store().connection();
    let mut statement = connection
        .prepare(
            "SELECT name, tier, within_id, disposition, \
             location_id IS NULL AND character_id IS NULL AND disposition = 'intact' \
             FROM items WHERE playthrough_id = ?1 AND kit_key LIKE 'forest/old beech/%' \
             ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([playthrough], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_part_is_nothing_to_take_and_nothing_lying_here() {
    let (mut engine, playthrough) = clearing(10);
    let looked = engine.play(playthrough, "look", &mut |_| {}).unwrap();
    assert_eq!(
        names(&looked.state.here),
        ["fallen log", "old beech", "pine cone"]
    );
    let copied = parts(&engine, playthrough);
    assert_eq!(
        copied.len(),
        3,
        "the game copies the beech's parts: {copied:?}"
    );
    assert!(copied
        .iter()
        .all(|(_, tier, within, _, _)| tier == "part" && within.is_some()));
    let taken = engine
        .play(playthrough, "take branch", &mut |_| {})
        .unwrap();
    assert!(taken.report.refusal.is_some(), "{:?}", taken.report);
    let glance = engine.glance(playthrough).unwrap();
    assert!(glance.lying_here.iter().all(|thing| thing.name != "branch"));
}

#[test]
fn a_branch_that_comes_away_is_in_the_hand_and_the_other_stays_on_the_tree() {
    let (mut engine, playthrough) = clearing(23);
    let outcome = engine
        .play(playthrough, "/break branch off old beech", &mut |_| {})
        .unwrap();
    let change = outcome.report.change.expect("a branch that came away");
    assert!(
        change.starts_with("You broke the branch off the old beech. strength -> d20("),
        "{change}"
    );
    assert!(
        change.ends_with("<= 20 PASS. You are holding it now."),
        "{change}"
    );
    assert_eq!(names(&outcome.state.carrying), ["branch"]);
    assert_eq!(
        names(&outcome.state.here),
        ["fallen log", "old beech", "pine cone"]
    );
    let copied = parts(&engine, playthrough);
    assert_eq!(
        (copied[0].1.as_str(), copied[0].2, copied[0].4),
        ("portable", None, true),
        "the first branch is in the hand: {copied:?}"
    );
    assert_eq!(copied[1].1, "part", "the other branch is still on the tree");
    let dropped = engine
        .play(playthrough, "drop branch", &mut |_| {})
        .unwrap();
    assert_eq!(
        names(&dropped.state.here),
        ["branch", "fallen log", "old beech", "pine cone"]
    );
}

#[test]
fn a_branch_that_holds_moves_nothing() {
    let (mut engine, playthrough) = clearing(3);
    let before = parts(&engine, playthrough);
    let outcome = engine
        .play(playthrough, "/break branch off old beech", &mut |_| {})
        .unwrap();
    assert_eq!(outcome.report.change, None);
    assert_eq!(outcome.report.refusal, None);
    assert_eq!(
        outcome.report.note,
        [
            "You tried to break the branch off the old beech. strength -> 3 - 3 = 0, \
          IMPOSSIBLE (no roll). It held; nothing came away."
        ]
    );
    assert!(outcome.state.carrying.is_empty());
    assert_eq!(parts(&engine, playthrough), before);
}

#[test]
fn brittle_bark_that_comes_away_comes_away_whole_or_in_pieces() {
    let (mut engine, playthrough) = clearing(23);
    let outcome = engine
        .play(playthrough, "/break the bark", &mut |_| {})
        .unwrap();
    let change = outcome.report.change.expect("bark that came away");
    let bark = parts(&engine, playthrough)
        .into_iter()
        .find(|part| part.0 == "bark")
        .unwrap();
    assert_eq!(bark.1, "portable");
    if change.ends_with("It came away in pieces; you are holding nothing new.") {
        assert_eq!(bark.3, "broken");
        assert!(outcome.state.carrying.is_empty());
    } else {
        assert!(change.ends_with("You are holding it now."), "{change}");
        assert!(bark.4);
        assert_eq!(names(&outcome.state.carrying), ["bark"]);
    }
    let again = engine
        .play(playthrough, "/break the bark", &mut |_| {})
        .unwrap();
    assert!(again.report.refusal.is_some(), "the bark is off the tree");
}

/// The narrator's words, the fact it was handed and the action the scene
/// records, for one break typed with a narrator answering `words`.
fn narrated(
    engine: &mut Engine,
    playthrough: i64,
    words: &str,
    includes: &[&str],
) -> (String, Option<String>, String) {
    let mut replay = Replay::new(vec![Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": words,
        "prompt_includes": includes,
        "prompt_excludes": ["You broke", "You tried", "strength ->"],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(
            playthrough,
            "/break branch off old beech",
            "b",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
    engine
        .store()
        .connection()
        .query_row(
            "SELECT description, engine_fact, resolved_action FROM scenes WHERE id = ?1",
            [submitted.turned.scene.unwrap()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap()
}

#[test]
fn a_break_tells_the_narrator_what_came_away_and_what_is_left() {
    let (mut engine, playthrough) = clearing(23);
    let (words, fact, action) = narrated(
        &mut engine,
        playthrough,
        "A branch cracks away in your hands.",
        &[
            "The player is carrying: branch.",
            "What has ALREADY happened, recorded by the game: ON THIS TURN, and not before it, \
             Mira Vell broke a branch off the old beech. It was a strength check, a d20 under 23 \
             less 3 for a firm hold: it came up ",
            ", and passed. The branch is in their hands now. The old beech has one branch left \
             that could be broken off. Do not write anything else coming away, being picked up \
             or being carried.\nNarrate it as done.",
            "The physical result above belongs to this exact typed command.",
        ],
    );
    assert_eq!(words, "A branch cracks away in your hands.");
    assert!(fact.as_deref().is_some_and(|fact| fact.starts_with(
        "ON THIS TURN, and not before it, Mira Vell broke a branch off the old beech."
    )));
    assert_eq!(action, "break");
}

#[test]
fn a_break_resumed_after_its_effect_tells_the_narrator_the_check_it_threw() {
    let (mut engine, playthrough) = clearing(23);
    let mut nobody = Replay::new(Vec::new());
    let stopped = engine.submit_stopping(
        playthrough,
        "/break branch off old beech",
        "b",
        &mut nobody,
        &mut |_| {},
        Some("physical_effect"),
    );
    assert!(stopped.is_err(), "the turn stops after its effect");
    let (_, resumed, _) = narrated(
        &mut engine,
        playthrough,
        "A branch cracks away in your hands.",
        &["Mira Vell broke a branch off the old beech. It was a strength check, a d20 under 23"],
    );
    let (mut engine, playthrough) = clearing(23);
    let (_, played, _) = narrated(
        &mut engine,
        playthrough,
        "A branch cracks away in your hands.",
        &[],
    );
    assert!(played.is_some());
    assert_eq!(resumed, played);
}

#[test]
fn a_break_that_holds_tells_the_narrator_nothing_came_away() {
    let (mut engine, playthrough) = clearing(3);
    let held = "ON THIS TURN, and not before it, Mira Vell tried to break a branch off the old \
                beech. It was beyond them: strength 3 less 3 for a firm hold leaves nothing to \
                roll under. The branch held: it is still on the old beech, nothing came away and \
                they are holding nothing new. Do not write it breaking, bending away or coming \
                loose.";
    let (_, fact, action) = narrated(
        &mut engine,
        playthrough,
        "The branch will not give.",
        &[held, "The player is carrying: nothing."],
    );
    assert_eq!(fact.as_deref(), Some(held));
    assert_eq!(action, "break");
}
