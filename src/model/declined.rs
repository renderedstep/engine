//! `BaseAgent::Refusal`: what a model said instead of narrating, told apart
//! from narration by its shape rather than by its words.
//!
//! The narrator writes in the second person and characters speak inside
//! quotation marks, so an unquoted "I" near the top is the model talking
//! about itself, and a bulleted or numbered list is a menu, which the
//! narrator's instructions forbid. Real-world crisis resources are a third
//! shape, read separately because the engine does something different with
//! them: it suppresses the answer and does not ask another model.
//!
//! Ruby's `\s` and `\d` are ASCII here as they are there; its `\b` is
//! Unicode-aware, as the regex crate's is.

use regex::Regex;
use std::sync::OnceLock;

/// How much of an answer counts as its opening, in characters.
pub const OPENING: usize = 300;

/// One thing wrong with an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flag {
    UnquotedFirstPerson,
    List,
    CrisisResource,
}

impl Flag {
    pub fn name(self) -> &'static str {
        match self {
            Flag::UnquotedFirstPerson => "unquoted_first_person",
            Flag::List => "list",
            Flag::CrisisResource => "crisis_resource",
        }
    }
}

struct Patterns {
    quoted: Regex,
    first_person: Regex,
    list: Regex,
    crisis: Regex,
}

fn patterns() -> &'static Patterns {
    static PATTERNS: OnceLock<Patterns> = OnceLock::new();
    PATTERNS.get_or_init(|| Patterns {
        quoted: Regex::new(r#"["“”][^"“”\n]{0,600}?["“”]"#).expect("QUOTED_DIALOGUE"),
        first_person: Regex::new(r"\bI\b|\bI['’](m|ll|ve|d)\b").expect("FIRST_PERSON"),
        list: Regex::new(r"(?:\A|\n)[\t\n\x0B\x0C\r ]*(?:[-*•][\t\n\x0B\x0C\r ]+|[0-9]+[.)][\t\n\x0B\x0C\r ]+)")
            .expect("LIST"),
        crisis: Regex::new(r"(?i)\b988\b|741741|Crisis Text Line|Suicide (&|and) Crisis|findahelpline|hotline|Lifeline")
            .expect("CRISIS_RESOURCES"),
    })
}

/// Everything wrong with this text (`BaseAgent::Refusal.flags`).
pub fn flags(text: &str) -> Vec<Flag> {
    let patterns = patterns();
    let mut found = Vec::new();
    let unquoted = patterns.quoted.replace_all(text, " ");
    let opening: String = unquoted.chars().take(OPENING).collect();
    if patterns.first_person.is_match(&opening) {
        found.push(Flag::UnquotedFirstPerson);
    }
    if patterns.list.is_match(text) {
        found.push(Flag::List);
    }
    if patterns.crisis.is_match(text) {
        found.push(Flag::CrisisResource);
    }
    found
}

/// The model declined, or answered with a menu: a failed call, and the next
/// model is asked.
pub fn refused(text: &str) -> bool {
    flags(text)
        .iter()
        .any(|flag| matches!(flag, Flag::UnquotedFirstPerson | Flag::List))
}

/// The model answered with real-world crisis resources: suppressed, and
/// never rotated past.
pub fn crisis_response(text: &str) -> bool {
    patterns().crisis.is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_models_own_voice_outside_the_dialogue() {
        assert!(refused("I'm not going to narrate that."));
        assert!(refused("I can't write that."));
        assert!(!refused("\"I won't stop until you tell me to,\" she says."));
        assert!(!refused("You wait. The rain keeps on."));
        assert!(!refused("éI stands for nothing here."));
    }

    #[test]
    fn a_list_anywhere_is_a_menu() {
        assert!(refused("You could:\n1. Leave\n2. Stay"));
        assert!(refused("- run\n- hide"));
        assert!(!refused("It costs 1.5 marks."));
    }

    #[test]
    fn a_crisis_line_is_its_own_shape() {
        assert!(crisis_response("\"Call 988,\" she says."));
        assert!(crisis_response("a HOTLINE number"));
        assert!(!refused("\"Call 988,\" she says."));
        assert_eq!(flags("\"Call 988,\" she says."), [Flag::CrisisResource]);
        assert!(!crisis_response("Room 9880 is empty."));
    }
}
