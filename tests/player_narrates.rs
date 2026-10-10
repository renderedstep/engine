//! A game the player narrates, walked for several turns over the sweep's
//! worlds with no model at all: every act the game picks is one of the
//! glance's targets, a line already played on a visit is not picked again
//! while anything new is on offer, a thing the arc asks the player to hold
//! is never let go of, a resumed turn plays the pick its journal kept, and
//! the player's paragraph is kept beside the scene and read by no prompt.

use renderedstep_engine::engine::{Engine, Error};
use renderedstep_engine::glance::Glance;
use renderedstep_engine::grammar::unslashed;
use renderedstep_engine::model::Replay;
use renderedstep_engine::narration;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::{Game, Mode};
use renderedstep_engine::records::{int, text};
use renderedstep_engine::turn::chooser::{self, Candidate};
use serde_json::Value;
use std::path::Path;

fn open(world: &str, title: &str, mode: Mode) -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("parity/worlds/{world}.sql")),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("{title}{TITLE_SUFFIX}"))
        .unwrap();
    let playthrough = engine.start_in(story, mode).unwrap();
    (engine, playthrough)
}

/// The ids of every thing an open arc asks the player to hold: the world's
/// own rows and every game's copies of them.
fn arc_items(engine: &Engine) -> Vec<i64> {
    let records = engine.store().load().unwrap();
    let open: Vec<i64> = records
        .select("quests", |quest| text(quest, "status") == Some("open"))
        .iter()
        .filter_map(|quest| int(quest, "id"))
        .collect();
    let held: Vec<i64> = records
        .select("quest_steps", |step| {
            text(step, "trigger_kind") == Some("hold_item")
                && int(step, "quest_id").is_some_and(|quest| open.contains(&quest))
        })
        .iter()
        .filter_map(|step| int(step, "target_id"))
        .collect();
    records
        .select("items", |item| {
            int(item, "id").is_some_and(|id| held.contains(&id))
                || int(item, "template_id").is_some_and(|template| held.contains(&template))
        })
        .iter()
        .filter_map(|item| int(item, "id"))
        .collect()
}

fn scene_row(engine: &Engine, scene: i64) -> serde_json::Map<String, Value> {
    engine
        .store()
        .load()
        .unwrap()
        .find("scenes", scene)
        .cloned()
        .expect("the scene")
}

fn listed(glance: &Glance, candidate: &Candidate) -> bool {
    glance.verbs.iter().any(|verb| {
        verb.name == candidate.verb
            && verb.available()
            && verb.targets.iter().any(|target| {
                target.name == candidate.target
                    && target.id == candidate.id
                    && target.token == candidate.token
            })
    })
}

/// What one walk saw on each turn: the line picked, and whether the game
/// ended on it.
struct Walked {
    picks: Vec<String>,
    over: bool,
}

/// Lets the game act `turns` times, checking every rule the chooser keeps
/// on every turn, and writes the player's paragraph after each.
fn walk(world: &str, title: &str, turns: usize) -> Walked {
    let (mut engine, playthrough) = open(world, title, Mode::PlayerNarrates);
    let arc = arc_items(&engine);
    let mut picks = Vec::new();
    for turn in 0..turns {
        let glance = engine.glance(playthrough).unwrap();
        assert_eq!(glance.mode, Mode::PlayerNarrates);
        if glance.over {
            return Walked { picks, over: true };
        }
        let candidates = engine.candidates(playthrough, false).unwrap();
        assert!(
            !candidates.is_empty(),
            "{world}: nothing to choose on turn {turn}"
        );
        for candidate in &candidates {
            assert!(
                listed(&glance, candidate),
                "{world}: {} is not a glance target",
                candidate.line
            );
            assert!(chooser::VERBS.contains(&candidate.verb.as_str()));
            let spends = candidate.token.as_deref().is_some_and(|token| {
                token.starts_with("use:consume:") || token.starts_with("use:burn:")
            });
            let spent = candidate
                .intent
                .physical
                .as_ref()
                .and_then(|choice| choice.item.as_ref())
                .map(|item| item.id);
            assert!(
                !(candidate.verb == "drop" && candidate.id.is_some_and(|id| arc.contains(&id))),
                "{world}: the game may drop {}, which the arc asks the player to hold",
                candidate.target
            );
            assert!(
                !(spends && spent.is_some_and(|id| arc.contains(&id))),
                "{world}: the game may spend {}, which the arc asks the player to hold",
                candidate.target
            );
            assert!(
                !candidate
                    .token
                    .as_deref()
                    .is_some_and(|token| token.starts_with("use:offer:")),
                "{world}: an offer is a conversation, and the game does not choose one"
            );
        }

        let mut replay = Replay::new(Vec::new());
        let submitted = engine
            .act(
                playthrough,
                &format!("act-{turn}"),
                false,
                &mut replay,
                &mut |_| {},
            )
            .unwrap();
        replay.finish().unwrap();
        assert!(
            replay.sent().is_empty(),
            "{world}: a chosen turn asked a model"
        );
        let scene = submitted
            .turned
            .scene
            .expect("a chosen turn writes a scene");
        let row = scene_row(&engine, scene);
        let records = engine.store().load().unwrap();
        let acted = if matches!(text(&row, "resolved_action"), Some("conclude" | "ending")) {
            records
                .find("scenes", int(&row, "previous_scene_id").unwrap())
                .cloned()
                .unwrap()
        } else {
            row.clone()
        };
        assert_eq!(text(&acted, "resolved_by"), Some(chooser::RESOLVED_BY));
        assert_eq!(acted.get("engine_fallback"), Some(&Value::Bool(false)));
        let typed = text(&acted, "typed").unwrap().to_string();
        let pick = candidates
            .iter()
            .find(|candidate| unslashed(&candidate.line) == typed)
            .unwrap_or_else(|| panic!("{world}: {typed:?} was not among the candidates"));
        if candidates.iter().any(|candidate| candidate.weight > 0) {
            assert!(
                pick.weight > 0 && !pick.repeated,
                "{world}: {typed:?} was played on this visit already, and something new was on offer"
            );
        }

        let glance = engine.glance(playthrough).unwrap();
        assert_eq!(
            glance.waiting,
            Some(scene),
            "{world}: the scene waits for the paragraph"
        );
        let card = engine
            .facts_card(playthrough, scene)
            .unwrap()
            .expect("a card");
        assert_eq!(card.scene, scene);
        assert!(
            !card.act.is_empty(),
            "{world}: the card says what the game did"
        );
        let words = format!("PLAYER-WRITTEN PARAGRAPH {turn}.");
        engine.write_paragraph(playthrough, scene, &words).unwrap();
        assert_eq!(engine.glance(playthrough).unwrap().waiting, None);
        assert!(
            !text(&scene_row(&engine, scene), "description")
                .unwrap_or_default()
                .contains("PLAYER-WRITTEN"),
            "{world}: the paragraph went into the scene"
        );

        // The next prompt is told the engine's fact for the turn, never the
        // player's paragraph.
        let records = engine.store().load().unwrap();
        let game = Game::new(&records, playthrough);
        let prompt = narration::call(game, "look around", None, None, None).user;
        assert!(
            !prompt.contains("PLAYER-WRITTEN"),
            "{world}: a prompt read the paragraph"
        );
        let current = game.current_scene().expect("a current scene");
        let told = text(current, "engine_fact")
            .filter(|fact| !fact.trim().is_empty())
            .or_else(|| text(current, "description"))
            .unwrap();
        assert!(
            prompt.contains(&format!("What just happened: {told}")),
            "{world}: the prompt was not told the engine's fact:\n{prompt}"
        );
        println!("{world} turn {turn}: {typed} (card: {card:?})");
        picks.push(typed);
    }
    let over = engine.glance(playthrough).unwrap().over;
    Walked { picks, over }
}

#[test]
fn the_game_walks_the_salt_assizes_by_the_rules() {
    let walked = walk("the-salt-assizes", "The Salt Assizes", 10);
    assert!(walked.picks.len() >= 4, "{:?}", walked.picks);
}

#[test]
fn the_game_walks_the_unrecorded_hour_by_the_rules() {
    let walked = walk("the-unrecorded-hour", "The Unrecorded Hour", 10);
    assert!(walked.picks.len() >= 4, "{:?}", walked.picks);
}

#[test]
fn a_game_on_a_keyless_install_walks_only_into_written_rooms() {
    // Every way out of the Lunar Cartographer's first room leads to a room
    // nobody has written, so with no model to write one there is nothing the
    // game may choose, and it says so in its own words.
    let (mut engine, playthrough) = open(
        "the-lunar-cartographer",
        "The Lunar Cartographer",
        Mode::PlayerNarrates,
    );
    assert!(engine.candidates(playthrough, false).unwrap().is_empty());
    let unwritten = engine.candidates(playthrough, true).unwrap();
    assert!(!unwritten.is_empty());
    assert!(unwritten.iter().all(|candidate| candidate.verb == "move"));
    let mut replay = Replay::new(Vec::new());
    let turned = engine
        .act(playthrough, "act", false, &mut replay, &mut |_| {})
        .unwrap()
        .turned;
    let refusal = turned.refusal.expect("a refusal");
    assert_eq!(refusal.kind, "unplayable");
    assert_eq!(refusal.fact, chooser::NOTHING_TO_DO);
    assert!(replay.sent().is_empty());
}

#[test]
fn the_game_walks_the_furnished_rooms_by_the_rules() {
    let walked = walk("the-furnished-rooms", "The Furnished Rooms", 8);
    assert_eq!(walked.picks.len(), 8);
    assert!(!walked.over);
}

#[test]
fn a_seeded_walk_picks_the_same_acts_every_time() {
    let first = walk("the-salt-assizes", "The Salt Assizes", 6).picks;
    let second = walk("the-salt-assizes", "The Salt Assizes", 6).picks;
    assert_eq!(first, second);
}

#[test]
fn a_protagonist_with_no_pursuit_weighs_every_act_the_same() {
    let (engine, playthrough) = open(
        "the-furnished-rooms",
        "The Furnished Rooms",
        Mode::PlayerNarrates,
    );
    let candidates = engine.candidates(playthrough, false).unwrap();
    assert!(!candidates.is_empty());
    assert!(candidates.iter().all(|candidate| candidate.pursued == 1));
}

#[test]
fn a_protagonist_who_keeps_weighs_a_look_over_a_walk() {
    let (engine, playthrough) = open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    let candidates = engine.candidates(playthrough, false).unwrap();
    let pursued = |verb: &str| {
        candidates
            .iter()
            .find(|candidate| candidate.verb == verb)
            .map(|candidate| candidate.pursued)
    };
    // Coraith Vell keeps: weights.yml gives keep wait 6 and move 1, over a
    // base of 1, and an examine weighs as a wait.
    assert_eq!(pursued("examine"), Some(7));
    assert_eq!(pursued("move"), Some(2));
}

#[test]
fn the_way_to_the_next_beat_is_boosted() {
    let (engine, playthrough) = open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    let candidates = engine.candidates(playthrough, false).unwrap();
    let boosted: Vec<&Candidate> = candidates
        .iter()
        .filter(|candidate| candidate.arc)
        .collect();
    assert!(!boosted.is_empty(), "{candidates:#?}");
    for candidate in boosted {
        assert_eq!(candidate.weight, candidate.pursued + chooser::ARC_BOOST);
    }
}

#[test]
fn a_resumed_turn_plays_the_pick_its_journal_kept() {
    let (mut straight, a) = open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    let (mut stopped, b) = open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    for (turn, stop) in [
        "chosen",
        "refusal",
        "told_tolls",
        "volition",
        "chosen",
        "arc",
    ]
    .iter()
    .enumerate()
    {
        let token = format!("act-{turn}");
        let mut replay = Replay::new(Vec::new());
        let played = straight
            .act(a, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        let mut replay = Replay::new(Vec::new());
        match stopped.act_stopping(b, &token, false, &mut replay, &mut |_| {}, Some(stop)) {
            Err(Error::Stopped(step)) => assert_eq!(&step, stop),
            other => panic!("expected the turn to stop after {stop}, got {other:?}"),
        }
        let journal = stopped
            .store()
            .connection()
            .query_row(
                "SELECT journal FROM playthrough_commands WHERE request_token = ?1",
                [&token],
                |row| row.get::<_, String>(0),
            )
            .unwrap();
        assert!(
            journal.contains("\"chosen\""),
            "the pick was kept: {journal}"
        );
        let mut replay = Replay::new(Vec::new());
        let resumed = stopped
            .act(b, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        let typed = |engine: &Engine, scene: Option<i64>| {
            text(&scene_row(engine, scene.unwrap()), "typed").map(str::to_string)
        };
        assert_eq!(
            typed(&straight, played.turned.scene),
            typed(&stopped, resumed.turned.scene),
            "turn {turn}, stopped after {stop}"
        );
        assert_eq!(
            played.state, resumed.state,
            "turn {turn}, stopped after {stop}"
        );
        // A second delivery of a finished turn plays nothing again.
        let mut replay = Replay::new(Vec::new());
        let again = stopped
            .act(b, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        assert_eq!(again.turned.scene, resumed.turned.scene);
        if played.state.dead {
            break;
        }
    }
}

#[test]
fn a_narrated_game_does_not_let_the_game_act() {
    let (mut engine, playthrough) = open("the-salt-assizes", "The Salt Assizes", Mode::Narrated);
    let glance = engine.glance(playthrough).unwrap();
    assert_eq!(glance.mode, Mode::Narrated);
    assert_eq!(glance.waiting, None);
    let mut replay = Replay::new(Vec::new());
    let result = engine.act(playthrough, "act", false, &mut replay, &mut |_| {});
    assert_eq!(
        result.err(),
        Some(Error::WrongMode {
            playthrough,
            mode: "narrated".into()
        })
    );
    let commands: i64 = engine
        .store()
        .connection()
        .query_row("SELECT COUNT(*) FROM playthrough_commands", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(commands, 0);
}

#[test]
fn a_paragraph_is_written_only_for_a_scene_the_game_chose() {
    let (mut engine, playthrough) =
        open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    let opening = int(
        engine
            .store()
            .load()
            .unwrap()
            .find("playthroughs", playthrough)
            .unwrap(),
        "current_scene_id",
    )
    .unwrap();
    assert_eq!(
        engine.write_paragraph(playthrough, opening, "Words."),
        Err(Error::NotChosen {
            playthrough,
            scene: opening
        })
    );
    assert_eq!(engine.facts_card(playthrough, opening).unwrap(), None);
    let mut replay = Replay::new(Vec::new());
    let scene = engine
        .act(playthrough, "act", false, &mut replay, &mut |_| {})
        .unwrap()
        .turned
        .scene
        .unwrap();
    let first = engine
        .write_paragraph(playthrough, scene, "First.")
        .unwrap();
    let second = engine
        .write_paragraph(playthrough, scene, "Second.")
        .unwrap();
    assert_eq!(first, second, "a second paragraph replaces the first");
    let (author, words): (String, String) = engine
        .store()
        .connection()
        .query_row(
            "SELECT author, text FROM playthrough_paragraphs WHERE scene_id = ?1",
            [scene],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((author.as_str(), words.as_str()), ("player", "Second."));
}

#[test]
fn the_line_that_lets_the_game_act_is_a_typed_line_in_a_narrated_game() {
    let (mut engine, playthrough) = open("the-salt-assizes", "The Salt Assizes", Mode::Narrated);
    let outcome = engine
        .play(playthrough, chooser::LINE, &mut |_| {})
        .unwrap();
    assert_ne!(
        outcome.report.resolved_by.as_deref(),
        Some(chooser::RESOLVED_BY)
    );
}
