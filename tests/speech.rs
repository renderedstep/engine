//! Somebody speaking up unasked on an ordinary turn: decided on its own die
//! before the paragraph, told by that paragraph, and one choice per person
//! per line. Every test stands in the clerk's office, with Rowe beside Bell
//! where a second person is needed, and moves the story's clock until the
//! speech die throws what the test needs; the die is seeded off that clock.

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

/// How many five-minute steps of the clock a search tries.
const TRIES: i64 = 400;

fn world() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("parity/worlds/a-clerk-with-somewhere-to-be.sql"),
    )
    .expect("the clerk's world")
}

/// The clerk's office, with Rowe standing beside Bell when `rowe`, and the
/// clock `steps` five-minute steps on from where the story starts it.
fn office(rowe: bool, steps: i64) -> (Engine, i64) {
    let mut engine = open_world(&world()).unwrap();
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

/// What each of `people` would say on the speech die, from the rows as they
/// stand, in the room the player is in.
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

/// The first clock at which the die throws what `want` asks of `people`, on
/// a line that changes nothing before somebody may speak.
fn when(rowe: bool, people: &[i64], want: impl Fn(&[Option<String>]) -> bool) -> i64 {
    (0..TRIES)
        .find(|&steps| {
            let (engine, playthrough) = office(rowe, steps);
            want(&throws(&engine, playthrough, people))
        })
        .expect("the die throws it within the tries")
}

/// The first clock at which playing `play` leaves somebody's speech row.
fn when_spoken(rowe: bool, play: impl Fn(&mut Engine, i64)) -> i64 {
    (0..TRIES)
        .find(|&steps| {
            let (mut engine, playthrough) = office(rowe, steps);
            play(&mut engine, playthrough);
            !speech(&engine).is_empty()
        })
        .expect("somebody speaks within the tries")
}

/// `(id, character, chosen, status, fact, serves, scene)` of every volition,
/// in id order.
type Act = (i64, i64, String, String, String, String, Option<i64>);

fn volitions(engine: &Engine) -> Vec<Act> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT id, character_id, chosen, status, fact, serves, scene_id
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
                row.get(6)?,
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn speech(engine: &Engine) -> Vec<Act> {
    volitions(engine)
        .into_iter()
        .filter(|act| act.2.starts_with(volition::SPEAK))
        .collect()
}

fn reply(value: Value) -> Reply {
    Reply::from_value(&value).unwrap()
}

fn narration(content: &str) -> Reply {
    reply(json!({ "purpose": "narration", "content": content }))
}

fn submit(
    engine: &mut Engine,
    playthrough: i64,
    line: &str,
    token: &str,
    replies: Vec<Reply>,
) -> Option<i64> {
    let mut replay = Replay::new(replies);
    let submitted = engine
        .submit(playthrough, line, token, &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    submitted.turned.scene
}

fn look(engine: &mut Engine, playthrough: i64, token: &str) -> Option<i64> {
    submit(
        engine,
        playthrough,
        "/examine quarter receipt",
        token,
        vec![narration("You look.")],
    )
}

fn talk(content: &str) -> Vec<Reply> {
    vec![
        reply(json!({ "purpose": "character", "content": {
            "pre_thought": "Wick is here.", "pre_feeling": "hurried",
            "action": "Bell looks up.", "post_thought": "I am late.",
            "post_feeling": "restless", "inner_resolution": "I will go up.",
            "engine_action": "none" } })),
        reply(json!({ "purpose": "interaction-narration", "content": content })),
    ]
}

/// Test 4: one speaker on an ordinary turn, the first in id order, however
/// many would speak.
#[test]
fn one_person_speaks_on_a_turn_and_the_first_in_id_order() {
    let steps = when(true, &[BELL, ROWE], |thrown| {
        thrown.iter().all(Option::is_some)
    });
    let (mut engine, playthrough) = office(true, steps);
    look(&mut engine, playthrough, "look");
    let said: Vec<i64> = speech(&engine).iter().map(|act| act.1).collect();
    assert_eq!(said, [BELL], "Rowe would have spoken too, and waits");
    assert_eq!(data::speech().max_speakers.turn, 1);
}

/// Test 5: somebody who spoke takes no act on that line, and everybody else's
/// act is what it would have been had nobody spoken.
#[test]
fn a_speaker_takes_no_act_and_nobody_elses_act_moves() {
    let steps = when(true, &[BELL, ROWE], |thrown| {
        thrown[0].is_none() && thrown[1].is_some()
    });
    let (mut spoken, playthrough) = office(true, steps);
    look(&mut spoken, playthrough, "look");
    let acts = volitions(&spoken);
    let rowe: Vec<&str> = acts
        .iter()
        .filter(|act| act.1 == ROWE)
        .map(|act| act.2.as_str())
        .collect();
    assert_eq!(rowe.len(), 1, "one row for Rowe on the line: {rowe:?}");
    assert!(
        rowe[0].starts_with(volition::SPEAK),
        "and it is what Rowe said"
    );

    // The same line with Rowe still quiet from something said before it:
    // Rowe acts instead, and Bell, who acts first, acts the same.
    let (mut quiet, playthrough) = office(true, steps);
    quiet
        .store()
        .connection()
        .execute(
            "INSERT INTO playthrough_volitions (playthrough_id, character_id, location_id,
               chosen, status, fact, serves, round, decided_by, created_at, updated_at)
             SELECT ?1, ?2, location_id, 'speak:dismiss', 'applied', 'Rowe spoke up.', 'none',
               1, 'die', datetime('now'), datetime('now')
             FROM characters WHERE id = ?2",
            [playthrough, ROWE],
        )
        .unwrap();
    look(&mut quiet, playthrough, "look");
    let bell = |acts: &[Act]| -> Vec<(String, String, String)> {
        acts.iter()
            .filter(|act| act.1 == BELL)
            .map(|act| (act.2.clone(), act.3.clone(), act.4.clone()))
            .collect()
    };
    assert_eq!(bell(&acts), bell(&volitions(&quiet)));
    assert!(
        volitions(&quiet)
            .iter()
            .any(|act| act.1 == ROWE && !act.2.starts_with(volition::SPEAK)),
        "a quiet Rowe takes an act"
    );
}

/// Test 6: what was said is in the prompt of the paragraph for the line it
/// was said on, that scene tells it, and the next paragraph does not repeat it.
#[test]
fn what_is_said_is_told_by_the_paragraph_of_its_own_line() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let token = throws(&engine, playthrough, &[BELL])[0].clone().unwrap();
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let told = volition::speech_choices(
        &game,
        game.character(BELL),
        game.current_location().unwrap(),
    )
    .into_iter()
    .find(|(offered, _)| *offered == token)
    .unwrap()
    .1;
    let scene = submit(
        &mut engine,
        playthrough,
        "/examine quarter receipt",
        "look",
        vec![reply(json!({
            "purpose": "narration",
            "content": "Bell speaks.",
            "prompt_includes": [format!("What else happened here, recorded by the game: {told}")],
        }))],
    );
    let said = speech(&engine);
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].4, told);
    assert_eq!(said[0].6, scene, "the scene of its own line tells it");

    submit(
        &mut engine,
        playthrough,
        "/examine quarter receipt",
        "again",
        vec![reply(json!({
            "purpose": "narration",
            "content": "You look again.",
            "prompt_excludes": [told],
        }))],
    );
}

/// Test 6, the fallback: the engine's own words tell nobody's speech, so it
/// waits for the next paragraph, which states it.
#[test]
fn the_engines_own_words_leave_what_was_said_for_the_next_paragraph() {
    let take = |engine: &mut Engine, playthrough: i64| {
        submit(
            engine,
            playthrough,
            "/take quarter receipt",
            "take",
            vec![reply(
                json!({ "purpose": "narration", "unavailable": true }),
            )],
        );
    };
    let steps = when_spoken(false, take);
    let (mut engine, playthrough) = office(false, steps);
    take(&mut engine, playthrough);
    let said = speech(&engine);
    assert_eq!(said.len(), 1);
    assert_eq!(
        said[0].6, None,
        "the engine's own words told nobody's speech"
    );

    submit(
        &mut engine,
        playthrough,
        "/drop quarter receipt",
        "drop",
        vec![reply(json!({
            "purpose": "narration",
            "content": "You put it down.",
            "prompt_includes": [said[0].4.clone()],
        }))],
    );
    assert!(
        speech(&engine)[0].6.is_some(),
        "the paragraph that said it tells it"
    );
}

/// Test 8: a worker killed after the speech step committed never throws the
/// die again; the next delivery finishes the line with what was said.
#[test]
fn a_worker_killed_after_speaking_does_not_speak_twice() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let mut replay = Replay::new(vec![]);
    let stopped = engine.submit_stopping(
        playthrough,
        "/examine quarter receipt",
        "look",
        &mut replay,
        &mut |_| {},
        Some("speech"),
    );
    assert!(matches!(stopped, Err(Error::Stopped(step)) if step == "speech"));
    assert_eq!(speech(&engine).len(), 1);

    let scene = submit(
        &mut engine,
        playthrough,
        "/examine quarter receipt",
        "look",
        vec![narration("You look.")],
    );
    let said = speech(&engine);
    assert_eq!(said.len(), 1, "said once, not twice");
    assert_eq!(said[0].6, scene, "and told by the finished line");
}

/// Test 8, the scene a journal kept: the ids it stated are what it claims,
/// and a receipt written when a scene only said whether it stated any reads
/// `false` as a prompt that stated none.
#[test]
fn a_kept_scene_claims_the_volitions_its_prompt_stated() {
    let take = |engine: &mut Engine, playthrough: i64| {
        submit(
            engine,
            playthrough,
            "/take quarter receipt",
            "take",
            vec![reply(
                json!({ "purpose": "narration", "unavailable": true }),
            )],
        );
    };
    let steps = when_spoken(false, take);
    let (mut engine, playthrough) = office(false, steps);
    let mut replay = Replay::new(vec![reply(
        json!({ "purpose": "narration", "unavailable": true }),
    )]);
    let stopped = engine.submit_stopping(
        playthrough,
        "/take quarter receipt",
        "take",
        &mut replay,
        &mut |_| {},
        Some("narrated"),
    );
    assert!(matches!(stopped, Err(Error::Stopped(_))));
    let conn = engine.store().connection();
    let kept: String = conn
        .query_row(
            "SELECT json_extract(journal, '$.steps.narrated') FROM playthrough_commands",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let kept: Value = serde_json::from_str(&kept).unwrap();
    assert_eq!(
        kept["volitions"],
        json!([]),
        "a fallback states no volition"
    );
    conn.execute(
        "UPDATE playthrough_commands
         SET journal = json_set(journal, '$.steps.narrated.volitions', json('false'))",
        [],
    )
    .unwrap();
    let mut replay = Replay::new(vec![]);
    engine
        .submit(
            playthrough,
            "/take quarter receipt",
            "take",
            &mut replay,
            &mut |_| {},
        )
        .unwrap();
    let said = speech(&engine);
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].6, None, "`false` claimed nothing");
}

/// Test 9: the person spoken to answers in their exchange and does not
/// speak up over it; somebody else in the room may, and the exchange's
/// narrator is told it and asked to narrate it, which it is not asked when
/// nobody else spoke.
#[test]
fn the_one_you_address_does_not_interrupt() {
    let steps = when(true, &[BELL, ROWE], |thrown| {
        thrown[0].is_some() && thrown[1].is_none()
    });
    let (mut engine, playthrough) = office(true, steps);
    let mut replies = talk("Bell looks up.");
    replies[1] = reply(json!({
        "purpose": "interaction-narration",
        "content": "Bell looks up.",
        "prompt_excludes": ["also spoke up unasked"],
    }));
    submit(&mut engine, playthrough, "/talk Bell", "talk", replies);
    assert!(
        speech(&engine).iter().all(|act| act.1 != BELL),
        "Bell does not interrupt her own exchange"
    );

    let steps = when(true, &[ROWE], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(true, steps);
    let mut replies = talk("Bell looks up, and Rowe says something.");
    replies[1] = reply(json!({
        "purpose": "interaction-narration",
        "content": "Bell looks up, and Rowe says something.",
        "prompt_includes": [
            "What else happened here, recorded by the game: Rowe spoke up unasked",
            "Rowe also spoke up unasked, as \"What else happened here\" above records. \
             Narrate that too, as part of this exchange, in a sentence or two of its own. \
             Nothing Rowe said changes what is recorded above.",
        ],
    }));
    submit(&mut engine, playthrough, "/talk Bell", "talk", replies);
    let said = speech(&engine);
    assert_eq!(said.len(), 1);
    assert_eq!(said[0].1, ROWE);
}

/// Test 10: the no-model mode and a submitted line throw the same speech
/// die over the same rows, and write the same rows for it.
#[test]
fn both_modes_say_the_same() {
    let take = |engine: &mut Engine, playthrough: i64| {
        submit(
            engine,
            playthrough,
            "/take quarter receipt",
            "take",
            vec![narration("You take it.")],
        );
    };
    let steps = when_spoken(false, take);
    let (mut submitted, playthrough) = office(false, steps);
    take(&mut submitted, playthrough);
    let (mut played, playthrough) = office(false, steps);
    let outcome = played
        .play(playthrough, "/take quarter receipt", &mut |_| {})
        .unwrap();
    let rows = |engine: &Engine| -> Vec<(i64, String, String, String, String)> {
        speech(engine)
            .into_iter()
            .map(|act| (act.1, act.2, act.3, act.4, act.5))
            .collect()
    };
    assert_eq!(rows(&played), rows(&submitted));
    let said = &speech(&played)[0].4;
    assert!(
        outcome
            .report
            .note
            .contains(&format!("someone here: {said}")),
        "the no-model mode says it in its notes: {:?}",
        outcome.report.note
    );
}

/// Every act on offer to a speaker is a record's, and one of them was said.
#[test]
fn a_speaker_says_what_was_on_offer() {
    let steps = when(false, &[BELL], |thrown| thrown[0].is_some());
    let (mut engine, playthrough) = office(false, steps);
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let offered = volition::speech_choices(
        &game,
        game.character(BELL),
        game.current_location().unwrap(),
    );
    look(&mut engine, playthrough, "look");
    let said = speech(&engine);
    assert!(offered.contains(&(said[0].2.clone(), said[0].4.clone())));
}
