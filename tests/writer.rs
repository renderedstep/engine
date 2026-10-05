//! A turn waiting on a model holds no write on the database: another
//! connection writes while the call is out, and the turn still finishes.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::http::{Options, Posted, Transport, Unreached};
use renderedstep_engine::model::route::Endpoint;
use renderedstep_engine::model::{Live, Route, Secret};
use renderedstep_engine::parity::TITLE_SUFFIX;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// A database file of its own under the system's temporary directory,
/// removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Scratch {
        let sql = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-turn-at-the-gate.sql"),
        )
        .expect("a world fixture");
        let path = std::env::temp_dir().join(format!(
            "renderedstep-engine-{name}-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        rusqlite::Connection::open(&path)
            .and_then(|conn| conn.execute_batch(&sql))
            .expect("the fixture loads");
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// A provider that, before it answers each call, writes to the database on
/// a connection of its own that waits for nobody: a write the turn held open
/// across the call would make that write fail at once.
struct Writing {
    database: PathBuf,
    answers: Vec<Value>,
    wrote: Vec<Result<(), String>>,
}

impl Transport for Writing {
    fn post(
        &mut self,
        _endpoint: &Endpoint,
        _body: &Value,
        _options: &Options,
        _on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Posted, Unreached> {
        let wrote = rusqlite::Connection::open(&self.database)
            .and_then(|conn| {
                conn.busy_timeout(std::time::Duration::ZERO)?;
                conn.execute("UPDATE stories SET updated_at = updated_at", [])
            })
            .map(|_| ())
            .map_err(|error| error.to_string());
        self.wrote.push(wrote);
        let content = match self.answers.remove(0) {
            Value::String(text) => text,
            other => other.to_string(),
        };
        Ok(Posted {
            status: 200,
            body: json!({"choices": [{"message": {"content": content}, "finish_reason": "stop"}]})
                .to_string(),
        })
    }
}

#[test]
fn another_connection_can_write_while_a_turn_waits_on_its_model() {
    let scratch = Scratch::new("writer");
    let mut engine = Engine::open(&scratch.0).expect("the database opens");
    let story = engine
        .story_titled(&format!("A Turn at the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    let reaction = json!({
        "pre_thought": "I will listen.", "pre_feeling": "attentive", "action": "Maren nods.",
        "post_thought": "That was fair.", "post_feeling": "calm",
        "inner_resolution": "I will sweep the market.", "engine_action": "none",
    });
    let transport = Writing {
        database: scratch.0.clone(),
        answers: vec![reaction, json!("Maren nods to you.")],
        wrote: Vec::new(),
    };
    let route = Route::Direct {
        key: Secret::new("own"),
    };
    let mut live = Live::with_transport(route, transport);
    let submitted = engine
        .submit(playthrough, "/talk Maren", "one", &mut live, &mut |_| {})
        .unwrap();
    assert!(submitted.turned.scene.is_some(), "the turn finished");
    let wrote = &live.transport().wrote;
    assert_eq!(wrote.len(), 2, "both of the turn's calls were made");
    assert!(
        wrote.iter().all(Result::is_ok),
        "a write on another connection waited on the turn: {wrote:?}"
    );
}
