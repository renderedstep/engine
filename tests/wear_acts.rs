//! Putting a thing on and taking it off: the `wear` and `remove` attempts,
//! reached through `wear`, `don`, `put on`, `remove`, `doff` and `take off`.
//! A thing lying here is picked up and put on in one act; a worn thing stays
//! carried, and comes off before it is dropped, thrown, offered or burned.
//! Every refusal is in the engine's own words.
//!
//! No reader is handed either attempt yet: they join the readers' lists
//! with their retrain, so every reader request is as it was before anybody
//! could wear anything.

use renderedstep_engine::cascade;
use renderedstep_engine::classifier;
use renderedstep_engine::engine::Engine;
use renderedstep_engine::grammar::Grammar;
use renderedstep_engine::intent::Intent;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Mode;
use renderedstep_engine::records::Records;
use renderedstep_engine::refusal::Refusal;
use renderedstep_engine::room::{Choice, Person, Place, Record, Room, Thing, REMOVE, WEAR};
use renderedstep_engine::turn::{chooser, Mechanics};
use std::path::Path;

fn wearable(id: i64, name: &str, carried: bool, place: &str, worn: bool) -> Thing {
    Thing {
        bulk: "light".into(),
        wear: Some(place.into()),
        worn,
        ..Thing::new(id, name, carried)
    }
}

/// Wren in the cloakroom: an oilskin coat worn over a shirt she has not put
/// on, a scarf in her hands, a cap and a hat by the door and a bell beside
/// them.
fn cloakroom() -> Room {
    Room {
        protagonist: Some(Person {
            id: 1,
            fullname: "Wren Calloway".into(),
            nickname: None,
        }),
        here: Some(Place {
            id: 10,
            name: "The Cloakroom".into(),
        }),
        cast: vec![Person {
            id: 2,
            fullname: "Abel Moss".into(),
            nickname: None,
        }],
        carried: vec![
            wearable(20, "oilskin coat", true, "outer", true),
            wearable(21, "linen shirt", true, "body", false),
            wearable(22, "wool scarf", true, "neck", false),
        ],
        lying: vec![
            wearable(30, "flat cap", false, "head", false),
            wearable(31, "felt hat", false, "head", false),
            Thing {
                bulk: "light".into(),
                ..Thing::new(32, "hand-bell", false)
            },
        ],
        ..Room::default()
    }
}

/// The same room with nothing wearable in it: every thing as it was before
/// a thing could be worn.
fn undressed(room: &Room) -> Room {
    let strip = |things: &[Thing]| -> Vec<Thing> {
        things
            .iter()
            .map(|thing| Thing {
                wear: None,
                worn: false,
                ..thing.clone()
            })
            .collect()
    };
    Room {
        carried: strip(&room.carried),
        lying: strip(&room.lying),
        ..room.clone()
    }
}

fn dressing(room: &Room) -> Vec<(String, String, String)> {
    room.physical_actions()
        .into_iter()
        .filter(|choice| matches!(choice.kind.as_str(), WEAR | REMOVE))
        .map(|choice| (choice.token(), choice.name(), choice.argument()))
        .collect()
}

fn refusal(room: &Room, line: &str) -> String {
    let intent = Grammar::new(room)
        .parse(line)
        .intent
        .unwrap_or_else(|| panic!("{line:?} reads as an act"));
    assert!(intent.refused(), "{line:?} is refused");
    Refusal::for_intent(
        &intent,
        line,
        &room.offered_for(&intent.action),
        &room.items_carried(),
        None,
    )
    .expect("a refused line has the engine's words")
    .text()
}

fn played(room: &Room, line: &str) -> Choice {
    let intent = Grammar::new(room).parse(line).intent.unwrap();
    assert!(!intent.refused(), "{line:?} is played");
    intent.physical.expect("an attempt")
}

#[test]
fn a_wear_is_offered_where_the_place_is_free_and_a_remove_where_nothing_lies_over_it() {
    let room = cloakroom();
    assert_eq!(
        dressing(&room),
        [
            (
                "use:wear:22:0:0:0".to_string(),
                "Put on the wool scarf".to_string(),
                "wool scarf".to_string()
            ),
            (
                "use:wear:30:0:0:0".into(),
                "Put on the flat cap".into(),
                "flat cap".into()
            ),
            (
                "use:wear:31:0:0:0".into(),
                "Put on the felt hat".into(),
                "felt hat".into()
            ),
            (
                "use:remove:20:0:0:0".into(),
                "Take off the oilskin coat".into(),
                "oilskin coat".into()
            ),
        ],
        "no shirt under a worn coat, no bell, and the wait still last"
    );
    assert_eq!(room.physical_actions().last().unwrap().kind, "wait");

    // With the coat off, the shirt can go on, and with the shirt on and the
    // coat back over it, the shirt does not come off.
    let mut off = cloakroom();
    off.carried[0].worn = false;
    assert!(dressing(&off)
        .iter()
        .any(|(token, _, _)| token == "use:wear:21:0:0:0"));
    let mut layered = cloakroom();
    layered.carried[1].worn = true;
    assert!(!dressing(&layered)
        .iter()
        .any(|(token, _, _)| token == "use:remove:21:0:0:0"));

    // One thing on the head: with the cap on, the hat is not offered.
    let mut capped = cloakroom();
    let mut cap = capped.lying.remove(0);
    cap.carried = true;
    cap.worn = true;
    capped.carried.push(cap);
    assert!(!dressing(&capped)
        .iter()
        .any(|(token, _, _)| token == "use:wear:31:0:0:0"));

    // A thing that cannot be picked up is not put on from the floor.
    let mut fixed = cloakroom();
    fixed.lying[0].bulk = "fixed".into();
    assert!(!dressing(&fixed)
        .iter()
        .any(|(token, _, _)| token == "use:wear:30:0:0:0"));
}

#[test]
fn every_word_for_them_plays_the_attempt_and_the_slash_menu_completes_it() {
    let room = cloakroom();
    for line in [
        "/wear wool scarf",
        "/don the scarf",
        "/put on scarf",
        "wear scarf",
    ] {
        assert_eq!(played(&room, line).token(), "use:wear:22:0:0:0", "{line}");
    }
    assert_eq!(
        played(&room, "/put on the flat cap").token(),
        "use:wear:30:0:0:0"
    );
    for line in ["/remove oilskin coat", "/doff coat", "/take off the coat"] {
        assert_eq!(played(&room, line).token(), "use:remove:20:0:0:0", "{line}");
    }
    // `take` and `put down` read as they always did.
    let take = Grammar::new(&room).parse("/take flat cap").intent.unwrap();
    assert_eq!(take.action, "take");
    let drop = Grammar::new(&room).parse("/put down scarf").intent.unwrap();
    assert_eq!(drop.action, "drop");

    let grammar = Grammar::new(&room);
    for choice in room.physical_actions() {
        if matches!(choice.kind.as_str(), WEAR | REMOVE) {
            let line = grammar.line_for(&choice).expect("a line plays it");
            assert_eq!(played(&room, &line), choice);
        }
    }
    let menu = renderedstep_engine::slash_menu::SlashMenu::for_room(&room);
    let completes = |word: &str| {
        menu.targets
            .iter()
            .find(|(verb, _)| verb == word)
            .map(|(_, names)| names.clone())
    };
    assert_eq!(
        completes("wear"),
        Some(vec![
            "wool scarf".to_string(),
            "flat cap".into(),
            "felt hat".into()
        ])
    );
    assert_eq!(completes("remove"), Some(vec!["oilskin coat".to_string()]));
}

#[test]
fn a_line_that_cannot_dress_is_refused_in_the_engines_words() {
    let room = cloakroom();
    let nothing = " Nothing has changed.";
    let cases = [
        ("/wear linen shirt", "Take off the oilskin coat first."),
        (
            "/wear hand-bell",
            "The hand-bell is not something you can wear.",
        ),
        (
            "/wear oilskin coat",
            "You are already wearing the oilskin coat.",
        ),
        ("/remove scarf", "You are not wearing the wool scarf."),
        (
            "/remove gloves",
            "You are not wearing anything called that.",
        ),
        (
            "/remove flat cap",
            "You are not wearing anything called that.",
        ),
    ];
    for (line, said) in cases {
        assert_eq!(refusal(&room, line), format!("{said}{nothing}"), "{line}");
    }

    let mut capped = cloakroom();
    let mut cap = capped.lying.remove(0);
    cap.carried = true;
    cap.worn = true;
    capped.carried.push(cap);
    assert_eq!(
        refusal(&capped, "/wear felt hat"),
        format!("You are already wearing the flat cap on your head. Take it off first.{nothing}")
    );
    let mut layered = cloakroom();
    layered.carried[1].worn = true;
    assert_eq!(
        refusal(&layered, "/remove linen shirt"),
        format!("Take off the oilskin coat first.{nothing}")
    );

    let bare = Room {
        carried: Vec::new(),
        lying: Vec::new(),
        ..cloakroom()
    };
    assert_eq!(
        refusal(&bare, "/wear"),
        format!("You have nothing here you can wear.{nothing}")
    );
    assert_eq!(
        refusal(&bare, "/remove"),
        format!("You are not wearing anything.{nothing}")
    );
}

#[test]
fn a_worn_thing_comes_off_before_a_hand_lets_go_of_it() {
    let room = cloakroom();
    let said = "You are wearing the oilskin coat. Take it off first. Nothing has changed.";
    for line in [
        "/drop oilskin coat",
        "/throw oilskin coat at Abel Moss",
        "/give oilskin coat to Abel Moss",
    ] {
        assert_eq!(refusal(&room, line), said, "{line}");
    }
    // Read by a model too: the same refusal whoever read the line.
    let coat = Record::Thing(room.carried[0].clone());
    let dropped =
        Intent::new("drop").with(renderedstep_engine::intent::Slot::Item, Some(coat.clone()));
    assert!(dropped.refused());
    assert_eq!(dropped.lets_go_of_the_worn().map(|t| t.id), Some(20));
    // What is not worn is let go of as always.
    let scarf = Intent::new("drop").with(
        renderedstep_engine::intent::Slot::Item,
        Some(Record::Thing(room.carried[2].clone())),
    );
    assert!(!scarf.refused());
    // A pair is "them".
    let mut gloved = cloakroom();
    gloved
        .carried
        .push(wearable(23, "leather gloves", true, "hands", true));
    assert_eq!(
        refusal(&gloved, "/drop leather gloves"),
        "You are wearing the leather gloves. Take them off first. Nothing has changed."
    );
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
fn every_reader_request_is_byte_for_byte_what_it_was_before_anything_could_be_worn() {
    let dressed = cloakroom();
    let before = undressed(&dressed);
    assert!(dressing(&before).is_empty());
    assert!(!dressing(&dressed).is_empty());
    assert_eq!(dressed.reader_actions(), before.reader_actions());
    for action in renderedstep_engine::room::INTENTS {
        assert_eq!(
            dressed.reader_offered_for(action),
            before.reader_offered_for(action),
            "{action}"
        );
    }
    for typed in [
        "put on the flat cap",
        "take off my coat",
        "wear the scarf",
        "give the coat to Abel",
        "drop the coat",
    ] {
        let sent = reader_requests(&dressed, typed);
        assert_eq!(sent, reader_requests(&before, typed), "{typed}");
        for word in ["use:wear", "use:remove", "Put on the", "Take off the"] {
            assert!(!sent.contains(word), "a reader was handed {word:?}");
        }
    }
}

/// The Cloakroom at the Ferry, a told game started in it.
fn ferry() -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-cloakroom-at-the-ferry.sql"),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("The Cloakroom at the Ferry{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start_in(story, Mode::Narrated).unwrap();
    (engine, playthrough)
}

#[test]
fn a_told_wear_with_nothing_else_to_tell_is_said_in_the_engines_words() {
    let (mut engine, playthrough) = ferry();
    let mut replay = Replay::new(Vec::new());
    let submitted = engine
        .submit(playthrough, "/wear scarf", "w", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(submitted.turned.refusal, None);
    let scene = submitted.turned.scene.expect("the wear's scene");
    let connection = engine.store().connection();
    let told: (String, Option<String>) = connection
        .query_row(
            "SELECT description, engine_fact FROM scenes WHERE id = ?1",
            [scene],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(told, ("You put on the wool scarf.".to_string(), None));
    let worn: bool = connection
        .query_row(
            "SELECT worn FROM items WHERE playthrough_id = ?1 AND name = 'wool scarf'",
            [playthrough],
            |row| row.get(0),
        )
        .unwrap();
    assert!(worn);

    // Refused in the engine's words, and no reader is asked.
    let mut replay = Replay::new(Vec::new());
    let submitted = engine
        .submit(playthrough, "/drop scarf", "d", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(
        submitted.turned.refusal.map(|refusal| refusal.fact),
        Some("You are wearing the wool scarf. Take it off first.".to_string())
    );
}

#[test]
fn a_told_wear_with_more_to_tell_asks_the_narrator_with_no_fact_of_its_own() {
    let (mut engine, playthrough) = ferry();
    // A foe in the room strikes back, so the turn is more than the act.
    engine
        .store()
        .connection()
        .execute(
            "UPDATE characters SET hostile = 1 WHERE fullname = 'Abel Moss'",
            [],
        )
        .unwrap();
    let mut replay = Replay::new(vec![Reply::from_value(&serde_json::json!({
        "purpose": "narration",
        "content": "You wind the scarf twice about your throat.",
        "prompt_includes": ["The player types: wear scarf"],
        "prompt_excludes": ["What has ALREADY happened", "You put on", "wearing"],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(playthrough, "/wear scarf", "w", &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(submitted.turned.refusal, None);
    let scene = submitted.turned.scene.expect("the wear's scene");
    let told: (String, Option<String>) = engine
        .store()
        .connection()
        .query_row(
            "SELECT description, engine_fact FROM scenes WHERE id = ?1",
            [scene],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        told,
        (
            "You wind the scarf twice about your throat.".to_string(),
            None
        )
    );
}

#[test]
fn the_game_never_puts_a_thing_on_or_takes_it_off_for_the_player() {
    let (engine, playthrough) = ferry();
    let glance = engine.glance(playthrough).unwrap();
    let kinds: Vec<String> = glance
        .verbs
        .iter()
        .find(|verb| verb.name == "use")
        .map(|verb| verb.targets.iter().filter_map(|t| t.kind.clone()).collect())
        .unwrap_or_default();
    assert!(kinds.iter().any(|kind| kind == WEAR), "{kinds:?}");
    assert!(kinds.iter().any(|kind| kind == REMOVE), "{kinds:?}");
    let drop_targets: Vec<String> = glance
        .verbs
        .iter()
        .find(|verb| verb.name == "drop")
        .map(|verb| verb.targets.iter().map(|t| t.name.clone()).collect())
        .unwrap_or_default();
    assert!(
        !drop_targets.iter().any(|name| name == "oilskin coat"),
        "a worn coat is not offered to put down: {drop_targets:?}"
    );
    let mechanics = Mechanics::new(engine.store(), playthrough).unwrap();
    let candidates = chooser::candidates(&mechanics, false);
    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|c| c
        .intent
        .physical
        .as_ref()
        .is_none_or(|p| !matches!(p.kind.as_str(), WEAR | REMOVE))));
}
