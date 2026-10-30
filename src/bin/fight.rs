//! The fight simulator (`renderedstep_engine::sim`): plays one matchup's
//! fights through the engine's own turn loop, with no model, and prints
//! what they did.
//!
//! `fight MATCHUP.yml` plays every seed the file names and prints a table:
//! how often the player won, died and got away, with 95% intervals, how
//! many rounds a fight took, the hit points the survivors kept, and the
//! damage and blows on each side. `--json` prints the same as JSON.
//!
//! A file with a `sweep:` key is a grid of matchups (`sim::sweep`), each
//! played and printed as a row. `--database FILE --story TITLE` plays that
//! seeded world's own hostiles against its protagonist instead
//! (`sim::world`), at `--levels A..B` (default: the protagonist's own); a
//! matchup file beside them sets the rest (seeds, carrying, policy). The
//! database is copied into memory and never written.
//!
//! `--csv` prints a row a matchup as CSV. `--fight-table FILE` writes each
//! matchup's chance to win and its hit-point loss as JSON (`-` prints it),
//! for a dungeon-level study to read (`sim::export::fight_table`).
//!
//! `--seeds A..B`, `--driver told|mechanics` and `--threads N` stand in for
//! the file's own.
//!
//! `fight MATCHUP.yml --replay SEED` plays that one fight again and prints
//! it line by line: what was typed, every blow, and everybody's hit points.
//!
//! `fight --check DIR` plays every matchup in `DIR/goldens` and fails when
//! a table differs from the golden kept beside it (`sim::golden`).
//! `fight --write DIR` writes each golden again whose table moved.

use renderedstep_engine::sim::export::{self, Played};
use renderedstep_engine::sim::sweep::{self, Point};
use renderedstep_engine::sim::{self, golden, matchup::Matchup, tally, world, Driver};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage: fight [MATCHUP.yml] [--database FILE --story TITLE [--levels A..B]] \
                     [--seeds A..B] [--driver told|mechanics] [--threads N] \
                     [--json | --csv] [--fight-table FILE|-] [--replay SEED] \
                     | fight --check DIR | fight --write DIR";

fn main() -> ExitCode {
    // SQLite counts every allocation under one lock for the whole process,
    // which holds fights on several threads to the speed of about three.
    // Nothing here reads the count, so it is turned off before the first
    // connection opens (a later call is refused, and only costs speed).
    unsafe {
        rusqlite::ffi::sqlite3_config(rusqlite::ffi::SQLITE_CONFIG_MEMSTATUS, 0);
    }
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [flag, dir] if flag == "--check" => return check(Path::new(dir)),
        [flag, dir] if flag == "--write" => {
            let written = golden::write(Path::new(dir))?;
            println!("WROTE: {} golden(s): {}", written.len(), written.join(", "));
            return Ok(());
        }
        _ => {}
    }
    let mut file = None;
    let mut json = false;
    let mut csv = false;
    let mut fight_table = None;
    let mut replay = None;
    let mut seeds = None;
    let mut driver = None;
    let mut threads = None;
    let mut database = None;
    let mut story = None;
    let mut levels = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("{arg} needs a value\n{USAGE}"))
        };
        match arg.as_str() {
            "--json" => json = true,
            "--csv" => csv = true,
            "--fight-table" => fight_table = Some(value()?),
            "--database" => database = Some(value()?),
            "--story" => story = Some(value()?),
            "--levels" => levels = world::parse_levels(&value()?)?,
            "--replay" => {
                replay = Some(
                    value()?
                        .parse::<i64>()
                        .map_err(|_| "--replay takes a seed".to_string())?,
                )
            }
            "--seeds" => seeds = Some(sim::matchup::parse_seeds(&value()?)?),
            "--driver" => {
                let name = value()?;
                driver = Some(
                    Driver::parse(&name)
                        .ok_or_else(|| format!("--driver {name:?} is told or mechanics"))?,
                )
            }
            "--threads" => {
                threads = Some(
                    value()?
                        .parse::<usize>()
                        .map_err(|_| "--threads takes a number".to_string())?,
                )
            }
            _ if arg.starts_with("--") || file.is_some() => return Err(USAGE.to_string()),
            _ => file = Some(arg),
        }
    }
    if json && csv {
        return Err(format!("--json or --csv, not both\n{USAGE}"));
    }
    let mut points = match (database, story) {
        (Some(database), Some(story)) => {
            let base = match &file {
                Some(file) => Matchup::load(Path::new(file))?,
                None => Matchup::new("world", Vec::new()),
            };
            world::points(&base, Path::new(&database), &story, &levels)?
        }
        (None, None) if levels.is_empty() => sweep::load(Path::new(&file.ok_or(USAGE)?))?,
        (None, None) => return Err(format!("--levels is for --database\n{USAGE}")),
        _ => return Err(format!("--database and --story go together\n{USAGE}")),
    };
    for point in &mut points {
        let matchup = &mut point.matchup;
        if let Some(seeds) = &seeds {
            matchup.seeds = seeds.clone();
        }
        if let Some(driver) = driver {
            matchup.driver = driver;
        }
        if let Some(threads) = threads {
            matchup.threads = threads;
        }
    }

    if let Some(seed) = replay {
        if seed < 1 {
            return Err("--replay takes a seed of 1 or more".into());
        }
        let [point] = points.as_slice() else {
            return Err(format!(
                "--replay plays one matchup, and this names {}",
                points.len()
            ));
        };
        let fight = sim::replay(&point.matchup, seed)?;
        if json {
            println!("{}", tally::replay_json(&fight));
        } else {
            print!("{}", tally::replay(&fight));
        }
        return Ok(());
    }

    let started = Instant::now();
    if let ([point], false, None) = (points.as_slice(), csv, &fight_table) {
        if point.columns.is_empty() {
            single(&point.matchup, json)?;
            report(started, &points);
            return Ok(());
        }
    }
    let played = points
        .iter()
        .map(|point| {
            let matchup = &point.matchup;
            Ok(Played {
                point,
                fights: sim::run(matchup, &matchup.seeds, matchup.threads)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if let Some(path) = &fight_table {
        let table = format!("{:#}\n", export::fight_table(&played));
        if path == "-" {
            print!("{table}");
        } else {
            std::fs::write(path, table).map_err(|error| format!("{path}: {error}"))?;
            eprintln!(
                "  wrote the fight table of {} matchup(s) to {path}",
                played.len()
            );
        }
    }
    if csv {
        print!("{}", export::csv(&played));
    } else if json {
        println!("{}", export::json(&played));
    } else if fight_table.as_deref() != Some("-") {
        print!("{}", export::text(&played));
    }
    report(started, &points);
    Ok(())
}

/// One matchup on its own, printed as its golden would keep it.
fn single(matchup: &Matchup, json: bool) -> Result<(), String> {
    if !json {
        print!("{}", golden::table(matchup)?);
        return Ok(());
    }
    let fights = sim::run(matchup, &matchup.seeds, matchup.threads)?;
    let (first, last) = (
        matchup.seeds.first().copied().unwrap_or_default(),
        matchup.seeds.last().copied().unwrap_or_default(),
    );
    let mut out = tally::Summary::of(&fights).json();
    out["matchup"] = matchup.name.clone().into();
    out["driver"] = matchup.driver.as_str().into();
    out["seeds"] = format!("{first}..{last}").into();
    out["policy"] = matchup.policy.kind.clone().into();
    println!("{out}");
    Ok(())
}

/// How long the fights took, on standard error.
fn report(started: Instant, points: &[Point]) {
    let fights: usize = points.iter().map(|point| point.matchup.seeds.len()).sum();
    let seconds = started.elapsed().as_secs_f64();
    eprintln!(
        "  ({:.1}s, {:.0} fights/s, {} matchup(s))",
        seconds,
        fights as f64 / seconds.max(1e-9),
        points.len()
    );
}

fn check(dir: &Path) -> Result<(), String> {
    let (played, failures) = golden::check(dir)?;
    if failures.is_empty() {
        println!("AGREED: {} fight golden(s).", played.len());
        Ok(())
    } else {
        Err(format!(
            "{}\nMOVED: {} of {} fight golden(s); if the odds were meant to move, \
             run fight --write {} and keep the diff in the same PR.",
            failures.join("\n"),
            failures.len(),
            played.len(),
            dir.display()
        ))
    }
}
