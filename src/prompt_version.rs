//! The per-turn scaffold of a narrated turn, rendered once against fixed
//! placeholders (`Playthrough::PromptVersion::Scaffold`), so a digest can
//! cover it.
//!
//! A narrated turn's user message is the narration frame
//! ([`crate::facts::framing`]) around whatever the turn wrote as its fact
//! ([`crate::facts`]). Those sentences are instructions to the narrator as
//! much as the system message is, but they arrive interleaved with the
//! facts, so no reader of a stored message can tell the two apart. They are
//! code, though, so they are rendered here, every branch of every one,
//! against placeholders that stand where the records would go, and the
//! rendered text is what the game repository's `Playthrough::PromptVersion`
//! digests. A change of wording moves the digest; moving the same words
//! into another function does not.
//!
//! The order is fixed, and so are the placeholders: changing either moves
//! the digest for no change of prompt.

use crate::facts::{self, Throw};
use crate::moment::Direction;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// Where the moment, from the records, would go.
pub const FACTS: &str = "<the moment, from the records>";
/// Where what the player typed would go.
pub const COMMAND: &str = "<what the player typed>";
/// Where what the game already did would go.
pub const FACT: &str = "<what the app already did>";
/// Where the words written on a thing would go.
pub const WORDS: &str = "<what is written on it>";

const ITEM: &str = "<item>";
const DESCRIPTION: &str = "<its description>";
const WHO: &str = "<who>";
const WHERE: &str = "<where>";

/// What separates two rendered texts: a NUL no prompt can contain, so two
/// sentences cannot run together into a third (`PromptVersion::JOINER`).
pub const JOINER: &str = "\n\0\n";

fn row(value: Value) -> crate::records::Row {
    match value {
        Value::Object(row) => row,
        _ => unreachable!("a row is an object"),
    }
}

/// The whole scaffold as one string: every framing, then every fact
/// sentence, then the mark a moved row carries in the standing lists.
pub fn scaffold() -> String {
    let plain = row(json!({ "name": ITEM, "bulk": "handy" }));
    let described = row(json!({
        "name": ITEM,
        "description": DESCRIPTION,
        "inscription": WORDS,
        "readable": true,
        "bulk": "heavy",
    }));
    let somebody = row(json!({ "fullname": WHO }));
    let somewhere = row(json!({ "name": WHERE }));
    let bare = facts::bare_name(&plain);

    let mut texts = vec![
        facts::framing(FACTS, COMMAND, None, None),
        facts::framing(FACTS, COMMAND, Some(FACT), None),
    ];
    texts.extend(
        crate::data::narrator_doing_intents()
            .into_iter()
            .map(|intent| facts::framing(FACTS, COMMAND, Some(FACT), Some(intent))),
    );
    texts.extend([
        facts::taken(&plain, Some(&somebody), None),
        facts::taken(&described, Some(&somebody), Some(&somewhere)),
        facts::dropped(&plain, &somewhere, None),
        facts::dropped(&described, &somewhere, Some(&somebody)),
        facts::dropped_and_broke(&plain, &somewhere, None),
        facts::dropped_and_broke(&described, &somewhere, Some(&somebody)),
        facts::read(&plain, WORDS),
        facts::written_words(WORDS),
    ]);
    // Every way a throw comes out, and a fumble both ways: the sentence
    // says where the thing is, which depends on whether it was in hand.
    let throws = [
        ("handy", Throw::Fumbled { carried: true }),
        ("heavy", Throw::Fumbled { carried: false }),
        (
            "handy",
            Throw::Struck {
                target: WHO,
                broke: false,
            },
        ),
        (
            "handy",
            Throw::Struck {
                target: WHO,
                broke: true,
            },
        ),
        (
            "handy",
            Throw::Thrown {
                into: WHERE,
                broke: false,
            },
        ),
        (
            "handy",
            Throw::Thrown {
                into: WHERE,
                broke: true,
            },
        ),
        (
            "heavy",
            Throw::ShortOf {
                target: WHO,
                range: 3,
                distance: 5,
                broke: false,
            },
        ),
        (
            "heavy",
            Throw::ShortOf {
                target: WHO,
                range: 3,
                distance: 5,
                broke: true,
            },
        ),
        (
            "heavy",
            Throw::ShortOfTheWayOut {
                into: WHERE,
                here: WHERE,
                range: 3,
                distance: 5,
                broke: false,
            },
        ),
        (
            "heavy",
            Throw::ShortOfTheWayOut {
                into: WHERE,
                here: WHERE,
                range: 3,
                distance: 5,
                broke: true,
            },
        ),
        ("heavy", Throw::Immovable),
    ];
    texts.extend(
        throws
            .iter()
            .map(|(bulk, outcome)| facts::thrown(Some(WHO), &bare, bulk, *outcome)),
    );
    texts.push(facts::thrown(None, &bare, throws[0].0, throws[0].1));
    texts.push(facts::carries(&bare, "light", 12));
    texts.extend(
        [Direction::Taken, Direction::Dropped]
            .into_iter()
            .map(|direction| direction.note().to_string()),
    );
    texts.join(JOINER)
}

/// `Playthrough::PromptVersion::NARRATION`: the prose pass whose scaffold
/// [`scaffold`] covers.
pub const NARRATION: &str = "narration";

/// `Playthrough::PromptVersion::LENGTH`.
pub const LENGTH: usize = 16;

/// `Playthrough::PromptVersion.of`: the first [`LENGTH`] hex characters of
/// the SHA-256 of `text` stripped at its edges, as Ruby's `strip` strips;
/// none for a text with nothing in it.
pub fn of(text: &str) -> Option<String> {
    let body = strip(text);
    if body.is_empty() {
        return None;
    }
    let hash = Sha256::digest(body.as_bytes());
    Some(
        hash.iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()[..LENGTH]
            .to_string(),
    )
}

/// `Playthrough::PromptVersion.for_chat`, for a call built and not sent:
/// the digest of its instructions, and for the narrator's pass, of the
/// instructions and [`scaffold`] together. None for a call with no
/// instructions.
pub fn digest(purpose: &str, system: Option<&str>) -> Option<String> {
    let instructions = system.unwrap_or_default();
    if purpose != NARRATION {
        return of(instructions);
    }
    of(instructions)?;
    of(&[strip(instructions), &scaffold()].join(JOINER))
}

/// Ruby's `String#strip`: whitespace and NULs off both edges.
fn strip(text: &str) -> &str {
    text.trim_matches(|c| matches!(c, ' ' | '\t' | '\n' | '\x0b' | '\x0c' | '\r' | '\0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_narrators_digest_covers_its_instructions_and_the_scaffold() {
        let instructions = crate::data::narrator_instructions();
        let narrator = digest(NARRATION, Some(instructions)).unwrap();
        assert_eq!(narrator.len(), LENGTH);
        let joined = format!("{}{JOINER}{}", instructions.trim(), scaffold());
        assert_eq!(Some(narrator.clone()), of(&joined));
        assert_ne!(Some(narrator), of(instructions));
        assert_eq!(digest("arrival", Some(" words \n")), of("words"));
        assert_eq!(digest(NARRATION, None), None);
        assert_eq!(digest("ending", Some("  ")), None);
    }

    #[test]
    fn every_placeholder_is_rendered_and_no_record_is() {
        let text = scaffold();
        for placeholder in [FACTS, COMMAND, FACT, WORDS, ITEM, DESCRIPTION, WHO, WHERE] {
            assert!(text.contains(placeholder), "{placeholder} is not rendered");
        }
        assert!(text.contains("The party tried to pick up and throw the <item>"));
        assert!(text.contains("is still lying exactly where it was"));
    }

    #[test]
    fn every_branch_of_every_fact_sentence_is_rendered() {
        let text = scaffold();
        for said in [
            "Narrate it as done. Do not contradict it and do not undo it.",
            "The player types: <what the player typed>",
            "picked the <item> up",
            "it was lying in this room",
            "it was lying in <where>",
            "Now they are carrying it -- <its description>",
            "put the <item> down",
            "The party put the <item> down",
            "<who> put the <item> down",
            "The <item> has writing on it.",
            "word for word: \"<what is written on it>\"",
            "NOTHING WAS THROWN",
            "is still in the party's hands",
            "is still lying exactly where it was",
            "and it hit them",
            "through the way out into <where>",
            "put the <item> down, and it BROKE on the floor of <where>",
            "and it hit them, and it BROKE",
            "and it BROKE where it landed",
            "and it FELL SHORT: the <item> is heavy and carries only 3 paces",
            "It hit nobody, and it BROKE where it landed",
            "at the way out into <where> and it FELL SHORT",
            "It did not go through, and it BROKE where it landed",
            "a throw of it carries about 12 paces",
            "could not throw the <item> at all: it is heavy",
            "picked up just now, on this turn",
            "put down just now, on this turn",
        ] {
            assert!(text.contains(said), "{said:?} is not rendered");
        }
        for intent in crate::data::narrator_doing_intents() {
            let doing = crate::data::narrator_doing(intent).unwrap();
            assert!(text.contains(doing), "the {intent} line is not rendered");
        }
    }

    #[test]
    fn it_renders_the_same_text_every_time() {
        assert_eq!(scaffold(), scaffold());
    }
}
