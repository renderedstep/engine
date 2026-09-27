//! What one typed line turned out to be, whichever reader read it, and how
//! a model's answer is resolved back to the room's records.

use crate::room::{Choice, Record, Room, INTENTS, NOTHING};
use crate::text::{casecmp, ruby_strip};

/// The actions a reach that resolved to nothing is counted for.
pub const DRIFT_ACTIONS: &[&str] = &["move", "talk", "take", "drop", "attack", "use"];

/// Which slot an action's resolved record lands in. `use` binds a whole
/// attempt instead, and `other` and `throw` have no single slot.
pub fn slot_for(action: &str) -> Option<Slot> {
    match action {
        "move" => Some(Slot::Destination),
        "talk" | "attack" => Some(Slot::Speaker),
        "take" | "drop" | "examine" => Some(Slot::Item),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Destination,
    Speaker,
    Item,
}

/// One line, read. At most one of `destination`, `speaker` and `item` is
/// set, except on a throw, which names the thing in `item` and its aim in
/// `at`. `also_named` is a second name the line carried and the turn is not
/// acting on.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Intent {
    pub action: String,
    pub destination: Option<Record>,
    pub speaker: Option<Record>,
    pub item: Option<Record>,
    pub at: Option<Record>,
    pub also_named: Option<Record>,
    /// The word a model answered outside the table while still naming a
    /// record.
    pub unknown_action: Option<String>,
    pub physical: Option<Choice>,
}

impl Intent {
    pub fn new(action: &str) -> Intent {
        Intent {
            action: action.to_string(),
            ..Intent::default()
        }
    }

    /// The intent with `record` in `slot`.
    pub fn with(mut self, slot: Slot, record: Option<Record>) -> Intent {
        match slot {
            Slot::Destination => self.destination = record,
            Slot::Speaker => self.speaker = record,
            Slot::Item => self.item = record,
        }
        self
    }

    /// The record the turn acts on: the attempt's subject, or the one slot
    /// that is set. On a throw it is the thing thrown.
    pub fn subject(&self) -> Option<Record> {
        if let Some(choice) = &self.physical {
            if let Some(subject) = choice.subject() {
                return Some(subject);
            }
        }
        self.destination
            .clone()
            .or_else(|| self.speaker.clone())
            .or_else(|| self.item.clone())
    }

    pub fn is_throw(&self) -> bool {
        self.action == "throw"
    }

    /// The line named two things the records both have.
    pub fn named_more_than_one(&self) -> bool {
        self.also_named.is_some() && self.subject().is_some()
    }

    /// The line reached for a record and the closed set had none.
    pub fn reached_for_nothing(&self) -> bool {
        DRIFT_ACTIONS.contains(&self.action.as_str()) && self.subject().is_none()
    }

    /// The model answered outside the table.
    pub fn unreadable(&self) -> bool {
        self.unknown_action.is_some()
    }

    fn item_is_immovable(&self) -> bool {
        self.item
            .as_ref()
            .is_some_and(|item| item.thing().is_some_and(|thing| !thing.throwable()))
    }

    pub fn throws_the_immovable(&self) -> bool {
        self.is_throw() && self.item_is_immovable()
    }

    pub fn takes_the_immovable(&self) -> bool {
        self.action == "take" && self.item_is_immovable()
    }

    pub fn moves_the_immovable(&self) -> bool {
        self.throws_the_immovable() || self.takes_the_immovable()
    }

    /// A throw missing its thing or its aim.
    pub fn throws_at_nothing(&self) -> bool {
        self.is_throw() && (self.item.is_none() || self.at.is_none())
    }

    /// Whether the engine will refuse to play the line.
    pub fn refused(&self) -> bool {
        self.named_more_than_one()
            || self.reached_for_nothing()
            || self.unreadable()
            || self.throws_at_nothing()
            || self.moves_the_immovable()
    }
}

/// Whether an answer points at a record at all: `nothing` and a blank do
/// not.
fn named_something(value: Option<&str>) -> bool {
    let name = ruby_strip(value.unwrap_or(""));
    !name.is_empty() && name != NOTHING
}

fn find_exit(exits: &[Record], name: &str) -> Option<Record> {
    exits
        .iter()
        .find(|record| casecmp(&record.label(), name))
        .cloned()
}

/// A full name or a nickname; a person with no nickname answers to a blank.
fn find_character(cast: &[Record], name: &str) -> Option<Record> {
    cast.iter()
        .find(|record| match record {
            Record::Person(person) => {
                casecmp(&person.fullname, name)
                    || casecmp(person.nickname.as_deref().unwrap_or(""), name)
            }
            _ => false,
        })
        .cloned()
}

fn find_item(items: &[Record], name: &str) -> Option<Record> {
    items
        .iter()
        .find(|record| casecmp(&record.label(), name))
        .cloned()
}

type Finder = fn(&[Record], &str) -> Option<Record>;

/// A model's answer -- an intent, a target, a second name and a throw's aim
/// -- resolved against the room's four closed sets and its attempts.
pub fn build_intent(
    room: &Room,
    intent: Option<&str>,
    target: Option<&str>,
    also: Option<&str>,
    thrown_at: Option<&str>,
) -> Intent {
    let exits = room.exits_here();
    let cast = room.characters_here();
    let items = room.items_here();
    let carried = room.items_carried();

    let known = intent.is_some_and(|i| INTENTS.contains(&i));
    let action = if known { intent.unwrap_or("") } else { "other" };
    let name = target.unwrap_or("");

    if !known && named_something(Some(name)) {
        return Intent {
            unknown_action: Some(intent.unwrap_or("").to_string()),
            ..Intent::new(action)
        };
    }

    if action == "use" {
        let choices = room.physical_actions();
        let found = choices.iter().find(|c| c.token() == name).cloned();
        let extra = choices
            .iter()
            .find(|c| Some(c.token().as_str()) == also && Some(*c) != found.as_ref());
        let extra_record = match extra {
            Some(choice) => choice.subject(),
            None => [&exits[..], &cast, &items, &carried]
                .concat()
                .into_iter()
                .find(|record| {
                    Some(record.label().as_str()) == also
                        && !found.as_ref().is_some_and(|f| f.records().contains(record))
                }),
        };
        return Intent {
            physical: found,
            also_named: extra_record,
            ..Intent::new(action)
        };
    }

    if action == "throw" {
        let aim = ruby_strip(thrown_at.unwrap_or(""));
        return Intent {
            item: find_item(&[&carried[..], &items].concat(), name),
            at: find_character(&cast, aim).or_else(|| find_exit(&exits, aim)),
            ..Intent::new(action)
        };
    }

    let (set, finder): (Vec<Record>, Finder) = match action {
        "move" => (exits, find_exit),
        "talk" | "attack" => (cast, find_character),
        "take" => (items, find_item),
        "drop" => (carried, find_item),
        "examine" => ([items, carried].concat(), find_item),
        _ => return Intent::new(action),
    };
    let slot = slot_for(action).expect("an action with a set has a slot");
    let found = finder(&set, name);
    let also_named = if named_something(also) {
        finder(&set, ruby_strip(also.unwrap_or(""))).filter(|other| Some(other) != found.as_ref())
    } else {
        None
    };
    Intent {
        also_named,
        ..Intent::new(action)
    }
    .with(slot, found)
}
