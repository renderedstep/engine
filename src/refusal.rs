//! What the engine says when it will not play a line, and the one author of
//! it. A refused line does nothing: the player is answered in the engine's
//! own words, built from records it already holds.

use crate::carry::{self, Place, Refused};
use crate::cascade::Leave;
use crate::intent::{Intent, Unread};
use crate::playthrough::Ended;
use crate::room::{Dressing, Person, Record, Thing};
use crate::text::{to_sentence, upcase_first};

pub const KINDS: &[&str] = &[
    "named_more_than_one",
    "unresolved",
    "immovable",
    "cannot_carry",
    "unreadable",
    "unsure",
    "unplayable",
    "dead",
    "concluded",
    "stopped",
];

/// The kinds that are a state of the game rather than a reading of a line.
const GAME_OVER: &[&str] = &["dead", "concluded", "stopped"];

/// The kind of a line the local reader read and was unsure of: the game asks
/// instead of playing it ([`crate::intent::Unread::Unsure`]).
pub const UNSURE: &str = "unsure";

/// Said on every shape that leaves the player another line to type.
pub const UNCHANGED: &str = "Nothing has changed.";

fn asked(action: &str) -> &'static str {
    match action {
        "move" => "go to %s",
        "talk" => "talk to %s",
        "take" => "take %s",
        "drop" => "drop %s",
        "examine" => "read %s",
        "attack" => "attack %s",
        "throw" => "throw %s",
        _ => "%s",
    }
}

fn missed(action: &str) -> Option<&'static str> {
    Some(match action {
        "move" => "That did not resolve to one of the ways out of here.",
        "talk" => "That did not resolve to anybody who is here.",
        "take" => "That did not resolve to anything lying here.",
        "drop" => "That did not resolve to anything you are carrying.",
        "attack" => "That did not resolve to anybody here to swing at.",
        "use" => {
            "That did not resolve to an available physical action with these items and doorways."
        }
        _ => return None,
    })
}

/// What an action that reads a closed set says when that set is empty.
pub fn empty(action: &str) -> Option<&'static str> {
    Some(match action {
        "move" => "There is no way out of here at all.",
        "talk" => "There is nobody here to talk to.",
        "take" => "There is nothing lying here to pick up.",
        "drop" => "You are carrying nothing, so there is nothing to put down.",
        "attack" => {
            "There is nobody here to fight. An attack is aimed at a person standing here, \
             not at the room or anything built into it -- look around, or use, take or throw \
             something you can reach."
        }
        "use" => "These items and doorways offer no matching physical action.",
        _ => return None,
    })
}

const NOTHING_MATCHED: &str = "That resolved to nothing.";

/// What `/end` says before the story has met everything it asked for.
pub const NOTHING_DUE: &str = "The story cannot end here yet: it is still asking for something.";

/// A line that only leaves, in a room with more than one way out.
const ONE_OF_SEVERAL: &str = "There is more than one way out of here, so that does not say which.";

fn no_protagonist(action: &str) -> Option<&'static str> {
    match action {
        "take" => Some("There is nobody here to pick anything up"),
        "throw" => Some("There is nobody here to throw anything"),
        _ => None,
    }
}

fn nowhere(action: &str) -> Option<&'static str> {
    match action {
        "drop" => Some("You are standing nowhere, so there is no floor to put anything down on"),
        _ => None,
    }
}

const NO_PROTAGONIST_FACT: &str = "this story has no player character yet, so there is nobody for \
     anything to belong to. `rake game:doctor` reports it as `no_protagonist` and says how to give \
     the story one.";

const NOWHERE_FACT: &str = "this playthrough is not standing in any room.";

fn offers(action: &str) -> Option<&'static str> {
    match action {
        "move" => Some("The ways out are: %s."),
        "talk" | "attack" => Some("Here with you: %s."),
        "take" => Some("Lying here: %s."),
        "drop" => Some("You are carrying: %s."),
        "use" => Some("Available attempts: %s."),
        _ => None,
    }
}

/// What a refused line can be said again as.
const PLAIN_ACTION: &str = "Say it again as one plain action -- go somewhere, talk to somebody, \
     take something, or put something down.";

const UNREADABLE: &str = "That did not come back as anything the game knows how to do. \
     Say it again as one plain action -- go somewhere, talk to somebody, \
     take something, or put something down.";

/// A line the local reader was unsure of: the captain's words of
/// 2026-10-29, followed by the reader's best guess as a line to type
/// ([`guessed`]), or where the guess has none, by [`PLAIN_ACTION`]. It says
/// itself that nothing happened, so [`UNCHANGED`] is not added to it.
pub const UNSURE_FACT: &str =
    "The game is not sure what you meant by that, so nothing has happened.";

/// The guess, as a line the grammar plays with no reader.
fn guessed(line: &str) -> String {
    format!("Its best guess was `{line}`. Type that to do it, or say what you meant another way.")
}

/// The slash line the grammar plays as `intent`, in the words the slash menu
/// completes ([`crate::slash_menu::SlashMenu`]), or `None` where the intent
/// has none: an `other`, or a reach with nothing to reach.
pub fn slash_line(intent: &Intent) -> Option<String> {
    if let Some(choice) = &intent.physical {
        let argument = choice.argument();
        return Some(if argument.is_empty() {
            format!("/{}", choice.kind)
        } else {
            format!("/{} {argument}", choice.kind)
        });
    }
    if intent.action == "examine" && intent.item.is_none() {
        return Some("/look".to_string());
    }
    let (word, _) = crate::grammar::RESOLVING
        .iter()
        .find(|(_, action)| *action == intent.action)?;
    let name = intent.subject()?.label();
    (!crate::text::is_blank(&name)).then(|| format!("/{word} {name}"))
}

/// A free-typed throw, which the local reader leaves unread.
pub const THROW_UNREAD: &str =
    "A throw is read only when it is typed as a command: /throw <thing> at <somebody here, or a way out>.";

/// A free-typed line that puts a thing on or takes it off, which no reader
/// reads.
pub const DRESS_UNREAD: &str = "Putting a thing on or taking it off is read only when it is typed \
     as a command: /wear <thing> or /remove <thing>.";

/// Any free-typed line while the local reader is down or not set up.
pub const READER_DOWN: &str = "The game cannot read free-typed lines right now. \
     Type the line as a command instead: the slash menu lists them.";

/// The kind asked for is not one of [`KINDS`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownKind(pub String);

impl std::fmt::Display for UnknownKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kinds: Vec<String> = KINDS.iter().map(|kind| format!(":{kind}")).collect();
        write!(f, ":{} is not one of [{}]", self.0, kinds.join(", "))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Refusal {
    pub kind: String,
    pub typed: String,
    pub fact: String,
    /// What would have worked, for a reader with no list of the room in
    /// front of them.
    pub offer: Option<String>,
}

impl Refusal {
    pub fn new(
        kind: &str,
        typed: &str,
        fact: &str,
        offer: Option<String>,
    ) -> Result<Refusal, UnknownKind> {
        if !KINDS.contains(&kind) {
            return Err(UnknownKind(kind.to_string()));
        }
        Ok(Refusal {
            kind: kind.to_string(),
            typed: typed.to_string(),
            fact: fact.to_string(),
            offer,
        })
    }

    fn of(kind: &str, typed: &str, fact: String, offer: Option<String>) -> Refusal {
        Refusal::new(kind, typed, &fact, offer).expect("a kind this file names")
    }

    /// The refusal a read line earns, or `None` when the engine can play it.
    /// `offered` is the closed set the intent's action reads against;
    /// `carried` is what the player holds, and `named` the one of those the
    /// line's name lands on: a take that found nothing lying here says so.
    pub fn for_intent(
        intent: &Intent,
        typed: &str,
        offered: &[Record],
        carried: &[Record],
        named: Option<&Record>,
    ) -> Option<Refusal> {
        if let Some(fact) = &intent.declined {
            return Some(Refusal::of("unplayable", typed, fact.clone(), None));
        }
        if intent.named_more_than_one() {
            let fact = format!(
                "You asked for two things at once: {}, and {}. One line is one act -- \
                 pick one and type it on its own.",
                phrase(&intent.action, intent.subject().as_ref()),
                phrase(&intent.action, intent.also_named.as_ref())
            );
            return Some(Refusal::of("named_more_than_one", typed, fact, None));
        }
        if intent.reached_for_nothing() {
            let taking = intent.action == "take";
            if let (true, Some(held)) = (taking, named) {
                return Some(Refusal::of(
                    "unresolved",
                    typed,
                    already_carrying(held),
                    None,
                ));
            }
            let fact = if offered.is_empty() {
                empty(&intent.action)
            } else {
                missed(&intent.action)
            };
            let mut offer = if offered.is_empty() {
                None
            } else {
                offers(&intent.action).map(|template| template.replace("%s", &labels(offered)))
            };
            if taking && !carried.is_empty() {
                let held = format!("You are already carrying: {}.", labels(carried));
                offer = Some(match offer {
                    Some(lying) => format!("{lying} {held}"),
                    None => held,
                });
            }
            return Some(Refusal::of(
                "unresolved",
                typed,
                fact.unwrap_or(NOTHING_MATCHED).to_string(),
                offer,
            ));
        }
        if intent.unread == Some(Unread::Unsure) {
            let offer = slash_line(intent).map_or(PLAIN_ACTION.to_string(), |line| guessed(&line));
            return Some(Refusal::of(
                UNSURE,
                typed,
                UNSURE_FACT.to_string(),
                Some(offer),
            ));
        }
        if intent.unreadable() {
            let fact = match intent.unread {
                Some(Unread::Throw) => THROW_UNREAD,
                Some(Unread::Dress) => DRESS_UNREAD,
                Some(Unread::ReaderDown) => READER_DOWN,
                Some(Unread::Unsure) | None => UNREADABLE,
            };
            return Some(Refusal::of("unreadable", typed, fact.into(), None));
        }
        if intent.throws_at_nothing() {
            let fact = match &intent.item {
                None => "Nothing was thrown: that did not resolve to anything in your hands or \
                         lying here."
                    .to_string(),
                Some(item) => format!(
                    "Nothing was thrown: {} {}. A throw is aimed at somebody here or through a \
                     way out, and that did not resolve to either.",
                    item.definite_name(),
                    if item.carried() {
                        "stays in your hands"
                    } else {
                        "stays where it is lying"
                    }
                ),
            };
            return Some(Refusal::of("unresolved", typed, fact, None));
        }
        if intent.moves_the_immovable() {
            let thing = intent.item.as_ref().and_then(Record::thing)?;
            let attempt = if intent.is_throw() {
                "it cannot be picked up and thrown at all, so no die was thrown for it"
            } else {
                "it cannot be picked up"
            };
            let fact = format!(
                "{} is {} and does not move for anybody: {attempt}.",
                upcase_first(&thing.definite_name()),
                thing.bulk
            );
            return Some(Refusal::of("immovable", typed, fact, None));
        }
        if let Some(worn) = intent.lets_go_of_the_worn() {
            return Some(Refusal::of("unplayable", typed, wearing(worn), None));
        }
        None
    }

    /// What a line that only leaves the room the player stands in is told,
    /// or `None` where the engine walks it through the one way out. `here`
    /// is that room's name and `exits` its ways out.
    pub fn leaving(leave: &Leave, typed: &str, here: &str, exits: &[Record]) -> Option<Refusal> {
        let fact = match leave {
            Leave::Through(_) => return None,
            Leave::Nowhere => empty("move").unwrap_or(NOTHING_MATCHED).to_string(),
            Leave::Which => ONE_OF_SEVERAL.to_string(),
            Leave::Either(thing) => format!(
                "That could mean leaving {here} or putting down {}.",
                own_name(thing)
            ),
        };
        let offer = (!exits.is_empty())
            .then(|| offers("move").map(|template| template.replace("%s", &labels(exits))))
            .flatten();
        Some(Refusal::of("unresolved", typed, fact, offer))
    }

    /// The act this game cannot perform at all, or `None` when it can: a
    /// story nobody plays has no hands, and a game standing nowhere has no
    /// floor.
    pub fn unplayable(
        action: &str,
        has_protagonist: bool,
        stands_somewhere: bool,
        typed: &str,
    ) -> Option<Refusal> {
        if !has_protagonist {
            if let Some(missing) = no_protagonist(action) {
                let fact = format!("{missing}: {NO_PROTAGONIST_FACT}");
                return Some(Refusal::of("unplayable", typed, fact, None));
            }
        }
        if !stands_somewhere {
            if let Some(missing) = nowhere(action) {
                let fact = format!("{missing}: {NOWHERE_FACT}");
                return Some(Refusal::of("unplayable", typed, fact, None));
            }
        }
        None
    }

    /// A thing the player cannot come to hold (`carry::fits`), in the
    /// rule's own reason: `held` is what the rule counts, each where it is,
    /// and `carried` everything in the player's hands, which the offer
    /// lists.
    pub fn cannot_carry(
        typed: &str,
        thing: &Thing,
        refused: Refused,
        strength: i64,
        held: &[(Thing, Place)],
        carried: &[Record],
    ) -> Refusal {
        let name = thing.own_name();
        let in_hand: Vec<&Thing> = held
            .iter()
            .filter(|(_, place)| *place == Place::Hands)
            .map(|(thing, _)| thing)
            .collect();
        let weighing: Vec<String> = held
            .iter()
            .filter(|(thing, _)| carry::weight(&thing.bulk) > 0)
            .map(|(thing, _)| thing.own_name())
            .collect();
        let fact = match refused {
            Refused::Immovable => format!(
                "{} is {} and does not move for anybody: it cannot be picked up.",
                upcase_first(&name),
                thing.bulk
            ),
            Refused::Heavy { .. } if thing.bulk == "heavy" => format!(
                "{} is heavy. At strength {strength} you cannot lift something that heavy.",
                upcase_first(&name)
            ),
            Refused::Heavy { .. } => format!(
                "{} is more than you can lift at strength {strength}.",
                upcase_first(&name)
            ),
            Refused::Load { carried, bears, .. } if carried >= bears => format!(
                "You are already carrying as much as you can bear at strength {strength} ({}). \
                 Put something down first.",
                to_sentence(&weighing)
            ),
            Refused::Load { .. } => format!(
                "{} is more than you can bear at strength {strength} on top of {}. Put \
                 something down first.",
                upcase_first(&name),
                to_sentence(&weighing)
            ),
            Refused::Hands { free: 0, .. } => format!(
                "Your hands are full: you are holding {}, and there is no room about you for \
                 {name}. Put something down first.",
                holding(&in_hand)
            ),
            Refused::Hands { .. } => format!(
                "{} takes both hands, and you are holding {}. Put something down first.",
                upcase_first(&name),
                holding(&in_hand)
            ),
        };
        let offer = (!carried.is_empty()).then(|| {
            offers("drop")
                .expect("a drop offers what is carried")
                .replace("%s", &labels(carried))
        });
        Refusal::of("cannot_carry", typed, fact, offer)
    }

    /// `/end` typed while the story has no ending due.
    pub fn nothing_due(typed: &str) -> Refusal {
        Refusal::of("unplayable", typed, NOTHING_DUE.to_string(), None)
    }

    /// A line typed after the player died.
    pub fn dead(typed: &str, character: Option<&Person>) -> Refusal {
        Refusal::of("dead", typed, death_sentence(character), None)
    }

    /// A line typed into a finished game, in the words for why it is over.
    pub fn over(ended: Ended, character: Option<&Person>, typed: &str) -> Refusal {
        match ended {
            Ended::Concluded => {
                Refusal::of("concluded", typed, story_over_sentence(character), None)
            }
            Ended::Died => Refusal::dead(typed, character),
            Ended::Unrecorded => Refusal::of("stopped", typed, stopped_sentence(character), None),
        }
    }

    pub fn game_over(&self) -> bool {
        GAME_OVER.contains(&self.kind.as_str())
    }

    /// Whether [`UNCHANGED`] follows the words: not where the game is over,
    /// and not where the words already say that nothing happened.
    fn says_unchanged(&self) -> bool {
        !self.game_over() && self.kind != UNSURE
    }

    /// The fact alone, for a reader that prints the room's lists beneath it.
    pub fn reason(&self) -> String {
        if self.says_unchanged() {
            format!("{} {UNCHANGED}", self.fact)
        } else {
            self.fact.clone()
        }
    }

    /// The fact, what would have worked, and that nothing changed.
    pub fn text(&self) -> String {
        let mut parts = vec![self.fact.clone()];
        parts.extend(self.offer.clone());
        if self.says_unchanged() {
            parts.push(UNCHANGED.to_string());
        }
        parts
            .into_iter()
            .filter(|part| !crate::text::is_blank(part))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// A `tear`, `smash` or `burn` of a thing an open beat of the story is bound
/// to (the beat guard, [`crate::divide`]).
pub fn still_needed(thing: &crate::room::Thing, kind: &str) -> String {
    let done = match kind {
        crate::divide::TEAR => "torn up",
        crate::divide::SMASH => "smashed",
        _ => "burned",
    };
    format!(
        "The story still needs {}, so it cannot be {done} yet.",
        thing.definite_name()
    )
}

/// A take of a thing the player already holds.
pub fn already_carrying(held: &Record) -> String {
    format!("You are already carrying {}.", own_name(held))
}

/// A hand act on a thing the player is wearing: it comes off first, as an
/// act of its own.
pub fn wearing(worn: &Thing) -> String {
    format!(
        "You are wearing {}. Take {} off first.",
        worn.own_name(),
        crate::wear::it(&worn.name)
    )
}

/// Why a `wear` or a `remove` of a thing is not played, in the engine's
/// words. `wearing` says which of the two the line asked for.
pub fn dressing(thing: &Thing, why: Dressing, wearing: bool) -> String {
    let name = thing.own_name();
    match why {
        Dressing::Unwearable if wearing => {
            format!("{} is not something you can wear.", upcase_first(&name))
        }
        Dressing::Unwearable | Dressing::NotWorn => format!("You are not wearing {name}."),
        Dressing::Worn => format!("You are already wearing {name}."),
        Dressing::Taken(other) => format!(
            "You are already wearing {} {}. Take {} off first.",
            other.own_name(),
            thing
                .wear_place()
                .map_or_else(String::new, crate::wear::on_a_body),
            crate::wear::it(&other.name)
        ),
        Dressing::Over(over) => format!("Take off {} first.", over.own_name()),
    }
}

/// A `wear` with no name, when nothing carried or lying here can be put on.
pub const NOTHING_TO_WEAR: &str = "You have nothing here you can wear.";

/// A `remove` with no name, when the player wears nothing.
pub const WEARING_NOTHING: &str = "You are not wearing anything.";

/// A `remove` whose name lands on nothing the player wears.
pub const NOT_WEARING_THAT: &str = "You are not wearing anything called that.";

fn own_name(record: &Record) -> String {
    match record.thing() {
        Some(thing) => thing.own_name(),
        None => record.definite_name(),
    }
}

/// What is in the player's hands, by name: a thing that takes both of them
/// held alone says so.
fn holding(in_hand: &[&Thing]) -> String {
    match in_hand {
        [one] if carry::hands_for(&one.bulk) > 1 => {
            format!("{}, which takes both hands", one.own_name())
        }
        _ => to_sentence(
            &in_hand
                .iter()
                .map(|thing| thing.own_name())
                .collect::<Vec<_>>(),
        ),
    }
}

fn phrase(action: &str, record: Option<&Record>) -> String {
    asked(action).replace("%s", &record.map(Record::label).unwrap_or_default())
}

fn labels(records: &[Record]) -> String {
    records
        .iter()
        .map(Record::label)
        .collect::<Vec<_>>()
        .join(", ")
}

fn present_name(character: Option<&Person>) -> Option<&str> {
    character
        .map(|person| person.fullname.as_str())
        .filter(|name| !crate::text::is_blank(name))
}

/// What a dead player is told.
pub fn death_sentence(character: Option<&Person>) -> String {
    let who = present_name(character).unwrap_or("You");
    let verb = if character.is_some() { "is" } else { "are" };
    format!(
        "{who} {verb} dead, and this playthrough is over. Nothing you type can change it. \
         Start a new playthrough to play this world again."
    )
}

/// What a player whose story ended is told.
pub fn story_over_sentence(character: Option<&Person>) -> String {
    let whose = match present_name(character) {
        Some(name) => format!("{name}'s story"),
        None => "Your story".to_string(),
    };
    format!(
        "{whose} is over, and this playthrough with it. Nothing you type can change how it \
         ended. Start a new playthrough to play this world again."
    )
}

/// What a player whose game stopped with neither an ending nor a death is told.
pub fn stopped_sentence(character: Option<&Person>) -> String {
    let whose = match present_name(character) {
        Some(name) => format!("{name}'s story"),
        None => "Your story".to_string(),
    };
    format!(
        "{whose} stopped before it reached an ending, and this playthrough with it. Nothing \
         you type can change it. Start a new playthrough to play this world again."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Thing;

    fn thing(id: i64, name: &str, carried: bool) -> Record {
        Record::Thing(Thing::new(id, name, carried))
    }

    fn take(offered: &[Record], carried: &[Record], named: Option<&Record>) -> String {
        Refusal::for_intent(&Intent::new("take"), "a line", offered, carried, named)
            .expect("a take of nothing is refused")
            .text()
    }

    #[test]
    fn a_take_of_a_carried_thing_says_it_is_carried() {
        let daybook = thing(1, "Ward Office 12 daybook", true);
        let lying = [thing(2, "ward stamp", false)];
        assert_eq!(
            take(&lying, std::slice::from_ref(&daybook), Some(&daybook)),
            "You are already carrying the Ward Office 12 daybook. Nothing has changed."
        );
    }

    #[test]
    fn a_take_that_names_nothing_carried_lists_what_is() {
        let carried = [
            thing(1, "Ward Office 12 daybook", true),
            thing(3, "Perrin's private index", true),
        ];
        let lying = [
            thing(2, "blank closure writ", false),
            thing(4, "ward stamp", false),
        ];
        assert_eq!(
            take(&lying, &carried, None),
            "That did not resolve to anything lying here. Lying here: blank closure writ, \
             ward stamp. You are already carrying: Ward Office 12 daybook, Perrin's private \
             index. Nothing has changed."
        );
        assert_eq!(
            take(&[], &carried[..1], None),
            "There is nothing lying here to pick up. You are already carrying: Ward Office 12 \
             daybook. Nothing has changed."
        );
        assert_eq!(
            take(&lying[..1], &[], None),
            "That did not resolve to anything lying here. Lying here: blank closure writ. \
             Nothing has changed."
        );
    }

    fn bulky(id: i64, name: &str, bulk: &str) -> Thing {
        Thing {
            bulk: bulk.to_string(),
            ..Thing::new(id, name, false)
        }
    }

    fn cannot_carry(thing: &Thing, refused: Refused, strength: i64, held: &[Thing]) -> String {
        let body = carry::Body::bare(strength);
        let bulks: Vec<&str> = held.iter().map(|thing| thing.bulk.as_str()).collect();
        let held: Vec<(Thing, Place)> = held
            .iter()
            .cloned()
            .zip(carry::arrange(&bulks, &body).into_iter().map(|h| h.place))
            .collect();
        let carried: Vec<Record> = held
            .iter()
            .map(|(thing, _)| Record::Thing(thing.clone()))
            .collect();
        Refusal::cannot_carry("a line", thing, refused, strength, &held, &carried).text()
    }

    #[test]
    fn a_thing_that_does_not_fit_is_refused_in_its_reason_with_what_is_carried() {
        let slate = bulky(1, "Assize tide-slate", "heavy");
        let lantern = bulky(2, "lantern", "handy");
        let daybook = bulky(3, "daybook", "handy");
        let stamp = bulky(4, "ward stamp", "handy");
        let cord = bulky(5, "Neb's knotted cord", "light");
        assert_eq!(
            cannot_carry(
                &lantern,
                Refused::Hands { free: 0, needs: 1 },
                13,
                &[slate.clone(), cord.clone()]
            ),
            "Your hands are full: you are holding the Assize tide-slate, which takes both \
             hands, and there is no room about you for the lantern. Put something down first. \
             You are carrying: Assize tide-slate, Neb's knotted cord. Nothing has changed."
        );
        assert_eq!(
            cannot_carry(
                &slate,
                Refused::Hands { free: 1, needs: 2 },
                13,
                std::slice::from_ref(&daybook)
            ),
            "The Assize tide-slate takes both hands, and you are holding the daybook. Put \
             something down first. You are carrying: daybook. Nothing has changed."
        );
        assert_eq!(
            cannot_carry(
                &slate,
                Refused::Heavy {
                    weight: 4,
                    bears: 3
                },
                6,
                &[]
            ),
            "The Assize tide-slate is heavy. At strength 6 you cannot lift something that \
             heavy. Nothing has changed."
        );
        assert_eq!(
            cannot_carry(
                &lantern,
                Refused::Heavy {
                    weight: 1,
                    bears: 0
                },
                1,
                &[]
            ),
            "The lantern is more than you can lift at strength 1. Nothing has changed."
        );
        assert_eq!(
            cannot_carry(
                &lantern,
                Refused::Load {
                    carried: 2,
                    weight: 1,
                    bears: 2
                },
                5,
                &[daybook.clone(), stamp.clone(), cord.clone()]
            ),
            "You are already carrying as much as you can bear at strength 5 (the daybook and \
             the ward stamp). Put something down first. You are carrying: daybook, ward stamp, \
             Neb's knotted cord. Nothing has changed."
        );
        assert_eq!(
            cannot_carry(
                &slate,
                Refused::Load {
                    carried: 1,
                    weight: 4,
                    bears: 4
                },
                8,
                std::slice::from_ref(&daybook)
            ),
            "The Assize tide-slate is more than you can bear at strength 8 on top of the \
             daybook. Put something down first. You are carrying: daybook. Nothing has changed."
        );
    }

    #[test]
    fn a_name_with_its_owner_in_it_takes_no_article() {
        assert_eq!(
            already_carrying(&thing(3, "Perrin's private index", true)),
            "You are already carrying Perrin's private index."
        );
        assert_eq!(
            already_carrying(&thing(5, "the clerk's ledger", true)),
            "You are already carrying the clerk's ledger."
        );
        assert_eq!(
            already_carrying(&thing(6, "Ward Office 12 daybook", true)),
            "You are already carrying the Ward Office 12 daybook."
        );
    }
}
