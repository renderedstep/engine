//! A conversation (`InteractionAgent`) and a throw, on the model path.
//!
//! Talking is two calls with the engine between them. The person answers
//! first, as themselves, in their own durable conversation with this game:
//! what they thought and felt either side of answering, and one action out
//! of the closed set the engine offered (`Playthrough::NpcAction`). The
//! engine applies that action, if it is still theirs to take, and the
//! narrator is then handed the reaction and the engine's receipt and writes
//! the prose. A failed narration after the engine applied the choice is
//! answered with the receipt itself.
//!
//! A throw moves the row first and is told afterwards, as a take is.

use super::kept::NpcEffect;
use super::{model, Told, Turn};
use crate::dialogue;
use crate::engine::Error;
use crate::model::receipts::{self, Filed};
use crate::model::{Agent, Book, Call, Failure};
use crate::playthrough::Game;
use crate::records::{id, int, string, text, Row};
use crate::room::Record;
use serde_json::{Map, Value};

/// `Interaction::Schema`'s fields and the cap each is written under.
const REACTION: [(&str, usize); 6] = [
    ("pre_thought", 320),
    ("pre_feeling", 120),
    ("action", 480),
    ("post_feeling", 120),
    ("post_thought", 320),
    ("inner_resolution", 320),
];

/// `Interaction`'s required fields.
const REQUIRED: [&str; 5] = [
    "pre_thought",
    "pre_feeling",
    "action",
    "post_feeling",
    "post_thought",
];

/// `Scene::TURN_MINUTES["conversation"]`, in seconds.
const CONVERSATION_SECONDS: i64 = 10 * 60;

/// `InteractionAgent#reaction_fields`, checked the way the row that keeps
/// them is (`#verified_reaction_fields`): every field sanitized under its
/// cap, a field that arrived at its cap refused as cut off, and the five
/// the row needs present.
fn reaction_fields(content: &Value) -> Result<Map<String, Value>, String> {
    let mut fields = Map::new();
    for (name, cap) in REACTION {
        let raw = content
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default();
        if raw.chars().count() >= cap {
            return Err(format!(
                "generated text arrived at its {cap}-character cap ({} characters), so it was cut off rather than finished",
                raw.chars().count()
            ));
        }
        fields.insert(name.to_string(), Value::from(dialogue::sanitize(raw)));
    }
    if let Some(missing) = REQUIRED
        .iter()
        .find(|name| fields[**name].as_str().is_none_or(crate::text::is_blank))
    {
        return Err(format!("Validation failed: {missing} can't be blank"));
    }
    Ok(fields)
}

impl Turn<'_, '_> {
    /// `Playthrough::Turn#talk_to`.
    pub(super) fn talk_to(
        &mut self,
        character: i64,
        command: &str,
        offered: Option<i64>,
    ) -> Result<Option<Told>, Error> {
        if self.done("talked") {
            return self.saved("talked").map(Option::flatten);
        }
        if offered.is_some() {
            return Err(Error::Unsupported(
                "offering a thing to somebody through the models".into(),
            ));
        }
        let person = self.m.row("characters", character)?;
        let playthrough = self.m.playthrough;
        let chat = {
            let mut book = Book {
                store: self.m.store,
                records: &mut self.m.records,
            };
            let chat = receipts::conversation_with(&book, character, playthrough);
            if let Some(chat) = chat {
                receipts::pick_up(&mut book, chat)?;
            }
            chat
        };
        let filed = Filed {
            purpose: receipts::CHARACTER.into(),
            character: Some(character),
            ..self.filed(receipts::CHARACTER)
        };
        let mut character_agent = Agent::continuing(filed, chat);

        let (reaction, fields) = self.remember("character_answer", |turn| {
            let game = Game::new(&turn.m.records, playthrough);
            let request = dialogue::character_request(&game, &person, command);
            let call = Call::from_request(&request);
            let mut verified = None;
            let mut verify = |content: &Value| {
                verified = Some(reaction_fields(content)?);
                Ok(())
            };
            let mut book = Book {
                store: turn.m.store,
                records: &mut turn.m.records,
            };
            let answer = turn
                .models
                .ask(
                    &mut book,
                    &mut character_agent,
                    &call,
                    Some(&mut verify),
                    None,
                )
                .map_err(model)?;
            let fields = match verified {
                Some(fields) => fields,
                None => reaction_fields(&answer.content)
                    .map_err(|reason| model(Failure::Rejected(reason)))?,
            };
            Ok((answer.content, fields))
        })?;

        let choice = reaction
            .get("engine_action")
            .and_then(Value::as_str)
            .unwrap_or(dialogue::NONE)
            .to_string();
        let NpcEffect {
            action,
            status,
            fact,
        } = self.commit("character_effect", |turn| {
            let (action, status, fact) = turn.m.apply_npc(&person, &choice, offered)?;
            Ok(NpcEffect {
                action,
                status: status.to_string(),
                fact,
            })
        })?;

        let reaction_map = reaction.as_object().cloned().unwrap_or_default();
        let request = {
            let game = Game::new(&self.m.records, playthrough);
            dialogue::narrator_request(&game, &person, command, &reaction_map, &fact)
        };
        let mut narrator = Agent::new(self.filed("interaction-narration"));
        let asked = {
            let mut book = Book {
                store: self.m.store,
                records: &mut self.m.records,
            };
            let call = Call::from_request(&request);
            self.models
                .ask(
                    &mut book,
                    &mut narrator,
                    &call,
                    None,
                    Some(&mut |_: &str| {}),
                )
                .and_then(|answer| {
                    let text = answer.text().to_string();
                    if crate::text::is_blank(&text) {
                        Err(Failure::SchemaIgnored(
                            "The interaction narrator returned no prose".into(),
                        ))
                    } else {
                        Ok(text)
                    }
                })
        };
        let (narration, fallback, safety) = match asked {
            Ok(text) => (text, false, false),
            Err(failure) => (
                format!("You speak with {}. {fact}", string(&person, "fullname")),
                true,
                failure.crisis(),
            ),
        };
        (self.on_chunk)(&narration);

        let summary = [
            format!("The player spoke with {}.", string(&person, "fullname")),
            fields["action"].as_str().unwrap_or_default().to_string(),
            fact.clone(),
        ]
        .join(" ");
        let told = self.commit("talked", |turn| {
            let here = turn.m.here().map(|room| id(&room));
            let at = turn.m.story_now() + CONVERSATION_SECONDS;
            let scene = turn.m.write_scene(
                vec![
                    ("location_id", here.map_or(Value::Null, Value::from)),
                    ("description", Value::from(narration.as_str())),
                    ("engine_fact", Value::from(fact.as_str())),
                    ("engine_fallback", Value::Bool(fallback)),
                    ("summary", Value::from(summary.as_str())),
                    ("story_timestamp", Value::from(at)),
                ],
                &[],
            )?;
            let mut values: Vec<(&str, Value)> = REACTION
                .iter()
                .map(|(name, _)| (*name, fields[*name].clone()))
                .collect();
            values.extend([
                ("engine_action", Value::from(action.as_str())),
                ("action_status", Value::from(status)),
                ("action_fact", Value::from(fact.as_str())),
                ("character_id", Value::from(character)),
                ("scene_id", Value::from(scene)),
                ("location_id", here.map_or(Value::Null, Value::from)),
                ("user_input", Value::from(command)),
            ]);
            values.push(("summary", interaction_summary(&values)));
            turn.m.insert("interactions", values)?;
            turn.m.update(
                "playthroughs",
                turn.m.playthrough,
                vec![("current_scene_id", Value::from(scene))],
            )?;
            Ok(Told {
                id: scene,
                tolls: None,
                safety,
                setup: false,
            })
        })?;
        for agent in [&character_agent, &narrator] {
            let mut book = Book {
                store: self.m.store,
                records: &mut self.m.records,
            };
            self.models.attribute(&mut book, agent, told.id)?;
        }
        Ok(Some(told))
    }

    /// `Playthrough::Turn#throw_at`.
    pub(super) fn throw_at(
        &mut self,
        item: i64,
        at: Option<&Record>,
        command: &str,
    ) -> Result<Option<Told>, Error> {
        let row = self.m.row("items", item)?;
        let thrower = self.m.player();
        let report = self.commit("throw", |turn| turn.m.throw_it(item, at, None))?;
        if report.refusal.is_some() {
            return self.narrate(command, None, None, None, None);
        }
        let thing = bare_name(&row);
        let who = thrower
            .as_ref()
            .map_or("The party", |who| string(who, "fullname"))
            .to_string();
        let (fact, words) = match (&report.change, at) {
            (Some(landed), Some(Record::Person(person))) => (
                format!(
                    "{who} threw the {thing} at {name} and it hit them. The {thing} is NO LONGER CARRIED: it is lying \
                     on the floor at {name}'s feet, where it stays until somebody picks it up.",
                    name = person.fullname
                ),
                landed.clone(),
            ),
            (Some(landed), Some(Record::Place(place))) => (
                format!(
                    "{who} threw the {thing} through the way out into {name}. The {thing} is NO LONGER CARRIED and is \
                     no longer in this room at all: it is lying in {name}, where it stays until somebody picks it up.",
                    name = place.name
                ),
                landed.clone(),
            ),
            _ => {
                let now = self.m.row("items", item)?;
                let carried = int(&now, "location_id").is_none() && int(&now, "character_id").is_none();
                let fumbled = report.note.get(1).is_some_and(|note| note.starts_with("the lift failed"));
                if !fumbled {
                    (
                        format!(
                            "{who} could not throw the {thing} at all: it is {} and does not move. Nothing happened.",
                            text(&row, "bulk").unwrap_or_default()
                        ),
                        "it does not move for anybody, so no die was thrown".to_string(),
                    )
                } else {
                    (
                        format!(
                            "{who} tried to pick up and throw the {thing} and could not get it moving: it is {} and the \
                             attempt failed. NOTHING WAS THROWN and nothing was hit -- the {thing} {}. The turn was \
                             spent on the attempt.",
                            text(&row, "bulk").unwrap_or_default(),
                            if carried {
                                "is still in the party's hands"
                            } else {
                                "is still lying exactly where it was"
                            }
                        ),
                        "the lift failed: nothing was thrown and no row moved".to_string(),
                    )
                }
            }
        };
        let fallback = format!("Your throw of the {thing}: {words}.");
        self.narrate(command, Some(fact), None, None, Some(fallback))
    }
}

/// `Item#bare_name`: the name without an article it arrived with.
fn bare_name(item: &Row) -> String {
    let definite = crate::moment::definite_name(string(item, "name"));
    definite
        .strip_prefix("the ")
        .unwrap_or(&definite)
        .to_string()
}

/// `Interaction#compose_summary`: what the player said, what the person did
/// and resolved, and the engine's receipt, as the row's one line.
fn interaction_summary(values: &[(&str, Value)]) -> Value {
    let field = |name: &str| {
        values
            .iter()
            .find(|(column, _)| *column == name)
            .and_then(|(_, value)| value.as_str())
            .filter(|text| !crate::text::is_blank(text))
    };
    let said = field("user_input")
        .map(|line| format!("the player said \"{}\"", crate::text::truncate(line, 80)));
    let parts: Vec<String> = said
        .into_iter()
        .chain(
            ["action", "inner_resolution", "action_fact"]
                .iter()
                .filter_map(|name| field(name).map(str::to_string)),
        )
        .collect();
    if parts.is_empty() {
        Value::Null
    } else {
        Value::from(parts.join(" -- "))
    }
}
