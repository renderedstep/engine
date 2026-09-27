//! Posting a body and reading what comes back, over HTTPS.
//!
//! [`Transport`] is the seam: [`Https`] is the real one, and a test stands a
//! transport of its own in its place, so nothing in this crate's tests
//! reaches a provider. A failure is described by what happened (the status
//! and the provider's own message, or the connection that failed), never by
//! the request or the credential that went with it.

use super::route::Endpoint;
use serde_json::Value;
use std::io::{BufRead, BufReader};
use std::time::Duration;

/// How one request is sent.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The longest the whole exchange may take; none waits as long as the
    /// provider does.
    pub timeout: Option<Duration>,
    /// Headers beside `Authorization` and `Content-Type`.
    pub headers: Vec<(&'static str, &'static str)>,
}

/// An answer: its status, and its body when it was not streamed or was an
/// error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Posted {
    pub status: u16,
    pub body: String,
}

impl Posted {
    pub fn success(&self) -> bool {
        (200..300).contains(&self.status)
    }
}

/// The request never reached an answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unreached(pub String);

pub trait Transport {
    /// Posts `body` as JSON to `endpoint`. With `on_line`, a successful
    /// answer is read as a stream: each line goes to `on_line` as it arrives
    /// and the body returned is empty.
    fn post(
        &mut self,
        endpoint: &Endpoint,
        body: &Value,
        options: &Options,
        on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Posted, Unreached>;
}

/// The real transport: TLS with the Mozilla roots compiled in, so it
/// behaves the same on every platform.
pub struct Https {
    agent: ureq::Agent,
}

impl Default for Https {
    fn default() -> Https {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .user_agent("renderedstep-engine")
            .build()
            .into();
        Https { agent }
    }
}

impl Https {
    pub fn new() -> Https {
        Https::default()
    }
}

impl Transport for Https {
    fn post(
        &mut self,
        endpoint: &Endpoint,
        body: &Value,
        options: &Options,
        on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Posted, Unreached> {
        let mut request = self
            .agent
            .post(&endpoint.url)
            .config()
            .timeout_global(options.timeout)
            .build()
            .header(
                "Authorization",
                format!("Bearer {}", endpoint.bearer.expose()),
            )
            .header("Content-Type", "application/json");
        for (name, value) in &options.headers {
            request = request.header(*name, *value);
        }
        let response = request
            .send(body.to_string())
            .map_err(|error| Unreached(error.to_string()))?;
        let status = response.status().as_u16();
        let mut text = String::new();
        let reader = BufReader::new(response.into_body().into_reader());
        match on_line {
            Some(on_line) if (200..300).contains(&status) => {
                for line in reader.lines() {
                    let line = line.map_err(|error| Unreached(error.to_string()))?;
                    on_line(&line);
                }
            }
            _ => {
                for line in reader.lines() {
                    let line = line.map_err(|error| Unreached(error.to_string()))?;
                    text.push_str(&line);
                    text.push('\n');
                }
            }
        }
        Ok(Posted { status, body: text })
    }
}
