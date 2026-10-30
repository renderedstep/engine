//! A real world's own fights: every hostile of a story in a seeded
//! database, each one against that story's protagonist, at the player
//! levels asked for (`fight --database FILE --story TITLE --levels 3..6`).
//!
//! The database is copied into memory and the file is closed before
//! anything is read, so the world is never written. What is taken from it
//! is each body's stat block; the fights are played in the arena
//! (`sim::arena`), with the world's protagonist as the player and one of
//! its hostiles as the foe, so a world with no written room beside its
//! hostiles' still needs no room writer. A stat the world left empty is
//! thrown for each seed (`rolled`), as a matchup file's would be.

use super::matchup::{Foes, Matchup, Stat, Stats};
use super::sweep::Point;
use rusqlite::{Connection, OpenFlags, MAIN_DB};
use std::path::Path;

/// One body of the world: its name and stat block.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub name: String,
    pub stats: Stats,
}

/// A story's protagonist and its hostiles, read off a copy of the database.
pub fn bodies(database: &Path, story: &str) -> Result<(Body, Vec<Body>), String> {
    let failed = |error: rusqlite::Error| format!("{}: {error}", database.display());
    if !database.is_file() {
        return Err(format!("{}: no such database", database.display()));
    }
    let image = {
        let file = Connection::open_with_flags(
            database,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(failed)?;
        let image = file.serialize(MAIN_DB).map_err(failed)?.to_vec();
        file.close().map_err(|(_, error)| failed(error))?;
        image
    };
    let mut conn = Connection::open_in_memory().map_err(failed)?;
    conn.deserialize_read_exact(MAIN_DB, &image[..], image.len(), false)
        .map_err(failed)?;
    let id: i64 = conn
        .query_row("SELECT id FROM stories WHERE title = ?1", [story], |row| {
            row.get(0)
        })
        .map_err(|_| format!("{}: no story titled {story:?}", database.display()))?;
    let mut statement = conn
        .prepare(
            "SELECT is_protagonist, COALESCE(fullname, nickname, 'character ' || id), \
             level, hit_die, strength, dexterity, will FROM characters \
             WHERE story_id = ?1 AND (is_protagonist = 1 OR hostile = 1) ORDER BY id",
        )
        .map_err(failed)?;
    let rows: Vec<(bool, Body)> = statement
        .query_map([id], |row| {
            let stat = |n: usize| -> rusqlite::Result<Stat> {
                Ok(row
                    .get::<_, Option<i64>>(n)?
                    .map_or(Stat::Rolled, Stat::Fixed))
            };
            Ok((
                row.get(0)?,
                Body {
                    name: row.get(1)?,
                    stats: Stats {
                        level: stat(2)?,
                        hit_die: stat(3)?,
                        strength: stat(4)?,
                        dexterity: stat(5)?,
                        will: stat(6)?,
                    },
                },
            ))
        })
        .and_then(|rows| rows.collect())
        .map_err(failed)?;
    let mut protagonist = None;
    let mut hostiles = Vec::new();
    for (is_protagonist, body) in rows {
        if is_protagonist {
            protagonist.get_or_insert(body);
        } else {
            hostiles.push(body);
        }
    }
    let protagonist = protagonist.ok_or_else(|| format!("{story:?} has no protagonist"))?;
    if hostiles.is_empty() {
        return Err(format!("{story:?} has no hostile to fight"));
    }
    Ok((protagonist, hostiles))
}

/// Every hostile of the story against its protagonist at each level, the
/// rest of each fight (seeds, driver, what the player carries, the policy)
/// as `base` has it. With no levels, the protagonist's own.
pub fn points(
    base: &Matchup,
    database: &Path,
    story: &str,
    levels: &[i64],
) -> Result<Vec<Point>, String> {
    let (protagonist, hostiles) = bodies(database, story)?;
    let levels = if levels.is_empty() {
        vec![protagonist.stats.level]
    } else {
        levels.iter().map(|level| Stat::Fixed(*level)).collect()
    };
    let mut points = Vec::new();
    for hostile in &hostiles {
        for level in &levels {
            let level_text = match level {
                Stat::Fixed(n) => n.to_string(),
                Stat::Rolled => "rolled".into(),
            };
            let name = format!("{} vs L{level_text}", hostile.name);
            let mut matchup = base.clone();
            matchup.name = name.clone();
            matchup.player = Stats {
                level: *level,
                ..protagonist.stats.clone()
            };
            matchup.foes = vec![Foes {
                count: 1,
                name: hostile.name.clone(),
                stats: hostile.stats.clone(),
                columns: Vec::new(),
            }];
            points.push(Point {
                name,
                columns: vec![
                    ("hostile".into(), hostile.name.clone()),
                    ("player.stats.level".into(), level_text),
                ],
                matchup,
            });
        }
    }
    Ok(points)
}

/// `3..6`, `3..=6` or `3,5`.
pub fn parse_levels(text: &str) -> Result<Vec<i64>, String> {
    let bad = || format!("--levels {text:?} is a range A..B or a list A,B");
    let levels: Vec<i64> = match text.split_once("..") {
        Some((from, to)) => {
            let from: i64 = from.trim().parse().map_err(|_| bad())?;
            let to: i64 = to
                .trim_start_matches('=')
                .trim()
                .parse()
                .map_err(|_| bad())?;
            (from..=to).collect()
        }
        None => text
            .split(',')
            .map(|level| level.trim().parse().map_err(|_| bad()))
            .collect::<Result<_, _>>()?,
    };
    if levels.is_empty() || levels.iter().any(|level| *level < 1) {
        return Err(bad());
    }
    Ok(levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_levels() {
        assert_eq!(parse_levels("3..6").unwrap(), vec![3, 4, 5, 6]);
        assert_eq!(parse_levels("3,5").unwrap(), vec![3, 5]);
        assert!(parse_levels("0..2").is_err());
        assert!(parse_levels("6..3").is_err());
    }
}
