//! `Location::Generator`'s requests: the prompt that writes a room out in
//! full, and the one that asks for its ways out. They build the request and
//! send nothing.

use crate::arrival::prompt_details;
use crate::cast::{MAX_PER_ROOM, MAX_PER_STORY};
use crate::data;
use crate::interior::MAX_EXITS;
use crate::plan::{box_of, interior, Plan};
use crate::records::{id, int, string, text, Records, Row};
use crate::schemas;
use crate::text::{is_blank, natural_key, presence};
use serde_json::{json, Value};

/// How many things may lie loose in one room, and in one story
/// (`Item::Registry::MAX_PER_ROOM`, `MAX_PER_STORY`).
pub const ITEMS_PER_ROOM: i64 = 3;
pub const ITEMS_PER_STORY: i64 = 60;

/// `Item.bespoke`: a thing the caps count -- portable, and written by the
/// room writer or a world file rather than a kit.
pub fn bespoke(item: &Row) -> bool {
    text(item, "tier") != Some(crate::kit::FIXTURE) && text(item, "kit_key").is_none()
}

/// `Character::PURSUITS`: each label a desire is pursued by, and what it
/// means.
pub const PURSUITS: [(&str, &str); 7] = [
    (
        "keep",
        "They are holding on to something they already have.",
    ),
    (
        "obtain",
        "They are trying to get hold of something they do not have.",
    ),
    ("reach", "They are trying to get to a particular place."),
    (
        "attend",
        "They are trying to stay near a particular person.",
    ),
    (
        "avoid",
        "They are trying to get away from a person or a place.",
    ),
    (
        "withhold",
        "They are trying to stop somebody else getting something.",
    ),
    (
        "offer",
        "They are trying to put something into somebody else's hands.",
    ),
];

/// One person the room is written with, already decided: a race's name, an
/// age and a sex as stored.
#[derive(Clone, Debug)]
pub struct Slot {
    pub race: String,
    pub age: i64,
    pub sex: String,
}

/// A room being written, and the people the engine rolled for it.
pub struct Realization<'a> {
    pub records: &'a Records,
    pub location: &'a Row,
    pub slots: Vec<Slot>,
}

fn pluralize(word: &str, count: i64) -> String {
    match (word, count) {
        (_, 1) => word.to_string(),
        ("person", _) => "people".into(),
        _ => format!("{word}s"),
    }
}

fn ordinalize(n: usize) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

/// `Location::Interior.placeholder_name?`: whether a name is the shape
/// "<place> room <number>".
pub fn placeholder_name(place: &Row, name: &str) -> bool {
    let key = natural_key(name);
    let prefix = format!("{} room ", natural_key(string(place, "name")));
    key.strip_prefix(&prefix)
        .is_some_and(|number| !number.is_empty() && number.chars().all(|c| c.is_ascii_digit()))
}

impl<'a> Realization<'a> {
    fn story(&self) -> &'a Row {
        self.records
            .find("stories", int(self.location, "story_id").unwrap())
            .expect("the room's story")
    }

    fn story_id(&self) -> Option<i64> {
        int(self.location, "story_id")
    }

    fn locations(&self) -> Vec<&'a Row> {
        let story = self.story_id();
        self.records
            .select("locations", |row| int(row, "story_id") == story)
    }

    fn characters(&self) -> Vec<&'a Row> {
        let story = self.story_id();
        self.records
            .select("characters", |row| int(row, "story_id") == story)
    }

    /// `Location#place?`: a footprint and no position.
    fn place(&self) -> bool {
        interior(self.location) && box_of(self.location).is_none()
    }

    fn parent(&self) -> Option<&'a Row> {
        int(self.location, "parent_location_id")
            .and_then(|parent| self.records.find("locations", parent))
    }

    /// `Location::RoomName.for`: the place this room is a placeholder of.
    fn naming(&self) -> Option<&'a Row> {
        box_of(self.location)?;
        let place = self.parent()?;
        placeholder_name(place, string(self.location, "name")).then_some(place)
    }

    /// The story's template items (`Item.in_story(story).templates`).
    fn story_items(&self) -> Vec<&'a Row> {
        let characters: Vec<i64> = self.characters().iter().map(|row| id(row)).collect();
        let locations: Vec<i64> = self.locations().iter().map(|row| id(row)).collect();
        self.records.select("items", |item| {
            int(item, "playthrough_id").is_none()
                && (int(item, "character_id").is_some_and(|c| characters.contains(&c))
                    || int(item, "location_id").is_some_and(|l| locations.contains(&l)))
        })
    }

    fn exits(&self) -> Vec<&'a Row> {
        let here = Some(id(self.location));
        let mut rooms: Vec<&'a Row> = self
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id") == here
            })
            .iter()
            .map(|edge| {
                self.records
                    .find("locations", int(edge, "connected_location_id").unwrap())
                    .unwrap()
            })
            .collect();
        rooms.sort_by_key(|room| id(room));
        rooms
    }

    fn connected(&self, other: &Row) -> bool {
        let (here, there) = (Some(id(self.location)), Some(id(other)));
        self.records
            .first("location_connections", |edge| {
                let (from, to) = (int(edge, "location_id"), int(edge, "connected_location_id"));
                (from == here && to == there) || (from == there && to == here)
            })
            .is_some()
    }

    fn story_context(&self) -> String {
        let story = self.story();
        let universe = self
            .records
            .find("universes", int(story, "universe_id").unwrap())
            .expect("the story's universe");
        format!(
            "## Universe Details\n{}\n\n## Story Details\ntitle: {}\ngenre: {}\npreface: {}\nsummary: {}\n{}",
            prompt_details(
                self.records,
                universe,
                &["physics", "technology", "geographies", "race_names", "civilizations"]
            ),
            string(story, "title"),
            string(story, "genre"),
            string(story, "preface"),
            string(story, "summary"),
            self.arc_block()
        )
    }

    /// `#open_step`: the main arc's first step that still wants a row.
    fn open_step(&self) -> Option<&'a Row> {
        let story = self.story_id();
        let arc = self.records.first("quests", |quest| {
            int(quest, "story_id") == story && int(quest, "parent_quest_id").is_none()
        })?;
        if text(arc, "status") == Some("doomed") {
            return None;
        }
        let mut steps = self
            .records
            .select("quest_steps", |step| int(step, "quest_id") == Some(id(arc)));
        steps.sort_by_key(|step| int(step, "position"));
        steps.into_iter().find(|step| {
            int(step, "target_id").is_none() && text(step, "trigger_kind") != Some("time_passed")
        })
    }

    fn arc_block(&self) -> String {
        let Some(step) = self.open_step() else {
            return String::new();
        };
        let target = string(step, "target_name");
        let wanted = match string(step, "trigger_kind") {
            "reach_location" => format!(
                "This story needs a PLACE called \"{target}\", somewhere a player can walk to. \
                 There is no such place in this world yet."
            ),
            "speak_to" => format!(
                "This story needs a PERSON called \"{target}\", standing somewhere a player can reach. \
                 There is nobody of that name in this world yet."
            ),
            "hold_item" => format!(
                "This story needs a THING called \"{target}\", lying somewhere a player can pick it up. \
                 There is no such thing in this world yet."
            ),
            _ => String::new(),
        };
        let teaser = presence(text(step, "teaser"))
            .map(|line| format!("{line}\n"))
            .unwrap_or_default();
        format!(
            "\n## Where This Story Is Going\nnext: {}\n{wanted}\n{teaser}If this room is a natural place for it, this is where it\n\
             comes from; if it is not, leave it and somewhere else will do. Do not bend\n\
             this room to fit it.\n",
            string(step, "summary")
        )
    }

    fn geometry_facts(&self) -> String {
        let Some(plan) = Plan::of(self.records, self.location) else {
            return String::new();
        };
        format!(
            "## Where This Room Is\n\
             The game's own records of this room, already decided and not yours to\n\
             change. Write the room around them: do not contradict a measurement, and\n\
             do not give it a way out this list does not have.\n{}",
            plan.to_prompt()
        )
        .trim_end()
        .to_string()
    }

    fn name_instruction(&self) -> String {
        let Some(place) = self.naming() else {
            return String::new();
        };
        let name = string(place, "name");
        let named: Vec<&str> = self
            .records
            .select("locations", |row| {
                int(row, "parent_location_id") == Some(id(place)) && id(row) != id(self.location)
            })
            .iter()
            .map(|row| string(row, "name"))
            .filter(|sibling| !is_blank(sibling) && !placeholder_name(place, sibling))
            .collect();
        let note = if named.is_empty() {
            String::new()
        } else {
            format!(
                ". Rooms of {name} that are already named, so do not reuse one: {}",
                named.join("; ")
            )
        };
        format!(
            "\n- NAME THIS ROOM. It is one room inside {name}. The player reads the room and\n  \
             the place together -- \"the <your name> of {name}\" -- so name the ROOM only,\n  \
             and never put the place's own name into it. A short noun phrase carrying the\n  \
             article English wants on it, 2 to 4 words: \"the counting room\", \"the cold\n  \
             store\", \"the harbourmaster's office\". Name it for what the floor plan above\n  \
             says this room is and for what you have just described standing in it. Never\n  \
             a comma in it{note}"
        )
    }

    /// The world's own things lying in this room, in the order they were
    /// written (`Item.lying_in(location).templates.order(:id)`).
    fn lying_here(&self) -> Vec<&'a Row> {
        let here = Some(id(self.location));
        let mut rows = self.records.select("items", |item| {
            text(item, "disposition") == Some("intact")
                && int(item, "location_id") == here
                && int(item, "character_id").is_none()
                && int(item, "playthrough_id").is_none()
        });
        rows.sort_by_key(|item| id(item));
        rows
    }

    /// `#already_here`: what the game has already put in the room, read off
    /// the records, or nothing for a room that holds none of it.
    fn already_here(&self) -> String {
        let here = self.lying_here();
        let fixture = |item: &Row| text(item, "tier") == Some(crate::kit::FIXTURE);
        let fixed: Vec<&Row> = here.iter().copied().filter(|item| fixture(item)).collect();
        let loose: Vec<&Row> = here
            .iter()
            .copied()
            .filter(|item| {
                !fixture(item)
                    && int(item, "within_id").is_none()
                    && text(item, "kit_key").is_some()
            })
            .collect();
        if fixed.is_empty() && loose.is_empty() {
            return String::new();
        }
        let definite = |item: &Row| crate::turn::thing_of(item, false).definite_name();
        let closed = |item: &Row| text(item, "holds") == Some("closed");
        let mut lines: Vec<String> = vec![
            "## Already Here, Decided By The Game".into(),
            "The game's own records of what is in this room, already decided and not yours".into(),
            "to change, and it will tell the player so. Write the room around them, and do".into(),
            "not add another piece of furniture or fixed thing a player could reach for.".into(),
        ];
        if !fixed.is_empty() {
            let names: Vec<String> = fixed
                .iter()
                .map(|item| {
                    if closed(item) {
                        format!("{} (unsearched)", string(item, "name"))
                    } else {
                        string(item, "name").to_string()
                    }
                })
                .collect();
            lines.push(format!("Fixed in place: {}.", names.join(", ")));
        }
        for piece in &fixed {
            let resting: Vec<&Row> = here
                .iter()
                .copied()
                .filter(|item| int(item, "within_id") == Some(id(piece)))
                .collect();
            let Some(first) = resting.first() else {
                continue;
            };
            let names: Vec<&str> = resting.iter().map(|item| string(item, "name")).collect();
            lines.push(format!(
                "{} {}: {}.",
                if text(first, "how") == Some("in") {
                    "In"
                } else {
                    "On"
                },
                definite(piece),
                names.join(", ")
            ));
        }
        if !loose.is_empty() {
            let names: Vec<&str> = loose.iter().map(|item| string(item, "name")).collect();
            lines.push(format!(
                "Loose, and could be picked up: {}.",
                names.join(", ")
            ));
        }
        let shut: Vec<String> = fixed
            .iter()
            .filter(|item| closed(item))
            .map(|item| definite(item))
            .collect();
        if let Some((last, rest)) = shut.split_last() {
            let (them, it) = if rest.is_empty() {
                (last.clone(), "it")
            } else {
                (format!("{} or {last}", rest.join(", ")), "them")
            };
            lines.push(format!(
                "Nobody has searched {them} yet, so do not say what is in {it}."
            ));
        }
        lines.push("Do not list any of them again as a thing lying here.".into());
        format!("\n{}", lines.join("\n"))
    }

    fn items_instructions(&self) -> String {
        let lying = self
            .lying_here()
            .into_iter()
            .filter(|item| bespoke(item))
            .count() as i64;
        let room = (ITEMS_PER_ROOM - lying).max(0);
        let items = self.story_items();
        let mut names: Vec<&str> = items
            .iter()
            .filter(|item| bespoke(item))
            .filter_map(|item| text(item, "name"))
            .collect();
        names.sort();
        names.dedup();
        let world = (ITEMS_PER_STORY - names.len() as i64).max(0);
        let allowance = room.min(world);
        if allowance == 0 {
            return "Do not list any items: this place already holds everything it can.".into();
        }
        let mut taken: Vec<&str> = self
            .characters()
            .iter()
            .take(20)
            .filter_map(|who| text(who, "fullname"))
            .collect();
        taken.extend(
            items
                .iter()
                .filter(|item| text(item, "kit_key").is_none())
                .take(20)
                .filter_map(|item| text(item, "name")),
        );
        taken.retain(|name| !is_blank(name));
        let known = if taken.is_empty() {
            String::new()
        } else {
            format!(
                ". Already spoken for in this story, so do not reuse: {}",
                taken.join(", ")
            )
        };
        // `#besides_already_here`: the floor list's first line in a furnished
        // room, and nothing in any other.
        let besides = if self.already_here().is_empty() {
            ""
        } else {
            ", besides the ones Already Here above: those are written already, \
             and a thing named again is not a new one"
        };
        format!(
            "## What Is Lying Here\n\
             List AT MOST {allowance} portable thing{} a player could pick up and carry away{besides}.\n\
             - Nothing is the right answer for most rooms. An empty list is a complete answer\n\
             - Only loose, portable things. Not the door, not the floor, not the machinery\n  \
             bolted to it -- something a person could put in a pocket or under an arm\n\
             - Each one must be consistent with the description you just wrote, and worth\n  \
             the player noticing\n\
             - Pick its use_kind from the supplied physical profiles. Ordinary is the\n  \
             default. Food and drink can be consumed without healing; healing is only\n  \
             a restorative dose this world permits. Firestarters burn combustible\n  \
             objects, levers pry jammed passages, lockpicks try locks, and a key only\n  \
             opens a passage explicitly matched to it. Mark combustible only when an\n  \
             ordinary firestarter can destroy the object. The engine owns the effects\n  \
             and their amounts; its description cannot grant extra powers\n\
             - If a thing has WRITING on it -- a note, a letter, a handbill, a label, a\n  \
             docket, a page, a sign -- mark it readable and WRITE OUT WHAT IS WRITTEN\n  \
             ON IT, exactly as it appears on the thing. The words themselves, not a\n  \
             description of them, and short enough to finish -- a few words, a line,\n  \
             or a few short lines. The game keeps those words and a player reading it\n  \
             twice reads the same ones\n\
             - Never name it after a person or after a place{known}",
            if allowance == 1 { "" } else { "s" }
        )
    }

    fn people_instructions(&self) -> String {
        let wanted = self.slots.len() as i64;
        if wanted == 0 {
            return data::location_generator("nobody_here").to_string();
        }
        let lines: Vec<String> = self
            .slots
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                format!(
                    "  the {} is {}, about {}, {}",
                    ordinalize(index + 1),
                    slot.race,
                    slot.age,
                    slot.sex
                )
            })
            .collect();
        format!(
            "## Who Is Here\n\
             Write EXACTLY {wanted} {} who {} in this place right now.\n\
             - Anyone you write is somebody the player can walk up to and talk to, so they\n  \
             have to have a reason to be standing here and an enduring aim that today's\n  \
             reason serves, threatens or exposes\n\
             - Do not write the player, and do not write somebody passing through\n\
             - Never give them the name of a place, of a thing, or any name already\n  \
             spoken for above\n\n\
             Who they are is already decided. Write these people and do not change them:\n{}\n\n{}",
            pluralize("person", wanted),
            if wanted == 1 { "is" } else { "are" },
            lines.join("\n"),
            desire_instructions(wanted > 1).trim_end()
        )
    }

    /// `#detail_prompt`.
    pub fn detail_prompt(&self) -> String {
        let place = format!(
            "## The Place\nname: {}\nteaser: {}\n",
            string(self.location, "name"),
            string(self.location, "teaser")
        );
        if self.place() {
            return format!(
                "{}\n\n{place}\n## Instructions\n\
                 Write this place out in full. It is a BUILDING -- somewhere with rooms\n\
                 inside it that a player walks into and moves around in.\n\
                 - The description is what the player reads as they come in. Address them as \"you\"\n\
                 - Describe what is here now, not the history -- the history is the lore\n\
                 - Stay consistent with the universe and with the teaser above\n\
                 - Do NOT describe the floor plan: how many rooms there are, where the stairs\n  \
                 are and which door leads where are the game's to decide, out of the answers\n  \
                 below, and it will tell you room by room as the player reaches them\n\
                 - Do not name anybody standing here and do not list anything lying here.\n  \
                 People and things belong to the rooms, and each room is written as it is\n  \
                 reached\n\
                 - Respect the stated length of each field\n\n{}\n",
                self.story_context(),
                data::location_generator("parameters_instructions")
            );
        }
        format!(
            "{}\n\n{place}{}{}\n## Instructions\n\
             Write this place out in full.\n\
             - The description is what the player reads on arrival. Address them as \"you\"\n\
             - Describe what is here now, not the history -- the history is the lore\n\
             - Stay consistent with the universe and with the teaser above\n\
             - Respect the stated length of each field{}\n\n{}\n\n{}\n",
            self.story_context(),
            self.geometry_facts(),
            self.already_here(),
            self.name_instruction(),
            self.items_instructions(),
            self.people_instructions()
        )
    }

    fn room_for_exits(&self) -> i64 {
        (MAX_EXITS as i64 - self.exits().len() as i64).max(0)
    }

    fn known_location_names(&self) -> String {
        let parent = int(self.location, "parent_location_id");
        self.locations()
            .into_iter()
            .filter(|place| id(place) != id(self.location))
            .filter(|place| {
                let elsewhere = interior(place)
                    && box_of(place).is_some()
                    && int(place, "parent_location_id").is_some_and(|p| Some(p) != parent);
                !elsewhere
            })
            .map(|place| {
                if text(place, "detail_level") == Some("realized") && !self.connected(place) {
                    format!(
                        "{} (already written -- do not open a new way into it)",
                        string(place, "name")
                    )
                } else {
                    string(place, "name").to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// `#exits_prompt`.
    pub fn exits_prompt(&self) -> String {
        let name = string(self.location, "name");
        let reachable: Vec<&str> = self
            .exits()
            .iter()
            .map(|room| string(room, "name"))
            .collect();
        let already = if reachable.is_empty() {
            "This room has no ways out yet.".to_string()
        } else {
            format!(
                "## Where This Room Already Leads\n{}\nThose exist already and do not need naming again. Do not contradict them.",
                reachable.iter().map(|name| format!("- {name}")).collect::<Vec<_>>().join("\n")
            )
        };
        let known = self.known_location_names();
        let room = self.room_for_exits();
        format!(
            "Now list the ways out of {name}.\n\n{already}\n\n\
             ## Places That Already Exist In This Story\n\
             Reuse a name from this list when an exit leads somewhere already known.\n\
             Only invent a name when the exit leads somewhere genuinely new.\n\
             A place marked (already written) has had its own ways out written down\n\
             already, so naming it here would open a door it does not have: leave it\n\
             out and name somewhere new instead.\n{}\n\n\
             ## Instructions\n\
             - Name AT MOST {room} {} out. That is what is left of this\n  \
             room's {MAX_EXITS}, not a target: fewer is a better answer than a door\n  \
             nobody needed\n\
             - Each exit is somewhere the player can reach directly from {name}\n\
             - One way out is a complete answer. A dead end, a cell, the bottom of a\n  \
             shaft: if the only way out is back the place the player came from, list\n  \
             that place and nothing else. Never invent a passage to reach a second\n\
             - When there is more than one, give the player a reason to prefer one\n  \
             over another\n\
             - Do not list {name} itself\n\
             - Say which of them have an INSIDE, and say NO INSIDE for almost all of\n  \
             them. Saying anything else makes the game build a whole floor plan of\n  \
             rooms in that place and send the player walking through them, so it is\n  \
             only ever right for a BUILDING somebody goes in at a door -- an inn, a\n  \
             keep, a counting house, a warren. A road, a shore, a clearing, a bridge,\n  \
             a square, a cave mouth, a stair, a courtyard: no inside. A room, an\n  \
             office, a hall, a chamber: no inside either, because those are already\n  \
             somewhere the player stands. Where it really is a building, pick the size\n  \
             it would really be rather than the most interesting one\n\
             - Distance and travel method must be consistent with the description you\n  \
             just wrote, and must be true in both directions -- the way back is the\n  \
             same edge\n\
             - Say how populated each place is, in one of the words offered. A place is\n  \
             peopled by what it is FOR: a market, a taproom, a guardhouse, a\n  \
             workshop, a shrine somebody keeps -- somewhere with a reason for\n  \
             somebody to be standing in it. A place is empty when nothing is asked of\n  \
             anybody there: a cellar, a back stair, a stretch of road, a room that is\n  \
             locked, a place the story has already emptied. Neither answer is the\n  \
             safe one\n\
             - Respect the stated length of each field\n",
            if known.is_empty() {
                "None yet."
            } else {
                &known
            },
            pluralize("way", room),
        )
    }

    /// The conversation the room writer's chat already holds, but its system
    /// message, as `{role, content}`.
    fn history(&self, chat: Option<&Row>) -> Vec<Value> {
        let Some(chat) = chat else {
            return Vec::new();
        };
        self.records
            .select("messages", |message| {
                int(message, "chat_id") == Some(id(chat))
            })
            .iter()
            .filter(|message| text(message, "role") != Some("system"))
            .map(|message| {
                let content = match text(message, "content") {
                    Some(content) => content.to_string(),
                    None => message
                        .get("content_raw")
                        .map(|raw| serde_json::to_string(raw).expect("JSON"))
                        .unwrap_or_default(),
                };
                json!({ "role": string(message, "role"), "content": content })
            })
            .collect()
    }

    /// The request the realization bench keeps: the detail request, or the
    /// exits request for a room whose detail is already written.
    pub fn request(&self, chat: Option<&Row>) -> Value {
        let retrying = self
            .location
            .get("generation_checkpoint")
            .is_some_and(|c| !c.is_null());
        let (schema, prompt) = if retrying {
            (schemas::location_exits(), self.exits_prompt())
        } else if self.place() {
            (schemas::location_place(), self.detail_prompt())
        } else {
            (
                schemas::detail(self.slots.len() as i64),
                self.detail_prompt(),
            )
        };
        json!({
            "system": data::location_generator("system_prompt"),
            "user": prompt,
            "schema": schema,
            "history": self.history(chat),
        })
    }
}

/// `Character::Desires.instructions(several_people:)`: the block both
/// generation prompts append.
pub fn desire_instructions(several_people: bool) -> String {
    let list: Vec<String> = PURSUITS
        .iter()
        .map(|(label, meaning)| format!("  {label:<9}{meaning}"))
        .collect();
    format!(
        "## Their Four Objects of Desire\n\n\
         Write four STORY-SCALE things this person is after, then pick two TURN-SCALE labels.\n\
         Work from everything above -- especially the years in their backstory, their repeated\n\
         choices, what they fear losing, the future they imagine, and where today's situation\n\
         presses on that life.\n\n\
         An object of desire must satisfy both tests:\n\
         1. It was shaping this person before the opening scene.\n\
         2. Finishing today's errand would not finish it; it can drive choices across the\n   \
         whole story.\n\n\
         The two labels are picked from this list and nothing else:\n\n{}\n\n\
         conscious_desire\n  \
         The future, standing, relationship, legacy, place or way of life they knowingly\n  \
         organize their choices around and would name if asked what they want from their\n  \
         life. Make it specific to this person and world. Do not substitute the clue,\n  \
         deadline, delivery, payment or room they are dealing with today. One sentence.\n\n\
         unconscious_desire\n  \
         The deeper reward their choices have pursued for years and they would deny: the\n  \
         recognition, absolution, dependence, belonging, power or intimacy beneath the\n  \
         conscious account. It must explain a repeated pattern in the backstory and pull at\n  \
         an angle to the conscious desire, not merely restate today's hidden motive. One\n  \
         sentence.\n\n\
         recognized_need\n  \
         The duty, oath, debt, craft, family burden or bodily discipline they believe they\n  \
         must keep over years whether they want to or not. It predates today's assignment,\n  \
         survives it, and can repeatedly obstruct the conscious desire. One sentence.\n\n\
         unrecognized_need\n  \
         The enduring change, truth or relationship their life requires and their repeated\n  \
         pattern prevents them from seeing. A reader can infer it from the backstory. It\n  \
         cannot be completed by one confession, realization or errand: it demands a new way\n  \
         of choosing across the story. If they never move toward it, pursuit of the\n  \
         conscious desire ruins them. One sentence.\n\n\
         desire_pursuit\n  \
         Pick the listed ENGINE ACT that could repeatedly advance or protect the conscious\n  \
         desire in ordinary rooms. The label is the next-step expression of the larger\n  \
         desire, not its timescale.\n\n\
         need_pursuit\n  \
         Pick the listed ENGINE ACT that could repeatedly move them toward the unrecognized\n  \
         need in ordinary rooms. It may match desire_pursuit, but a different label should\n  \
         reflect a real conflict rather than manufactured variety.\n\n\
         Rules:\n\
         - Every field names something to move TOWARD. State what lies beyond a fear, refusal\n  \
         or escape.\n\
         - The object is larger than any single room act. Picking something up, handing it\n  \
         over, walking, waiting or staying near somebody may advance, protect, rehearse or\n  \
         betray it; that act must not become the whole desire.\n\
         - Use today's room as pressure or evidence, not as the horizon of the person's life.\n\
         - Keep the four consistent with the backstory, fears and dislikes. A fear is often\n  \
         the shadow of an object, but is not the object itself.\n\
         - The conscious desire and unrecognized need must not be the same thing said twice.\n\
         - Write in the third person, by name.\n\
         - Do not resolve any object or say how the story ends.\n\
         - Respect the stated length of each field.\n{}\n",
        list.join("\n"),
        if several_people {
            data::desires("more_than_one")
        } else {
            ""
        }
    )
}

/// How many more people a room and its story may hold
/// (`Character::Registry#room_for_people`, `#world_for_people`).
pub fn room_for_people(records: &Records, location: &Row) -> (i64, i64) {
    let here = Some(id(location));
    let story = int(location, "story_id");
    let present = records
        .select("characters", |row| int(row, "location_id") == here)
        .len() as i64;
    let in_story = records
        .select("characters", |row| int(row, "story_id") == story)
        .len() as i64;
    (
        (MAX_PER_ROOM - present).max(0),
        (MAX_PER_STORY - in_story).max(0),
    )
}
