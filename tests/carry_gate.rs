//! The carrying gate in The Weighing House, the one world whose carrying is
//! limited, and the same world with the switch off: Nell Fisk (strength 5)
//! is refused what she cannot hold, Bram Hale (strength 16, holding a heavy
//! yoke and a weigh ticket) is offered only what fits him to take and only
//! what fits her to give, and with no `universes.carrying` every one of
//! those plays exactly as it did before. A break is held to the same gate in
//! The Furnished Rooms' clearing, its beech given a heavy branch.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::volition;
use std::path::Path;

const BRAM: i64 = 1_000_000_002;
const HOUSE: i64 = 1_000_000_001;

/// What switches the gate off: the world says nothing about carrying.
const UNLIMITED: &str = "UPDATE universes SET carrying = NULL;";

/// The Weighing House with `before` run over the world, and a game started
/// in it.
fn house(before: &str) -> (Engine, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-weighing-house.sql"),
    )
    .expect("the weighing house");
    let mut engine = open_world(&format!("{world}{before}")).unwrap();
    let story = engine
        .story_titled(&format!("The Weighing House{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    (engine, playthrough)
}

/// The line's refusal, or none where it was played.
fn refusal(engine: &mut Engine, playthrough: i64, line: &str) -> Option<String> {
    engine
        .play(playthrough, line, &mut |_| {})
        .unwrap()
        .report
        .refusal
}

/// The acts on offer to Bram, as tokens, less the ones that are not about a
/// thing.
fn brams_offers(engine: &Engine, playthrough: i64) -> Vec<String> {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let bram = records.find("characters", BRAM).unwrap();
    let house = records.find("locations", HOUSE).unwrap();
    let names = |token: &str| -> Option<String> {
        let (shape, item) = token.split_once(':')?;
        let item = records.find("items", item.parse().ok()?)?;
        let name = item.get("name")?.as_str()?.to_string();
        matches!(shape, "take" | "give").then(|| format!("{shape}:{name}"))
    };
    volition::choices(&game, bram, house)
        .iter()
        .filter_map(|(token, _)| names(token))
        .collect()
}

#[test]
fn a_heavy_thing_is_refused_to_a_weak_body_only_where_carrying_is_limited() {
    let (mut limited, game) = house("");
    let refused = refusal(&mut limited, game, "/take market slate").expect("refused");
    assert_eq!(
        refused,
        "The market slate is heavy. At strength 5 you cannot lift something that heavy. \
         Nothing has changed."
    );

    let (mut unlimited, game) = house(UNLIMITED);
    assert_eq!(refusal(&mut unlimited, game, "/take market slate"), None);
    let carrying = unlimited.read(game, Vec::new()).unwrap().state.carrying;
    assert_eq!(carrying.len(), 2, "the tally board and the slate");
}

#[test]
fn a_throw_from_the_floor_is_refused_before_any_die_only_where_carrying_is_limited() {
    let (mut limited, game) = house("");
    let outcome = limited
        .play(game, "/throw market slate at Bram Hale", &mut |_| {})
        .unwrap();
    assert!(outcome
        .report
        .refusal
        .is_some_and(|refused| refused.starts_with("The market slate is heavy.")));
    assert!(outcome.report.note.is_empty(), "{:?}", outcome.report.note);

    let (mut unlimited, game) = house(UNLIMITED);
    let outcome = unlimited
        .play(game, "/throw market slate at Bram Hale", &mut |_| {})
        .unwrap();
    assert_eq!(outcome.report.refusal, None);
    assert!(
        outcome.report.note[0].contains("strength -> "),
        "the lift is a strength check as before: {:?}",
        outcome.report.note
    );
}

#[test]
fn a_thing_already_carried_is_never_asked_again() {
    let (mut engine, game) = house("");
    assert_eq!(refusal(&mut engine, game, "/take brass scoop"), None);
    assert!(
        refusal(&mut engine, game, "/take pewter jug").is_some(),
        "her hands and her load are full"
    );
    let outcome = engine
        .play(game, "/throw brass scoop at Bram Hale", &mut |_| {})
        .unwrap();
    assert_eq!(outcome.report.refusal, None);
}

#[test]
fn the_panels_offer_no_take_the_gate_would_refuse() {
    let (engine, game) = house("");
    let glance = engine.glance(game).unwrap();
    let take = glance
        .verbs
        .iter()
        .find(|verb| verb.name == "take")
        .unwrap();
    let targets: Vec<&str> = take
        .targets
        .iter()
        .map(|target| target.name.as_str())
        .collect();
    assert!(!targets.contains(&"market slate"), "{targets:?}");
    assert!(targets.contains(&"brass scoop"), "{targets:?}");

    let (engine, game) = house(UNLIMITED);
    let glance = engine.glance(game).unwrap();
    let take = glance
        .verbs
        .iter()
        .find(|verb| verb.name == "take")
        .unwrap();
    assert!(take
        .targets
        .iter()
        .any(|target| target.name == "market slate"));
}

#[test]
fn a_person_is_offered_no_take_that_would_not_fit_them_and_no_give_that_would_not_fit_the_player() {
    let (engine, game) = house("");
    assert_eq!(
        brams_offers(&engine, game),
        ["give:weigh ticket"],
        "the yoke fills both his hands and nothing light lies here, and the yoke is more \
         than Nell bears"
    );

    let weaker = "UPDATE characters SET strength = 7 WHERE id = 1000000002;
        UPDATE items SET character_id = NULL, location_id = 1000000002 WHERE name = 'yoke';";
    let (engine, game) = house(weaker);
    assert_eq!(
        brams_offers(&engine, game),
        ["take:brass scoop", "take:pewter jug", "give:weigh ticket"],
        "with his hands free, at strength 7 he cannot lift the slate"
    );

    let (engine, game) = house(UNLIMITED);
    assert_eq!(
        brams_offers(&engine, game),
        [
            "take:market slate",
            "take:brass scoop",
            "take:pewter jug",
            "give:yoke",
            "give:weigh ticket"
        ]
    );
}

#[test]
fn a_wear_of_a_thing_lying_here_picks_it_up_and_is_held_to_the_gate() {
    let wearable = "UPDATE items SET wear = 'head' WHERE name = 'pewter jug';";
    let (mut limited, game) = house(wearable);
    assert_eq!(refusal(&mut limited, game, "/take brass scoop"), None);
    assert_eq!(
        refusal(&mut limited, game, "/wear pewter jug").as_deref(),
        Some(
            "You are already carrying as much as you can bear at strength 5 (the tally board \
             and the brass scoop). Put something down first. Nothing has changed."
        )
    );

    let (mut unlimited, game) = house(&format!("{wearable}{UNLIMITED}"));
    assert_eq!(refusal(&mut unlimited, game, "/take brass scoop"), None);
    assert_eq!(refusal(&mut unlimited, game, "/wear pewter jug"), None);
    let carrying = unlimited.read(game, Vec::new()).unwrap().state.carrying;
    assert_eq!(carrying.len(), 3, "the board, the scoop and the jug, worn");
}

#[test]
fn a_body_with_no_strength_recorded_is_not_limited() {
    let (mut engine, game) = house("UPDATE characters SET strength = NULL WHERE id = 1000000001;");
    assert_eq!(refusal(&mut engine, game, "/take market slate"), None);
}

/// The Furnished Rooms' clearing, its old beech given one heavy branch and
/// its protagonist strength 7, under `carrying`, with a game walked into the
/// clearing.
fn clearing(carrying: &str) -> (Engine, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-furnished-rooms.sql"),
    )
    .expect("the furnished rooms");
    let mut engine = open_world(&format!(
        "{world}INSERT INTO items (name, description, tier, location_id, within_id, how, hold, \
         bulk, fragility, kit_key, created_at, updated_at) VALUES ('branch', 'A bough.', \
         'part', 1000000002, 1000000007, 'attached', 'loose', 'heavy', 'sturdy', \
         'forest/old beech/attached/branch', '2026-10-02 13:25:45', '2026-10-02 13:25:45');\n\
         UPDATE items SET kit_key = 'forest/old beech' WHERE id = 1000000007;\n\
         UPDATE characters SET strength = 7 WHERE id = 1000000001;\n\
         UPDATE universes SET carrying = {carrying};\n"
    ))
    .unwrap();
    let story = engine
        .story_titled(&format!("The Furnished Rooms{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    engine
        .play(playthrough, "go to The Beech Clearing", &mut |_| {})
        .unwrap();
    (engine, playthrough)
}

#[test]
fn a_break_that_would_put_too_much_in_hand_is_refused_before_its_check() {
    let (mut limited, game) = clearing("'limited'");
    let outcome = limited
        .play(game, "/break branch off old beech", &mut |_| {})
        .unwrap();
    assert_eq!(
        outcome.report.refusal.as_deref(),
        Some(
            "The branch is heavy. At strength 7 you cannot lift something that heavy. \
             Nothing has changed."
        )
    );
    assert!(outcome.report.note.is_empty(), "{:?}", outcome.report.note);

    let (mut unlimited, game) = clearing("NULL");
    let outcome = unlimited
        .play(game, "/break branch off old beech", &mut |_| {})
        .unwrap();
    assert_eq!(outcome.report.refusal, None);
    assert!(
        outcome.report.note[0].contains("strength -> "),
        "the break is its strength check as before: {:?}",
        outcome.report.note
    );
}

#[test]
fn a_conversation_offers_no_give_that_would_not_fit_the_player() {
    let (mut engine, game) = house("");
    let outcome = engine
        .play_deciding(game, "talk to Bram", "give:yoke", &mut |_| {})
        .unwrap();
    assert!(outcome
        .report
        .refusal
        .is_some_and(|refused| refused.contains("it is unavailable")));
    assert_eq!(outcome.state.carrying.len(), 1, "the tally board alone");

    let (mut engine, game) = house(UNLIMITED);
    let outcome = engine
        .play_deciding(game, "talk to Bram", "give:yoke", &mut |_| {})
        .unwrap();
    assert_eq!(outcome.report.refusal, None);
    assert_eq!(
        outcome.state.carrying.len(),
        2,
        "the tally board and the yoke"
    );
}
