//! The engine as a caller holds it: one database on a connection of its own,
//! a line in, the structured outcome out.
//!
//! Every call returns its failures as an [`Error`] value and never unwinds
//! past this module: a panic inside a turn is caught here, the turn's writes
//! are rolled back, and the caller gets [`Error::Panicked`]. That is the
//! boundary a host language calls through, where an unwinding Rust panic
//! could not be rescued.

use crate::outcome::{Outcome, State};
use crate::records::{flag, id, int, text};
use crate::store::Store;
use crate::turn::{Mechanics, Report};
use serde_json::Value;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;

/// Everything a call can fail with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The database is at a schema version this engine is not written
    /// against (`found` is none where it has no migrations table).
    SchemaMismatch {
        found: Option<String>,
        expected: String,
    },
    NoSuchPlaythrough(i64),
    NoSuchStory(String),
    /// SQLite refused a statement, or a row the engine needed is missing.
    Database(String),
    /// The line reached a rule this engine does not play yet. Nothing it
    /// wrote is kept.
    Unsupported(String),
    /// A bug in the engine. Nothing the turn wrote is kept.
    Panicked(String),
    /// A model call failed where the turn has no words of its own to put in
    /// its place. What the turn committed before the call stands.
    Model(crate::model::Failure),
    /// A submission whose last worker stopped before the journal existed,
    /// which cannot be replayed safely (`Playthrough::Command::InterruptedError`).
    Interrupted,
    /// A submission that already failed (`PreviouslyFailedError`).
    PreviouslyFailed,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SchemaMismatch { found, expected } => write!(
                f,
                "the database is at schema {}, and this engine is written against {expected}",
                found.as_deref().unwrap_or("(none)")
            ),
            Error::NoSuchPlaythrough(id) => write!(f, "there is no playthrough {id}"),
            Error::NoSuchStory(title) => write!(f, "there is no story titled {title:?}"),
            Error::Database(message) => write!(f, "database: {message}"),
            Error::Unsupported(what) => write!(f, "this engine does not play {what} yet"),
            Error::Panicked(message) => write!(f, "the engine failed: {message}"),
            Error::Model(failure) => write!(f, "the model call failed: {failure}"),
            Error::Interrupted => f.write_str("a previous worker stopped during this turn"),
            Error::PreviouslyFailed => f.write_str("this submission has already failed"),
        }
    }
}

impl std::error::Error for Error {}

/// How a submitted line ended, and the records it left.
#[derive(Clone, Debug)]
pub struct Submitted {
    pub turned: crate::turn::Turned,
    pub state: State,
}

/// One database, open for play.
pub struct Engine {
    store: Store,
}

impl Engine {
    /// Opens the database at `path` on a new connection, refusing a schema
    /// this engine is not written against.
    pub fn open(path: &Path) -> Result<Engine, Error> {
        guarded(|| Store::open(path).map(|store| Engine { store }))
    }

    /// Plays on a connection already open.
    pub fn from_connection(conn: rusqlite::Connection) -> Result<Engine, Error> {
        guarded(|| Store::from_connection(conn).map(|store| Engine { store }))
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    /// Plays one typed line in one playthrough, in one transaction, and
    /// returns what it did and the records it left.
    ///
    /// `on_chunk` receives the turn's prose as it is written. This engine
    /// plays with no model, so it narrates nothing and never calls it; the
    /// read-out in the [`Outcome`] is the whole answer.
    pub fn play(
        &mut self,
        playthrough: i64,
        line: &str,
        on_chunk: &mut dyn FnMut(&str),
    ) -> Result<Outcome, Error> {
        let _ = on_chunk;
        let store = &self.store;
        transaction(store, || {
            let mut mechanics = Mechanics::new(store, playthrough)?;
            let report = mechanics.run(line)?;
            Ok(Outcome {
                report,
                state: State::read(mechanics.records(), playthrough),
            })
        })
    }

    /// Plays a line that talks to somebody, with a fixed decision standing
    /// in for the answer a model would give (`turn::Mechanics::run_deciding`).
    /// The engine sweep plays its conversations this way.
    pub fn play_deciding(
        &mut self,
        playthrough: i64,
        line: &str,
        decision: &str,
        on_chunk: &mut dyn FnMut(&str),
    ) -> Result<Outcome, Error> {
        let _ = on_chunk;
        let store = &self.store;
        transaction(store, || {
            let mut mechanics = Mechanics::new(store, playthrough)?;
            let report = mechanics.run_deciding(line, decision)?;
            Ok(Outcome {
                report,
                state: State::read(mechanics.records(), playthrough),
            })
        })
    }

    /// Plays one submitted line the way every front end plays it
    /// (`Playthrough::Session#play` with a request token): through the
    /// models, told in prose, and kept in the submission queue, so a second
    /// delivery of the same token and line plays nothing again.
    ///
    /// There is no transaction around the line. Each effect is committed
    /// with its journal receipt as the turn reaches it, and no transaction
    /// is open while a model is asked, so a failure keeps what was committed
    /// before it, as the Ruby engine does. `on_chunk` receives prose as it
    /// streams.
    pub fn submit(
        &mut self,
        playthrough: i64,
        line: &str,
        token: &str,
        models: &mut dyn crate::model::Models,
        on_chunk: &mut dyn FnMut(&str),
    ) -> Result<Submitted, Error> {
        let store = &self.store;
        let result = guarded(|| {
            let mut turn = crate::turn::Turn::new(store, playthrough, models, on_chunk)?;
            let turned = turn.play(line, token)?;
            Ok(Submitted {
                turned,
                state: State::read(turn.records(), playthrough),
            })
        });
        if matches!(result, Err(Error::Panicked(_))) {
            store.rollback();
        }
        result
    }

    /// Accepts a line into a game's submission queue without playing it,
    /// as a browser does for a line typed while a turn is still running.
    pub fn accept(&mut self, playthrough: i64, line: &str, token: &str) -> Result<(), Error> {
        let store = &self.store;
        guarded(|| {
            let mut records = store.load()?;
            crate::command::accept(store, &mut records, playthrough, line, token).map(|_| ())
        })
    }

    /// The records with nothing played: what a console prints before the
    /// first line. Writes nothing.
    pub fn read(&self, playthrough: i64, note: Vec<String>) -> Result<Outcome, Error> {
        let store = &self.store;
        guarded(|| {
            let mechanics = Mechanics::new(store, playthrough)?;
            Ok(Outcome {
                report: Report {
                    note,
                    ..Report::default()
                },
                state: State::read(mechanics.records(), playthrough),
            })
        })
    }

    /// Starts a new playthrough of a story, as the browser does: the story's
    /// protagonist, standing in its lowest-id written room at its opening
    /// scene, with this game's copies of what they carry and of that room.
    /// Returns the new playthrough's id.
    pub fn start(&mut self, story: i64) -> Result<i64, Error> {
        let store = &self.store;
        transaction(store, || {
            let records = store.load()?;
            if records.find("stories", story).is_none() {
                return Err(Error::NoSuchStory(format!("#{story}")));
            }
            let protagonist = records
                .first("characters", |row| {
                    int(row, "story_id") == Some(story) && flag(row, "is_protagonist")
                })
                .map(id);
            let opening = records
                .first("locations", |row| {
                    int(row, "story_id") == Some(story)
                        && text(row, "detail_level") == Some("realized")
                })
                .map(id);
            let scene = records
                .first("scenes", |row| {
                    int(row, "story_id") == Some(story) && flag(row, "is_opening")
                })
                .map(id);
            let taken: Vec<&str> = records
                .table("playthroughs")
                .iter()
                .filter_map(|row| text(row, "token"))
                .collect();
            let token = token(&taken);
            let row = store.insert(
                "playthroughs",
                &[
                    ("story_id", Value::from(story)),
                    ("character_id", protagonist.map_or(Value::Null, Value::from)),
                    (
                        "current_location_id",
                        opening.map_or(Value::Null, Value::from),
                    ),
                    ("current_scene_id", scene.map_or(Value::Null, Value::from)),
                    ("token", Value::from(token)),
                ],
            )?;
            let playthrough = id(&row);
            let mut mechanics = Mechanics::new(store, playthrough)?;
            mechanics.snapshot_party()?;
            let here =
                opening.and_then(|room| mechanics.records().find("locations", room).cloned());
            mechanics.snapshot_room(here.as_ref())?;
            Ok(playthrough)
        })
    }

    /// The story with this exact title.
    pub fn story_titled(&self, title: &str) -> Result<i64, Error> {
        let store = &self.store;
        guarded(|| {
            store
                .connection()
                .query_row("SELECT id FROM stories WHERE title = ?1", [title], |row| {
                    row.get(0)
                })
                .map_err(|_| Error::NoSuchStory(title.to_string()))
        })
    }
}

/// Runs `body` in one transaction, committed only if it succeeds.
fn transaction<T>(store: &Store, body: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    guarded(|| {
        store.begin()?;
        let result = panic::catch_unwind(AssertUnwindSafe(body));
        match result {
            Ok(Ok(value)) => {
                store.commit()?;
                Ok(value)
            }
            Ok(Err(error)) => {
                store.rollback();
                Err(error)
            }
            Err(panicked) => {
                store.rollback();
                panic::resume_unwind(panicked)
            }
        }
    })
}

/// Catches a panic and returns it as an error.
fn guarded<T>(body: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    panic::catch_unwind(AssertUnwindSafe(body)).unwrap_or_else(|panicked| {
        let message = panicked
            .downcast_ref::<&str>()
            .map(|text| text.to_string())
            .or_else(|| panicked.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "a panic with no message".to_string());
        Err(Error::Panicked(message))
    })
}

/// `has_secure_token`'s shape: 32 characters of base58, unused so far.
fn token(taken: &[&str]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut state = (nanos as u64) ^ ((nanos >> 64) as u64) ^ 0x9e37_79b9_7f4a_7c15;
    state ^= (std::process::id() as u64) << 32;
    loop {
        let token: String = (0..32)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                ALPHABET[(state % ALPHABET.len() as u64) as usize] as char
            })
            .collect();
        if !taken.contains(&token.as_str()) {
            return token;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORLD: &str = include_str!("../parity/worlds/the-quay-house.sql");

    fn playthroughs(engine: &Engine) -> i64 {
        engine
            .store
            .connection()
            .query_row("SELECT COUNT(*) FROM playthroughs", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn a_panic_inside_a_turn_is_an_error_and_keeps_nothing() {
        let engine = crate::parity::open_world(WORLD).unwrap();
        let story = engine
            .story_titled(&format!("The Quay House{}", crate::parity::TITLE_SUFFIX))
            .unwrap();
        let before = playthroughs(&engine);
        let result: Result<(), Error> = transaction(&engine.store, || {
            engine.store.insert(
                "playthroughs",
                &[
                    ("story_id", Value::from(story)),
                    ("token", Value::from("a-token-that-is-rolled-back")),
                ],
            )?;
            panic!("a bug in a rule");
        });
        assert_eq!(result, Err(Error::Panicked("a bug in a rule".into())));
        assert_eq!(playthroughs(&engine), before);
    }

    #[test]
    fn an_error_inside_a_turn_keeps_nothing() {
        let engine = crate::parity::open_world(WORLD).unwrap();
        let story = engine
            .story_titled(&format!("The Quay House{}", crate::parity::TITLE_SUFFIX))
            .unwrap();
        let before = playthroughs(&engine);
        let result: Result<(), Error> = transaction(&engine.store, || {
            engine.store.insert(
                "playthroughs",
                &[
                    ("story_id", Value::from(story)),
                    ("token", Value::from("another-rolled-back-token")),
                ],
            )?;
            Err(Error::Unsupported("the rest of this turn".into()))
        });
        assert!(matches!(result, Err(Error::Unsupported(_))));
        assert_eq!(playthroughs(&engine), before);
    }
}
