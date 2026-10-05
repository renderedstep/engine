//! The model calls: where they go, what they send, what the engine accepts
//! back, and what they leave in the database.
//!
//! A turn asks through [`Models`], and never builds a request of its own: the
//! request builders ([`crate::arrival`], [`crate::realization`],
//! [`crate::dialogue`], [`crate::cascade`], [`crate::volition`]) write each
//! call, and a [`wire::Call`] carries it. Two implementations answer:
//!
//! - [`Live`], which sends the call over a [`Route`] (the player's own key
//!   straight to OpenRouter, or the owner's relay) under `BaseAgent`'s policy:
//!   the model rotation, a refusal or an ignored schema asking the next
//!   model, a crisis answer suppressed and never rotated past (and never
//!   streamed as far as the words it is suppressed for), a rejected key
//!   never rotated past, and the conversation written to `chats` and
//!   `messages` as it goes;
//! - [`Replay`], which answers each call with the next of a list of fixed
//!   replies tagged by purpose, checking what each prompt includes and
//!   leaves out, as the engine sweep's browser steps do. It sends nothing.
//!
//! No key or token is ever logged, printed or written: [`route::Secret`]
//! has no `Display` and a redacted `Debug`, and a failure is described by
//! the provider's own message, never by the request.

pub mod declined;
pub mod http;
mod live;
pub mod receipts;
mod replay;
pub mod route;
pub mod system_one;
pub mod wire;

pub use live::{remote_models, Live, MAX_ATTEMPTS, REMOTE_MODEL_IDS};
pub use receipts::Filed;
pub use replay::{Replay, Reply};
pub use route::{Preference, Route, Secret};
pub use wire::Call;

use crate::engine::Error;
use crate::records::Records;
use crate::store::Store;
use serde_json::Value;

/// Where a call's receipts go: the database, and the turn's copy of its
/// rows, so a request built after this call reads the conversation it left.
pub struct Book<'a> {
    pub store: &'a Store,
    pub records: &'a mut Records,
}

/// One conversation, as a caller holds it for the length of a turn
/// (`BaseAgent`'s instance): who it is filed under, the conversation it
/// continues, and which model it is on.
#[derive(Clone, Debug)]
pub struct Agent {
    pub filed: Filed,
    chat: Option<i64>,
    model: usize,
    recorded: Vec<i64>,
}

impl Agent {
    pub fn new(filed: Filed) -> Agent {
        Agent {
            filed,
            chat: None,
            model: 0,
            recorded: Vec::new(),
        }
    }

    /// An agent that continues a conversation already in the database.
    pub fn continuing(filed: Filed, chat: Option<i64>) -> Agent {
        Agent {
            chat,
            ..Agent::new(filed)
        }
    }

    pub fn purpose(&self) -> &str {
        &self.filed.purpose
    }

    /// The conversation's row, once something was asked in it.
    pub fn chat(&self) -> Option<i64> {
        self.chat
    }

    /// The messages this agent wrote, which a turn's scene claims.
    pub fn recorded(&self) -> &[i64] {
        &self.recorded
    }
}

/// What a model answered: prose as a string, a schema'd answer as the object
/// it parsed to.
#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub content: Value,
    pub model: Option<String>,
}

impl Answer {
    pub fn text(&self) -> &str {
        self.content.as_str().unwrap_or_default()
    }
}

/// Every way a chat call fails (`BaseAgent`'s errors).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// There is no model to ask (`NoModelConfiguredError`).
    NoModel,
    /// The route's credential was refused; no other model is asked
    /// (`UnauthorizedProviderError`).
    Unauthorized(String),
    /// Real-world crisis resources, suppressed and never rotated past
    /// (`CrisisResponseError`).
    Crisis(String),
    /// Prose about the request instead of prose answering it
    /// (`RefusalError`).
    Refused(String),
    /// A schema'd call answered in prose or left a field out
    /// (`SchemaIgnoredError`).
    SchemaIgnored(String),
    /// The caller's own check on the parsed answer failed.
    Rejected(String),
    /// The provider failed, or could not be reached.
    Provider(String),
    /// A replayed provider that a step declares unavailable.
    Unavailable(String),
    /// A replayed call nobody declared, or one out of order.
    Unexpected(String),
}

impl Failure {
    /// The failure a kind names, as the extension reports one (`no_model`,
    /// `unauthorized`, `crisis`, `refused`, `schema_ignored`, `rejected`,
    /// `provider`, `unavailable`); anything else is unexpected.
    pub fn named(kind: &str, message: String) -> Failure {
        match kind {
            "no_model" => Failure::NoModel,
            "unauthorized" => Failure::Unauthorized(message),
            "crisis" => Failure::Crisis(message),
            "refused" => Failure::Refused(message),
            "schema_ignored" => Failure::SchemaIgnored(message),
            "rejected" => Failure::Rejected(message),
            "provider" => Failure::Provider(message),
            "unavailable" => Failure::Unavailable(message),
            _ => Failure::Unexpected(message),
        }
    }

    /// `BaseAgent::UnusableResponseError`: text that must never be kept.
    pub fn unusable(&self) -> bool {
        matches!(self, Failure::Refused(_) | Failure::Crisis(_))
    }

    pub fn crisis(&self) -> bool {
        matches!(self, Failure::Crisis(_))
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::NoModel => f.write_str(
                "no model is configured: add your own OpenRouter key, or ask the maintainer for a relay invitation",
            ),
            Failure::Unauthorized(m)
            | Failure::Crisis(m)
            | Failure::Refused(m)
            | Failure::SchemaIgnored(m)
            | Failure::Rejected(m)
            | Failure::Provider(m)
            | Failure::Unavailable(m)
            | Failure::Unexpected(m) => f.write_str(m),
        }
    }
}

/// A System One request that could not be answered, for any reason
/// (`SystemOneAgent::Unavailable`). Its caller makes the call it would have
/// made anyway.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unavailable(pub String);

/// A caller's check on a parsed answer, run inside the attempt loop so a
/// rejected answer asks the next model (`BaseAgent#ask`'s `verify:`).
pub type Verify<'v> = &'v mut dyn FnMut(&Value) -> Result<(), String>;

/// What a turn asks its models through.
pub trait Models {
    /// One chat call (`BaseAgent#ask`). With `on_chunk` the answer streams,
    /// and prose goes there as it arrives, as far as
    /// [`declined::Held`] can tell it will be kept; a rotation streams both
    /// attempts.
    fn ask(
        &mut self,
        book: &mut Book,
        agent: &mut Agent,
        call: &Call,
        verify: Option<Verify>,
        on_chunk: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Answer, Failure>;

    /// Whether System One is on (`SystemOneAgent.configured?`).
    fn system_one(&self) -> bool;

    /// One System One request (`SystemOneAgent#ask_questions`): the answer
    /// payload, which `cascade::Answers` verifies against the questions.
    fn ask_questions(
        &mut self,
        book: &mut Book,
        filed: &Filed,
        state: &Value,
        questions: &Value,
    ) -> Result<Value, Unavailable>;

    /// Puts an exchange a conversation lost back into it before it is
    /// continued (`BaseAgent#add_message`, twice): the line asked and the
    /// answer given, under `system`'s instructions. Returns the
    /// conversation it wrote, if it keeps one.
    fn restore(
        &mut self,
        book: &mut Book,
        agent: &mut Agent,
        system: Option<&str>,
        user: &str,
        answer: &Value,
    ) -> Result<Option<i64>, Error> {
        let _ = (book, agent, system, user, answer);
        Ok(None)
    }

    /// Stamps an agent's messages with the scene its turn wrote
    /// (`BaseAgent#attribute_to!`).
    fn attribute(&mut self, book: &mut Book, agent: &Agent, scene: i64) -> Result<(), Error> {
        receipts::attribute(book, agent.recorded(), scene)
    }
}
