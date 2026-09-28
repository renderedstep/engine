//! What the engine says when it will not play a line, and the one author of
//! it. A refused line does nothing: the player is answered in the engine's
//! own words, built from records it already holds.

use crate::intent::Intent;
use crate::room::{Person, Record};
use crate::text::upcase_first;

pub const KINDS: &[&str] = &[
    "named_more_than_one",
    "unresolved",
    "immovable",
    "unreadable",
    "unplayable",
    "dead",
    "concluded",
];

/// The kinds that are a state of the game rather than a reading of a line.
const GAME_OVER: &[&str] = &["dead", "concluded"];

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

const UNREADABLE: &str = "That did not come back as anything the game knows how to do. \
     Say it again as one plain action -- go somewhere, talk to somebody, \
     take something, or put something down.";

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
    /// `offered` is the closed set the intent's action reads against.
    pub fn for_intent(intent: &Intent, typed: &str, offered: &[Record]) -> Option<Refusal> {
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
            let fact = if offered.is_empty() {
                empty(&intent.action)
            } else {
                missed(&intent.action)
            };
            let offer = if offered.is_empty() {
                None
            } else {
                offers(&intent.action).map(|template| template.replace("%s", &labels(offered)))
            };
            return Some(Refusal::of(
                "unresolved",
                typed,
                fact.unwrap_or(NOTHING_MATCHED).to_string(),
                offer,
            ));
        }
        if intent.unreadable() {
            return Some(Refusal::of("unreadable", typed, UNREADABLE.into(), None));
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
        None
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

    /// A line typed after the player died.
    pub fn dead(typed: &str, character: Option<&Person>) -> Refusal {
        Refusal::of("dead", typed, death_sentence(character), None)
    }

    /// A line typed into a finished game. It concluded when the game reached
    /// an ending; otherwise the player died.
    pub fn over(concluded: bool, character: Option<&Person>, typed: &str) -> Refusal {
        if concluded {
            Refusal::of("concluded", typed, story_over_sentence(character), None)
        } else {
            Refusal::dead(typed, character)
        }
    }

    pub fn game_over(&self) -> bool {
        GAME_OVER.contains(&self.kind.as_str())
    }

    /// The fact alone, for a reader that prints the room's lists beneath it.
    pub fn reason(&self) -> String {
        if self.game_over() {
            self.fact.clone()
        } else {
            format!("{} {UNCHANGED}", self.fact)
        }
    }

    /// The fact, what would have worked, and that nothing changed.
    pub fn text(&self) -> String {
        let mut parts = vec![self.fact.clone()];
        parts.extend(self.offer.clone());
        if !self.game_over() {
            parts.push(UNCHANGED.to_string());
        }
        parts
            .into_iter()
            .filter(|part| !crate::text::is_blank(part))
            .collect::<Vec<_>>()
            .join(" ")
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
