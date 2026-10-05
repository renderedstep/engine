//! What a story's arc reaches and what it records: the beats a game reaches
//! off its own rows, the ending its outcome rules choose when the last beat
//! lands, and the one event a game that ends short of the arc leaves behind.
//!
//! The Iron Gate Descends is walked as `an-ending-with-words` and
//! `an-arc-that-ends-the-other-way` walk it, and The Lunar Cartographer as
//! `an-ending-taken-under-his-hands` does, each with its world's rows edited
//! first; a beat that waits on a word with somebody is bound at the gate of
//! A Conversation at the Gate. No walk here moves the story clock on its
//! own, so a test that needs time to pass writes the scene that says it has.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{load_script, open_world, TITLE_SUFFIX};
use serde_json::json;
use std::path::Path;

/// The Church's Decree, The Iron Gate Descends' one arc.
const DECREE: i64 = 1_000_000_001;
const THE_DRY_CELL: i64 = 1_000_000_014;
const WARREN_ROOM_2: &str = "Blackfang Warren room 2";

/// Marek Sollen, the Ringer, whom the Lunar Cartographer's second ending
/// names.
const THE_RINGER: i64 = 1_000_000_003;
/// The Fixed Bearing's third beat: the bearing book taken off the boards.
const THE_BOOK_BEAT: i64 = 1_000_000_003;

/// Maren, whom a game at the gate talks to.
const MAREN: i64 = 1_000_000_002;
/// Orrin Vale, standing beside her, written in by [`after_talking_to_maren`].
const ORRIN: i64 = 1_000_000_008;

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

/// A world fixture with `sql` run over it, and the story of `title` in it.
fn world(name: &str, title: &str, sql: &str) -> (Engine, i64) {
    let engine = open_world(&format!(
        "{}{sql}",
        read(&format!("parity/worlds/{name}.sql"))
    ))
    .unwrap();
    let story = engine
        .story_titled(&format!("{title}{TITLE_SUFFIX}"))
        .unwrap();
    (engine, story)
}

/// The Iron Gate Descends with `sql` run over it, and a game started.
fn the_iron_gate(sql: &str) -> (Engine, i64) {
    let (mut engine, story) = world("the-iron-gate-descends", "The Iron Gate Descends", sql);
    let game = engine.start(story).unwrap();
    (engine, game)
}

/// The lines a script types, in order.
fn lines(script: &str) -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("parity/scripts/{script}.yml"));
    load_script(&path)
        .unwrap()
        .steps
        .into_iter()
        .filter_map(|step| step.typed)
        .collect()
}

/// The lines a script types before it first types `line`.
fn before(script: &str, line: &str) -> Vec<String> {
    let all = lines(script);
    let at = all
        .iter()
        .position(|typed| typed == line)
        .unwrap_or_else(|| panic!("{script} types {line:?}"));
    all[..at].to_vec()
}

fn play(engine: &mut Engine, game: i64, lines: &[String]) {
    for line in lines {
        engine.play(game, line, &mut |_| {}).unwrap();
    }
}

fn play_line(engine: &mut Engine, game: i64, line: &str) {
    engine.play(game, line, &mut |_| {}).unwrap();
}

fn sql(engine: &Engine, sql: &str) {
    engine.store().connection().execute_batch(sql).unwrap();
}

fn count(engine: &Engine, query: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(query, [], |row| row.get(0))
        .unwrap()
}

/// Writes a scene `minutes` after the story's start in the room the game
/// stands in, and makes it the game's current one: the clock as a turn
/// that took that long would leave it.
fn pass_to(engine: &Engine, game: i64, minutes: i64) {
    let conn = engine.store().connection();
    conn.execute(
        "INSERT INTO scenes (story_id, location_id, previous_scene_id, description, story_timestamp, engine_fallback, is_opening, created_at, updated_at)
         SELECT s.id, p.current_location_id, p.current_scene_id, 'Time passes.', datetime(s.start_time, '+' || ?2 || ' minutes'), 0, 0, s.start_time, s.start_time
         FROM playthroughs p JOIN stories s ON s.id = p.story_id WHERE p.id = ?1",
        [game, minutes],
    )
    .unwrap();
    conn.execute(
        "UPDATE playthroughs SET current_scene_id = last_insert_rowid() WHERE id = ?1",
        [game],
    )
    .unwrap();
}

/// The name of the ending this game reached, if any.
fn ending(engine: &Engine, game: i64) -> Option<String> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT o.name FROM playthrough_endings e JOIN quest_outcomes o ON o.id = e.quest_outcome_id
             WHERE e.playthrough_id = ?1",
        )
        .unwrap();
    let names: Vec<String> = statement
        .query_map([game], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert!(names.len() <= 1, "one ending at most: {names:?}");
    names.into_iter().next()
}

/// The positions of the beats this game reached, lowest first.
fn beats(engine: &Engine, game: i64) -> Vec<i64> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT s.position FROM playthrough_beats b JOIN quest_steps s ON s.id = b.quest_step_id
             WHERE b.playthrough_id = ?1 ORDER BY s.position",
        )
        .unwrap();
    statement
        .query_map([game], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn ended(engine: &Engine, game: i64) -> bool {
    count(
        engine,
        &format!("SELECT COUNT(*) FROM playthroughs WHERE id = {game} AND ended_at IS NOT NULL"),
    ) == 1
}

/// Every quest event: `(playthrough_id, world_mechanic_id, scheduled, summary)`.
fn quest_events(engine: &Engine) -> Vec<(Option<i64>, Option<i64>, bool, String)> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT playthrough_id, world_mechanic_id, scheduled_for IS NOT NULL, summary
             FROM world_events WHERE source = 'quest' ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// An outcome row on the Decree.
fn outcome(id: i64, name: &str, condition: &str, minutes: &str) -> String {
    format!(
        "INSERT INTO quest_outcomes (id, condition, created_at, is_default, minutes, name, quest_id, summary, updated_at)
         VALUES ({id}, {condition}, '2026-10-02', 0, {minutes}, '{name}', {DECREE}, 'It ends as {name}.', '2026-10-02');"
    )
}

/// The Iron Gate walked as `script` to the dry cell, with `sql` run over
/// the world and the clock moved to `minutes` after the start just before
/// the last step in (or not moved at all); the ending reached.
fn the_cell_reached(script: &str, sql: &str, minutes: Option<i64>) -> Option<String> {
    let (mut engine, game) = the_iron_gate(sql);
    let all = lines(script);
    let last = before(script, "go to the dry cell").len();
    play(&mut engine, game, &all[..last]);
    if let Some(minutes) = minutes {
        pass_to(&engine, game, minutes);
    }
    play(&mut engine, game, &all[last..]);
    assert_eq!(beats(&engine, game), [1, 2, 3]);
    ending(&engine, game)
}

/// Every beat reached with no ending written for the arc to reach: the game
/// goes on, and the world hears nothing of it.
#[test]
fn a_finished_arc_with_no_ending_to_reach_closes_nothing() {
    let (mut engine, game) = the_iron_gate("DELETE FROM quest_outcomes;");
    play(&mut engine, game, &lines("an-ending-with-words"));
    assert_eq!(beats(&engine, game), [1, 2, 3]);
    assert_eq!(ending(&engine, game), None);
    assert!(!ended(&engine, game));
    assert!(quest_events(&engine).is_empty());
}

/// An ending with no rule and no default is never chosen, even written
/// before the default.
#[test]
fn an_ending_nothing_selects_is_never_reached_whatever_order_it_is_written_in() {
    let unreachable = outcome(1_000_000_000, "unreachable", "NULL", "NULL");
    assert_eq!(
        the_cell_reached("an-ending-with-words", &unreachable, None).as_deref(),
        Some("rescued")
    );
}

/// `slower_than` holds once the game's clock is past the start and the
/// budget, and not on the stroke of it.
#[test]
fn a_game_past_its_budget_reaches_the_slower_ending_and_one_on_the_stroke_does_not() {
    let slow = outcome(1_000_000_003, "too-slow", "'slower_than'", "120");
    assert_eq!(
        the_cell_reached("an-ending-with-words", &slow, Some(121)).as_deref(),
        Some("too-slow")
    );
    assert_eq!(
        the_cell_reached("an-ending-with-words", &slow, Some(120)).as_deref(),
        Some("rescued")
    );
    assert_eq!(
        the_cell_reached("an-ending-with-words", &slow, None).as_deref(),
        Some("rescued")
    );
}

/// When two rules hold, the one the world wrote first is the ending,
/// whichever kind of rule it is.
#[test]
fn the_first_rule_the_world_wrote_wins_when_two_hold() {
    let script = "an-arc-that-ends-the-other-way";
    let late_and_wrong_way = |id| {
        let (mut engine, game) = the_iron_gate(&outcome(id, "too-slow", "'slower_than'", "60"));
        let all = lines(script);
        play(&mut engine, game, &all[..all.len() - 1]);
        pass_to(&engine, game, 300);
        play(&mut engine, game, &all[all.len() - 1..]);
        assert_eq!(beats(&engine, game), [1, 2, 3]);
        ending(&engine, game)
    };
    assert_eq!(
        late_and_wrong_way(1_000_000_000).as_deref(),
        Some("too-slow")
    );
    assert_eq!(
        late_and_wrong_way(1_000_000_003).as_deref(),
        Some("too-late")
    );
}

/// A beat that waits on time is reached once this game's clock stands at
/// the start plus its minutes, and not a minute before.
#[test]
fn a_time_passed_beat_is_reached_once_the_games_clock_comes_to_it() {
    let wait = format!(
        "INSERT INTO quest_steps (id, created_at, minutes, position, quest_id, summary, trigger_kind, updated_at)
         VALUES (1000000004, '2026-10-02', 40, 4, {DECREE}, 'Wait out the guards.', 'time_passed', '2026-10-02');"
    );
    let (mut engine, game) = the_iron_gate(&wait);
    play(
        &mut engine,
        game,
        &before("an-ending-with-words", "talk to the prince"),
    );
    assert_eq!(beats(&engine, game), [1, 2, 3]);
    assert_eq!(ending(&engine, game), None);

    pass_to(&engine, game, 39);
    play_line(&mut engine, game, &format!("go to {WARREN_ROOM_2}"));
    assert_eq!(beats(&engine, game), [1, 2, 3]);

    pass_to(&engine, game, 40);
    play_line(&mut engine, game, "go to the dry cell");
    assert_eq!(beats(&engine, game), [1, 2, 3, 4]);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_beats b JOIN stories s
             WHERE b.quest_step_id = 1000000004 AND b.reached_at = datetime(s.start_time, '+40 minutes')"
        ),
        1
    );
    assert_eq!(ending(&engine, game).as_deref(), Some("rescued"));
}

/// The player dies on the turn they step into the dry cell: the last beat is
/// not reached and no ending is written, though the ending the world would
/// have chosen schedules a consequence. The game's failure is one quest
/// event saying where it stopped, carrying the game and no mechanic, and the
/// lines refused after it write no second one.
#[test]
fn dying_on_the_turn_the_last_beat_would_land_fails_the_arc_once_and_schedules_nothing() {
    let (mut engine, game) = the_iron_gate(&format!(
        "UPDATE quest_outcomes SET ramification_minutes = 180, ramification_summary = 'The low door is barred.' WHERE name = 'rescued';
         UPDATE locations SET hazard = 'airless', hazard_die = 6 WHERE name = '{WARREN_ROOM_2}';"
    ));
    play(
        &mut engine,
        game,
        &before("an-ending-with-words", "go to the dry cell"),
    );
    assert_eq!(beats(&engine, game), [1, 2]);
    sql(
        &engine,
        &format!(
            "UPDATE playthrough_vitals SET hp_current = 1
             WHERE playthrough_id = {game} AND character_id = (SELECT character_id FROM playthroughs WHERE id = {game});"
        ),
    );

    play_line(&mut engine, game, "go to the dry cell");
    assert!(ended(&engine, game), "the player died on the way in");
    assert_eq!(
        count(
            &engine,
            &format!("SELECT current_location_id FROM playthroughs WHERE id = {game}")
        ),
        THE_DRY_CELL
    );
    assert_eq!(beats(&engine, game), [1, 2]);
    assert_eq!(ending(&engine, game), None);
    let failed = quest_events(&engine);
    assert_eq!(failed.len(), 1, "{failed:?}");
    let (playthrough, mechanic, scheduled, summary) = &failed[0];
    assert_eq!(
        (*playthrough, *mechanic, *scheduled),
        (Some(game), None, false)
    );
    assert!(
        summary.starts_with(
            "The Church's Decree was left unfinished: it stopped at Find the cell they are keeping him in."
        ),
        "{summary}"
    );

    for line in ["look", "go to Blackfang Warren room 2"] {
        let outcome = engine.play(game, line, &mut |_| {}).unwrap();
        assert!(outcome.report.refusal.is_some(), "{line} is refused");
    }
    assert_eq!(quest_events(&engine), failed);
}

/// The Lunar Cartographer, each of `others` played by a game of its own,
/// and then a game whose clock stands ten minutes into the story walked as
/// `an-ending-taken-under-his-hands` up to the tally.
fn up_to_the_tally(others: &[Vec<String>]) -> (Engine, i64) {
    let (mut engine, story) = world("the-lunar-cartographer", "The Lunar Cartographer", "");
    for lines in others {
        let other = engine.start(story).unwrap();
        play(&mut engine, other, lines);
    }
    let game = engine.start(story).unwrap();
    pass_to(&engine, game, 10);
    play(
        &mut engine,
        game,
        &before("an-ending-taken-under-his-hands", "take the tally"),
    );
    (engine, game)
}

/// Takes the tally, the arc's last beat, and returns the ending reached.
fn take_the_tally(engine: &mut Engine, game: i64) -> String {
    play_line(engine, game, "take the tally");
    assert_eq!(beats(engine, game), [1, 2, 3]);
    ending(engine, game).expect("the arc ended")
}

/// The ending that needs the Ringer alive at the book is reached when the
/// game's records put his death after it, and not when a hazard's toll put
/// it before.
#[test]
fn a_toll_that_killed_the_ringer_before_the_beat_is_his_death() {
    let (mut engine, game) = up_to_the_tally(&[]);
    assert_eq!(take_the_tally(&mut engine, game), "taken-under-his-hands");

    let (mut engine, game) = up_to_the_tally(&[]);
    sql(
        &engine,
        &format!(
            "INSERT INTO playthrough_tolls (character_id, created_at, damage, hazard, hp_after, location_id, playthrough_id, saved, sequence, story_timestamp, updated_at)
             SELECT {THE_RINGER}, start_time, 40, 'airless', 0, 1000000007, {game}, 0, -99, start_time, start_time
             FROM stories;"
        ),
    );
    assert_eq!(take_the_tally(&mut engine, game), "the-bearing-holds");
}

/// A Ringer at zero hit points with no blow or toll saying when he got
/// there died at a moment nobody can read, and is not alive at the beat.
#[test]
fn a_ringer_at_zero_with_no_record_of_dying_is_not_alive_at_the_beat() {
    let (mut engine, game) = up_to_the_tally(&[]);
    assert_eq!(
        count(
            &engine,
            &format!(
                "SELECT hp_current FROM playthrough_vitals WHERE playthrough_id = {game} AND character_id = {THE_RINGER}"
            )
        ),
        0,
        "the walk kills the Ringer before the tally"
    );
    sql(
        &engine,
        &format!(
            "DELETE FROM playthrough_blows WHERE playthrough_id = {game} AND target_id = {THE_RINGER} AND hp_after = 0;"
        ),
    );
    assert_eq!(take_the_tally(&mut engine, game), "the-bearing-holds");
}

/// The Ringer killed in another game before this one took the book is
/// still standing in this one.
#[test]
fn a_ringer_killed_in_another_game_is_alive_in_this_one() {
    let killer = before(
        "an-ending-the-ringer-did-not-stand-for",
        "take the bearing book",
    );
    let (mut engine, game) = up_to_the_tally(&[killer]);
    let other = count(&engine, "SELECT MIN(id) FROM playthroughs");
    assert_eq!(
        count(
            &engine,
            &format!(
                "SELECT COUNT(*) FROM playthrough_blows k, playthrough_beats b
                 WHERE k.playthrough_id = {other} AND k.target_id = {THE_RINGER} AND k.hp_after = 0
                   AND b.playthrough_id = {game} AND b.quest_step_id = {THE_BOOK_BEAT}
                   AND k.story_timestamp < b.reached_at"
            )
        ),
        1,
        "the other game killed him before this one took the book"
    );
    assert_eq!(take_the_tally(&mut engine, game), "taken-under-his-hands");
}

/// An arc for the gate: speak to `target`, then get into the courtyard.
fn a_word_with(target: i64) -> String {
    format!(
        "INSERT INTO quests (id, contributes, created_at, origin, premise, status, story_id, title, updated_at)
         VALUES (1000000001, 1, '2026-10-02', 'seeded', 'Open the gate.', 'open', 1000000001, 'The Closed Gate', '2026-10-02');
         INSERT INTO quest_steps (id, created_at, position, quest_id, summary, target_id, target_name, target_type, trigger_kind, updated_at)
         SELECT 1000000001, '2026-10-02', 1, 1000000001, 'Speak to ' || fullname || '.', id, fullname, 'Character', 'speak_to', '2026-10-02'
         FROM characters WHERE id = {target};
         INSERT INTO quest_steps (id, created_at, position, quest_id, summary, target_id, target_name, target_type, trigger_kind, updated_at)
         VALUES (1000000002, '2026-10-02', 2, 1000000001, 'Get into the courtyard.', 1000000002, 'Courtyard', 'Location', 'reach_location', '2026-10-02');
         INSERT INTO quest_outcomes (id, created_at, is_default, name, quest_id, summary, updated_at)
         VALUES (1000000001, '2026-10-02', 1, 'opened', 1000000001, 'The gate opens.', '2026-10-02');"
    )
}

/// The beats a game at the gate reached after talking to Maren, with the
/// arc's first beat bound to `target`.
fn after_talking_to_maren(target: i64) -> Vec<i64> {
    let somebody_else = format!(
        "INSERT INTO characters (id, age, created_at, updated_at, fullname, nickname, race_id,
           story_id, sex, level, hit_die, strength, dexterity, will, location_id)
         SELECT {ORRIN}, 45, created_at, updated_at, 'Orrin Vale', 'Orrin', race_id, story_id,
           'male', 10, 8, 12, 12, 12, location_id FROM characters WHERE id = {MAREN};"
    );
    let (mut engine, story) = world(
        "a-conversation-at-the-gate",
        "A Conversation at the Gate",
        &format!(
            "UPDATE characters SET hostile = 0 WHERE id = {MAREN};{somebody_else}{}",
            a_word_with(target)
        ),
    );
    let game = engine.start(story).unwrap();
    let reply = |value| Reply::from_value(&value).unwrap();
    let mut replay = Replay::new(vec![
        reply(json!({ "purpose": "character", "content": {
            "pre_thought": "They want the gate open.",
            "pre_feeling": "wary",
            "action": "I hear them out.",
            "post_feeling": "calmer",
            "post_thought": "Perhaps.",
            "inner_resolution": "I will think on it.",
            "engine_action": "none",
        }})),
        reply(json!({ "purpose": "interaction-narration", "content": "Maren listens." })),
    ]);
    engine
        .submit(game, "/talk Maren", "talk", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(
        count(
            &engine,
            &format!("SELECT COUNT(*) FROM interactions WHERE character_id = {MAREN}")
        ),
        1
    );
    beats(&engine, game)
}

/// A beat that waits on a word with somebody is reached by talking to
/// them, and not by talking to somebody else in the same room.
#[test]
fn speaking_to_somebody_else_does_not_reach_the_beat() {
    assert_eq!(after_talking_to_maren(MAREN), [1]);
    assert_eq!(after_talking_to_maren(ORRIN), Vec::<i64>::new());
}
