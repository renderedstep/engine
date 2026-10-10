//! The engine as a caller holds it: one database on a connection of its own,
//! a line in, the structured outcome out.
//!
//! Every call returns its failures as an [`Error`] value and never unwinds
//! past this module: a panic inside a turn is caught here, the turn's writes
//! are rolled back, and the caller gets [`Error::Panicked`]. That is the
//! boundary a host language calls through, where an unwinding Rust panic
//! could not be rescued.

use crate::glance::Glance;
use crate::narrates::{self, Consent, FactsCard, WouldAsk};
use crate::outcome::{Outcome, State};
use crate::playthrough::{Game, Mode};
use crate::records::{flag, id, int, text};
use crate::store::Store;
use crate::turn::chooser::{self, Candidate};
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
    /// The database is at a newer schema version than this engine is
    /// written against, and that newer schema changed a table the engine
    /// touches: each difference is a fact of `store::SHAPE` it lacks
    /// (`expected ...`) or one it has that the shape does not (`found ...`).
    SchemaChanged {
        found: String,
        differences: Vec<String>,
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
    /// The turn was stopped after this journal step, as a worker that dies
    /// there stops: what it committed stands and the submission stays
    /// running, to be finished by the next delivery.
    Stopped(String),
    /// A call that belongs to a game the player narrates, made of a game
    /// told another way (`playthroughs.mode`). Nothing was written.
    WrongMode {
        playthrough: i64,
        mode: String,
    },
    /// The scene is not one a turn the game chose was answered with in this
    /// game, so no paragraph is written for it.
    NotChosen {
        playthrough: i64,
        scene: i64,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::SchemaMismatch { found, expected } => write!(
                f,
                "the database is at schema {}, and this engine is written against {expected}",
                found.as_deref().unwrap_or("(none)")
            ),
            Error::SchemaChanged { found, differences } => write!(
                f,
                "the database is at schema {found}, which changes what this engine reads and writes: {}",
                differences.join("; ")
            ),
            Error::NoSuchPlaythrough(id) => write!(f, "there is no playthrough {id}"),
            Error::NoSuchStory(title) => write!(f, "there is no story titled {title:?}"),
            Error::Database(message) => write!(f, "database: {message}"),
            Error::Unsupported(what) => write!(f, "this engine does not play {what} yet"),
            Error::Panicked(message) => write!(f, "the engine failed: {message}"),
            Error::Model(failure) => write!(f, "the model call failed: {failure}"),
            Error::Interrupted => f.write_str("a previous worker stopped during this turn"),
            Error::PreviouslyFailed => f.write_str("this submission has already failed"),
            Error::Stopped(step) => write!(f, "the turn was stopped after its {step} step"),
            Error::WrongMode { playthrough, mode } => write!(
                f,
                "playthrough {playthrough} is {mode}, and only a game the player narrates lets the game act"
            ),
            Error::NotChosen { playthrough, scene } => write!(
                f,
                "scene {scene} is not one the game chose in playthrough {playthrough}"
            ),
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
        self.submit_stopping(playthrough, line, token, models, on_chunk, None)
    }

    /// [`Engine::submit`], with the turn stopped right after the journal
    /// step `stop_after` commits, as a worker killed there stops. The engine
    /// sweep plays its interrupted workers this way.
    pub fn submit_stopping(
        &mut self,
        playthrough: i64,
        line: &str,
        token: &str,
        models: &mut dyn crate::model::Models,
        on_chunk: &mut dyn FnMut(&str),
        stop_after: Option<&str>,
    ) -> Result<Submitted, Error> {
        let store = &self.store;
        let result = guarded(|| {
            let mut turn = crate::turn::Turn::new(store, playthrough, models, on_chunk)?;
            turn.stop_after(stop_after);
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

    /// [`Engine::submit`], with the line read as `fixed` says wherever the
    /// classifier would have been asked (`turn::Fixed`): a bench that
    /// measures what a turn does after its reading plays its cases this
    /// way, so the one model call it does not measure is never made.
    pub fn submit_fixed(
        &mut self,
        playthrough: i64,
        line: &str,
        token: &str,
        fixed: &crate::turn::Fixed,
        models: &mut dyn crate::model::Models,
        on_chunk: &mut dyn FnMut(&str),
    ) -> Result<Submitted, Error> {
        let store = &self.store;
        let result = guarded(|| {
            let mut turn = crate::turn::Turn::new(store, playthrough, models, on_chunk)?;
            turn.read_as(Some(fixed.clone()));
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

    /// Lets the game act, in a game the player narrates: a submission whose
    /// line is [`chooser::LINE`], played like any other, where the turn
    /// picks the protagonist's act with one die from what the player's
    /// panels offer ([`chooser`]) and keeps the pick in its journal before
    /// reading it. The scene is told in the engine's own words, no narrator
    /// is asked, and it waits for the player's paragraph
    /// ([`Engine::write_paragraph`]). `unwritten` lets the pick walk into a
    /// room nobody has written, which writes it through `models`; without it
    /// a game on a written world asks no model at all.
    ///
    /// Refused with [`Error::WrongMode`], having written nothing, in a game
    /// told any other way.
    pub fn act(
        &mut self,
        playthrough: i64,
        token: &str,
        unwritten: bool,
        models: &mut dyn crate::model::Models,
        on_chunk: &mut dyn FnMut(&str),
    ) -> Result<Submitted, Error> {
        self.act_stopping(playthrough, token, unwritten, models, on_chunk, None)
    }

    /// [`Engine::act`], stopped right after the journal step `stop_after`
    /// commits, as a worker killed there stops.
    pub fn act_stopping(
        &mut self,
        playthrough: i64,
        token: &str,
        unwritten: bool,
        models: &mut dyn crate::model::Models,
        on_chunk: &mut dyn FnMut(&str),
        stop_after: Option<&str>,
    ) -> Result<Submitted, Error> {
        let store = &self.store;
        let result = guarded(|| {
            let mode = mode_of(store, playthrough)?;
            if mode != Mode::PlayerNarrates {
                return Err(Error::WrongMode {
                    playthrough,
                    mode: mode.as_str().to_string(),
                });
            }
            let mut turn = crate::turn::Turn::new(store, playthrough, models, on_chunk)?;
            turn.stop_after(stop_after);
            turn.walk_into_unwritten(unwritten);
            let turned = turn.play(chooser::LINE, token)?;
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

    /// Every act the game could choose now in this game, and what the die
    /// would weigh each at ([`chooser::candidates`]). Writes nothing and
    /// throws no die.
    pub fn candidates(&self, playthrough: i64, unwritten: bool) -> Result<Vec<Candidate>, Error> {
        let store = &self.store;
        guarded(|| {
            let mechanics = Mechanics::new(store, playthrough)?;
            Ok(chooser::candidates(&mechanics, unwritten))
        })
    }

    /// What the player writes a chosen turn's paragraph from
    /// ([`narrates::card`]): none for a scene no turn the game chose was
    /// answered with. Writes nothing.
    pub fn facts_card(&self, playthrough: i64, scene: i64) -> Result<Option<FactsCard>, Error> {
        let store = &self.store;
        guarded(|| {
            let records = store.load()?;
            if records.find("playthroughs", playthrough).is_none() {
                return Err(Error::NoSuchPlaythrough(playthrough));
            }
            Ok(narrates::card(&records, playthrough, scene))
        })
    }

    /// What the game's doctor reports about the noticed records of a story's
    /// games ([`crate::noticed::findings`]). Writes nothing.
    pub fn noticed_findings(&self, story: i64) -> Result<Vec<crate::noticed::Finding>, Error> {
        let store = &self.store;
        guarded(|| {
            let records = store.load()?;
            if records.find("stories", story).is_none() {
                return Err(Error::NoSuchStory(format!("#{story}")));
            }
            Ok(crate::noticed::findings(&records, story))
        })
    }

    /// The requests a narrator would have been sent for a turn the game
    /// chose, built when the turn was played and never sent
    /// ([`narrates::would_ask`]): empty for any other scene. Writes nothing.
    pub fn would_ask(&self, playthrough: i64, scene: i64) -> Result<Vec<WouldAsk>, Error> {
        let store = &self.store;
        guarded(|| {
            let records = store.load()?;
            if records.find("playthroughs", playthrough).is_none() {
                return Err(Error::NoSuchPlaythrough(playthrough));
            }
            Ok(narrates::would_ask(&records, playthrough, scene))
        })
    }

    /// Keeps the player's paragraph for a scene a turn the game chose was
    /// answered with, beside the scene and never in it, and returns its row's
    /// id. A second paragraph for the same scene replaces the first, and
    /// clears what the game's check found in the first (`audit`). The engine
    /// reads nothing from it. The row carries the requests a narrator would
    /// have been sent for the turn and their prompt digest
    /// ([`Engine::would_ask`]); what the player let it be used for stays as
    /// it was, [`Consent::None`] on a first paragraph.
    pub fn write_paragraph(
        &mut self,
        playthrough: i64,
        scene: i64,
        words: &str,
    ) -> Result<i64, Error> {
        self.keep_paragraph(playthrough, scene, words, None)
    }

    /// [`Engine::write_paragraph`], kept with what the player let it be used
    /// for.
    pub fn write_paragraph_with_consent(
        &mut self,
        playthrough: i64,
        scene: i64,
        words: &str,
        consent: Consent,
    ) -> Result<i64, Error> {
        self.keep_paragraph(playthrough, scene, words, Some(consent))
    }

    fn keep_paragraph(
        &mut self,
        playthrough: i64,
        scene: i64,
        words: &str,
        consent: Option<Consent>,
    ) -> Result<i64, Error> {
        let store = &self.store;
        transaction(store, || {
            let records = store.load()?;
            if records.find("playthroughs", playthrough).is_none() {
                return Err(Error::NoSuchPlaythrough(playthrough));
            }
            let game = Game::new(&records, playthrough);
            if !game.player_narrates() {
                return Err(Error::WrongMode {
                    playthrough,
                    mode: game.mode().as_str().to_string(),
                });
            }
            if !narrates::chosen_scenes(&records, playthrough).contains(&scene) {
                return Err(Error::NotChosen { playthrough, scene });
            }
            let asked = narrates::would_ask(&records, playthrough, scene);
            let mut values = vec![
                ("text", Value::from(words)),
                (
                    "requests",
                    if asked.is_empty() {
                        Value::Null
                    } else {
                        asked.iter().map(WouldAsk::to_json).collect()
                    },
                ),
                (
                    "prompt_digest",
                    asked
                        .first()
                        .and_then(WouldAsk::prompt_digest)
                        .map_or(Value::Null, Value::from),
                ),
            ];
            if let Some(consent) = consent {
                values.push(("consent", Value::from(consent.as_str())));
            }
            match narrates::paragraph(&records, scene).map(id) {
                Some(kept) => {
                    values.push(("audit", Value::Null));
                    store.update("playthrough_paragraphs", kept, &values)?;
                    Ok(kept)
                }
                None => {
                    values.extend([
                        ("playthrough_id", Value::from(playthrough)),
                        ("scene_id", Value::from(scene)),
                        ("author", Value::from(narrates::PLAYER)),
                    ]);
                    store
                        .insert("playthrough_paragraphs", &values)
                        .map(|row| id(&row))
                }
            }
        })
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

    /// What a front end's panels show between turns, and which verbs are
    /// open ([`Glance`]). Writes nothing.
    pub fn glance(&self, playthrough: i64) -> Result<Glance, Error> {
        let store = &self.store;
        guarded(|| {
            let mechanics = Mechanics::new(store, playthrough)?;
            Ok(Glance::read(&mechanics))
        })
    }

    /// Starts a new playthrough of a story, as the browser does: the story's
    /// protagonist, standing in its lowest-id written room at its opening
    /// scene, with this game's copies of what they carry and of that room.
    /// Returns the new playthrough's id.
    pub fn start(&mut self, story: i64) -> Result<i64, Error> {
        self.start_in(story, Mode::Narrated)
    }

    /// [`Engine::start`], told in `mode`, which the game keeps for good.
    pub fn start_in(&mut self, story: i64, mode: Mode) -> Result<i64, Error> {
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
            let mut values = vec![
                ("story_id", Value::from(story)),
                ("character_id", protagonist.map_or(Value::Null, Value::from)),
                (
                    "current_location_id",
                    opening.map_or(Value::Null, Value::from),
                ),
                ("current_scene_id", scene.map_or(Value::Null, Value::from)),
                ("token", Value::from(token)),
            ];
            if mode != Mode::Narrated {
                values.push(("mode", Value::from(mode.as_str())));
            }
            let row = store.insert("playthroughs", &values)?;
            let playthrough = id(&row);
            let mut mechanics = Mechanics::new(store, playthrough)?;
            mechanics.snapshot_party()?;
            let here =
                opening.and_then(|room| mechanics.records().find("locations", room).cloned());
            mechanics.snapshot_room(here.as_ref())?;
            if let Some(room) = &here {
                mechanics.notice_on_arrival(room)?;
            }
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

/// How a game is told, off its row.
fn mode_of(store: &Store, playthrough: i64) -> Result<Mode, Error> {
    let mode: String = store
        .connection()
        .query_row(
            "SELECT mode FROM playthroughs WHERE id = ?1",
            [playthrough],
            |row| row.get(0),
        )
        .map_err(|_| Error::NoSuchPlaythrough(playthrough))?;
    Ok(Mode::parse(&mode).unwrap_or_default())
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
