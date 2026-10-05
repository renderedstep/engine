//! Bodies and what a turn may do to them: a sheet with no stat block or no
//! abilities, the mark a blow leaves, death on the story's clock, foes in
//! one room, a thing burned, a thing read with no words yet, the world's
//! mechanics caught up, an arrival from where the party actually stands,
//! and a step that fails inside its own commit.

use renderedstep_engine::engine::{Engine, Error};
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::outcome::{Named, Outcome};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use std::path::Path;

fn world(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("parity/worlds")
        .join(format!("{name}.sql"));
    std::fs::read_to_string(path).expect("a world fixture")
}

/// The world in `file`, with `sql` run over it, and a game of the story
/// titled `title` started in it.
fn begun(file: &str, title: &str, sql: &str) -> (Engine, i64) {
    let mut engine = open_world(&format!("{}{sql}", world(file))).expect("the world opens");
    let game = started(&mut engine, title);
    (engine, game)
}

/// A new game of the story titled `title`.
fn started(engine: &mut Engine, title: &str) -> i64 {
    let story = engine
        .story_titled(&format!("{title}{TITLE_SUFFIX}"))
        .unwrap();
    engine.start(story).unwrap()
}

/// The Ropewalk at Saltmarsh, with `sql` run over it, and a game of it
/// started on the green, where Wat Coyle stands.
fn ropewalk(sql: &str) -> (Engine, i64) {
    begun(
        "the-ropewalk-at-saltmarsh",
        "The Ropewalk at Saltmarsh",
        sql,
    )
}

fn play(engine: &mut Engine, game: i64, line: &str) -> Outcome {
    engine.play(game, line, &mut |_| {}).unwrap()
}

fn refusal(outcome: &Outcome) -> &str {
    outcome
        .report
        .refusal
        .as_deref()
        .unwrap_or_else(|| panic!("a refusal, not {:?}", outcome.report))
}

fn names(named: &[Named]) -> Vec<&str> {
    named.iter().map(|thing| thing.name.as_str()).collect()
}

fn count(engine: &Engine, sql: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

/// The one value `sql` selects, as text, or none for a null.
fn value(engine: &Engine, sql: &str) -> Option<String> {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

fn execute(engine: &Engine, sql: &str) {
    engine.store().connection().execute_batch(sql).unwrap();
}

/// Moves the story's clock: the current scene of `game` is set at `at`.
fn clock_at(engine: &Engine, game: i64, at: &str) {
    execute(
        engine,
        &format!(
            "UPDATE scenes SET story_timestamp = '{at}' \
             WHERE id = (SELECT current_scene_id FROM playthroughs WHERE id = {game});"
        ),
    );
}

#[test]
fn a_player_with_no_stat_block_has_no_body_to_harm_mend_or_swing() {
    let (mut engine, game) = ropewalk(
        "UPDATE characters SET level = NULL, hit_die = NULL WHERE fullname = 'Jory Pask';",
    );
    for (line, verb) in [("harm 2", "harm"), ("mend 2", "mend")] {
        let outcome = play(&mut engine, game, line);
        assert_eq!(
            refusal(&outcome),
            format!(
                "Jory Pask has no stat block, so there is nothing to {verb}. \
                 `rake game:backfill_stat_blocks` rolls one, offline"
            )
        );
        assert_eq!(outcome.report.change, None);
        assert_eq!(outcome.state.hp, None);
    }
    let outcome = play(&mut engine, game, "attack Wat Coyle");
    assert!(
        refusal(&outcome).starts_with("Jory Pask has no stat block, so there is no hit die"),
        "{outcome:?}"
    );
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_blows"), 0);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_vitals v JOIN characters c ON c.id = v.character_id \
             WHERE c.fullname = 'Jory Pask'"
        ),
        0,
        "nothing was written for a body with no stat block"
    );
}

#[test]
fn a_player_with_no_abilities_can_neither_be_checked_nor_throw() {
    let (mut engine, game) = ropewalk(
        "UPDATE characters SET strength = NULL, dexterity = NULL, will = NULL \
         WHERE fullname = 'Jory Pask';",
    );
    let checked = play(&mut engine, game, "check will");
    assert_eq!(
        refusal(&checked),
        "Jory Pask has no abilities, so there is nothing to check. \
         `rake game:backfill_stat_blocks` rolls them, offline"
    );
    let thrown = play(&mut engine, game, "/throw cork float at Wat Coyle");
    assert_eq!(
        refusal(&thrown),
        "Jory Pask has no abilities, so there is no strength to throw with. \
         `rake game:backfill_stat_blocks` rolls them, offline"
    );
    assert!(names(&thrown.state.carrying).contains(&"cork float"));
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_blows"), 0);
}

#[test]
fn somebody_with_no_stat_block_cannot_be_struck_and_a_thing_thrown_at_them_lands() {
    // Strength 25 passes every lift, so the throw always lands.
    let (mut engine, game) = ropewalk(
        "UPDATE characters SET level = NULL, hit_die = NULL WHERE fullname = 'Wat Coyle'; \
         UPDATE characters SET strength = 25 WHERE fullname = 'Jory Pask';",
    );
    let struck = play(&mut engine, game, "attack Wat Coyle");
    assert_eq!(
        refusal(&struck),
        "Wat Coyle has no stat block, so there is no body to hurt. \
         `rake game:backfill_stat_blocks` rolls one, offline"
    );
    let thrown = play(&mut engine, game, "/throw cork float at Wat Coyle");
    assert_eq!(
        thrown.report.change.as_deref(),
        Some("it hit Wat Coyle and is lying at their feet; there is no stat block to hurt")
    );
    assert!(names(&thrown.state.here).contains(&"cork float"));
    assert!(!names(&thrown.state.carrying).contains(&"cork float"));
    assert!(thrown.state.foes.is_empty(), "nobody was provoked");
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_blows"), 0);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_vitals v JOIN characters c ON c.id = v.character_id \
             WHERE c.fullname = 'Wat Coyle'"
        ),
        0
    );
}

#[test]
fn the_mark_a_blow_leaves_keeps_the_moment_the_fight_started() {
    // Both bodies are made sturdy enough that two blows kill nobody.
    let (mut engine, game) = ropewalk("UPDATE characters SET level = 10;");
    play(&mut engine, game, "attack Wat Coyle");
    let started = "SELECT provoked_at FROM playthrough_vitals v JOIN characters c \
                   ON c.id = v.character_id WHERE c.fullname = 'Wat Coyle'";
    let first = value(&engine, started).expect("the first blow marks him");
    clock_at(&engine, game, "2026-09-21 09:30:00");
    let outcome = play(&mut engine, game, "attack Wat Coyle");
    assert!(names(&outcome.state.foes).contains(&"Wat Coyle"));
    let moments: Vec<String> = {
        let conn = engine.store().connection();
        let mut statement = conn
            .prepare(
                "SELECT story_timestamp FROM playthrough_blows b JOIN characters c \
                 ON c.id = b.target_id WHERE c.fullname = 'Wat Coyle' ORDER BY b.id",
            )
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(moments.len(), 2);
    assert_ne!(moments[0], moments[1], "the clock moved between the blows");
    assert_eq!(value(&engine, started), Some(first.clone()));
    assert_eq!(first, moments[0]);
}

#[test]
fn a_player_killed_ends_the_game_on_the_storys_clock() {
    let (mut engine, game) = ropewalk("");
    clock_at(&engine, game, "2031-05-06 07:08:09");
    let outcome = play(&mut engine, game, "harm 99");
    assert!(outcome.state.dead);
    assert_eq!(outcome.state.hp, Some(0));
    assert_eq!(
        value(
            &engine,
            &format!("SELECT ended_at FROM playthroughs WHERE id = {game}")
        )
        .as_deref(),
        Some("2031-05-06 07:08:09")
    );
}

#[test]
fn every_foe_in_the_room_strikes_back_in_id_order() {
    // Hester Vane comes onto the green beside Wat Coyle, and both are
    // hostile. Wat's id is the lower and his name sorts after hers, so the
    // order the blows land in is the ids' and not the names'. The player is
    // made sturdy enough to take both.
    let (mut engine, game) = ropewalk(
        "UPDATE characters SET hostile = 1, location_id = \
         (SELECT id FROM locations WHERE name = 'Saltmarsh Green') \
         WHERE fullname IN ('Wat Coyle', 'Hester Vane'); \
         UPDATE characters SET level = 10 WHERE fullname = 'Jory Pask';",
    );
    let outcome = play(&mut engine, game, "/drop hank of twine");
    let answered: Vec<&str> = outcome
        .report
        .note
        .iter()
        .filter_map(|note| note.strip_prefix("answered: "))
        .collect();
    assert_eq!(answered.len(), 2, "{:?}", outcome.report.note);
    assert!(answered[0].starts_with("Wat Coyle "), "{answered:?}");
    assert!(answered[1].starts_with("Hester Vane "), "{answered:?}");
    let attackers: Vec<i64> = {
        let conn = engine.store().connection();
        let mut statement = conn
            .prepare("SELECT attacker_id FROM playthrough_blows ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(attackers.len(), 2);
    assert!(attackers[0] < attackers[1], "{attackers:?}");
}

#[test]
fn burning_takes_a_carried_firestarter_and_a_combustible_thing_and_keeps_the_tool() {
    // A tinderbox and a folded note lie on the green; Wat Coyle is sent up
    // the ropewalk so that nobody else lays a hand on either.
    let (mut engine, game) = ropewalk(
        "INSERT INTO items (name, location_id, combustible, use_kind, bulk, created_at, \
         updated_at) VALUES \
         ('tinderbox', 1000000001, 0, 'firestarter', 'handy', '2026-09-21 07:00:00', \
          '2026-09-21 07:00:00'), \
         ('folded note', 1000000001, 1, 'ordinary', 'light', '2026-09-21 07:00:00', \
          '2026-09-21 07:00:00'); \
         UPDATE characters SET location_id = \
         (SELECT id FROM locations WHERE name = 'The Ropewalk') WHERE fullname = 'Wat Coyle';",
    );
    let note = "SELECT disposition FROM items WHERE name = 'folded note' AND playthrough_id = ";
    let unlit = play(&mut engine, game, "/burn folded note with tinderbox");
    assert!(unlit.report.refusal.is_some(), "{unlit:?}");
    assert_eq!(unlit.report.change, None);

    play(&mut engine, game, "take tinderbox");
    let stone = play(&mut engine, game, "/burn hank of twine with tinderbox");
    assert!(stone.report.refusal.is_some(), "{stone:?}");
    assert_eq!(stone.report.change, None);

    let scenes = count(&engine, "SELECT COUNT(*) FROM scenes");
    let burned = play(&mut engine, game, "/burn folded note with tinderbox");
    assert_eq!(
        burned.report.change.as_deref(),
        Some(
            "You burned folded note using tinderbox. The burned item is gone; you still \
             carry tinderbox."
        )
    );
    assert_eq!(
        value(&engine, &format!("{note}{game}")).as_deref(),
        Some("burned")
    );
    assert!(!names(&burned.state.here).contains(&"folded note"));
    assert!(!names(&burned.state.carrying).contains(&"folded note"));
    for kept in ["tinderbox", "hank of twine", "cork float"] {
        assert!(names(&burned.state.carrying).contains(&kept), "{kept}");
    }
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM scenes"), scenes);

    let again = play(&mut engine, game, "/burn folded note with tinderbox");
    assert!(again.report.refusal.is_some(), "{again:?}");
    assert_eq!(again.report.change, None);

    assert_eq!(
        value(
            &engine,
            "SELECT disposition FROM items WHERE name = 'folded note' AND playthrough_id IS NULL"
        )
        .as_deref(),
        Some("intact"),
        "the world's own note is never burned"
    );
    let other = started(&mut engine, "The Ropewalk at Saltmarsh");
    assert_eq!(
        value(&engine, &format!("{note}{other}")).as_deref(),
        Some("intact")
    );
    let there = engine.read(other, Vec::new()).unwrap();
    assert!(names(&there.state.here).contains(&"folded note"));
}

#[test]
fn a_readable_thing_with_no_words_yet_is_refused_offline_and_left_blank() {
    let (mut engine, game) = begun(
        "the-unrecorded-hour",
        "The Unrecorded Hour",
        "UPDATE items SET readable = 1, inscription = NULL WHERE name = 'ward stamp';",
    );
    let outcome = play(&mut engine, game, "read the ward stamp");
    assert!(
        refusal(&outcome).contains("the records do not hold the words yet"),
        "{outcome:?}"
    );
    assert_eq!(outcome.report.change, None);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM items WHERE name = 'ward stamp' AND inscription IS NULL"
        ),
        2,
        "neither the world's stamp nor this game's copy has words"
    );
}

#[test]
fn every_mechanic_a_story_has_is_caught_up() {
    // The story opens at 23:00; with the clock at half past midnight, the
    // nightly rearrangement and an hourly one both owe the midnight run.
    let (mut engine, game) = begun(
        "the-lunar-cartographer",
        "The Lunar Cartographer",
        "INSERT INTO world_mechanics (cadence, created_at, kind, name, story_id, updated_at) \
         VALUES ('hourly', '2026-08-31 23:00:00', 'shuffle_connections', 'The hourly bells', \
         1000000001, '2026-08-31 23:00:00');",
    );
    clock_at(&engine, game, "2026-09-01 00:30:00");
    play(&mut engine, game, "look");
    let runs: Vec<(String, Option<String>)> = {
        let conn = engine.store().connection();
        let mut statement = conn
            .prepare("SELECT name, last_run_at FROM world_mechanics ORDER BY id")
            .unwrap();
        statement
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let midnight = Some("2026-09-01 00:00:00".to_string());
    assert_eq!(
        runs,
        [
            ("The nightly rearrangement".to_string(), midnight.clone()),
            ("The hourly bells".to_string(), midnight),
        ]
    );
}

#[test]
fn a_game_whose_closing_scene_is_in_another_room_departs_from_where_it_stands() {
    // An older game: its current scene closed a fight in the yard, but the
    // party stands in the kitchen. The way from the kitchen to the loft is
    // made a long one, and the yard has no way to the loft at all, so the
    // time the walk takes says which room it was taken from.
    let (mut engine, game) = begun(
        "a-yard-before-the-winter",
        "A Yard Before the Winter",
        "UPDATE location_connections SET distance = 'across the district' \
         WHERE location_id = 1000000002 AND connected_location_id = 1000000003;",
    );
    execute(
        &engine,
        &format!(
            "INSERT INTO scenes (created_at, description, location_id, previous_scene_id, \
             resolved_action, story_id, story_timestamp, updated_at) VALUES \
             ('2026-09-12 06:06:00', 'The fight in the yard is over.', 1000000001, 1000000001, \
             'attack', 1000000001, '2026-09-12 06:06:00', '2026-09-12 06:06:00'); \
             UPDATE playthroughs SET current_scene_id = last_insert_rowid(), \
             current_location_id = 1000000002 WHERE id = {game};"
        ),
    );
    let closing = count(
        &engine,
        &format!("SELECT current_scene_id FROM playthroughs WHERE id = {game}"),
    );
    let arrival = Reply::from_value(&serde_json::json!({
        "purpose": "arrival",
        "content": {"description": "You climb into the loft.", "summary": "Into the loft."},
        "prompt_includes": ["The player has come from Kitchen."],
        "prompt_excludes": ["The player has come from Yard."],
    }))
    .unwrap();
    let mut replay = Replay::new(vec![arrival]);
    let submitted = engine
        .submit(game, "/move Loft", "loft", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(submitted.state.location.unwrap().name, "Loft");
    let minutes = renderedstep_engine::arrival::travel_minutes("across the district", "walking")
        .expect("a priced walk");
    let (previous, at): (i64, String) = engine
        .store()
        .connection()
        .query_row(
            "SELECT previous_scene_id, story_timestamp FROM scenes WHERE id = \
             (SELECT current_scene_id FROM playthroughs WHERE id = ?1)",
            [game],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(previous, closing);
    let departed = renderedstep_engine::clock::parse("2026-09-12 06:06:00").unwrap();
    assert_eq!(
        renderedstep_engine::clock::parse(&at),
        Some(departed + (minutes * 60.0) as i64)
    );
    assert_eq!(
        value(
            &engine,
            &format!("SELECT CAST(location_id AS TEXT) FROM scenes WHERE id = {closing}")
        )
        .as_deref(),
        Some("1000000001"),
        "the closing scene stays in the yard"
    );
}

#[test]
fn a_step_that_fails_inside_its_commit_keeps_neither_the_healing_nor_its_receipt() {
    let (mut engine, game) = begun("a-dose-beyond-two-doors", "A Dose Beyond Two Doors", "");
    play(&mut engine, game, "take healing draught");
    let hurt = play(&mut engine, game, "harm 6").state.hp;
    // The write that spends the draught is refused by the database itself,
    // after the healing was already written in the same step.
    execute(
        &engine,
        "CREATE TEMP TRIGGER interrupted BEFORE UPDATE OF disposition ON items \
         WHEN NEW.disposition = 'consumed' \
         BEGIN SELECT RAISE(ABORT, 'interrupted persistence'); END;",
    );
    let mut replay = Replay::new(Vec::new());
    let failed = engine.submit(
        game,
        "/consume healing draught",
        "failed-save",
        &mut replay,
        &mut |_| {},
    );
    assert!(
        matches!(&failed, Err(Error::Database(message)) if message.contains("interrupted persistence")),
        "{failed:?}"
    );
    let after = engine.read(game, Vec::new()).unwrap();
    assert_eq!(after.state.hp, hurt);
    assert!(names(&after.state.carrying).contains(&"healing draught"));
    assert_eq!(
        value(
            &engine,
            &format!(
                "SELECT disposition FROM items WHERE name = 'healing draught' \
                 AND playthrough_id = {game}"
            )
        )
        .as_deref(),
        Some("intact")
    );
    let journal = value(
        &engine,
        "SELECT journal FROM playthrough_commands ORDER BY id DESC LIMIT 1",
    )
    .unwrap_or_default();
    assert!(!journal.contains("physical_effect"), "{journal}");
}
