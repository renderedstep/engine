//! What the place itself takes: a room's hazard paid at the moment the
//! catalogue names and at no other, a doorway's paid by whoever walks it,
//! a toll that can end the game, and the paragraph that tells a toll and
//! claims it. Every test stands in A Turn at the Gate, whose one hazard is
//! the drop on the way from the Market into the Courtyard; anything else a
//! test needs is written onto the world's rows in SQL. Maren is
//! made peaceable throughout, so no blow of hers is mixed into what the
//! place took.

use renderedstep_engine::engine::{Engine, Submitted};
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::outcome::Outcome;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::{json, Value};
use std::path::Path;

const MARKET: i64 = 1_000_000_001;
const COURTYARD: i64 = 1_000_000_002;
const MAREN: i64 = 1_000_000_002;

/// The doorway from the Market into the Courtyard, which carries the drop.
const THE_DROP: i64 = 1_000_000_001;

/// A Turn at the Gate with `sql` run over it, and a game started in the
/// Market.
fn gate(sql: &str) -> (Engine, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-turn-at-the-gate.sql"),
    )
    .expect("the gate's world");
    let mut engine = open_world(&format!(
        "{world}UPDATE characters SET hostile = 0 WHERE id = {MAREN};{sql}"
    ))
    .unwrap();
    let story = engine
        .story_titled(&format!("A Turn at the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    (engine, game)
}

/// One line played with no model.
fn play(engine: &mut Engine, game: i64, line: &str) -> Outcome {
    engine.play(game, line, &mut |_| {}).unwrap()
}

/// One line submitted with exactly `replies` to answer its model calls.
fn submit(engine: &mut Engine, game: i64, line: &str, replies: Value) -> Submitted {
    let replies = replies
        .as_array()
        .unwrap()
        .iter()
        .map(|reply| Reply::from_value(reply).unwrap())
        .collect();
    let mut replay = Replay::new(replies);
    let submitted = engine
        .submit(game, line, line, &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    submitted
}

/// A toll as its row holds it.
#[derive(Debug)]
struct Toll {
    hazard: String,
    room: i64,
    doorway: Option<i64>,
    scene: Option<i64>,
    saved: bool,
    damage: i64,
    hp_after: i64,
}

/// Every toll written, oldest first.
fn tolls(engine: &Engine) -> Vec<Toll> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT hazard, location_id, location_connection_id, scene_id, saved, damage,
                    hp_after
             FROM playthrough_tolls ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |row| {
            Ok(Toll {
                hazard: row.get(0)?,
                room: row.get(1)?,
                doorway: row.get(2)?,
                scene: row.get(3)?,
                saved: row.get(4)?,
                damage: row.get(5)?,
                hp_after: row.get(6)?,
            })
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn hazards(engine: &Engine) -> Vec<String> {
    tolls(engine).into_iter().map(|toll| toll.hazard).collect()
}

/// The scene the game is standing in.
fn current_scene(engine: &Engine, game: i64) -> Option<i64> {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT current_scene_id FROM playthroughs WHERE id = ?1",
            [game],
            |row| row.get(0),
        )
        .unwrap()
}

/// The read-out's lines saying what the world took.
fn the_world(outcome: &Outcome) -> Vec<&str> {
    outcome
        .report
        .note
        .iter()
        .map(String::as_str)
        .filter(|line| line.starts_with("the world: "))
        .collect()
}

fn names(named: &[renderedstep_engine::outcome::Named]) -> Vec<&str> {
    named.iter().map(|thing| thing.name.as_str()).collect()
}

/// An airless room costs nothing to walk into, a toll on every line played
/// in it, and nothing for a read-out or an engine-view line; airless names
/// no save, so its die always lands.
#[test]
fn an_every_turn_room_is_free_to_walk_into_and_charges_each_line_played_in_it() {
    let (mut engine, game) = gate(&format!(
        "UPDATE locations SET hazard = 'airless', hazard_die = 4 WHERE id = {COURTYARD};
         UPDATE location_connections SET hazard = NULL, hazard_die = NULL;
         UPDATE characters SET location_id = {COURTYARD} WHERE id = {MAREN};"
    ));
    let arrived = play(&mut engine, game, "go to the courtyard");
    assert_eq!(
        arrived.report.change.as_deref(),
        Some("moved: Market -> Courtyard")
    );
    assert!(tolls(&engine).is_empty(), "walking in is free");

    play(&mut engine, game, "look");
    play(&mut engine, game, "stats");
    assert!(tolls(&engine).is_empty(), "a read-out is not a turn");

    for paid in 1..=2 {
        let talked = play(&mut engine, game, "talk to Maren");
        let tolls = tolls(&engine);
        assert_eq!(tolls.len(), paid, "one toll a line");
        let toll = tolls.last().unwrap();
        assert_eq!(
            (toll.hazard.as_str(), toll.room, toll.doorway),
            ("airless", COURTYARD, None)
        );
        assert!(!toll.saved, "airless is never saved against");
        assert!((1..=4).contains(&toll.damage), "one d4: {toll:?}");
        assert_eq!(talked.state.hp, Some(toll.hp_after));
        let said = the_world(&talked);
        assert_eq!(said.len(), 1);
        assert!(said[0].starts_with("the world: airless on Courtyard cost Cal "));
    }
}

/// An offer typed with no model is refused as prose: the thing stays in the
/// player's hands, no scene is written, and the room still takes its toll.
#[test]
fn an_offer_with_no_model_is_refused_as_prose_and_the_room_still_charges() {
    let (mut engine, game) = gate(&format!(
        "UPDATE locations SET hazard = 'airless', hazard_die = 4 WHERE id = {MARKET};"
    ));
    play(&mut engine, game, "take the red coin");
    assert_eq!(tolls(&engine).len(), 1);
    let scene = current_scene(&engine, game);

    let offered = play(&mut engine, game, "/offer red coin to Maren");
    assert!(
        offered
            .report
            .refusal
            .as_deref()
            .is_some_and(|refusal| refusal.contains("talking is prose")),
        "{:?}",
        offered.report
    );
    assert_eq!(offered.report.change, None);
    assert_eq!(names(&offered.state.carrying), ["red coin"]);
    let hers: Vec<String> = {
        let conn = engine.store().connection();
        let mut statement = conn
            .prepare("SELECT name FROM items WHERE playthrough_id = ?1 AND character_id = ?2")
            .unwrap();
        statement
            .query_map([game, MAREN], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(hers, ["brass key"]);
    assert_eq!(hazards(&engine), ["airless", "airless"]);
    assert_eq!(current_scene(&engine, game), scene);
}

/// A hazard key the catalogue has no entry for, on a room or a doorway,
/// rolls nothing at any moment.
#[test]
fn a_hazard_the_catalogue_has_no_entry_for_costs_nothing() {
    let (mut engine, game) = gate(
        "UPDATE locations SET hazard = 'haunted', hazard_die = 4;
         UPDATE location_connections SET hazard = 'haunted', hazard_die = 4;",
    );
    for line in ["talk to Maren", "go to the courtyard", "go to the market"] {
        let outcome = play(&mut engine, game, line);
        assert!(
            the_world(&outcome).is_empty(),
            "{line}: {:?}",
            outcome.report
        );
    }
    assert!(tolls(&engine).is_empty());
}

/// A hazardous doorway into a room flooded on arrival is two tolls, the
/// doorway's and then the room's, both paid where the player ends up.
#[test]
fn a_hazardous_door_into_a_hazardous_room_is_two_tolls_doorway_first() {
    let (mut engine, game) = gate(&format!(
        "UPDATE locations SET hazard = 'flooded', hazard_die = 4 WHERE id = {COURTYARD};"
    ));
    let arrived = play(&mut engine, game, "go to the courtyard");
    let tolls = tolls(&engine);
    let paid: Vec<(&str, i64, Option<i64>)> = tolls
        .iter()
        .map(|toll| (toll.hazard.as_str(), toll.room, toll.doorway))
        .collect();
    assert_eq!(
        paid,
        [
            ("drop", COURTYARD, Some(THE_DROP)),
            ("flooded", COURTYARD, None)
        ]
    );
    let said = the_world(&arrived);
    assert_eq!(said.len(), 2);
    assert!(said[0].contains("drop on the way from Market into Courtyard"));
    assert!(said[1].contains("flooded on Courtyard"));
}

/// A toll that takes the last hit point kills, and the game is over.
#[test]
fn a_toll_that_takes_the_last_hit_point_ends_the_game() {
    let (mut engine, game) = gate("");
    play(&mut engine, game, "harm 52");
    engine
        .store()
        .connection()
        .execute(
            "UPDATE locations SET hazard = 'airless', hazard_die = 4 WHERE id = ?1",
            [MARKET],
        )
        .unwrap();

    let killed = play(&mut engine, game, "talk to Maren");
    let paid = tolls(&engine);
    assert_eq!(paid.len(), 1);
    assert_eq!((paid[0].hazard.as_str(), paid[0].hp_after), ("airless", 0));
    assert!(killed.state.dead);
    assert_eq!(killed.state.hp, Some(0));

    let after = play(&mut engine, game, "talk to Maren");
    assert!(after
        .report
        .refusal
        .as_deref()
        .is_some_and(|refusal| refusal.contains("this playthrough is over")));
    assert_eq!(
        tolls(&engine).len(),
        1,
        "a game that is over pays nothing more"
    );
}

/// Once a doorway's toll has killed, the room it led into charges nothing
/// off the body. The player has no dexterity to save with, so the drop
/// always lands.
#[test]
fn a_doorway_that_kills_leaves_the_room_nothing_to_charge() {
    let (mut engine, game) = gate(&format!(
        "UPDATE characters SET dexterity = NULL WHERE is_protagonist = 1;
         UPDATE locations SET hazard = 'flooded', hazard_die = 4 WHERE id = {COURTYARD};"
    ));
    play(&mut engine, game, "harm 52");

    let arrived = play(&mut engine, game, "go to the courtyard");
    assert_eq!(hazards(&engine), ["drop"]);
    assert_eq!(tolls(&engine)[0].hp_after, 0);
    assert!(arrived.state.dead);
    assert_eq!(the_world(&arrived).len(), 1);
}

/// A protagonist with no stat block has no hit points to pay with, and a
/// game with no protagonist has nobody to pay: neither writes a toll on a
/// line played in an airless room or a walk through a drop into a flood.
#[test]
fn a_body_with_no_stat_block_and_a_game_with_no_body_pay_no_toll() {
    let hazardous = format!(
        "UPDATE locations SET hazard = 'airless', hazard_die = 4 WHERE id = {MARKET};
         UPDATE locations SET hazard = 'flooded', hazard_die = 4 WHERE id = {COURTYARD};"
    );
    for (who, sql) in [
        (
            "no stat block",
            "UPDATE characters SET level = NULL, hit_die = NULL WHERE is_protagonist = 1;",
        ),
        (
            "no protagonist",
            "UPDATE characters SET is_protagonist = 0;",
        ),
    ] {
        let (mut engine, game) = gate(&format!("{hazardous}{sql}"));
        let talked = play(&mut engine, game, "talk to Maren");
        let arrived = play(&mut engine, game, "go to the courtyard");
        assert_eq!(
            arrived.report.change.as_deref(),
            Some("moved: Market -> Courtyard"),
            "{who}"
        );
        assert!(tolls(&engine).is_empty(), "{who}");
        assert_eq!(arrived.state.hp, None, "{who}");
        assert!(
            the_world(&talked).is_empty() && the_world(&arrived).is_empty(),
            "{who}"
        );
    }
}

/// A narration reply with `content`, whose prompt must include and leave out
/// the texts given.
fn narration(includes: &[&str], excludes: &[&str]) -> Value {
    json!({
        "purpose": "narration",
        "content": "It is done.",
        "prompt_includes": includes,
        "prompt_excludes": excludes,
    })
}

/// What the narrator is told the place took.
const THE_PLACE: &str = "The place itself, recorded by the game: ";

/// An airless room charges a line that did something else entirely, after
/// that line's paragraph; the next paragraph is told the toll and claims it,
/// and its own line's toll waits in turn.
#[test]
fn an_every_turn_toll_is_told_by_the_next_paragraph_and_claimed_by_it() {
    let (mut engine, game) = gate(&format!(
        "UPDATE locations SET hazard = 'airless', hazard_die = 4 WHERE id = {MARKET};"
    ));
    let took = submit(
        &mut engine,
        game,
        "/take red coin",
        json!([narration(&[], &[THE_PLACE])]),
    );
    assert!(took.turned.scene.is_some());
    let first = tolls(&engine);
    assert_eq!(first.len(), 1, "the line paid for standing there");
    assert_eq!(first[0].scene, None, "and it was paid after the paragraph");

    let dropped = submit(
        &mut engine,
        game,
        "/drop red coin",
        json!([narration(&[&format!("{THE_PLACE}Market cost Cal ")], &[])]),
    );
    let scene = dropped.turned.scene.expect("a paragraph");
    assert_eq!(current_scene(&engine, game), Some(scene));
    let scenes: Vec<Option<i64>> = tolls(&engine).iter().map(|toll| toll.scene).collect();
    assert_eq!(scenes, [Some(scene), None]);
}

/// The toll an arrival pays is paid before the arrival is told, so the
/// arrival's paragraph tells it and claims it, and the next paragraph is
/// not told it again.
#[test]
fn an_arrivals_toll_is_told_and_claimed_by_the_arrival() {
    let (mut engine, game) = gate(&format!(
        "UPDATE locations SET hazard = 'flooded', hazard_die = 4 WHERE id = {COURTYARD};"
    ));
    let crossing = "the way from Market into Courtyard";
    let arrival = |includes: &[&str], excludes: &[&str]| {
        json!({
            "purpose": "arrival",
            "content": {"description": "The courtyard opens before you.", "summary": "Arrived."},
            "prompt_includes": includes,
            "prompt_excludes": excludes,
        })
    };
    let arrived = submit(
        &mut engine,
        game,
        "/move Courtyard",
        json!([arrival(&[crossing], &[])]),
    );
    let scene = arrived.turned.scene.expect("an arrival");
    let claimed: Vec<(String, Option<i64>)> = tolls(&engine)
        .into_iter()
        .map(|toll| (toll.hazard, toll.scene))
        .collect();
    assert_eq!(
        claimed,
        [
            ("drop".to_string(), Some(scene)),
            ("flooded".to_string(), Some(scene))
        ]
    );

    submit(
        &mut engine,
        game,
        "/move Market",
        json!([arrival(&[], &[crossing, THE_PLACE])]),
    );
    assert_eq!(tolls(&engine).len(), 2, "the way back is free");
}

/// An attack writes no scene, so the toll its line paid is left for the
/// next paragraph, which tells it and claims it.
#[test]
fn an_attack_turn_leaves_its_toll_for_the_next_paragraph() {
    let (mut engine, game) = gate(&format!(
        "UPDATE locations SET hazard = 'airless', hazard_die = 4 WHERE id = {MARKET};"
    ));
    let attacked = submit(&mut engine, game, "/attack Maren", json!([]));
    assert_eq!(attacked.turned.scene, None);
    let paid = tolls(&engine);
    assert_eq!(paid.len(), 1);
    assert_eq!((paid[0].hazard.as_str(), paid[0].scene), ("airless", None));

    let took = submit(
        &mut engine,
        game,
        "/take red coin",
        json!([narration(&[&format!("{THE_PLACE}Market cost Cal ")], &[])]),
    );
    let scene = took.turned.scene.expect("a paragraph");
    assert_eq!(tolls(&engine)[0].scene, Some(scene));
}
