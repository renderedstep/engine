//! What a person in a room may be moved to do on their own, and what
//! becomes of an act once it is picked: the offer is built over this game's
//! records, and rebuilt when the act is applied, so a pick that is no longer
//! on offer moves nothing and is still written down. Every test stands in
//! the clerk's office, with Rowe beside Bell where a second person is
//! needed, at a clock where the speech die lets nobody speak, and has
//! System One pick the act.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::roll::{self, Seed};
use renderedstep_engine::{data, volition};
use serde_json::json;
use std::path::Path;

const BELL: i64 = 1_000_000_002;
const ROWE: i64 = 1_000_000_003;
const OFFICE: i64 = 1_000_000_001;
const LANDING: i64 = 1_000_000_002;
/// The world's own quarter receipt, lying in the office.
const RECEIPT_TEMPLATE: i64 = 1_000_000_001;
const BACK_STAIR: i64 = -910_002;

/// How many five-minute steps of the clock a search tries.
const TRIES: i64 = 400;

/// Rowe, standing beside Bell with a pursuit of her own.
const ROWE_BESIDE_BELL: &str = "
    CREATE TEMP TABLE rowe AS SELECT * FROM characters WHERE id = 1000000002;
    UPDATE rowe SET id = 1000000003, fullname = 'Rowe', nickname = 'Rowe',
      desire_pursuit = 'keep';
    INSERT INTO characters SELECT * FROM rowe;
    DROP TABLE rowe;";

/// A back stair out of the office whose row has a negative id, built once
/// the game has started in the office, the story's first room by id.
const A_BACK_STAIR: &str = "
    CREATE TEMP TABLE stair AS SELECT * FROM locations WHERE id = 1000000002;
    UPDATE stair SET id = -910002, name = 'The Back Stair';
    INSERT INTO locations SELECT * FROM stair;
    DROP TABLE stair;
    CREATE TEMP TABLE way AS SELECT * FROM location_connections WHERE id = 1000000001;
    UPDATE way SET id = 1000000009, connected_location_id = -910002;
    INSERT INTO location_connections SELECT * FROM way;
    DROP TABLE way;";

/// The clerk's office with `before` run over the world, a game started in
/// it, `after` run over that, and the clock `steps` five-minute steps on
/// from where the story starts it.
fn office(before: &str, after: &str, steps: i64) -> (Engine, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("parity/worlds/a-clerk-with-somewhere-to-be.sql"),
    )
    .expect("the clerk's world");
    let mut engine = open_world(&format!("{world}{before}")).unwrap();
    let story = engine
        .story_titled(&format!("A Clerk With Somewhere To Be{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    let conn = engine.store().connection();
    conn.execute_batch(after).unwrap();
    conn.execute(
        "UPDATE scenes SET story_timestamp = datetime(story_timestamp, ?1)
             WHERE id = (SELECT current_scene_id FROM playthroughs WHERE id = ?2)",
        rusqlite::params![format!("+{} seconds", steps * 300), playthrough],
    )
    .unwrap();
    (engine, playthrough)
}

/// The office as [`office`] builds it, at the first clock where the speech
/// die lets none of `people` speak.
fn quiet_office(before: &str, after: &str, people: &[i64]) -> (Engine, i64) {
    let steps = (0..TRIES)
        .find(|&steps| {
            let (engine, playthrough) = office(before, after, steps);
            let records = engine.store().load().unwrap();
            let game = Game::new(&records, playthrough);
            let room = game.current_location().expect("a room");
            people.iter().all(|&who| {
                let mut rng = Seed {
                    story: game.story_id().into(),
                    playthrough: playthrough.into(),
                    at: game.story_now().into(),
                    sequence: who.into(),
                    kind: roll::SPEECH,
                }
                .generator();
                volition::throw_speech(
                    &game,
                    game.character(who),
                    room,
                    data::speech().silent.turn,
                    &mut rng,
                )
                .is_none()
            })
        })
        .expect("a quiet clock within the tries");
    office(before, after, steps)
}

/// The acts on offer to `character` in `location`, as tokens.
fn tokens(engine: &Engine, playthrough: i64, character: i64, location: i64) -> Vec<String> {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    volition::choices(&game, game.character(character), game.location(location))
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

/// The label System One is offered `token` under, for `character` in the
/// office.
fn label(engine: &Engine, playthrough: i64, character: i64, token: &str) -> String {
    let at = tokens(engine, playthrough, character, OFFICE)
        .iter()
        .position(|offered| offered == token)
        .unwrap_or_else(|| panic!("{token} is on offer"));
    format!("act_{}", at + 1)
}

/// Plays `/examine quarter receipt` with System One answering `acts`.
fn look(engine: &mut Engine, playthrough: i64, acts: serde_json::Value) {
    let mut replay = Replay::new(vec![
        Reply::from_value(&json!({ "purpose": "narration", "content": "You look." })).unwrap(),
        Reply::from_value(&json!({ "purpose": "system_one", "content": acts })).unwrap(),
    ]);
    engine
        .submit(
            playthrough,
            "/examine quarter receipt",
            "look",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
}

/// `(character, chosen, status, fact)` of every volition, in id order.
fn volitions(engine: &Engine) -> Vec<(i64, String, String, String)> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare("SELECT character_id, chosen, status, fact FROM playthrough_volitions ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// `(location_id, following)` of this game's row for somebody.
fn npc_state(engine: &Engine, character: i64) -> (Option<i64>, bool) {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT location_id, following FROM playthrough_npc_states WHERE character_id = ?1",
            [character],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
}

/// This game's copy of the quarter receipt.
fn receipt(engine: &Engine, playthrough: i64) -> i64 {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT id FROM items WHERE template_id = ?1 AND playthrough_id = ?2",
            [RECEIPT_TEMPLATE, playthrough],
            |row| row.get(0),
        )
        .unwrap()
}

/// A thing that does not move for anybody is offered to nobody to pick up,
/// though the same thing a hand can lift is.
#[test]
fn a_thing_that_does_not_move_is_offered_to_nobody() {
    let (engine, playthrough) = office("", "", 0);
    let take = format!("take:{}", receipt(&engine, playthrough));
    assert!(tokens(&engine, playthrough, BELL, OFFICE).contains(&take));
    engine
        .store()
        .connection()
        .execute(
            "UPDATE items SET bulk = 'immovable' WHERE playthrough_id = ?1",
            [playthrough],
        )
        .unwrap();
    assert!(!tokens(&engine, playthrough, BELL, OFFICE).contains(&take));
}

/// Somebody who is not in the room is offered nothing there but waiting.
#[test]
fn somebody_absent_is_offered_only_waiting() {
    let (engine, playthrough) = office("", "", 0);
    engine
        .store()
        .connection()
        .execute(
            "INSERT INTO playthrough_npc_states (playthrough_id, character_id, location_id,
               created_at, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'), datetime('now'))",
            [playthrough, BELL, LANDING],
        )
        .unwrap();
    assert_eq!(tokens(&engine, playthrough, BELL, OFFICE), [volition::WAIT]);
    assert!(tokens(&engine, playthrough, BELL, LANDING).len() > 1);
}

/// Walking out of the room on their own ends any agreement to travel with
/// the party.
#[test]
fn a_walk_ends_a_travel_agreement() {
    let (mut engine, playthrough) = quiet_office("", "", &[BELL]);
    engine
        .store()
        .connection()
        .execute(
            "INSERT INTO playthrough_npc_states (playthrough_id, character_id, location_id,
               following, created_at, updated_at)
             VALUES (?1, ?2, ?3, 1, datetime('now'), datetime('now'))",
            [playthrough, BELL, OFFICE],
        )
        .unwrap();
    let walk = format!("move:{LANDING}");
    let act = label(&engine, playthrough, BELL, &walk);
    look(
        &mut engine,
        playthrough,
        json!({ "person_1:act": act, "person_1:pressure": 0.9 }),
    );
    let rows = volitions(&engine);
    assert_eq!(
        (rows[0].1.as_str(), rows[0].2.as_str()),
        (walk.as_str(), "applied")
    );
    assert_eq!(npc_state(&engine, BELL), (Some(LANDING), false));
}

/// Two people offered the same thing on one line: the first takes it, and
/// the second's pick, no longer on offer when it is applied, is rejected,
/// moves nothing and is still on the record.
#[test]
fn a_pick_no_longer_on_offer_is_rejected_moves_nothing_and_is_recorded() {
    let (mut engine, playthrough) = quiet_office(ROWE_BESIDE_BELL, "", &[BELL, ROWE]);
    let take = format!("take:{}", receipt(&engine, playthrough));
    let (bell, rowe) = (
        label(&engine, playthrough, BELL, &take),
        label(&engine, playthrough, ROWE, &take),
    );
    look(
        &mut engine,
        playthrough,
        json!({
            "person_1:act": bell, "person_1:pressure": 0.9,
            "person_2:act": rowe, "person_2:pressure": 0.9,
        }),
    );
    let rows = volitions(&engine);
    assert_eq!(
        rows.iter()
            .map(|row| (row.0, row.1.as_str(), row.2.as_str()))
            .collect::<Vec<_>>(),
        [
            (BELL, take.as_str(), "applied"),
            (ROWE, take.as_str(), "rejected")
        ]
    );
    assert_eq!(
        rows[1].3,
        "Rowe was going to act and could not: the act is no longer available. Nothing moved."
    );
    let holder: Option<i64> = engine
        .store()
        .connection()
        .query_row(
            "SELECT character_id FROM items WHERE id = ?1",
            [receipt(&engine, playthrough)],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(holder, Some(BELL), "still with whoever took it first");
}

/// A room whose row has a negative id is offered as a way out, picked and
/// walked into like any other.
#[test]
fn a_negative_row_id_is_offered_and_walked_like_any_other() {
    let (mut engine, playthrough) = quiet_office("", A_BACK_STAIR, &[BELL]);
    let walk = format!("move:{BACK_STAIR}");
    assert_eq!(walk, "move:-910002");
    let act = label(&engine, playthrough, BELL, &walk);
    look(
        &mut engine,
        playthrough,
        json!({ "person_1:act": act, "person_1:pressure": 0.9 }),
    );
    let rows = volitions(&engine);
    assert_eq!(
        (rows[0].1.as_str(), rows[0].2.as_str()),
        (walk.as_str(), "applied")
    );
    assert!(rows[0].3.contains("The Back Stair"), "{}", rows[0].3);
    assert_eq!(npc_state(&engine, BELL).0, Some(BACK_STAIR));
}
