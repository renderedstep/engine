//! Fixed answers, in order, for a turn that must make no model call: the
//! engine sweep's browser steps (`EngineSweep::BrowserTurn`).
//!
//! Each reply names the purpose of the call it answers. Both providers
//! share one queue, so a step can pin that a line asked System One first and
//! the model second. A call out of order, or one nobody declared, fails the
//! step; so does a reply left over, and a prompt that leaves out what a
//! reply says it must include or includes what it must leave out. Those
//! misses are kept until the turn is over, because narration may fall back
//! on its own words when a call fails, and that must never hide a broken
//! assertion. A chat reply is checked against the line asked; a System One
//! reply against the state sent, as JSON.
//!
//! A System One reply names only the answers it cares about. Every other
//! question is answered with the uninteresting answer: `nothing` for a
//! choice, 0.0 for a noul.

use super::{Agent, Answer, Book, Call, Failure, Filed, Models, Unavailable, Verify};
use serde_json::{json, Map, Value};
use std::collections::VecDeque;

/// `EngineSweep::BrowserTurn::TypedAgent::PURPOSE`.
pub const SYSTEM_ONE: &str = "system_one";

/// `Playthrough::IntentSchema::NOTHING`.
const NOTHING: &str = "nothing";

/// One declared answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Reply {
    pub purpose: String,
    /// What the provider answers; none for an unavailable one.
    pub content: Option<Value>,
    pub unavailable: bool,
    pub prompt_includes: Vec<String>,
    pub prompt_excludes: Vec<String>,
}

fn strings(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_string))
            .collect(),
        Value::String(text) => vec![text.clone()],
        _ => Vec::new(),
    }
}

impl Reply {
    /// Reads one entry of a step's `replies`.
    pub fn from_value(reply: &Value) -> Result<Reply, String> {
        let purpose = reply["purpose"]
            .as_str()
            .ok_or_else(|| format!("a reply with no purpose: {reply}"))?
            .to_string();
        Ok(Reply {
            purpose,
            content: reply
                .get("content")
                .filter(|content| !content.is_null())
                .cloned(),
            unavailable: reply["unavailable"].as_bool().unwrap_or(false),
            prompt_includes: strings(&reply["prompt_includes"]),
            prompt_excludes: strings(&reply["prompt_excludes"]),
        })
    }

    /// A provider for this purpose that is unavailable (a step's `fail:`).
    pub fn unavailable(purpose: &str) -> Reply {
        Reply {
            purpose: purpose.to_string(),
            content: None,
            unavailable: true,
            prompt_includes: Vec::new(),
            prompt_excludes: Vec::new(),
        }
    }
}

/// The queue, and what was asked of it.
#[derive(Debug, Default)]
pub struct Replay {
    declared: Vec<String>,
    replies: VecDeque<Reply>,
    calls: Vec<String>,
    misses: Vec<String>,
    keyed: bool,
    sent: Vec<Value>,
}

impl Replay {
    pub fn new(replies: Vec<Reply>) -> Replay {
        Replay {
            declared: replies.iter().map(|reply| reply.purpose.clone()).collect(),
            keyed: replies.iter().any(|reply| reply.purpose == SYSTEM_ONE),
            replies: replies.into(),
            calls: Vec::new(),
            misses: Vec::new(),
            sent: Vec::new(),
        }
    }

    /// Every request answered so far, in order: a chat call as its purpose,
    /// instructions, line and schema, a System One call as its state and
    /// questions.
    pub fn sent(&self) -> &[Value] {
        &self.sent
    }

    /// The purposes asked so far, in order.
    pub fn calls(&self) -> &[String] {
        &self.calls
    }

    fn next(&mut self, purpose: &str) -> Result<Reply, String> {
        self.calls.push(purpose.to_string());
        match self.replies.front() {
            Some(reply) if reply.purpose == purpose => {
                Ok(self.replies.pop_front().expect("a reply"))
            }
            _ => Err(format!(
                "browser step unexpectedly called {}",
                ruby_inspect(purpose)
            )),
        }
    }

    fn check(&mut self, reply: &Reply, sent: &str, what: &str) {
        for text in &reply.prompt_includes {
            if !sent.contains(text.as_str()) {
                self.misses.push(format!(
                    "{} {what} omitted {}",
                    reply.purpose,
                    ruby_inspect(text)
                ));
            }
        }
        for text in &reply.prompt_excludes {
            if sent.contains(text.as_str()) {
                self.misses.push(format!(
                    "{} {what} disclosed {}",
                    reply.purpose,
                    ruby_inspect(text)
                ));
            }
        }
    }

    /// Whether the turn asked exactly what was declared, in order, and every
    /// prompt said what its reply asked of it.
    pub fn finish(&self) -> Result<(), String> {
        if !self.misses.is_empty() {
            return Err(self.misses.join("; "));
        }
        if self.calls != self.declared {
            return Err(format!(
                "browser step expected {} rendering calls, got {}",
                inspect_list(&self.declared),
                inspect_list(&self.calls)
            ));
        }
        Ok(())
    }
}

fn ruby_inspect(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

fn inspect_list(items: &[String]) -> String {
    format!(
        "[{}]",
        items
            .iter()
            .map(|item| ruby_inspect(item))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// One answer as a provider writes it.
fn answer(value: &Value) -> Value {
    if value.is_number() {
        return json!({ "type": "noul", "noul": value });
    }
    let chosen = match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    let mut probabilities = Map::new();
    probabilities.insert(chosen.clone(), json!(1.0));
    json!({ "type": "choice", "choice": chosen, "probabilities": probabilities, "confidence": 1.0 })
}

impl Models for Replay {
    fn ask(
        &mut self,
        _book: &mut Book,
        agent: &mut Agent,
        call: &Call,
        verify: Option<Verify>,
        _on_chunk: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Answer, Failure> {
        self.sent.push(json!({
            "purpose": agent.purpose(),
            "system": call.system,
            "user": call.user,
            "schema": call.schema,
        }));
        let reply = self.next(agent.purpose()).map_err(Failure::Unexpected)?;
        self.check(&reply, &call.user, "prompt");
        if reply.unavailable {
            return Err(Failure::Unavailable(
                "the sweep's provider is unavailable".into(),
            ));
        }
        let content = reply.content.clone().ok_or_else(|| {
            Failure::Unexpected(format!("a {} reply with no content", reply.purpose))
        })?;
        if let Some(check) = verify {
            check(&content).map_err(Failure::Rejected)?;
        }
        Ok(Answer {
            content,
            model: None,
        })
    }

    fn system_one(&self) -> bool {
        self.keyed
    }

    fn ask_questions(
        &mut self,
        _book: &mut Book,
        _filed: &Filed,
        state: &Value,
        questions: &Value,
    ) -> Result<Value, Unavailable> {
        self.sent
            .push(json!({ "purpose": SYSTEM_ONE, "state": state, "questions": questions }));
        let reply = self.next(SYSTEM_ONE).map_err(Unavailable)?;
        let sent = serde_json::to_string(state).unwrap_or_default();
        self.check(&reply, &sent, "state");
        if reply.unavailable {
            return Err(Unavailable(
                "the sweep's System One provider is unavailable".into(),
            ));
        }
        let named = reply.content.clone().unwrap_or(Value::Null);
        let answers: Map<String, Value> = questions
            .as_object()
            .map(|questions| {
                questions
                    .iter()
                    .map(|(id, question)| {
                        let given = match named.get(id) {
                            Some(value) => answer(value),
                            None if question["type"] == "noul" => answer(&json!(0.0)),
                            None => answer(&json!(NOTHING)),
                        };
                        (id.clone(), given)
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(json!({ "answers": answers, "usage": { "input_tokens": 0, "output_tokens": 0 } }))
    }

    fn attribute(
        &mut self,
        _book: &mut Book,
        _agent: &Agent,
        _scene: i64,
    ) -> Result<(), crate::engine::Error> {
        Ok(())
    }
}
