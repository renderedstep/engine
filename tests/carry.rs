//! The doctor's carry finding over the checked-in worlds: none of them binds
//! an arc beat to a thing nothing can pick up, and The Salt Assizes does once
//! its hand-bell is made immovable.

use renderedstep_engine::carry::{OVER_THE_LIMIT, UNCARRIABLE};
use renderedstep_engine::parity::open_world;
use std::path::Path;

/// Every world fixture's one story.
const STORY: i64 = 1_000_000_001;
/// The Salt Assizes' hand-bell, and the beat that asks the player to hold it.
const THE_HAND_BELL: i64 = 1_000_000_001;
const THE_BELL_BEAT: i64 = 1_000_000_003;
/// The Ropewalk's protagonist, strength 18, who starts out holding a heavy
/// sinker, a hank of twine, a cork float and a tar pot.
const JORY: i64 = 1_000_000_001;

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn no_checked_in_world_asks_the_player_to_hold_what_cannot_be_carried() {
    let worlds = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds");
    let mut paths: Vec<_> = std::fs::read_dir(&worlds)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "sql"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty());
    for path in paths {
        let engine = open_world(&read(&path)).unwrap();
        assert_eq!(
            engine.carry_findings(STORY).unwrap(),
            Vec::new(),
            "{}",
            path.display()
        );
    }
}

#[test]
fn an_immovable_thing_an_arc_asks_for_is_fatal() {
    let world = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-salt-assizes.sql");
    let engine = open_world(&format!(
        "{}UPDATE items SET bulk = 'immovable' WHERE id = {THE_HAND_BELL};\n",
        read(&world)
    ))
    .unwrap();
    let found = engine.carry_findings(STORY).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    let finding = &found[0];
    assert_eq!(finding.code, UNCARRIABLE);
    assert_eq!(finding.severity, "fatal");
    assert_eq!(finding.remedy, "manual");
    assert_eq!(finding.subject, ("quest_steps", THE_BELL_BEAT));
    assert!(
        finding
            .message
            .contains("\"Assize hand-bell\", which is immovable"),
        "{}",
        finding.message
    );
}

/// What switches the gate on: the world limits carrying.
const LIMITED: &str = "UPDATE universes SET carrying = 'limited';\n";

/// The world fixture `slug` with its carrying limited.
fn limited(slug: &str) -> String {
    let world = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("parity/worlds")
        .join(format!("{slug}.sql"));
    format!("{}{LIMITED}", read(&world))
}

#[test]
fn the_three_handmade_worlds_start_nobody_over_the_limit_once_it_holds() {
    for slug in [
        "the-unrecorded-hour",
        "the-salt-assizes",
        "the-lunar-cartographer",
    ] {
        let engine = open_world(&limited(slug)).unwrap();
        assert_eq!(engine.carry_findings(STORY).unwrap(), Vec::new(), "{slug}");
    }
}

#[test]
fn a_world_that_starts_somebody_over_the_limit_warns_only_where_it_holds() {
    let world =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-ropewalk-at-saltmarsh.sql");
    let unlimited = open_world(&read(&world)).unwrap();
    assert_eq!(unlimited.carry_findings(STORY).unwrap(), Vec::new());

    let engine = open_world(&limited("the-ropewalk-at-saltmarsh")).unwrap();
    let found = engine.carry_findings(STORY).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    let finding = &found[0];
    assert_eq!(finding.code, OVER_THE_LIMIT);
    assert_eq!(finding.severity, "warning");
    assert_eq!(finding.remedy, "manual");
    assert_eq!(finding.subject, ("characters", JORY));
    assert!(
        finding.message.starts_with("Jory Pask starts out holding")
            && finding.message.contains("strength 18"),
        "{}",
        finding.message
    );
}
