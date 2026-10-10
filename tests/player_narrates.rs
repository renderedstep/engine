//! A game the player narrates, walked for several turns over the sweep's
//! worlds with no model at all: every act the game picks is one of the
//! glance's targets, a line already played on a visit is not picked again
//! while anything new is on offer, a thing the arc asks the player to hold
//! is never let go of, a resumed turn plays the pick its journal kept, and
//! the player's paragraph is kept beside the scene and read by no prompt.
//!
//! The room is revealed in tiers ([`noticed`]): every turn checks that the
//! panels, the closed sets and the picks hold only what the game has noticed,
//! that what was noticed stays noticed, that each turn noticed what its tier
//! says (an arrival every fixed piece and a rolled few, a look everything in
//! plain sight, a turn in the room one more), that the card names what the
//! turn noticed, and that the doctor finds nothing out of line. A walk back
//! into a room the game left keeps what it noticed there as it left, and the
//! card says what has changed among it ([`revisit`]).

use renderedstep_engine::data;
use renderedstep_engine::engine::{Engine, Error, Submitted};
use renderedstep_engine::glance::Glance;
use renderedstep_engine::grammar::unslashed;
use renderedstep_engine::model::Replay;
use renderedstep_engine::narrates::{Consent, WouldAsk, WOULD_ASK};
use renderedstep_engine::narration;
use renderedstep_engine::noticed;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::{Game, Mode};
use renderedstep_engine::prompt_version;
use renderedstep_engine::records::{flag, int, text, Records, Row};
use renderedstep_engine::revisit;
use renderedstep_engine::turn::chooser::{self, Candidate};
use serde_json::Value;
use std::collections::HashMap;
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

/// This game's things, by id, and when each was noticed.
fn stamps(engine: &Engine, playthrough: i64) -> Vec<(i64, Option<i64>)> {
    let records = engine.store().load().unwrap();
    Game::new(&records, playthrough)
        .own("items")
        .iter()
        .map(|item| (int(item, "id").unwrap(), int(item, "noticed_at")))
        .collect()
}

fn noticed_ids(engine: &Engine, playthrough: i64) -> Vec<i64> {
    stamps(engine, playthrough)
        .into_iter()
        .filter_map(|(item, at)| at.map(|_| item))
        .collect()
}

fn story_of(engine: &Engine, playthrough: i64) -> i64 {
    let records = engine.store().load().unwrap();
    Game::new(&records, playthrough).story_id()
}

/// The panels, the read-out and every verb's targets hold only what the game
/// has noticed or carries, and the count of what a look would show is the
/// records' own.
fn shows_only_what_is_noticed(world: &str, engine: &Engine, playthrough: i64, glance: &Glance) {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let noticed = noticed_ids(engine, playthrough);
    let carried: Vec<i64> = game
        .carried()
        .iter()
        .map(|item| int(item, "id").unwrap())
        .collect();
    let items: Vec<i64> = game
        .own("items")
        .iter()
        .map(|item| int(item, "id").unwrap())
        .collect();
    let shown = |item: i64| noticed.contains(&item) || carried.contains(&item);
    for thing in &glance.lying_here {
        assert!(
            noticed.contains(&thing.id),
            "{world}: {} is listed unnoticed",
            thing.name
        );
        assert!(thing.noticed_at.is_some());
    }
    for fixture in &glance.fixtures {
        assert!(
            noticed.contains(&fixture.id),
            "{world}: {} is listed unnoticed",
            fixture.name
        );
    }
    for named in &glance.state.here {
        assert!(
            noticed.contains(&named.id),
            "{world}: the read-out lists {}",
            named.name
        );
    }
    for verb in &glance.verbs {
        for target in verb.targets.iter().chain(verb.aims.iter().flatten()) {
            if let Some(item) = target.id.filter(|id| items.contains(id)) {
                assert!(
                    shown(item),
                    "{world}: {} offers {}, unnoticed",
                    verb.name,
                    target.name
                );
            }
        }
    }
    let here = game.current_location();
    assert_eq!(
        glance.counts.unnoticed,
        noticed::unnoticed_in(&game, here).len()
    );
    let looks = glance
        .verbs
        .iter()
        .flat_map(|verb| verb.targets.iter())
        .any(|target| target.kind.as_deref() == Some(noticed::LOOK));
    assert_eq!(
        looks,
        noticed::worth_a_look(&game, here),
        "{world}: the look is offered wrongly"
    );
}

/// What one walk saw on each turn: the line picked, whether the game ended
/// on it, and how many of its moves walked back into a room it had left.
struct Walked {
    picks: Vec<String>,
    over: bool,
    returns: usize,
}

/// The room the game stands in, the things lying there it has noticed and
/// the people there, by id, read off the records as the test reads them.
fn standing(engine: &Engine, playthrough: i64) -> (i64, Vec<i64>, Vec<i64>) {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    let room = game.current_location().expect("a room");
    let things = game
        .items_noticed_in(Some(room))
        .iter()
        .map(|item| int(item, "id").unwrap())
        .collect();
    let walking: Vec<i64> = game
        .followers()
        .iter()
        .map(|who| int(who, "id").unwrap())
        .collect();
    let mut people: Vec<i64> = game
        .characters_located_in(room)
        .into_iter()
        .filter(|who| !flag(who, "is_protagonist"))
        .map(|who| int(who, "id").unwrap())
        .filter(|who| !walking.contains(who))
        .collect();
    people.sort();
    (int(room, "id").unwrap(), things, people)
}

/// The card's changes on a turn: on a walk back into a room the game left,
/// what the move kept, compared with what the test saw there as it left,
/// and nothing on any other turn.
fn changed_since_it_left(
    world: &str,
    records: &Records,
    playthrough: i64,
    scene: i64,
    left: Option<&(i64, Vec<i64>, Vec<i64>)>,
    card: &[String],
) -> bool {
    let since = revisit::since_on(records, playthrough, scene);
    let Some((room, things, people)) = left else {
        assert_eq!(since, None, "{world}: a change on a room never left");
        assert!(card.is_empty(), "{world}: the card says {card:?}");
        return false;
    };
    let since = since.unwrap_or_else(|| panic!("{world}: a return kept no record"));
    assert_eq!(since.seen.room, *room);
    let seen: Vec<i64> = since.seen.things.iter().map(|thing| thing.id).collect();
    assert_eq!(&seen, things, "{world}: the record is not what was noticed");
    let met: Vec<i64> = since.seen.people.iter().map(|who| who.id).collect();
    assert_eq!(&met, people, "{world}: the record is not who was there");
    for change in &since.changes {
        if change.subject.0 == "items" {
            assert!(
                things.contains(&change.subject.1),
                "{world}: {} is about a thing the game never noticed",
                change.fact
            );
        }
    }
    assert!(!card.is_empty());
    assert_eq!(card, since.facts().as_slice());
    true
}

/// Lets the game act `turns` times, checking every rule the chooser keeps
/// on every turn, and writes the player's paragraph after each.
fn walk(world: &str, title: &str, turns: usize) -> Walked {
    let (mut engine, playthrough) = open(world, title, Mode::PlayerNarrates);
    let arc = arc_items(&engine);
    let story = story_of(&engine, playthrough);
    let mut picks = Vec::new();
    let mut left: HashMap<i64, (i64, Vec<i64>, Vec<i64>)> = HashMap::new();
    let mut returns = 0;
    for turn in 0..turns {
        let glance = engine.glance(playthrough).unwrap();
        assert_eq!(glance.mode, Mode::PlayerNarrates);
        shows_only_what_is_noticed(world, &engine, playthrough, &glance);
        assert_eq!(
            engine.noticed_findings(story).unwrap(),
            Vec::new(),
            "{world}: the doctor finds a noticed record out of line before turn {turn}"
        );
        if glance.over {
            return Walked {
                picks,
                over: true,
                returns,
            };
        }
        let before = noticed_ids(&engine, playthrough);
        let stood = standing(&engine, playthrough);
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

        let after = noticed_ids(&engine, playthrough);
        assert!(
            before.iter().all(|item| after.contains(item)),
            "{world}: something noticed was forgotten on turn {turn}"
        );
        let records = engine.store().load().unwrap();
        let newly: Vec<i64> = after
            .iter()
            .copied()
            .filter(|item| !before.contains(item))
            .filter(|item| {
                !Game::new(&records, playthrough)
                    .carried()
                    .iter()
                    .any(|held| int(held, "id") == Some(*item))
            })
            .collect();
        let tier = if pick.verb == "move" {
            Tier::Arrival
        } else if pick.line == noticed::LOOK_LINE {
            Tier::Look
        } else {
            Tier::Time
        };
        noticed_by_its_tier(
            world,
            &records,
            playthrough,
            tier,
            &newly,
            glance.counts.unnoticed,
        );

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
        let mut named = card.noticed.clone();
        named.sort();
        let mut expected = noticed::names(&records, &newly);
        expected.sort();
        if !matches!(text(&row, "resolved_action"), Some("conclude" | "ending")) {
            assert_eq!(
                named, expected,
                "{world}: the card names what the turn noticed"
            );
        }
        let now = standing(&engine, playthrough).0;
        let moved = now != stood.0;
        if changed_since_it_left(
            world,
            &records,
            playthrough,
            scene,
            left.get(&now).filter(|_| moved),
            &card.changed,
        ) {
            returns += 1;
        }
        if moved {
            left.insert(stood.0, stood);
        }
        let words = format!("PLAYER-WRITTEN PARAGRAPH {turn}.");
        engine.write_paragraph(playthrough, scene, &words).unwrap();
        assert_eq!(engine.glance(playthrough).unwrap().waiting, None);
        captured_beside_the_paragraph(
            world,
            &engine,
            playthrough,
            scene,
            &typed,
            pick.verb == "move",
            matches!(text(&row, "resolved_action"), Some("conclude" | "ending")),
        );
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
    Walked {
        picks,
        over,
        returns,
    }
}

/// The paragraph's row carries every request a narrator would have been
/// sent for the turn, built and never sent, and the prompt digest of the
/// act's: the arrival writer's on a walk into a room, the narrator's told the
/// engine's act as the line typed otherwise, and the ending's after it where
/// the turn closed the story.
fn captured_beside_the_paragraph(
    world: &str,
    engine: &Engine,
    playthrough: i64,
    scene: i64,
    typed: &str,
    moved: bool,
    ended: bool,
) {
    let (requests, digest, consent): (Option<String>, Option<String>, String) = engine
        .store()
        .connection()
        .query_row(
            "SELECT requests, prompt_digest, consent FROM playthrough_paragraphs WHERE scene_id = ?1",
            [scene],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(
        consent, "none",
        "{world}: a paragraph is kept for the game alone"
    );
    let requests: Value = serde_json::from_str(
        &requests.unwrap_or_else(|| panic!("{world}: the paragraph carries no request")),
    )
    .unwrap();
    let requests = requests.as_array().unwrap();
    let digest = digest.unwrap_or_else(|| panic!("{world}: the paragraph carries no digest"));
    let would = engine.would_ask(playthrough, scene).unwrap();
    assert_eq!(
        requests,
        &would.iter().map(WouldAsk::to_json).collect::<Vec<_>>()
    );
    let purposes: Vec<&str> = would.iter().map(|asked| asked.purpose.as_str()).collect();
    let act = if moved { "arrival" } else { "narration" };
    let expected: &[&str] = if ended { &[act, "ending"] } else { &[act] };
    assert_eq!(purposes, expected, "{world}: {typed:?}");
    assert_eq!(Some(digest.clone()), would[0].prompt_digest());
    let system = would[0].request["system"].as_str().unwrap();
    let user = would[0].request["user"].as_str().unwrap();
    if moved {
        assert_eq!(Some(digest), prompt_version::of(system));
        assert!(would[0].request["schema"].is_object());
    } else {
        assert_eq!(system, data::narrator_instructions());
        assert_eq!(
            Some(digest),
            prompt_version::digest(prompt_version::NARRATION, Some(system))
        );
        assert!(
            user.contains(&format!("The player types: {typed}")),
            "{world}: the request is not told the engine's act:\n{user}"
        );
    }
    assert!(
        !user.contains("PLAYER-WRITTEN"),
        "{world}: a request read the paragraph"
    );
    if ended {
        assert_eq!(
            would[1].request["system"].as_str(),
            Some(data::ending_instructions())
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tier {
    Arrival,
    Look,
    Time,
}

/// What a turn noticed is what its tier holds.
fn noticed_by_its_tier(
    world: &str,
    records: &Records,
    playthrough: i64,
    tier: Tier,
    newly: &[i64],
    unnoticed_before: usize,
) {
    let game = Game::new(records, playthrough);
    let here = game.current_location();
    let row = |item: &i64| records.find("items", *item).unwrap();
    let fixed = |item: &Row| text(item, "tier") == Some("fixture");
    match tier {
        Tier::Arrival => {
            for item in game.items_lying_in(here) {
                if fixed(item) && !noticed::concealed(item) {
                    assert!(
                        noticed::stamped(item),
                        "{world}: {} is fixed here and was not noticed on the way in",
                        text(item, "name").unwrap_or_default()
                    );
                }
            }
            let loose = newly.iter().filter(|item| !fixed(row(item))).count();
            assert!(
                loose as i64 <= noticed::ON_ARRIVAL,
                "{world}: {loose} loose things were noticed on the way in"
            );
            let lying: Vec<i64> = game
                .items_lying_in(here)
                .iter()
                .map(|item| int(item, "id").unwrap())
                .collect();
            assert!(newly
                .iter()
                .all(|item| lying.contains(item) || !fixed(row(item))));
        }
        Tier::Look => {
            assert!(
                noticed::unnoticed_in(&game, here).is_empty()
                    || noticed::room_left(&game, here) == 0,
                "{world}: a look left something in plain sight unnoticed"
            );
        }
        Tier::Time => {
            assert!(
                newly.len() <= 1,
                "{world}: a turn in the room noticed {newly:?}"
            );
            assert!(newly.iter().all(|item| !noticed::concealed(row(item))));
            if unnoticed_before > 0 && noticed::room_left(&game, here) > 0 {
                assert_eq!(
                    newly.len(),
                    1,
                    "{world}: a turn in the room noticed nothing"
                );
            }
        }
    }
    assert!(
        newly.iter().all(|item| !noticed::concealed(row(item))),
        "{world}: something inside a fixture was noticed with no search"
    );
}

#[test]
fn the_game_walks_the_salt_assizes_by_the_rules() {
    let walked = walk("the-salt-assizes", "The Salt Assizes", 10);
    assert!(walked.picks.len() >= 4, "{:?}", walked.picks);
    assert!(walked.returns >= 1, "{:?}", walked.picks);
}

#[test]
fn the_game_walks_the_unrecorded_hour_by_the_rules() {
    let walked = walk("the-unrecorded-hour", "The Unrecorded Hour", 10);
    assert!(walked.picks.len() >= 4, "{:?}", walked.picks);
    assert!(walked.returns >= 1, "{:?}", walked.picks);
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
    assert!(walked.returns >= 1, "{:?}", walked.picks);
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
fn a_paragraph_keeps_what_the_player_let_it_be_used_for() {
    let (mut engine, playthrough) =
        open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    let mut replay = Replay::new(Vec::new());
    let scene = engine
        .act(playthrough, "act", false, &mut replay, &mut |_| {})
        .unwrap()
        .turned
        .scene
        .unwrap();
    let consent = |engine: &Engine| -> String {
        engine
            .store()
            .connection()
            .query_row(
                "SELECT consent FROM playthrough_paragraphs WHERE scene_id = ?1",
                [scene],
                |row| row.get(0),
            )
            .unwrap()
    };
    engine
        .write_paragraph_with_consent(playthrough, scene, "Mine.", Consent::Owner)
        .unwrap();
    assert_eq!(consent(&engine), "owner");
    engine
        .write_paragraph(playthrough, scene, "Mine, again.")
        .unwrap();
    assert_eq!(consent(&engine), "owner", "a rewrite keeps what was let");
    engine
        .write_paragraph_with_consent(playthrough, scene, "Mine.", Consent::None)
        .unwrap();
    assert_eq!(consent(&engine), "none");
    assert_eq!(Consent::parse("owner"), Some(Consent::Owner));
    assert_eq!(Consent::parse("everyone"), None);
}

#[test]
fn a_resumed_turn_keeps_the_request_the_uninterrupted_turn_built() {
    let (mut straight, a) = open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    let (mut stopped, b) = open("the-salt-assizes", "The Salt Assizes", Mode::PlayerNarrates);
    for turn in 0..4 {
        let token = format!("act-{turn}");
        let mut replay = Replay::new(Vec::new());
        let played = straight
            .act(a, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        let mut replay = Replay::new(Vec::new());
        match stopped.act_stopping(b, &token, false, &mut replay, &mut |_| {}, Some(WOULD_ASK)) {
            Err(Error::Stopped(step)) => assert_eq!(step, WOULD_ASK),
            other => panic!("expected the turn to stop after {WOULD_ASK}, got {other:?}"),
        }
        let mut replay = Replay::new(Vec::new());
        let resumed = stopped
            .act(b, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        assert!(replay.sent().is_empty());
        let would = |engine: &Engine, game: i64, scene: Option<i64>| {
            engine.would_ask(game, scene.unwrap()).unwrap()
        };
        let kept = would(&straight, a, played.turned.scene);
        assert!(!kept.is_empty(), "turn {turn} built no request");
        assert_eq!(
            kept,
            would(&stopped, b, resumed.turned.scene),
            "turn {turn}"
        );
        if played.state.dead || straight.glance(a).unwrap().over {
            break;
        }
    }
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

/// The Furnished Rooms with `loose` more things lying in plain sight in its
/// first room, the Clerk's Study, and a folded note shut in its desk, then
/// a game started in `mode`.
fn furnished(loose: &[(&str, &str)], mode: Mode) -> (Engine, i64) {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-furnished-rooms.sql"),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("The Furnished Rooms{TITLE_SUFFIX}"))
        .unwrap();
    let row = |name: &str, bulk: &str, within: Option<(i64, &str)>| {
        let mut values = vec![
            ("name", Value::from(name)),
            ("bulk", Value::from(bulk)),
            ("location_id", Value::from(STUDY)),
            ("description", Value::from(format!("A {name}."))),
        ];
        if let Some((fixture, how)) = within {
            values.push(("within_id", Value::from(fixture)));
            values.push(("how", Value::from(how)));
        }
        engine.store().insert("items", &values).unwrap();
    };
    for (name, bulk) in loose {
        row(name, bulk, None);
    }
    row("folded note", "light", Some((DESK, "in")));
    let playthrough = engine.start_in(story, mode).unwrap();
    (engine, playthrough)
}

const STUDY: i64 = 1_000_000_001;
const DESK: i64 = 1_000_000_001;

const LOOSE: [(&str, &str); 6] = [
    ("brass button", "light"),
    ("iron kettle", "heavy"),
    ("ledger", "handy"),
    ("coil of string", "light"),
    ("heavy bookend", "heavy"),
    ("pewter mug", "handy"),
];

/// This game's copies lying in the room it stands in, each with whether it
/// is noticed.
fn lying_here(engine: &Engine, playthrough: i64) -> Vec<Row> {
    let records = engine.store().load().unwrap();
    let game = Game::new(&records, playthrough);
    game.items_lying_in(game.current_location())
        .into_iter()
        .cloned()
        .collect()
}

fn named<'r>(rows: &'r [Row], name: &str) -> &'r Row {
    rows.iter()
        .find(|row| text(row, "name") == Some(name))
        .unwrap_or_else(|| panic!("no {name} here"))
}

fn submit(engine: &mut Engine, playthrough: i64, line: &str, token: &str) -> Submitted {
    let mut replay = Replay::new(Vec::new());
    let submitted = engine
        .submit(playthrough, line, token, &mut replay, &mut |_| {})
        .unwrap();
    replay.finish().unwrap();
    assert!(replay.sent().is_empty(), "{line:?} asked a model");
    submitted
}

#[test]
fn arriving_notices_every_fixed_piece_and_a_rolled_few_heaviest_first() {
    let (engine, playthrough) = furnished(&LOOSE, Mode::PlayerNarrates);
    let here = lying_here(&engine, playthrough);
    let fixed = |row: &&Row| text(row, "tier") == Some("fixture");
    assert!(here.iter().filter(fixed).all(noticed::stamped));
    let loose: Vec<&Row> = here
        .iter()
        .filter(|row| !fixed(row) && !noticed::concealed(row))
        .collect();
    let (seen, unseen): (Vec<&Row>, Vec<&Row>) =
        loose.iter().partition(|row| noticed::stamped(row));
    assert!(
        (1..=noticed::ON_ARRIVAL as usize).contains(&seen.len()),
        "{} loose things were noticed on the way in",
        seen.len()
    );
    let rank = |row: &Row| noticed::bulk_rank(text(row, "bulk").unwrap_or("handy"));
    let lightest_seen = seen.iter().map(|row| rank(row)).min().unwrap();
    assert!(
        unseen.iter().all(|row| rank(row) <= lightest_seen),
        "a lighter thing was noticed before a heavier one"
    );
    for hidden in ["folded note", "lump of coal"] {
        assert!(
            !noticed::stamped(named(&here, hidden)),
            "{hidden} is shut away"
        );
    }
    let glance = engine.glance(playthrough).unwrap();
    shows_only_what_is_noticed("the-furnished-rooms", &engine, playthrough, &glance);
    assert_eq!(glance.counts.unnoticed, unseen.len());
    assert!(glance
        .verbs
        .iter()
        .find(|verb| verb.name == "examine")
        .unwrap()
        .targets
        .iter()
        .any(|target| target.kind.as_deref() == Some(noticed::LOOK)
            && target.line.as_deref() == Some(noticed::LOOK_LINE)));
}

#[test]
fn a_turn_in_the_room_notices_one_more_and_a_look_shows_the_rest() {
    let (mut engine, playthrough) = furnished(&LOOSE, Mode::PlayerNarrates);
    let mut unnoticed = engine.glance(playthrough).unwrap().counts.unnoticed;
    assert!(unnoticed >= 2);
    for turn in 0..2 {
        let before = noticed_ids(&engine, playthrough);
        let played = submit(
            &mut engine,
            playthrough,
            "/inspect desk",
            &format!("t{turn}"),
        );
        assert!(played.turned.refusal.is_none());
        let after = noticed_ids(&engine, playthrough);
        assert_eq!(
            after.len(),
            before.len() + 1,
            "turn {turn} noticed one more"
        );
        let glance = engine.glance(playthrough).unwrap();
        assert_eq!(glance.counts.unnoticed, unnoticed - 1);
        unnoticed = glance.counts.unnoticed;
    }
    let before = noticed_ids(&engine, playthrough);
    let looked = submit(&mut engine, playthrough, noticed::LOOK_LINE, "look");
    let scene = scene_row(&engine, looked.turned.scene.expect("a look writes a scene"));
    let words = text(&scene, "description").unwrap().to_string();
    assert!(
        words.starts_with("You look around The Clerk's Study. You notice: "),
        "{words}"
    );
    let records = engine.store().load().unwrap();
    let seen: Vec<i64> = noticed_ids(&engine, playthrough)
        .into_iter()
        .filter(|item| !before.contains(item))
        .collect();
    assert_eq!(seen.len(), unnoticed);
    assert_eq!(
        words,
        noticed::look_words("The Clerk's Study", &noticed::names(&records, &seen))
    );
    let glance = engine.glance(playthrough).unwrap();
    assert_eq!(glance.counts.unnoticed, 0);
    assert!(glance
        .verbs
        .iter()
        .flat_map(|verb| verb.targets.iter())
        .all(|target| target.kind.as_deref() != Some(noticed::LOOK)));
    let here = lying_here(&engine, playthrough);
    for hidden in ["folded note", "lump of coal"] {
        assert!(
            !noticed::stamped(named(&here, hidden)),
            "a look found {hidden}"
        );
    }
    // Nothing is left to notice, so a turn in the room notices nothing.
    let before = noticed_ids(&engine, playthrough);
    submit(&mut engine, playthrough, "/inspect desk", "after");
    assert_eq!(noticed_ids(&engine, playthrough), before);
}

#[test]
fn a_line_naming_an_unnoticed_thing_is_refused_until_a_look() {
    let (mut engine, playthrough) = furnished(&LOOSE, Mode::PlayerNarrates);
    let here = lying_here(&engine, playthrough);
    let unseen = here
        .iter()
        .find(|row| {
            text(row, "tier") != Some("fixture")
                && !noticed::stamped(row)
                && !noticed::concealed(row)
        })
        .map(|row| text(row, "name").unwrap().to_string())
        .expect("something unnoticed in plain sight");
    let before = stamps(&engine, playthrough);
    let refused = submit(&mut engine, playthrough, &format!("/take {unseen}"), "take");
    let refusal = refused.turned.refusal.expect("a refusal");
    assert_eq!(refusal.kind, "unresolved");
    assert_eq!(refusal.fact, noticed::UNNOTICED);
    let offer = refusal.offer.expect("what is noticed");
    assert!(offer.starts_with("You have noticed: "), "{offer}");
    assert!(
        offer.contains("desk") && !offer.contains(&unseen),
        "{offer}"
    );
    assert_eq!(
        stamps(&engine, playthrough),
        before,
        "a refused line noticed something"
    );
    assert!(engine
        .glance(playthrough)
        .unwrap()
        .carrying
        .iter()
        .all(|thing| thing.name != unseen));

    // The console reads the line the same way.
    let console = engine
        .play(playthrough, &format!("take {unseen}"), &mut |_| {})
        .unwrap();
    assert!(
        console
            .report
            .refusal
            .as_deref()
            .is_some_and(|said| said.starts_with(noticed::UNNOTICED)),
        "{:?}",
        console.report
    );

    submit(&mut engine, playthrough, "look around", "look");
    let taken = submit(
        &mut engine,
        playthrough,
        &format!("/take {unseen}"),
        "take again",
    );
    assert!(taken.turned.refusal.is_none(), "{:?}", taken.turned.refusal);
    assert!(engine
        .glance(playthrough)
        .unwrap()
        .carrying
        .iter()
        .any(|thing| thing.name == unseen));

    // What lies shut away is not found by a look.
    let shut = submit(&mut engine, playthrough, "/take lump of coal", "coal");
    assert_eq!(
        shut.turned.refusal.expect("a refusal").fact,
        noticed::UNNOTICED
    );
}

#[test]
fn a_look_stops_at_what_one_room_may_show() {
    let names: Vec<String> = (0..30).map(|n| format!("pebble number {n}")).collect();
    let loose: Vec<(&str, &str)> = names.iter().map(|name| (name.as_str(), "light")).collect();
    let (mut engine, playthrough) = furnished(&loose, Mode::PlayerNarrates);
    submit(&mut engine, playthrough, "/look around", "look");
    let shown = lying_here(&engine, playthrough)
        .iter()
        .filter(|row| noticed::stamped(row))
        .count();
    assert_eq!(shown, noticed::MAX_VISIBLE_PER_ROOM);
    let glance = engine.glance(playthrough).unwrap();
    assert!(glance.counts.unnoticed > 0);
    assert_eq!(
        glance.fixtures.len() + glance.lying_here.len(),
        noticed::MAX_VISIBLE_PER_ROOM
    );
    assert!(glance
        .verbs
        .iter()
        .flat_map(|verb| verb.targets.iter())
        .all(|target| target.kind.as_deref() != Some(noticed::LOOK)));
    let before = noticed_ids(&engine, playthrough);
    submit(&mut engine, playthrough, "/inspect desk", "after");
    assert_eq!(noticed_ids(&engine, playthrough), before);
}

#[test]
fn a_narrated_game_notices_nothing_and_shows_everything() {
    let (mut engine, playthrough) = furnished(&LOOSE, Mode::Narrated);
    for line in [
        "inspect desk",
        "take ledger",
        "go The Beech Clearing",
        "go The Clerk's Study",
    ] {
        engine.play(playthrough, line, &mut |_| {}).unwrap();
    }
    assert!(stamps(&engine, playthrough)
        .iter()
        .all(|(_, at)| at.is_none()));
    let glance = engine.glance(playthrough).unwrap();
    assert_eq!(glance.counts.unnoticed, 0);
    let lying = lying_here(&engine, playthrough);
    assert_eq!(glance.fixtures.len() + glance.lying_here.len(), lying.len());
    assert!(glance
        .verbs
        .iter()
        .flat_map(|verb| verb.targets.iter())
        .all(|target| target.kind.as_deref() != Some(noticed::LOOK)));
    let records = engine.store().load().unwrap();
    assert_eq!(
        noticed::refusal_for_line(&records, playthrough, "take lump of coal"),
        None
    );
}

#[test]
fn the_doctor_finds_a_noticed_record_out_of_line() {
    let (engine, playthrough) = furnished(&LOOSE, Mode::PlayerNarrates);
    let story = story_of(&engine, playthrough);
    assert_eq!(engine.noticed_findings(story).unwrap(), Vec::new());
    let conn = engine.store().connection();
    // The world's own desk, stamped.
    conn.execute(
        "UPDATE items SET noticed_at = '2026-01-01 00:00:00' WHERE id = ?1",
        [DESK],
    )
    .unwrap();
    // This game's copy of the windowsill, its stamp lost.
    let sill: i64 = conn
        .query_row(
            "SELECT id FROM items WHERE playthrough_id = ?1 AND name = 'windowsill'",
            [playthrough],
            |row| row.get(0),
        )
        .unwrap();
    conn.execute("UPDATE items SET noticed_at = NULL WHERE id = ?1", [sill])
        .unwrap();
    let found = engine.noticed_findings(story).unwrap();
    let subjects: Vec<(&str, i64)> = found.iter().map(|finding| finding.subject).collect();
    assert!(found.iter().all(|finding| finding.code == noticed::FINDING));
    assert!(subjects.contains(&("items", DESK)), "{found:#?}");
    assert!(subjects.contains(&("items", sill)), "{found:#?}");
    assert_eq!(found.len(), 2, "{found:#?}");

    // A stamp in a narrated game is out of line too.
    conn.execute(
        "UPDATE playthroughs SET mode = 'narrated' WHERE id = ?1",
        [playthrough],
    )
    .unwrap();
    let found = engine.noticed_findings(story).unwrap();
    assert!(
        found
            .iter()
            .any(|finding| finding.message.contains("which is narrated")),
        "{found:#?}"
    );
}

#[test]
fn a_resumed_turn_notices_what_the_uninterrupted_turn_did() {
    let (mut straight, a) = furnished(&LOOSE, Mode::PlayerNarrates);
    let (mut stopped, b) = furnished(&LOOSE, Mode::PlayerNarrates);
    let mut steps_stopped = Vec::new();
    for turn in 0..8 {
        let token = format!("act-{turn}");
        let mut replay = Replay::new(Vec::new());
        straight
            .act(a, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        let journal = straight
            .store()
            .connection()
            .query_row(
                "SELECT journal FROM playthrough_commands WHERE request_token = ?1",
                [&token],
                |row| row.get::<_, String>(0),
            )
            .unwrap();
        let journal: Value = serde_json::from_str(&journal).unwrap();
        let Some(stop) = noticed::STEPS
            .iter()
            .find(|step| journal["steps"].get(**step).is_some())
        else {
            panic!("turn {turn} noticed nothing at all: {journal}");
        };
        let mut replay = Replay::new(Vec::new());
        match stopped.act_stopping(b, &token, false, &mut replay, &mut |_| {}, Some(stop)) {
            Err(Error::Stopped(step)) => assert_eq!(&step, stop),
            other => panic!("expected the turn to stop after {stop}, got {other:?}"),
        }
        let mut replay = Replay::new(Vec::new());
        stopped
            .act(b, &token, false, &mut replay, &mut |_| {})
            .unwrap();
        assert_eq!(
            stamps(&straight, a),
            stamps(&stopped, b),
            "turn {turn}, stopped after {stop}"
        );
        steps_stopped.push(*stop);
        if straight.glance(a).unwrap().over {
            break;
        }
    }
    for step in noticed::STEPS {
        assert!(
            steps_stopped.contains(&step),
            "no turn stopped after {step}: {steps_stopped:?}"
        );
    }
}
