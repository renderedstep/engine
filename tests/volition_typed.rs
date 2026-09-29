//! System One deciding what the die leaves open: what somebody the speech die
//! let speak says, or that they say nothing, and, read pressured enough, what
//! somebody does. The die still decides whether anybody speaks, and every
//! way the typed call fails ends at the die's own pick, saying why. The
//! ordinary turns stand in the clerk's office, as `tests/speech.rs` does;
//! the arrival walks into the porters' lodge, as `tests/reactions.rs` does.

use renderedstep_engine::engine::{Engine, Error};
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::roll::{self, Seed};
use renderedstep_engine::{data, volition};
use serde_json::{json, Value};
use std::path::Path;

const BELL: i64 = 1_000_000_002;
const ROWE: i64 = 1_000_000_003;
const MABRY: i64 = 1_000_000_002;
const OSRIC: i64 = 1_000_000_003;

/// How many five-minute steps of the clock a search tries.
const TRIES: i64 = 400;

fn world(file: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("parity/worlds")
            .join(file),
    )
    .expect("the world")
}

/// The clerk's office, with Rowe beside Bell when `rowe`, and the clock
/// `steps` five-minute steps on from where the story starts it.
fn office(rowe: bool, steps: i64) -> (Engine, i64) {
    let mut engine = open_world(&world("a-clerk-with-somewhere-to-be.sql")).unwrap();
    if rowe {
        engine
            .store()
            .connection()
            .execute_batch(&format!(
                "CREATE TEMP TABLE rowe AS SELECT * FROM characters WHERE id = {BELL};
                 UPDATE rowe SET id = {ROWE}, fullname = 'Rowe', nickname = 'Rowe',
                   desire_pursuit = 'keep';
                 INSERT INTO characters SELECT * FROM rowe;
                 DROP TABLE rowe;"
            ))
            .unwrap();
    }
    let story = engine
        .story_titled(&format!("A Clerk With Somewhere To Be{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    engine
        .store()
        .connection()
        .execute(
            "UPDATE scenes SET story_timestamp = datetime(story_timestamp, ?1)
             WHERE id = (SELECT current_scene_id FROM playthroughs WHERE id = ?2)",
            rusqlite::params![format!("+{} seconds", steps * 300), playthrough],
        )
        .unwrap();
    (engine, playthrough)
}

/// What each of `people` would say on the speech die in the player's room.
fn throws(engine: &Engine, playthrough: i64, people: &[i64]) -> Vec<Option<String>> {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let room = game.current_location().expect("a room");
    people
        .iter()
        .map(|&who| {
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
        })
        .collect()
}

/// The first clock at which the die throws what `want` asks of `people`.
fn when(rowe: bool, people: &[i64], want: impl Fn(&[Option<String>]) -> bool) -> i64 {
    (0..TRIES)
        .find(|&steps| {
            let (engine, playthrough) = office(rowe, steps);
            want(&throws(&engine, playthrough, people))
        })
        .expect("the die throws it within the tries")
}

/// The label System One is offered `token` under, for Bell in the office,
/// and the fact it is told by.
fn offered(engine: &Engine, playthrough: i64, token: &str) -> (String, String) {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let (bell, room) = (game.character(BELL), game.current_location().unwrap());
    let at = volition::speech_options(&game, bell, room)
        .iter()
        .position(|(offered, _)| offered == token)
        .expect("on offer");
    let fact = volition::speech_choices(&game, bell, room)[at].1.clone();
    (format!("speech_{}", at + 2), fact)
}

/// `(character, chosen, status, decided_by, system_one_error)` of every
/// volition, in id order.
type Row = (i64, String, String, String, Option<String>);

fn volitions(engine: &Engine) -> Vec<Row> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT character_id, chosen, status, decided_by, system_one_error
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
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn reply(value: Value) -> Reply {
    Reply::from_value(&value).unwrap()
}

fn typed(content: Value) -> Reply {
    reply(json!({ "purpose": "system_one", "content": content }))
}

fn unavailable() -> Reply {
    Reply::unavailable("system_one")
}

fn narration(content: &str) -> Reply {
    reply(json!({ "purpose": "narration", "content": content }))
}

/// Plays `/examine quarter receipt` answered by `replies`, and what was sent.
fn look(engine: &mut Engine, playthrough: i64, replies: Vec<Reply>) -> Vec<Value> {
    let mut replay = Replay::new(replies);
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
    replay.sent().to_vec()
}

fn questions(sent: &Value) -> Vec<String> {
    sent["questions"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}

/// Test 11: only whoever the die lets speak is asked what they say, the
/// question offers saying nothing first and then exactly what is on offer,
/// the answer is what is said and told, and the act of somebody read below
/// the pressure is the die's.
#[test]
fn the_die_decides_who_speaks_and_system_one_what() {
    let steps = when(true, &[BELL, ROWE], |thrown| {
        thrown.iter().all(Option::is_some)
    });
    let (mut engine, playthrough) = office(true, steps);
    let (label, told) = offered(&engine, playthrough, "speak:dismiss");
    let on_offer: Vec<Value> = {
        let records = engine.store().load().unwrap();
        let game = Game::new(&records, playthrough);
        let room = game.current_location().unwrap();
        std::iter::once(json!("Say nothing."))
            .chain(
                volition::speech_options(&game, game.character(BELL), room)
                    .into_iter()
                    .map(|(_, option)| json!(option)),
            )
            .collect()
    };
    let sent = look(
        &mut engine,
        playthrough,
        vec![
            typed(json!({ "person_1:speech": label })),
            reply(json!({
                "purpose": "narration",
                "content": "Bell waves you off.",
                "prompt_includes": [told],
            })),
            typed(json!({})),
        ],
    );

    let speech = &sent[0];
    assert_eq!(
        questions(speech),
        ["person_1:speech"],
        "Bell, whom the die let speak, is asked what; Rowe, capped, is not"
    );
    assert_eq!(speech["state"]["characters"]["person_1"]["name"], "Bell");
    assert_eq!(speech["state"]["player_action"], "examine quarter receipt");
    let options = &speech["questions"]["person_1:speech"];
    assert_eq!(options["type"], "choice");
    assert_eq!(options["instructions"], "Choose one option.");
    let criteria = options["criteria"].as_object().unwrap();
    assert_eq!(criteria.values().cloned().collect::<Vec<_>>(), on_offer);
    assert_eq!(
        criteria.keys().cloned().collect::<Vec<_>>(),
        (1..=on_offer.len())
            .map(|at| format!("speech_{at}"))
            .collect::<Vec<_>>()
    );
    assert!(on_offer.contains(&json!("Tell Wick to leave The Records Office.")));
    let spoken = volitions(&engine);

    let acts = &sent[2];
    assert_eq!(
        questions(acts),
        ["person_1:act", "person_1:serves", "person_1:pressure"],
        "only Rowe, who did not speak, is asked an act"
    );
    assert_eq!(acts["state"]["characters"]["person_1"]["name"], "Rowe");
    assert_eq!(
        spoken,
        [
            (
                BELL,
                "speak:dismiss".into(),
                "applied".into(),
                "system_one".into(),
                None
            ),
            (
                ROWE,
                spoken[1].1.clone(),
                spoken[1].2.clone(),
                "die".into(),
                None
            ),
        ],
        "Rowe's act, read below the pressure, is the die's"
    );
    assert!(!spoken[1].1.starts_with(volition::SPEAK));
}

/// Test 11: saying nothing writes nothing and leaves the person their act,
/// which System One decides when it reads them pressured enough.
#[test]
fn saying_nothing_leaves_the_act_and_a_pressured_act_replaces_the_die() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let sent = look(
        &mut engine,
        playthrough,
        vec![
            typed(json!({ "person_1:speech": "speech_1" })),
            narration("Bell says nothing."),
            typed(json!({ "person_1:act": "act_1", "person_1:pressure": 0.9 })),
        ],
    );
    assert_eq!(
        sent[2]["questions"]["person_1:act"]["criteria"]["act_1"],
        "Stay where you are and change nothing."
    );
    assert_eq!(
        volitions(&engine),
        [(
            BELL,
            "wait".into(),
            "none".into(),
            "system_one".into(),
            None
        )],
        "no speech row, and the act Bell was read pressured into"
    );
}

/// Test 11: a call that fails leaves the die its pick, and every row the
/// die then decides says so and why.
#[test]
fn a_failed_call_is_the_dies_pick_and_says_why() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let thrown = throws(&engine, playthrough, &[BELL])[0].clone().unwrap();
    look(
        &mut engine,
        playthrough,
        vec![unavailable(), narration("Bell speaks.")],
    );
    let rows = volitions(&engine);
    assert_eq!(rows.len(), 1, "Bell spoke, and so takes no act: {rows:?}");
    assert_eq!(rows[0].1, thrown, "what the die threw");
    assert_eq!(rows[0].3, "die_after_system_one_failed");
    assert_eq!(
        rows[0].4.as_deref(),
        Some("the sweep's System One provider is unavailable")
    );

    let steps = when(false, &[BELL], |thrown| thrown[0].is_none());
    let (mut engine, playthrough) = office(false, steps);
    look(
        &mut engine,
        playthrough,
        vec![narration("You look."), unavailable()],
    );
    let rows = volitions(&engine);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].3, "die_after_system_one_failed", "{rows:?}");
    assert!(rows[0].4.is_some());
}

/// Test 11: an answer outside the offered set is not acted on: what the die
/// threw is said, and the receipt says which answer it was.
#[test]
fn an_answer_off_the_offer_is_the_dies_pick() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let thrown = throws(&engine, playthrough, &[BELL])[0].clone().unwrap();
    look(
        &mut engine,
        playthrough,
        vec![
            typed(json!({ "person_1:speech": "speech_99" })),
            narration("Bell speaks."),
        ],
    );
    let rows = volitions(&engine);
    assert_eq!(rows[0].1, thrown);
    assert_eq!(rows[0].3, "die_after_system_one_failed");
    assert_eq!(
        rows[0].4.as_deref(),
        Some("person_1:speech answered \"speech_99\", which is not one of the options sent")
    );
}

/// Test 8 with System One on: a worker killed after the speech step never
/// asks again, and the next delivery finishes the line with what was said.
#[test]
fn a_worker_killed_after_a_typed_speech_does_not_ask_again() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let (label, _) = offered(&engine, playthrough, "speak:dismiss");
    let mut replay = Replay::new(vec![typed(json!({ "person_1:speech": label }))]);
    let stopped = engine.submit_stopping(
        playthrough,
        "/examine quarter receipt",
        "look",
        &mut replay,
        &mut |_| {},
        Some("speech"),
    );
    assert!(matches!(stopped, Err(Error::Stopped(step)) if step == "speech"));
    replay.finish().unwrap();

    look(
        &mut engine,
        playthrough,
        vec![narration("Bell waves you off.")],
    );
    let said: Vec<Row> = volitions(&engine)
        .into_iter()
        .filter(|row| row.1.starts_with(volition::SPEAK))
        .collect();
    assert_eq!(
        said,
        [(
            BELL,
            "speak:dismiss".into(),
            "applied".into(),
            "system_one".into(),
            None
        )],
        "said once, as System One chose"
    );
}

/// Test 11 on an arrival: one call asks about everybody who may react, with
/// what they say for whoever the die lets speak and what they do for
/// everybody not fighting the party. Somebody who says nothing acts, and
/// somebody read below the pressure acts as the die throws.
#[test]
fn the_room_reacts_as_system_one_answers() {
    let die = {
        let mut engine = open_world(&world("a-lodge-inside-the-gate.sql")).unwrap();
        let story = engine
            .story_titled(&format!("A Lodge Inside the Gate{TITLE_SUFFIX}"))
            .unwrap();
        let playthrough = engine.start(story).unwrap();
        let mut replay = Replay::new(vec![reply(json!({
            "purpose": "arrival",
            "content": { "description": "In.", "summary": "In." },
        }))]);
        engine
            .submit(
                playthrough,
                "/move The Porters' Lodge",
                "in",
                &mut replay,
                &mut |_| {},
            )
            .unwrap();
        replay.finish().unwrap();
        volitions(&engine)
    };
    assert_eq!(
        die.iter()
            .map(|row| (row.0, row.1.as_str()))
            .collect::<Vec<_>>()[..1],
        [(MABRY, "speak:greet")],
        "the die has Mabry greet the party: {die:?}"
    );

    let mut engine = open_world(&world("a-lodge-inside-the-gate.sql")).unwrap();
    let story = engine
        .story_titled(&format!("A Lodge Inside the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    let mut replay = Replay::new(vec![
        typed(json!({
            "person_1:speech": "speech_1",
            "person_1:act": "act_1",
            "person_1:pressure": 0.9,
        })),
        reply(json!({
            "purpose": "arrival",
            "content": { "description": "In.", "summary": "In." },
            "prompt_excludes": ["Mabry Quill spoke up unasked"],
        })),
    ]);
    engine
        .submit(
            playthrough,
            "/move The Porters' Lodge",
            "in",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    replay.finish().unwrap();
    let sent = &replay.sent()[0];
    assert_eq!(sent["state"]["location"], "The Porters' Lodge");
    assert_eq!(sent["state"]["player_action"], "move The Porters' Lodge");
    let asked = questions(sent);
    assert!(
        asked.starts_with(&[
            "person_1:act".into(),
            "person_1:serves".into(),
            "person_1:pressure".into(),
            "person_1:speech".into(),
        ]),
        "{asked:?}"
    );
    assert!(
        !asked.contains(&"person_2:speech".to_string()),
        "the die let only Mabry speak: {asked:?}"
    );
    let rows = volitions(&engine);
    assert_eq!(
        rows[0],
        (
            MABRY,
            "wait".into(),
            "none".into(),
            "system_one".into(),
            None
        ),
        "Mabry said nothing, and did as System One read her pressed to"
    );
    let osric_by_die = die.iter().find(|row| row.0 == OSRIC).unwrap();
    let osric = rows.iter().find(|row| row.0 == OSRIC).unwrap();
    assert_eq!(
        (osric.1.as_str(), osric.3.as_str()),
        (osric_by_die.1.as_str(), "die"),
        "Osric, read below the pressure, did as the die threw"
    );
}
