//! A typed line read through System One: one request of typed questions
//! over sets the engine closed, and then the engine deciding what the
//! answers mean.
//!
//! This module builds the request ([`State`], [`request`]) and composes a
//! recorded answer set into an [`Intent`] ([`compose`]). It never sends
//! anything: the caller hands over the provider's body, or the fact that the
//! provider failed.
//!
//! Two flags send a line on to the model call instead: `named_more_than_one`
//! at or above [`TWO_NAME_THRESHOLD`], or `target_present` below
//! [`PRESENCE_THRESHOLD`]. That is escalation, not refusal.
//!
//! A move read off a line that opens by speaking to somebody ([`spoken`]) is
//! composed as the talk it is.

use crate::data;
use crate::intent::{slot_for, Intent};
use crate::room::{Record, Room, INTENTS, NOTHING};
use serde_json::{json, Map, Value};

/// Below this, the thing the line asks for is taken to be absent.
pub const PRESENCE_THRESHOLD: f64 = 0.15;

/// At or above this, the line is taken to name two records for one act.
pub const TWO_NAME_THRESHOLD: f64 = 0.5;

/// The verbs a line opens with when the player is speaking to somebody.
pub const SPEECH_VERBS: &[&str] = &[
    "tell", "ask", "say", "shout", "yell", "whisper", "warn", "inform", "remind", "promise",
];

/// Whether the line opens with a verb of speaking. Such a line is the player
/// talking, whatever place it names: "tell Rowe I am going into the closet"
/// says where the player means to go and does not go there. Only the first
/// word counts, so "go back to the court and tell Brace" is still a move.
pub fn spoken(command: &str) -> bool {
    let line = crate::text::ruby_downcase(command);
    line.split(|c: char| !c.is_alphanumeric())
        .find(|word| !word.is_empty())
        .is_some_and(|word| SPEECH_VERBS.contains(&word))
}

/// The state blocks, and the intent whose closed set is each one.
struct Group {
    key: &'static str,
    prefix: &'static str,
    kind: &'static str,
    intent: &'static str,
}

const GROUPS: &[Group] = &[
    Group {
        key: "ways_out",
        prefix: "way",
        kind: "way out",
        intent: "move",
    },
    Group {
        key: "other_characters",
        prefix: "person",
        kind: "person present here",
        intent: "talk",
    },
    Group {
        key: "available_items",
        prefix: "available_item",
        kind: "item lying here",
        intent: "take",
    },
    Group {
        key: "player_items",
        prefix: "player_item",
        kind: "item the player is carrying",
        intent: "drop",
    },
    Group {
        key: "physical_actions",
        prefix: "attempt",
        kind: "one complete physical action attempt",
        intent: "use",
    },
];

/// The position written out as records a question can point at, each under
/// a key the answers come back naming. Every membership fact is the room's
/// [`Room::offered_for`] and nothing else.
pub struct State<'a> {
    room: &'a Room,
    command: String,
    offered: Vec<(&'static str, Vec<Record>)>,
    records: Vec<(String, Record)>,
}

impl<'a> State<'a> {
    pub fn new(room: &'a Room, command: &str) -> State<'a> {
        let offered: Vec<(&'static str, Vec<Record>)> = INTENTS
            .iter()
            .map(|intent| (*intent, room.offered_for(intent)))
            .collect();
        let mut state = State {
            room,
            command: command.to_string(),
            offered,
            records: Vec::new(),
        };
        let mut records = Vec::new();
        for group in GROUPS {
            for (offset, record) in state.offered(group.intent).iter().enumerate() {
                records.push((format!("{}_{}", group.prefix, offset + 1), record.clone()));
            }
        }
        state.records = records;
        state
    }

    fn offered(&self, action: &str) -> &[Record] {
        self.offered
            .iter()
            .find(|(intent, _)| *intent == action)
            .map_or(&[], |(_, records)| records.as_slice())
    }

    /// What an answer's key means; `nothing`, and anything this position
    /// never offered, means no record.
    pub fn record_for(&self, key: &str) -> Option<&Record> {
        self.records.iter().find(|(k, _)| k == key).map(|(_, r)| r)
    }

    fn key_of(&self, record: &Record) -> Option<&str> {
        // The last key wins, as a map inverted would have it.
        self.records
            .iter()
            .rev()
            .find(|(_, r)| r == record)
            .map(|(k, _)| k.as_str())
    }

    /// The keys, in state order, of every record one action may reach.
    pub fn keys_for(&self, action: &str) -> Vec<String> {
        self.offered(action)
            .iter()
            .filter_map(|record| self.key_of(record).map(str::to_string))
            .collect()
    }

    /// Every key in the position, in state order.
    pub fn all_keys(&self) -> Vec<String> {
        self.records.iter().map(|(k, _)| k.clone()).collect()
    }

    /// Whether one action's closed set holds this record.
    pub fn offers(&self, action: &str, record: &Record) -> bool {
        self.offered(action).contains(record)
    }

    /// The state object as it is sent. The key order is the measured one:
    /// `player_action` last, every block present, an empty one empty.
    pub fn to_json(&self) -> Value {
        let mut state = Map::new();
        let location = self
            .room
            .here
            .as_ref()
            .map_or("Nowhere in particular.".to_string(), |p| p.name.clone());
        state.insert("location".into(), Value::String(location));
        for group in GROUPS {
            let mut entries = Map::new();
            for (offset, record) in self.offered(group.intent).iter().enumerate() {
                entries.insert(
                    format!("{}_{}", group.prefix, offset + 1),
                    self.entry_for(record, group),
                );
            }
            state.insert(group.key.into(), Value::Object(entries));
        }
        state.insert("player_action".into(), Value::String(self.command.clone()));
        Value::Object(state)
    }

    fn entry_for(&self, record: &Record, group: &Group) -> Value {
        let name = record.label();
        let mut entry = Map::new();
        entry.insert("kind".into(), json!(group.kind));
        entry.insert("name".into(), json!(name));
        if let Record::Person(person) = record {
            let nickname = crate::text::ruby_strip(person.nickname.as_deref().unwrap_or(""));
            if !crate::text::is_blank(nickname) && nickname != name {
                entry.insert("aliases".into(), json!([nickname]));
            }
        }
        // The block's own intent first, then the rest in table order.
        let mut intents = vec![group.intent];
        for intent in INTENTS {
            if !intents.contains(intent) && self.offers(intent, record) {
                intents.push(intent);
            }
        }
        entry.insert("valid_intents".into(), json!(intents));
        Value::Object(entry)
    }
}

/// The id of the question one action's target is read from.
pub fn target_id(action: &str) -> String {
    format!("target_{action}")
}

fn criteria(pairs: &[(String, String)]) -> Value {
    Value::Object(
        pairs
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect(),
    )
}

fn with_nothing(text: &str) -> String {
    text.replace("%{nothing}", NOTHING)
}

/// The questions one line is asked, keyed by the ids the answers come back
/// under. A target question over an empty set is not asked.
pub fn request(state: &State) -> Value {
    let texts = data::request_texts();
    let described = |keys: &[String]| -> Map<String, Value> {
        keys.iter()
            .map(|key| {
                let label = state.record_for(key).map(Record::label).unwrap_or_default();
                (key.clone(), Value::String(label))
            })
            .collect()
    };
    let mut questions = Map::new();
    questions.insert(
        "intent".into(),
        json!({ "type": "choice", "instructions": texts.intent_instructions,
                "criteria": criteria(&texts.intent_criteria) }),
    );
    for target in &texts.targets {
        let keys = state.keys_for(&target.action);
        if keys.is_empty() {
            continue;
        }
        let mut options = described(&keys);
        options.insert(NOTHING.into(), Value::String(target.nothing.clone()));
        questions.insert(
            target_id(&target.action),
            json!({ "type": "choice",
                    "instructions": format!(
                        "Assume, for this question only, that the player is {}. {}",
                        target.premise, with_nothing(&texts.target_instructions)),
                    "criteria": Value::Object(options) }),
        );
    }
    let keys = state.all_keys();
    if !keys.is_empty() {
        let mut options = described(&keys);
        options.insert(
            NOTHING.into(),
            Value::String(texts.also_named_nothing.clone()),
        );
        questions.insert(
            "also_named".into(),
            json!({ "type": "choice",
                    "instructions": with_nothing(&texts.also_named_instructions),
                    "criteria": Value::Object(options) }),
        );
    }
    questions.insert(
        "named_more_than_one".into(),
        json!({ "type": "noul", "instructions": texts.named_more_than_one_instructions,
                "criteria": criteria(&texts.named_more_than_one_criteria) }),
    );
    questions.insert(
        "target_present".into(),
        json!({ "type": "noul", "instructions": texts.target_present_instructions,
                "criteria": criteria(&texts.target_present_criteria) }),
    );
    Value::Object(questions)
}

/// An answer set that cannot be believed, or a provider that did not
/// answer. Either way the line goes to the model call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unavailable(pub String);

/// One request's answers, verified against the questions that were sent.
pub struct Answers<'q> {
    answers: &'q Map<String, Value>,
    questions: &'q Map<String, Value>,
}

/// Ruby's `to_s` of a JSON value, for a choice compared against options.
fn to_s(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) if n.is_f64() => {
            let f = n.as_f64().unwrap_or_default();
            if f.fract() == 0.0 && f.abs() < 1e16 {
                format!("{f:.1}")
            } else {
                f.to_string()
            }
        }
        other => other.to_string(),
    }
}

impl<'q> Answers<'q> {
    /// Checks the body: an `answers` map answering every question sent, and
    /// a numeric cost if it reports one.
    pub fn new(body: &'q Value, questions: &'q Value) -> Result<Answers<'q>, Unavailable> {
        let questions = questions
            .as_object()
            .ok_or_else(|| Unavailable("the questions are not a map".into()))?;
        let body = body.as_object().ok_or_else(|| {
            Unavailable("the provider answered something other than a body".into())
        })?;
        let answers = body
            .get("answers")
            .and_then(Value::as_object)
            .ok_or_else(|| Unavailable("the provider answered no `answers` map".into()))?;
        let missing: Vec<&str> = questions
            .keys()
            .filter(|id| !answers.contains_key(*id))
            .map(String::as_str)
            .collect();
        if !missing.is_empty() {
            return Err(Unavailable(format!(
                "the provider did not answer {}",
                missing.join(", ")
            )));
        }
        if let Some(cost) = body
            .get("usage")
            .and_then(Value::as_object)
            .and_then(|usage| usage.get("cost"))
        {
            if !cost.is_number() {
                return Err(Unavailable(format!(
                    "usage.cost answered {cost} rather than a number"
                )));
            }
        }
        Ok(Answers { answers, questions })
    }

    fn answer_of(&self, id: &str, kind: &str) -> Result<&Map<String, Value>, Unavailable> {
        let answer = self
            .answers
            .get(id)
            .and_then(Value::as_object)
            .ok_or_else(|| Unavailable(format!("{id} was not answered with a map")))?;
        if answer.get("type").and_then(Value::as_str) != Some(kind) {
            return Err(Unavailable(format!(
                "{id} came back as something other than a {kind}"
            )));
        }
        Ok(answer)
    }

    /// The option chosen for one choice question, verified to be one sent.
    pub fn choice(&self, id: &str) -> Result<String, Unavailable> {
        let answer = self.answer_of(id, "choice")?;
        let chosen = to_s(answer.get("choice").unwrap_or(&Value::Null));
        let sent = self
            .questions
            .get(id)
            .and_then(|q| q.get("criteria"))
            .and_then(Value::as_object)
            .is_some_and(|options| options.contains_key(&chosen));
        if !sent {
            return Err(Unavailable(format!(
                "{id} answered {chosen:?}, which is not one of the options sent"
            )));
        }
        Ok(chosen)
    }

    /// The yes probability for one noul question.
    pub fn noul(&self, id: &str) -> Result<f64, Unavailable> {
        let reading = self.answer_of(id, "noul")?.get("noul");
        match reading.and_then(Value::as_f64) {
            Some(p) if (0.0..=1.0).contains(&p) => Ok(p),
            _ => Err(Unavailable(format!(
                "{id} answered something other than a probability"
            ))),
        }
    }

    /// Whether a question was sent at all.
    pub fn asked(&self, id: &str) -> bool {
        self.questions.contains_key(id)
    }
}

/// Whether the two flags send a line on. [`Escalation::Off`] reads the
/// presence gate on its own, as a measurement of that gate does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escalation {
    On,
    Off,
}

/// What one line composed to.
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    /// `typed_model`, `typed_model_escalated` or `typed_model_unavailable`.
    pub path: &'static str,
    pub target_present: Option<f64>,
    pub named_more_than_one: Option<f64>,
    /// `None` when the line goes on to the model call.
    pub intent: Option<Intent>,
}

impl Outcome {
    fn unavailable(target_present: Option<f64>, named_more_than_one: Option<f64>) -> Outcome {
        Outcome {
            path: "typed_model_unavailable",
            target_present,
            named_more_than_one,
            intent: None,
        }
    }
}

/// Composes the provider's reply -- its whole body, or the reason it did not
/// answer -- to the questions `questions` asked about `state`.
pub fn compose(
    state: &State,
    questions: &Value,
    reply: Result<&Value, Unavailable>,
    escalation: Escalation,
) -> Outcome {
    let Ok(body) = reply else {
        return Outcome::unavailable(None, None);
    };
    let Ok(answers) = Answers::new(body, questions) else {
        return Outcome::unavailable(None, None);
    };
    let readings = answers.choice("intent").and_then(|action| {
        let presence = answers.noul("target_present")?;
        let two_name = answers.noul("named_more_than_one")?;
        Ok((action, presence, two_name))
    });
    let Ok((action, presence, two_name)) = readings else {
        return Outcome::unavailable(None, None);
    };
    let escalate = escalation == Escalation::On
        && (two_name >= TWO_NAME_THRESHOLD || presence < PRESENCE_THRESHOLD);
    if escalate {
        return Outcome {
            path: "typed_model_escalated",
            target_present: Some(presence),
            named_more_than_one: Some(two_name),
            intent: None,
        };
    }
    // The talk's own target question names who is spoken to; with nobody
    // here it was never asked, and the talk reaches for nothing.
    let action = if action == "move" && spoken(&state.command) {
        "talk".to_string()
    } else {
        action
    };
    let chosen = if presence >= PRESENCE_THRESHOLD {
        chosen_target(state, &answers, &action).and_then(|target| {
            let also = chosen_also(state, &answers, &action, target.as_ref())?;
            Ok((target, also))
        })
    } else {
        Ok((None, None))
    };
    let Ok((target, also)) = chosen else {
        return Outcome::unavailable(Some(presence), Some(two_name));
    };
    Outcome {
        path: "typed_model",
        target_present: Some(presence),
        named_more_than_one: Some(two_name),
        intent: Some(intent_for(&action, target, also)),
    }
}

fn chosen_target(
    state: &State,
    answers: &Answers,
    action: &str,
) -> Result<Option<Record>, Unavailable> {
    let id = target_id(action);
    if !answers.asked(&id) {
        return Ok(None);
    }
    Ok(state.record_for(&answers.choice(&id)?).cloned())
}

/// The second name, narrowed to this intent's own set, and dropped when it
/// is the target itself.
fn chosen_also(
    state: &State,
    answers: &Answers,
    action: &str,
    target: Option<&Record>,
) -> Result<Option<Record>, Unavailable> {
    if !answers.asked("also_named") {
        return Ok(None);
    }
    let key = answers.choice("also_named")?;
    Ok(state
        .record_for(&key)
        .filter(|record| Some(*record) != target && state.offers(action, record))
        .cloned())
}

fn intent_for(action: &str, target: Option<Record>, also: Option<Record>) -> Intent {
    if action == "use" {
        let physical = target.as_ref().and_then(Record::attempt).cloned();
        let extra = also
            .as_ref()
            .and_then(Record::attempt)
            .and_then(|choice| choice.subject())
            .filter(|extra| {
                !physical
                    .as_ref()
                    .is_some_and(|p| p.records().contains(extra))
            });
        return Intent {
            physical,
            also_named: extra,
            ..Intent::new(action)
        };
    }
    match slot_for(action) {
        Some(slot) => Intent {
            also_named: also,
            ..Intent::new(action)
        }
        .with(slot, target),
        None => Intent::new(action),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::{Exit, Person, Place};

    fn room(cast: Vec<Person>) -> Room {
        Room {
            protagonist: Some(Person {
                id: 1,
                fullname: "Cal".into(),
                nickname: None,
            }),
            here: Some(Place {
                id: 10,
                name: "Ward Office 12".into(),
            }),
            exits: vec![Exit {
                place: Place {
                    id: 11,
                    name: "The Supply Closet".into(),
                },
                barrier: "open".into(),
                edge: 100,
                key: None,
            }],
            cast,
            lying: Vec::new(),
            carried: Vec::new(),
            over: false,
        }
    }

    fn rowe() -> Person {
        Person {
            id: 2,
            fullname: "Halkett Rowe".into(),
            nickname: Some("Rowe".into()),
        }
    }

    /// System One's body for every question sent: the choices named, and
    /// `nothing` or a probability of zero for the rest.
    fn body(questions: &Value, chosen: &[(&str, Value)]) -> Value {
        let answers: Map<String, Value> = questions
            .as_object()
            .expect("questions")
            .iter()
            .map(|(id, question)| {
                let given = chosen.iter().find(|(key, _)| key == id).map(|(_, v)| v);
                let answer = if question["type"] == "noul" {
                    json!({ "type": "noul", "noul": given.cloned().unwrap_or(json!(0.0)) })
                } else {
                    json!({ "type": "choice", "choice": given.cloned().unwrap_or(json!(NOTHING)) })
                };
                (id.clone(), answer)
            })
            .collect();
        json!({ "answers": answers })
    }

    /// The line read as a move to the closet, as System One reads a reported
    /// intention.
    fn read_as_move(room: &Room, typed: &str) -> Outcome {
        let state = State::new(room, typed);
        let questions = request(&state);
        let reply = body(
            &questions,
            &[
                ("intent", json!("move")),
                ("target_move", json!("way_1")),
                ("target_talk", json!("person_1")),
                ("target_present", json!(0.84)),
                ("named_more_than_one", json!(0.19)),
            ],
        );
        compose(&state, &questions, Ok(&reply), Escalation::On)
    }

    #[test]
    fn a_reported_move_is_the_talk_to_whoever_is_told() {
        let room = room(vec![rowe()]);
        let outcome = read_as_move(&room, "tell Rowe I am going into the closet");
        let intent = outcome.intent.expect("composed");
        assert_eq!(outcome.path, "typed_model");
        assert_eq!(intent.action, "talk");
        assert_eq!(intent.speaker, Some(Record::Person(rowe())));
        assert_eq!(intent.destination, None);
        assert!(!intent.refused());
    }

    #[test]
    fn a_reported_move_with_nobody_to_tell_reaches_for_nobody() {
        let room = room(Vec::new());
        let outcome = read_as_move(&room, "tell her you are going to climb the bell");
        let intent = outcome.intent.expect("composed");
        assert_eq!(outcome.path, "typed_model");
        assert_eq!(intent.action, "talk");
        assert_eq!(intent.subject(), None);
        assert!(intent.reached_for_nothing());
    }

    #[test]
    fn a_move_that_goes_on_to_tell_somebody_is_still_a_move() {
        let room = room(vec![rowe()]);
        for typed in [
            "go into the closet and tell Rowe what I found",
            "walk into the closet",
            "Telling nobody, go into the closet",
        ] {
            let intent = read_as_move(&room, typed).intent.expect("composed");
            assert_eq!(intent.action, "move", "{typed}");
            assert_eq!(
                intent.destination.map(|d| d.label()),
                Some("The Supply Closet".to_string()),
                "{typed}"
            );
        }
    }

    #[test]
    fn a_line_opens_by_speaking_only_on_its_first_word() {
        for typed in [
            "tell Rowe I am going into the closet",
            "Tell me plainly, Rowe.",
            "ask: can I go?",
            "  whisper to Rowe that I am leaving",
            "say goodbye and go",
        ] {
            assert!(spoken(typed), "{typed}");
        }
        for typed in [
            "go back to the court and tell Brace what Neb said",
            "Rowe, tell me what you found",
            "tellurium is in the closet",
            "",
        ] {
            assert!(!spoken(typed), "{typed}");
        }
    }
}
