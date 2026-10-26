//! Carrying: whether one more thing fits about a body, given what it already
//! holds, its strength and the storage places it has, where it would go, and
//! why not when it does not. And the doctor's finding for an arc that asks
//! the player to hold a thing nothing can pick up.
//!
//! Three checks, read off `data/physics.yml`'s `carry` table by the thing's
//! bulk (`items.bulk`), and no die, so a refusal says the same thing every
//! time it is asked:
//!
//! - **Weight.** A body bears its strength over `load_divisor`, rounded
//!   down, against the weight of everything it carries, in hand or stowed.
//!   A thing heavier on its own than that cannot be lifted at all.
//! - **Hands.** A body has `hands`, and a thing in hand takes `hands_for` of
//!   them.
//! - **Storage.** A place that holds things without a hand takes the bulks
//!   it `fits`, each taking its `size` of the place's `room`. A bare body
//!   has one, [`ABOUT_THE_PERSON`], until it wears something with room.
//!
//! A thing goes where a person would put it without thinking: a light thing
//! into the first storage place it fits with room for it, anything else into
//! free hands, and failing that into storage that fits. Nothing already held
//! is moved to make room.
//!
//! THE RULE GATES ACTS, NEVER STATE. It answers for the next thing only:
//! nothing here says what is held is too much, and a body already carrying
//! more than fits keeps it.
//!
//! A WORLD TURNS IT ON. Only a world whose `universes.carrying` is
//! [`LIMITED`] asks it ([`limited`]); a world with none plays as it always
//! has. Where it is on, the turn refuses a take, a throw of a thing lying
//! here and a break that would put in hand what does not fit, and a person
//! is offered no take of what would not fit them and no give of what would
//! not fit the player.
//!
//! WHERE A HELD THING IS, IS READ, NOT KEPT. No row says which hand or
//! which storage place: what a body holds is [`arrange`]d as a person would
//! hold it, small things put away first. A worn thing is left out of the
//! count: it takes no hand and weighs nothing until clothing says otherwise.

use crate::data::{self, Carry};
use crate::noticed::Finding;
use crate::records::{flag, id, int, string, text, Records, Row};

/// `universes.carrying`: the one word that turns the limits on.
pub const LIMITED: &str = "limited";

/// Every word `universes.carrying` may hold; none plays with no limits.
pub const CARRYINGS: &[&str] = &[LIMITED];

/// The storage place every body has of its own.
pub const ABOUT_THE_PERSON: &str = "about the person";

/// The bulk that goes into storage before a hand.
const SMALL: &str = "light";

/// The doctor's code for an arc beat whose thing nothing can pick up.
pub const UNCARRIABLE: &str = "hold_item_target_cannot_be_carried";

/// `data/physics.yml`'s `carry`.
pub fn table() -> &'static Carry {
    &data::physics().carry
}

/// Where a held thing is, or where a new one would go.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Hands,
    /// In the body's storage place at this index.
    Stowed(usize),
}

/// One thing a body already holds.
#[derive(Clone, Copy, Debug)]
pub struct Held<'a> {
    pub bulk: &'a str,
    pub place: Place,
}

/// What a body brings to carrying: its strength and its storage places, by
/// their words in the `storage` table.
#[derive(Clone, Debug)]
pub struct Body {
    pub strength: i64,
    pub storage: Vec<String>,
}

impl Body {
    /// A body wearing nothing with room: only [`ABOUT_THE_PERSON`].
    pub fn bare(strength: i64) -> Body {
        Body {
            strength,
            storage: vec![ABOUT_THE_PERSON.to_string()],
        }
    }
}

/// Why a thing does not fit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Nothing can hold a thing of this bulk: it does not move.
    Immovable,
    /// It weighs more on its own than the body bears.
    Heavy { weight: i64, bears: i64 },
    /// It and what is carried weigh more than the body bears.
    Load {
        carried: i64,
        weight: i64,
        bears: i64,
    },
    /// No free hands for it and no storage room that fits it.
    Hands { free: i64, needs: i64 },
}

impl Refused {
    /// The reason in one word, as a refusal will name it.
    pub fn reason(&self) -> &'static str {
        match self {
            Refused::Immovable => "immovable",
            Refused::Heavy { .. } => "heavy",
            Refused::Load { .. } => "load",
            Refused::Hands { .. } => "hands",
        }
    }
}

fn row(table: &[(String, i64)], bulk: &str) -> Option<i64> {
    table
        .iter()
        .find(|(name, _)| name == bulk)
        .map(|(_, value)| *value)
}

/// Whether anything can hold a thing of this bulk: it has a row of hands.
pub fn liftable(bulk: &str) -> bool {
    row(&table().hands_for, bulk).is_some()
}

/// The hands a thing of this bulk takes; none for one nothing can hold.
pub fn hands_for(bulk: &str) -> i64 {
    row(&table().hands_for, bulk).unwrap_or(0)
}

/// The weight a body of `strength` bears.
pub fn bears(strength: i64) -> i64 {
    strength.max(0) / table().load_divisor
}

/// The weight of a thing of this bulk; a bulk the table does not name
/// weighs nothing.
pub fn weight(bulk: &str) -> i64 {
    row(&table().weight, bulk).unwrap_or(0)
}

/// Where a thing of `bulk` would go about `body`, holding `held`, or why it
/// does not fit. Weight is asked before room, since no rearranging makes a
/// thing lighter.
pub fn fits(bulk: &str, held: &[Held], body: &Body) -> Result<Place, Refused> {
    if !liftable(bulk) {
        return Err(Refused::Immovable);
    }
    let bears = bears(body.strength);
    let carried: i64 = held.iter().map(|thing| weight(thing.bulk)).sum();
    let weight = weight(bulk);
    if weight > bears {
        return Err(Refused::Heavy { weight, bears });
    }
    if carried + weight > bears {
        return Err(Refused::Load {
            carried,
            weight,
            bears,
        });
    }
    room(bulk, held, body)
}

/// Where a thing of `bulk` would go about `body`, holding `held`, by room
/// alone, with no weight asked.
fn room(bulk: &str, held: &[Held], body: &Body) -> Result<Place, Refused> {
    let carry = table();
    let Some(needs) = row(&carry.hands_for, bulk) else {
        return Err(Refused::Immovable);
    };
    let in_hand: i64 = held
        .iter()
        .filter(|thing| thing.place == Place::Hands)
        .map(|thing| row(&carry.hands_for, thing.bulk).unwrap_or(0))
        .sum();
    let free = carry.hands - in_hand;
    let size = row(&carry.size, bulk).unwrap_or(0);
    let stowable = body.storage.iter().enumerate().find_map(|(index, place)| {
        let storage = carry
            .storage
            .iter()
            .find(|(name, _)| name == place)
            .map(|(_, storage)| storage)?;
        let used: i64 = held
            .iter()
            .filter(|thing| thing.place == Place::Stowed(index))
            .map(|thing| row(&carry.size, thing.bulk).unwrap_or(0))
            .sum();
        (storage.fits.iter().any(|fits| fits == bulk) && used + size <= storage.room)
            .then_some(Place::Stowed(index))
    });
    match stowable {
        Some(place) if bulk == SMALL => Ok(place),
        _ if needs <= free => Ok(Place::Hands),
        Some(place) => Ok(place),
        None => Err(Refused::Hands { free, needs }),
    }
}

/// Where each of the things a body holds is, in the order given: as a
/// person would hold them, every light thing put away first, each where
/// [`fits`] would put it with nothing asked of weight. A thing with no room
/// left is in hand all the same, since what is held is never refused.
pub fn arrange<'a>(bulks: &[&'a str], body: &Body) -> Vec<Held<'a>> {
    let mut placed: Vec<Option<Held>> = vec![None; bulks.len()];
    let small = (0..bulks.len()).filter(|&at| bulks[at] == SMALL);
    let rest = (0..bulks.len()).filter(|&at| bulks[at] != SMALL);
    for at in small.chain(rest) {
        let held: Vec<Held> = placed.iter().flatten().copied().collect();
        let place = room(bulks[at], &held, body).unwrap_or(Place::Hands);
        placed[at] = Some(Held {
            bulk: bulks[at],
            place,
        });
    }
    placed.into_iter().flatten().collect()
}

/// Whether the carrying limits hold in this story's world.
pub fn limited(records: &Records, story: i64) -> bool {
    records
        .find("stories", story)
        .and_then(|story| int(story, "universe_id"))
        .and_then(|universe| records.find("universes", universe))
        .is_some_and(|universe| text(universe, "carrying") == Some(LIMITED))
}

/// What a body holds that the rule counts: everything but what it wears.
pub fn counted<'a>(held: &[&'a Row]) -> Vec<&'a Row> {
    held.iter()
        .copied()
        .filter(|thing| !flag(thing, "worn"))
        .collect()
}

/// What a bare body of `strength` holds that the rule counts, each with
/// where it is ([`arrange`]), in the order given.
pub fn arranged<'a>(held: &[&'a Row], strength: i64) -> Vec<(&'a Row, Place)> {
    let counted = counted(held);
    let bulks: Vec<&str> = counted.iter().map(|row| bulk_of(row)).collect();
    let places = arrange(&bulks, &Body::bare(strength));
    counted
        .into_iter()
        .zip(places)
        .map(|(row, held)| (row, held.place))
        .collect()
}

/// [`fits`] over rows: `thing` about a bare body of `strength` holding
/// `held`.
pub fn fits_row(thing: &Row, strength: i64, held: &[&Row]) -> Result<Place, Refused> {
    let held: Vec<Held> = arranged(held, strength)
        .into_iter()
        .map(|(row, place)| Held {
            bulk: bulk_of(row),
            place,
        })
        .collect();
    fits(bulk_of(thing), &held, &Body::bare(strength))
}

/// A row's bulk, `handy` where it names none (the column's default).
pub fn bulk_of(row: &Row) -> &str {
    text(row, "bulk").unwrap_or("handy")
}

/// Whether a person may come to hold `thing`: always in a world with no
/// limits, and for a body with no strength recorded, which the rule has
/// nothing to read.
pub fn may_hold(records: &Records, story: i64, holder: &Row, thing: &Row, held: &[&Row]) -> bool {
    if !limited(records, story) {
        return true;
    }
    int(holder, "strength").is_none_or(|strength| fits_row(thing, strength, held).is_ok())
}

/// Every beat of a story's arcs that asks the player to hold a thing nothing
/// can pick up, which leaves the arc unfinishable: a `hold_item` step bound
/// to a thing whose bulk has no row of hands.
pub fn findings(records: &Records, story: i64) -> Vec<Finding> {
    let mut found = Vec::new();
    for quest in records.select("quests", |quest| int(quest, "story_id") == Some(story)) {
        let quest_id = id(quest);
        for step in records.select("quest_steps", |step| {
            int(step, "quest_id") == Some(quest_id)
                && text(step, "trigger_kind") == Some("hold_item")
                && text(step, "target_type") == Some("Item")
        }) {
            let Some(thing) =
                int(step, "target_id").and_then(|target| records.find("items", target))
            else {
                continue;
            };
            let Some(bulk) = text(thing, "bulk").filter(|bulk| !liftable(bulk)) else {
                continue;
            };
            found.push(Finding {
                code: UNCARRIABLE,
                severity: "fatal",
                message: format!(
                    "{:?} step {} asks the player to hold {:?}, which is {bulk}: nothing can pick \
                     it up, so the arc cannot be finished",
                    string(quest, "title"),
                    int(step, "position").unwrap_or_default(),
                    string(thing, "name")
                ),
                remedy: "manual",
                subject: ("quest_steps", id(step)),
            });
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hands(bulk: &str) -> Held<'_> {
        Held {
            bulk,
            place: Place::Hands,
        }
    }

    fn stowed(bulk: &str) -> Held<'_> {
        Held {
            bulk,
            place: Place::Stowed(0),
        }
    }

    #[test]
    fn a_thing_that_does_not_move_is_never_held() {
        assert_eq!(
            fits("immovable", &[], &Body::bare(18)),
            Err(Refused::Immovable)
        );
        assert_eq!(
            fits("enormous", &[], &Body::bare(18)),
            Err(Refused::Immovable)
        );
        assert!(!liftable("immovable"));
        assert!(liftable("heavy"));
    }

    #[test]
    fn a_light_thing_goes_about_the_person_before_a_hand() {
        let body = Body::bare(9);
        assert_eq!(fits("light", &[], &body), Ok(Place::Stowed(0)));
        assert_eq!(
            fits("light", &[stowed("light"), stowed("light")], &body),
            Ok(Place::Hands)
        );
        assert_eq!(fits("handy", &[], &body), Ok(Place::Hands));
    }

    #[test]
    fn a_heavy_thing_takes_both_hands() {
        let body = Body::bare(13);
        assert_eq!(fits("heavy", &[], &body), Ok(Place::Hands));
        assert_eq!(
            fits("handy", &[hands("heavy")], &body),
            Err(Refused::Hands { free: 0, needs: 1 })
        );
        assert_eq!(
            fits("heavy", &[hands("handy")], &body),
            Err(Refused::Hands { free: 1, needs: 2 })
        );
        assert_eq!(
            fits("light", &[hands("heavy")], &body),
            Ok(Place::Stowed(0))
        );
    }

    #[test]
    fn the_salt_assizes_slate_cord_and_bell_fit_with_room_about_the_person() {
        let body = Body::bare(13);
        let held = [hands("heavy"), stowed("light")];
        assert_eq!(fits("light", &held, &body), Ok(Place::Stowed(0)));
        let full = [hands("heavy"), stowed("light"), stowed("light")];
        assert_eq!(
            fits("light", &full, &body),
            Err(Refused::Hands { free: 0, needs: 1 })
        );
    }

    #[test]
    fn strength_over_two_is_the_load_a_body_bears() {
        assert_eq!(bears(7), 3);
        assert_eq!(bears(8), 4);
        assert_eq!(bears(-1), 0);
        assert_eq!(
            fits("heavy", &[], &Body::bare(7)),
            Err(Refused::Heavy {
                weight: 4,
                bears: 3
            })
        );
        assert_eq!(
            fits("handy", &[hands("heavy")], &Body::bare(9)),
            Err(Refused::Load {
                carried: 4,
                weight: 1,
                bears: 4
            })
        );
        assert_eq!(
            fits("handy", &[hands("handy")], &Body::bare(3)),
            Err(Refused::Load {
                carried: 1,
                weight: 1,
                bears: 1
            })
        );
        assert_eq!(
            fits("light", &[hands("heavy")], &Body::bare(8)),
            Ok(Place::Stowed(0))
        );
    }

    #[test]
    fn a_handy_thing_goes_into_storage_that_fits_when_the_hands_are_full() {
        let body = Body {
            strength: 18,
            storage: vec![ABOUT_THE_PERSON.to_string(), "satchel".to_string()],
        };
        let held = [hands("handy"), hands("handy")];
        assert_eq!(fits("handy", &held, &body), Ok(Place::Stowed(1)));
        assert_eq!(fits("light", &held, &body), Ok(Place::Stowed(0)));
    }

    #[test]
    fn a_storage_place_the_table_does_not_name_holds_nothing() {
        let body = Body {
            strength: 12,
            storage: vec!["hatbox".to_string()],
        };
        assert_eq!(fits("light", &[], &body), Ok(Place::Hands));
    }

    #[test]
    fn what_is_already_held_over_the_limit_is_never_the_rules_business() {
        let held = [hands("heavy"), hands("heavy")];
        assert_eq!(
            fits("light", &held, &Body::bare(18)),
            Ok(Place::Stowed(0)),
            "a light thing weighs nothing and the room about the person is free"
        );
    }

    #[test]
    fn what_is_held_is_arranged_small_things_put_away_first() {
        let body = Body::bare(9);
        let places = |bulks: &[&str]| -> Vec<Place> {
            arrange(bulks, &body)
                .into_iter()
                .map(|held| held.place)
                .collect()
        };
        assert_eq!(
            places(&["handy", "light", "handy", "light"]),
            [
                Place::Hands,
                Place::Stowed(0),
                Place::Hands,
                Place::Stowed(0)
            ]
        );
        assert_eq!(
            places(&["light", "light", "light"]),
            [Place::Stowed(0), Place::Stowed(0), Place::Hands],
            "the third light thing has no room about the person"
        );
        assert_eq!(
            places(&["heavy", "heavy", "light"]),
            [Place::Hands, Place::Hands, Place::Stowed(0)],
            "what is held over the limit is in hand all the same"
        );
    }

    #[test]
    fn every_reason_has_its_word() {
        assert_eq!(Refused::Immovable.reason(), "immovable");
        assert_eq!(
            Refused::Heavy {
                weight: 4,
                bears: 3
            }
            .reason(),
            "heavy"
        );
        assert_eq!(
            Refused::Load {
                carried: 1,
                weight: 1,
                bears: 1
            }
            .reason(),
            "load"
        );
        assert_eq!(Refused::Hands { free: 0, needs: 1 }.reason(), "hands");
    }
}
