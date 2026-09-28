//! `BaseAgent`'s policy, over a [`Route`]: which model is asked, what makes
//! an answer a failed call, and what is written as it goes.

use super::http::{Https, Options, Transport};
use super::route::{Route, CHAT_COMPLETIONS};
use super::system_one::SystemOne;
use super::wire::{self, Call, Completion};
use super::{declined, receipts, Agent, Answer, Book, Failure, Filed, Models, Unavailable, Verify};
use serde_json::Value;
use std::time::Duration;

/// The hosted models, in the order they are asked
/// (`BaseAgent::REMOTE_MODEL_IDS`). Every one must honour a JSON schema.
pub const REMOTE_MODEL_IDS: [&str; 2] = ["mistralai/mistral-medium-3.1", "minimax/minimax-m3"];

/// `BaseAgent::MAX_ATTEMPTS`.
pub const MAX_ATTEMPTS: usize = 3;

/// How long a chat call may take (`LLM_REQUEST_TIMEOUT`'s default).
pub const CHAT_TIMEOUT: Duration = Duration::from_secs(600);

/// The rotation, with a chosen model asked first when there is one
/// (`BaseAgent.remote_model_options` with `OPENROUTER_MODEL`).
pub fn remote_models(first: Option<&str>) -> Vec<String> {
    let mut models: Vec<String> = Vec::new();
    for model in first
        .filter(|model| !model.trim().is_empty())
        .into_iter()
        .chain(REMOTE_MODEL_IDS)
    {
        if !models.iter().any(|known| known == model) {
            models.push(model.to_string());
        }
    }
    models
}

/// The models, asked for real.
pub struct Live<T: Transport = Https> {
    route: Route,
    models: Vec<String>,
    transport: T,
    system_one: SystemOne,
}

impl Live<Https> {
    /// Asks over `route` through HTTPS, System One on OpenRouter's decisions
    /// route unless [`Live::with_system_one`] says otherwise.
    pub fn new(route: Route) -> Live<Https> {
        Live::with_transport(route, Https::new())
    }
}

impl<T: Transport> Live<T> {
    pub fn with_transport(route: Route, transport: T) -> Live<T> {
        let models = if route.is_none() {
            Vec::new()
        } else {
            remote_models(None)
        };
        let system_one = if route.is_none() {
            SystemOne::Off
        } else {
            SystemOne::Decisions
        };
        Live {
            route,
            models,
            transport,
            system_one,
        }
    }

    /// Asks this model first.
    pub fn with_model(mut self, first: &str) -> Live<T> {
        if !self.route.is_none() {
            self.models = remote_models(Some(first));
        }
        self
    }

    /// Which System One transport answers.
    pub fn with_system_one(mut self, system_one: SystemOne) -> Live<T> {
        self.system_one = system_one;
        self
    }

    pub fn route(&self) -> &Route {
        &self.route
    }

    pub fn models(&self) -> &[String] {
        &self.models
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    fn send(
        &mut self,
        model: &str,
        call: &Call,
        on_chunk: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Completion, Sent> {
        let endpoint = self
            .route
            .endpoint(CHAT_COMPLETIONS)
            .ok_or(Sent::Failed("no model access".into()))?;
        let body = wire::chat_body(model, call, on_chunk.is_some());
        let options = Options {
            timeout: Some(CHAT_TIMEOUT),
            headers: vec![
                ("HTTP-Referer", "https://github.com/renderedstep/engine"),
                ("X-OpenRouter-Title", "renderedstep"),
            ],
        };
        match on_chunk {
            Some(on_chunk) => {
                let mut stream = wire::Stream::new();
                let posted = self
                    .transport
                    .post(
                        &endpoint,
                        &body,
                        &options,
                        Some(&mut |line: &str| stream.line(line, on_chunk)),
                    )
                    .map_err(|unreached| Sent::Failed(unreached.0))?;
                if !posted.success() {
                    return Err(Sent::status(posted.status, &posted.body));
                }
                stream.finish().map_err(Sent::Failed)
            }
            None => {
                let posted = self
                    .transport
                    .post(&endpoint, &body, &options, None)
                    .map_err(|unreached| Sent::Failed(unreached.0))?;
                if !posted.success() {
                    return Err(Sent::status(posted.status, &posted.body));
                }
                let parsed: Value = serde_json::from_str(&posted.body).map_err(|_| {
                    Sent::Failed("the provider answered something that is not JSON".into())
                })?;
                wire::completion(&parsed).map_err(Sent::Failed)
            }
        }
    }
}

/// How sending went wrong.
enum Sent {
    Unauthorized(String),
    Failed(String),
}

impl Sent {
    fn status(status: u16, body: &str) -> Sent {
        let said = serde_json::from_str::<Value>(body)
            .ok()
            .and_then(|body| wire::error_message(&body))
            .unwrap_or_default();
        let message = if said.is_empty() {
            format!("the provider answered {status}")
        } else {
            format!("the provider answered {status}: {said}")
        };
        if status == 401 {
            Sent::Unauthorized(message)
        } else {
            Sent::Failed(message)
        }
    }
}

/// Ruby's `present?` on a parsed field: nil, an empty or blank string, an
/// empty list and an empty object are all absent.
fn present(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::String(text)) => !text.trim().is_empty(),
        Some(Value::Array(items)) => !items.is_empty(),
        Some(Value::Object(map)) => !map.is_empty(),
        Some(_) => true,
    }
}

/// `verify_schema_honored!`, then the crisis and refusal reads of prose.
fn judge(model: &str, call: &Call, completion: &Completion) -> Result<Value, Failure> {
    if call.schema.is_some() {
        let parsed: Value = serde_json::from_str(&completion.content)
            .unwrap_or_else(|_| Value::String(completion.content.clone()));
        let Value::Object(fields) = &parsed else {
            return Err(Failure::SchemaIgnored(format!(
                "{model} ignored the schema and returned prose"
            )));
        };
        let missing: Vec<String> = call
            .required()
            .into_iter()
            .filter(|key| {
                !(present(fields.get(key)) || fields.get(key) == Some(&Value::Bool(false)))
            })
            .collect();
        if !missing.is_empty() {
            return Err(Failure::SchemaIgnored(format!(
                "{model} omitted schema fields: {}",
                missing.join(", ")
            )));
        }
        return Ok(parsed);
    }
    let text = &completion.content;
    if declined::crisis_response(text) {
        return Err(Failure::Crisis(format!(
            "{model} answered with real-world crisis resources"
        )));
    }
    if declined::refused(text) {
        let flags: Vec<&str> = declined::flags(text)
            .iter()
            .map(|flag| flag.name())
            .collect();
        return Err(Failure::Refused(format!(
            "{model} declined the prompt instead of answering it ({})",
            flags.join(", ")
        )));
    }
    Ok(Value::String(text.clone()))
}

impl<T: Transport> Models for Live<T> {
    fn ask(
        &mut self,
        book: &mut Book,
        agent: &mut Agent,
        call: &Call,
        mut verify: Option<Verify>,
        mut on_chunk: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Answer, Failure> {
        let count = self.models.len();
        if count == 0 {
            return Err(Failure::NoModel);
        }
        let mut attempts = 0;
        loop {
            attempts += 1;
            let model = self.models[agent.model % count].clone();
            let chat = receipts::conversation(
                book,
                agent.chat,
                &agent.filed,
                &model,
                call.system.as_deref(),
            )
            .map_err(|error| Failure::Provider(error.to_string()))?;
            agent.chat = Some(chat);
            let mark = receipts::mark(book, chat);
            let written = |result: Result<(), crate::engine::Error>| {
                result.map_err(|error| Failure::Provider(error.to_string()))
            };
            written(receipts::message(book, chat, "user", &call.user, None).map(|_| ()))?;

            // Prose is shown only as far as the judgement below cannot turn
            // on it; the rest follows once the answer is kept.
            let mut held = declined::Held::new();
            let sent = match on_chunk.as_deref_mut() {
                Some(show) if call.schema.is_none() => {
                    let mut guarded = |part: &str| {
                        let shown = held.take(part);
                        if !shown.is_empty() {
                            show(shown);
                        }
                    };
                    self.send(&model, call, Some(&mut guarded))
                }
                streaming => self.send(&model, call, streaming),
            };
            let judged = match sent {
                Err(Sent::Unauthorized(message)) => {
                    written(receipts::usage(book, chat, None, &model, None))?;
                    written(receipts::rewind(book, chat, mark))?;
                    return Err(Failure::Unauthorized(format!(
                        "{} refused the credential ({message}); not asking another model, because a refused credential is not fixed by a different model",
                        self.route.describe()
                    )));
                }
                Err(Sent::Failed(message)) => {
                    written(receipts::usage(book, chat, None, &model, None))?;
                    Err(Failure::Provider(format!("{model} failed: {message}")))
                }
                Ok(completion) => {
                    let answered = receipts::message(
                        book,
                        chat,
                        "assistant",
                        &completion.content,
                        completion.finish_reason.as_deref(),
                    );
                    let answered = written(answered.map(|_| ())).and_then(|_| {
                        let id = receipts::mark(book, chat);
                        written(receipts::usage(
                            book,
                            chat,
                            Some(id),
                            &model,
                            Some(&completion.usage),
                        ))
                    });
                    answered
                        .and_then(|_| judge(&model, call, &completion))
                        .and_then(|content| match verify.as_mut() {
                            Some(check) => {
                                check(&content).map(|_| content).map_err(Failure::Rejected)
                            }
                            None => Ok(content),
                        })
                }
            };
            match judged {
                Ok(content) => {
                    if let Some(show) = on_chunk.as_deref_mut() {
                        let rest = held.rest();
                        if !rest.is_empty() {
                            show(rest);
                        }
                    }
                    let exchanged = receipts::since(book, chat, mark);
                    agent.recorded.extend(exchanged);
                    return Ok(Answer {
                        content,
                        model: Some(model),
                    });
                }
                Err(failure) => {
                    written(receipts::rewind(book, chat, mark))?;
                    if !failure.crisis() && attempts < MAX_ATTEMPTS && attempts < count {
                        agent.model = (agent.model + 1) % count;
                        continue;
                    }
                    return Err(failure);
                }
            }
        }
    }

    fn restore(
        &mut self,
        book: &mut Book,
        agent: &mut Agent,
        system: Option<&str>,
        user: &str,
        answer: &Value,
    ) -> Result<Option<i64>, crate::engine::Error> {
        let Some(model) = self
            .models
            .get(agent.model % self.models.len().max(1))
            .cloned()
        else {
            return Ok(None);
        };
        let chat = receipts::conversation(book, agent.chat, &agent.filed, &model, system)?;
        agent.chat = Some(chat);
        receipts::message(book, chat, "user", user, None)?;
        receipts::structured_message(book, chat, "assistant", answer)?;
        Ok(Some(chat))
    }

    fn system_one(&self) -> bool {
        self.system_one.configured(&self.route)
    }

    fn ask_questions(
        &mut self,
        book: &mut Book,
        filed: &Filed,
        state: &Value,
        questions: &Value,
    ) -> Result<Value, Unavailable> {
        self.system_one.ask(
            &mut self.transport,
            &self.route,
            book,
            filed,
            state,
            questions,
        )
    }
}
