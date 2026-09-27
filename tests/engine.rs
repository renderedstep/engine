//! The engine as a caller holds it: a database of its own, a line in, the
//! outcome out, and every failure a value.

use renderedstep_engine::engine::{Engine, Error};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::store::SCHEMA_VERSION;
use std::path::{Path, PathBuf};

fn world(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("parity/worlds")
        .join(format!("{name}.sql"));
    std::fs::read_to_string(path).expect("a world fixture")
}

/// A database file of its own under the system's temporary directory,
/// removed when dropped.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str, sql: &str) -> Scratch {
        let path = std::env::temp_dir().join(format!(
            "renderedstep-engine-{name}-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        rusqlite::Connection::open(&path)
            .and_then(|conn| conn.execute_batch(sql))
            .expect("the fixture loads");
        Scratch(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn a_turn_is_written_to_the_database_and_read_back_by_the_next_connection() {
    let scratch = Scratch::new("turn", &world("the-quay-house"));
    let (playthrough, room) = {
        let mut engine = Engine::open(&scratch.0).expect("the database opens");
        let story = engine
            .story_titled(&format!("The Quay House{TITLE_SUFFIX}"))
            .unwrap();
        let playthrough = engine.start(story).unwrap();
        let mut chunks = Vec::new();
        let outcome = engine
            .play(playthrough, "go to The Custom House room 1", &mut |chunk| {
                chunks.push(chunk.to_string())
            })
            .unwrap();
        assert!(chunks.is_empty(), "nothing narrates with no model");
        assert_eq!(
            outcome.report.change.as_deref(),
            Some(
                "moved: The Quay -> The Custom House room 1 (a stub -- nobody has written this \
                 room, and no-model mode cannot)"
            )
        );
        (playthrough, outcome.state.location.unwrap())
    };
    let engine = Engine::open(&scratch.0).expect("the database opens again");
    let outcome = engine.read(playthrough, Vec::new()).unwrap();
    assert_eq!(outcome.state.location, Some(room));
}

#[test]
fn a_database_at_another_schema_is_refused() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world("the-quay-house")).unwrap();
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES ('29990101000000')",
        [],
    )
    .unwrap();
    match Engine::from_connection(conn) {
        Err(Error::SchemaMismatch { found, expected }) => {
            assert_eq!(found.as_deref(), Some("29990101000000"));
            assert_eq!(expected, SCHEMA_VERSION);
        }
        other => panic!("expected a schema mismatch, got {:?}", other.err()),
    }
}

#[test]
fn a_database_with_no_migrations_is_refused() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    assert!(matches!(
        Engine::from_connection(conn),
        Err(Error::SchemaMismatch { found: None, .. })
    ));
}

#[test]
fn a_playthrough_that_is_not_there_is_an_error() {
    let mut engine = open_world(&world("the-quay-house")).unwrap();
    assert_eq!(
        engine.play(42, "look", &mut |_| {}).unwrap_err(),
        Error::NoSuchPlaythrough(42)
    );
}

fn gate() -> (Engine, i64) {
    let mut engine = open_world(&world("a-turn-at-the-gate")).unwrap();
    let story = engine
        .story_titled(&format!("A Turn at the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    (engine, playthrough)
}

fn count(engine: &Engine, sql: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

#[test]
fn a_submitted_line_is_told_and_a_second_delivery_plays_nothing() {
    use renderedstep_engine::model::{Replay, Reply};
    let (mut engine, playthrough) = gate();
    let reply = Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You pocket the red coin.",
        "prompt_includes": ["The player types: take red coin", "picked the red coin up"],
    }))
    .unwrap();
    let mut replay = Replay::new(vec![reply]);
    let submitted = engine
        .submit(
            playthrough,
            "/take red coin",
            "one",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
    let scene = submitted.turned.scene.expect("the turn's scene");
    assert!(submitted
        .state
        .carrying
        .iter()
        .any(|thing| thing.name == "red coin"));
    assert_eq!(
        engine
            .store()
            .connection()
            .query_row(
                "SELECT description || '|' || resolved_action || '|' || resolved_by FROM scenes WHERE id = ?1",
                [scene],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "You pocket the red coin.|take|grammar"
    );
    let scenes = count(&engine, "SELECT COUNT(*) FROM scenes");

    let mut nothing = Replay::new(Vec::new());
    let again = engine
        .submit(
            playthrough,
            "/take red coin",
            "one",
            &mut nothing,
            &mut |_| {},
        )
        .unwrap();
    nothing.finish().unwrap();
    assert_eq!(again.turned.scene, Some(scene));
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM scenes"), scenes);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE status = 'completed'"
        ),
        1
    );
}

#[test]
fn with_no_model_access_an_effect_is_told_in_the_engines_own_words() {
    use renderedstep_engine::model::{Live, Route};
    let (mut engine, playthrough) = gate();
    let mut live = Live::new(Route::None);
    let mut chunks = String::new();
    let submitted = engine
        .submit(
            playthrough,
            "/take red coin",
            "t",
            &mut live,
            &mut |chunk| chunks.push_str(chunk),
        )
        .unwrap();
    assert!(submitted.turned.setup);
    assert!(chunks.is_empty());
    assert_eq!(
        engine
            .store()
            .connection()
            .query_row(
                "SELECT description FROM scenes WHERE id = ?1 AND engine_fallback = 1",
                [submitted.turned.scene.unwrap()],
                |row| row.get::<_, String>(0)
            )
            .unwrap(),
        "You pick up the red coin."
    );
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE error_kind = 'setup'"
        ),
        1
    );
    let unread = engine
        .submit(playthrough, "look around", "u", &mut live, &mut |_| {})
        .unwrap_err();
    assert_eq!(
        unread,
        Error::Model(renderedstep_engine::model::Failure::NoModel),
        "a line with no slash needs the classifier, and there is no model to ask"
    );
}

/// Answers each post with the next body, and keeps what was sent.
struct Scripted {
    answers: Vec<(u16, serde_json::Value)>,
    sent: Vec<serde_json::Value>,
}

impl renderedstep_engine::model::http::Transport for Scripted {
    fn post(
        &mut self,
        _endpoint: &renderedstep_engine::model::route::Endpoint,
        body: &serde_json::Value,
        _options: &renderedstep_engine::model::http::Options,
        _on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<renderedstep_engine::model::http::Posted, renderedstep_engine::model::http::Unreached>
    {
        self.sent.push(body.clone());
        let (status, answer) = self.answers.remove(0);
        Ok(renderedstep_engine::model::http::Posted {
            status,
            body: answer.to_string(),
        })
    }
}

fn answered(content: serde_json::Value) -> (u16, serde_json::Value) {
    let content = match content {
        serde_json::Value::String(text) => text,
        other => other.to_string(),
    };
    (
        200,
        serde_json::json!({"choices": [{"message": {"content": content}, "finish_reason": "stop"}]}),
    )
}

#[test]
fn a_room_whose_ways_out_failed_is_picked_up_where_it_stopped_on_the_live_path() {
    use renderedstep_engine::model::{Live, Route, Secret};
    use serde_json::json;
    let mut engine = open_world(&world("the-unfinished-workshop")).unwrap();
    let story = engine
        .story_titled(&format!("The Unfinished Workshop{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    let detail = json!({
        "description": "A bench, a vice and a brass token.",
        "lore": "The town's oldest workshop.",
        "items": [{"name": "brass token", "description": "A small brass disc.", "use_kind": "ordinary", "combustible": false, "readable": false}],
        "people": [{
            "fullname": "Sella Reed", "nickname": "Sella", "appearance": "A patched apron.",
            "personality": "Patient and careful.", "backstory": "A lifelong maker of keys.",
            "likes": "Honest work.", "dislikes": "Waste.", "fears": "Fire.",
        }],
    });
    let failed = (500, json!({"error": {"message": "upstream"}}));
    let transport = Scripted {
        answers: vec![answered(detail.clone()), failed.clone(), failed],
        sent: Vec::new(),
    };
    let route = Route::Direct {
        key: Secret::new("own"),
    };
    let mut live = Live::with_transport(route.clone(), transport);
    let first = engine.submit(playthrough, "/move Workshop", "a", &mut live, &mut |_| {});
    assert!(matches!(first, Err(Error::Model(_))), "{first:?}");
    let pending: Option<String> = engine
        .store()
        .connection()
        .query_row(
            "SELECT generation_checkpoint FROM locations WHERE name = 'Workshop'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(pending.unwrap().contains("exits_pending"));
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM items WHERE name = 'brass token' AND playthrough_id IS NULL"
        ),
        1
    );

    let exits = json!({"exits": [{"name": "Market", "teaser": "Back out.", "distance": "adjacent", "travel_method": "walking", "population": "nobody"}]});
    let arrival = json!({"description": "You step in.", "summary": "Into the workshop."});
    let transport = Scripted {
        answers: vec![answered(exits), answered(arrival)],
        sent: Vec::new(),
    };
    let mut live = Live::with_transport(route, transport);
    let submitted = engine
        .submit(playthrough, "/move Workshop", "b", &mut live, &mut |_| {})
        .unwrap();
    assert_eq!(submitted.state.location.as_ref().unwrap().name, "Workshop");
    let asked = &live.transport().sent[0]["messages"];
    let roles: Vec<&str> = asked
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["role"].as_str().unwrap())
        .collect();
    assert_eq!(
        roles,
        ["developer", "user", "assistant", "user"],
        "the ways out are asked after the kept detail"
    );
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM characters WHERE fullname = 'Sella Reed'"
        ),
        1
    );
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM items WHERE name = 'brass token' AND playthrough_id IS NULL"
        ),
        1,
        "the room's things are admitted once"
    );
    assert_eq!(
        count(&engine, "SELECT COUNT(*) FROM locations WHERE name = 'Workshop' AND detail_level = 'realized' AND generation_checkpoint IS NULL"),
        1
    );
}

#[test]
fn a_conversation_is_picked_up_again_on_the_next_talk_on_the_live_path() {
    use renderedstep_engine::model::{Live, Route, Secret};
    use serde_json::json;
    let (mut engine, playthrough) = gate();
    let reaction = json!({
        "pre_thought": "I will listen.", "pre_feeling": "attentive", "action": "Maren nods.",
        "post_thought": "That was fair.", "post_feeling": "calm",
        "inner_resolution": "I will sweep the market.", "engine_action": "none",
    });
    let transport = Scripted {
        answers: vec![
            answered(reaction.clone()),
            answered(json!("Maren nods to you.")),
            answered(reaction),
            answered(json!("Maren nods again.")),
        ],
        sent: Vec::new(),
    };
    let route = Route::Direct {
        key: Secret::new("own"),
    };
    let mut live = Live::with_transport(route, transport);
    for token in ["one", "two"] {
        engine
            .submit(playthrough, "/talk Maren", token, &mut live, &mut |_| {})
            .unwrap();
    }
    let sent = &live.transport().sent;
    let roles = |body: &serde_json::Value| -> Vec<String> {
        body["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| m["role"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(roles(&sent[0]), ["developer", "user"]);
    assert_eq!(
        roles(&sent[2]),
        ["developer", "user", "assistant", "user"],
        "the second talk replays the first exchange"
    );
    assert_eq!(
        roles(&sent[1]),
        ["user"],
        "the narrator pass has no history"
    );
    assert_eq!(
        count(&engine, "SELECT COUNT(*) FROM interactions"),
        2,
        "each exchange keeps the person's side of it"
    );
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM chats WHERE purpose = 'character'"
        ),
        1
    );
}
