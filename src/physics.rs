//! Falls: how far the player dropped through a doorway, at the gravity of
//! the world they are in, and what the landing cost.
//!
//! A fall is a doorway's hazard (`location_connections.hazard` = `fall`)
//! whose die is not on the row: the storeys between its two rooms and the
//! world's gravity (`universes.gravity`) decide how many dice are thrown,
//! out of `data/physics.yml`. A world with no gravity, or a doorway that
//! does not go down, throws nothing, so a world that says nothing about
//! gravity plays exactly as it did before falls existed.

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
}
