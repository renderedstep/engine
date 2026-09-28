//! The people in a room reacting to the party walking in: decided before the
//! arrival is asked for, told by that arrival, and claimed by its scene. The
//! test walks Wren from the gate passage into the porters' lodge, whose clock
//! the lodge's world starts where the dice have Mabry greet her and Osric
//! walk out to the back stair.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use serde_json::json;
use std::path::Path;

const MABRY: i64 = 1_000_000_002;
const OSRIC: i64 = 1_000_000_003;
const GRIST: i64 = 1_000_000_004;
/// A second porter, walking with Wren from the gate passage.
const TAM: i64 = 1_000_000_009;
const PASSAGE: i64 = 1_000_000_001;

fn world() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-lodge-inside-the-gate.sql"),
    )
    .expect("the lodge's world")
}

/// The lodge's world with Tam following Wren in the gate passage, and an
/// older walk of Grist's that no paragraph has told yet.
fn gate() -> (Engine, i64) {
    let mut engine = open_world(&world()).unwrap();
    engine
        .store()
        .connection()
        .execute_batch(&format!(
            "CREATE TEMP TABLE tam AS SELECT * FROM characters WHERE id = {MABRY};
             UPDATE tam SET id = {TAM}, fullname = 'Tam Quill', nickname = 'Tam',
               location_id = {PASSAGE};
             INSERT INTO characters SELECT * FROM tam;
             DROP TABLE tam;"
        ))
        .unwrap();
    let story = engine
        .story_titled(&format!("A Lodge Inside the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    engine
        .store()
        .connection()
        .execute_batch(&format!(
            "INSERT INTO playthrough_npc_states (playthrough_id, character_id, location_id,
               following, created_at, updated_at)
             VALUES ({playthrough}, {TAM}, {PASSAGE}, 1, datetime('now'), datetime('now'));
             INSERT INTO playthrough_volitions (playthrough_id, character_id, location_id,
               chosen, status, fact, serves, round, decided_by, created_at, updated_at)
             VALUES ({playthrough}, {GRIST}, 1000000003, 'move:{PASSAGE}', 'applied',
               'Grist Harrow walked out of The Toll Booth.', 'keep', 1, 'die',
               datetime('now'), datetime('now'));"
        ))
        .unwrap();
    (engine, playthrough)
}

/// `(id, character, chosen, status, fact, scene)` of every volition, in id
/// order.
type Act = (i64, i64, String, String, String, Option<i64>);

fn volitions(engine: &Engine) -> Vec<Act> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT id, character_id, chosen, status, fact, scene_id
             FROM playthrough_volitions ORDER BY id",
        )
        .unwrap();
    statement
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// Test 7: the reactions are written before the arrival is asked for, its
/// request tells them in the block after the records, its scene claims
/// exactly them, an older untold walk stays untold, whoever walked out is
/// named in the block and not among who is here, and the follower walking in
/// with the party does not react.
#[test]
fn the_room_reacts_before_its_arrival_and_the_arrival_tells_it() {
    let (mut engine, playthrough) = gate();
    let before = volitions(&engine);
    let greeted = "Mabry Quill spoke up unasked and greeted Wren Adley.";
    let left = "Osric Venn walked out of The Porters' Lodge to The Back Stair \
                and is no longer in The Porters' Lodge.";
    let mut replay = Replay::new(vec![Reply::from_value(&json!({
        "purpose": "arrival",
        "content": {
            "description": "Mabry greets you; the under-porter slips out.",
            "summary": "Wren comes into the lodge.",
        },
        "prompt_includes": [
            format!("## As You Come In\nThese people reacted to your arrival, recorded by \
                     the game. Narrate each as part of this arrival, in the order given. \
                     Anyone below who walked out is seen leaving as you come in; add nobody \
                     else. Nothing anyone says changes what is recorded above.\n\
                     {greeted}\n{left}\n"),
        ],
        "prompt_excludes": ["Grist Harrow walked out"],
    }))
    .unwrap()]);
    let submitted = engine
        .submit(
            playthrough,
            "/move The Porters' Lodge",
            "in",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
    let scene = submitted.turned.scene.expect("an arrival scene");

    let prompt = replay.sent()[0].to_string();
    let here = prompt
        .lines()
        .chain(prompt.split("\\n"))
        .find(|line| line.starts_with("Also here:"))
        .expect("who is here");
    assert!(here.contains("Mabry Quill"), "{here}");
    assert!(!here.contains("Osric Venn"), "the leaver is gone: {here}");
    assert!(here.contains("Tam Quill"), "the follower walked in: {here}");

    let after = volitions(&engine);
    let new: Vec<&Act> = after.iter().skip(before.len()).collect();
    let reacted: Vec<(i64, &str)> = new.iter().map(|act| (act.1, act.4.as_str())).collect();
    assert_eq!(reacted, [(MABRY, greeted), (OSRIC, left)]);
    assert!(
        new.iter().all(|act| act.5 == Some(scene)),
        "the arrival's scene claims exactly its reactions: {new:?}"
    );
    assert!(
        !after.iter().any(|act| act.1 == TAM),
        "the follower walked in with the party and did not react"
    );
    let grist = after.iter().find(|act| act.1 == GRIST).unwrap();
    assert_eq!(
        grist.5, None,
        "an older walk stays for a paragraph that tells it"
    );
}
