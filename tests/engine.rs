//! The engine as a caller holds it: a database of its own, a line in, the
//! outcome out, and every failure a value.

use renderedstep_engine::engine::{Engine, Error};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::store::{shape, SCHEMA_VERSION, SHAPE};
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
fn a_database_at_an_older_schema_is_refused() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world("the-quay-house")).unwrap();
    conn.execute_batch(&format!(
        "DELETE FROM schema_migrations WHERE version = '{SCHEMA_VERSION}'"
    ))
    .unwrap();
    match Engine::from_connection(conn) {
        Err(Error::SchemaMismatch { found, expected }) => {
            assert!(found.unwrap().as_str() < SCHEMA_VERSION);
            assert_eq!(expected, SCHEMA_VERSION);
        }
        other => panic!("expected a schema mismatch, got {:?}", other.err()),
    }
}

/// A world with one more migration run over it: `sql`, and its version
/// recorded after this engine's.
fn migrated(name: &str, sql: &str) -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.execute_batch(&world(name)).unwrap();
    conn.execute_batch(sql).unwrap();
    conn.execute(
        "INSERT INTO schema_migrations (version) VALUES ('29990101000000')",
        [],
    )
    .unwrap();
    conn
}

fn changed(conn: rusqlite::Connection) -> Vec<String> {
    match Engine::from_connection(conn) {
        Err(Error::SchemaChanged { found, differences }) => {
            assert_eq!(found, "29990101000000");
            differences
        }
        other => panic!("expected a changed schema, got {:?}", other.err()),
    }
}

#[test]
fn every_world_has_the_shape_the_engine_is_written_against() {
    let worlds = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds");
    for entry in std::fs::read_dir(worlds).unwrap() {
        let path = entry.unwrap().path();
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(&std::fs::read_to_string(&path).unwrap())
            .unwrap();
        let found = shape(&conn).unwrap();
        assert!(
            found.iter().map(String::as_str).eq(SHAPE.lines()),
            "{} does not have store::SHAPE; its shape is:\n{}\n",
            path.display(),
            found.join("\n")
        );
    }
}

#[test]
fn a_newer_schema_that_only_adds_a_table_the_engine_never_touches_is_opened() {
    let conn = migrated(
        "the-quay-house",
        r#"CREATE TABLE "lanterns" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "player_id" integer NOT NULL, "lit" boolean DEFAULT FALSE NOT NULL, "created_at" datetime(6) NOT NULL, "updated_at" datetime(6) NOT NULL, CONSTRAINT "fk_rails_lanterns" FOREIGN KEY ("player_id") REFERENCES "players" ("id"));
           CREATE INDEX "index_lanterns_on_player_id" ON "lanterns" ("player_id");"#,
    );
    let mut engine = Engine::from_connection(conn).expect("the newer schema is opened");
    let story = engine
        .story_titled(&format!("The Quay House{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    let outcome = engine
        .play(playthrough, "look around", &mut |_| {})
        .unwrap();
    assert!(outcome.state.location.is_some());
}

#[test]
fn a_newer_schema_that_changes_a_column_the_engine_uses_is_refused() {
    let differences = changed(migrated(
        "the-quay-house",
        r#"CREATE TABLE "characters_scenes_new" ("character_id" integer NOT NULL, "scene_id" varchar NOT NULL);
           INSERT INTO "characters_scenes_new" SELECT * FROM "characters_scenes";
           DROP TABLE "characters_scenes";
           ALTER TABLE "characters_scenes_new" RENAME TO "characters_scenes";
           CREATE INDEX "index_characters_scenes_on_character_id_and_scene_id" ON "characters_scenes" ("character_id", "scene_id");
           CREATE INDEX "index_characters_scenes_on_scene_id_and_character_id" ON "characters_scenes" ("scene_id", "character_id");"#,
    ));
    assert_eq!(
        differences,
        [
            "expected column characters_scenes.scene_id integer not null pk=0",
            "found column characters_scenes.scene_id varchar not null pk=0",
        ]
    );
}

#[test]
fn a_newer_schema_that_adds_a_column_to_a_table_the_engine_uses_is_refused() {
    let differences = changed(migrated(
        "the-quay-house",
        r#"ALTER TABLE "items" ADD "weight" decimal NOT NULL DEFAULT 0;"#,
    ));
    assert_eq!(
        differences,
        ["found column items.weight decimal not null default 0 pk=0"]
    );
}

#[test]
fn a_newer_schema_that_adds_a_unique_index_to_a_table_the_engine_uses_is_refused() {
    let differences = changed(migrated(
        "the-quay-house",
        r#"CREATE UNIQUE INDEX "index_items_on_name" ON "items" ("name");"#,
    ));
    assert_eq!(differences, ["found index items (name) unique"]);
}

#[test]
fn a_newer_table_that_points_into_one_the_engine_deletes_from_is_refused() {
    let differences = changed(migrated(
        "the-quay-house",
        r#"CREATE TABLE "message_flags" ("id" integer PRIMARY KEY AUTOINCREMENT NOT NULL, "message_id" integer NOT NULL, CONSTRAINT "fk_rails_message_flags" FOREIGN KEY ("message_id") REFERENCES "messages" ("id"));"#,
    ));
    assert_eq!(
        differences,
        ["found foreign key message_flags.message_id -> messages.id on update NO ACTION on delete NO ACTION"]
    );
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

#[test]
fn a_turn_stopped_after_its_take_is_finished_by_the_next_delivery_and_takes_once() {
    use renderedstep_engine::model::{Replay, Reply};
    let (mut engine, playthrough) = gate();
    let mut nothing = Replay::new(Vec::new());
    let stopped = engine.submit_stopping(
        playthrough,
        "/take red coin",
        "pickup",
        &mut nothing,
        &mut |_| {},
        Some("take"),
    );
    assert_eq!(stopped.unwrap_err(), Error::Stopped("take".into()));
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE status = 'running'"
        ),
        1,
        "a stopped worker leaves its submission running"
    );
    let taken = "SELECT COUNT(*) FROM items WHERE name = 'red coin' AND playthrough_id IS NOT NULL AND location_id IS NULL AND character_id IS NULL";
    assert_eq!(
        count(&engine, taken),
        1,
        "the take committed with its receipt"
    );

    let reply = Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You pocket the red coin.",
    }))
    .unwrap();
    let mut replay = Replay::new(vec![reply]);
    let finished = engine
        .submit(
            playthrough,
            "/take red coin",
            "pickup",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
    assert!(finished.turned.scene.is_some());
    assert_eq!(count(&engine, taken), 1);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE status = 'completed'"
        ),
        1
    );
}
