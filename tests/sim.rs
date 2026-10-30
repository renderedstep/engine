//! The fight simulator (`sim`) against what it must not get wrong: the
//! rules' exact odds, the two turn loops' agreement on a fight with no move
//! in it, the same seed playing the same fight, and its tally agreeing with
//! the blows the game wrote down.

use renderedstep_engine::sim::export::{self, Played};
use renderedstep_engine::sim::matchup::{Foes, Matchup, Stat, Stats};
use renderedstep_engine::sim::policy::Policy;
use renderedstep_engine::sim::{self, sweep, tally::Summary, world, Driver, End, Fight};

fn foes(count: usize, level: i64, hit_die: i64) -> Foes {
    Foes {
        count,
        name: "Foe".into(),
        stats: Stats {
            level: Stat::Fixed(level),
            hit_die: Stat::Fixed(hit_die),
            ..Stats::foe()
        },
        columns: Vec::new(),
    }
}

fn matchup(driver: Driver, count: usize, level: i64, hit_die: i64) -> Matchup {
    Matchup {
        driver,
        ..Matchup::new("test", vec![foes(count, level, hit_die)])
    }
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get())
}

/// The exact chance the house body (18 hit points, d8) wins attacking to
/// the end, worked out from the rules as stated rather than played: the
/// player's blow always lands for one die at the first foe standing, then
/// every foe standing strikes once for one of its own. These are the
/// figures of the fight simulator's scope, Appendix B. A broken arena or a
/// seed that does not move the dice fails here, even if a later golden is
/// written again over it.
#[test]
fn plays_the_rules_exact_odds() {
    const FIGHTS: i64 = 600;
    for (count, level, hit_die, exact) in [(1, 1, 8, 0.9854), (1, 3, 10, 0.2338), (3, 1, 6, 0.1914)]
    {
        let matchup = matchup(Driver::Mechanics, count, level, hit_die);
        let seeds: Vec<i64> = (1..=FIGHTS).collect();
        let fights = sim::run(&matchup, &seeds, threads()).unwrap();
        let won = Summary::of(&fights).rate(End::Won);
        let error = (exact * (1.0 - exact) / FIGHTS as f64).sqrt();
        assert!(
            (won - exact).abs() <= 4.0 * error,
            "{count} x L{level} d{hit_die}: won {won:.4}, the rules say {exact:.4} (4 sd = {:.4})",
            4.0 * error
        );
    }
}

/// With no move and no clock in it, a fight is the same fight seed for seed
/// on either loop: a rule added to one loop and not the other fails here.
/// (A move is not: the told loop writes its arrival before the riposte, so
/// the parting blow is thrown at a later story time and from other dice.)
#[test]
fn the_two_loops_play_the_same_fight() {
    for (count, level, hit_die) in [(1, 3, 10), (3, 1, 6)] {
        let told = matchup(Driver::Told, count, level, hit_die);
        let mechanics = matchup(Driver::Mechanics, count, level, hit_die);
        let told_arena = sim::arena::Arena::build(&told).unwrap();
        let mechanics_arena = sim::arena::Arena::build(&mechanics).unwrap();
        for seed in 1..=40 {
            let a = sim::fight(&told_arena, &told, seed, true).unwrap();
            let b = sim::fight(&mechanics_arena, &mechanics, seed, true).unwrap();
            assert_eq!(a, b, "{count} x L{level} d{hit_die}, seed {seed}");
        }
    }
}

/// The same seed plays the same fight, alone or among others, on one
/// thread or several.
#[test]
fn a_seed_plays_the_same_fight_again() {
    let mut matchup = matchup(Driver::Told, 2, 1, 8);
    matchup.carrying = vec![vec![
        ("name".into(), "healing draught".into()),
        ("use_kind".into(), "healing".into()),
    ]];
    matchup.policy = Policy::named("careful").unwrap();
    let first = sim::replay(&matchup, 17).unwrap();
    let again = sim::replay(&matchup, 17).unwrap();
    assert_eq!(first, again);
    assert!(!first.log.is_empty());

    let seeds: Vec<i64> = (1..=24).collect();
    let alone = sim::run(&matchup, &seeds, 1).unwrap();
    let shared = sim::run(&matchup, &seeds, 4).unwrap();
    assert_eq!(alone, shared);
    let unlogged = Fight {
        log: Vec::new(),
        ..first
    };
    assert_eq!(alone[16], unlogged);
    assert_ne!(alone[0], alone[1], "two seeds threw the same dice");
}

/// A fight's tally is what its blows say: the damage on each side, the hit
/// points left, a blow struck by the player each round, and every foe of a
/// won fight at zero.
#[test]
fn the_tally_is_what_the_blows_say() {
    let matchup = matchup(Driver::Told, 3, 1, 6);
    let mut checked = [false; 2];
    for seed in 1..=30 {
        let fight = sim::replay(&matchup, seed).unwrap();
        let blows: Vec<_> = fight.log.iter().flat_map(|line| &line.blows).collect();
        let struck: Vec<_> = blows.iter().filter(|blow| blow.by_player).collect();
        let taken: Vec<_> = blows.iter().filter(|blow| blow.on_player).collect();
        assert_eq!(fight.blows_struck, struck.len());
        assert_eq!(fight.blows_taken, taken.len());
        assert_eq!(
            fight.damage_dealt,
            struck.iter().map(|b| b.damage).sum::<i64>()
        );
        assert_eq!(
            fight.damage_taken,
            taken.iter().map(|b| b.damage).sum::<i64>()
        );
        assert_eq!(
            fight.blows_struck, fight.lines,
            "seed {seed}: one swing a round"
        );
        let last_taken = taken.last().map_or(fight.max_hp, |blow| blow.hp_after);
        assert_eq!(fight.hp_left, last_taken, "seed {seed}");
        assert_eq!(
            fight.hp_left,
            (fight.max_hp - fight.damage_taken).max(0),
            "seed {seed}"
        );
        let last = fight.log.last().unwrap();
        assert_eq!(last.hp, fight.hp_left);
        match fight.end {
            End::Won => {
                assert!(last.foes.iter().all(|(_, hp)| *hp == 0), "seed {seed}");
                checked[0] = true;
            }
            End::Died => {
                assert_eq!(fight.hp_left, 0);
                assert!(last.foes.iter().any(|(_, hp)| *hp > 0), "seed {seed}");
                checked[1] = true;
            }
            end => panic!("seed {seed}: {end:?} in a fight with no way out taken"),
        }
    }
    assert_eq!(checked, [true, true], "both a win and a death were checked");
}

/// Every way of playing types only lines the turn loop plays: none is
/// refused, none asks a model, and each ends its fights.
#[test]
fn every_policy_plays_to_an_end() {
    for kind in ["attack", "throw_first", "naive", "careful"] {
        for driver in [Driver::Told, Driver::Mechanics] {
            let mut matchup = matchup(driver, 2, 2, 8);
            matchup.carrying = vec![
                vec![
                    ("name".into(), "healing draught".into()),
                    ("use_kind".into(), "healing".into()),
                ],
                vec![
                    ("name".into(), "brick".into()),
                    ("bulk".into(), "handy".into()),
                ],
            ];
            matchup.policy = Policy::named(kind).unwrap();
            matchup.policy.flee_below = Some(4);
            let seeds: Vec<i64> = (1..=12).collect();
            let fights = sim::run(&matchup, &seeds, threads()).unwrap();
            assert!(
                fights.iter().all(|fight| fight.end != End::Stalled),
                "{kind} on {driver:?}"
            );
            if kind == "throw_first" {
                assert!(fights.iter().all(|fight| fight.throws == 1), "{kind}");
            }
        }
    }
}

/// A point of a sweep is the matchup it names: the same fights, seed for
/// seed, as that matchup written out on its own.
#[test]
fn a_sweep_point_is_its_own_matchup() {
    let points = sweep::grid(
        "grid",
        "seeds: 1..20\ndriver: mechanics\nfoes:\n  - {stats: {hit_die: 10}}\n\
         sweep: {player.stats.level: [3, 5], foes.0.stats.level: [1, 3]}\n",
    )
    .unwrap();
    assert_eq!(points.len(), 4);
    let point = &points[1];
    assert_eq!(point.name, "grid player.stats.level=3 foes.0.stats.level=3");
    let alone = matchup(Driver::Mechanics, 1, 3, 10);
    let seeds: Vec<i64> = (1..=20).collect();
    assert_eq!(
        sim::run(&point.matchup, &seeds, 2).unwrap(),
        sim::run(&alone, &seeds, 2).unwrap()
    );
    let higher = sim::run(&points[3].matchup, &seeds, 2).unwrap();
    assert!(
        Summary::of(&higher).rate(End::Won)
            > Summary::of(&sim::run(&alone, &seeds, 2).unwrap()).rate(End::Won)
    );
}

/// A seeded world's hostiles are fought from a copy: the file is the same
/// after as before, every hostile meets the protagonist at every level, and
/// each fight is the one its stat blocks written into a matchup would play.
#[test]
fn a_world_s_hostiles_are_fought_from_a_copy() {
    let path = std::env::temp_dir().join(format!("fight-world-{}.sqlite3", std::process::id()));
    let _ = std::fs::remove_file(&path);
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(sim::arena::WORLD).unwrap();
        conn.execute_batch(
            "INSERT INTO characters (created_at, updated_at, fullname, hostile, race_id, \
             story_id, level, hit_die, strength, dexterity, will) \
             SELECT created_at, updated_at, 'Rust Hound', 1, race_id, story_id, 2, 10, 10, 10, 10 \
             FROM characters WHERE is_protagonist = 1; \
             INSERT INTO characters (created_at, updated_at, fullname, hostile, race_id, \
             story_id, level, hit_die) \
             SELECT created_at, updated_at, 'Hull Mite', 1, race_id, story_id, 1, 6 \
             FROM characters WHERE is_protagonist = 1;",
        )
        .unwrap();
    }
    let before = std::fs::read(&path).unwrap();
    let mut base = Matchup::new("base", Vec::new());
    base.driver = Driver::Mechanics;
    base.seeds = (1..=15).collect();
    let points = world::points(&base, &path, sim::arena::STORY, &[3, 5]).unwrap();
    let names: Vec<&str> = points.iter().map(|point| point.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "Rust Hound vs L3",
            "Rust Hound vs L5",
            "Hull Mite vs L3",
            "Hull Mite vs L5"
        ]
    );
    let played: Vec<Played> = points
        .iter()
        .map(|point| Played {
            point,
            fights: sim::run(&point.matchup, &point.matchup.seeds, 2).unwrap(),
        })
        .collect();
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "the world file was written"
    );
    let _ = std::fs::remove_file(&path);

    // The world's protagonist is the player, at the level asked for.
    let player = &points[0].matchup.player;
    assert_eq!(player.level, Stat::Fixed(3));
    assert_eq!(
        (
            player.hit_die,
            player.strength,
            player.dexterity,
            player.will
        ),
        (
            Stat::Fixed(8),
            Stat::Fixed(12),
            Stat::Fixed(12),
            Stat::Fixed(12)
        )
    );
    // A stat the world left empty is thrown for each seed.
    assert_eq!(points[2].matchup.foes[0].stats.strength, Stat::Rolled);
    let mut hound = matchup(Driver::Mechanics, 1, 2, 10);
    hound.player = player.clone();
    hound.foes[0].name = "Rust Hound".into();
    assert_eq!(played[0].fights, sim::run(&hound, &base.seeds, 2).unwrap());

    // The fight table's chances add up: each fight is won, lost, fled or
    // stalled, and its hit-point loss with death is one whole draw.
    let table = export::fight_table(&played);
    for row in table["matchups"].as_array().unwrap() {
        let ends: f64 = ["p_win", "p_died", "p_fled", "p_stalled"]
            .iter()
            .map(|key| row[key].as_f64().unwrap())
            .sum();
        assert!((ends - 1.0).abs() < 1e-9, "{row}");
        let lost: f64 = row["hp_loss"]
            .as_object()
            .unwrap()
            .values()
            .map(|share| share.as_f64().unwrap())
            .sum();
        assert!(
            (lost + row["p_died"].as_f64().unwrap() - 1.0).abs() < 1e-9,
            "{row}"
        );
    }
    let csv = export::csv(&played);
    assert_eq!(csv.lines().count(), 5);
    assert!(
        csv.starts_with("matchup,hostile,player.stats.level,fights,won,"),
        "{csv}"
    );
}
