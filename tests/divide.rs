//! Breaking a thing apart: a `tear` of paper or cloth and a `smash` of
//! glass or pottery, offered on a thing in the player's hands or lying here
//! where its material allows it, and refused in the engine's words on a
//! thing an open beat of the story is bound to. The thing ends, `divided`,
//! and one row of what it leaves begins where it was.
//!
//! No reader is handed them yet: they join the readers' lists with their
//! retrain, so every reader request is byte for byte what it would be if
//! nothing in the room could be broken apart.

use renderedstep_engine::cascade;
use renderedstep_engine::classifier;
use renderedstep_engine::divide::{DIVIDED, SMASH, TEAR};
use renderedstep_engine::engine::Engine;
use renderedstep_engine::grammar::Grammar;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Mode;
use renderedstep_engine::records::Records;
use renderedstep_engine::room::{Person, Place, Room, Thing};
use renderedstep_engine::turn::{chooser, Mechanics};
use std::path::Path;

fn made_of(id: i64, name: &str, material: &str, carried: bool) -> Thing {
    Thing {
        material: Some(material.into()),
        combustible: matches!(material, "paper" | "cloth"),
        ..Thing::new(id, name, carried)
    }
}

fn copying_room() -> Room {
    let tinderbox = Thing {
        use_kind: "firestarter".into(),
        ..made_of(21, "tinderbox", "metal", true)
    };
    Room {
        protagonist: Some(Person {
            id: 1,
            fullname: "Tamsin Rook".into(),
            nickname: None,
        }),
        here: Some(Place {
            id: 10,
            name: "The Copying Room".into(),
        }),
        cast: vec![Person {
            id: 2,
            fullname: "Wren".into(),
            nickname: None,
        }],
        carried: vec![made_of(20, "letter", "paper", true), tinderbox],
        lying: vec![
            Thing {
                bound: true,
                ..made_of(30, "blank writ", "paper", false)
            },
            made_of(31, "linen apron", "cloth", false),
            made_of(32, "glass jar", "glass", false),
            made_of(33, "clay pot", "pottery", false),
            made_of(34, "iron key", "metal", false),
        ],
        abilities: vec![
            ("strength".into(), 12),
            ("dexterity".into(), 12),
            ("will".into(), 12),
        ],
        ..Room::default()
    }
}

/// The same room with nothing in it of a known material: what every reader
/// request was before anything could be broken apart.
fn unmade(room: &Room) -> Room {
    let forget = |things: &[Thing]| -> Vec<Thing> {
        things
            .iter()
            .cloned()
            .map(|thing| Thing {
                material: None,
                ..thing
            })
            .collect()
    };
    Room {
        carried: forget(&room.carried),
        lying: forget(&room.lying),
        ..room.clone()
    }
}

#[test]
fn the_room_offers_each_break_apart_its_materials_allow_after_burning() {
    let room = copying_room();
    let tokens: Vec<String> = room
        .physical_actions()
        .iter()
        .filter(|choice| [TEAR, SMASH, "burn"].contains(&choice.kind.as_str()))
        .map(|choice| format!("{} {}", choice.token(), choice.name()))
        .collect();
    assert_eq!(
        tokens,
        [
            "use:burn:20:0:0:21 Burn letter with tinderbox",
            "use:burn:31:0:0:21 Burn linen apron with tinderbox",
            "use:tear:20:0:0:0 Tear the letter",
            "use:tear:31:0:0:0 Tear the linen apron",
            "use:smash:32:0:0:0 Smash the glass jar",
            "use:smash:33:0:0:0 Smash the clay pot",
        ],
        "nothing of the writ the story needs, and nothing of the iron key"
    );
    let guarded: Vec<String> = room
        .guarded_actions()
        .iter()
        .map(|choice| choice.token())
        .collect();
    assert_eq!(guarded, ["use:burn:30:0:0:21", "use:tear:30:0:0:0"]);
}

#[test]
fn a_weak_player_is_not_offered_the_cloth() {
    let room = Room {
        abilities: vec![("strength".into(), 2)],
        ..copying_room()
    };
    let kinds: Vec<String> = room
        .physical_actions()
        .iter()
        .filter(|choice| choice.kind == TEAR)
        .map(|choice| choice.name())
        .collect();
    assert_eq!(kinds, ["Tear the letter"]);
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
fn every_reader_request_is_byte_for_byte_what_it_was_before() {
    let room = copying_room();
    let before = unmade(&room);
    assert!(room.physical_actions().len() > before.physical_actions().len());
    assert_eq!(room.reader_actions(), before.reader_actions());
    for typed in [
        "tear up the letter",
        "smash the glass jar",
        "rip the apron",
        "burn the letter",
        "take the clay pot",
    ] {
        let sent = reader_requests(&room, typed);
        assert_eq!(sent, reader_requests(&before, typed), "{typed}");
        for word in ["use:tear", "use:smash", "Tear the", "Smash the"] {
            assert!(!sent.contains(word), "a reader was handed {word:?}");
        }
    }
}

#[test]
fn the_game_never_chooses_to_break_a_thing_apart() {
    let (engine, playthrough) = copying_desk();
    let glance = engine.glance(playthrough).unwrap();
    let lines: Vec<String> = glance
        .verbs
        .iter()
        .find(|verb| verb.name == "use")
        .map(|verb| verb.targets.clone())
        .unwrap_or_default()
        .iter()
        .filter_map(|target| target.line.clone())
        .collect();
    assert!(
        lines.contains(&"/tear letter".to_string()) && lines.contains(&"/smash glass jar".into()),
        "the player's panel offers them: {lines:?}"
    );
    let mechanics = Mechanics::new(engine.store(), playthrough).unwrap();
    let candidates = chooser::candidates(&mechanics, false);
    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|c| c
        .intent
        .physical
        .as_ref()
        .is_none_or(|p| ![TEAR, SMASH].contains(&p.kind.as_str()))));
}

/// A Writ On The Copying Desk, a told game started in it.
fn copying_desk() -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-writ-on-the-copying-desk.sql"),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("A Writ On The Copying Desk{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start_in(story, Mode::Narrated).unwrap();
    (engine, playthrough)
}

/// `(name, disposition, where)` of every row of this game's whose name ends
/// in `like`: `where` is `hands`, `here` or, for a thing in no place,
/// `nowhere`.
fn rows_named(engine: &Engine, playthrough: i64, like: &str) -> Vec<(String, String, String)> {
    let connection = engine.store().connection();
    let mut statement = connection
        .prepare(
            "SELECT name, disposition, CASE WHEN location_id IS NOT NULL THEN 'here' \
             WHEN character_id IS NOT NULL THEN 'held' WHEN disposition = 'intact' \
             THEN 'hands' ELSE 'nowhere' END \
             FROM items WHERE playthrough_id = ?1 AND name LIKE ?2 ORDER BY id",
        )
        .unwrap();
    statement
        .query_map(rusqlite::params![playthrough, format!("%{like}")], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn a_told_tear_ends_the_letter_and_puts_its_scraps_in_hand() {
    let (mut engine, playthrough) = copying_desk();
    let mut replay = Replay::new(vec![Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You tear the letter across, and across again.",
        "prompt_includes": [
            "ON THIS TURN, and not before it, Tamsin Rook tore the letter apart.",
            "What is left of it is the scraps of the letter, in their hands.",
        ],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(
            playthrough,
            "/tear up the letter",
            "t",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(submitted.turned.refusal, None);
    assert_eq!(
        rows_named(&engine, playthrough, "letter"),
        [
            ("letter".into(), DIVIDED.into(), "nowhere".into()),
            (
                "scraps of the letter".into(),
                "intact".into(),
                "hands".into()
            ),
        ]
    );
    let scene = submitted.turned.scene.expect("the tear's scene");
    let action: String = engine
        .store()
        .connection()
        .query_row(
            "SELECT resolved_action FROM scenes WHERE id = ?1",
            [scene],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(action, TEAR);
}

#[test]
fn the_writ_the_story_needs_is_refused_in_the_engines_words_until_it_is_held() {
    let (mut engine, playthrough) = copying_desk();
    for line in [
        "/tear the blank writ",
        "/burn the blank writ with the tinderbox",
    ] {
        let mut replay = Replay::new(Vec::new());
        let submitted = engine
            .submit(playthrough, line, line, &mut replay, &mut |_| {})
            .unwrap();
        let refusal = submitted.turned.refusal.expect("refused");
        assert_eq!(refusal.kind, "unplayable");
        assert!(
            refusal
                .fact
                .starts_with("The story still needs the blank writ, so it cannot be "),
            "{}",
            refusal.fact
        );
    }
    assert_eq!(
        rows_named(&engine, playthrough, "writ"),
        [("blank writ".into(), "intact".into(), "here".into())]
    );
    let mechanics = Mechanics::new(engine.store(), playthrough).unwrap();
    let room = renderedstep_engine::turn::room_of(mechanics.records(), playthrough);
    assert!(Grammar::new(&room)
        .guarded("/tear the blank writ")
        .is_some());
    assert!(Grammar::new(&room).guarded("/tear the letter").is_none());

    engine
        .play(playthrough, "take blank writ", &mut |_| {})
        .unwrap();
    let outcome = engine
        .play(playthrough, "/tear the blank writ", &mut |_| {})
        .unwrap();
    assert_eq!(outcome.report.refusal, None);
    assert_eq!(
        rows_named(&engine, playthrough, "writ"),
        [
            ("blank writ".into(), DIVIDED.into(), "nowhere".into()),
            (
                "scraps of the blank writ".into(),
                "intact".into(),
                "hands".into()
            ),
        ]
    );
}
