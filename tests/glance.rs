//! What a front end's panels read between turns, off a database the engine
//! plays: the room, its ways out, who and what is here, and which verbs are
//! open -- each target one the turn plays, and each closed verb closed in
//! the refusal's own words.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::glance::{Glance, Verb, VERBS};
use renderedstep_engine::parity::open_world;

fn worlds() -> Vec<(String, String)> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds");
    let mut worlds: Vec<(String, String)> = std::fs::read_dir(directory)
        .expect("the world fixtures")
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let name = path.file_stem()?.to_str()?.to_string();
            Some((name, std::fs::read_to_string(&path).ok()?))
        })
        .collect();
    worlds.sort();
    worlds
}

fn stories(engine: &Engine) -> Vec<i64> {
    let conn = engine.store().connection();
    let mut statement = conn.prepare("SELECT id FROM stories ORDER BY id").unwrap();
    let ids = statement
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<Vec<i64>, _>>()
        .unwrap();
    ids
}

/// A new game of every story of every world fixture, each on a database of
/// its own, with what its glance says before anything is typed.
fn openings() -> Vec<(String, String, i64, Glance)> {
    let mut openings = Vec::new();
    for (name, sql) in worlds() {
        let probe = open_world(&sql).expect("the world loads");
        for story in stories(&probe) {
            let mut engine = open_world(&sql).unwrap();
            let Ok(playthrough) = engine.start(story) else {
                continue;
            };
            let glance = engine.glance(playthrough).expect("a glance");
            openings.push((name.clone(), sql.clone(), story, glance));
        }
    }
    assert!(!openings.is_empty(), "no world fixture opened");
    openings
}

fn verb<'g>(glance: &'g Glance, name: &str) -> &'g Verb {
    glance
        .verbs
        .iter()
        .find(|verb| verb.name == name)
        .unwrap_or_else(|| panic!("no {name} verb"))
}

#[test]
fn every_verb_in_the_closed_set_is_answered_in_order() {
    for (name, _, _, glance) in openings() {
        let names: Vec<&str> = glance.verbs.iter().map(|verb| verb.name.as_str()).collect();
        assert_eq!(names, VERBS, "{name}");
        for verb in &glance.verbs {
            assert_eq!(
                verb.available(),
                !verb.targets.is_empty(),
                "{name}: {}",
                verb.name
            );
            assert_eq!(
                verb.aims.is_some(),
                verb.name == "throw",
                "{name}: {}",
                verb.name
            );
        }
        assert_eq!(verb(&glance, "move").word.as_deref(), Some("go"));
        assert_eq!(verb(&glance, "use").word, None);
    }
}

#[test]
fn every_offered_target_is_one_a_slashed_line_plays() {
    for (name, sql, story, glance) in openings() {
        for (action, word) in [("move", "go"), ("take", "take"), ("drop", "drop")] {
            for target in &verb(&glance, action).targets {
                let mut engine = open_world(&sql).unwrap();
                let playthrough = engine.start(story).unwrap();
                let line = format!("/{word} {}", target.name);
                let outcome = engine.play(playthrough, &line, &mut |_| {}).unwrap();
                assert_eq!(outcome.report.refusal, None, "{name}: {line}");
            }
        }
        // An offer is a conversation, which a line with no model does not
        // play; every other attempt is written by the engine alone.
        for target in &verb(&glance, "use").targets {
            let Some(line) = target
                .line
                .as_ref()
                .filter(|_| target.kind.as_deref() != Some("offer"))
            else {
                continue;
            };
            let mut engine = open_world(&sql).unwrap();
            let playthrough = engine.start(story).unwrap();
            let outcome = engine.play(playthrough, line, &mut |_| {}).unwrap();
            assert_eq!(outcome.report.refusal, None, "{name}: {line}");
        }
    }
}

#[test]
fn the_panels_are_the_room_the_turn_reads() {
    for (name, _, _, glance) in openings() {
        let menu = &glance.slash_menu;
        let completes = |word: &str| {
            menu.targets
                .iter()
                .find(|(offered, _)| offered == word)
                .map(|(_, names)| names.clone())
                .unwrap_or_default()
        };
        let names = |names: Vec<String>| -> Vec<String> {
            let mut unique: Vec<String> = Vec::new();
            for name in names {
                if !unique.contains(&name) {
                    unique.push(name);
                }
            }
            unique
        };
        let exits = glance.exits.iter().map(|exit| exit.name.clone()).collect();
        assert_eq!(completes("go"), names(exits), "{name}");
        let lying = glance
            .lying_here
            .iter()
            .map(|thing| thing.name.clone())
            .collect();
        assert_eq!(completes("take"), names(lying), "{name}");
        assert_eq!(glance.over, glance.state.dead, "{name}");
        assert_eq!(
            glance.here.as_ref().map(|here| here.name.clone()),
            glance.state.location.as_ref().map(|room| room.name.clone()),
            "{name}"
        );
    }
}

#[test]
fn a_finished_game_closes_every_verb_in_the_end_notice_words() {
    let (name, sql, story, _) = openings().into_iter().next().unwrap();
    let mut engine = open_world(&sql).unwrap();
    let playthrough = engine.start(story).unwrap();
    engine
        .store()
        .connection()
        .execute(
            "UPDATE playthroughs SET ended_at = '2026-01-01 00:00:00' WHERE id = ?1",
            [playthrough],
        )
        .unwrap();
    let glance = engine.glance(playthrough).unwrap();
    assert!(glance.over, "{name}");
    // Marked over with no ending row and nobody at zero: the records say
    // neither that the story ended nor that anybody died.
    let ended = glance.ended.clone().expect("the end notice's sentence");
    assert!(
        ended.contains("stopped before it reached an ending"),
        "{ended}"
    );
    for verb in &glance.verbs {
        assert!(!verb.available(), "{}", verb.name);
        assert!(verb.targets.is_empty(), "{}", verb.name);
        assert_eq!(
            verb.reason.as_deref(),
            Some(ended.as_str()),
            "{}",
            verb.name
        );
    }
}

#[test]
fn a_glance_writes_nothing() {
    let (_, sql, story, _) = openings().into_iter().next().unwrap();
    let mut engine = open_world(&sql).unwrap();
    let playthrough = engine.start(story).unwrap();
    let changes = |engine: &Engine| -> i64 {
        engine
            .store()
            .connection()
            .query_row("SELECT total_changes()", [], |row| row.get(0))
            .unwrap()
    };
    let before = changes(&engine);
    engine.glance(playthrough).unwrap();
    engine.glance(playthrough).unwrap();
    assert_eq!(changes(&engine), before);
}
