//! `Playthrough::Classifier#classify` inside a turn: the reading is
//! [`crate::classifier::read`]'s, asked through the turn's models, and a
//! reach that found nothing is counted (`Playthrough::Drift`), and so is a
//! line that named two things (`Playthrough::Overreach`). Where the host
//! asks for it, a line the local reader's reading refuses is kept in the
//! refusal corpus ([`REFUSED_LINES`]).
//!
//! A turn played with a fixed reading ([`Fixed`]) takes it in place of both
//! calls, as a bench that measures what comes after the reading does
//! (`Eval::Prompt::Bench::FixedClassifier`).

use super::{model, Turn};
use crate::cascade;
use crate::classifier::{self, Reader, Reading};
use crate::command;
use crate::engine::Error;
use crate::intent::{build_intent, Intent};
use crate::model::{Agent, Answer, Book, Call, Failure, Filed, Models, Unavailable};
use crate::records::{int, Records};
use crate::room::{Record, Room};
use crate::store::{Store, REFUSED_LINES};
use serde_json::{json, Value};

pub(super) use crate::classifier::MODEL_PATHS;

/// `scenes.resolved_by` for a line read as a [`Fixed`] reading said.
pub const FIXED: &str = "fixed";

/// The refusals about how a line was read, which the refusal corpus keeps:
/// the reader's to learn from. The rest refuse a reading that was right.
pub const KEPT: &[&str] = &["unsure", "unresolved", "named_more_than_one", "unreadable"];

/// A reading decided before the turn, in place of the classifier's: an
/// action out of the intent table and the name of its target, resolved
/// against the room the way a model's answer is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fixed {
    pub action: String,
    pub target: Option<String>,
}

/// The turn's models, as the classifier reads a line through them.
struct Asking<'a, 'm> {
    models: &'a mut (dyn Models + 'm),
    store: &'a Store,
    records: &'a mut Records,
    filed: Filed,
    agent: Option<Agent>,
}

impl Reader for Asking<'_, '_> {
    fn system_one(&self) -> bool {
        self.models.classifier_system_one()
    }

    fn local(&self) -> bool {
        self.models.local_classifier()
    }

    fn questions(&mut self, state: &Value, questions: &Value) -> Result<Value, Unavailable> {
        let mut book = Book {
            store: self.store,
            records: self.records,
        };
        let answered =
            self.models
                .ask_classifier_questions(&mut book, &self.filed, state, questions);
        // Every free-typed line is refused while the local reader is down,
        // so it is said where an operator will see it.
        if let (true, Err(unavailable)) = (self.models.local_classifier(), &answered) {
            eprintln!(
                "renderedstep: the local reader is down; free-typed lines are refused: {}",
                unavailable.0
            );
        }
        answered
    }

    fn classifier(&mut self, call: &Call) -> Result<Answer, Failure> {
        let mut agent = Agent::new(self.filed.clone());
        let mut book = Book {
            store: self.store,
            records: self.records,
        };
        let answer = self.models.ask(&mut book, &mut agent, call, None, None);
        self.agent = Some(agent);
        answer
    }
}

impl Turn<'_, '_> {
    /// Reads `typed` and says which reader answered, keeping the classifier's
    /// conversation for the turn's scene to claim.
    pub(super) fn classify(&mut self, typed: &str) -> Result<(Intent, String), Error> {
        let room = self.m.room();
        let (intent, path, read) = match self.fixed.clone() {
            Some(fixed) => (
                build_intent(
                    &room,
                    Some(&fixed.action),
                    fixed.target.as_deref(),
                    None,
                    None,
                ),
                FIXED,
                None,
            ),
            None => {
                let call = classifier::call(&self.m.records, &room, typed);
                let filed = self.filed("classifier");
                let mut asking = Asking {
                    models: &mut *self.models,
                    store: self.m.store,
                    records: &mut self.m.records,
                    filed,
                    agent: None,
                };
                let reading = classifier::read(&room, &call, typed, &mut asking).map_err(model)?;
                if asking.agent.is_some() {
                    self.classifier = asking.agent;
                }
                (reading.intent.clone(), reading.path, Some(reading))
            }
        };
        if intent.reached_for_nothing() {
            self.record_drift(&room, typed, &intent)?;
        }
        if intent.named_more_than_one() {
            self.record_overreach(typed, &intent)?;
        }
        if let Some(reading) = read {
            if self.models.local_classifier() && self.models.keeps_refusals() {
                self.keep_refusal(&room, typed, &reading)?;
            }
        }
        Ok((intent, path.to_string()))
    }

    /// Keeps a line the local reader's reading refuses, as the turn's
    /// refusal step will refuse it: the line, the position as the reader
    /// was shown it, its whole reply, what it read and how sure it was, and
    /// what the player was told. The label slots are left for whoever
    /// labels it.
    fn keep_refusal(&mut self, room: &Room, typed: &str, reading: &Reading) -> Result<(), Error> {
        let Some(refusal) = self.m.refusal_for(&reading.intent, typed, room) else {
            return Ok(());
        };
        if !KEPT.contains(&refusal.kind.as_str()) {
            return Ok(());
        }
        let intent = &reading.intent;
        let label = |record: Option<Record>| {
            record.map_or(Value::Null, |record| Value::from(record.label()))
        };
        let target = match &intent.physical {
            Some(choice) => Value::from(choice.name()),
            None => label(intent.subject()),
        };
        let confidence = |pick: Option<f64>| pick.map_or(Value::Null, Value::from);
        let (_, location, _) = self.filed_row();
        self.m.store.insert(
            REFUSED_LINES,
            &[
                ("typed", Value::from(typed)),
                ("state", cascade::State::new(room, typed).to_json()),
                ("answers", reading.answers.clone().unwrap_or(Value::Null)),
                (
                    "reading",
                    json!({
                        "action": intent.action,
                        "target": target,
                        "also_named": label(intent.also_named.clone()),
                        "resolved_by": reading.path,
                    }),
                ),
                (
                    "act_confidence",
                    confidence(reading.confidence.map(|c| c.act)),
                ),
                (
                    "thing_confidence",
                    confidence(reading.confidence.and_then(|c| c.thing)),
                ),
                ("refusal", command::refusal_columns(&refusal)),
                (
                    "place",
                    room.here
                        .as_ref()
                        .map_or(Value::Null, |place| Value::from(place.name.as_str())),
                ),
                (
                    "weights",
                    self.models
                        .classifier_weights()
                        .map_or(Value::Null, Value::from),
                ),
                ("playthrough_id", Value::from(self.m.playthrough)),
                ("location_id", location.map_or(Value::Null, Value::from)),
                ("status", Value::from("new")),
            ],
        )?;
        Ok(())
    }

    fn filed_row(&self) -> (Option<i64>, Option<i64>, i64) {
        let game = self.m.game();
        (
            int(game.row, "current_scene_id"),
            int(game.row, "current_location_id"),
            game.story_now(),
        )
    }

    fn record_drift(&mut self, room: &Room, typed: &str, intent: &Intent) -> Result<(), Error> {
        let names = |records: Vec<Record>| -> Vec<String> {
            records
                .iter()
                .flat_map(|record| match record {
                    Record::Person(person) => {
                        let mut names = vec![person.fullname.clone()];
                        names.extend(person.nickname.clone());
                        names
                    }
                    other => vec![other.label()],
                })
                .collect()
        };
        let offered = match intent.action.as_str() {
            "move" => names(room.exits_here()),
            "talk" | "attack" => names(room.characters_here()),
            "take" => names(room.items_here()),
            "drop" => names(room.items_carried()),
            _ => Vec::new(),
        };
        let (scene, location, now) = self.filed_row();
        self.m.insert(
            "playthrough_drifts",
            vec![
                ("playthrough_id", Value::from(self.m.playthrough)),
                ("scene_id", scene.map_or(Value::Null, Value::from)),
                ("location_id", location.map_or(Value::Null, Value::from)),
                ("action", Value::from(intent.action.as_str())),
                ("command", Value::from(typed)),
                ("offered", Value::from(offered.join(", "))),
                ("story_timestamp", Value::from(now)),
            ],
        )?;
        Ok(())
    }

    fn record_overreach(&mut self, typed: &str, intent: &Intent) -> Result<(), Error> {
        let label =
            |record: Option<Record>| record.map(|record| record.label()).unwrap_or_default();
        let (scene, location, now) = self.filed_row();
        self.m.insert(
            "playthrough_overreaches",
            vec![
                ("playthrough_id", Value::from(self.m.playthrough)),
                ("scene_id", scene.map_or(Value::Null, Value::from)),
                ("location_id", location.map_or(Value::Null, Value::from)),
                ("action", Value::from(intent.action.as_str())),
                ("command", Value::from(typed)),
                ("acted", Value::from(label(intent.subject()))),
                ("unacted", Value::from(label(intent.also_named.clone()))),
                ("story_timestamp", Value::from(now)),
            ],
        )?;
        Ok(())
    }
}
