//! What goes over the wire and what comes back: a builder's request turned
//! into the chat completions body RubyLLM's OpenRouter provider sends, and
//! an answer, whole or streamed, read back into prose, a model id, a finish
//! reason and the tokens it cost.
//!
//! The body is written key for key as that provider writes it (`model`,
//! `messages`, `stream`, then `temperature`, `response_format` and
//! `stream_options` where they apply), with the instructions sent in the
//! `developer` role and a schema's name made safe the way RubyLLM makes it
//! (`Location::DetailSchema` goes as `Location__DetailSchema`).

use serde_json::{json, Map, Value};

/// One message a conversation replays.
#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub role: String,
    /// Null for a message with no text at all.
    pub content: Value,
}

/// One call, as the request builders describe it: the instructions, the
/// line asked, the schema the answer must take, and the conversation it
/// continues. Two things the builders leave to the caller ride along: the
/// temperature and whether the answer streams.
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub system: Option<String>,
    pub user: String,
    /// A `to_json_schema` output, `{name, description, schema}`.
    pub schema: Option<Value>,
    pub history: Vec<Message>,
    pub temperature: Option<Value>,
}

impl Call {
    /// A call with nothing but the line.
    pub fn prompt(user: impl Into<String>) -> Call {
        Call {
            system: None,
            user: user.into(),
            schema: None,
            history: Vec::new(),
            temperature: None,
        }
    }

    /// Reads a builder's `{system, user, schema, history}`.
    pub fn from_request(request: &Value) -> Call {
        let history = request["history"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .map(|row| Message {
                        role: row["role"].as_str().unwrap_or_default().to_string(),
                        content: row["content"].clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Call {
            system: request["system"].as_str().map(str::to_string),
            user: request["user"].as_str().unwrap_or_default().to_string(),
            schema: request
                .get("schema")
                .filter(|schema| !schema.is_null())
                .cloned(),
            history,
            temperature: None,
        }
    }

    /// The call back as a builder writes it.
    pub fn to_request(&self) -> Value {
        json!({
            "system": self.system,
            "user": self.user,
            "schema": self.schema,
            "history": self
                .history
                .iter()
                .map(|message| json!({ "role": message.role, "content": message.content }))
                .collect::<Vec<_>>(),
        })
    }

    pub fn with_temperature(mut self, temperature: impl Into<Value>) -> Call {
        self.temperature = Some(temperature.into());
        self
    }

    /// The fields a schema'd answer must carry (`required_properties`).
    pub fn required(&self) -> Vec<String> {
        self.schema
            .as_ref()
            .and_then(|schema| schema["schema"]["required"].as_array())
            .map(|keys| {
                keys.iter()
                    .filter_map(|key| key.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// `sanitize_schema_name`: every character outside `[a-zA-Z0-9_-]` becomes
/// an underscore.
pub fn schema_name(name: &str) -> String {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if safe.is_empty() {
        "response".to_string()
    } else {
        safe
    }
}

/// `strict_schema?`: false as soon as any object leaves a property out of
/// `required`.
fn strict_schema(node: &Value) -> bool {
    match node {
        Value::Object(map) => {
            if let Some(Value::Object(properties)) = map.get("properties") {
                let required: Vec<&str> = map
                    .get("required")
                    .and_then(Value::as_array)
                    .map(|keys| keys.iter().filter_map(Value::as_str).collect())
                    .unwrap_or_default();
                if properties
                    .keys()
                    .any(|key| !required.contains(&key.as_str()))
                {
                    return false;
                }
            }
            map.values().all(strict_schema)
        }
        Value::Array(items) => items.iter().all(strict_schema),
        _ => true,
    }
}

/// The `response_format` a schema is sent as: its definition without the
/// `strict` flag it carries, which moves up beside the name.
fn response_format(schema: &Value) -> Value {
    let mut definition = schema
        .get("schema")
        .cloned()
        .unwrap_or_else(|| schema.clone());
    let strict = match definition
        .as_object_mut()
        .and_then(|map| map.remove("strict"))
    {
        Some(flag) if !flag.is_null() => flag,
        _ => Value::Bool(strict_schema(&definition)),
    };
    let strict = schema.get("strict").cloned().unwrap_or(strict);
    if let Some(map) = definition.as_object_mut() {
        for key in ["$schema", "$id", "title"] {
            map.remove(key);
        }
    }
    json!({
        "type": "json_schema",
        "json_schema": {
            "name": schema_name(schema["name"].as_str().unwrap_or("response")),
            "schema": definition,
            "strict": strict,
        },
    })
}

fn message(role: &str, content: &Value) -> Value {
    let mut out = Map::new();
    out.insert("role".into(), Value::from(role));
    if !content.is_null() {
        out.insert("content".into(), content.clone());
    }
    Value::Object(out)
}

/// The chat completions body for one call to one model.
pub fn chat_body(model: &str, call: &Call, stream: bool) -> Value {
    let mut messages = Vec::new();
    match &call.system {
        Some(system) => messages.push(message("developer", &Value::from(system.as_str()))),
        None => messages.extend(
            call.history
                .iter()
                .filter(|m| m.role == "system")
                .map(|m| message("developer", &m.content)),
        ),
    }
    for replayed in call.history.iter().filter(|m| m.role != "system") {
        messages.push(message(&replayed.role, &replayed.content));
    }
    messages.push(message("user", &Value::from(call.user.as_str())));

    let mut body = Map::new();
    body.insert("model".into(), Value::from(model));
    body.insert("messages".into(), Value::Array(messages));
    body.insert("stream".into(), Value::Bool(stream));
    if let Some(temperature) = &call.temperature {
        body.insert("temperature".into(), temperature.clone());
    }
    if let Some(schema) = &call.schema {
        body.insert("response_format".into(), response_format(schema));
    }
    if stream {
        body.insert("stream_options".into(), json!({ "include_usage": true }));
    }
    Value::Object(body)
}

/// What an answer cost, as OpenRouter's usage block reports it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Usage {
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_read_tokens: Option<i64>,
    pub cache_write_tokens: Option<i64>,
    pub thinking_tokens: Option<i64>,
    /// The cost OpenRouter charged, in dollars.
    pub cost: Option<f64>,
}

impl Usage {
    fn read(usage: &Value) -> Option<Usage> {
        let usage = usage.as_object()?;
        let number = |value: Option<&Value>| value.and_then(Value::as_i64);
        let mut cost = usage.get("cost").and_then(Value::as_f64);
        if usage.get("is_byok").and_then(Value::as_bool) == Some(true) {
            cost = cost.map(|cost| {
                cost + usage
                    .get("cost_details")
                    .and_then(|details| details.get("upstream_inference_cost"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0)
            });
        }
        Some(Usage {
            input_tokens: number(usage.get("prompt_tokens")),
            output_tokens: number(usage.get("completion_tokens")),
            cache_read_tokens: number(
                usage
                    .get("prompt_tokens_details")
                    .and_then(|details| details.get("cached_tokens")),
            ),
            cache_write_tokens: number(
                usage
                    .get("prompt_tokens_details")
                    .and_then(|details| details.get("cache_write_tokens")),
            ),
            thinking_tokens: number(
                usage
                    .get("completion_tokens_details")
                    .and_then(|details| details.get("reasoning_tokens")),
            ),
            cost,
        })
    }
}

/// One finished answer.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Completion {
    pub content: String,
    pub model: Option<String>,
    pub finish_reason: Option<String>,
    pub usage: Usage,
}

/// The provider's own words for a failure it reported in a body, never the
/// request that caused it.
pub fn error_message(body: &Value) -> Option<String> {
    let error = body.get("error")?;
    match error {
        Value::String(text) => Some(text.clone()),
        Value::Object(map) => map
            .get("message")
            .and_then(Value::as_str)
            .map(str::to_string),
        _ => None,
    }
}

fn text_of(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .collect(),
        _ => String::new(),
    }
}

/// A whole answer's body.
pub fn completion(body: &Value) -> Result<Completion, String> {
    if let Some(message) = error_message(body) {
        return Err(message);
    }
    let choice = &body["choices"][0];
    let message = choice
        .get("message")
        .filter(|message| message.is_object())
        .ok_or_else(|| "the provider answered with no message".to_string())?;
    Ok(Completion {
        content: text_of(&message["content"]),
        model: body["model"].as_str().map(str::to_string),
        finish_reason: choice["finish_reason"].as_str().map(str::to_string),
        usage: Usage::read(&body["usage"]).unwrap_or_default(),
    })
}

/// A streamed answer being read, one server-sent line at a time.
#[derive(Debug, Default)]
pub struct Stream {
    answer: Completion,
    done: bool,
    failed: Option<String>,
}

impl Stream {
    pub fn new() -> Stream {
        Stream::default()
    }

    /// Reads one line of the stream, handing any prose it carries to
    /// `on_chunk`. Comments, blank lines and the closing `[DONE]` carry none.
    pub fn line(&mut self, line: &str, on_chunk: &mut dyn FnMut(&str)) {
        let Some(data) = line.strip_prefix("data:") else {
            return;
        };
        let data = data.trim();
        if data == "[DONE]" {
            self.done = true;
            return;
        }
        let Ok(event) = serde_json::from_str::<Value>(data) else {
            return;
        };
        if let Some(message) = error_message(&event) {
            self.failed.get_or_insert(message);
            return;
        }
        if let Some(model) = event["model"].as_str() {
            self.answer.model = Some(model.to_string());
        }
        let choice = &event["choices"][0];
        if let Some(part) = choice["delta"]["content"].as_str() {
            if !part.is_empty() {
                self.answer.content.push_str(part);
                on_chunk(part);
            }
        }
        if let Some(reason) = choice["finish_reason"].as_str() {
            self.answer.finish_reason = Some(reason.to_string());
        }
        if let Some(usage) = Usage::read(&event["usage"]) {
            self.answer.usage = usage;
        }
    }

    /// The answer the stream carried, or the failure it reported.
    pub fn finish(self) -> Result<Completion, String> {
        match self.failed {
            Some(message) => Err(message),
            None => Ok(self.answer),
        }
    }
}

/// The System One body (`SystemOneAgent#ask_questions`): the model, the
/// state and the questions, the same on both transports.
pub fn decisions_body(model: &str, state: &Value, questions: &Value) -> Value {
    json!({ "model": model, "state": state, "questions": questions })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exits_schema() -> Value {
        json!({
            "name": "Location::ExitsSchema",
            "description": null,
            "schema": {
                "type": "object",
                "properties": {"exits": {"type": "array", "items": {
                    "type": "object",
                    "properties": {"name": {"type": "string"}, "inside": {"type": "string"}},
                    "required": ["name"],
                    "additionalProperties": false,
                }}},
                "required": ["exits"],
                "additionalProperties": false,
                "strict": true,
            },
        })
    }

    #[test]
    fn writes_a_structured_call_as_rubyllm_writes_it() {
        let call = Call {
            system: Some("REALIZE.".into()),
            user: "And the ways out.".into(),
            schema: Some(exits_schema()),
            history: vec![
                Message {
                    role: "user".into(),
                    content: json!("Describe it."),
                },
                Message {
                    role: "assistant".into(),
                    content: json!("{\"description\":\"A room.\"}"),
                },
            ],
            temperature: None,
        };
        let body = chat_body("mistralai/mistral-medium-3.1", &call, false);
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            concat!(
                r#"{"model":"mistralai/mistral-medium-3.1","messages":[{"role":"developer","content":"REALIZE."},"#,
                r#"{"role":"user","content":"Describe it."},{"role":"assistant","content":"{\"description\":\"A room.\"}"},"#,
                r#"{"role":"user","content":"And the ways out."}],"stream":false,"response_format":{"type":"json_schema","#,
                r#""json_schema":{"name":"Location__ExitsSchema","schema":{"type":"object","properties":{"exits":{"type":"array","#,
                r#""items":{"type":"object","properties":{"name":{"type":"string"},"inside":{"type":"string"}},"required":["name"],"#,
                r#""additionalProperties":false}}},"required":["exits"],"additionalProperties":false},"strict":true}}}"#
            )
        );
    }

    #[test]
    fn writes_a_streamed_prose_call_with_its_temperature() {
        let body = chat_body(
            "m",
            &Call::prompt("The player types: look").with_temperature(0),
            true,
        );
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            r#"{"model":"m","messages":[{"role":"user","content":"The player types: look"}],"stream":true,"temperature":0,"stream_options":{"include_usage":true}}"#
        );
    }

    #[test]
    fn reads_a_stream_and_its_usage() {
        let mut stream = Stream::new();
        let mut chunks = Vec::new();
        for line in [
            ": OPENROUTER PROCESSING",
            r#"data: {"model":"m","choices":[{"delta":{"content":"You look "}}]}"#,
            "",
            r#"data: {"model":"m","choices":[{"delta":{"content":"around."},"finish_reason":null}]}"#,
            r#"data: {"model":"m","choices":[{"delta":{},"finish_reason":"stop"}]}"#,
            r#"data: {"model":"m","choices":[],"usage":{"prompt_tokens":20,"completion_tokens":9,"cost":0.0007}}"#,
            "data: [DONE]",
        ] {
            stream.line(line, &mut |chunk| chunks.push(chunk.to_string()));
        }
        let answer = stream.finish().unwrap();
        assert_eq!(chunks, ["You look ", "around."]);
        assert_eq!(answer.content, "You look around.");
        assert_eq!(answer.finish_reason.as_deref(), Some("stop"));
        assert_eq!(answer.usage.input_tokens, Some(20));
        assert_eq!(answer.usage.cost, Some(0.0007));
    }

    #[test]
    fn a_reported_error_is_the_providers_words() {
        assert_eq!(
            completion(&json!({"error": {"message": "No auth credentials found", "code": 401}})),
            Err("No auth credentials found".to_string())
        );
    }
}
