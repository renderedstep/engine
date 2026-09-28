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
}
