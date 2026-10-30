//! Who a new person is before anybody writes them: the seeded draws of
//! `Character::Registry#slots` (the people a room is born with) and of
//! `Character::Generator` (one generated person).

use crate::danger;
use crate::population;
use crate::random::Random;
use crate::roll::{self, Seed};

pub const SEXES: [&str; 5] = ["male", "female", "non-binary", "trans woman", "trans man"];
pub const NPC_AGES: (i64, i64) = (18, 80);
pub const GENERATED_AGES: (i64, i64) = (18, 120);
pub const ATTRACTIVENESS: [&str; 4] = ["very_attractive", "attractive", "average", "unattractive"];
pub const BIRTH_PLACES: [&str; 10] = [
    "large city",
    "small town",
    "cave",
    "remote wilderness",
    "isolated island",
    "floating city",
    "jungle",
    "desert",
    "mountain",
    "boat",
];
/// Spelled as the Ruby table spells it, typo included, because the word is
/// part of what the draw answers.
pub const RAISED_BY: [&str; 11] = [
    "parents",
    "guardian",
    "siblings",
    "grandparents",
    "aunt/uncle",
    "neighbor",
    "teacher",
    "mentor",
    "pet",
    "wild aninmals",
    "self",
];
pub const MAX_PER_ROOM: i64 = 3;
pub const MAX_PER_STORY: i64 = 12;

/// A race of the story's universe, in the order the universe lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Race<'a> {
    pub name: &'a str,
    pub monstrous: bool,
}

/// One person a room is born with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot<'a> {
    pub race: &'a str,
    pub age: i64,
    pub sex: &'static str,
}

/// The room the slots are drawn for.
#[derive(Clone, Copy, Debug)]
pub struct Room<'a> {
    pub story_id: i64,
    pub location_id: i64,
    pub name: &'a str,
    pub danger: &'a str,
    pub population: Option<&'a str>,
    /// How many people the room already holds.
    pub present: i64,
    /// How many people the story already holds.
    pub in_story: i64,
    /// How many people the story may hold: [`MAX_PER_STORY`] unless the
    /// story names its own (`realization::people_limit`).
    pub story_limit: i64,
}

/// The races a draw picks from: those of the wanted kind, or every race when
/// the universe has none of that kind.
fn pool<'r, 'a>(races: &'r [Race<'a>], monstrous: bool) -> Vec<&'r Race<'a>> {
    let wanted: Vec<_> = races.iter().filter(|r| r.monstrous == monstrous).collect();
    if wanted.is_empty() {
        races.iter().collect()
    } else {
        wanted
    }
}

fn race_from<'a>(races: &[Race<'a>], monstrous: bool, rng: &mut Random) -> &'a str {
    let pool = pool(races, monstrous);
    pool[roll::one_of(pool.len(), rng)].name
}

/// `Character::Registry#slots`: every monstrous throw first, then each
/// person's race, age and sex, all from the room's danger generator.
pub fn slots<'a>(room: &Room, races: &[Race<'a>]) -> Vec<Slot<'a>> {
    let (_, count) = population::for_room(room.name, room.population);
    let drawn = count
        .min((MAX_PER_ROOM - room.present).max(0))
        .min((room.story_limit - room.in_story).max(0));
    let mut rng = danger::generator_for(room.story_id, room.location_id);
    let throws: Vec<bool> = (0..drawn)
        .map(|_| danger::monstrous(room.danger, &mut rng))
        .collect();
    throws
        .into_iter()
        .map(|monstrous| Slot {
            race: race_from(races, monstrous, &mut rng),
            age: rng.range(NPC_AGES.0, NPC_AGES.1),
            sex: SEXES[roll::one_of(SEXES.len(), &mut rng)],
        })
        .collect()
}

/// What `Character::Generator` settles before the model writes a person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generated<'a> {
    pub race: &'a str,
    pub age: i64,
    pub sex: &'static str,
    pub attractiveness: &'static str,
    pub born_in: &'static str,
    pub raised_by: &'static str,
}

/// `Character::Generator`'s draws, seeded from the story and how many people
/// it already has (`sequence`). The race is always one of the peoples when
/// there are any.
pub fn generated<'a>(story_id: i64, sequence: i64, races: &[Race<'a>]) -> Generated<'a> {
    let mut rng = Seed {
        story: story_id.into(),
        sequence: sequence.into(),
        kind: roll::CAST,
        ..Seed::default()
    }
    .generator();
    let race = race_from(races, false, &mut rng);
    let age = rng.range(GENERATED_AGES.0, GENERATED_AGES.1);
    let sex = SEXES[roll::one_of(SEXES.len(), &mut rng)];
    let attractiveness = ATTRACTIVENESS[roll::one_of(ATTRACTIVENESS.len(), &mut rng)];
    let born_in = BIRTH_PLACES[roll::one_of(BIRTH_PLACES.len(), &mut rng)];
    let raised_by = RAISED_BY[roll::one_of(RAISED_BY.len(), &mut rng)];
    Generated {
        race,
        age,
        sex,
        attractiveness,
        born_in,
        raised_by,
    }
}

#[cfg(test)]
mod tests {
    use super::{slots, Race, Room, MAX_PER_STORY};

    const RACES: [Race; 1] = [Race {
        name: "Countryfolk",
        monstrous: false,
    }];

    /// A crowd's room with `in_story` people already in a story that may
    /// hold `story_limit`.
    fn crowd(in_story: i64, story_limit: i64) -> Room<'static> {
        Room {
            story_id: 1,
            location_id: 5,
            name: "Cellar",
            danger: "safe",
            population: Some("a crowd"),
            present: 0,
            in_story,
            story_limit,
        }
    }

    #[test]
    fn a_story_at_the_ordinary_limit_draws_nobody_more() {
        assert_eq!(slots(&crowd(11, MAX_PER_STORY), &RACES).len(), 1);
        assert!(slots(&crowd(12, MAX_PER_STORY), &RACES).is_empty());
    }

    #[test]
    fn a_raised_limit_draws_past_twelve_with_the_same_dice() {
        let raised = slots(&crowd(12, 20), &RACES);
        assert_eq!(raised.len(), 3, "the crowd's roll, with places to spare");
        assert_eq!(slots(&crowd(0, MAX_PER_STORY), &RACES), raised);
        assert_eq!(slots(&crowd(19, 20), &RACES), raised[..1]);
        assert!(slots(&crowd(20, 20), &RACES).is_empty());
    }
}
