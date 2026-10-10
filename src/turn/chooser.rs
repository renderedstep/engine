//! The protagonist's act in a game the player narrates: the game picks it,
//! with one die, from the acts the player's own panels offer.
//!
//! THE LIST IS THE GLANCE'S. Every candidate is a target of
//! [`Glance::read`]'s verbs, which the turn's own refusal check already
//! accepted, so a pick is a line the turn plays for a typing player, through
//! the same journal, refusal check and writers. The chooser adds no rule; it
//! only weighs and picks:
//!
//! - **The verbs** are move, examine, take, drop and use. Talk and attack
//!   are never chosen, and neither is an offer, which plays as a
//!   conversation. A move into a room nobody has written is a candidate only
//!   when the caller says rooms may be written on this turn.
//! - **The pursuit.** Each candidate weighs the protagonist's own
//!   `desire_pursuit` row of the table the people in a room act by
//!   (`data/playthrough/volition/weights.yml`), through the shape its verb
//!   reads as ([`shape`]), plus that table's base. A protagonist with no
//!   pursuit the table names weighs every candidate the same.
//! - **The arc.** A candidate that serves the main arc's next beat gains
//!   [`ARC_BOOST`]: a take of the thing a `hold_item` beat asks for, and a
//!   move that is the first step of a shortest walk to the room a
//!   `reach_location` beat names, or to the room where that thing lies.
//! - **Novelty.** A line already played on this visit to the room weighs
//!   nothing, so the die does not inspect one slate four times. When every
//!   candidate weighs nothing, the die is even over them all.
//! - **The arc's things.** A thing an open arc asks the player to hold is
//!   never dropped, burned or eaten by the chooser, as nobody else may pick
//!   one up (`volition::arc_item_ids`).
//!
//! The die is `Roll::CHOICE`, seeded off the story's time and the submission
//! the pick answers, so the same submission picks the same act, and the
//! journal keeps the pick before the turn reads it, so a resumed turn plays
//! it again without throwing.

use super::{weight_of, weights_for, Mechanics};
use crate::data;
use crate::glance::Glance;
use crate::grammar;
use crate::intent::{slot_for, Intent};
use crate::records::{id, int, text};
use crate::room::{Record, Room};
use crate::{roll, volition};
use std::collections::{HashMap, VecDeque};

/// The line a submission carries when the player lets the game act. A game
/// the player narrates reads it as "choose for me"; any other game reads it
/// as a typed line, like any other.
pub const LINE: &str = "(the game acts)";

/// The reader label a chosen turn's scene carries (`scenes.resolved_by`).
pub const RESOLVED_BY: &str = "engine";

/// The verbs the game may choose.
pub const VERBS: [&str; 5] = ["move", "examine", "take", "drop", "use"];

/// What a candidate that serves the arc's next beat gains: more than any
/// weight the pursuit table gives, so the beat is the likeliest act without
/// being the only one.
pub const ARC_BOOST: i64 = 12;

/// Said when a game the player narrates has nothing the game may choose.
pub const NOTHING_TO_DO: &str = "There is nothing here the game can choose to do: nowhere to go, \
     and nothing to pick up, put down, look at or use.";

/// The shape of the people's act table a verb weighs as: a move is a move
/// and a take a take; a drop lets a thing go, as a give does; an examine
/// stays and changes nothing, as a wait does; and a use works on the
/// things at hand, as a take does.
pub fn shape(verb: &str) -> &'static str {
    match verb {
        "move" => "move",
        "drop" => "give",
        "examine" => "wait",
        _ => "take",
    }
}

/// One act the game could choose, and how it is weighed.
#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    pub verb: String,
    /// The record it acts on, by the name the glance lists it under.
    pub target: String,
    /// The glance target's id: the record's, or none for an attempt.
    pub id: Option<i64>,
    /// An attempt's `use:` token.
    pub token: Option<String>,
    /// The line that plays it, slashed.
    pub line: String,
    /// The pursuit's weight, before the boost and the novelty rule.
    pub pursued: i64,
    /// It serves the main arc's next beat.
    pub arc: bool,
    /// It was played already on this visit.
    pub repeated: bool,
    /// What the die weighs it at.
    pub weight: i64,
    pub intent: Intent,
}

/// Every act the game may choose now, weighed, in the glance's order.
/// `unwritten` lets a move into a room nobody has written be chosen.
pub fn candidates(mechanics: &Mechanics, unwritten: bool) -> Vec<Candidate> {
    if mechanics.over() {
        return Vec::new();
    }
    let glance = Glance::read(mechanics);
    let room = mechanics.room();
    let game = mechanics.game();
    let pursuit = game
        .protagonist()
        .and_then(|who| weights_for(text(who, "desire_pursuit")));
    let base = data::weights().base;
    let arc_items = volition::arc_item_ids(&game);
    let held_for_arc = |record: &Record| {
        record.thing().is_some_and(|thing| {
            arc_items.contains(&thing.id)
                || thing
                    .template
                    .is_some_and(|template| arc_items.contains(&template))
        })
    };
    let beat = Beat::of(mechanics);
    let played = played_on_this_visit(mechanics);
    let mut found = Vec::new();
    for verb in glance
        .verbs
        .iter()
        .filter(|verb| VERBS.contains(&verb.name.as_str()))
    {
        let offered = room.offered_for(&verb.name);
        for target in &verb.targets {
            let matching = |record: &&Record| match (&target.token, record.attempt()) {
                (Some(token), Some(choice)) => choice.token() == *token,
                (None, None) => record.id() == target.id,
                _ => false,
            };
            let Some(record) = offered.iter().find(matching) else {
                continue;
            };
            let intent = match record.attempt() {
                Some(choice) => {
                    let spends = matches!(choice.kind.as_str(), "consume" | "burn");
                    let spent = choice.item.clone().map(Record::Thing);
                    if choice.kind == "offer"
                        || (spends && spent.is_some_and(|item| held_for_arc(&item)))
                    {
                        continue;
                    }
                    Intent {
                        physical: Some(choice.clone()),
                        ..Intent::new("use")
                    }
                }
                None => {
                    if verb.name == "drop" && held_for_arc(record) {
                        continue;
                    }
                    if verb.name == "move" && !unwritten && !written(mechanics, record) {
                        continue;
                    }
                    let slot = slot_for(&verb.name).expect("every chosen verb but use has a slot");
                    Intent::new(&verb.name).with(slot, Some(record.clone()))
                }
            };
            let line = match (&target.line, &verb.word, record.attempt()) {
                (Some(line), _, _) => line.clone(),
                (None, Some(word), _) => format!("/{word} {}", target.name),
                (None, None, Some(choice)) => format!("/{} {}", choice.kind, choice.argument()),
                (None, None, None) => continue,
            };
            let pursued = pursuit.map_or(1, |row| base + weight_of(row, shape(&verb.name)));
            let arc = beat.served_by(mechanics, &room, &verb.name, record);
            let repeated = played.contains(&grammar::unslashed(&line));
            let weight = match (repeated, arc) {
                (true, _) => 0,
                (false, true) => pursued + ARC_BOOST,
                (false, false) => pursued,
            };
            found.push(Candidate {
                verb: verb.name.clone(),
                target: target.name.clone(),
                id: target.id,
                token: target.token.clone(),
                line,
                pursued,
                arc,
                repeated,
                weight,
                intent,
            });
        }
    }
    found
}

/// One weighted throw of `Roll::CHOICE` over [`candidates`], for the
/// submission `sequence`; none when there is nothing to choose.
pub fn choose(mechanics: &Mechanics, sequence: i64, unwritten: bool) -> Option<Candidate> {
    let mut found = candidates(mechanics, unwritten);
    if found.is_empty() {
        return None;
    }
    let weights: Vec<i64> = found.iter().map(|candidate| candidate.weight).collect();
    let mut rng = mechanics.generator(mechanics.story_now(), sequence, roll::CHOICE);
    let pick = roll::weighted_one_of(found.len(), &weights, &mut rng);
    Some(found.swap_remove(pick))
}

/// Whether the room a way out leads to is written.
fn written(mechanics: &Mechanics, record: &Record) -> bool {
    record
        .id()
        .and_then(|room| mechanics.records().find("locations", room))
        .is_some_and(|room| text(room, "detail_level") != Some("stub"))
}

/// The lines this game has played since it last walked into the room it
/// stands in, as each scene keeps what was typed.
fn played_on_this_visit(mechanics: &Mechanics) -> Vec<String> {
    let game = mechanics.game();
    let here = int(game.row, "current_location_id");
    let chain = game.scene_chain();
    let arrived = chain
        .iter()
        .rposition(|scene| {
            text(scene, "resolved_action") == Some("move") && int(scene, "location_id") == here
        })
        .map_or(0, |at| at + 1);
    chain[arrived..]
        .iter()
        .filter_map(|scene| text(scene, "typed"))
        .map(str::to_string)
        .collect()
}

/// The main arc's next beat, as far as the chooser can serve it.
enum Beat {
    None,
    /// The room a `reach_location` beat names, and its way in.
    Reach(Vec<i64>),
    /// The world's thing a `hold_item` beat names.
    Hold(i64),
}

impl Beat {
    fn of(mechanics: &Mechanics) -> Beat {
        let Some(step) = mechanics
            .open_main_arc()
            .and_then(|arc| mechanics.next_step(&arc))
        else {
            return Beat::None;
        };
        let Some(target) = int(&step, "target_id") else {
            return Beat::None;
        };
        match (text(&step, "trigger_kind"), text(&step, "target_type")) {
            (Some("reach_location"), Some("Location")) => {
                Beat::Reach(vec![target, mechanics.way_in(target)])
            }
            (Some("hold_item"), Some("Item")) => Beat::Hold(target),
            _ => Beat::None,
        }
    }

    fn served_by(&self, mechanics: &Mechanics, room: &Room, verb: &str, record: &Record) -> bool {
        let toward = |goals: &[i64]| {
            record
                .id()
                .is_some_and(|way| first_steps(mechanics, room, goals).contains(&way))
        };
        match (self, verb) {
            (Beat::Hold(template), "take") => record
                .thing()
                .is_some_and(|thing| thing.id == *template || thing.template == Some(*template)),
            (Beat::Reach(goals), "move") => toward(goals),
            (Beat::Hold(template), "move") => toward(&rooms_holding(mechanics, *template)),
            _ => false,
        }
    }
}

/// The rooms where this game's copy of the world's thing `template` lies on
/// a floor, or where the world's own row lies while the game has no copy.
fn rooms_holding(mechanics: &Mechanics, template: i64) -> Vec<i64> {
    let records = mechanics.records();
    let game = Some(mechanics.playthrough());
    let copies = records.select("items", |item| {
        int(item, "playthrough_id") == game && int(item, "template_id") == Some(template)
    });
    let lying = if copies.is_empty() {
        records.select("items", |item| id(item) == template)
    } else {
        copies
    };
    lying
        .iter()
        .filter(|item| int(item, "character_id").is_none())
        .filter_map(|item| int(item, "location_id"))
        .collect()
}

/// The ways out of `room` that begin a shortest walk, over the world's
/// doorways, to any of `goals`; none from a room that is a goal already.
fn first_steps(mechanics: &Mechanics, room: &Room, goals: &[i64]) -> Vec<i64> {
    let Some(start) = room.here.as_ref().map(|here| here.id) else {
        return Vec::new();
    };
    let reached = |at: i64| goals.contains(&at) || goals.contains(&mechanics.way_in(at));
    if goals.is_empty() || reached(start) {
        return Vec::new();
    }
    let edges = mechanics.records().table("location_connections");
    let ways_from = |at: i64| -> Vec<i64> {
        let mut ways: Vec<i64> = edges
            .iter()
            .filter(|edge| int(edge, "location_id") == Some(at))
            .filter_map(|edge| int(edge, "connected_location_id"))
            .collect();
        ways.sort_unstable();
        ways
    };
    // Each room reached, by the way out of `start` its walk began with.
    let mut began: HashMap<i64, i64> = HashMap::from([(start, start)]);
    let mut queue = VecDeque::new();
    for way in ways_from(start) {
        if let std::collections::hash_map::Entry::Vacant(entry) = began.entry(way) {
            entry.insert(way);
            queue.push_back((way, 1));
        }
    }
    let mut found = Vec::new();
    let mut shortest = None;
    while let Some((at, depth)) = queue.pop_front() {
        if shortest.is_some_and(|shortest| depth > shortest) {
            break;
        }
        if reached(at) {
            shortest = Some(depth);
            if !found.contains(&began[&at]) {
                found.push(began[&at]);
            }
            continue;
        }
        let first = began[&at];
        for way in ways_from(at) {
            if let std::collections::hash_map::Entry::Vacant(entry) = began.entry(way) {
                entry.insert(first);
                queue.push_back((way, depth + 1));
            }
        }
    }
    found
}
