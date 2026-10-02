//! Falls and breakage: how far the player dropped through a doorway, at the
//! gravity of the world they are in, and what the landing cost; and whether
//! a thing that came down on a floor broke.
//!
//! A fall is a doorway's hazard (`location_connections.hazard` = `fall`)
//! whose die is not on the row: the storeys between its two rooms and the
//! world's gravity (`universes.gravity`) decide how many dice are thrown,
//! out of `data/physics.yml`. A world with no gravity, or a doorway that
//! does not go down, throws nothing, so a world that says nothing about
//! gravity plays exactly as it did before falls existed.
//!
//! A break is a thing's fragility (`items.fragility`), how it came down and
//! what it landed on (`locations.surface`): together they are a share of
//! one die, and the thing breaks when the die comes up at or under it. A
//! sturdy thing throws no die at all, so a world that gives nothing a
//! fragility plays exactly as it did before things could break.
//!
//! A throw's range is how far a thing carries: its bulk, the thrower's
//! strength and the world's gravity, in paces, and no die. Where the room is
//! laid out it is held against the paces to what the thing was thrown at,
//! and a thing that falls short lands along the line of the throw; where it
//! is not, the range is only told. A world with no gravity has no range, so
//! a world that says nothing about gravity throws exactly as it did before
//! range existed.

use crate::boxes::{Axis, Box, Spot};
use crate::data;
use crate::random::Random;
use crate::records::Row;
use crate::roll;
use crate::turn::{check_ability, Check};

/// The doorway hazard that is a fall.
pub const FALL: &str = "fall";

/// One fall: the dice it threw, the save thrown before them, what the dice
/// came up and what it cost.
#[derive(Clone, Debug)]
pub struct Fall {
    pub dice: i64,
    /// The save, or none for a body with no abilities, which cannot make one.
    pub save: Option<Check>,
    pub rolled: i64,
    pub damage: i64,
}

impl Fall {
    pub fn saved(&self) -> bool {
        self.save.as_ref().is_some_and(Check::passed)
    }
}

/// `data/physics.yml`'s `gravity`: each gravity and its dice per storey.
pub fn gravities() -> &'static [(String, i64)] {
    &data::physics().gravity
}

/// The die a fall throws, and the ability that saves against it.
pub fn fall_die() -> (i64, &'static str) {
    let physics = data::physics();
    (physics.fall_die, &physics.fall_save)
}

/// How many storeys a doorway goes down, from the storey index of the room
/// it leaves to that of the room it enters: none unless both rooms have one
/// and the first is higher.
pub fn storeys(from: Option<i64>, to: Option<i64>) -> Option<i64> {
    let storeys = from? - to?;
    (storeys > 0).then_some(storeys)
}

/// The dice a fall of `storeys` throws at `gravity`: none for a world with
/// no gravity, or one the table does not name.
pub fn dice(storeys: i64, gravity: Option<&str>) -> Option<i64> {
    let gravity = gravity?;
    let per_storey = gravities()
        .iter()
        .find(|(name, _)| name == gravity)
        .map(|(_, dice)| *dice)?;
    (storeys > 0).then_some(storeys * per_storey)
}

/// A fall of `dice` dice taken by `who`: the save first, then the dice, out
/// of the one generator, in that order for ever. A passed save halves what
/// the dice came up, rounded down.
pub fn fall(dice: i64, who: &Row, rng: &mut Random) -> Fall {
    let (die, ability) = fall_die();
    let save = check_ability(who, ability, 0, rng);
    let rolled = roll::pool(dice, die, rng);
    let saved = save.as_ref().is_some_and(Check::passed);
    Fall {
        dice,
        save,
        rolled,
        damage: if saved { rolled / 2 } else { rolled },
    }
}

/// What a fall toll says it was, for the sentences about it: how far down,
/// and whether a landing halved it.
pub fn words(storeys: Option<i64>, saved: bool, damage: i64) -> String {
    let mut words = match storeys {
        Some(1) => "a fall of 1 storey".to_string(),
        Some(storeys) => format!("a fall of {storeys} storeys"),
        None => "a fall".to_string(),
    };
    if saved && damage > 0 {
        words.push_str(", and a good landing halved it");
    }
    words
}

/// The disposition of a thing that broke. Like a thing used up, it stays
/// as a row in no place, so the thing it was copied from is never copied
/// into the same game again.
pub const BROKEN: &str = "broken";

/// How a thing came down on the floor it landed on: the rows of
/// `data/physics.yml`'s `height_step`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Landing {
    /// Put down by the player.
    Dropped,
    /// Thrown at somebody or through a way out on the same storey.
    Thrown,
    /// Thrown through a doorway that is a fall.
    Fell,
}

impl Landing {
    pub fn name(self) -> &'static str {
        match self {
            Landing::Dropped => "dropped",
            Landing::Thrown => "thrown",
            Landing::Fell => "fell",
        }
    }
}

/// One break roll: the share of the die the thing breaks on, the die, and
/// whether it broke.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Break {
    pub share: i64,
    pub die: i64,
    pub broke: bool,
}

/// `data/physics.yml`'s `fragility`: each fragility and its base share, none
/// for a thing that never breaks.
pub fn fragilities() -> &'static [(String, Option<i64>)] {
    &data::physics().fragility
}

/// `data/physics.yml`'s `height_step`: what each way of coming down adds to a
/// share.
pub fn height_steps() -> &'static [(String, i64)] {
    &data::physics().height_step
}

/// `data/physics.yml`'s `surface`: each surface and what it adds to a share.
pub fn surfaces() -> &'static [(String, i64)] {
    &data::physics().surface
}

/// The die a break is thrown on.
pub fn break_die() -> i64 {
    data::physics().break_die
}

/// The share of the break die a thing of `fragility` breaks on, landing
/// the way `landing` says on a floor of `surface`: its base, plus how it
/// came down, plus the floor, held between 0 and the die. None, and no die
/// thrown, for a thing that never breaks or whose fragility the table does
/// not name; a surface the table does not name adds nothing.
pub fn share(fragility: Option<&str>, landing: Landing, surface: Option<&str>) -> Option<i64> {
    let physics = data::physics();
    let fragility = fragility?;
    let base = physics
        .fragility
        .iter()
        .find(|(name, _)| name == fragility)
        .and_then(|(_, base)| *base)?;
    let step = height_steps()
        .iter()
        .find(|(name, _)| name == landing.name())
        .map_or(0, |(_, step)| *step);
    let floor = surface
        .and_then(|surface| physics.surface.iter().find(|(name, _)| name == surface))
        .map_or(0, |(_, added)| *added);
    Some((base + step + floor).clamp(0, physics.break_die))
}

/// A break roll on `share`: one die, and the thing breaks when it comes up
/// at or under the share.
pub fn break_roll(share: i64, rng: &mut Random) -> Break {
    let die = roll::die(break_die(), rng);
    Break {
        share,
        die,
        broke: die <= share,
    }
}

/// `data/physics.yml`'s `throw_range`: the paces a thing of each bulk
/// carries at ordinary gravity.
pub fn throw_ranges() -> &'static [(String, i64)] {
    &data::physics().throw_range
}

/// `data/physics.yml`'s `throw_reach_per_strength` and `throw_reach_cap`:
/// the paces each point of strength over 10 adds, and the most it adds.
pub fn throw_reach() -> (i64, i64) {
    let physics = data::physics();
    (physics.throw_reach_per_strength, physics.throw_reach_cap)
}

/// The gravity `throw_range` is measured at.
const ORDINARY: &str = "ordinary";

/// The strength over which a thrower adds reach.
const REACH_FROM: i64 = 10;

/// How many paces a thing of `bulk` carries when somebody of `strength`
/// throws it at `gravity`: its paces at ordinary gravity, scaled by
/// ordinary's dice over this gravity's (a light world carries twice as far,
/// a heavy one two thirds as far, rounded down, and never under one pace),
/// plus the thrower's reach. None, and nothing held against the throw, in a
/// world with no gravity, or for a bulk or a gravity the tables do not name
/// (a thing that does not move has no range).
pub fn range(bulk: Option<&str>, strength: i64, gravity: Option<&str>) -> Option<i64> {
    let dice_of = |name: &str| {
        gravities()
            .iter()
            .find(|(row, _)| row == name)
            .map(|(_, dice)| *dice)
    };
    let dice = dice_of(gravity?)?;
    let ordinary = dice_of(ORDINARY)?;
    let bulk = bulk?;
    let paces = throw_ranges()
        .iter()
        .find(|(row, _)| row == bulk)
        .map(|(_, paces)| *paces)?;
    let (per_strength, cap) = throw_reach();
    let reach = (strength - REACH_FROM).clamp(0, cap) * per_strength;
    Some((paces * ordinary / dice).max(1) + reach)
}

/// A throw's range, and the paces it was held against: none where the room
/// gives nothing to measure, and the range is only told.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reach {
    pub range: i64,
    pub distance: Option<i64>,
}

impl Reach {
    /// The thing did not carry as far as what it was thrown at.
    pub fn short(&self) -> bool {
        self.distance.is_some_and(|distance| distance > self.range)
    }
}

/// The paces between two spots in one room, walking along the axes as
/// `Box#paces_to` measures between rooms.
pub fn paces(from: Spot, to: Spot) -> i64 {
    (from.x - to.x).abs() + (from.y - to.y).abs()
}

/// The cell of `room` beside the doorway it shares with `other`, nearest
/// `from`: where a thing thrown at that way out leaves the room. None for
/// two rooms with no wall between them on one storey (a stair, a window),
/// which gives nothing to measure.
pub fn by_the_way_out(room: &Box, other: &Box, from: Spot) -> Option<Spot> {
    let wall = room.shared_wall(other)?;
    let beside = |line: i64, start: i64| if line == start { line } else { line - 1 };
    Some(match wall.axis {
        Axis::X => Spot {
            x: beside(wall.line, room.x),
            y: from.y.clamp(wall.from, wall.to - 1),
        },
        Axis::Y => Spot {
            x: from.x.clamp(wall.from, wall.to - 1),
            y: beside(wall.line, room.y),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roll::Seed;
    use serde_json::json;

    fn body(dexterity: i64) -> Row {
        json!({ "strength": 10, "dexterity": dexterity, "will": 10 })
            .as_object()
            .unwrap()
            .clone()
    }

    #[test]
    fn a_doorway_falls_only_when_it_goes_down_between_two_storeys() {
        assert_eq!(storeys(Some(1), Some(0)), Some(1));
        assert_eq!(storeys(Some(3), Some(1)), Some(2));
        assert_eq!(storeys(Some(0), Some(0)), None);
        assert_eq!(storeys(Some(0), Some(1)), None);
        assert_eq!(storeys(None, Some(0)), None);
        assert_eq!(storeys(Some(1), None), None);
    }

    #[test]
    fn gravity_decides_the_dice_and_no_gravity_throws_none() {
        assert_eq!(dice(1, Some("light")), Some(1));
        assert_eq!(dice(1, Some("ordinary")), Some(2));
        assert_eq!(dice(2, Some("heavy")), Some(6));
        assert_eq!(dice(1, None), None);
        assert_eq!(dice(1, Some("sideways")), None);
        assert_eq!(dice(0, Some("heavy")), None);
    }

    #[test]
    fn a_passed_save_halves_the_fall_and_a_missed_one_takes_it_whole() {
        let seed = Seed {
            story: 1,
            playthrough: 1,
            at: 0,
            sequence: 7,
            kind: roll::FALL,
        };
        let sure = fall(2, &body(20), &mut seed.generator());
        assert!(sure.saved());
        assert_eq!(sure.damage, sure.rolled / 2);
        let hopeless = fall(2, &body(0), &mut seed.generator());
        assert!(!hopeless.saved());
        assert_eq!(hopeless.save.as_ref().and_then(|save| save.die), None);
        assert_eq!(hopeless.damage, hopeless.rolled);
        assert!((2..=12).contains(&hopeless.rolled));
    }

    #[test]
    fn a_body_with_no_abilities_makes_no_save() {
        let nobody = Row::new();
        let taken = fall(1, &nobody, &mut Random::new(3));
        assert!(taken.save.is_none());
        assert_eq!(taken.damage, taken.rolled);
    }

    #[test]
    fn the_words_say_how_far_and_whether_the_landing_halved_it() {
        assert_eq!(words(Some(1), false, 5), "a fall of 1 storey");
        assert_eq!(words(Some(2), false, 9), "a fall of 2 storeys");
        assert_eq!(
            words(Some(1), true, 2),
            "a fall of 1 storey, and a good landing halved it"
        );
        assert_eq!(words(Some(1), true, 0), "a fall of 1 storey");
        assert_eq!(words(None, false, 3), "a fall");
    }

    #[test]
    fn a_sturdy_thing_or_one_the_table_does_not_name_throws_no_die() {
        assert_eq!(share(Some("sturdy"), Landing::Fell, Some("hard")), None);
        assert_eq!(share(Some("porcelain"), Landing::Fell, Some("hard")), None);
        assert_eq!(share(None, Landing::Dropped, None), None);
    }

    #[test]
    fn the_share_is_the_base_how_it_came_down_and_the_floor_held_to_the_die() {
        assert_eq!(share(Some("fragile"), Landing::Dropped, None), Some(2));
        assert_eq!(
            share(Some("fragile"), Landing::Dropped, Some("soft")),
            Some(0)
        );
        assert_eq!(
            share(Some("fragile"), Landing::Thrown, Some("hard")),
            Some(4)
        );
        assert_eq!(share(Some("fragile"), Landing::Fell, Some("hard")), Some(5));
        assert_eq!(
            share(Some("brittle"), Landing::Dropped, Some("hard")),
            Some(5)
        );
        assert_eq!(
            share(Some("brittle"), Landing::Thrown, Some("hard")),
            Some(6)
        );
        assert_eq!(share(Some("brittle"), Landing::Fell, Some("hard")), Some(6));
        assert_eq!(
            share(Some("brittle"), Landing::Dropped, Some("mud")),
            Some(4)
        );
    }

    #[test]
    fn a_thing_breaks_when_the_die_is_at_or_under_its_share() {
        let seed = Seed {
            story: 1,
            playthrough: 1,
            at: 0,
            sequence: 7,
            kind: roll::BREAK,
        };
        let sure = break_roll(6, &mut seed.generator());
        assert!(sure.broke);
        assert!((1..=6).contains(&sure.die));
        let never = break_roll(0, &mut seed.generator());
        assert!(!never.broke);
        assert_eq!(never.die, sure.die, "one seed, one die");
        for share in 0..=6 {
            let rolled = break_roll(share, &mut seed.generator());
            assert_eq!(rolled.broke, rolled.die <= share);
        }
    }

    #[test]
    fn range_falls_with_bulk_and_gravity_and_rises_with_strength_to_its_cap() {
        assert_eq!(range(Some("light"), 10, Some("ordinary")), Some(12));
        assert_eq!(range(Some("handy"), 10, Some("ordinary")), Some(8));
        assert_eq!(range(Some("heavy"), 9, Some("ordinary")), Some(3));
        assert_eq!(range(Some("light"), 10, Some("light")), Some(24));
        assert_eq!(range(Some("handy"), 10, Some("heavy")), Some(5));
        assert_eq!(range(Some("heavy"), 10, Some("heavy")), Some(2));
        assert_eq!(range(Some("handy"), 12, Some("ordinary")), Some(10));
        assert_eq!(range(Some("handy"), 18, Some("ordinary")), Some(12));
        assert_eq!(range(Some("heavy"), 3, Some("ordinary")), Some(3));
    }

    #[test]
    fn no_gravity_and_no_named_bulk_give_no_range() {
        assert_eq!(range(Some("light"), 12, None), None);
        assert_eq!(range(Some("light"), 12, Some("sideways")), None);
        assert_eq!(range(Some("immovable"), 12, Some("ordinary")), None);
        assert_eq!(range(None, 12, Some("ordinary")), None);
    }

    #[test]
    fn a_throw_is_short_only_when_it_was_measured_and_did_not_carry() {
        let reach = |distance| Reach { range: 3, distance };
        assert!(reach(Some(4)).short());
        assert!(!reach(Some(3)).short());
        assert!(!reach(None).short());
    }

    #[test]
    fn a_way_out_is_the_cell_beside_its_doorway_nearest_the_thrower() {
        let room = Box {
            x: 0,
            y: 0,
            z: 0,
            width: 6,
            depth: 4,
        };
        let east = Box {
            x: 6,
            y: 1,
            z: 0,
            width: 3,
            depth: 2,
        };
        let from = Spot { x: 1, y: 0 };
        assert_eq!(
            by_the_way_out(&room, &east, from),
            Some(Spot { x: 5, y: 1 })
        );
        let north = Box {
            x: 2,
            y: -3,
            z: 0,
            width: 2,
            depth: 3,
        };
        assert_eq!(
            by_the_way_out(&room, &north, Spot { x: 5, y: 3 }),
            Some(Spot { x: 3, y: 0 })
        );
        let above = Box {
            x: 0,
            y: 0,
            z: 1,
            width: 6,
            depth: 4,
        };
        assert_eq!(by_the_way_out(&room, &above, from), None);
    }
}
