//! What a grid of matchups did, for reading elsewhere: a table a row a
//! matchup for a terminal, the same as CSV for a spreadsheet, and the fight
//! table a dungeon-level study reads instead of copying the fight rules
//! (each matchup's chance to win and how many hit points a fight costs).

use super::sweep::Point;
use super::tally::{wilson, Summary};
use super::{End, Fight};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write;

/// One matchup of a grid, played.
pub struct Played<'a> {
    pub point: &'a Point,
    pub fights: Vec<Fight>,
}

impl Played<'_> {
    fn summary(&self) -> Summary {
        Summary::of(&self.fights)
    }
}

/// Every swept column, in the order the points first name them.
fn keys(played: &[Played]) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for one in played {
        for (key, _) in &one.point.columns {
            if !keys.contains(key) {
                keys.push(key.clone());
            }
        }
    }
    keys
}

fn value<'a>(point: &'a Point, key: &str) -> &'a str {
    point
        .columns
        .iter()
        .find(|(k, _)| k == key)
        .map_or("", |(_, v)| v.as_str())
}

/// A row a matchup, for a terminal.
pub fn text(played: &[Played]) -> String {
    let keys = keys(played);
    // Each swept value under its key, padded to the wider of the two.
    let widths: Vec<usize> = keys
        .iter()
        .map(|key| {
            played
                .iter()
                .map(|one| value(one.point, key).len())
                .chain([key.len()])
                .max()
                .unwrap_or_default()
        })
        .collect();
    let line = |cells: Vec<&str>| {
        cells
            .iter()
            .zip(&widths)
            .map(|(cell, width)| format!("{cell:<width$}"))
            .collect::<Vec<_>>()
            .join("  ")
    };
    let labels: Vec<String> = played
        .iter()
        .map(|one| {
            if keys.is_empty() {
                one.point.name.clone()
            } else {
                line(keys.iter().map(|key| value(one.point, key)).collect())
            }
        })
        .collect();
    let heading = if keys.is_empty() {
        "matchup".to_string()
    } else {
        line(keys.iter().map(String::as_str).collect())
    };
    let width = labels
        .iter()
        .map(String::len)
        .chain([heading.len()])
        .max()
        .unwrap_or_default();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{heading:<width$}  {:>6}  {:>6}  won 95%          {:>5}  {:>5}  {:>6}  {:>7}",
        "fights", "won", "died", "fled", "rounds", "hp left"
    );
    for (label, one) in labels.iter().zip(played) {
        let summary = one.summary();
        let (low, high) = wilson(summary.count(End::Won), summary.fights);
        let hp_left = summary
            .mean_hp_left
            .map_or("-".to_string(), |hp| format!("{hp:.2}"));
        let _ = writeln!(
            out,
            "{label:<width$}  {:>6}  {:>6.3}  [{low:.3}, {high:.3}]  {:>5.3}  {:>5.3}  {:>6.2}  {hp_left:>7}",
            summary.fights,
            summary.rate(End::Won),
            summary.rate(End::Died),
            summary.rate(End::Fled),
            summary.mean_rounds,
        );
    }
    out
}

/// Every matchup's [`Summary::json`], with its name and swept values.
pub fn json(played: &[Played]) -> Value {
    Value::Array(
        played
            .iter()
            .map(|one| {
                let mut out = one.summary().json();
                out["matchup"] = one.point.name.clone().into();
                out["columns"] = columns(one.point);
                out
            })
            .collect(),
    )
}

fn columns(point: &Point) -> Value {
    Value::Object(
        point
            .columns
            .iter()
            .map(|(key, value)| (key.clone(), Value::from(value.clone())))
            .collect(),
    )
}

/// A row a matchup, as CSV: the swept values, then each way a fight ended
/// with its interval, and the means.
pub fn csv(played: &[Played]) -> String {
    let keys = keys(played);
    let mut header: Vec<String> = vec!["matchup".into()];
    header.extend(keys.iter().cloned());
    header.push("fights".into());
    for end in End::ALL {
        for part in ["", "_low", "_high"] {
            header.push(format!("{}{part}", end.as_str()));
        }
    }
    header.extend(
        [
            "mean_rounds",
            "mean_hp_left",
            "damage_taken",
            "damage_dealt",
            "blows_taken",
            "blows_struck",
            "drinks",
            "throws",
        ]
        .map(String::from),
    );
    let mut out = String::new();
    let _ = writeln!(out, "{}", row(&header));
    for one in played {
        let summary = one.summary();
        let mut fields: Vec<String> = vec![one.point.name.clone()];
        fields.extend(keys.iter().map(|key| value(one.point, key).to_string()));
        fields.push(summary.fights.to_string());
        for end in End::ALL {
            let (low, high) = wilson(summary.count(end), summary.fights);
            fields.push(format!("{:.4}", summary.rate(end)));
            fields.push(format!("{low:.4}"));
            fields.push(format!("{high:.4}"));
        }
        fields.push(format!("{:.4}", summary.mean_rounds));
        fields.push(
            summary
                .mean_hp_left
                .map_or(String::new(), |hp| format!("{hp:.4}")),
        );
        for mean in [
            summary.mean_damage_taken,
            summary.mean_damage_dealt,
            summary.mean_blows_taken,
            summary.mean_blows_struck,
            summary.mean_drinks,
            summary.mean_throws,
        ] {
            fields.push(format!("{mean:.4}"));
        }
        let _ = writeln!(out, "{}", row(&fields));
    }
    out
}

fn row(fields: &[String]) -> String {
    fields
        .iter()
        .map(|field| {
            if field.contains([',', '"', '\n', '\r']) {
                format!("\"{}\"", field.replace('"', "\"\""))
            } else {
                field.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(",")
}

/// The fight table: for each matchup, the chance the player wins, dies, gets
/// away or is still fighting, and the hit points a fight costs them. Its
/// `hp_loss` is the share of all fights that cost the player that many hit
/// points and left them alive (max less what was left); with `died` it sums
/// to one, so a study draws one fight's cost from it whole.
pub fn fight_table(played: &[Played]) -> Value {
    let matchups: Vec<Value> = played
        .iter()
        .map(|one| {
            let summary = one.summary();
            let n = summary.fights.max(1) as f64;
            let mut loss: BTreeMap<i64, usize> = BTreeMap::new();
            for fight in one.fights.iter().filter(|fight| fight.end != End::Died) {
                *loss.entry(fight.max_hp - fight.hp_left).or_insert(0) += 1;
            }
            let (low, high) = wilson(summary.count(End::Won), summary.fights);
            let matchup = &one.point.matchup;
            json!({
                "matchup": one.point.name,
                "columns": columns(one.point),
                "driver": matchup.driver.as_str(),
                "policy": matchup.policy.kind,
                "fights": summary.fights,
                "p_win": summary.rate(End::Won),
                "p_win_low": low,
                "p_win_high": high,
                "p_died": summary.rate(End::Died),
                "p_fled": summary.rate(End::Fled),
                "p_stalled": summary.rate(End::Stalled),
                "mean_rounds": summary.mean_rounds,
                "hp_loss": Value::Object(loss.iter().map(|(hp, k)| {
                    (hp.to_string(), Value::from(*k as f64 / n))
                }).collect()),
            })
        })
        .collect();
    json!({"fight_table": 1, "matchups": matchups})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_a_field_with_a_comma() {
        let fields = ["a".to_string(), "b, c".into(), "say \"hi\"".into()];
        assert_eq!(row(&fields), "a,\"b, c\",\"say \"\"hi\"\"\"");
    }
}
