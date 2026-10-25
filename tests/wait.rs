//! Letting time pass: the `wait` every room offers, reached through `wait`,
//! `rest`, `idle`, `think` and `pray`. It changes no thing, person or door;
//! a told turn writes its scene, the clock moves on, and the world takes its
//! turn as after any other line.
//!
//! No reader is handed it yet: it joins the readers' lists with their
//! retrain, so every reader request is as it was before the wait existed.

use renderedstep_engine::cascade;
use renderedstep_engine::classifier;
use renderedstep_engine::engine::Engine;
use renderedstep_engine::grammar::{Grammar, MORE_THAN_ONE_ACT};
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Mode;
use renderedstep_engine::records::Records;
use renderedstep_engine::room::{Person, Place, Room, Thing, WAIT};
use renderedstep_engine::turn::{chooser, Mechanics};
use std::path::Path;

/// Bell's row in the world file: the clerk who is due upstairs.
const BELL: i64 = 1_000_000_002;
const LANDING: i64 = 1_000_000_002;

fn office() -> Room {
    Room {
        protagonist: Some(Person {
            id: 1,
            fullname: "Wick".into(),
            nickname: None,
        }),
        here: Some(Place {
            id: 10,
            name: "The Records Office".into(),
        }),
        cast: vec![Person {
            id: 2,
            fullname: "Bell".into(),
            nickname: None,
        }],
        carried: vec![Thing::new(20, "folded paper", true)],
        ..Room::default()
    }
}

/// A Clerk With Somewhere To Be, a told game started in it.
fn clerk() -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("parity/worlds/a-clerk-with-somewhere-to-be.sql"),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("A Clerk With Somewhere To Be{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start_in(story, Mode::Narrated).unwrap();
    (engine, playthrough)
}

#[test]
fn every_room_offers_a_wait_last_and_a_finished_game_none() {
    let room = office();
    let offered = room.physical_actions();
    let wait = offered.last().expect("a wait at least");
    assert_eq!(wait.kind, WAIT);
    assert_eq!(wait.token(), "use:wait:0:0:0:0");
    assert_eq!(wait.name(), "Wait and let time pass");
    assert!(wait.records().is_empty());
    assert_eq!(offered.iter().filter(|c| c.kind == WAIT).count(), 1);
    assert_eq!(offered[0].kind, "offer", "the room's own attempts first");

    let bare = Room {
        cast: Vec::new(),
        carried: Vec::new(),
        ..office()
    };
    assert_eq!(bare.physical_actions().len(), 1);
    let over = Room {
        over: true,
        ..office()
    };
    assert!(over.physical_actions().is_empty());
    let nobody = Room {
        protagonist: None,
        ..office()
    };
    assert!(nobody.physical_actions().is_empty());
}

#[test]
fn every_word_for_it_reads_as_the_wait_whatever_follows() {
    let room = office();
    let grammar = Grammar::new(&room);
    for line in [
        "/wait",
        "/rest",
        "/idle",
        "/think",
        "/pray",
        "/Wait",
        "/wait for him to leave",
        "/think about what could have carried a total",
        "/pray to the Fair Copy that the water holds",
    ] {
        let reading = grammar.reading_first(line).expect("a slashed line");
        assert_eq!(reading.refusal, None, "{line}");
        let intent = reading.intent.expect(line);
        assert_eq!(intent.action, "use", "{line}");
        assert_eq!(
            intent.physical.as_ref().map(|c| c.kind.as_str()),
            Some(WAIT)
        );
        assert!(!intent.refused(), "{line}");
        assert!(intent.subject().is_none());
        assert_eq!(
            reading.understood.as_deref(),
            Some("use -> Wait and let time pass")
        );
        let parsed = grammar.parse(&line[1..]);
        assert_eq!(parsed.intent.and_then(|i| i.physical), intent.physical);
    }
    let wait = room.physical_actions().pop().unwrap();
    assert_eq!(grammar.line_for(&wait).as_deref(), Some("/wait"));
}

#[test]
fn a_wait_joined_to_a_second_act_is_refused() {
    let room = office();
    let grammar = Grammar::new(&room);
    for line in ["/wait and give Bell the folded paper", "/rest then go"] {
        let reading = grammar.parse(line);
        assert_eq!(
            reading.refusal.as_deref(),
            Some(MORE_THAN_ONE_ACT),
            "{line}"
        );
        assert_eq!(reading.intent, None);
        let first = grammar.reading_first(line).unwrap();
        assert_eq!(first.intent, None, "{line}");
    }
}

#[test]
fn a_wait_in_a_finished_game_reaches_nothing_and_is_refused() {
    let room = Room {
        over: true,
        ..office()
    };
    let intent = Grammar::new(&room).parse("wait").intent.unwrap();
    assert_eq!(intent.physical, None);
    assert!(intent.refused());
}

#[test]
fn a_told_wait_writes_its_scene_moves_the_clock_and_the_world_takes_its_turn() {
    let (mut engine, playthrough) = clerk();
    let connection = || engine.store().connection();
    let items: i64 = connection()
        .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
        .unwrap();
    let before: String = connection()
        .query_row("SELECT MAX(story_timestamp) FROM scenes", [], |row| {
            row.get(0)
        })
        .unwrap();
    let mut replay = Replay::new(vec![Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You stand by the counter and let the minutes go.",
        "prompt_includes": ["The player types: wait"],
        "prompt_excludes": ["What has ALREADY happened", "You let time pass"],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(playthrough, "/wait", "w", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(submitted.turned.refusal, None);
    let scene = submitted.turned.scene.expect("the wait's scene");
    let connection = engine.store().connection();
    let row: (
        String,
        Option<String>,
        String,
        String,
        Option<String>,
        String,
    ) = connection
        .query_row(
            "SELECT description, engine_fact, resolved_action, resolved_by, acted_on_type, \
             story_timestamp FROM scenes WHERE id = ?1",
            [scene],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(
        row,
        (
            "You stand by the counter and let the minutes go.".into(),
            None,
            WAIT.into(),
            "grammar".into(),
            None,
            row.5.clone(),
        )
    );
    let moved: i64 = connection
        .query_row(
            "SELECT CAST(strftime('%s', ?1) AS INTEGER) - CAST(strftime('%s', ?2) AS INTEGER)",
            [&row.5, &before],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(moved, 5 * 60, "one action's five minutes");
    let after: i64 = connection
        .query_row("SELECT COUNT(*) FROM items", [], |row| row.get(0))
        .unwrap();
    assert_eq!(after, items, "a wait makes and spends nothing");
    let bell: Option<i64> = connection
        .query_row(
            "SELECT location_id FROM playthrough_npc_states \
             WHERE playthrough_id = ?1 AND character_id = ?2",
            [playthrough, BELL],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(bell, Some(LANDING), "Bell walked out while Wick waited");
}

#[test]
fn the_game_never_chooses_a_wait_for_the_player() {
    let (engine, playthrough) = clerk();
    let glance = engine.glance(playthrough).unwrap();
    let use_targets = glance
        .verbs
        .iter()
        .find(|verb| verb.name == "use")
        .map(|verb| verb.targets.clone())
        .unwrap_or_default();
    assert!(
        use_targets
            .iter()
            .any(|target| target.token.as_deref() == Some("use:wait:0:0:0:0")
                && target.line.as_deref() == Some("/wait")),
        "the player's panel offers it: {use_targets:?}"
    );
    let mechanics = Mechanics::new(engine.store(), playthrough).unwrap();
    let candidates = chooser::candidates(&mechanics, false);
    assert!(!candidates.is_empty());
    assert!(candidates
        .iter()
        .all(|c| c.intent.physical.as_ref().is_none_or(|p| p.kind != WAIT)));
}

/// Every request a reader is sent for `typed` in `room`: the classifier
/// call's prompt and schema, and System One's state and questions.
fn reader_requests(room: &Room, typed: &str) -> String {
    let call = classifier::call(&Records::default(), room, typed);
    let state = cascade::State::new(room, typed);
    let questions = cascade::request(&state);
    format!(
        "{}\n{}\n{}\n{}\n{}",
        call.system.unwrap_or_default(),
        call.user,
        call.schema.unwrap_or_default(),
        state.to_json(),
        questions
    )
}

#[test]
fn no_reader_request_carries_the_wait() {
    let bare = Room {
        cast: Vec::new(),
        carried: Vec::new(),
        ..office()
    };
    assert_eq!(bare.physical_actions().len(), 1, "the room offers its wait");
    assert!(bare.reader_actions().is_empty());
    assert!(bare.reader_offered_for("use").is_empty());
    let sent = reader_requests(&bare, "wait for him to leave");
    assert!(sent.contains("## Physical Actions (token: one attempt)\nNone are available.\n"));
    let state = cascade::State::new(&bare, "wait").to_json();
    assert_eq!(state["physical_actions"], serde_json::json!({}));
    assert!(
        !sent.contains("target_use"),
        "no attempt question without attempts"
    );

    let room = office();
    assert_eq!(
        room.reader_actions(),
        room.physical_actions()[..room.physical_actions().len() - 1]
    );
    let sent = [bare, room]
        .iter()
        .map(|room| reader_requests(room, "wait"))
        .collect::<Vec<_>>()
        .join("\n");
    for word in ["use:wait", "Wait and let time pass", "attempt_2"] {
        assert!(!sent.contains(word), "a reader was handed {word:?}");
    }
}
