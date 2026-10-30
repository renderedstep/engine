//! Many matchups from one: a matchup file's `sweep:` key, and a world's
//! own hostiles (`sim::world`), each made into a grid of [`Point`]s that are
//! played one after another and counted on their own.
//!
//! ```yaml
//! sweep: {player.stats.level: [3, 4, 5, 6], foes.0.stats.level: [1, 2, 3]}
//! ```
//!
//! Each key is a path into the file, its parts map keys or list places, and
//! each value a list of what to set there. The grid is every combination,
//! the first key outermost, and each point is the file with its values set
//! and read again as a whole, so any key a matchup reads can be swept,
//! `story` columns and `policy` options too.

use super::matchup::{self, Matchup};
use std::path::Path;
use yaml_rust2::yaml::Hash;
use yaml_rust2::Yaml;

/// One matchup of a grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    /// `one-l3-d10 player.stats.level=4 foes.0.stats.level=2`.
    pub name: String,
    /// What this point set, in the grid's key order, as written.
    pub columns: Vec<(String, String)>,
    pub matchup: Matchup,
}

impl Point {
    /// A matchup played on its own, with nothing swept.
    pub fn single(matchup: Matchup) -> Point {
        Point {
            name: matchup.name.clone(),
            columns: Vec::new(),
            matchup,
        }
    }
}

/// Every point a matchup file names: one with no `sweep:` key.
pub fn load(path: &Path) -> Result<Vec<Point>, String> {
    let source =
        std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("matchup");
    grid(name, &source)
}

/// Every point of a matchup file's text.
pub fn grid(name: &str, source: &str) -> Result<Vec<Point>, String> {
    let document = matchup::document(name, source)?;
    let sweep = &document["sweep"];
    if sweep.is_badvalue() || sweep.is_null() {
        return Ok(vec![Point::single(Matchup::read(name, &document)?)]);
    }
    let failed = |error: String| format!("{name}: sweep: {error}");
    let axes: Vec<(String, Vec<Yaml>)> = sweep
        .as_hash()
        .ok_or_else(|| failed("is not a map".into()))?
        .iter()
        .map(|(key, values)| {
            let key = key
                .as_str()
                .ok_or_else(|| failed("a key is not a path".into()))?;
            let values = values
                .as_vec()
                .filter(|values| !values.is_empty())
                .ok_or_else(|| failed(format!("{key} is not a list of values")))?;
            Ok((key.to_string(), values.clone()))
        })
        .collect::<Result<_, String>>()?;
    let mut base = document.clone();
    if let Yaml::Hash(map) = &mut base {
        map.remove(&Yaml::String("sweep".into()));
    }
    let mut points = Vec::new();
    let mut at = vec![0; axes.len()];
    loop {
        let mut document = base.clone();
        let mut columns = Vec::new();
        for ((key, values), n) in axes.iter().zip(&at) {
            set(&mut document, key, values[*n].clone()).map_err(failed)?;
            columns.push((key.clone(), written(&values[*n])));
        }
        let label: Vec<String> = columns
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect();
        let point = format!("{name} {}", label.join(" "));
        points.push(Point {
            matchup: Matchup::read(&point, &document)?,
            name: point,
            columns,
        });
        // The next combination, the last key turning fastest.
        let mut axis = axes.len();
        loop {
            if axis == 0 {
                return Ok(points);
            }
            axis -= 1;
            at[axis] += 1;
            if at[axis] < axes[axis].1.len() {
                break;
            }
            at[axis] = 0;
        }
    }
}

/// Sets the value at a dotted path, making the maps on the way that are not
/// there yet.
fn set(document: &mut Yaml, path: &str, value: Yaml) -> Result<(), String> {
    let mut here = document;
    for part in path.split('.') {
        if here.is_null() || here.is_badvalue() {
            *here = Yaml::Hash(Hash::new());
        }
        here = match here {
            Yaml::Hash(map) => map
                .entry(Yaml::String(part.to_string()))
                .or_insert(Yaml::Null),
            Yaml::Array(list) => part
                .parse::<usize>()
                .ok()
                .and_then(|n| list.get_mut(n))
                .ok_or_else(|| format!("{path}: {part} is not a place in its list"))?,
            _ => return Err(format!("{path}: {part} is inside one value, not a map")),
        };
    }
    *here = value;
    Ok(())
}

/// A swept value as it would be written in the file.
fn written(value: &Yaml) -> String {
    match value {
        Yaml::Integer(n) => n.to_string(),
        Yaml::Real(text) | Yaml::String(text) => text.clone(),
        Yaml::Boolean(flag) => flag.to_string(),
        Yaml::Null => "null".into(),
        _ => {
            let mut out = String::new();
            let _ = yaml_rust2::YamlEmitter::new(&mut out).dump(value);
            out.trim_start_matches("---").trim().replace('\n', " ")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::matchup::Stat;

    #[test]
    fn sweeps_a_grid() {
        let points = grid(
            "g",
            "seeds: 3\nfoes:\n  - {count: 2, stats: {hit_die: 10}}\n\
             sweep: {player.stats.level: [3, 4], foes.0.stats.level: [1, 2, 3]}\n",
        )
        .unwrap();
        assert_eq!(points.len(), 6);
        assert_eq!(
            points[0].name,
            "g player.stats.level=3 foes.0.stats.level=1"
        );
        assert_eq!(
            points[5].name,
            "g player.stats.level=4 foes.0.stats.level=3"
        );
        let point = &points[4].matchup;
        assert_eq!(point.player.level, Stat::Fixed(4));
        assert_eq!(point.foes[0].stats.level, Stat::Fixed(2));
        assert_eq!(point.foes[0].stats.hit_die, Stat::Fixed(10));
        assert_eq!(point.foes[0].count, 2);
        assert_eq!(point.seeds, vec![1, 2, 3]);
        assert_eq!(
            points[4].columns,
            vec![
                ("player.stats.level".to_string(), "4".to_string()),
                ("foes.0.stats.level".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn a_file_with_no_sweep_is_one_point() {
        let points = grid("one", "foes: [{stats: {level: 2}}]\n").unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].name, "one");
        assert!(points[0].columns.is_empty());
    }

    #[test]
    fn refuses_a_path_that_is_not_there() {
        let error = grid("bad", "foes: [{}]\nsweep: {foes.3.count: [1]}\n").unwrap_err();
        assert!(error.contains("foes.3.count"), "{error}");
        let error = grid("bad", "foes: [{}]\nsweep: {foes: 2}\n").unwrap_err();
        assert!(error.contains("not a list"), "{error}");
    }
}
