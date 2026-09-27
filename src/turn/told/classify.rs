//! `Playthrough::Classifier#classify`: a line the grammar did not claim,
//! read by System One first where it is on, and by the classifier model
//! when System One is off, escalates, or is unavailable.
//!
//! System One is asked ten typed questions over the room's closed sets and
//! the engine composes the answers into a reading
//! ([`crate::cascade::compose`]). The model is asked for one intent and one
//! target out of the same sets, at temperature zero, and its answer is
//! resolved to records the same way ([`crate::intent::build_intent`]). A
//! reach that found nothing is counted (`Playthrough::Drift`), and so is a
//! line that named two things (`Playthrough::Overreach`).

use super::{model, Turn};
use crate::cascade::{self, Escalation};
use crate::data;
use crate::engine::Error;
use crate::intent::{build_intent, Intent};
use crate::model::{Agent, Book, Call};
use crate::records::{int, string};
use crate::room::{Record, Room};
use crate::schemas;
use serde_json::Value;

/// `Playthrough::Classifier::MODEL_PATHS`: the readings a model call made.
pub(super) const MODEL_PATHS: [&str; 3] =
    ["model", "typed_model_escalated", "typed_model_unavailable"];

impl Turn<'_, '_> {
    /// Reads `typed` and says which reader answered, keeping the classifier's
    /// conversation for the turn's scene to claim.
    pub(super) fn classify(&mut self, typed: &str) -> Result<(Intent, String), Error> {
        let room = self.m.room();
        let (composed, path) = if self.models.system_one() {
            self.cascaded(&room, typed)
        } else {
            (None, "model")
        };
        let intent = match composed {
            Some(intent) => intent,
            None => self.ask_the_model(&room, typed)?,
        };
        if intent.reached_for_nothing() {
            self.record_drift(&room, typed, &intent)?;
        }
        if intent.named_more_than_one() {
            self.record_overreach(typed, &intent)?;
        }
        Ok((intent, path.to_string()))
    }

    fn cascaded(&mut self, room: &Room, typed: &str) -> (Option<Intent>, &'static str) {
        let state = cascade::State::new(room, typed);
        let questions = cascade::request(&state);
        let filed = self.filed("classifier");
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        let reply = self
            .models
            .ask_questions(&mut book, &filed, &state.to_json(), &questions);
        let reply = reply.map_err(|unavailable| cascade::Unavailable(unavailable.0));
        let outcome = cascade::compose(
            &state,
            &questions,
            reply.as_ref().map_err(Clone::clone),
            Escalation::On,
        );
        (outcome.intent, outcome.path)
    }

    fn ask_the_model(&mut self, room: &Room, typed: &str) -> Result<Intent, Error> {
        let mut targets: Vec<String> = Vec::new();
        targets.extend(room.exits.iter().map(|exit| exit.place.name.clone()));
        for person in &room.cast {
            targets.push(person.fullname.clone());
            targets.extend(person.nickname.clone());
        }
        targets.extend(room.lying.iter().map(|thing| thing.name.clone()));
        targets.extend(room.carried.iter().map(|thing| thing.name.clone()));
        targets.extend(room.physical_actions().iter().map(|choice| choice.token()));
        let names: Vec<&str> = targets.iter().map(String::as_str).collect();
        let call = Call {
            system: Some(data::classifier_instructions().to_string()),
            schema: Some(schemas::intent(&names)),
            ..Call::prompt(self.command_prompt(room, typed))
        }
        .with_temperature(0.0);
        let mut agent = Agent::new(self.filed("classifier"));
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        let answer = self
            .models
            .ask(&mut book, &mut agent, &call, None, None)
            .map_err(model)?;
        self.classifier = Some(agent);
        let field = |key: &str| {
            answer
                .content
                .get(key)
                .and_then(Value::as_str)
                .map(str::to_string)
        };
        Ok(build_intent(
            room,
            field("intent").as_deref(),
            field("target").as_deref(),
            field("also_named").as_deref(),
            field("thrown_at").as_deref(),
        ))
    }

    /// `#command_prompt`.
    fn command_prompt(&self, room: &Room, typed: &str) -> String {
        let here = room
            .here
            .as_ref()
            .map_or("Nowhere in particular.".to_string(), |place| {
                place.name.clone()
            });
        let exits: Vec<String> = room
            .exits
            .iter()
            .map(|exit| {
                let note = self
                    .m
                    .records
                    .find("location_connections", exit.edge)
                    .map(|edge| {
                        let barrier = match exit.barrier.as_str() {
                            "open" => String::new(),
                            "keyed" => "; locked, open before crossing".into(),
                            _ => "; jammed, open before crossing".into(),
                        };
                        format!(
                            " ({}, {}{barrier})",
                            string(edge, "distance"),
                            string(edge, "travel_method")
                        )
                    })
                    .unwrap_or_default();
                format!("- {}{note}", exit.place.name)
            })
            .collect();
        let list = |lines: Vec<String>, empty: &str| {
            if lines.is_empty() {
                empty.to_string()
            } else {
                lines.join("\n")
            }
        };
        let cast = room
            .cast
            .iter()
            .map(
                |person| match person.nickname.as_deref().filter(|n| !n.trim().is_empty()) {
                    Some(nickname) => format!("- {} ({nickname})", person.fullname),
                    None => format!("- {}", person.fullname),
                },
            )
            .collect();
        let things = |things: &[crate::room::Thing]| {
            things
                .iter()
                .map(|thing| format!("- {}", thing.name))
                .collect()
        };
        let physical: Vec<String> = room
            .physical_actions()
            .iter()
            .map(|choice| format!("{}: {}", choice.token(), choice.name()))
            .collect();
        format!(
            "## Where The Player Is\n{here}\n\n## Ways Out\n{}\n\n## Who Is Here\n{}\n\n## What Is Lying Here\n{}\n\n\
             ## What The Player Is Carrying\n{}\n\n## Physical Actions (token: one attempt)\n{}\n\n## The Player Types\n{typed}\n",
            list(exits, "None. The player cannot go anywhere from here."),
            list(cast, "Nobody. There is no one here to talk to."),
            list(things(&room.lying), "Nothing. There is nothing here to pick up."),
            list(things(&room.carried), "Nothing. The player is carrying nothing at all."),
            list(physical, "None are available."),
        )
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
