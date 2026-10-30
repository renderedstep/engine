//! The room a simulated fight is played in: the engine sweep's one-room
//! world (`parity/worlds/a-chamber-of-one-room.sql`), with the player's
//! stat block and what they carry set from the matchup, its foes standing in
//! the first room, and every room written, so that a move into the second
//! asks no room writer.
//!
//! The arena is built once and kept as an image of its database. Each fight
//! opens a fresh copy of that image in memory ([`Arena::fresh`]), so nothing
//! one fight wrote, not even a row of the world it changed, reaches the
//! next: reopening the image costs a fraction of loading the world's SQL
//! again. A world fixture at an older schema is refused when the copy is
//! opened (`store::Store::from_connection`), so the arena cannot fall
//! behind a migration unnoticed.

use super::matchup::{Matchup, Stat, Stats};
use super::Blow;
use crate::engine::Engine;
use crate::stat_block::{self, StatBlock};
use rusqlite::{params, Connection, ToSql, MAIN_DB};
use serde_json::{Map, Value};

/// The world every fight is played in.
pub const WORLD: &str = include_str!("../../parity/worlds/a-chamber-of-one-room.sql");

/// Its story's title.
pub const STORY: &str = "A Chamber of One Room (engine sweep)";

/// A fight's playthrough id is this plus its seed: the dice of every blow
/// are seeded off the playthrough, so the seed picks the fight.
pub const SEED_BASE: i64 = 1_000_000_000;

/// When the rows the arena writes were made.
const WRITTEN_AT: &str = "2026-10-02 13:25:38";

/// A body the arena put in the fight.
#[derive(Clone, Debug)]
struct Body {
    id: i64,
    name: String,
    stats: Stats,
}

/// The arena's database image, and what a fight needs to know of it.
#[derive(Debug)]
pub struct Arena {
    image: Vec<u8>,
    pub story: i64,
    /// The room the fight starts in.
    pub room: i64,
    /// The written room beside it, to run to.
    pub exit: i64,
    pub exit_name: String,
    player: Body,
    foes: Vec<Body>,
    /// Carried things by name, and whether each one heals.
    carrying: Vec<(String, bool)>,
}

impl Arena {
    /// Builds the arena for a matchup.
    pub fn build(matchup: &Matchup) -> Result<Arena, String> {
        let failed = |error: rusqlite::Error| format!("the arena: {error}");
        let conn = Connection::open_in_memory().map_err(failed)?;
        conn.execute_batch(WORLD).map_err(failed)?;
        let story: i64 = conn
            .query_row("SELECT id FROM stories WHERE title = ?1", [STORY], |row| {
                row.get(0)
            })
            .map_err(failed)?;
        let (player, race): (i64, i64) = conn
            .query_row(
                "SELECT id, race_id FROM characters WHERE story_id = ?1 AND is_protagonist = 1",
                [story],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(failed)?;
        let rooms: Vec<(i64, String, String)> = conn
            .prepare("SELECT id, name, detail_level FROM locations WHERE story_id = ?1 ORDER BY id")
            .and_then(|mut statement| {
                statement
                    .query_map([story], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                    .collect()
            })
            .map_err(failed)?;
        let room = rooms
            .iter()
            .find(|(_, _, detail)| detail == "realized")
            .map(|(id, _, _)| *id)
            .ok_or("the arena has no written room to start in")?;
        let (exit, exit_name) = rooms
            .iter()
            .find(|(id, _, _)| *id != room)
            .map(|(id, name, _)| (*id, name.clone()))
            .ok_or("the arena has no second room to run to")?;
        conn.execute(
            "UPDATE locations SET detail_level = 'realized', \
             description = COALESCE(description, teaser) WHERE story_id = ?1",
            [story],
        )
        .map_err(failed)?;
        update(&conn, "stories", story, &matchup.story)?;
        update(&conn, "characters", player, &fixed(&matchup.player))?;
        let mut carrying = Vec::new();
        for thing in &matchup.carrying {
            let mut values = vec![
                ("created_at".to_string(), Value::from(WRITTEN_AT)),
                ("updated_at".to_string(), Value::from(WRITTEN_AT)),
                ("character_id".to_string(), Value::from(player)),
            ];
            values.extend(thing.iter().cloned());
            insert(&conn, "items", &values)?;
            let name = |key: &str| {
                thing
                    .iter()
                    .find(|(column, _)| column == key)
                    .and_then(|(_, value)| value.as_str())
                    .map(str::to_string)
            };
            carrying.push((
                name("name").unwrap_or_default(),
                name("use_kind").as_deref() == Some("healing"),
            ));
        }
        let names = matchup.foe_names();
        let specs = matchup
            .foes
            .iter()
            .flat_map(|foes| std::iter::repeat_n(foes, foes.count));
        let mut foes = Vec::new();
        for (name, spec) in names.into_iter().zip(specs) {
            let mut values = vec![
                ("created_at".to_string(), Value::from(WRITTEN_AT)),
                ("updated_at".to_string(), Value::from(WRITTEN_AT)),
                ("fullname".to_string(), Value::from(name.clone())),
                ("hostile".to_string(), Value::from(true)),
                ("is_protagonist".to_string(), Value::from(false)),
                ("location_id".to_string(), Value::from(room)),
                ("race_id".to_string(), Value::from(race)),
                ("story_id".to_string(), Value::from(story)),
            ];
            values.extend(fixed(&spec.stats));
            for (column, value) in &spec.columns {
                match values.iter_mut().find(|(key, _)| key == column) {
                    Some(slot) => slot.1 = value.clone(),
                    None => values.push((column.clone(), value.clone())),
                }
            }
            let id = insert(&conn, "characters", &values)?;
            foes.push(Body {
                id,
                name,
                stats: spec.stats.clone(),
            });
        }
        let image = conn.serialize(MAIN_DB).map_err(failed)?.to_vec();
        Ok(Arena {
            image,
            story,
            room,
            exit,
            exit_name,
            player: Body {
                id: player,
                name: String::new(),
                stats: matchup.player.clone(),
            },
            foes,
            carrying,
        })
    }

    /// A fresh copy of the arena for one seed: its rolled numbers thrown,
    /// and the next playthrough to start numbered for the seed.
    pub fn fresh(&self, seed: i64) -> Result<Engine, String> {
        let failed = |error: rusqlite::Error| format!("seed {seed}: the arena: {error}");
        let mut conn = Connection::open_in_memory().map_err(failed)?;
        conn.deserialize_read_exact(MAIN_DB, &self.image[..], self.image.len(), false)
            .map_err(failed)?;
        for body in self.bodies() {
            let rolled = rolled(&body.stats, self.roll(seed, body.id));
            update(&conn, "characters", body.id, &rolled)?;
        }
        conn.execute(
            "UPDATE sqlite_sequence SET seq = ?1 WHERE name = 'playthroughs'",
            [SEED_BASE + seed - 1],
        )
        .map_err(failed)?;
        Engine::from_connection(conn).map_err(|error| format!("seed {seed}: {error}"))
    }

    fn bodies(&self) -> impl Iterator<Item = &Body> {
        std::iter::once(&self.player).chain(&self.foes)
    }

    /// The engine's own roll of a body's stat block for one seed.
    fn roll(&self, seed: i64, body: i64) -> StatBlock {
        let place = self.bodies().position(|b| b.id == body).unwrap_or_default();
        stat_block::roll(self.story, seed, place as i64)
    }

    /// A foe's hit die in the fight a seed plays.
    pub fn hit_die_of(&self, name: &str, seed: i64) -> i64 {
        self.foes
            .iter()
            .find(|foe| foe.name == name)
            .map(|foe| match foe.stats.hit_die {
                Stat::Fixed(die) => die,
                Stat::Rolled => self.roll(seed, foe.id).hit_die,
            })
            .unwrap_or_default()
    }

    /// The thing is carried and heals.
    pub fn heals(&self, name: &str) -> bool {
        self.carrying
            .iter()
            .any(|(thing, heals)| thing == name && *heals)
    }

    /// The thing is carried and does not heal, so it is for throwing.
    pub fn throws(&self, name: &str) -> bool {
        self.carrying
            .iter()
            .any(|(thing, heals)| thing == name && !*heals)
    }

    /// The blows this game has struck since the one numbered `seen`, which
    /// moves to the last.
    pub fn blows(&self, engine: &Engine, game: i64, seen: &mut i64) -> Result<Vec<Blow>, String> {
        let mut statement = engine
            .store()
            .connection()
            .prepare_cached(
                "SELECT b.id, a.fullname, t.fullname, b.damage, b.hp_after, b.attacker_id, \
                 b.target_id FROM playthrough_blows b \
                 JOIN characters a ON a.id = b.attacker_id \
                 JOIN characters t ON t.id = b.target_id \
                 WHERE b.playthrough_id = ?1 AND b.id > ?2 ORDER BY b.id",
            )
            .map_err(|error| error.to_string())?;
        let player = self.player.id;
        let rows = statement
            .query_map(params![game, *seen], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    Blow {
                        attacker: row.get(1)?,
                        target: row.get(2)?,
                        damage: row.get(3)?,
                        hp_after: row.get(4)?,
                        by_player: row.get::<_, i64>(5)? == player,
                        on_player: row.get::<_, i64>(6)? == player,
                    },
                ))
            })
            .and_then(|rows| rows.collect::<Result<Vec<_>, _>>())
            .map_err(|error| error.to_string())?;
        if let Some((last, _)) = rows.last() {
            *seen = *last;
        }
        Ok(rows.into_iter().map(|(_, blow)| blow).collect())
    }

    /// Every foe's hit points in a game, by name, the dead at zero.
    pub fn foes_hp(&self, engine: &Engine, game: i64) -> Result<Vec<(String, i64)>, String> {
        let conn = engine.store().connection();
        self.foes
            .iter()
            .map(|foe| {
                let current: Option<i64> = conn
                    .query_row(
                        "SELECT hp_current FROM playthrough_vitals \
                         WHERE playthrough_id = ?1 AND character_id = ?2",
                        [game, foe.id],
                        |row| row.get(0),
                    )
                    .ok();
                let hp = match current {
                    Some(hp) => hp,
                    None => {
                        let (level, hit_die): (i64, i64) = conn
                            .query_row(
                                "SELECT level, hit_die FROM characters WHERE id = ?1",
                                [foe.id],
                                |row| Ok((row.get(0)?, row.get(1)?)),
                            )
                            .map_err(|error| error.to_string())?;
                        let mut row = Map::new();
                        row.insert("level".into(), Value::from(level));
                        row.insert("hit_die".into(), Value::from(hit_die));
                        crate::playthrough::max_hp(&row).unwrap_or_default()
                    }
                };
                Ok((foe.name.clone(), hp))
            })
            .collect()
    }
}

/// A stat block's fixed numbers as columns.
fn fixed(stats: &Stats) -> Vec<(String, Value)> {
    stats
        .columns()
        .into_iter()
        .filter_map(|(column, stat)| match stat {
            Stat::Fixed(n) => Some((column.to_string(), Value::from(n))),
            Stat::Rolled => None,
        })
        .collect()
}

/// A stat block's rolled numbers as columns, taken from one roll.
fn rolled(stats: &Stats, roll: StatBlock) -> Vec<(String, Value)> {
    let thrown = [
        roll.level,
        roll.hit_die,
        roll.strength,
        roll.dexterity,
        roll.will,
    ];
    stats
        .columns()
        .into_iter()
        .zip(thrown)
        .filter(|((_, stat), _)| *stat == Stat::Rolled)
        .map(|((column, _), n)| (column.to_string(), Value::from(n)))
        .collect()
}

fn quoted(column: &str) -> String {
    format!("\"{}\"", column.replace('"', "\"\""))
}

/// A JSON value as SQLite stores it: a flag as 0 or 1.
fn sql(value: &Value) -> rusqlite::types::Value {
    use rusqlite::types::Value as Sql;
    match value {
        Value::Bool(flag) => Sql::Integer(i64::from(*flag)),
        Value::Number(n) => n
            .as_i64()
            .map(Sql::Integer)
            .unwrap_or_else(|| Sql::Real(n.as_f64().unwrap_or_default())),
        Value::String(text) => Sql::Text(text.clone()),
        _ => Sql::Null,
    }
}

fn update(
    conn: &Connection,
    table: &str,
    id: i64,
    values: &[(String, Value)],
) -> Result<(), String> {
    if values.is_empty() {
        return Ok(());
    }
    let set: Vec<String> = values
        .iter()
        .enumerate()
        .map(|(n, (column, _))| format!("{} = ?{}", quoted(column), n + 1))
        .collect();
    let mut bound: Vec<rusqlite::types::Value> = values.iter().map(|(_, v)| sql(v)).collect();
    bound.push(rusqlite::types::Value::Integer(id));
    let sql_text = format!(
        "UPDATE {table} SET {} WHERE id = ?{}",
        set.join(", "),
        values.len() + 1
    );
    let refs: Vec<&dyn ToSql> = bound.iter().map(|v| v as &dyn ToSql).collect();
    conn.execute(&sql_text, refs.as_slice())
        .map(|_| ())
        .map_err(|error| format!("the arena: {table}: {error}"))
}

fn insert(conn: &Connection, table: &str, values: &[(String, Value)]) -> Result<i64, String> {
    let columns: Vec<String> = values.iter().map(|(column, _)| quoted(column)).collect();
    let marks: Vec<String> = (1..=values.len()).map(|n| format!("?{n}")).collect();
    let bound: Vec<rusqlite::types::Value> = values.iter().map(|(_, v)| sql(v)).collect();
    let refs: Vec<&dyn ToSql> = bound.iter().map(|v| v as &dyn ToSql).collect();
    conn.execute(
        &format!(
            "INSERT INTO {table} ({}) VALUES ({})",
            columns.join(", "),
            marks.join(", ")
        ),
        refs.as_slice(),
    )
    .map_err(|error| format!("the arena: {table}: {error}"))?;
    Ok(conn.last_insert_rowid())
}
