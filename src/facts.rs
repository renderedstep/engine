//! What a turn hands the narrator as already done, and the frame the
//! narration prompt puts around it (`Scene::Narrator#prompt_for` and
//! `Playthrough::Turn`'s `_fact` builders).
//!
//! Each builder is a function of the values it states and reads nothing
//! else, so the turn and [`crate::prompt_version`] say the same words: the
//! turn with the records it just wrote, the prompt version with fixed
//! placeholders standing where the records would go.

use crate::records::{flag, string, text, Row};
use crate::text::{is_blank, presence, upcase_first};

/// `Item#definite_name`: "the" in front of the name, with any article the
/// name arrived with taken off first.
pub fn definite_name(item: &Row) -> String {
    crate::moment::definite_name(string(item, "name"))
}

/// `Item#bare_name`: the name without an article it arrived with.
pub fn bare_name(item: &Row) -> String {
    let definite = definite_name(item);
    definite
        .strip_prefix("the ")
        .unwrap_or(&definite)
        .to_string()
}

/// `Scene::Narrator#prompt_for`: the moment, what the game already did,
/// the line about what kind of turn this is, and what the player typed.
pub fn framing(context: &str, command: &str, fact: Option<&str>, doing: Option<&str>) -> String {
    let fact = match fact.filter(|fact| !is_blank(fact)) {
        Some(fact) => format!(
            "\nWhat has ALREADY happened, recorded by the game: {fact}\nNarrate it as done. Do not contradict it and do not undo it.\n"
        ),
        None => String::new(),
    };
    let doing = match doing.and_then(crate::data::narrator_doing) {
        Some(sentence) => format!("\n{sentence}\n"),
        None => String::new(),
    };
    format!("{context}\n{fact}\n{doing}\nThe player types: {command}\n\nNarrate what happens.\n")
}

/// `Playthrough::Turn#written_words_fact`.
pub fn written_words(words: &str) -> String {
    format!(
        "This is exactly what is written on it, word for word: \"{words}\" -- those are \
         the words on it, and they do not change between readings. Quote them as they \
         are; do not add to them, and do not write different ones."
    )
}

/// `Playthrough::Turn#taken_fact`.
pub fn taken(item: &Row, taker: Option<&Row>, from: Option<&Row>) -> String {
    let lying = match from {
        Some(room) => format!("in {}", string(room, "name")),
        None => "in this room".into(),
    };
    let description = match presence(text(item, "description")) {
        Some(description) => format!(" -- {description}"),
        None => String::new(),
    };
    let inscribed = match presence(text(item, "inscription")).filter(|_| flag(item, "readable")) {
        Some(words) => format!(" {}", written_words(words)),
        None => String::new(),
    };
    format!(
        "ON THIS TURN, and not before it, {} picked {} up. Until this turn it was NOT in their \
         hands at all: it was lying {lying}. Now they are carrying it{description}. The picking up \
         is what has just happened and it is what to narrate. Do not write it as something they \
         already had, already held, or turn out to be holding. {} is the only thing that moved: \
         nothing else was lifted, opened, drawn out or taken into anybody's hands.{inscribed}",
        taker.map_or("", |who| string(who, "fullname")),
        definite_name(item),
        upcase_first(&definite_name(item)),
    )
}

/// `Playthrough::Turn#dropped_fact`.
pub fn dropped(item: &Row, here: &Row, dropper: Option<&Row>) -> String {
    format!(
        "ON THIS TURN, and not before it, {} put the {} down. Until this turn it WAS in their \
         hands: it is no longer carried, and it is now lying in {}, where it stays until somebody \
         picks it up. The putting down is what has just happened and it is what to narrate. Do \
         not write them picking it up or finding it. {} is the only thing that moved: nothing \
         else was lifted, opened, drawn out or taken into anybody's hands.",
        dropper.map_or("The party", |who| string(who, "fullname")),
        bare_name(item),
        string(here, "name"),
        upcase_first(&definite_name(item)),
    )
}

/// `Playthrough::Turn#read_fact`: a readable thing's words, quoted.
pub fn read(item: &Row, words: &str) -> String {
    format!(
        "{} has writing on it. {}",
        upcase_first(&definite_name(item)),
        written_words(words)
    )
}

/// How a throw came out, as `Playthrough::Turn#thrown_fact` states it.
#[derive(Clone, Copy, Debug)]
pub enum Throw<'a> {
    /// It hit the person named.
    Struck { target: &'a str },
    /// It went through the way out into the room named.
    Thrown { into: &'a str },
    /// It does not move for anybody, so no die was thrown.
    Immovable,
    /// The lift failed; `carried` says whether it is still in the party's
    /// hands rather than lying where it was.
    Fumbled { carried: bool },
}

/// `Playthrough::Turn#thrown_fact`: whether the thing left the hands, and
/// where it is now. `thrower` is the player's full name, or none for a
/// game nobody plays.
pub fn thrown(thrower: Option<&str>, thing: &str, bulk: &str, outcome: Throw) -> String {
    let who = thrower.unwrap_or("The party");
    match outcome {
        Throw::Struck { target } => format!(
            "{who} threw the {thing} at {target} and it hit them. The {thing} is NO LONGER CARRIED: it is lying \
             on the floor at {target}'s feet, where it stays until somebody picks it up."
        ),
        Throw::Thrown { into } => format!(
            "{who} threw the {thing} through the way out into {into}. The {thing} is NO LONGER CARRIED and is \
             no longer in this room at all: it is lying in {into}, where it stays until somebody picks it up."
        ),
        Throw::Immovable => format!(
            "{who} could not throw the {thing} at all: it is {bulk} and does not move. Nothing happened."
        ),
        Throw::Fumbled { carried } => format!(
            "{who} tried to pick up and throw the {thing} and could not get it moving: it is {bulk} and the \
             attempt failed. NOTHING WAS THROWN and nothing was hit -- the {thing} {}. The turn was \
             spent on the attempt.",
            if carried {
                "is still in the party's hands"
            } else {
                "is still lying exactly where it was"
            }
        ),
    }
}
