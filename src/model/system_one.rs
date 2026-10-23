//! `SystemOneAgent`'s transports: TypeSafe direct, with a TypeSafe key of
//! its own; OpenRouter's decisions route over the player's [`Route`], which
//! the cascade runs from when the player has an OpenRouter key or a relay
//! invitation and nothing else; and a local reader, a process on the server's
//! own machine that answers the same body with no key at all. The body is the
//! same on every one; each names its own model.
//!
//! Every failure is [`Unavailable`], and it never blocks a turn: its caller
//! decides what an unanswered request means. A failure says the status and
//! never the body, because a refused body is about the credential.

use super::http::{Options, Transport};
use super::route::{Endpoint, Route, Secret, DECISIONS};
use super::{receipts, wire, Book, Filed, Unavailable};
use serde_json::Value;
use std::time::{Duration, Instant};

/// `SystemOneAgent::TYPESAFE_ENDPOINT`.
pub const TYPESAFE_ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";

/// `SystemOneAgent::TYPESAFE_MODEL`.
pub const TYPESAFE_MODEL: &str = "jev-1.13.0";

/// `SystemOneAgent::OPENROUTER_MODEL`.
pub const OPENROUTER_MODEL: &str = "typesafe/jev-1.13";

/// `SystemOneAgent::TIMEOUT`: a turn waits no longer than this for it.
pub const TIMEOUT: Duration = Duration::from_secs(4);

/// Which System One transport answers, if one does.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum SystemOne {
    /// TypeSafe's own route, with its own key; preferred when there is one.
    TypeSafe { key: Secret },
    /// OpenRouter's decisions route, over the chat calls' route.
    #[default]
    Decisions,
    /// A reader on the server's own machine, at `endpoint` (the whole URL),
    /// answering with the weights `model` names (their digest). It costs
    /// nothing and takes no key; either left blank, it is not set up.
    Local { endpoint: String, model: String },
    /// No System One: every line goes to the model call.
    Off,
}

impl SystemOne {
    /// Whether the cascade runs (`SystemOneAgent.configured?`).
    pub fn configured(&self, route: &Route) -> bool {
        match self {
            SystemOne::TypeSafe { key } => !key.is_blank(),
            SystemOne::Decisions => !route.is_none(),
            SystemOne::Local { endpoint, model } => {
                !endpoint.trim().is_empty() && !model.trim().is_empty()
            }
            SystemOne::Off => false,
        }
    }

    /// The transport's name as a receipt keeps it.
    pub fn name(&self) -> &'static str {
        match self {
            SystemOne::TypeSafe { .. } => "typesafe_direct",
            SystemOne::Local { .. } => "local",
            SystemOne::Decisions | SystemOne::Off => "openrouter_decisions",
        }
    }

    pub fn model(&self) -> &str {
        match self {
            SystemOne::TypeSafe { .. } => TYPESAFE_MODEL,
            SystemOne::Local { model, .. } => model,
            SystemOne::Decisions | SystemOne::Off => OPENROUTER_MODEL,
        }
    }

    /// Whether this is the local reader, set up or not.
    pub fn local(&self) -> bool {
        matches!(self, SystemOne::Local { .. })
    }

    /// What a request is booked at before any answer reports a cost: the
    /// local reader costs nothing, and the rest the conservative estimate.
    fn estimate(&self) -> f64 {
        match self {
            SystemOne::Local { .. } => 0.0,
            _ => receipts::SYSTEM_ONE_COST,
        }
    }

    /// One request of typed questions, and its answer payload.
    pub fn ask(
        &self,
        transport: &mut dyn Transport,
        route: &Route,
        book: &mut Book,
        filed: &Filed,
        state: &Value,
        questions: &Value,
    ) -> Result<Value, Unavailable> {
        if !self.configured(route) {
            return Err(Unavailable(match self {
                SystemOne::Local { .. } => "the local reader is not set up".into(),
                _ => "no System One credential here".into(),
            }));
        }
        if questions.as_object().is_none_or(|map| map.is_empty()) {
            return Err(Unavailable("a System One request with no questions".into()));
        }
        let keyless = Secret::new("");
        let endpoint = match self {
            SystemOne::TypeSafe { key } => Endpoint {
                url: TYPESAFE_ENDPOINT.to_string(),
                bearer: key,
            },
            SystemOne::Local { endpoint, .. } => Endpoint {
                url: endpoint.clone(),
                bearer: &keyless,
            },
            _ => route
                .endpoint(DECISIONS)
                .ok_or_else(|| Unavailable("no System One credential here".into()))?,
        };
        let body = wire::decisions_body(self.model(), state, questions);
        let receipt = receipts::system_one(book, filed, self.name(), self.estimate())
            .map_err(|error| Unavailable(error.to_string()))?;
        let options = Options {
            timeout: Some(TIMEOUT),
            headers: Vec::new(),
        };
        let started = Instant::now();
        let posted = transport.post(&endpoint, &body, &options, None);
        receipts::system_one_timed(book, receipt, receipts::latency_ms(started))
            .map_err(|error| Unavailable(error.to_string()))?;
        let posted = posted.map_err(|unreached| Unavailable(unreached.0))?;
        if !posted.success() {
            return Err(Unavailable(format!(
                "the provider answered {}",
                posted.status
            )));
        }
        let payload: Value = serde_json::from_str(&posted.body)
            .map_err(|_| Unavailable("the provider answered something that is not JSON".into()))?;
        if !payload.is_object() {
            return Err(Unavailable(
                "the provider answered something other than a body".into(),
            ));
        }
        // The local reader says so in its body when it will not answer a
        // question it was not trained for.
        if self.local() && payload.get("error").is_some() {
            return Err(Unavailable("the local reader answered an error".into()));
        }
        if let Some(usage) = payload.get("usage") {
            receipts::system_one_reported(book, receipt, usage)
                .map_err(|error| Unavailable(error.to_string()))?;
        }
        Ok(payload)
    }
}
