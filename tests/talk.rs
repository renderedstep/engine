//! Talking to somebody: what a person's answer must be before the engine
//! keeps it, what the one action they pick may change, and what a talk
//! leaves when its narration fails or its worker stops. Every test stands at
//! the gate with Maren, at peace with the party and holding the brass key.

use renderedstep_engine::dialogue;
use renderedstep_engine::engine::{Engine, Error, Submitted};
use renderedstep_engine::model::{Failure, Replay, Reply};
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::schemas;
use serde_json::{json, Value};
use std::path::Path;

const MAREN: i64 = 1_000_000_002;
const MARKET: i64 = 1_000_000_001;
const COURTYARD: i64 = 1_000_000_002;
/// The world's own brass key, which Maren holds in every game.
const KEY_TEMPLATE: i64 = 1_000_000_001;
const OWNER: i64 = 1_000_000_008;
/// The world's own silver crown, which the owner holds.
const CROWN_TEMPLATE: i64 = 1_000_000_008;
const PELL: i64 = 1_000_000_009;
/// The world's own lantern, which Pell carries.
const PELLS_LANTERN: i64 = 1_000_000_009;
/// A lantern the world puts in Maren's hands once a game has begun.
const A_NEW_LANTERN: i64 = 1_000_000_007;

/// Somebody standing in the courtyard holding a silver crown.
const AN_OWNER_IN_THE_COURTYARD: &str = "
    INSERT INTO characters (id, age, created_at, updated_at, fullname, nickname, race_id,
      story_id, sex, level, hit_die, strength, dexterity, will, location_id)
    SELECT 1000000008, 45, created_at, updated_at, 'Orrin Vale', 'Orrin', race_id, story_id,
      'male', 10, 8, 12, 12, 12, 1000000002 FROM characters WHERE id = 1000000001;
    INSERT INTO items (id, character_id, created_at, updated_at, name, properties)
    VALUES (1000000008, 1000000008, '2026-10-02', '2026-10-02', 'silver crown', '{}');";

/// A companion of the party, carrying a lantern, who stands nowhere of
/// their own.
const A_COMPANION: &str = "
    INSERT INTO characters (id, age, created_at, updated_at, fullname, nickname, is_companion,
      race_id, story_id, sex, level, hit_die, strength, dexterity, will)
    SELECT 1000000009, 32, created_at, updated_at, 'Pell', 'Pell', 1, race_id, story_id,
      'female', 10, 8, 12, 12, 12 FROM characters WHERE id = 1000000001;
    INSERT INTO items (id, character_id, created_at, updated_at, name, properties)
    VALUES (1000000009, 1000000009, '2026-10-02', '2026-10-02', 'companion lantern', '{}');";

/// The gate with Maren at peace, `sql` run over the world, and a game
/// started in it.
fn gate(sql: &str) -> (Engine, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-conversation-at-the-gate.sql"),
    )
    .expect("the gate's world");
    let mut engine = open_world(&format!(
        "{world}UPDATE characters SET hostile = 0 WHERE id = {MAREN};{sql}"
    ))
    .unwrap();
    let story = engine
        .story_titled(&format!("A Conversation at the Gate{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start(story).unwrap();
    (engine, playthrough)
}

/// A replayed reply, read as a sweep step's.
fn reply(value: Value) -> Reply {
    Reply::from_value(&value).unwrap()
}

/// A whole answer from the person talked to, picking `engine_action`.
fn reaction(engine_action: &str) -> Value {
    json!({
        "pre_thought": "They need my help.",
        "pre_feeling": "concerned",
        "action": "I give them my key.",
        "post_feeling": "hopeful",
        "post_thought": "They can open it now.",
        "inner_resolution": "I will wait here.",
        "engine_action": engine_action,
    })
}

/// The person's answer to the character call.
fn answer(content: Value) -> Reply {
    reply(json!({ "purpose": "character", "content": content }))
}

/// The exchange's narration.
fn narrated(content: &str) -> Reply {
    reply(json!({ "purpose": "interaction-narration", "content": content }))
}

/// Submits `/talk Maren` under `token`, answered by `replies`; what it
/// returned and the replay that answered it.
fn talk(
    engine: &mut Engine,
    playthrough: i64,
    token: &str,
    replies: Vec<Reply>,
) -> (Result<Submitted, Error>, Replay) {
    let mut replay = Replay::new(replies);
    let submitted = engine.submit(playthrough, "/talk Maren", token, &mut replay, &mut |_| {});
    (submitted, replay)
}

/// This game's copy of what `template` became.
fn copy_of(engine: &Engine, playthrough: i64, template: i64) -> i64 {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT id FROM items WHERE template_id = ?1 AND playthrough_id = ?2",
            [template, playthrough],
            |row| row.get(0),
        )
        .unwrap()
}

/// `(character_id, location_id)` of an item.
fn whereabouts(engine: &Engine, item: i64) -> (Option<i64>, Option<i64>) {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT character_id, location_id FROM items WHERE id = ?1",
            [item],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap()
}

/// How many rows `table` holds.
fn count(engine: &Engine, table: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .unwrap()
}

/// The names of what the player carries.
fn carrying(engine: &Engine, playthrough: i64) -> Vec<String> {
    engine
        .read(playthrough, Vec::new())
        .unwrap()
        .state
        .carrying
        .into_iter()
        .map(|thing| thing.name)
        .collect()
}

/// The names of who is in the player's room.
fn present(engine: &Engine, playthrough: i64) -> Vec<String> {
    engine
        .read(playthrough, Vec::new())
        .unwrap()
        .state
        .present
        .into_iter()
        .map(|who| who.name)
        .collect()
}

/// `(engine_action, action_status, action_fact)` of every interaction, in
/// id order.
fn interactions(engine: &Engine) -> Vec<(String, String, String)> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare("SELECT engine_action, action_status, action_fact FROM interactions ORDER BY id")
        .unwrap();
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

/// `(location_id, following)` of this game's row for somebody.
fn npc_state(engine: &Engine, character: i64) -> Option<(Option<i64>, bool)> {
    engine
        .store()
        .connection()
        .query_row(
            "SELECT location_id, following FROM playthrough_npc_states WHERE character_id = ?1",
            [character],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok()
}

/// The tokens offered to somebody in the room the player stands in.
fn offered(engine: &Engine, playthrough: i64, character: i64) -> Vec<String> {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    dialogue::choices(&game, game.character(character))
        .into_iter()
        .map(|(token, _)| token)
        .collect()
}

/// Attacks `name` until they are dead, with their last hit point all that
/// is left before each blow.
fn kill(engine: &mut Engine, playthrough: i64, name: &str, character: i64) {
    for _ in 0..20 {
        engine
            .store()
            .connection()
            .execute(
                "UPDATE playthrough_vitals SET hp_current = 1 WHERE character_id = ?1",
                [character],
            )
            .unwrap();
        engine
            .play(playthrough, &format!("attack {name}"), &mut |_| {})
            .unwrap();
        let hp: i64 = engine
            .store()
            .connection()
            .query_row(
                "SELECT hp_current FROM playthrough_vitals WHERE character_id = ?1",
                [character],
                |row| row.get(0),
            )
            .unwrap();
        if hp == 0 {
            return;
        }
    }
    panic!("{name} outlived twenty blows");
}

/// The fields `Interaction::Schema` names, each with the cap it is written
/// under.
fn schema_fields() -> Vec<(String, usize)> {
    schemas::interaction()["schema"]["properties"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(name, property)| {
            (
                name.clone(),
                property["maxLength"].as_u64().expect("a cap") as usize,
            )
        })
        .collect()
}

/// The answer is kept under exactly the names the schema gives its fields,
/// each one sanitized: an emoji dropped and the debris of a JSON envelope cut
/// off the end.
#[test]
fn a_kept_reaction_holds_every_field_the_schema_names_sanitized() {
    let (mut engine, playthrough) = gate("");
    let mut content = reaction("none");
    content["pre_feeling"] = json!("surprised, wary \u{1F30A}");
    content["post_thought"] = json!("Say something.\u{201D}}");
    let (submitted, replay) = talk(
        &mut engine,
        playthrough,
        "talk",
        vec![answer(content.clone()), narrated("Maren looks up.")],
    );
    submitted.unwrap();
    replay.finish().unwrap();

    let schema = schemas::interaction();
    let fields = schema_fields();
    let names: Vec<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
    let required: Vec<&str> = schema["schema"]["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap())
        .collect();
    assert_eq!(names, required, "every field the schema names it requires");
    let sent = &replay.sent()[0]["schema"]["schema"]["properties"];
    for (name, _) in &fields {
        assert_eq!(
            sent[name], schema["schema"]["properties"][name],
            "the character is asked for {name} as the schema has it"
        );
        let kept: String = engine
            .store()
            .connection()
            .query_row(&format!("SELECT {name} FROM interactions"), [], |row| {
                row.get(0)
            })
            .unwrap();
        let expected = match name.as_str() {
            "pre_feeling" => "surprised, wary",
            "post_thought" => "Say something.",
            _ => content[name].as_str().unwrap(),
        };
        assert_eq!(kept, expected, "{name} is kept, sanitized");
    }
}

/// An answer that leaves a field out is refused, not kept blank: the line
/// fails at the person's call and the narrator is never asked.
#[test]
fn a_reaction_with_a_field_left_out_fails_before_the_narrator_is_asked() {
    let (mut engine, playthrough) = gate("");
    let scenes = count(&engine, "scenes");
    let (submitted, replay) = talk(
        &mut engine,
        playthrough,
        "talk",
        vec![answer(
            json!({ "action": "She shrugs.", "engine_action": "none" }),
        )],
    );
    assert!(
        matches!(
            &submitted,
            Err(Error::Model(Failure::Rejected(reason))) if reason.contains("can't be blank")
        ),
        "{:?}",
        submitted.map(|_| ())
    );
    replay.finish().unwrap();
    assert_eq!(replay.calls(), ["character"]);
    assert_eq!(count(&engine, "interactions"), 0);
    assert_eq!(count(&engine, "scenes"), scenes);
}

/// Every field of the answer is held to the cap the schema writes it under:
/// a field that arrives at its cap was cut off, so the line fails before
/// the narrator is asked and before the gift applies, and one character
/// short of it is kept.
#[test]
fn every_reaction_field_at_its_schema_cap_fails_before_the_narrator_is_asked() {
    for (name, cap) in schema_fields() {
        let (mut engine, playthrough) = gate("");
        let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
        let mut content = reaction(&format!("give:{key}"));
        content[&name] = json!(format!("{:e<cap$}", "wary, guarded, hopeful for a (v"));
        let (submitted, replay) = talk(&mut engine, playthrough, "cut", vec![answer(content)]);
        assert!(
            matches!(
                &submitted,
                Err(Error::Model(Failure::Rejected(reason))) if reason.contains("cut off")
            ),
            "{name} at its cap of {cap} is unguarded"
        );
        replay.finish().unwrap();
        assert_eq!(replay.calls(), ["character"], "{name}: no narrator");
        assert_eq!(whereabouts(&engine, key), (Some(MAREN), None), "{name}");
        assert_eq!(count(&engine, "interactions"), 0, "{name}");

        let (mut engine, playthrough) = gate("");
        let mut content = reaction("none");
        content[&name] = json!("a".repeat(cap - 1));
        let (submitted, replay) = talk(
            &mut engine,
            playthrough,
            "whole",
            vec![answer(content), narrated("Maren looks up.")],
        );
        submitted.unwrap();
        replay.finish().unwrap();
        assert_eq!(count(&engine, "interactions"), 1, "{name} under its cap");
    }
}

/// A required field that is nothing but an emoji is blank once sanitized:
/// the line fails before the gift the answer picked applies, and writes no
/// scene, no interaction and no row for Maren.
#[test]
fn a_required_field_sanitized_to_blank_fails_before_its_gift_applies() {
    let (mut engine, playthrough) = gate("");
    let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
    let scenes = count(&engine, "scenes");
    let mut content = reaction(&format!("give:{key}"));
    content["pre_feeling"] = json!("\u{1F642}");
    let (submitted, replay) = talk(&mut engine, playthrough, "blank", vec![answer(content)]);
    assert!(
        matches!(
            &submitted,
            Err(Error::Model(Failure::Rejected(reason)))
                if reason == "Validation failed: pre_feeling can't be blank"
        ),
        "{:?}",
        submitted.map(|_| ())
    );
    replay.finish().unwrap();
    assert_eq!(replay.calls(), ["character"]);
    assert_eq!(count(&engine, "scenes"), scenes);
    assert_eq!(count(&engine, "interactions"), 0);
    assert_eq!(count(&engine, "playthrough_npc_states"), 0);
    assert_eq!(whereabouts(&engine, key), (Some(MAREN), None));
    assert!(carrying(&engine, playthrough).is_empty());
}

/// The narrator's words reach the front end once, whole, after they are
/// kept, however the provider streamed them.
#[test]
fn a_talk_publishes_its_kept_narration_once() {
    let (mut engine, playthrough) = gate("");
    let mut replay = Replay::new(vec![
        answer(reaction("none")),
        narrated("Maren looks up at you."),
    ]);
    let mut chunks = Vec::new();
    engine
        .submit(
            playthrough,
            "/talk Maren",
            "talk",
            &mut replay,
            &mut |chunk| chunks.push(chunk.to_string()),
        )
        .unwrap();
    replay.finish().unwrap();
    assert_eq!(chunks, ["Maren looks up at you."]);
}

/// An action the engine never offered is rejected whatever it names: a
/// thing nobody has, a thing somebody elsewhere holds, a move no person can
/// make. The narrator is told so, the interaction says so, and nothing
/// changes hands.
#[test]
fn an_action_that_was_never_offered_is_rejected_and_moves_nothing() {
    let (mut engine, playthrough) = gate(AN_OWNER_IN_THE_COURTYARD);
    for line in ["go to Courtyard", "go to Market"] {
        engine.play(playthrough, line, &mut |_| {}).unwrap();
    }
    let crown = copy_of(&engine, playthrough, CROWN_TEMPLATE);
    let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
    assert_eq!(whereabouts(&engine, crown), (Some(OWNER), None));

    let invented = [
        "give:999".to_string(),
        format!("give:{crown}"),
        "teleport".into(),
    ];
    for (at, action) in invented.iter().enumerate() {
        let (submitted, replay) = talk(
            &mut engine,
            playthrough,
            &format!("invented-{at}"),
            vec![
                answer(reaction(action)),
                reply(json!({
                    "purpose": "interaction-narration",
                    "content": "Maren has nothing like that to give you.",
                    "prompt_includes": ["The proposed action was rejected"],
                })),
            ],
        );
        submitted.unwrap();
        replay.finish().unwrap();
    }
    let kept = interactions(&engine);
    assert_eq!(
        kept.iter()
            .map(|(action, status, _)| (action.as_str(), status.as_str()))
            .collect::<Vec<_>>(),
        invented
            .iter()
            .map(|action| (action.as_str(), "rejected"))
            .collect::<Vec<_>>()
    );
    assert!(carrying(&engine, playthrough).is_empty());
    assert_eq!(whereabouts(&engine, crown), (Some(OWNER), None));
    assert_eq!(whereabouts(&engine, key), (Some(MAREN), None));
    assert_eq!(count(&engine, "playthrough_npc_states"), 0);
}

/// A narration that fails after the gift was applied leaves the gift
/// given: the line completes on the engine's own words, chained after the
/// scene before it and later in the story, and the world's key stays
/// Maren's.
#[test]
fn a_failed_narration_keeps_the_gift_and_completes_the_line() {
    let (mut engine, playthrough) = gate("");
    let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
    let opening: i64 = engine
        .store()
        .connection()
        .query_row(
            "SELECT current_scene_id FROM playthroughs WHERE id = ?1",
            [playthrough],
            |row| row.get(0),
        )
        .unwrap();
    let (submitted, replay) = talk(
        &mut engine,
        playthrough,
        "gift",
        vec![
            answer(reaction(&format!("give:{key}"))),
            reply(json!({
                "purpose": "interaction-narration",
                "failure": { "kind": "provider", "message": "narrator unavailable" },
            })),
        ],
    );
    let scene = submitted.unwrap().turned.scene.expect("a scene");
    replay.finish().unwrap();

    assert_eq!(carrying(&engine, playthrough), ["brass key"]);
    assert_eq!(interactions(&engine)[0].1, "applied");
    let (fallback, description, previous, later): (bool, String, Option<i64>, bool) = engine
        .store()
        .connection()
        .query_row(
            "SELECT s.engine_fallback, s.description, s.previous_scene_id,
                    s.story_timestamp > o.story_timestamp
             FROM scenes s, scenes o WHERE s.id = ?1 AND o.id = ?2",
            [scene, opening],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert!(fallback, "the engine's own words");
    assert_eq!(
        description,
        "You speak with Maren. Maren gave brass key to Cal; the player now carries it."
    );
    assert_eq!(previous, Some(opening));
    assert!(later, "the talk took story time");
    assert_eq!(whereabouts(&engine, KEY_TEMPLATE), (Some(MAREN), None));
}

/// A talk stopped right after the person's choice was applied is finished
/// by the next delivery with the choice it stored: the person is not asked
/// again, and the gift is given once.
#[test]
fn a_talk_stopped_after_its_effect_resumes_with_the_stored_decision() {
    let (mut engine, playthrough) = gate("");
    let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
    let items = count(&engine, "items");
    let mut replay = Replay::new(vec![answer(reaction(&format!("give:{key}")))]);
    let stopped = engine.submit_stopping(
        playthrough,
        "/talk Maren",
        "gift",
        &mut replay,
        &mut |_| {},
        Some("character_effect"),
    );
    assert!(matches!(stopped, Err(Error::Stopped(step)) if step == "character_effect"));
    replay.finish().unwrap();
    assert_eq!(carrying(&engine, playthrough), ["brass key"]);

    let (submitted, replay) = talk(
        &mut engine,
        playthrough,
        "gift",
        vec![narrated("Maren gives you the brass key.")],
    );
    submitted.unwrap();
    replay.finish().unwrap();
    assert_eq!(replay.calls(), ["interaction-narration"], "one decision");
    let kept = interactions(&engine);
    assert_eq!(kept.len(), 1);
    assert_eq!(
        (kept[0].0.as_str(), kept[0].1.as_str()),
        (format!("give:{key}").as_str(), "applied")
    );
    let action: String = engine
        .store()
        .connection()
        .query_row("SELECT action FROM interactions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(action, "I give them my key.");
    assert_eq!(carrying(&engine, playthrough), ["brass key"]);
    assert_eq!(count(&engine, "items"), items, "given, not copied");
    assert_eq!(whereabouts(&engine, KEY_TEMPLATE), (Some(MAREN), None));
}

/// A game with no player character offers no gift, and offers the rest of
/// the set with "the player" in its words.
#[test]
fn a_game_with_no_protagonist_offers_no_gift() {
    let (mut engine, playthrough) = gate("");
    engine
        .store()
        .connection()
        .execute("UPDATE playthroughs SET character_id = NULL", [])
        .unwrap();
    let (submitted, replay) = talk(
        &mut engine,
        playthrough,
        "talk",
        vec![answer(reaction("none")), narrated("Maren looks up.")],
    );
    submitted.unwrap();
    replay.finish().unwrap();
    let request = &replay.sent()[0];
    assert_eq!(
        request["schema"]["schema"]["properties"]["engine_action"]["enum"],
        json!(["none", "follow"])
    );
    assert!(request["user"]
        .as_str()
        .unwrap()
        .contains("follow: Accompany the player when they leave this room."));
}

/// Somebody who is not in the room, or who is dead, is offered nothing but
/// to say something, and a talk with them is refused and moves nothing.
#[test]
fn somebody_absent_or_dead_is_offered_nothing_and_cannot_be_talked_into_acting() {
    let (mut engine, playthrough) = gate("");
    assert_eq!(
        offered(&engine, playthrough, MAREN),
        [
            "none".to_string(),
            format!("give:{}", copy_of(&engine, playthrough, KEY_TEMPLATE)),
            "follow".into()
        ]
    );
    engine
        .play(playthrough, "go to Courtyard", &mut |_| {})
        .unwrap();
    assert_eq!(offered(&engine, playthrough, MAREN), ["none"], "absent");
    let absent = engine
        .play(playthrough, "talk to Maren", &mut |_| {})
        .unwrap();
    assert!(
        absent.report.refusal.is_some() && absent.report.change.is_none(),
        "{:?}",
        absent.report
    );
    assert_eq!(npc_state(&engine, MAREN), None);

    let (mut engine, playthrough) = gate("");
    kill(&mut engine, playthrough, "Maren", MAREN);
    assert_eq!(offered(&engine, playthrough, MAREN), ["none"], "dead");
    let dead = engine
        .play(playthrough, "talk to Maren", &mut |_| {})
        .unwrap();
    assert!(
        dead.report.refusal.is_some() && dead.report.change.is_none(),
        "{:?}",
        dead.report
    );
    assert_eq!(npc_state(&engine, MAREN), Some((Some(MARKET), false)));
}

/// A follower walks with the party holding what they held, and a thing the
/// world later puts in their hands is copied into this game once, however
/// often the room is looked at.
#[test]
fn a_follower_keeps_what_they_hold_and_a_new_possession_is_copied_once() {
    let (mut engine, playthrough) = gate("");
    let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
    engine
        .play_deciding(playthrough, "talk to Maren", "follow", &mut |_| {})
        .unwrap();
    engine
        .play(playthrough, "go to Courtyard", &mut |_| {})
        .unwrap();
    assert_eq!(present(&engine, playthrough), ["Maren"]);
    assert_eq!(whereabouts(&engine, key), (Some(MAREN), None));

    engine
        .store()
        .connection()
        .execute(
            "INSERT INTO items (id, character_id, created_at, updated_at, name, properties)
             VALUES (?1, ?2, '2026-10-02', '2026-10-02', 'lantern', '{}')",
            [A_NEW_LANTERN, MAREN],
        )
        .unwrap();
    for _ in 0..2 {
        engine.play(playthrough, "look", &mut |_| {}).unwrap();
    }
    let lanterns: i64 = engine
        .store()
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM items WHERE template_id = ?1 AND playthrough_id = ?2",
            [A_NEW_LANTERN, playthrough],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(lanterns, 1, "copied once");
    let lantern = copy_of(&engine, playthrough, A_NEW_LANTERN);
    assert_eq!(whereabouts(&engine, lantern), (Some(MAREN), None));
    assert_eq!(whereabouts(&engine, key), (Some(MAREN), None));
    assert_eq!(whereabouts(&engine, A_NEW_LANTERN), (Some(MAREN), None));
}

/// A follower killed on the way stays where they fell when the party moves
/// on, with what they held lying beside them; the world still puts them
/// where it always did.
#[test]
fn a_follower_killed_on_the_way_stays_where_they_fell() {
    let (mut engine, playthrough) = gate("");
    let key = copy_of(&engine, playthrough, KEY_TEMPLATE);
    engine
        .play_deciding(playthrough, "talk to Maren", "follow", &mut |_| {})
        .unwrap();
    engine
        .play(playthrough, "go to Courtyard", &mut |_| {})
        .unwrap();
    kill(&mut engine, playthrough, "Maren", MAREN);
    engine
        .play(playthrough, "go to Market", &mut |_| {})
        .unwrap();

    assert!(present(&engine, playthrough).is_empty());
    assert_eq!(npc_state(&engine, MAREN), Some((Some(COURTYARD), false)));
    assert_eq!(whereabouts(&engine, key), (None, Some(COURTYARD)));
    let world: i64 = engine
        .store()
        .connection()
        .query_row(
            "SELECT location_id FROM characters WHERE id = ?1",
            [MAREN],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(world, MARKET);
}

/// A companion who has never been talked to is already travelling with the
/// party, so they are offered to stay rather than to follow, and once they
/// agree they stay behind.
#[test]
fn a_companion_is_offered_to_stay_and_stays_behind() {
    let (mut engine, playthrough) = gate(A_COMPANION);
    let tokens = offered(&engine, playthrough, PELL);
    assert!(tokens.contains(&"stop_following".to_string()), "{tokens:?}");
    assert!(!tokens.contains(&"follow".to_string()), "{tokens:?}");

    let stayed = engine
        .play_deciding(playthrough, "talk to Pell", "stop_following", &mut |_| {})
        .unwrap();
    assert!(stayed.report.change.is_some(), "{:?}", stayed.report);
    engine
        .play(playthrough, "go to Courtyard", &mut |_| {})
        .unwrap();
    assert!(!present(&engine, playthrough).contains(&"Pell".to_string()));
    engine
        .play(playthrough, "go to Market", &mut |_| {})
        .unwrap();
    assert!(present(&engine, playthrough).contains(&"Pell".to_string()));
}

/// A companion killed stays where they fell when the party moves on, with
/// what they carried lying there; the world gives them no place of their
/// own.
#[test]
fn a_dead_companion_stays_where_they_fell() {
    let (mut engine, playthrough) = gate(A_COMPANION);
    let lantern = copy_of(&engine, playthrough, PELLS_LANTERN);
    kill(&mut engine, playthrough, "Pell", PELL);
    engine
        .play(playthrough, "go to Courtyard", &mut |_| {})
        .unwrap();

    assert!(!present(&engine, playthrough).contains(&"Pell".to_string()));
    assert_eq!(npc_state(&engine, PELL), Some((Some(MARKET), false)));
    assert_eq!(whereabouts(&engine, lantern), (None, Some(MARKET)));
    let world: Option<i64> = engine
        .store()
        .connection()
        .query_row(
            "SELECT location_id FROM characters WHERE id = ?1",
            [PELL],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(world, None);
}
