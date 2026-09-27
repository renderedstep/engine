//! The model client: the bodies it sends against what RubyLLM sent for the
//! same calls, `BaseAgent`'s policy over a transport that answers with fixed
//! replies, the receipts it leaves, the no-leak contract between the two
//! routes, and the replay the engine sweep's browser steps play with.
//!
//! Nothing here reaches a provider. The one socket opened is a server on
//! this machine, bound to a port the system picks.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::http::{Https, Options, Posted, Transport, Unreached};
use renderedstep_engine::model::route::{Endpoint, CHAT_COMPLETIONS};
use renderedstep_engine::model::system_one::SystemOne;
use renderedstep_engine::model::wire::{self, Call, Message};
use renderedstep_engine::model::{
    receipts, Agent, Book, Failure, Filed, Live, Models, Replay, Reply, Route, Secret,
};
use renderedstep_engine::parity;
use renderedstep_engine::records::Records;
use renderedstep_engine::schemas;
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

fn fixture(name: &str) -> Value {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn captured(flow: &str) -> String {
    fixture("rubyllm_wire.json")["bodies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|body| body["flow"] == flow)
        .unwrap()["body"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn a_prose_call_goes_as_rubyllm_sent_it() {
    let call = Call {
        system: Some("NARRATE.".into()),
        ..Call::prompt("The player types: look")
    };
    let body = wire::chat_body("mistralai/mistral-medium-3.1", &call, true);
    assert_eq!(serde_json::to_string(&body).unwrap(), captured("narration"));
    let body = wire::chat_body(
        "minimax/minimax-m3",
        &Call {
            user: "The player types: wait".into(),
            ..call.clone()
        },
        true,
    );
    assert_eq!(
        serde_json::to_string(&body).unwrap(),
        captured("refusal_rotates")
    );
    let body = wire::chat_body(
        "mistralai/mistral-medium-3.1",
        &Call {
            user: "The player types: wait".into(),
            ..call
        },
        false,
    );
    assert_eq!(
        serde_json::to_string(&body).unwrap(),
        captured("server_error_rotates")
    );
}

#[test]
fn a_structured_call_and_a_continued_one_go_as_rubyllm_sent_them() {
    let call = Call {
        system: Some("REALIZE.".into()),
        user: "And the ways out.".into(),
        schema: Some(schemas::location_exits()),
        history: vec![
            Message {
                role: "user".into(),
                content: json!("Describe it."),
            },
            Message {
                role: "assistant".into(),
                content: json!(
                    "{\"description\":\"A room.\",\"lore\":\"Old.\",\"items\":[],\"people\":[]}"
                ),
            },
        ],
        temperature: None,
    };
    let body = wire::chat_body("mistralai/mistral-medium-3.1", &call, false);
    assert_eq!(serde_json::to_string(&body).unwrap(), captured("two_asks"));

    let call = Call {
        system: Some("BE Maren.".into()),
        user: "Hello.".into(),
        schema: Some(schemas::interaction()),
        history: Vec::new(),
        temperature: None,
    };
    let body = wire::chat_body("mistralai/mistral-medium-3.1", &call, false);
    assert_eq!(serde_json::to_string(&body).unwrap(), captured("durable"));
}

#[test]
fn the_refusal_detector_reads_the_corpus_as_ruby_does() {
    let corpus = fixture("refusal_corpus.json");
    let mut checked = 0;
    for case in corpus["cases"].as_array().unwrap() {
        let text = case["text"].as_str().unwrap();
        let flags: Vec<&str> = renderedstep_engine::model::declined::flags(text)
            .iter()
            .map(|flag| flag.name())
            .collect();
        let expected: Vec<&str> = case["flags"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f.as_str().unwrap())
            .collect();
        assert_eq!(flags, expected, "{text:?}");
        checked += 1;
    }
    println!(
        "refusal detector: {checked} of {checked} texts flagged as the Ruby engine flags them"
    );
}

// --- the policy over fixed replies ------------------------------------------

/// A transport that answers with the next canned reply and keeps what it
/// was sent.
#[derive(Default)]
struct Canned {
    replies: Vec<Posted>,
    streams: Vec<Option<Vec<String>>>,
    sent: Vec<(String, String, Value)>,
}

impl Canned {
    fn json(mut self, status: u16, body: Value) -> Canned {
        self.replies.push(Posted {
            status,
            body: body.to_string(),
        });
        self.streams.push(None);
        self
    }

    fn answer(self, content: &str) -> Canned {
        self.json(
            200,
            json!({"model": "m", "choices": [{"message": {"role": "assistant", "content": content}, "finish_reason": "stop"}],
                   "usage": {"prompt_tokens": 11, "completion_tokens": 7, "cost": 0.00042}}),
        )
    }

    fn stream(mut self, parts: &[&str]) -> Canned {
        let mut lines: Vec<String> = parts
            .iter()
            .map(|part| {
                format!(
                    "data: {}",
                    json!({"model": "m", "choices": [{"delta": {"content": part}}]})
                )
            })
            .collect();
        lines.push(format!(
            "data: {}",
            json!({"choices": [{"delta": {}, "finish_reason": "stop"}]})
        ));
        lines.push(format!("data: {}", json!({"choices": [], "usage": {"prompt_tokens": 20, "completion_tokens": 9, "cost": 0.0007}})));
        lines.push("data: [DONE]".into());
        self.replies.push(Posted {
            status: 200,
            body: String::new(),
        });
        self.streams.push(Some(lines));
        self
    }

    fn models_asked(&self) -> Vec<String> {
        self.sent
            .iter()
            .map(|(_, _, body)| body["model"].as_str().unwrap().to_string())
            .collect()
    }
}

impl Transport for Canned {
    fn post(
        &mut self,
        endpoint: &Endpoint,
        body: &Value,
        _options: &Options,
        on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<Posted, Unreached> {
        let bearer = format!("{:?}", endpoint.bearer);
        self.sent.push((endpoint.url.clone(), bearer, body.clone()));
        if self.replies.is_empty() {
            return Err(Unreached("no reply".into()));
        }
        let reply = self.replies.remove(0);
        let stream = self.streams.remove(0);
        if let (Some(lines), Some(on_line)) = (stream, on_line) {
            for line in lines {
                on_line(&line);
            }
        }
        Ok(reply)
    }
}

struct Game {
    engine: Engine,
    records: Records,
    playthrough: i64,
}

fn game() -> Game {
    let mut engine =
        parity::open_world(include_str!("../parity/worlds/the-quay-house.sql")).unwrap();
    let story = engine
        .story_titled(&format!("The Quay House{}", parity::TITLE_SUFFIX))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    let records = engine.store().load().unwrap();
    Game {
        engine,
        records,
        playthrough,
    }
}

fn rows(game: &Game, sql: &str) -> Vec<Vec<Value>> {
    let conn = game.engine.store().connection();
    let mut statement = conn.prepare(sql).unwrap();
    let width = statement.column_count();
    statement
        .query_map([], |row| {
            Ok((0..width)
                .map(|i| match row.get_ref(i).unwrap() {
                    rusqlite::types::ValueRef::Null => Value::Null,
                    rusqlite::types::ValueRef::Integer(n) => json!(n),
                    rusqlite::types::ValueRef::Real(f) => json!(f),
                    rusqlite::types::ValueRef::Text(t) => json!(String::from_utf8_lossy(t)),
                    rusqlite::types::ValueRef::Blob(_) => Value::Null,
                })
                .collect())
        })
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

fn own_key() -> Route {
    Route::Direct {
        key: Secret::new("sk-or-own-key"),
    }
}

fn narration(playthrough: i64) -> Agent {
    Agent::new(Filed {
        purpose: "narration".into(),
        playthrough: Some(playthrough),
        ..Filed::default()
    })
}

#[test]
fn a_narration_streams_and_leaves_its_receipts() {
    let mut game = game();
    let mut live = Live::with_transport(
        own_key(),
        Canned::default().stream(&["You look ", "around."]),
    );
    let mut agent = narration(game.playthrough);
    let mut chunks = Vec::new();
    let call = Call {
        system: Some("NARRATE.".into()),
        ..Call::prompt("The player types: look")
    };
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let answer = live
        .ask(
            &mut book,
            &mut agent,
            &call,
            None,
            Some(&mut |chunk: &str| chunks.push(chunk.to_string())),
        )
        .unwrap();
    assert_eq!(answer.text(), "You look around.");
    assert_eq!(chunks, ["You look ", "around."]);
    let (url, bearer, body) = &live.transport().sent[0];
    assert_eq!(url, "https://openrouter.ai/api/v1/chat/completions");
    assert_eq!(bearer, "Secret(set)");
    assert_eq!(body["stream"], json!(true));

    assert_eq!(
        rows(&game, "SELECT purpose, playthrough_id FROM chats"),
        vec![vec![json!("narration"), json!(game.playthrough)]]
    );
    assert_eq!(
        rows(
            &game,
            "SELECT role, content, finish_reason FROM messages ORDER BY id"
        ),
        vec![
            vec![json!("system"), json!("NARRATE."), Value::Null],
            vec![json!("user"), json!("The player types: look"), Value::Null],
            vec![json!("assistant"), json!("You look around."), json!("stop")],
        ]
    );
    assert_eq!(
        rows(&game, "SELECT status, model, input_tokens, output_tokens, total_cost, message_id IS NOT NULL FROM ruby_llm_usages"),
        vec![vec![json!("succeeded"), json!("mistralai/mistral-medium-3.1"), json!(20), json!(9), json!(0.0007), json!(1)]]
    );
    assert_eq!(
        agent.recorded().len(),
        2,
        "the exchange this agent had, for the scene to claim"
    );
}

#[test]
fn a_refusal_asks_the_next_model_in_the_same_context() {
    let mut game = game();
    let transport = Canned::default()
        .stream(&["I can't write that."])
        .stream(&["You wait."]);
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = narration(game.playthrough);
    let call = Call {
        system: Some("NARRATE.".into()),
        ..Call::prompt("The player types: wait")
    };
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let answer = live
        .ask(&mut book, &mut agent, &call, None, Some(&mut |_: &str| {}))
        .unwrap();
    assert_eq!(answer.text(), "You wait.");
    assert_eq!(
        live.transport().models_asked(),
        ["mistralai/mistral-medium-3.1", "minimax/minimax-m3"]
    );
    assert_eq!(
        live.transport().sent[0].2["messages"],
        live.transport().sent[1].2["messages"]
    );
    assert_eq!(
        rows(&game, "SELECT role, content FROM messages ORDER BY id"),
        vec![
            vec![json!("system"), json!("NARRATE.")],
            vec![json!("user"), json!("The player types: wait")],
            vec![json!("assistant"), json!("You wait.")],
        ],
        "the refused attempt is taken back out"
    );
    assert_eq!(
        rows(
            &game,
            "SELECT COUNT(*) FROM ruby_llm_usages WHERE message_id IS NULL"
        ),
        vec![vec![json!(1)]]
    );
}

#[test]
fn a_provider_failure_rotates_and_a_refused_key_does_not() {
    let mut game = game();
    let transport = Canned::default()
        .json(500, json!({"error": {"message": "upstream"}}))
        .answer("You wait.");
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = narration(game.playthrough);
    let call = Call::prompt("The player types: wait");
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    assert_eq!(
        live.ask(&mut book, &mut agent, &call, None, None)
            .unwrap()
            .text(),
        "You wait."
    );
    assert_eq!(live.transport().sent.len(), 2);

    let transport = Canned::default().json(
        401,
        json!({"error": {"message": "No auth credentials found"}}),
    );
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = narration(game.playthrough);
    let failure = live
        .ask(&mut book, &mut agent, &call, None, None)
        .unwrap_err();
    assert!(
        matches!(&failure, Failure::Unauthorized(m) if m.contains("No auth credentials found")),
        "{failure:?}"
    );
    assert!(!failure.to_string().contains("sk-or-own-key"));
    assert_eq!(
        live.transport().sent.len(),
        1,
        "a refused key is not fixed by another model"
    );
}

#[test]
fn a_crisis_answer_is_suppressed_and_never_rotated_past() {
    let mut game = game();
    let transport = Canned::default()
        .answer("\"Call 988,\" she says.")
        .answer("You wait.");
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = narration(game.playthrough);
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let failure = live
        .ask(
            &mut book,
            &mut agent,
            &Call::prompt("The player types: wait"),
            None,
            None,
        )
        .unwrap_err();
    assert!(failure.crisis() && failure.unusable());
    assert_eq!(live.transport().sent.len(), 1);
    assert_eq!(
        rows(
            &game,
            "SELECT COUNT(*) FROM messages WHERE role = 'assistant'"
        ),
        vec![vec![json!(0)]],
        "nothing anywhere keeps the suppressed text"
    );
}

#[test]
fn a_schema_ignored_or_a_rejected_answer_asks_the_next_model() {
    let mut game = game();
    let transport = Canned::default()
        .answer("Just prose.")
        .answer(r#"{"exits":[]}"#);
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = narration(game.playthrough);
    let call = Call {
        schema: Some(schemas::location_exits()),
        ..Call::prompt("The ways out.")
    };
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let failure = live
        .ask(&mut book, &mut agent, &call, None, None)
        .unwrap_err();
    assert_eq!(
        failure,
        Failure::SchemaIgnored("minimax/minimax-m3 omitted schema fields: exits".into())
    );
    assert_eq!(
        live.transport().sent.len(),
        2,
        "two models, so two attempts"
    );

    let transport = Canned::default()
        .answer(r#"{"exits":[{"name":"Quay"}]}"#)
        .answer(r#"{"exits":[{"name":"Quay Steps"}]}"#);
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = narration(game.playthrough);
    let mut verify = |content: &Value| {
        if content["exits"][0]["name"] == "Quay" {
            Err("cut off".to_string())
        } else {
            Ok(())
        }
    };
    let answer = live
        .ask(&mut book, &mut agent, &call, Some(&mut verify), None)
        .unwrap();
    assert_eq!(answer.content["exits"][0]["name"], "Quay Steps");
}

#[test]
fn a_second_ask_continues_the_conversation_it_left() {
    let mut game = game();
    let transport = Canned::default()
        .answer(r#"{"description":"A room.","lore":"Old.","items":[],"people":[]}"#)
        .answer(r#"{"exits":[{"name":"Quay"}]}"#);
    let mut live = Live::with_transport(own_key(), transport);
    let mut agent = Agent::new(Filed {
        purpose: "location".into(),
        playthrough: Some(game.playthrough),
        ..Filed::default()
    });
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let first = Call {
        system: Some("REALIZE.".into()),
        schema: Some(schemas::location_exits()),
        ..Call::prompt("Describe it.")
    };
    live.ask(
        &mut book,
        &mut agent,
        &Call {
            schema: None,
            ..first.clone()
        },
        None,
        None,
    )
    .ok();
    let chat = agent.chat().unwrap();
    let history: Vec<Message> = book
        .records
        .select("messages", |m| {
            m["chat_id"] == json!(chat) && m["role"] != "system"
        })
        .iter()
        .map(|m| Message {
            role: m["role"].as_str().unwrap().into(),
            content: m["content"].clone(),
        })
        .collect();
    let second = Call {
        history,
        ..Call {
            user: "And the ways out.".into(),
            ..first
        }
    };
    live.ask(&mut book, &mut agent, &second, None, None)
        .unwrap();
    let messages = &live.transport().sent[1].2["messages"];
    assert_eq!(messages.as_array().unwrap().len(), 4);
    assert_eq!(messages[2]["role"], "assistant");
    assert_eq!(
        rows(&game, "SELECT COUNT(*) FROM chats"),
        vec![vec![json!(1)]]
    );
}

#[test]
fn no_route_is_no_model() {
    let mut game = game();
    let mut live = Live::with_transport(Route::None, Canned::default());
    let mut agent = narration(game.playthrough);
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    assert_eq!(
        live.ask(&mut book, &mut agent, &Call::prompt("x"), None, None),
        Err(Failure::NoModel)
    );
    assert!(!live.system_one());
    assert!(live.transport().sent.is_empty());
}

#[test]
fn a_picked_up_conversation_drops_its_unanswered_prompt_and_keeps_two_exchanges() {
    let mut game = game();
    let character = game.records.table("characters")[0]["id"].as_i64();
    let playthrough = Some(game.playthrough);
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let filed = Filed {
        purpose: receipts::CHARACTER.into(),
        playthrough,
        character,
        player: None,
    };
    let chat = receipts::conversation(&mut book, None, &filed, "m", Some("BE.")).unwrap();
    for n in 0..3 {
        receipts::message(&mut book, chat, "user", &format!("q{n}"), None).unwrap();
        receipts::message(&mut book, chat, "assistant", &format!("a{n}"), Some("stop")).unwrap();
    }
    receipts::message(&mut book, chat, "user", "killed", None).unwrap();
    receipts::pick_up(&mut book, chat).unwrap();
    let left: Vec<String> = book
        .records
        .select("messages", |m| m["chat_id"] == json!(chat))
        .iter()
        .map(|m| m["content"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(left, ["BE.", "q1", "a1", "q2", "a2"]);
}

// --- System One -------------------------------------------------------------

#[test]
fn system_one_goes_to_the_decisions_route_or_to_typesafe() {
    let mut game = game();
    let answers = json!({"answers": {"verb": {"type": "choice", "choice": "look"}}, "usage": {"cost": 0.003}});
    let mut live = Live::with_transport(own_key(), Canned::default().json(200, answers.clone()));
    assert!(live.system_one());
    let filed = Filed {
        purpose: "classifier".into(),
        playthrough: Some(game.playthrough),
        ..Filed::default()
    };
    let questions = json!({"verb": {"type": "choice", "criteria": {"look": "x"}}});
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let payload = live
        .ask_questions(&mut book, &filed, &json!({"line": "look"}), &questions)
        .unwrap();
    assert_eq!(payload, answers);
    let (url, _, body) = &live.transport().sent[0];
    assert_eq!(url, "https://openrouter.ai/api/alpha/decisions");
    assert_eq!(
        body,
        &json!({"model": "typesafe/jev-1.13", "state": {"line": "look"}, "questions": questions})
    );
    assert_eq!(
        rows(
            &game,
            "SELECT purpose, transport, cost_usd FROM system_one_receipts"
        ),
        vec![vec![
            json!("classifier"),
            json!("openrouter_decisions"),
            json!(0.003)
        ]]
    );

    let typesafe = SystemOne::TypeSafe {
        key: Secret::new("ts-key"),
    };
    let mut live = Live::with_transport(
        Route::None,
        Canned::default().json(403, json!({"error": "about the key"})),
    )
    .with_system_one(typesafe);
    assert!(live.system_one());
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let unavailable = live
        .ask_questions(&mut book, &filed, &json!({}), &questions)
        .unwrap_err();
    assert_eq!(
        unavailable.0, "the provider answered 403",
        "the status, never the body"
    );
    assert_eq!(
        live.transport().sent[0].0,
        "https://api.typesafe.ai/v1/systemone"
    );
    assert_eq!(live.transport().sent[0].2["model"], "jev-1.13.0");
}

// --- the two routes over a real socket --------------------------------------

/// Serves one request on a port the system picks, answering with `reply`,
/// and hands back what it was sent: the request line, the headers and the
/// body.
fn serve_once(
    reply: String,
) -> (
    String,
    std::thread::JoinHandle<(String, Vec<String>, String)>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handle = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        let mut headers = Vec::new();
        let mut length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            let line = line.trim_end().to_string();
            if line.is_empty() {
                break;
            }
            if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = value.trim().parse().unwrap();
            }
            headers.push(line);
        }
        let mut body = vec![0; length];
        reader.read_exact(&mut body).unwrap();
        let mut stream = stream;
        stream.write_all(reply.as_bytes()).unwrap();
        (
            request_line.trim_end().to_string(),
            headers,
            String::from_utf8(body).unwrap(),
        )
    });
    (base, handle)
}

fn http_reply(content_type: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn a_relay_carries_the_players_token_to_the_relay_and_nothing_else() {
    let events = [
        json!({"model": "m", "choices": [{"delta": {"content": "You wait."}}]}).to_string(),
        json!({"choices": [], "usage": {"prompt_tokens": 3, "completion_tokens": 2}}).to_string(),
    ];
    let stream = format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        events[0], events[1]
    );
    let (base, server) = serve_once(http_reply("text/event-stream", &stream));
    let route = Route::Relay {
        base_url: format!("{base}/relay/openrouter"),
        token: Secret::new("player-token"),
    };
    let mut game = game();
    let mut live = Live::with_transport(route, Https::new());
    let mut agent = narration(game.playthrough);
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let mut chunks = String::new();
    let answer = live
        .ask(
            &mut book,
            &mut agent,
            &Call::prompt("The player types: wait"),
            None,
            Some(&mut |c: &str| chunks.push_str(c)),
        )
        .unwrap();
    assert_eq!(answer.text(), "You wait.");
    assert_eq!(chunks, "You wait.");
    let (request_line, headers, body) = server.join().unwrap();
    assert_eq!(
        request_line,
        "POST /relay/openrouter/api/v1/chat/completions HTTP/1.1"
    );
    assert!(headers
        .iter()
        .any(|h| h == "authorization: Bearer player-token"
            || h == "Authorization: Bearer player-token"));
    let body: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(
        body,
        wire::chat_body(
            "mistralai/mistral-medium-3.1",
            &Call::prompt("The player types: wait"),
            true
        )
    );
}

#[test]
fn the_same_body_goes_on_both_routes_and_only_the_base_and_bearer_differ() {
    let own = own_key();
    let relay = Route::Relay {
        base_url: "https://relay.example/relay/openrouter".into(),
        token: Secret::new("t"),
    };
    let direct = own.endpoint(CHAT_COMPLETIONS).unwrap();
    let relayed = relay.endpoint(CHAT_COMPLETIONS).unwrap();
    assert!(direct.url.starts_with("https://openrouter.ai/"));
    assert!(relayed
        .url
        .starts_with("https://relay.example/relay/openrouter/"));
    assert_ne!(direct.bearer, relayed.bearer);
    let mut game = game();
    let mut sent = Vec::new();
    for route in [own, relay] {
        let mut live = Live::with_transport(route, Canned::default().answer("You wait."));
        let mut agent = narration(game.playthrough);
        let mut book = Book {
            store: game.engine.store(),
            records: &mut game.records,
        };
        live.ask(
            &mut book,
            &mut agent,
            &Call::prompt("The player types: wait"),
            None,
            None,
        )
        .unwrap();
        sent.push(live.transport().sent[0].2.clone());
    }
    assert_eq!(sent[0], sent[1]);
}

// --- the replay -------------------------------------------------------------

#[test]
fn the_replay_answers_in_order_and_checks_every_prompt() {
    let mut game = game();
    let mut replay = Replay::new(vec![
        Reply::from_value(&json!({"purpose": "system_one", "content": {"verb": "look"}, "prompt_includes": ["look"]})).unwrap(),
        Reply::from_value(&json!({"purpose": "narration", "content": "You look.", "prompt_includes": ["The player types"], "prompt_excludes": ["secret"]})).unwrap(),
    ]);
    assert!(replay.system_one());
    let questions = json!({"verb": {"type": "choice"}, "present": {"type": "noul"}, "target": {"type": "choice"}});
    let filed = Filed::default();
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let payload = replay
        .ask_questions(&mut book, &filed, &json!({"line": "look"}), &questions)
        .unwrap();
    assert_eq!(
        payload["answers"]["verb"],
        json!({"type": "choice", "choice": "look", "probabilities": {"look": 1.0}, "confidence": 1.0})
    );
    assert_eq!(
        payload["answers"]["present"],
        json!({"type": "noul", "noul": 0.0})
    );
    assert_eq!(payload["answers"]["target"]["choice"], "nothing");
    let mut agent = narration(game.playthrough);
    let answer = replay
        .ask(
            &mut book,
            &mut agent,
            &Call::prompt("The player types: look, secret"),
            None,
            None,
        )
        .unwrap();
    assert_eq!(answer.text(), "You look.");
    assert_eq!(
        replay.finish(),
        Err("narration prompt disclosed \"secret\"".into())
    );
}

#[test]
fn a_call_out_of_order_or_a_reply_left_over_fails_the_step() {
    let mut game = game();
    let mut book = Book {
        store: game.engine.store(),
        records: &mut game.records,
    };
    let mut replay = Replay::new(vec![
        Reply::unavailable("arrival"),
        Reply::unavailable("narration"),
    ]);
    let mut agent = narration(game.playthrough);
    assert_eq!(
        replay.ask(&mut book, &mut agent, &Call::prompt("x"), None, None),
        Err(Failure::Unexpected(
            "browser step unexpectedly called \"narration\"".into()
        ))
    );
    assert_eq!(
        replay.finish(),
        Err("browser step expected [\"arrival\", \"narration\"] rendering calls, got [\"narration\"]".into())
    );
    assert!(!replay.system_one());
}
