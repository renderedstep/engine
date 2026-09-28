//! `Playthrough::Classifier#classify` inside a turn: the reading is
//! [`crate::classifier::read`]'s, asked through the turn's models, and a
//! reach that found nothing is counted (`Playthrough::Drift`), and so is a
//! line that named two things (`Playthrough::Overreach`).
//!
//! A turn played with a fixed reading ([`Fixed`]) takes it in place of both
//! calls, as a bench that measures what comes after the reading does
//! (`Eval::Prompt::Bench::FixedClassifier`).

use super::{model, Turn};
use crate::classifier::{self, Reader};
use crate::engine::Error;
use crate::intent::{build_intent, Intent};
use crate::model::{Agent, Answer, Book, Call, Failure, Filed, Models, Unavailable};
use crate::records::{int, Records};
use crate::room::{Record, Room};
use crate::store::Store;
use serde_json::Value;

pub(super) use crate::classifier::MODEL_PATHS;

/// `scenes.resolved_by` for a line read as a [`Fixed`] reading said.
pub const FIXED: &str = "fixed";

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
        self.models.system_one()
    }

    fn questions(&mut self, state: &Value, questions: &Value) -> Result<Value, Unavailable> {
        let mut book = Book {
            store: self.store,
            records: self.records,
        };
        self.models
            .ask_questions(&mut book, &self.filed, state, questions)
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
        let (intent, path) = match self.fixed.clone() {
            Some(fixed) => (
                build_intent(
                    &room,
                    Some(&fixed.action),
                    fixed.target.as_deref(),
                    None,
                    None,
                ),
                FIXED,
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
                (reading.intent, reading.path)
            }
        };
        if intent.reached_for_nothing() {
            self.record_drift(&room, typed, &intent)?;
        }
        if intent.named_more_than_one() {
            self.record_overreach(typed, &intent)?;
        }
        Ok((intent, path.to_string()))
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
