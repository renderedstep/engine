//! What stands in a room and what lies about in it (`Item::Kit.roll`),
//! rolled from `data/item/kits.yml` on the room's kind and density, which a
//! model picked on the call next door (`kind`). The game reads the same
//! file and rolls the same room the same way.
//!
//! One generator, seeded on the room's name (`Roll::KIT`), thrown in one
//! order: a die per piece the kind lists; then, for each piece that came
//! up, how many things lie on or in it and a draw per thing; then how many
//! small things lie about at the room's density and a draw per thing; then,
//! for each piece that came up, how many of each of its parts it was born
//! with ([`furnish`]; [`roll`] is the same list less the parts). Parts are
//! drawn last, so a room whose pieces have none rolls what it always rolled.
//! No name is drawn twice in one room, but a part (a tree's two branches),
//! and the list is cut at `visible`, pieces first. A room with no kind, or
//! one the table lacks, gets nothing.

use crate::data::{self, Kits};
use crate::random::Random;
use crate::roll::{self, Seed};
use crate::text::{crc32, natural_key};

/// `Item::FIXTURE`, `Item::PORTABLE` and `Item::PART`.
pub const FIXTURE: &str = "fixture";
pub const PORTABLE: &str = "portable";
/// Attached to a fixed piece (`how: attached`) until somebody breaks it off.
pub const PART: &str = "part";

/// `items.how` on a part.
pub const ATTACHED: &str = "attached";

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
    /// `on` a top, `in` a hollow, `attached` to the piece as a part.
    pub how: Option<&'static str>,
    pub kit_key: String,
    /// How hard a part holds on (`hold_penalty`); `None` on anything else.
    pub hold: Option<String>,
    /// A part's fragility once it comes away; `None` leaves the column's
    /// default.
    pub fragility: Option<String>,
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

    /// What it is made of (`materials`), or none where the table does not
    /// say.
    pub fn material(&self) -> Option<&'static str> {
        crate::material::of(&self.name)
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

/// `Item::Kit.roll`: the room's kit, as values, as the game rolls it: what
/// [`furnish`] writes, less the parts. The parts are drawn after everything
/// else and cut first, so this is what the roll was before parts were drawn.
pub fn roll(name: &str, kind: Option<&str>, density: Option<&str>) -> Vec<Thing> {
    furnish(name, kind, density)
        .into_iter()
        .filter(|thing| thing.tier != PART)
        .collect()
}

/// Everything a room is furnished with, its pieces' parts included, in the
/// order it is written: [`roll`]'s list, then the parts, cut at `visible`.
/// A part's `within` is its piece's index in the same list.
pub fn furnish(name: &str, kind: Option<&str>, density: Option<&str>) -> Vec<Thing> {
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
                hold: None,
                fragility: None,
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
                hold: None,
                fragility: None,
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
                hold: None,
                fragility: None,
            });
        }
    }
    for (index, piece) in pieces.iter().enumerate() {
        let Some((_, parts)) = kits.parts.iter().find(|(name, _)| name == *piece) else {
            continue;
        };
        for (part, spec) in parts {
            let count = spec.count[roll::one_of(spec.count.len(), &mut rng)];
            for _ in 0..count {
                things.push(Thing {
                    name: part.clone(),
                    tier: PART,
                    holds: None,
                    bulk: spec.bulk.clone(),
                    within: Some(index),
                    how: Some(ATTACHED),
                    kit_key: format!("{kind}/{piece}/{ATTACHED}/{part}"),
                    hold: Some(spec.hold.clone()),
                    fragility: Some(spec.fragility.clone()),
                });
            }
        }
    }
    things.truncate(kits.visible);
    things
}

/// `Item::Contents.roll`: the names lying inside a closed piece, drawn once,
/// the first time anybody searches it, from the pool `contents` names for
/// the piece. One generator, seeded on the story and the piece's own row
/// (`Roll::CONTENTS`): how many, one of the pool's count drawn, then a draw
/// per thing, never a name already lying in the room (`used`). A piece the
/// table has no inside for holds nothing.
pub fn contents(piece: &str, story: i64, fixture: i64, used: &[String]) -> Vec<String> {
    let kits = data::kits();
    let Some(pool) = kits
        .contents
        .iter()
        .find(|(name, _)| name == piece)
        .and_then(|(_, pool)| kits.pools.iter().find(|(name, _)| name == pool))
        .map(|(_, pool)| pool)
    else {
        return Vec::new();
    };
    let mut rng = Seed {
        story: story.into(),
        sequence: fixture.into(),
        kind: roll::CONTENTS,
        ..Seed::default()
    }
    .generator();
    let count = pool.count[roll::one_of(pool.count.len(), &mut rng)];
    draw(&pool.from, count, &mut used.to_vec(), &mut rng)
}

/// What a name drawn into a closed piece is, as a kit [`Thing`] lying in it,
/// for its use, its burning and its words.
pub fn inside(name: &str, piece: &str) -> Thing {
    Thing {
        name: name.to_string(),
        tier: PORTABLE,
        holds: None,
        bulk: "handy".into(),
        within: None,
        how: Some("in"),
        kit_key: format!("{piece}/inside/{name}"),
        hold: None,
        fragility: None,
    }
}

/// `hold_penalty`: what a part's hold takes off strength on the check to
/// break it off; a hold the table does not name takes nothing.
pub fn hold_penalty(hold: Option<&str>) -> i64 {
    data::kits()
        .hold_penalty
        .iter()
        .find(|(name, _)| Some(name.as_str()) == hold)
        .map_or(0, |(_, penalty)| *penalty)
}

/// The hold that takes `penalty` off strength, where the table names one:
/// what a part held by, read back off the check thrown to break it.
pub fn hold_of_penalty(penalty: i64) -> Option<&'static str> {
    data::kits()
        .hold_penalty
        .iter()
        .find(|(_, taken)| *taken == penalty)
        .map(|(name, _)| name.as_str())
}

/// `Item::Kit#description`: one engine line, never prose.
pub fn description(thing: &Thing, within: Option<&str>) -> String {
    match (within, thing.how) {
        (Some(piece), Some(ATTACHED)) => format!("The {}, attached to the {piece}.", thing.name),
        (Some(piece), Some(how)) => format!("The {}, {how} the {piece}.", thing.name),
        _ if thing.tier == FIXTURE => format!("The {}, fixed in place.", thing.name),
        _ => plain_description(&thing.name),
    }
}

/// The engine's line for a thing that lies on, in and against nothing, as
/// a part is once it has been broken off.
pub fn plain_description(name: &str) -> String {
    format!("The {name}.")
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
                let unattached: Vec<&Thing> = things.iter().filter(|t| t.tier != PART).collect();
                let mut names: Vec<&str> = unattached.iter().map(|t| t.name.as_str()).collect();
                names.sort_unstable();
                names.dedup();
                assert_eq!(names.len(), unattached.len(), "{kind}: {things:?}");
                for thing in &things {
                    if let Some(index) = thing.within {
                        assert_eq!(things[index].tier, FIXTURE);
                    }
                }
            }
        }
    }

    #[test]
    fn parts_are_drawn_last_attached_to_a_piece_that_came_up() {
        let mut seen = false;
        for n in 0..200 {
            let name = format!("clearing {n}");
            let things = furnish(&name, Some("forest"), Some("cluttered"));
            assert!(things.len() <= data::kits().visible);
            let unattached: Vec<Thing> =
                things.iter().filter(|t| t.tier != PART).cloned().collect();
            assert_eq!(unattached, roll(&name, Some("forest"), Some("cluttered")));
            let first = things.iter().position(|t| t.tier == PART);
            if let Some(first) = first {
                seen = true;
                assert!(things[first..].iter().all(|t| t.tier == PART), "{things:?}");
            }
            for part in things.iter().filter(|t| t.tier == PART) {
                let piece = &things[part.within.expect("a part is attached")];
                assert_eq!(piece.tier, FIXTURE);
                assert_eq!(part.how, Some(ATTACHED));
                assert!(part.hold.is_some() && part.fragility.is_some());
                assert_eq!(
                    part.kit_key,
                    format!("forest/{}/attached/{}", piece.name, part.name)
                );
            }
        }
        assert!(
            seen,
            "an old tree comes up with its branches in some forest"
        );
    }

    /// A room whose pieces have no parts rolls what it rolled before parts
    /// were drawn: the same things, in the same order.
    #[test]
    fn a_room_whose_pieces_have_no_parts_rolls_what_it_always_did() {
        let things = roll("The Reading Room", Some("study"), Some("lived-in"));
        let names: Vec<&str> = things.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "desk",
                "bookcase",
                "chair",
                "lamp",
                "ledger",
                "journal",
                "atlas",
                "letter opener"
            ]
        );
    }

    #[test]
    fn a_hold_takes_its_penalty_and_an_unknown_one_nothing() {
        assert_eq!(hold_penalty(Some("loose")), 0);
        assert_eq!(hold_penalty(Some("firm")), 3);
        assert_eq!(hold_penalty(Some("stout")), 6);
        assert_eq!(hold_penalty(Some("welded")), 0);
        assert_eq!(hold_penalty(None), 0);
    }

    #[test]
    fn a_closed_piece_is_filled_the_same_way_every_time_and_never_twice_by_name() {
        let used = vec!["pen".to_string()];
        for fixture in 1..200 {
            let drawn = contents("desk", 7, fixture, &used);
            assert_eq!(drawn, contents("desk", 7, fixture, &used));
            assert!(drawn.len() <= 3, "{drawn:?}");
            assert!(!drawn.contains(&"pen".to_string()));
            let mut names = drawn.clone();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), drawn.len());
        }
        assert!(contents("table", 7, 1, &[]).is_empty());
        assert!(contents("old beech", 7, 1, &[]).is_empty());
    }

    #[test]
    fn a_room_rolls_the_same_kit_by_its_name_whatever_its_article() {
        assert_eq!(
            roll("The Reading Room", Some("study"), Some("lived-in")),
            roll("reading   room", Some("study"), Some("lived-in"))
        );
    }
}
