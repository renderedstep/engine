//! A matchup file: who fights whom, how the player plays, and which seeds.
//!
//! ```yaml
//! seeds: 1..2000          # playthrough seeds; or a count, for 1..count
//! driver: told            # told (default) | mechanics
//! threads: 4             # default: every core; the fights are the same
//! story: {}               # columns of the arena's story, set as given
//! player:
//!   stats: {level: 3, hit_die: 8, strength: 10, dexterity: rolled, will: 10}
//!   carrying: [{name: healing draught, use_kind: healing}, {name: brick, bulk: heavy}]
//! foes:
//!   - {count: 2, stats: {level: 2, hit_die: 10}}
//! policy: {kind: flee_below, hp: 6, drink_below: 9}
//! ```
//!
//! The keys are a world file's: a stat block is pasted from one as it
//! stands. `rolled` takes that number from the engine's own roll
//! (`stat_block::roll`), thrown for each seed. Any other key of a foe, a
//! carried thing or the story is a column of its row and is written as
//! given, so a column a later migration adds needs no change here.

use super::policy::Policy;
use super::Driver;
use serde_json::Value;
use std::path::Path;
use yaml_rust2::{Yaml, YamlLoader};

/// One number of a stat block.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stat {
    Fixed(i64),
    /// Thrown for each seed by `stat_block::roll`.
    Rolled,
}

/// A stat block, in `stat_block::ABILITIES` order after level and hit die.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stats {
    pub level: Stat,
    pub hit_die: Stat,
    pub strength: Stat,
    pub dexterity: Stat,
    pub will: Stat,
}

impl Stats {
    /// The house body: level 3, d8, every ability 10.
    pub fn player() -> Stats {
        Stats {
            level: Stat::Fixed(crate::stat_block::PROTAGONIST_LEVEL),
            hit_die: Stat::Fixed(crate::stat_block::PROTAGONIST_HIT_DIE),
            strength: Stat::Fixed(10),
            dexterity: Stat::Fixed(10),
            will: Stat::Fixed(10),
        }
    }

    /// A foe with nothing said: level 1, d8, every ability 10.
    pub fn foe() -> Stats {
        Stats {
            level: Stat::Fixed(crate::stat_block::STARTING_LEVEL),
            ..Stats::player()
        }
    }

    /// The columns, in order.
    pub fn columns(&self) -> [(&'static str, Stat); 5] {
        [
            ("level", self.level),
            ("hit_die", self.hit_die),
            ("strength", self.strength),
            ("dexterity", self.dexterity),
            ("will", self.will),
        ]
    }

    fn read(defaults: Stats, yaml: &Yaml, at: &str) -> Result<Stats, String> {
        let mut stats = defaults;
        if yaml.is_badvalue() || yaml.is_null() {
            return Ok(stats);
        }
        let map = yaml
            .as_hash()
            .ok_or_else(|| format!("{at}.stats is not a map"))?;
        for (key, value) in map {
            let key = key.as_str().unwrap_or_default();
            let stat = match value {
                Yaml::Integer(n) => Stat::Fixed(*n),
                Yaml::String(word) if word == "rolled" => Stat::Rolled,
                _ => return Err(format!("{at}.stats.{key} is a whole number or \"rolled\"")),
            };
            let slot = match key {
                "level" => &mut stats.level,
                "hit_die" => &mut stats.hit_die,
                "strength" => &mut stats.strength,
                "dexterity" => &mut stats.dexterity,
                "will" => &mut stats.will,
                _ => return Err(format!("{at}.stats.{key} is not a stat")),
            };
            *slot = stat;
        }
        Ok(stats)
    }
}

/// A run of identical foes.
#[derive(Clone, Debug, PartialEq)]
pub struct Foes {
    pub count: usize,
    /// Each one's full name is this and a letter: `Foe A`, `Foe B`.
    pub name: String,
    pub stats: Stats,
    /// Other columns of each one's `characters` row.
    pub columns: Vec<(String, Value)>,
}

/// One fight's setup, and how many times to play it.
#[derive(Clone, Debug, PartialEq)]
pub struct Matchup {
    pub name: String,
    pub seeds: Vec<i64>,
    pub driver: Driver,
    pub threads: usize,
    /// Columns of the arena's `stories` row.
    pub story: Vec<(String, Value)>,
    pub player: Stats,
    /// Each carried thing's `items` columns; `name` is required.
    pub carrying: Vec<Vec<(String, Value)>>,
    pub foes: Vec<Foes>,
    pub policy: Policy,
}

impl Matchup {
    /// One player at the house body against `foes`, attacking to the end,
    /// on the told loop, over seeds 1 to 1000, on every core.
    pub fn new(name: &str, foes: Vec<Foes>) -> Matchup {
        Matchup {
            name: name.to_string(),
            seeds: (1..=1000).collect(),
            driver: Driver::Told,
            threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
            story: Vec::new(),
            player: Stats::player(),
            carrying: Vec::new(),
            foes,
            policy: Policy {
                kind: "attack".into(),
                ..Policy::default()
            },
        }
    }

    pub fn load(path: &Path) -> Result<Matchup, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("matchup");
        Matchup::parse(name, &source)
    }

    /// Reads a matchup from its YAML text.
    pub fn parse(name: &str, source: &str) -> Result<Matchup, String> {
        Matchup::read(name, &document(name, source)?)
    }

    /// Reads a matchup from its YAML document. A `sweep:` key is not read
    /// here: `sweep::grid` sets each of its points and reads every one.
    pub fn read(name: &str, document: &Yaml) -> Result<Matchup, String> {
        let inner = |document: &Yaml| -> Result<Matchup, String> {
            let mut matchup = Matchup::new(name, Vec::new());
            if !document["seeds"].is_badvalue() {
                matchup.seeds = seeds(&document["seeds"])?;
            }
            if let Some(driver) = document["driver"].as_str() {
                matchup.driver = Driver::parse(driver)
                    .ok_or_else(|| format!("driver {driver:?} is told or mechanics"))?;
            }
            if let Some(threads) = document["threads"].as_i64() {
                matchup.threads = usize::try_from(threads).map_err(|_| "threads < 0")?;
            }
            matchup.story = columns(&document["story"], "story", &[])?;
            let player = &document["player"];
            matchup.player = Stats::read(Stats::player(), &player["stats"], "player")?;
            if let Some(things) = player["carrying"].as_vec() {
                for (n, thing) in things.iter().enumerate() {
                    let at = format!("player.carrying.{n}");
                    let thing = columns(thing, &at, &[])?;
                    if !thing.iter().any(|(key, _)| key == "name") {
                        return Err(format!("{at} has no name"));
                    }
                    matchup.carrying.push(thing);
                }
            }
            let foes = document["foes"].as_vec().ok_or("foes is not a list")?;
            for (n, foe) in foes.iter().enumerate() {
                let at = format!("foes.{n}");
                let count = foe["count"].as_i64().unwrap_or(1);
                matchup.foes.push(Foes {
                    count: usize::try_from(count).map_err(|_| format!("{at}.count < 0"))?,
                    name: foe["name"].as_str().unwrap_or("Foe").to_string(),
                    stats: Stats::read(Stats::foe(), &foe["stats"], &at)?,
                    columns: columns(foe, &at, &["count", "name", "stats"])?,
                });
            }
            if !document["policy"].is_badvalue() {
                matchup.policy = Policy::read(&document["policy"])?;
            }
            Ok(matchup)
        };
        inner(document).map_err(|error| format!("{name}: {error}"))
    }

    /// Every foe's full name, in the order their rows are written.
    pub fn foe_names(&self) -> Vec<String> {
        let mut letters = 0..;
        self.foes
            .iter()
            .flat_map(|foes| {
                let name = foes.name.clone();
                (0..foes.count).map(move |_| name.clone())
            })
            .map(|name| format!("{name} {}", letter(letters.next().unwrap_or_default())))
            .collect()
    }
}

/// A matchup file's YAML document.
pub fn document(name: &str, source: &str) -> Result<Yaml, String> {
    YamlLoader::load_from_str(source)
        .map_err(|error| format!("{name}: {error}"))?
        .into_iter()
        .next()
        .ok_or_else(|| format!("{name}: an empty file"))
}

/// A, B, ... Z, then AA, AB: a name the grammar cannot read as a number.
fn letter(n: usize) -> String {
    let alphabet = b'A'..=b'Z';
    let width = alphabet.len();
    let mut n = n;
    let mut name = Vec::new();
    loop {
        name.insert(0, b'A' + (n % width) as u8);
        if n < width {
            break;
        }
        n = n / width - 1;
    }
    String::from_utf8(name).unwrap_or_default()
}

/// `A..B` (inclusive), `A..=B`, or a count `N` for `1..N`.
pub fn seeds(yaml: &Yaml) -> Result<Vec<i64>, String> {
    match yaml {
        Yaml::Integer(count) => Ok((1..=*count).collect()),
        Yaml::String(range) => parse_seeds(range),
        _ => Err("seeds is a range A..B or a count".into()),
    }
}

/// `A..B` (inclusive), `A..=B`, or a count `N` for `1..N`.
pub fn parse_seeds(range: &str) -> Result<Vec<i64>, String> {
    let bad = || format!("seeds {range:?} is a range A..B or a count");
    let seeds: Vec<i64> = match range.split_once("..") {
        Some((from, to)) => {
            let from: i64 = from.trim().parse().map_err(|_| bad())?;
            let to: i64 = to
                .trim_start_matches('=')
                .trim()
                .parse()
                .map_err(|_| bad())?;
            (from..=to).collect()
        }
        None => (1..=range.trim().parse::<i64>().map_err(|_| bad())?).collect(),
    };
    if seeds.is_empty() || seeds[0] < 1 {
        return Err(format!("seeds {range:?} must start at 1 or more"));
    }
    Ok(seeds)
}

/// A map's keys as columns, less `skip`.
fn columns(yaml: &Yaml, at: &str, skip: &[&str]) -> Result<Vec<(String, Value)>, String> {
    if yaml.is_badvalue() || yaml.is_null() {
        return Ok(Vec::new());
    }
    let map = yaml.as_hash().ok_or_else(|| format!("{at} is not a map"))?;
    let mut out = Vec::new();
    for (key, value) in map {
        let key = key
            .as_str()
            .ok_or_else(|| format!("{at} has a key that is not a word"))?;
        if skip.contains(&key) {
            continue;
        }
        let value = match value {
            Yaml::Integer(n) => Value::from(*n),
            Yaml::Real(text) => text
                .parse::<f64>()
                .map(Value::from)
                .map_err(|_| format!("{at}.{key} is not a number"))?,
            Yaml::String(text) => Value::from(text.clone()),
            Yaml::Boolean(flag) => Value::from(*flag),
            Yaml::Null => Value::Null,
            _ => return Err(format!("{at}.{key} is not one value")),
        };
        out.push((key.to_string(), value));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_matchup() {
        let matchup = Matchup::parse(
            "m",
            "seeds: 5..7\ndriver: mechanics\nplayer:\n  stats: {level: 4, dexterity: rolled}\n  \
             carrying: [{name: draught, use_kind: healing}]\nfoes:\n  - {count: 2, stats: {hit_die: 10}, hostile: true}\n\
             policy: {kind: flee_below, hp: 6}\n",
        )
        .unwrap();
        assert_eq!(matchup.seeds, vec![5, 6, 7]);
        assert_eq!(matchup.driver, Driver::Mechanics);
        assert_eq!(matchup.player.level, Stat::Fixed(4));
        assert_eq!(matchup.player.dexterity, Stat::Rolled);
        assert_eq!(matchup.foes[0].count, 2);
        assert_eq!(matchup.foes[0].stats.hit_die, Stat::Fixed(10));
        assert_eq!(matchup.foes[0].stats.level, Stat::Fixed(1));
        assert_eq!(
            matchup.foes[0].columns,
            vec![("hostile".to_string(), Value::from(true))]
        );
        assert_eq!(matchup.foe_names(), vec!["Foe A", "Foe B"]);
        assert_eq!(matchup.policy.flee_below, Some(6));
    }

    #[test]
    fn names_past_z() {
        assert_eq!(letter(0), "A");
        assert_eq!(letter(25), "Z");
        assert_eq!(letter(26), "AA");
        assert_eq!(letter(27), "AB");
    }

    #[test]
    fn refuses_a_seed_below_one() {
        assert!(parse_seeds("0..3").is_err());
        assert_eq!(parse_seeds("3").unwrap(), vec![1, 2, 3]);
        assert_eq!(parse_seeds("2..=3").unwrap(), vec![2, 3]);
    }
}
