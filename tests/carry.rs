//! The doctor's carry finding over the checked-in worlds: none of them binds
//! an arc beat to a thing nothing can pick up, and The Salt Assizes does once
//! its hand-bell is made immovable.

use renderedstep_engine::carry::UNCARRIABLE;
use renderedstep_engine::parity::open_world;
use std::path::Path;

/// Every world fixture's one story.
const STORY: i64 = 1_000_000_001;
/// The Salt Assizes' hand-bell, and the beat that asks the player to hold it.
const THE_HAND_BELL: i64 = 1_000_000_001;
const THE_BELL_BEAT: i64 = 1_000_000_003;

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
