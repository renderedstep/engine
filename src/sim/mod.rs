//! The fight simulator: fights played through the engine's own turn loop,
//! many times over, and counted.
//!
//! A fight is a throwaway game in memory ([`arena`]): a [`matchup::Matchup`]
//! puts the player and their foes in one room of a small world, with a
//! written room beside it to run to. A [`policy::Policy`] types one line a
//! turn (`attack`, `go`, `drink`, `throw`), and the line goes through
//! [`Engine::play`] (the mechanics loop, [`Driver::Mechanics`]) or
//! [`Engine::submit`] in a game the player narrates (the told loop the
//! browser plays, [`Driver::Told`]), with no model to ask. Nothing here
//! decides a blow, a riposte, a death or the end of a fight: the turn loop
//! does, and a fight is read back off the records it left. A rule that lands
//! in the engine is therefore in the simulator the moment it lands.
//!
//! A fight is named by its seed, which is its playthrough's id less
//! [`arena::SEED_BASE`]: the dice a blow throws are seeded off the
//! playthrough (`roll::Seed`), so the same seed plays the same fight again.
//! [`tally::Summary`] counts what many fights did.

pub mod arena;
pub mod golden;
pub mod matchup;
pub mod policy;
pub mod tally;

use crate::engine::{Engine, Error};
use crate::model::Replay;
use crate::outcome::State;
use crate::playthrough::Mode;
use arena::Arena;
use matchup::Matchup;
use policy::{Act, View};

/// Which turn loop plays the lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Driver {
    /// [`Engine::submit`] in a game the player narrates, every line
    /// slashed: the loop the browser plays, which moves the story's clock.
    Told,
    /// [`Engine::play`]: the no-model mechanics loop. Faster, and its clock
    /// stands still between fights' closings.
    Mechanics,
}

impl Driver {
    pub fn parse(name: &str) -> Option<Driver> {
        match name {
            "told" => Some(Driver::Told),
            "mechanics" => Some(Driver::Mechanics),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Driver::Told => "told",
            Driver::Mechanics => "mechanics",
        }
    }
}

/// How a fight ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum End {
    /// No foe is left fighting in the room.
    Won,
    /// The player is dead.
    Died,
    /// The player stands somewhere else.
    Fled,
    /// The fight was still going after [`MAX_LINES`] lines.
    Stalled,
}

impl End {
    pub const ALL: [End; 4] = [End::Won, End::Died, End::Fled, End::Stalled];

    pub fn as_str(self) -> &'static str {
        match self {
            End::Won => "won",
            End::Died => "died",
            End::Fled => "fled",
            End::Stalled => "stalled",
        }
    }
}

/// The most lines one fight plays before it is called stalled.
pub const MAX_LINES: usize = 200;

/// One blow, as its `playthrough_blows` row has it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blow {
    pub attacker: String,
    pub target: String,
    /// The die as thrown, before the target's hit points stop it at zero.
    pub damage: i64,
    pub hp_after: i64,
    /// The player struck it.
    pub by_player: bool,
    /// The player took it.
    pub on_player: bool,
}

/// One line the policy typed, and what it left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub typed: String,
    pub blows: Vec<Blow>,
    /// The player's hit points after the line.
    pub hp: i64,
    /// Every foe's hit points after the line, by name, the dead at zero.
    pub foes: Vec<(String, i64)>,
}

/// One fight, read off the records it left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fight {
    pub seed: i64,
    pub end: End,
    /// Lines typed, each one a round.
    pub lines: usize,
    pub max_hp: i64,
    pub hp_left: i64,
    /// Damage as thrown, summed over the blows the player struck and took.
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub blows_struck: usize,
    pub blows_taken: usize,
    pub drinks: usize,
    pub throws: usize,
    /// Every line, kept only when the fight was asked for line by line.
    pub log: Vec<Line>,
}

/// Plays one fight on a fresh copy of the arena.
pub fn fight(arena: &Arena, matchup: &Matchup, seed: i64, keep_log: bool) -> Result<Fight, String> {
    let failed = |error: Error| format!("seed {seed}: {error}");
    let mut engine = arena.fresh(seed)?;
    let mode = match matchup.driver {
        Driver::Told => Mode::PlayerNarrates,
        Driver::Mechanics => Mode::Narrated,
    };
    let game = engine.start_in(arena.story, mode).map_err(failed)?;
    let mut state = engine.read(game, Vec::new()).map_err(failed)?.state;
    let max_hp = state.hp.ok_or_else(|| {
        format!("seed {seed}: the player has no stat block, so there is nothing to fight with")
    })?;
    let mut fight = Fight {
        seed,
        end: End::Stalled,
        lines: 0,
        max_hp,
        hp_left: max_hp,
        damage_dealt: 0,
        damage_taken: 0,
        blows_struck: 0,
        blows_taken: 0,
        drinks: 0,
        throws: 0,
        log: Vec::new(),
    };
    let mut seen = 0;
    let mut thrown = Vec::new();
    while fight.lines < MAX_LINES {
        if let Some(end) = ended(arena, &state) {
            fight.end = end;
            break;
        }
        let view = View {
            seed,
            hp: state.hp.unwrap_or_default(),
            max_hp,
            foes: &state.foes,
            carrying: &state.carrying,
            thrown: &thrown,
        };
        let act = matchup.policy.choose(arena, &view);
        let typed = act.line();
        play(&mut engine, matchup.driver, game, &typed, seed, fight.lines)?;
        state = engine.read(game, Vec::new()).map_err(failed)?.state;
        fight.lines += 1;
        match act {
            Act::Drink(_) => fight.drinks += 1,
            Act::Throw(thing, _) => {
                fight.throws += 1;
                thrown.push(thing);
            }
            _ => {}
        }
        let blows = arena.blows(&engine, game, &mut seen)?;
        for blow in &blows {
            if blow.by_player {
                fight.blows_struck += 1;
                fight.damage_dealt += blow.damage;
            }
            if blow.on_player {
                fight.blows_taken += 1;
                fight.damage_taken += blow.damage;
            }
        }
        if keep_log {
            fight.log.push(Line {
                typed,
                blows,
                hp: state.hp.unwrap_or_default(),
                foes: arena.foes_hp(&engine, game)?,
            });
        }
    }
    if fight.lines == MAX_LINES {
        fight.end = ended(arena, &state).unwrap_or(End::Stalled);
    }
    fight.hp_left = state.hp.unwrap_or_default();
    Ok(fight)
}

/// How the fight stands after a line, or none while it goes on.
fn ended(arena: &Arena, state: &State) -> Option<End> {
    if state.dead || state.hp == Some(0) {
        Some(End::Died)
    } else if state.location.as_ref().map(|room| room.id) != Some(arena.room) {
        Some(End::Fled)
    } else if state.foes.is_empty() {
        Some(End::Won)
    } else {
        None
    }
}

/// Plays one typed line through the driver's turn loop. A refused line is
/// an error: it buys no riposte, so a policy that types one would stall.
fn play(
    engine: &mut Engine,
    driver: Driver,
    game: i64,
    line: &str,
    seed: i64,
    index: usize,
) -> Result<(), String> {
    let failed = |error: Error| format!("seed {seed}: {line:?}: {error}");
    match driver {
        Driver::Mechanics => {
            let outcome = engine.play(game, line, &mut |_| {}).map_err(failed)?;
            if let Some(refusal) = outcome.report.refusal {
                return Err(format!("seed {seed}: {line:?} was refused: {refusal}"));
            }
        }
        Driver::Told => {
            let mut models = Replay::new(Vec::new());
            let token = format!("fight-{seed}-{index}");
            let submitted = engine
                .submit(game, &format!("/{line}"), &token, &mut models, &mut |_| {})
                .map_err(failed)?;
            if !models.calls().is_empty() {
                return Err(format!(
                    "seed {seed}: {line:?} asked a model: {:?}",
                    models.calls()
                ));
            }
            if let Some(refusal) = submitted.turned.refusal {
                return Err(format!(
                    "seed {seed}: {line:?} was refused: {}",
                    refusal.fact
                ));
            }
        }
    }
    Ok(())
}

/// Plays every seed, on `threads` threads, and returns the fights in seed
/// order: the same seeds give the same fights however many threads play
/// them.
pub fn run(matchup: &Matchup, seeds: &[i64], threads: usize) -> Result<Vec<Fight>, String> {
    let arena = Arena::build(matchup)?;
    let threads = threads.clamp(1, seeds.len().max(1));
    let chunk = seeds.len().div_ceil(threads).max(1);
    let played: Vec<Result<Vec<Fight>, String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = seeds
            .chunks(chunk)
            .map(|chunk| {
                let arena = &arena;
                scope.spawn(move || {
                    chunk
                        .iter()
                        .map(|&seed| fight(arena, matchup, seed, false))
                        .collect::<Result<Vec<_>, _>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err("a fight thread panicked".into()))
            })
            .collect()
    });
    let mut fights = Vec::with_capacity(seeds.len());
    for chunk in played {
        fights.extend(chunk?);
    }
    Ok(fights)
}

/// Plays one seed line by line, for `--replay`.
pub fn replay(matchup: &Matchup, seed: i64) -> Result<Fight, String> {
    let arena = Arena::build(matchup)?;
    fight(&arena, matchup, seed, true)
}
