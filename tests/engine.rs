//! The engine as a caller holds it: a database of its own, a line in, the
//! outcome out, and every failure a value.

use renderedstep_engine::engine::{Engine, Error};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::store::SCHEMA_VERSION;
use std::path::{Path, PathBuf};

fn world(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("parity/worlds")
        .join(format!("{name}.sql"));
    std::fs::read_to_string(path).expect("a world fixture")
}

/// A database file of its own under the system's temporary directory,
/// removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, sql: &str) -> Scratch {
        let path = std::env::temp_dir().join(format!(
            "renderedstep-engine-{name}-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        rusqlite::Connection::open(&path)
            .and_then(|conn| conn.execute_batch(sql))
            .expect("the fixture loads");
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn a_turn_is_written_to_the_database_and_read_back_by_the_next_connection() {
    let scratch = Scratch::new("turn", &world("the-quay-house"));
    let (playthrough, room) = {
        let mut engine = Engine::open(&scratch.0).expect("the database opens");
        let story = engine
            .story_titled(&format!("The Quay House{TITLE_SUFFIX}"))
            .unwrap();
        let playthrough = engine.start(story).unwrap();
        let mut chunks = Vec::new();
        let outcome = engine
            .play(playthrough, "go to The Custom House room 1", &mut |chunk| {
                chunks.push(chunk.to_string())
            })
            .unwrap();
        assert!(chunks.is_empty(), "nothing narrates with no model");
        assert_eq!(
            outcome.report.change.as_deref(),
            Some(
                "moved: The Quay -> The Custom House room 1 (a stub -- nobody has written this \
                 room, and no-model mode cannot)"
            )
        );
        (playthrough, outcome.state.location.unwrap())
    };
    let engine = Engine::open(&scratch.0).expect("the database opens again");
    let outcome = engine.read(playthrough, Vec::new()).unwrap();
    assert_eq!(outcome.state.location, Some(room));
}

#[test]
fn a_database_at_another_schema_is_refused() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world("the-quay-house")).unwrap();
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES ('29990101000000')",
        [],
    )
    .unwrap();
    match Engine::from_connection(conn) {
        Err(Error::SchemaMismatch { found, expected }) => {
            assert_eq!(found.as_deref(), Some("29990101000000"));
            assert_eq!(expected, SCHEMA_VERSION);
        }
        other => panic!("expected a schema mismatch, got {:?}", other.err()),
    }
}

#[test]
fn a_database_with_no_migrations_is_refused() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    assert!(matches!(
        Engine::from_connection(conn),
        Err(Error::SchemaMismatch { found: None, .. })
    ));
}

#[test]
fn a_playthrough_that_is_not_there_is_an_error() {
    let mut engine = open_world(&world("the-quay-house")).unwrap();
    assert_eq!(
        engine.play(42, "look", &mut |_| {}).unwrap_err(),
        Error::NoSuchPlaythrough(42)
    );
}
