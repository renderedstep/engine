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
//!
//! [`Held`] reads the same three shapes while the answer is still arriving,
//! so a streamed answer never shows the player what the finished one will be
//! suppressed for.

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
    list_starting: Regex,
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
        list_starting: Regex::new(r"(?:\A|\n)[\t\n\x0B\x0C\r ]*(?:[-*•]|[0-9]+[.)]?)?\z")
            .expect("LIST_STARTING"),
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

/// The longest spelling the crisis watchlist matches ("Suicide and Crisis"),
/// in characters, and one more for the word boundary after "988". A match
/// that begins further back than this from the end of what has arrived has
/// arrived whole. A longer spelling added to the watchlist raises it, and
/// `no_crisis_resource_reaches_the_screen_at_any_offset` is where it is
/// added to be checked.
pub const CRISIS_REACH: usize = 19;

/// A streamed answer, shown only as far as the finished answer's judgement
/// cannot turn on it.
///
/// Each of the three reads decides differently on a prefix, so each holds
/// back differently:
///
/// - THE OPENING. The model's own voice is read in the first [`OPENING`]
///   characters once the dialogue is taken out, and a quotation still open
///   at the end of what has arrived may yet close and be taken out. So
///   nothing at all is shown until [`OPENING`] characters of the stripped
///   text are settled, or the answer is complete; an answer that turns out
///   to be the model talking about itself shows nothing.
/// - A LIST. A line that could still become a bulleted or numbered item is
///   held from the line break before it until it either is one or is not.
/// - A CRISIS LINE. The last [`CRISIS_REACH`] characters are held, so a
///   crisis resource has arrived whole before its first character could be
///   shown.
///
/// A list or a crisis resource found in what has arrived stops the stream:
/// nothing more is shown, and the finished answer fails the same read. What
/// was shown before it is prose that came first, which the end of the turn
/// replaces; the list and the resource themselves never reach the screen.
#[derive(Debug, Default)]
pub struct Held {
    text: String,
    shown: usize,
    opened: bool,
    stopped: bool,
}

impl Held {
    pub fn new() -> Held {
        Held::default()
    }

    /// Takes the next piece of the answer and answers what can now be
    /// shown, which may be nothing.
    pub fn take(&mut self, part: &str) -> &str {
        self.text.push_str(part);
        if self.stopped {
            return "";
        }
        if !self.opened {
            match settled_opening(&self.text) {
                None => return "",
                Some(true) => {
                    self.stopped = true;
                    return "";
                }
                Some(false) => self.opened = true,
            }
        }
        let patterns = patterns();
        let crisis_arrived = patterns
            .crisis
            .find_iter(&self.text)
            .any(|found| found.end() < self.text.len());
        if crisis_arrived || patterns.list.is_match(&self.text) {
            self.stopped = true;
            return "";
        }
        let mut until = self.text.len();
        if let Some(line) = patterns.list_starting.find(&self.text) {
            until = until.min(line.start());
        }
        let reach = self
            .text
            .char_indices()
            .rev()
            .nth(CRISIS_REACH - 1)
            .map_or(0, |(at, _)| at);
        until = until.min(reach);
        self.show(until)
    }

    /// The answer is complete and was judged fit to keep: the rest of it.
    pub fn rest(&mut self) -> &str {
        self.show(self.text.len())
    }

    /// Whether the stream was stopped for something the finished answer
    /// will be suppressed for.
    pub fn stopped(&self) -> bool {
        self.stopped
    }

    fn show(&mut self, until: usize) -> &str {
        let from = self.shown;
        if until <= from {
            return "";
        }
        self.shown = until;
        &self.text[from..until]
    }
}

/// The opening's verdict once the rest of the answer cannot change it:
/// whether the model speaks in its own voice there, or `None` while that is
/// still open.
///
/// A quoted span is taken out from its mark to the next one, so a match
/// found in what has arrived is the match the finished answer has. What can
/// still change is only the last quotation mark, when nothing after it has
/// closed it, broken it with a line break or run past the span's bound: the
/// stripped text is settled up to that mark.
fn settled_opening(text: &str) -> Option<bool> {
    let patterns = patterns();
    let last = text
        .char_indices()
        .rev()
        .find(|(_, c)| matches!(c, '"' | '“' | '”'));
    let settled = match last {
        Some((at, mark)) => {
            let after = &text[at..];
            let closes = patterns
                .quoted
                .find_iter(text)
                .any(|span| span.end() == at + mark.len_utf8());
            let open =
                !closes && !after.contains('\n') && after.chars().count() <= QUOTED_REACH + 1;
            if open {
                &text[..at]
            } else {
                text
            }
        }
        None => text,
    };
    let stripped = patterns.quoted.replace_all(settled, " ");
    if stripped.chars().count() < OPENING {
        return None;
    }
    let opening: String = stripped.chars().take(OPENING).collect();
    Some(patterns.first_person.is_match(&opening))
}

/// How far a quoted span may run between its marks (`QUOTED_DIALOGUE`).
const QUOTED_REACH: usize = 600;

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

    /// What a stream shows of `text` arriving `size` characters at a time,
    /// before it is complete, and whether it was stopped.
    fn streamed(text: &str, size: usize) -> (String, bool) {
        let mut held = Held::new();
        let mut shown = String::new();
        let chars: Vec<char> = text.chars().collect();
        for piece in chars.chunks(size) {
            let piece: String = piece.iter().collect();
            shown.push_str(held.take(&piece));
        }
        (shown, held.stopped())
    }

    fn narration(words: usize) -> String {
        "You walk on through the rain. ".repeat(words)
    }

    #[test]
    fn the_models_own_voice_shows_nothing_while_it_arrives() {
        let text = format!("I can't continue this scene. {}", narration(20));
        for size in [1, 7, 64] {
            assert_eq!(streamed(&text, size), (String::new(), true));
        }
        assert!(refused(&text));
    }

    #[test]
    fn a_short_answer_waits_to_be_judged_whole() {
        assert_eq!(streamed("I can't write that.", 1), (String::new(), false));
        let mut held = Held::new();
        assert_eq!(held.take("You wait."), "");
        assert_eq!(held.rest(), "You wait.");
    }

    #[test]
    fn a_kept_answer_is_shown_whole_by_the_end() {
        let text = format!(
            "{}\n\n\"I know,\" she says. {}",
            narration(12),
            narration(12)
        );
        let mut held = Held::new();
        let mut shown = String::new();
        for piece in text.split_inclusive(' ') {
            shown.push_str(held.take(piece));
        }
        assert!(!shown.is_empty(), "a long answer streams before it ends");
        assert!(!held.stopped());
        shown.push_str(held.rest());
        assert_eq!(shown, text);
    }

    #[test]
    fn an_open_quotation_holds_the_opening_until_it_closes() {
        let quoted = format!("\"I{}", " will not.".repeat(40));
        let text = format!("{quoted}\" {}", narration(15));
        let (shown, stopped) = streamed(&text, 1);
        assert!(!stopped, "the I was a character's, inside the quotation");
        assert!(shown.starts_with("\"I will not."));
        assert!(!refused(&text));
    }

    #[test]
    fn a_list_marker_never_reaches_the_screen() {
        for marker in ["- ", "* ", "• ", "1. ", "12) ", "  \n- "] {
            let text = format!("{}\n{marker}Leave\n2. Stay", narration(12));
            for size in [1, 3, 20] {
                let (shown, stopped) = streamed(&text, size);
                assert!(stopped, "{marker:?}");
                assert!(!shown.contains("Leave"), "{marker:?}");
                assert!(!patterns().list.is_match(&shown), "{marker:?} at {size}");
            }
            assert!(refused(&text));
        }
    }

    #[test]
    fn a_line_that_only_looks_like_a_list_is_let_through() {
        let text = format!(
            "{}\n2026 came and went.\n-the end. {}",
            narration(12),
            narration(2)
        );
        let (shown, stopped) = streamed(&text, 1);
        assert!(!stopped);
        assert!(shown.contains("2026 came and went."));
    }

    #[test]
    fn no_crisis_resource_reaches_the_screen_at_any_offset() {
        let spellings = [
            "988 ",
            "741741",
            "Crisis Text Line",
            "Suicide & Crisis",
            "SUICIDE AND CRISIS",
            "findahelpline",
            "hotline",
            "Lifeline",
        ];
        for spelling in spellings {
            assert!(spelling.trim().chars().count() < CRISIS_REACH, "{spelling}");
            for pad in 0..CRISIS_REACH + 2 {
                let text = format!(
                    "{}{} \"Call {spelling}now,\" she says. {}",
                    narration(11),
                    "x".repeat(pad),
                    narration(3)
                );
                let resource = text.find(&format!("Call {spelling}")).unwrap() + "Call ".len();
                for size in [1, 5, 32] {
                    let (shown, stopped) = streamed(&text, size);
                    assert!(stopped, "{spelling} after {pad} at {size}");
                    assert!(
                        shown.len() <= resource,
                        "{spelling} after {pad} at {size}: {shown:?}"
                    );
                }
                assert!(crisis_response(&text));
            }
        }
    }

    #[test]
    fn a_number_that_only_begins_988_is_let_through() {
        let text = format!("{}Room 9880 is empty. {}", narration(11), narration(3));
        let (shown, stopped) = streamed(&text, 1);
        assert!(!stopped);
        assert!(shown.contains("Room 9880"));
    }
}
