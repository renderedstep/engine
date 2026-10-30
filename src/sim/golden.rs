//! Fight goldens: the standard matchups in `DIR/goldens/*.yml`, each played
//! over its own seeds, and the table each one printed kept beside it as
//! `DIR/goldens/*.txt`. A rule that moves fight odds moves a table, so
//! `fight --check` fails until `fight --write` blesses the new one and the
//! diff shows what moved.

use super::matchup::Matchup;
use super::tally::Summary;
use std::path::{Path, PathBuf};

/// One golden matchup and the table it plays to now.
pub struct Played {
    pub name: String,
    pub golden: PathBuf,
    pub table: String,
}

/// The table a golden keeps: [`Summary::text`] under a heading that names
/// the fights, with nothing in it that varies from run to run.
pub fn table(matchup: &Matchup) -> Result<String, String> {
    let fights = super::run(matchup, &matchup.seeds, matchup.threads)?;
    let (first, last) = (
        matchup.seeds.first().copied().unwrap_or_default(),
        matchup.seeds.last().copied().unwrap_or_default(),
    );
    let heading = format!(
        "{}: {} fights, seeds {first}..{last}, {} driver, policy {}",
        matchup.name,
        fights.len(),
        matchup.driver.as_str(),
        matchup.policy.kind,
    );
    Ok(Summary::of(&fights).text(&heading))
}

/// Plays every matchup in `DIR/goldens`, in name order.
pub fn play(dir: &Path) -> Result<Vec<Played>, String> {
    let goldens = dir.join("goldens");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&goldens)
        .map_err(|error| format!("{}: {error}", goldens.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "yml"))
        .collect();
    files.sort();
    if files.is_empty() {
        return Err(format!("{}: no matchups", goldens.display()));
    }
    files
        .iter()
        .map(|file| {
            let matchup = Matchup::load(file)?;
            Ok(Played {
                name: matchup.name.clone(),
                golden: file.with_extension("txt"),
                table: table(&matchup)?,
            })
        })
        .collect()
}

/// Every golden that does not hold its matchup's table now, each with what
/// it holds and what was played; and every table with no matchup.
pub fn check(dir: &Path) -> Result<(Vec<Played>, Vec<String>), String> {
    let played = play(dir)?;
    let mut failures = Vec::new();
    for one in &played {
        match std::fs::read_to_string(&one.golden) {
            Ok(kept) if kept == one.table => {}
            Ok(kept) => failures.push(format!(
                "{} moved.\n--- kept\n{kept}+++ played\n{}",
                one.golden.display(),
                one.table
            )),
            Err(error) => failures.push(format!("{}: {error}", one.golden.display())),
        }
    }
    let goldens = dir.join("goldens");
    for entry in std::fs::read_dir(&goldens).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.extension().is_some_and(|ext| ext == "txt") && !path.with_extension("yml").exists()
        {
            failures.push(format!("{}: no matchup plays it", path.display()));
        }
    }
    Ok((played, failures))
}

/// Writes each golden whose table moved, and names them.
pub fn write(dir: &Path) -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    for one in play(dir)? {
        if std::fs::read_to_string(&one.golden).ok().as_deref() != Some(one.table.as_str()) {
            std::fs::write(&one.golden, &one.table)
                .map_err(|error| format!("{}: {error}", one.golden.display()))?;
            written.push(one.name);
        }
    }
    Ok(written)
}
