//! What a played line leaves in the rows: the scene it wrote and the one
//! before it, the reach it counted when a reading found nothing or named two
//! things, the blows it struck, the notice it kept, and the closing scene an
//! ending writes over the turn that reached it.

use renderedstep_engine::engine::{Engine, Error, Submitted};
use renderedstep_engine::model::{Failure, Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::{json, Value};
use std::path::Path;

fn world(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("parity/worlds")
        .join(format!("{name}.sql"));
    std::fs::read_to_string(path).expect("a world fixture")
}

/// The world `name` with `sql` run over it, and a game of the story titled
/// `title` started in it.
fn started(name: &str, title: &str, sql: &str) -> (Engine, i64) {
    let mut engine = open_world(&format!("{}{sql}", world(name))).unwrap();
    let story = engine
        .story_titled(&format!("{title}{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    (engine, playthrough)
}

/// A Turn at the Gate, with `sql` run over it: Cal in the market, Maren at
/// the gate, a red coin on the flagstones and the courtyard the one way out.
fn gate(sql: &str) -> (Engine, i64) {
    started("a-turn-at-the-gate", "A Turn at the Gate", sql)
}

/// What a classifier answers: an intent, its target and a second name.
fn reading(intent: &str, target: &str, also: &str) -> Value {
    json!({
        "purpose": "classifier",
        "content": { "intent": intent, "target": target, "also_named": also },
    })
}

/// `line` submitted under `token` with `replies` answered in order; the
/// replay is held to having been asked exactly those.
fn submit(engine: &mut Engine, game: i64, line: &str, token: &str, replies: &[Value]) -> Submitted {
    let mut replay = Replay::new(
        replies
            .iter()
            .map(|reply| Reply::from_value(reply).unwrap())
            .collect(),
    );
    let submitted = engine
        .submit(game, line, token, &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    submitted
}

fn count(engine: &Engine, sql: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(sql, [], |row| row.get(0))
        .unwrap()
}

/// One column of one row.
fn column<T: rusqlite::types::FromSql>(engine: &Engine, sql: &str, id: i64) -> T {
    engine
        .store()
        .connection()
        .query_row(sql, [id], |row| row.get(0))
        .unwrap()
}

/// One column of every row.
fn all<T: rusqlite::types::FromSql>(engine: &Engine, sql: &str, id: i64) -> Vec<T> {
    let conn = engine.store().connection();
    let mut statement = conn.prepare(sql).unwrap();
    statement
        .query_map([id], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn current_scene(engine: &Engine, game: i64) -> Option<i64> {
    column(
        engine,
        "SELECT current_scene_id FROM playthroughs WHERE id = ?1",
        game,
    )
}

/// The newest row of `table`: action, command, the names on it, the room
/// and the scene.
fn newest(engine: &Engine, table: &str, names: &str) -> (String, String, String, i64, i64) {
    engine
        .store()
        .connection()
        .query_row(
            &format!(
                "SELECT action, command, {names}, location_id, scene_id FROM {table} \
                 ORDER BY id DESC LIMIT 1"
            ),
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap()
}

/// The market's id.
const MARKET: i64 = 1_000_000_001;

/// A second person in the market, who answers to a nickname, and a tin cup
/// lying beside the coin.
const ROWE_AND_A_CUP: &str = "
    CREATE TEMP TABLE rowe AS SELECT * FROM characters WHERE fullname = 'Maren';
    UPDATE rowe SET id = 1000000003, fullname = 'Halkett Rowe', nickname = 'Rowe';
    INSERT INTO characters SELECT * FROM rowe;
    CREATE TEMP TABLE cup AS SELECT * FROM items WHERE name = 'red coin';
    UPDATE cup SET id = 1000000003, name = 'tin cup';
    INSERT INTO items SELECT * FROM cup;";

#[test]
fn a_reach_that_found_nothing_is_counted_with_what_was_on_offer_and_the_paragraph_just_read() {
    let (mut engine, game) = gate("");
    let opening = current_scene(&engine, game).expect("the opening scene");

    let moved = submit(
        &mut engine,
        game,
        "go through the cellar door",
        "move",
        &[reading("move", "nothing", "nothing")],
    );
    assert!(moved.turned.refusal.is_some());
    assert_eq!(
        newest(&engine, "playthrough_drifts", "offered"),
        (
            "move".into(),
            "go through the cellar door".into(),
            "Courtyard".into(),
            MARKET,
            opening
        )
    );
    assert_eq!(
        count(
            &engine,
            &format!("SELECT COUNT(*) FROM playthrough_drifts WHERE playthrough_id = {game}")
        ),
        1
    );

    let struck = submit(
        &mut engine,
        game,
        "hit the hound",
        "attack",
        &[reading("attack", "nothing", "nothing")],
    );
    assert!(struck.turned.refusal.is_some());
    let (action, _, offered, _, _) = newest(&engine, "playthrough_drifts", "offered");
    assert_eq!(action, "attack");
    assert!(offered.split(", ").any(|name| name == "Maren"), "{offered}");
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_blows"), 0);

    submit(
        &mut engine,
        game,
        "take the cellar key and the coin",
        "take",
        &[reading("take", "nothing", "red coin")],
    );
    let (action, _, offered, _, _) = newest(&engine, "playthrough_drifts", "offered");
    assert_eq!((action.as_str(), offered.as_str()), ("take", "red coin"));
    assert_eq!(
        count(&engine, "SELECT COUNT(*) FROM playthrough_overreaches"),
        0,
        "a reach that found nothing is not a second thing named"
    );

    let taken = submit(
        &mut engine,
        game,
        "/take red coin",
        "coin",
        &[json!({"purpose": "narration", "content": "You pocket the red coin."})],
    );
    let read = taken.turned.scene.expect("the take's scene");
    submit(
        &mut engine,
        game,
        "put down the lantern",
        "drop",
        &[reading("drop", "nothing", "nothing")],
    );
    assert_eq!(
        newest(&engine, "playthrough_drifts", "offered"),
        (
            "drop".into(),
            "put down the lantern".into(),
            "red coin".into(),
            MARKET,
            read
        ),
        "what the player holds is on offer, and the paragraph they had just read is the scene"
    );
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM playthrough_drifts"), 4);
    assert_eq!(current_scene(&engine, game), Some(read));
}

#[test]
fn a_line_that_named_two_things_is_counted_with_both_halves_by_the_names_a_player_would_type() {
    let (mut engine, game) = gate(ROWE_AND_A_CUP);
    let opening = current_scene(&engine, game).expect("the opening scene");

    let refused = submit(
        &mut engine,
        game,
        "pick up the coin and the cup",
        "things",
        &[reading("take", "red coin", "tin cup")],
    );
    assert!(refused.turned.refusal.is_some());
    assert_eq!(
        newest(
            &engine,
            "playthrough_overreaches",
            "acted || '|' || unacted"
        ),
        (
            "take".into(),
            "pick up the coin and the cup".into(),
            "red coin|tin cup".into(),
            MARKET,
            opening
        )
    );

    submit(
        &mut engine,
        game,
        "ask Rowe and Maren where the key went",
        "people",
        &[reading("talk", "Rowe", "Maren")],
    );
    let (action, _, halves, _, _) = newest(
        &engine,
        "playthrough_overreaches",
        "acted || '|' || unacted",
    );
    assert_eq!(action, "talk");
    assert_eq!(halves, "Halkett Rowe|Maren", "a person by their full name");
    assert_eq!(
        count(&engine, "SELECT COUNT(*) FROM playthrough_drifts"),
        0,
        "two things named is not a reach that found nothing"
    );
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM interactions"), 0);
}

#[test]
fn an_attack_the_classifier_placed_strikes_both_ways_and_holds_no_conversation() {
    let (mut engine, game) = gate("");
    let submitted = submit(
        &mut engine,
        game,
        "go for her",
        "attack",
        &[reading("attack", "Maren", "nothing")],
    );
    assert_eq!(
        submitted.turned.scene, None,
        "a blow writes no scene of its own"
    );
    assert!(submitted.turned.refusal.is_none());
    let blows = |attacker: &str| {
        count(
            &engine,
            &format!(
                "SELECT COUNT(*) FROM playthrough_blows WHERE attacker_id = \
                 (SELECT id FROM characters WHERE fullname = '{attacker}')"
            ),
        )
    };
    assert_eq!(blows("Cal"), 1);
    assert_eq!(blows("Maren"), 1, "and she answered in the same turn");
    assert_eq!(
        count(&engine, "SELECT COUNT(*) FROM interactions"),
        0,
        "a blow is not a conversation"
    );
}

#[test]
fn a_played_line_keeps_what_was_typed_the_fact_it_was_told_from_and_the_scene_before_it() {
    let (mut engine, game) = gate("");
    let opening = current_scene(&engine, game).expect("the opening scene");
    let mut replay = Replay::new(
        [
            reading("take", "red coin", "nothing"),
            json!({"purpose": "narration", "content": "You stoop for the coin."}),
            reading("drop", "red coin", "nothing"),
            json!({"purpose": "narration", "content": "You set the coin back down."}),
            reading("other", "nothing", "nothing"),
            json!({
                "purpose": "narration",
                "content": "You hum, and the gate creaks along.",
                "prompt_excludes": ["ALREADY happened", "looking more closely"],
            }),
        ]
        .iter()
        .map(|reply| Reply::from_value(reply).unwrap())
        .collect(),
    );
    let mut scenes = Vec::new();
    for (line, token) in [
        ("pick up that red coin", "take"),
        ("put the coin back down", "drop"),
        ("hum a tune", "hum"),
    ] {
        let submitted = engine
            .submit(game, line, token, &mut replay, &mut |_| {})
            .unwrap();
        scenes.push(submitted.turned.scene.expect("the line's scene"));
    }
    replay.finish().unwrap();

    let row = |scene: i64| -> (String, String, Option<String>, Option<i64>) {
        engine
            .store()
            .connection()
            .query_row(
                "SELECT typed, resolved_action, engine_fact, previous_scene_id FROM scenes \
                 WHERE id = ?1",
                [scene],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .unwrap()
    };
    let narrations: Vec<String> = replay
        .sent()
        .iter()
        .filter(|sent| sent["purpose"] == "narration")
        .map(|sent| sent["user"].as_str().unwrap().to_string())
        .collect();

    let (typed, action, fact, previous) = row(scenes[0]);
    assert_eq!(
        (typed.as_str(), action.as_str()),
        ("pick up that red coin", "take")
    );
    assert_eq!(previous, Some(opening));
    let fact = fact.expect("the take's fact");
    assert!(fact.contains("picked the red coin up"), "{fact}");
    assert!(
        narrations[0].contains(&fact),
        "the fact the prose was written against"
    );

    let (typed, action, fact, previous) = row(scenes[1]);
    assert_eq!(
        (typed.as_str(), action.as_str()),
        ("put the coin back down", "drop")
    );
    assert_eq!(previous, Some(scenes[0]));
    assert!(narrations[1].contains(&fact.expect("the drop's fact")));

    let (typed, action, fact, previous) = row(scenes[2]);
    assert_eq!((typed.as_str(), action.as_str()), ("hum a tune", "other"));
    assert_eq!(fact, None, "a line with no effect is told from no fact");
    assert_eq!(previous, Some(scenes[1]));
    assert_eq!(current_scene(&engine, game), Some(scenes[2]));
}

#[test]
fn a_blank_paragraph_with_no_words_of_the_engines_to_fall_back_on_fails_the_line_and_writes_no_scene(
) {
    let (mut engine, game) = gate("");
    let before = current_scene(&engine, game);
    let scenes = count(&engine, "SELECT COUNT(*) FROM scenes");
    let mut replay = Replay::new(
        [
            reading("other", "nothing", "nothing"),
            json!({"purpose": "narration", "content": ""}),
        ]
        .iter()
        .map(|reply| Reply::from_value(reply).unwrap())
        .collect(),
    );
    let failed = engine.submit(game, "hum a tune", "blank", &mut replay, &mut |_| {});
    replay.finish().unwrap();
    assert!(
        matches!(failed, Err(Error::Model(Failure::Rejected(_)))),
        "{failed:?}"
    );
    assert_eq!(count(&engine, "SELECT COUNT(*) FROM scenes"), scenes);
    assert_eq!(current_scene(&engine, game), before);
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE status = 'failed'"
        ),
        1
    );
}

#[test]
fn taking_a_readable_thing_with_no_words_on_record_asks_for_none_and_quotes_none() {
    let (mut engine, game) = started("a-bell-nobody-has-rung", "A Bell Nobody Has Rung", "");
    let submitted = submit(
        &mut engine,
        game,
        "/take sealed letter",
        "take",
        &[json!({
            "purpose": "narration",
            "content": "You tuck the letter into your coat.",
            "prompt_excludes": ["word for word"],
        })],
    );
    assert!(submitted
        .state
        .carrying
        .iter()
        .any(|thing| thing.name == "sealed letter"));
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM items WHERE name = 'sealed letter' AND inscription IS NOT NULL"
        ),
        0,
        "no words were written on any copy of it"
    );
}

#[test]
fn a_take_told_in_the_engines_own_words_leaves_a_toll_its_words_never_told() {
    let (mut engine, game) = gate("");
    engine
        .store()
        .connection()
        .execute(
            "INSERT INTO playthrough_tolls (character_id, created_at, damage, hazard, hp_after, \
             location_id, playthrough_id, sequence, story_timestamp, updated_at) VALUES \
             ((SELECT id FROM characters WHERE fullname = 'Cal'), '2026-10-02 13:26:08', 2, \
             'thorns', 10, ?2, ?1, 1, '2026-09-08 12:00:00', '2026-10-02 13:26:08')",
            [game, MARKET],
        )
        .unwrap();
    let toll = "SELECT scene_id FROM playthrough_tolls WHERE hazard = 'thorns' AND id = ?1";
    let toll_id = count(&engine, "SELECT MAX(id) FROM playthrough_tolls");

    let fallen_back = submit(
        &mut engine,
        game,
        "/take red coin",
        "take",
        &[json!({"purpose": "narration", "unavailable": true})],
    );
    let scene = fallen_back.turned.scene.expect("the take's scene");
    assert!(column::<bool>(
        &engine,
        "SELECT engine_fallback FROM scenes WHERE id = ?1",
        scene
    ));
    assert_eq!(column::<Option<i64>>(&engine, toll, toll_id), None);

    let told = submit(
        &mut engine,
        game,
        "/drop red coin",
        "drop",
        &[json!({"purpose": "narration", "content": "You let the coin fall."})],
    );
    assert_eq!(
        column::<Option<i64>>(&engine, toll, toll_id),
        told.turned.scene,
        "a paragraph the narrator wrote tells it"
    );
}

#[test]
fn a_crisis_answer_to_a_take_is_told_in_the_engines_words_and_the_notice_is_kept_for_a_redelivery()
{
    let (mut engine, game) = gate("");
    let crisis = json!({
        "purpose": "narration",
        "failure": {"kind": "crisis", "message": "If you are struggling, please reach out."},
    });
    let taken = submit(&mut engine, game, "/take red coin", "pickup", &[crisis]);
    assert!(taken.turned.safety_notice);
    assert!(taken
        .state
        .carrying
        .iter()
        .any(|thing| thing.name == "red coin"));
    let scene = taken.turned.scene.expect("the take's scene");
    assert_eq!(
        engine
            .store()
            .connection()
            .query_row(
                "SELECT description, engine_fallback FROM scenes WHERE id = ?1",
                [scene],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?))
            )
            .unwrap(),
        ("You pick up the red coin.".to_string(), true)
    );
    assert_eq!(current_scene(&engine, game), Some(scene));
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE status = 'completed' AND \
             error_kind = 'crisis'"
        ),
        1
    );

    let again = submit(&mut engine, game, "/take red coin", "pickup", &[]);
    assert_eq!(again.turned.scene, Some(scene));
    assert!(
        again.turned.safety_notice,
        "the redelivery reports it again"
    );
}

#[test]
fn a_crisis_answer_after_the_person_decided_keeps_their_side_and_the_notice() {
    let (mut engine, game) = gate("");
    let talked = submit(
        &mut engine,
        game,
        "/talk Maren",
        "talk",
        &[
            json!({
                "purpose": "character",
                "content": {
                    "pre_thought": "What does he want now.",
                    "pre_feeling": "wary",
                    "action": "Maren folds her arms.",
                    "post_thought": "Let him speak.",
                    "post_feeling": "guarded",
                    "inner_resolution": "I will hear him out.",
                    "engine_action": "none",
                },
            }),
            json!({
                "purpose": "interaction-narration",
                "failure": {"kind": "crisis", "message": "If you are struggling, please reach out."},
            }),
        ],
    );
    assert!(talked.turned.safety_notice);
    let scene = talked.turned.scene.expect("the talk's scene");
    let (description, fallback): (String, bool) = engine
        .store()
        .connection()
        .query_row(
            "SELECT description, engine_fallback FROM scenes WHERE id = ?1",
            [scene],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(fallback);
    assert!(
        description.starts_with("You speak with Maren."),
        "{description}"
    );
    assert_eq!(current_scene(&engine, game), Some(scene));
    assert_eq!(
        column::<i64>(
            &engine,
            "SELECT COUNT(*) FROM interactions WHERE scene_id = ?1 AND action = 'Maren folds her arms.'",
            scene
        ),
        1
    );
}

/// Answers each post with the next content, as a stream when the post asks
/// for one, and keeps what was sent.
struct Scripted {
    answers: Vec<Value>,
    sent: Vec<Value>,
}

impl renderedstep_engine::model::http::Transport for Scripted {
    fn post(
        &mut self,
        _endpoint: &renderedstep_engine::model::route::Endpoint,
        body: &Value,
        _options: &renderedstep_engine::model::http::Options,
        on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<renderedstep_engine::model::http::Posted, renderedstep_engine::model::http::Unreached>
    {
        self.sent.push(body.clone());
        let content = match self.answers.remove(0) {
            Value::String(text) => text,
            other => other.to_string(),
        };
        if let Some(on_line) = on_line {
            let delta =
                json!({"choices": [{"delta": {"content": content}, "finish_reason": "stop"}]});
            on_line(&format!("data: {delta}"));
            on_line("data: [DONE]");
            return Ok(renderedstep_engine::model::http::Posted {
                status: 200,
                body: String::new(),
            });
        }
        Ok(renderedstep_engine::model::http::Posted {
            status: 200,
            body: json!({"choices": [{"message": {"content": content}, "finish_reason": "stop"}]})
                .to_string(),
        })
    }
}

#[test]
fn two_lines_played_by_one_delivery_are_each_read_with_no_earlier_exchange() {
    use renderedstep_engine::model::system_one::SystemOne;
    use renderedstep_engine::model::{Live, Route, Secret};
    let (mut engine, game) = gate("");
    engine.accept(game, "look around", "first").unwrap();
    let not_a_move = json!({
        "intent": "other", "target": "nothing", "also_named": "nothing", "thrown_at": "nothing",
    });
    let transport = Scripted {
        answers: vec![
            not_a_move.clone(),
            json!("You look around the market."),
            not_a_move,
            json!("The market is quiet."),
        ],
        sent: Vec::new(),
    };
    let route = Route::Direct {
        key: Secret::new("own"),
    };
    let mut live = Live::with_transport(route, transport).with_system_one(SystemOne::Off);
    engine
        .submit(game, "wait quietly", "second", &mut live, &mut |_| {})
        .unwrap();

    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM playthrough_commands WHERE status = 'completed'"
        ),
        2
    );
    let read: Vec<&Value> = live
        .transport()
        .sent
        .iter()
        .filter(|body| body.to_string().contains("## The Player Types"))
        .collect();
    assert_eq!(read.len(), 2, "one line, one reading");
    for (body, line) in read.iter().zip(["look around", "wait quietly"]) {
        let messages = body["messages"].as_array().unwrap();
        assert!(
            messages
                .iter()
                .all(|message| message["role"] != "assistant"),
            "a one-shot reading carries no earlier exchange: {messages:?}"
        );
        let asked = messages.last().unwrap()["content"].to_string();
        assert!(asked.contains(line), "{asked}");
    }
    assert!(!read[1].to_string().contains("look around"));
    let conversations: Vec<i64> = all(
        &engine,
        "SELECT id FROM chats WHERE purpose = 'classifier' AND playthrough_id = ?1 ORDER BY id",
        game,
    );
    assert_eq!(
        conversations.len(),
        2,
        "one line, one classifier conversation"
    );
    for chat in conversations {
        let roles: Vec<String> = all(
            &engine,
            "SELECT role FROM messages WHERE chat_id = ?1 AND role <> 'system' ORDER BY id",
            chat,
        );
        assert_eq!(roles, ["user", "assistant"]);
    }
    let typed: Vec<String> = all(
        &engine,
        "SELECT typed FROM scenes WHERE typed IS NOT NULL AND story_id = (SELECT story_id FROM \
         playthroughs WHERE id = ?1) ORDER BY id",
        game,
    );
    assert_eq!(typed, ["look around", "wait quietly"]);
}

/// The Iron Gate Descends: a new game played to the threshold of the dry
/// cell, every line before it in the engine's own words.
fn at_the_cell(engine: &mut Engine) -> i64 {
    let story = engine
        .story_titled(&format!("The Iron Gate Descends{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    for line in [
        "take the signet ring",
        "go to the obsidian maw",
        "go to Blackfang Tunnel",
        "go to Blackfang Warren room 1",
        "go to Blackfang Warren room 2",
    ] {
        engine.play(game, line, &mut |_| {}).unwrap();
    }
    game
}

/// The way into the dry cell, told, and the ending answered with `ending`;
/// the closing scene, and the scene before it.
fn into_the_cell(engine: &mut Engine, game: i64, ending: Value) -> (i64, i64) {
    let arrival = json!({
        "purpose": "arrival",
        "content": {
            "description": "You push open the dry cell and find the prince alive.",
            "summary": "The dry cell is opened.",
        },
    });
    let mut ending = ending;
    ending["purpose"] = json!("ending");
    let submitted = submit(
        engine,
        game,
        "/move the dry cell",
        "cell",
        &[arrival, ending],
    );
    let closing = submitted.turned.scene.expect("the closing scene");
    assert_eq!(current_scene(engine, game), Some(closing));
    let previous = column::<Option<i64>>(
        engine,
        "SELECT previous_scene_id FROM scenes WHERE id = ?1",
        closing,
    )
    .expect("the turn's own scene");
    (closing, previous)
}

/// A scene's description, label, summary and fact, and whether its words are
/// the engine's own.
fn scene(engine: &Engine, id: i64) -> (String, String, String, String, bool) {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT description, resolved_action, COALESCE(summary, ''), \
             COALESCE(engine_fact, ''), COALESCE(engine_fallback, 0) FROM scenes WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap()
}

#[test]
fn an_ending_is_written_over_its_own_closing_scene_and_the_engines_sentence_outlives_it() {
    let mut engine = open_world(&world("the-iron-gate-descends")).unwrap();
    let rescued: String = engine
        .store()
        .connection()
        .query_row(
            "SELECT summary FROM quest_outcomes WHERE name = 'rescued'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let words = "You lift the prince to his feet, and the iron gate opens outward at last.";

    let game = at_the_cell(&mut engine);
    let (closing, turn) = into_the_cell(&mut engine, game, json!({ "content": words }));
    let (description, action, summary, _, fallback) = scene(&engine, turn);
    assert_eq!(
        (
            description.as_str(),
            action.as_str(),
            summary.as_str(),
            fallback
        ),
        (
            "You push open the dry cell and find the prince alive.",
            "move",
            "The dry cell is opened.",
            false
        ),
        "the turn's own paragraph is untouched by the ending after it"
    );
    assert_eq!(
        scene(&engine, closing),
        (
            words.into(),
            "ending".into(),
            rescued.clone(),
            rescued.clone(),
            false
        ),
        "the narrated ending is labelled as narration, over the outcome's own sentence"
    );

    for (token, ending) in [
        ("failed", json!({ "unavailable": true })),
        ("blank", json!({ "content": "   " })),
    ] {
        let game = at_the_cell(&mut engine);
        let (closing, turn) = into_the_cell(&mut engine, game, ending);
        assert_eq!(
            scene(&engine, closing),
            (
                rescued.clone(),
                "conclude".into(),
                rescued.clone(),
                rescued.clone(),
                false
            ),
            "{token}: the engine's sentence stands"
        );
        assert_eq!(
            column::<String>(
                &engine,
                "SELECT resolved_action FROM scenes WHERE id = ?1",
                turn
            ),
            "move"
        );
    }
}
