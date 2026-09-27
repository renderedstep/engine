//! The engine sweep, played through this engine.
//!
//! `parity --worlds DIR SCRIPT.yml` plays one sweep script and prints one
//! state dump per step, one JSON object per line: the command contract of
//! the Ruby engine's `bin/rails engine:parity_diff` (`docs/engine-parity.md`
//! in that repository). `DIR` holds each world as the SQL text of the
//! database the Ruby engine loaded, named after the world's title.
//!
//! `ENGINE_STEP=N parity --database FILE --player NAME SCRIPT.yml` plays
//! step `N` alone on a database the Ruby engine's runner prepared and keeps,
//! and prints its one dump: the shared-database contract, where the runner
//! plays each `reseed:` step itself and fills `shown`, which this engine
//! prints as null.
//!
//! `parity --check DIR` plays every script in `DIR/scripts` on the worlds in
//! `DIR/worlds`, compares each with its golden file in `DIR/goldens`, and
//! names each script's first divergence. It fails when a script named in
//! `DIR/PASSING` diverges, or when one that is not named there agrees.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::parity;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.as_slice() {
        [flag, dir] if flag == "--check" => check(Path::new(dir)),
        [flag, worlds, script] if flag == "--worlds" => dumps(Path::new(worlds), Path::new(script)),
        [database, file, player_flag, player, script]
            if database == "--database" && player_flag == "--player" =>
        {
            one_step(Path::new(file), player, Path::new(script))
        }
        _ => Err(USAGE.to_string()),
    };
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "usage: parity --worlds DIR SCRIPT.yml \
                     | ENGINE_STEP=N parity --database FILE --player NAME SCRIPT.yml \
                     | parity --check DIR";

fn one_step(database: &Path, player: &str, path: &Path) -> Result<ExitCode, String> {
    let index: usize = std::env::var("ENGINE_STEP")
        .map_err(|_| "ENGINE_STEP names the step to play, counting from 1".to_string())?
        .trim()
        .parse()
        .map_err(|_| "ENGINE_STEP is a step number, counting from 1".to_string())?;
    let script = parity::load_script(path)?;
    let mut engine = Engine::open(database).map_err(|error| error.to_string())?;
    let dump = parity::play_one(&mut engine, &script, index, player)
        .map_err(|error| format!("{}: step {index}: {error}", script.name))?;
    println!("{dump}");
    Ok(ExitCode::SUCCESS)
}

fn dumps(worlds: &Path, path: &Path) -> Result<ExitCode, String> {
    let script = parity::load_script(path)?;
    let mut engine = parity::open_world_for(worlds, &script)?;
    match parity::play(&mut engine, &script) {
        Ok(dumps) => {
            for dump in dumps {
                println!("{dump}");
            }
            Ok(ExitCode::SUCCESS)
        }
        Err(stopped) => {
            for dump in stopped.dumps {
                println!("{dump}");
            }
            Err(format!(
                "{}: {}: {}",
                script.name, stopped.step, stopped.error
            ))
        }
    }
}

fn check(dir: &Path) -> Result<ExitCode, String> {
    let checked = parity::check(dir)?;
    for divergence in &checked.divergences {
        println!("{divergence}\n");
    }
    println!(
        "AGREED: {} of {} script(s), step for step.",
        checked.agreed.len(),
        checked.total
    );
    if checked.failures.is_empty() {
        Ok(ExitCode::SUCCESS)
    } else {
        Err(checked.failures.join("\n"))
    }
}
