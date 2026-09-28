//! What stands in a room and what lies about in it (`Item::Kit.roll`),
//! rolled from `data/item/kits.yml` on the room's kind and density, which a
//! model picked on the call next door (`kind`). The game reads the same
//! file and rolls the same room the same way.
//!
//! One generator, seeded on the room's name (`Roll::KIT`), thrown in one
//! order: a die per piece the kind lists; then, for each piece that came
//! up, how many things lie on or in it and a draw per thing; then how many
//! small things lie about at the room's density and a draw per thing. No
//! name is drawn twice in one room, and the list is cut at `visible`,
//! pieces first. A room with no kind, or one the table lacks, gets nothing.

use crate::data::{self, Kits};
use crate::random::Random;
use crate::roll::{self, Seed};
use crate::text::{crc32, natural_key};

/// `Item::FIXTURE` and `Item::PORTABLE`.
pub const FIXTURE: &str = "fixture";
pub const PORTABLE: &str = "portable";

/// `Item::HOLDS`: what a fixed piece holds.
pub const HOLDS: [&str; 4] = ["nothing", "top", "hollow", "closed"];

/// One thing a kit writes, in the order it is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thing {
    pub name: String,
    pub tier: &'static str,
    /// What a fixed piece holds; `None` on a portable thing.
    pub holds: Option<String>,
    pub bulk: String,
    /// The index, in the same list, of the piece it lies on or in.
    pub within: Option<usize>,
    /// `on` a top, `in` a hollow.
    pub how: Option<&'static str>,
    pub kit_key: String,
}

impl Thing {
    pub fn use_kind(&self) -> &'static str {
        data::kits()
            .use_kinds
            .iter()
            .find(|(name, _)| *name == self.name)
            .map_or("ordinary", |(_, use_kind)| use_kind.as_str())
    }

    pub fn combustible(&self) -> bool {
        data::kits().combustible.contains(&self.name)
    }

    pub fn readable(&self) -> bool {
        data::kits().readable.contains(&self.name)
    }
}

/// The generator a room's kit is thrown from.
pub fn generator_for(name: &str) -> Random {
    Seed {
        story: 0,
        sequence: crc32(natural_key(name).as_bytes()).into(),
        kind: roll::KIT,
        ..Seed::default()
    }
    .generator()
}

fn word_of<'k>(kits: &'k Kits, piece: &str) -> &'k str {
    kits.pieces
        .iter()
        .find(|(name, _)| name == piece)
        .map(|(_, word)| word.as_str())
        .expect("a piece the table has")
}

/// `count` names out of `from`, never one already `used`, drawn one at a
/// time from what is left; fewer when the pool runs dry.
fn draw(from: &[String], count: i64, used: &mut Vec<String>, rng: &mut Random) -> Vec<String> {
    let mut drawn = Vec::new();
    for _ in 0..count {
        let left: Vec<&String> = from.iter().filter(|name| !used.contains(name)).collect();
        if left.is_empty() {
            continue;
        }
        let name = left[roll::one_of(left.len(), rng)].clone();
        used.push(name.clone());
        drawn.push(name);
    }
    drawn
}

/// `Item::Kit.roll`: the room's kit, as values.
pub fn roll(name: &str, kind: Option<&str>, density: Option<&str>) -> Vec<Thing> {
    let kits = data::kits();
    let Some((kind, kit)) = kind.and_then(|kind| kits.kinds.iter().find(|(k, _)| k == kind)) else {
        return Vec::new();
    };
    let mut rng = generator_for(name);
    let pieces: Vec<&String> = kit
        .pieces
        .iter()
        .filter(|(_, share)| roll::die(kits.kit_die, &mut rng) <= *share)
        .map(|(piece, _)| piece)
        .collect();
    let mut things: Vec<Thing> = pieces
        .iter()
        .map(|piece| {
            let word = word_of(kits, piece);
            let fixed = HOLDS.contains(&word);
            Thing {
                name: (*piece).clone(),
                tier: if fixed { FIXTURE } else { PORTABLE },
                holds: fixed.then(|| word.to_string()),
                bulk: if fixed { "immovable" } else { word }.to_string(),
                within: None,
                how: None,
                kit_key: format!("{kind}/{piece}"),
            }
        })
        .collect();
    let mut used: Vec<String> = pieces.iter().map(|piece| (*piece).clone()).collect();
    for (index, piece) in pieces.iter().enumerate() {
        let Some(pool) = kits
            .holding
            .iter()
            .find(|(name, _)| name == *piece)
            .and_then(|(_, pool)| kits.pools.iter().find(|(name, _)| name == pool))
            .map(|(_, pool)| pool)
        else {
            continue;
        };
        let how = if word_of(kits, piece) == "hollow" {
            "in"
        } else {
            "on"
        };
        let count = pool.count[roll::one_of(pool.count.len(), &mut rng)];
        for thing in draw(&pool.from, count, &mut used, &mut rng) {
            things.push(Thing {
                kit_key: format!("{kind}/{piece}/{thing}"),
                name: thing,
                tier: PORTABLE,
                holds: None,
                bulk: "handy".into(),
                within: Some(index),
                how: Some(how),
            });
        }
    }
    if let Some((_, band)) = density.and_then(|word| kits.density.iter().find(|(d, _)| d == word)) {
        let count = band[roll::one_of(band.len(), &mut rng)];
        for thing in draw(&kit.loose, count, &mut used, &mut rng) {
            things.push(Thing {
                kit_key: format!("{kind}/loose/{thing}"),
                name: thing,
                tier: PORTABLE,
                holds: None,
                bulk: "handy".into(),
                within: None,
                how: None,
            });
        }
    }
    things.truncate(kits.visible);
    things
}

/// `Item::Kit#description`: one engine line, never prose.
pub fn description(thing: &Thing, within: Option<&str>) -> String {
    match (within, thing.how) {
        (Some(piece), Some(how)) => format!("The {}, {how} the {piece}.", thing.name),
        _ if thing.tier == FIXTURE => format!("The {}, fixed in place.", thing.name),
        _ => format!("The {}.", thing.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_room_with_no_kind_or_an_unknown_one_gets_nothing() {
        assert!(roll("The Reading Room", None, Some("lived-in")).is_empty());
        assert!(roll("The Reading Room", Some("ballroom"), Some("lived-in")).is_empty());
    }

    #[test]
    fn every_kind_rolls_within_its_cap_and_names_nothing_twice() {
        for (kind, _) in &data::kits().kinds {
            for n in 0..50 {
                let things = roll(&format!("room {n}"), Some(kind), Some("cluttered"));
                assert!(things.len() <= data::kits().visible);
                let mut names: Vec<&str> = things.iter().map(|t| t.name.as_str()).collect();
                names.sort_unstable();
                names.dedup();
                assert_eq!(names.len(), things.len(), "{kind}: {things:?}");
                for thing in &things {
                    if let Some(index) = thing.within {
                        assert_eq!(things[index].tier, FIXTURE);
                    }
                }
            }
        }
    }

    #[test]
    fn a_room_rolls_the_same_kit_by_its_name_whatever_its_article() {
        assert_eq!(
            roll("The Reading Room", Some("study"), Some("lived-in")),
            roll("reading   room", Some("study"), Some("lived-in"))
        );
    }
}
