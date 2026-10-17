//! What a turn hands the narrator as already done, and the frame the
//! narration prompt puts around it (`Scene::Narrator#prompt_for` and
//! `Playthrough::Turn`'s `_fact` builders).
//!
//! Each builder is a function of the values it states and reads nothing
//! else, so the turn and [`crate::prompt_version`] say the same words: the
//! turn with the records it just wrote, the prompt version with fixed
//! placeholders standing where the records would go.

use crate::records::{flag, string, text, Row};
use crate::text::{is_blank, presence, to_sentence, upcase_first};

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

/// A search of a fixed piece, or the opening of a shut one (`opened`): what
/// it found inside, lying here now and picked up by nobody. `closed` is
/// whether the piece was shut, which the search leaves standing open; a
/// hollow piece's inside lay in plain view and nothing about it opens.
pub fn searched(
    piece: &Row,
    found: &[Row],
    searcher: Option<&Row>,
    opened: bool,
    closed: bool,
) -> String {
    let verb = if opened { "opened" } else { "searched" };
    let open = if closed {
        format!(" {} now stands open.", upcase_first(&definite_name(piece)))
    } else {
        String::new()
    };
    let start = format!(
        "ON THIS TURN, and not before it, {} {verb} {}.",
        searcher.map_or("the player", |who| string(who, "fullname")),
        definite_name(piece),
    );
    if found.is_empty() {
        return format!(
            "{start} There was nothing inside it.{open} Nothing was found and nothing moved: the \
             search is what has just happened and it is what to narrate. Do not write anything \
             being found, taken out or carried."
        );
    }
    let names: Vec<String> = found.iter().map(definite_name).collect();
    let (were, them) = if found.len() == 1 {
        (
            "it was not there to be seen before this turn, and now it is",
            "it",
        )
    } else {
        (
            "those were not there to be seen before this turn, and now they are",
            "them",
        )
    };
    format!(
        "{start} Inside it they found {}, and nothing else: {were} lying here and takeable.{open} \
         Nothing was picked up: finding {them} is what has just happened and it is what to \
         narrate. Do not write anything else being found, taken out or carried.",
        to_sentence(&names),
    )
}

/// What became of a part somebody tried to break off a fixed piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snapped {
    /// The check failed, or no die could be thrown: it is still attached.
    Held,
    /// It came away and is in the hand.
    InHand,
    /// It came away brittle or fragile and broke as it came loose.
    InPieces,
}

/// An attempt to break a part off a fixed piece: the strength check, what
/// came away, and how many more of its kind the piece has left (`left`,
/// told only of a part that came away). `before` is how many of its kind
/// the piece had before the attempt: one of several is "a branch", the only
/// one "the bark". `check` is none where the breaker has no abilities.
pub fn broke_off(
    part: &Row,
    piece: &Row,
    breaker: Option<&Row>,
    check: Option<&crate::turn::Check>,
    outcome: Snapped,
    before: i64,
    left: i64,
) -> String {
    let who = breaker.map_or("the player", |who| string(who, "fullname"));
    let noun = bare_name(part);
    let named = if before > 1 {
        format!("{} {noun}", indefinite_article(&noun))
    } else {
        definite_name(part)
    };
    let the_piece = definite_name(piece);
    let start = if outcome == Snapped::Held {
        format!("ON THIS TURN, and not before it, {who} tried to break {named} off {the_piece}.")
    } else {
        format!("ON THIS TURN, and not before it, {who} broke {named} off {the_piece}.")
    };
    let rolled = check.map(|check| {
        let hold = match presence(text(part, "hold")) {
            Some(hold) => format!(" for {} {hold} hold", indefinite_article(hold)),
            None => String::new(),
        };
        let less = if check.penalty == 0 {
            String::new()
        } else {
            format!(" less {}", check.penalty)
        };
        match check.die {
            Some(die) => format!(
                " It was {} {} check, a d{} under {}{less}{hold}: it came up {die}, and {}.",
                indefinite_article(&check.ability),
                check.ability,
                crate::turn::CHECK_DIE,
                check.score,
                if check.passed() { "passed" } else { "failed" },
            ),
            None => format!(
                " It was beyond them: {} {}{less}{hold} leaves nothing to roll under.",
                check.ability, check.score,
            ),
        }
    });
    let rolled = rolled.unwrap_or_default();
    let it = upcase_first(&definite_name(part));
    match outcome {
        Snapped::Held => format!(
            "{start}{rolled} {it} held: it is still on {the_piece}, nothing came away and they are \
             holding nothing new. Do not write it breaking, bending away or coming loose."
        ),
        Snapped::InHand => format!(
            "{start}{rolled} {it} is in their hands now. {} has {} left that could be broken off. \
             Do not write anything else coming away, being picked up or being carried.",
            upcase_first(&the_piece),
            how_many(left, &noun),
        ),
        Snapped::InPieces => format!(
            "{start}{rolled} It came away in pieces: {}, it broke as it came loose, and they are \
             holding nothing new. There {} {} left on {the_piece} that could be broken off. Do not \
             write them holding it.",
            string(part, "fragility"),
            if left > 1 { "are" } else { "is" },
            how_many(left, &noun),
        ),
    }
}

/// "a" or "an", by the word's first letter.
fn indefinite_article(word: &str) -> &'static str {
    match word.chars().next().map(|c| c.to_ascii_lowercase()) {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

/// "no branch", "one branch", "two branches".
fn how_many(count: i64, noun: &str) -> String {
    const WORDS: [&str; 10] = [
        "no", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine",
    ];
    let number = usize::try_from(count)
        .ok()
        .and_then(|n| WORDS.get(n))
        .map_or_else(|| count.to_string(), |word| (*word).to_string());
    if count <= 1 {
        return format!("{number} {noun}");
    }
    let plural = if ["s", "sh", "ch", "x", "z"]
        .iter()
        .any(|end| noun.ends_with(end))
    {
        format!("{noun}es")
    } else {
        format!("{noun}s")
    };
    format!("{number} {plural}")
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

/// A put-down thing that broke on the floor: gone, and in no place. It
/// stands in for [`dropped`] on the turn the thing broke.
pub fn dropped_and_broke(item: &Row, here: &Row, dropper: Option<&Row>) -> String {
    format!(
        "ON THIS TURN, and not before it, {} put the {} down, and it BROKE on the floor of {}. \
         Until this turn it WAS in their hands. It is not lying anywhere now: it is broken and \
         gone, and nobody can pick it up, carry it or use it again. The putting down and the \
         breaking are what has just happened and they are what to narrate. Do not write it \
         landing whole, or anybody picking it up or finding it. {} is the only thing that \
         moved: nothing else was lifted, opened, drawn out or taken into anybody's hands.",
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

/// A look at a thing with nothing written on it: what the game holds of it,
/// where it is now, and that the look moved nothing. `carried` is whether
/// the player holds it. Only a thing with no description is said to have
/// nothing written on it: a description may mention writing the thing
/// carries as part of its look, and the fact does not contradict it.
pub fn examined(item: &Row, looker: Option<&Row>, carried: bool) -> String {
    let now = if carried {
        "they are carrying it"
    } else {
        "it is lying here"
    };
    let held = match presence(text(item, "description")) {
        Some(description) => format!(
            "The game's description of it, written when it was put in the world: \"{description}\" \
             Describe how it looks from that, in your own words, and do not contradict it. Where \
             that description says where it is or who has it, it may be out of date: {now}."
        ),
        None => format!(
            "The game records nothing of it beyond its name. Now {now}. Nothing is written on it."
        ),
    };
    format!(
        "ON THIS TURN {} looked closely at {}. {held} The look moved nothing: nothing was \
         lifted, opened, drawn out or taken into anybody's hands.",
        looker.map_or("the player", |who| string(who, "fullname")),
        definite_name(item),
    )
}

/// The engine's own words for a look at a thing with nothing written on it.
pub fn examined_words(item: &Row) -> String {
    match presence(text(item, "description")) {
        Some(description) => format!("You look closely at {}. {description}", definite_name(item)),
        None => format!("You look closely at {}.", definite_name(item)),
    }
}

/// How a throw came out, as `Playthrough::Turn#thrown_fact` states it.
#[derive(Clone, Copy, Debug)]
pub enum Throw<'a> {
    /// It hit the person named, and broke if `broke`.
    Struck { target: &'a str, broke: bool },
    /// It went through the way out into the room named, and broke where it
    /// landed if `broke`.
    Thrown { into: &'a str, broke: bool },
    /// It carried only `range` paces of the `distance` to the person named
    /// and hit nobody, and broke where it landed if `broke`.
    ShortOf {
        target: &'a str,
        range: i64,
        distance: i64,
        broke: bool,
    },
    /// It carried only `range` paces of the `distance` to the way out into
    /// `into`, and came down in `here`, the room it was thrown in, and broke
    /// there if `broke`.
    ShortOfTheWayOut {
        into: &'a str,
        here: &'a str,
        range: i64,
        distance: i64,
        broke: bool,
    },
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
        Throw::Struck {
            target,
            broke: false,
        } => format!(
            "{who} threw the {thing} at {target} and it hit them. The {thing} is NO LONGER CARRIED: it is lying \
             on the floor at {target}'s feet, where it stays until somebody picks it up."
        ),
        Throw::Struck {
            target,
            broke: true,
        } => format!(
            "{who} threw the {thing} at {target} and it hit them, and it BROKE. The {thing} is NO LONGER \
             CARRIED and is not lying anywhere: it is broken and gone, and nobody can pick it up, carry \
             it or use it again."
        ),
        Throw::Thrown { into, broke: false } => format!(
            "{who} threw the {thing} through the way out into {into}. The {thing} is NO LONGER CARRIED and is \
             no longer in this room at all: it is lying in {into}, where it stays until somebody picks it up."
        ),
        Throw::Thrown { into, broke: true } => format!(
            "{who} threw the {thing} through the way out into {into}, and it BROKE where it landed. The \
             {thing} is NO LONGER CARRIED and is not lying anywhere, in this room or in {into}: it is \
             broken and gone, and nobody can pick it up, carry it or use it again."
        ),
        Throw::ShortOf {
            target,
            range,
            distance,
            broke,
        } => format!(
            "{who} threw the {thing} at {target} and it FELL SHORT: the {thing} is {bulk} and carries \
             only {}, and {target} was {} away. {}",
            paces(range),
            paces(distance),
            if broke {
                format!(
                    "It hit nobody, and it BROKE where it landed. The {thing} is NO LONGER CARRIED and is \
                     not lying anywhere: it is broken and gone, and nobody can pick it up, carry it or \
                     use it again."
                )
            } else {
                format!(
                    "It hit nobody. The {thing} is NO LONGER CARRIED: it is lying on the floor between \
                     them, where it stays until somebody picks it up."
                )
            }
        ),
        Throw::ShortOfTheWayOut {
            into,
            here,
            range,
            distance,
            broke,
        } => format!(
            "{who} threw the {thing} at the way out into {into} and it FELL SHORT: the {thing} is {bulk} \
             and carries only {}, and the way out was {} away. {}",
            paces(range),
            paces(distance),
            if broke {
                format!(
                    "It did not go through, and it BROKE where it landed. The {thing} is NO LONGER \
                     CARRIED and is not lying anywhere, in {here} or in {into}: it is broken and gone, \
                     and nobody can pick it up, carry it or use it again."
                )
            } else {
                format!(
                    "It did not go through, and it is not in {into}. The {thing} is NO LONGER CARRIED: \
                     it is lying on the floor of {here}, short of the way out, where it stays until \
                     somebody picks it up."
                )
            }
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

/// How far a thrown thing carries, told where the room gives nothing to
/// hold it against; it follows the sentence [`thrown`] gives a throw that
/// hit or went through.
pub fn carries(thing: &str, bulk: &str, range: i64) -> String {
    format!(
        "The {thing} is {bulk}; a throw of it carries about {}.",
        paces(range)
    )
}

fn paces(count: i64) -> String {
    if count == 1 {
        "1 pace".to_string()
    } else {
        format!("{count} paces")
    }
}
