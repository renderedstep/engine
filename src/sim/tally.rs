//! What many fights of one matchup did, counted: how often the player won,
//! died, got away or was still fighting, with a 95% interval on each; how
//! many rounds a fight took and how many hit points the survivors kept; and
//! the damage and blows on each side.

use super::{End, Fight, Line};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt::Write;

/// Fights of one matchup, counted.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub fights: usize,
    /// How many fights ended each way, in [`End::ALL`] order.
    pub ends: Vec<(End, usize)>,
    /// Rounds a fight took, and how many fights took that many.
    pub rounds: BTreeMap<usize, usize>,
    /// Hit points the player kept, over the fights they survived.
    pub hp_left: BTreeMap<i64, usize>,
    pub mean_rounds: f64,
    pub mean_hp_left: Option<f64>,
    pub mean_damage_taken: f64,
    pub mean_damage_dealt: f64,
    pub mean_blows_taken: f64,
    pub mean_blows_struck: f64,
    pub mean_drinks: f64,
    pub mean_throws: f64,
}

impl Summary {
    pub fn of(fights: &[Fight]) -> Summary {
        let n = fights.len();
        let mean = |f: &dyn Fn(&Fight) -> f64| {
            if n == 0 {
                0.0
            } else {
                fights.iter().map(f).sum::<f64>() / n as f64
            }
        };
        let mut rounds = BTreeMap::new();
        let mut hp_left = BTreeMap::new();
        for fight in fights {
            *rounds.entry(fight.lines).or_insert(0) += 1;
            if fight.end != End::Died {
                *hp_left.entry(fight.hp_left).or_insert(0) += 1;
            }
        }
        let survivors: usize = hp_left.values().sum();
        Summary {
            fights: n,
            ends: End::ALL
                .iter()
                .map(|end| (*end, fights.iter().filter(|f| f.end == *end).count()))
                .collect(),
            mean_rounds: mean(&|f| f.lines as f64),
            mean_hp_left: (survivors > 0).then(|| {
                hp_left
                    .iter()
                    .map(|(hp, k)| (*hp * *k as i64) as f64)
                    .sum::<f64>()
                    / survivors as f64
            }),
            rounds,
            hp_left,
            mean_damage_taken: mean(&|f| f.damage_taken as f64),
            mean_damage_dealt: mean(&|f| f.damage_dealt as f64),
            mean_blows_taken: mean(&|f| f.blows_taken as f64),
            mean_blows_struck: mean(&|f| f.blows_struck as f64),
            mean_drinks: mean(&|f| f.drinks as f64),
            mean_throws: mean(&|f| f.throws as f64),
        }
    }

    /// How many fights ended this way.
    pub fn count(&self, end: End) -> usize {
        self.ends
            .iter()
            .find(|(e, _)| *e == end)
            .map_or(0, |(_, k)| *k)
    }

    /// The share of fights that ended this way.
    pub fn rate(&self, end: End) -> f64 {
        if self.fights == 0 {
            0.0
        } else {
            self.count(end) as f64 / self.fights as f64
        }
    }

    /// The table a terminal prints.
    pub fn text(&self, heading: &str) -> String {
        let mut out = String::new();
        let _ = writeln!(out, "{heading}");
        let _ = writeln!(out, "  {:<9}{:>7}   95% interval", "", "rate");
        for (end, count) in &self.ends {
            if *end == End::Stalled && *count == 0 {
                continue;
            }
            let (low, high) = wilson(*count, self.fights);
            let _ = writeln!(
                out,
                "  {:<9}{:>7.3}   [{low:.3}, {high:.3}]",
                end.as_str(),
                self.rate(*end)
            );
        }
        let _ = writeln!(
            out,
            "  rounds         mean {:.2}   {}",
            self.mean_rounds,
            histogram(&self.rounds)
        );
        match self.mean_hp_left {
            Some(mean) => {
                let _ = writeln!(
                    out,
                    "  hp left        mean {mean:.2}   {} (survivors)",
                    histogram(&self.hp_left)
                );
            }
            None => {
                let _ = writeln!(out, "  hp left        (nobody survived)");
            }
        }
        let _ = writeln!(
            out,
            "  damage         taken {:.2}, dealt {:.2} a fight",
            self.mean_damage_taken, self.mean_damage_dealt
        );
        let _ = writeln!(
            out,
            "  blows          taken {:.2}, struck {:.2} a fight",
            self.mean_blows_taken, self.mean_blows_struck
        );
        if self.mean_drinks > 0.0 || self.mean_throws > 0.0 {
            let _ = writeln!(
                out,
                "  used           drinks {:.2}, throws {:.2} a fight",
                self.mean_drinks, self.mean_throws
            );
        }
        out
    }

    /// The same, as JSON.
    pub fn json(&self) -> Value {
        let ends: serde_json::Map<String, Value> = self
            .ends
            .iter()
            .map(|(end, count)| {
                let (low, high) = wilson(*count, self.fights);
                (
                    end.as_str().to_string(),
                    json!({"count": count, "rate": self.rate(*end), "low": low, "high": high}),
                )
            })
            .collect();
        json!({
            "fights": self.fights,
            "ends": ends,
            "rounds": {"mean": self.mean_rounds, "histogram": keyed(&self.rounds)},
            "hp_left": {"mean": self.mean_hp_left, "histogram": keyed(&self.hp_left)},
            "damage": {"taken": self.mean_damage_taken, "dealt": self.mean_damage_dealt},
            "blows": {"taken": self.mean_blows_taken, "struck": self.mean_blows_struck},
            "used": {"drinks": self.mean_drinks, "throws": self.mean_throws},
        })
    }
}

/// The Wilson score interval at 95% for `k` of `n`.
pub fn wilson(k: usize, n: usize) -> (f64, f64) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let z = 1.96_f64;
    let n = n as f64;
    let p = k as f64 / n;
    let centre = p + z * z / (2.0 * n);
    let spread = z * (p * (1.0 - p) / n + z * z / (4.0 * n * n)).sqrt();
    let denominator = 1.0 + z * z / n;
    (
        ((centre - spread) / denominator).max(0.0),
        ((centre + spread) / denominator).min(1.0),
    )
}

/// `1:12 2:40 3:7`.
fn histogram<K: std::fmt::Display>(counts: &BTreeMap<K, usize>) -> String {
    counts
        .iter()
        .map(|(key, count)| format!("{key}:{count}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A histogram as a JSON object, keyed by its numbers as text.
fn keyed<K: std::fmt::Display>(counts: &BTreeMap<K, usize>) -> Value {
    Value::Object(
        counts
            .iter()
            .map(|(key, count)| (key.to_string(), Value::from(*count)))
            .collect(),
    )
}

/// One fight, line by line, for `--replay`.
pub fn replay(fight: &Fight) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "seed {}: {} after {} line(s), {} of {} hp left",
        fight.seed,
        fight.end.as_str(),
        fight.lines,
        fight.hp_left,
        fight.max_hp
    );
    for (n, line) in fight.log.iter().enumerate() {
        let Line {
            typed,
            blows,
            hp,
            foes,
        } = line;
        let _ = writeln!(out, "{:>3}. > {typed}", n + 1);
        for blow in blows {
            let _ = writeln!(
                out,
                "       {} strikes {} for {} ({} left)",
                blow.attacker, blow.target, blow.damage, blow.hp_after
            );
        }
        let foes: Vec<String> = foes
            .iter()
            .map(|(name, hp)| format!("{name} {hp}"))
            .collect();
        let _ = writeln!(out, "       you {hp}; {}", foes.join(", "));
    }
    out
}

/// One fight, line by line, as JSON.
pub fn replay_json(fight: &Fight) -> Value {
    json!({
        "seed": fight.seed,
        "end": fight.end.as_str(),
        "lines": fight.lines,
        "max_hp": fight.max_hp,
        "hp_left": fight.hp_left,
        "damage": {"taken": fight.damage_taken, "dealt": fight.damage_dealt},
        "log": fight.log.iter().map(|line| json!({
            "typed": line.typed,
            "blows": line.blows.iter().map(|blow| json!({
                "attacker": blow.attacker,
                "target": blow.target,
                "damage": blow.damage,
                "hp_after": blow.hp_after,
            })).collect::<Vec<_>>(),
            "hp": line.hp,
            "foes": line.foes.iter().map(|(name, hp)| json!({"name": name, "hp": hp})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_interval() {
        let (low, high) = wilson(460, 2000);
        assert!((low - 0.2122).abs() < 0.001, "{low}");
        assert!((high - 0.2491).abs() < 0.001, "{high}");
        assert_eq!(wilson(0, 0), (0.0, 1.0));
    }
}
