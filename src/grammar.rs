//! The fixed grammar: one typed line read without a model, against the
//! closed sets of the room the playthrough stands in.
//!
//! A closed verb table, a name matched against the records, and an
//! [`Intent`] at the end of it -- the same `Intent` a model's answer is
//! resolved into. It claims a line only behind a leading slash
//! ([`Grammar::reading_first`]); [`Grammar::parse`] answers every line, for a
//! game with no model to fall back on. It never guesses: an ambiguous name,
//! an unknown verb and a name that lands on nothing are refusals.

use crate::cascade::{self, Leave, HERE_WORDS};
use crate::data;
use crate::intent::Intent;
use crate::room::{Choice, Record, Room, REMOVE, WAIT, WEAR};
use crate::text::{
    inspect, is_blank, is_ruby_space, ruby_downcase, ruby_strip, split_words, squish_ascii,
};

/// The verbs the engine answers itself rather than handing to a model.
pub const ENGINE_VIEW: &[&str] = &[
    "vitals",
    "hp",
    "condition",
    "stats",
    "abilities",
    "check",
    "attack",
    "hit",
    "strike",
    "throw",
    "hurl",
    "toss",
    "harm",
    "hurt",
    "damage",
    "mend",
    "heal",
    "help",
];

/// The words that resolve one record, as this grammar's help calls them,
/// with the action each produces. It is what the slash menu offers.
pub const RESOLVING: &[(&str, &str)] = &[
    ("go", "move"),
    ("talk", "talk"),
    ("take", "take"),
    ("drop", "drop"),
    ("inspect", "examine"),
    ("attack", "attack"),
];

/// The three abilities a `check` rolls against.
pub const ABILITIES: &[&str] = &["strength", "dexterity", "will"];

/// What a resolved reading is refused with when the line still joins a
/// second act on.
pub const MORE_THAN_ONE_ACT: &str =
    "that line joins two things together, and reading it is the model's job";

/// What `help` prints, and what an unknown word is refused with.
pub const HELP: &[&str] = &[
    "go <exit>        move into one of the ways out (also: move, walk, enter)",
    "take <item>      pick up something lying here (also: get, grab, pick up)",
    "drop <item>      put down something you are carrying (also: put down, leave)",
    "wear <item>      put on something you carry, or something lying here, which",
    "                 is picked up as part of it (also: don, put on). One thing",
    "                 on each place on a body, and what is worn over the body",
    "                 comes off before what is under it",
    "remove <item>    take off something you are wearing; it stays in your hands",
    "                 (also: doff, take off). A worn thing comes off before it",
    "                 is dropped, thrown, offered or burned",
    "talk <person>    resolve somebody standing here (also: speak, ask). The",
    "                 talking itself is prose, so this mode names them and stops",
    "inspect <item>   what is written on something, out of the records (also:",
    "                 read, examine, x, look at). Only a thing marked readable has",
    "                 words; this mode prints them and never writes them",
    "look             the engine's whole view of where you are (also: where,",
    "                 inventory, exits, items, who, state, vitals)",
    "stats            the player's level, hit die and three abilities, out of",
    "                 the world's own records (also: abilities)",
    "attack <person>  swing at somebody standing here (also: hit, strike). One",
    "                 blow of your own hit die, it always connects, and every",
    "                 live foe in the room answers in the same turn. Anybody",
    "                 can be attacked, and being attacked makes them a foe for",
    "                 this playthrough",
    "throw <thing> at <name|exit>",
    "                 throw something in your hands, or lying here, at somebody",
    "                 standing here or through one of the ways out (also: hurl,",
    "                 toss). Picking it up is part of the throw, not a turn of",
    "                 its own. One d20 under strength less what it weighs, and no",
    "                 second roll: if it leaves your hands it goes where you",
    "                 aimed it. A hit deals one die by bulk -- light d4, handy",
    "                 d6, heavy d8 -- and lands the thing at their feet. A",
    "                 failed lift is a spent turn and the thing stays in your",
    "                 hands. Something immovable is refused and costs nothing",
    "check <ability> [penalty]",
    "                 throw one d20 against strength, dexterity or will and",
    "                 print what it came up: d20-under the score, with the",
    "                 penalty taken off the TARGET. It writes nothing, and at a",
    "                 target of zero or less it says so instead of rolling",
    "harm <n>         take n hit points off the player, through",
    "                 Playthrough::Turn#harm!. Zero is death and death ends the",
    "                 playthrough (also: hurt, damage)",
    "mend <n>         put n back, up to the maximum. It never raises the dead",
    "                 (also: heal)",
    "wait             let time pass where you stand, and the world takes its",
    "                 turn (also: rest, idle, think, pray)",
    "tear <thing>     tear paper or cloth apart, in your hands or lying here,",
    "                 into scraps or strips (also: tear up, rip, rip up). Cloth",
    "                 takes a strength check",
    "smash <thing>    smash glass or pottery, in your hands or lying here, into",
    "                 shards",
    "end              end the story, once every goal it asked for is met",
    "help             this list",
    "",
    "A leading / is optional and is stripped: `/take slate` and `take slate` are",
    "the same line. A name is matched against the records: exactly first, then as",
    "an unambiguous prefix, then as an unambiguous fragment -- and a fragment is",
    "read both ways round, so `drop the tide-slate` finds the Assize",
    "tide-slate. A leading the/a/an/to/into/through/onto/at is dropped before",
    "any of that. Case and extra spaces do not matter. This is the no-model",
    "fallback -- drop it to have the classifier read what you type and the",
    "world generate as you walk.",
];

/// The attempts the beat guard can keep off a room's list
/// ([`Grammar::guarded`]).
const GUARDED: &[&str] = &["tear", "smash", "burn"];

/// Words taken off the front of a typed name, and nowhere else.
const LEADING_WORDS: &[&str] = &["the", "a", "an", "to", "into", "through", "onto", "at"];

const SLASH: char = '/';

/// The fixed grammar's verb table, in file order: each word a player may
/// type and the verb it is read as.
pub fn verbs() -> &'static [(String, String)] {
    data::verbs()
}

fn verb_of(word: &str) -> Option<&'static str> {
    verbs()
        .iter()
        .find(|(typed, _)| typed == word)
        .map(|(_, verb)| verb.as_str())
}

/// How one line was read. A reading with no intent and no refusal is a
/// command that only reads.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    pub intent: Option<Intent>,
    pub refusal: Option<String>,
    /// The whole help text, where the reading carries it.
    pub help: bool,
    /// `end`: the player taking the ending the story has fallen due with.
    pub end: bool,
    pub understood: Option<String>,
    /// `harm` or `mend`, and a number of hit points.
    pub wound: Option<(String, u64)>,
    /// An ability and a penalty.
    pub attempt: Option<(String, u64)>,
    /// Which reader answered: `grammar` or `engine_view`.
    pub resolved_by: Option<String>,
}

impl Reading {
    fn refused(refusal: String) -> Reading {
        Reading {
            refusal: Some(refusal),
            ..Reading::default()
        }
    }

    /// A record was named and found, or an attempt that binds none was
    /// read, so a caller can act on it with no model.
    pub fn resolved(&self) -> bool {
        self.intent
            .as_ref()
            .is_some_and(|i| i.subject().is_some() || i.physical.is_some())
    }
}

/// A typed name against the records it could have meant.
#[derive(Clone, Debug, Default)]
struct Match {
    record: Option<Record>,
    candidates: Vec<Record>,
}

impl Match {
    fn found(&self) -> bool {
        self.record.is_some()
    }

    fn ambiguous(&self) -> bool {
        self.record.is_none() && !self.candidates.is_empty()
    }
}

/// The line without its slash.
pub fn unslashed(command: &str) -> String {
    let text = ruby_strip(command);
    match text.strip_prefix(SLASH) {
        Some(rest) => ruby_strip(rest).to_string(),
        None => text.to_string(),
    }
}

pub fn slashed(command: &str) -> bool {
    ruby_strip(command).starts_with(SLASH)
}

/// Whether a line is `/end`, read the way every slashed line is read first
/// where a model reads the rest: the slash, the word and nothing after it.
pub fn ends(command: &str) -> bool {
    slashed(command) && verb_of(&ruby_downcase(&normalize_line(&unslashed(command)))) == Some("end")
}

/// The word a player types after the slash for one action, or `None` where
/// there is no one word (each physical attempt has its own).
pub fn word_for(action: &str) -> Option<String> {
    if let Some((word, _)) = RESOLVING.iter().find(|(_, a)| *a == action) {
        return Some(word.to_string());
    }
    (verb_of(action) == Some(action)).then(|| action.to_string())
}

/// How a line was read, in one line, for a person.
pub fn describe(intent: &Intent) -> String {
    let mut reading = format!(
        "{} -> {}",
        intent.action,
        intent
            .subject()
            .map(|s| s.label())
            .or_else(|| intent.physical.as_ref().map(Choice::name))
            .unwrap_or_else(|| "nothing".into())
    );
    if let Some(at) = &intent.at {
        reading += &format!(" at {}", at.label());
    }
    if intent.named_more_than_one() {
        let also = intent
            .also_named
            .as_ref()
            .map(Record::label)
            .unwrap_or_default();
        reading += &format!(" (and {also})");
    }
    reading
}

fn normalize(text: &str) -> String {
    squish_ascii(ruby_strip(&ruby_downcase(text)))
}

fn normalize_line(text: &str) -> String {
    squish_ascii(ruby_strip(text))
}

/// The names a record answers to: a person has two.
fn names_of(record: &Record) -> Vec<String> {
    let names = match record {
        Record::Person(person) => vec![Some(person.fullname.clone()), person.nickname.clone()],
        other => vec![Some(other.label())],
    };
    names
        .into_iter()
        .flatten()
        .filter(|name| !is_blank(name))
        .collect()
}

fn names(records: &[Record]) -> String {
    if records.is_empty() {
        return "nothing".into();
    }
    records
        .iter()
        .map(Record::label)
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_alnum(c: char) -> bool {
    c.is_alphanumeric()
}

/// Whether `needle` occurs in `hay` with no letter or digit either side.
fn contains_word(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let mut start = 0;
    while let Some(found) = hay[start..].find(needle) {
        let at = start + found;
        let end = at + needle.len();
        let before = hay[..at].chars().next_back();
        let after = hay[end..].chars().next();
        if !before.is_some_and(is_alnum) && !after.is_some_and(is_alnum) {
            return true;
        }
        start = at + hay[at..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// Ruby's `/(?<![[:alnum:]])word(?![[:alnum:]])/i`: the byte range of the
/// first standalone occurrence of an ASCII word, any case.
fn find_word_ci(hay: &str, word: &str) -> Option<(usize, usize)> {
    let lower: Vec<(usize, char)> = hay.char_indices().collect();
    let target: Vec<char> = word.chars().collect();
    for i in 0..lower.len() {
        if i + target.len() > lower.len() {
            break;
        }
        let hit = (0..target.len()).all(|k| lower[i + k].1.eq_ignore_ascii_case(&target[k]));
        if !hit {
            continue;
        }
        let before = i.checked_sub(1).map(|j| lower[j].1);
        let after = lower.get(i + target.len()).map(|(_, c)| *c);
        if !before.is_some_and(is_alnum) && !after.is_some_and(is_alnum) {
            let start = lower[i].0;
            let end = lower.get(i + target.len()).map_or(hay.len(), |(at, _)| *at);
            return Some((start, end));
        }
    }
    None
}

/// Ruby's `/(?<![[:alnum:]])(?:and|then)(?![[:alnum:]])|,/i`.
fn joins(text: &str) -> bool {
    text.contains(',')
        || find_word_ci(text, "and").is_some()
        || find_word_ci(text, "then").is_some()
}

/// `text.split(/\s+word\s+/i, 2)`: the text either side of the first
/// standalone word with spaces round it.
fn split_around(text: &str, word: &str) -> (String, Option<String>) {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let target: Vec<char> = word.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !is_ruby_space(chars[i].1) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < chars.len() && is_ruby_space(chars[j].1) {
            j += 1;
        }
        let k = j + target.len();
        let word_here = k <= chars.len()
            && (0..target.len()).all(|n| chars[j + n].1.eq_ignore_ascii_case(&target[n]));
        if word_here && k < chars.len() && is_ruby_space(chars[k].1) {
            let mut m = k;
            while m < chars.len() && is_ruby_space(chars[m].1) {
                m += 1;
            }
            let before = text[..chars[i].0].to_string();
            let after = chars.get(m).map_or("", |(at, _)| &text[*at..]).to_string();
            return (before, Some(after));
        }
        i += 1;
    }
    (text.to_string(), None)
}

fn chars_after(text: &str, count: usize) -> &str {
    match text.char_indices().nth(count) {
        Some((at, _)) => &text[at..],
        None => "",
    }
}

fn uniq(records: Vec<Record>) -> Vec<Record> {
    let mut out: Vec<Record> = Vec::new();
    for record in records {
        if !out.contains(&record) {
            out.push(record);
        }
    }
    out
}

/// The grammar over one room.
pub struct Grammar<'a> {
    room: &'a Room,
}

impl<'a> Grammar<'a> {
    pub fn new(room: &'a Room) -> Grammar<'a> {
        Grammar { room }
    }

    /// Whether the grammar reads this line first: a leading slash and
    /// nothing else.
    pub fn claims(&self, command: &str) -> bool {
        slashed(command)
    }

    /// The grammar's answer to a line it claims, or `None` for a line that
    /// belongs to the model. A resolved line that still joins a second act
    /// on goes to the model too.
    pub fn reading_first(&self, command: &str) -> Option<Reading> {
        if !self.claims(command) {
            return None;
        }
        let reading = self.parse(command);
        if !reading.resolved() {
            return Some(reading);
        }
        let intent = reading
            .intent
            .as_ref()
            .expect("a resolved reading has an intent");
        if self.joins_two_acts(command, intent) {
            return Some(Reading {
                intent: None,
                understood: None,
                refusal: Some(MORE_THAN_ONE_ACT.into()),
                ..reading
            });
        }
        Some(reading)
    }

    /// The line that plays one physical attempt, or `None` where the line
    /// would play a different one.
    pub fn line_for(&self, choice: &Choice) -> Option<String> {
        let line = match choice.argument() {
            argument if argument.is_empty() => format!("{SLASH}{}", choice.kind),
            argument => format!("{SLASH}{} {argument}", choice.kind),
        };
        let physical = self
            .reading_first(&line)
            .and_then(|reading| reading.intent)
            .and_then(|intent| intent.physical);
        (physical.as_ref() == Some(choice)).then_some(line)
    }

    fn joins_two_acts(&self, command: &str, intent: &Intent) -> bool {
        let mut rest = normalize(&unslashed(command));
        let records = match &intent.physical {
            Some(choice) => choice.records(),
            None => [intent.subject(), intent.at.clone()]
                .into_iter()
                .flatten()
                .collect(),
        };
        for record in &records {
            for name in names_of(record) {
                rest = rest.replace(&normalize(&name), " ");
            }
        }
        joins(&rest)
    }

    /// Reads any line, with no model to defer to.
    pub fn parse(&self, command: &str) -> Reading {
        let text = normalize_line(&unslashed(command));
        if text.is_empty() {
            return Reading::default();
        }
        let verb = self.verb_for(&text);
        let argument = match verb {
            Some(verb) => ruby_strip(chars_after(&text, verb.chars().count())).to_string(),
            None => text.clone(),
        };
        let reading = match verb.and_then(verb_of) {
            Some("look") => Reading::default(),
            Some("help") => Reading {
                help: true,
                ..Reading::default()
            },
            Some("end") if argument.is_empty() => Reading {
                end: true,
                ..Reading::default()
            },
            Some("go") => self.read_move(&argument),
            Some("take") => self.read_take(&argument),
            Some("drop") => self.read_drop(&argument),
            Some("talk") => self.read_talk(&argument),
            Some("read") => self.read_reading(&argument),
            Some(kind @ ("harm" | "mend")) => read_wound(&argument, kind),
            Some("check") => read_check(&argument),
            Some("attack") => self.read_attack(&argument),
            Some("throw") => self.read_throw(&argument),
            Some(
                kind @ ("consume" | "offer" | "burn" | "unlock" | "pick" | "pry" | "force"
                | "break" | "tear" | "smash"),
            ) => self.read_physical(kind, &argument),
            Some(WAIT) => self.read_wait(&argument),
            Some(kind @ (WEAR | REMOVE)) => self.read_dressing(kind, &argument),
            // A piece's inside, unless the line names no piece and does name
            // a way out ("open the narrow door into the closet"), which reads
            // as the move it always did.
            Some(kind @ ("search" | "open")) => {
                let reading = self.read_physical(kind, &argument);
                let found = reading
                    .intent
                    .as_ref()
                    .is_some_and(|intent| intent.physical.is_some());
                if !found && self.resolve(&self.room.exits_here(), &text).found() {
                    self.read_move(&text)
                } else {
                    reading
                }
            }
            _ => {
                if self.resolve(&self.room.exits_here(), &text).found() {
                    self.read_move(&text)
                } else {
                    unknown(&text)
                }
            }
        };
        let resolved_by = path_for(verb, &reading);
        Reading {
            resolved_by: Some(resolved_by.into()),
            ..reading
        }
    }

    /// One of the engine-view commands, or `None` for anything else. With a
    /// model behind it, a `check`, `attack` or `throw` that reads as English
    /// about the fiction is left to the model.
    pub fn engine_view_reading(&self, command: &str, model: bool) -> Option<Reading> {
        let text = normalize_line(&unslashed(command));
        let verb = self.verb_for(&text)?;
        if !ENGINE_VIEW.contains(&verb) || self.in_the_fiction(verb, &text, model) {
            return None;
        }
        Some(self.parse(&text))
    }

    fn in_the_fiction(&self, verb: &str, text: &str, model: bool) -> bool {
        if !model {
            return false;
        }
        let argument = ruby_strip(chars_after(text, verb.chars().count()));
        match verb_of(verb) {
            Some("check") => resolve_ability(
                argument
                    .split(is_ruby_space)
                    .next()
                    .filter(|w| !w.is_empty()),
            )
            .is_none(),
            Some("attack") => !self.resolve(&self.room.characters_here(), argument).found(),
            Some("throw") => !self.read_throw(argument).resolved(),
            _ => false,
        }
    }

    /// The longest verb the line begins with, followed by a space or
    /// nothing.
    pub fn verb_for(&self, text: &str) -> Option<&'static str> {
        let downcased = ruby_downcase(text);
        let mut table: Vec<&'static str> = verbs().iter().map(|(word, _)| word.as_str()).collect();
        table.sort_by_key(|verb| std::cmp::Reverse(verb.chars().count()));
        table
            .into_iter()
            .find(|verb| downcased == *verb || downcased.starts_with(&format!("{verb} ")))
    }

    /// `wait` and its synonyms: the room's one `wait`, whatever follows the
    /// word ("wait for him to leave" waits all the same), unless what
    /// follows joins a second act on.
    fn read_wait(&self, argument: &str) -> Reading {
        if joins(argument) {
            return Reading::refused(MORE_THAN_ONE_ACT.into());
        }
        let found = self
            .room
            .physical_actions()
            .into_iter()
            .find(|choice| choice.kind == WAIT);
        let intent = Intent {
            physical: found,
            ..Intent::new("use")
        };
        Reading {
            understood: Some(describe(&intent)),
            intent: Some(intent),
            ..Reading::default()
        }
    }

    /// `wear` and `remove`: the attempt on the thing named, where the room
    /// offers it; otherwise, where the name lands on a thing in the wrong
    /// state for it, the line is declined in the engine's words
    /// ([`crate::refusal::dressing`]). Every reading is a `use`, so no line
    /// with either word goes on to a reader.
    fn read_dressing(&self, kind: &str, argument: &str) -> Reading {
        let reading = self.read_physical(kind, argument);
        let found = reading
            .intent
            .as_ref()
            .is_some_and(|intent| intent.physical.is_some());
        if found {
            return reading;
        }
        let wearing = kind == WEAR;
        let declined = |fact: String| {
            let intent = Intent {
                declined: Some(fact),
                ..Intent::new("use")
            };
            Reading {
                understood: Some(describe(&intent)),
                intent: Some(intent),
                ..Reading::default()
            }
        };
        let offered = self
            .room
            .physical_actions()
            .iter()
            .any(|choice| choice.kind == kind);
        if is_blank(argument) {
            return match (offered, wearing) {
                (true, _) => reading,
                (false, true) => declined(crate::refusal::NOTHING_TO_WEAR.into()),
                (false, false) => declined(crate::refusal::WEARING_NOTHING.into()),
            };
        }
        let things = if wearing {
            [self.room.items_carried(), self.room.items_here()].concat()
        } else {
            self.room.items_carried()
        };
        let named = self.resolve(&things, argument);
        let thing = named.record.as_ref().and_then(Record::thing);
        let why = thing.and_then(|thing| {
            if wearing {
                self.room.wear_refused(thing)
            } else {
                self.room.remove_refused(thing)
            }
        });
        match (thing, why) {
            (Some(thing), Some(why)) => declined(crate::refusal::dressing(thing, why, wearing)),
            (None, _) if !wearing && !named.ambiguous() => {
                declined(crate::refusal::NOT_WEARING_THAT.into())
            }
            _ => reading,
        }
    }

    /// The attempt the beat guard keeps off the room's list that a line
    /// asks for ([`Room::guarded_actions`]): a `tear`, `smash` or `burn` of
    /// a thing the story still needs, which is refused in the engine's
    /// words. `None` for any other line.
    pub fn guarded(&self, command: &str) -> Option<Choice> {
        let text = normalize_line(&unslashed(command));
        let verb = self.verb_for(&text)?;
        let kind = verb_of(verb).filter(|kind| GUARDED.contains(kind))?;
        let argument = ruby_strip(chars_after(&text, verb.chars().count()));
        self.read_among(kind, argument, self.room.guarded_actions())
            .intent
            .and_then(|intent| intent.physical)
    }

    fn read_physical(&self, kind: &str, argument: &str) -> Reading {
        self.read_among(kind, argument, self.room.physical_actions())
    }

    /// A physical attempt of `kind` read among `offered`.
    fn read_among(&self, kind: &str, argument: &str, offered: Vec<Choice>) -> Reading {
        let offer = kind == "offer";
        let mut choices: Vec<Choice> = offered
            .into_iter()
            .filter(|choice| choice.kind == kind)
            .collect();
        let parted = kind == "break";
        let (target, second) = if argument.is_empty() {
            (None, None)
        } else {
            let word = match kind {
                "offer" => "to",
                "break" => "off",
                _ => "with",
            };
            let (target, second) = split_around(argument, word);
            (Some(target), second)
        };
        // Two pieces may each have a part of one name (two trees, each with
        // branches), so the piece after `off` is read first.
        if let (true, Some(second)) = (parted, &second) {
            let pieces = uniq(choices.iter().filter_map(Choice::piece).collect());
            match self.resolve(&pieces, second).record {
                Some(piece) => choices.retain(|choice| choice.piece().as_ref() == Some(&piece)),
                None => choices.clear(),
            }
        }
        let subjects = uniq(choices.iter().filter_map(Choice::subject).collect());
        let primary = self
            .resolve(&subjects, target.as_deref().unwrap_or(""))
            .record;
        match &primary {
            Some(primary) => choices.retain(|choice| choice.subject().as_ref() == Some(primary)),
            None => choices.clear(),
        }
        let partner = |choice: &Choice| -> Option<Record> {
            if offer {
                choice.recipient.clone().map(Record::Person)
            } else {
                choice.tool.clone().map(Record::Thing)
            }
        };
        match second {
            Some(second) if !parted => {
                let partners = uniq(choices.iter().filter_map(partner).collect());
                let found = self.resolve(&partners, &second).record;
                match &found {
                    Some(found) => {
                        choices.retain(|choice| partner(choice).as_ref() == Some(found));
                    }
                    None => choices.clear(),
                }
            }
            None if offer => choices.clear(),
            _ => {}
        }
        let found = if choices.len() == 1 {
            choices.pop()
        } else {
            None
        };
        let intent = Intent {
            physical: found,
            ..Intent::new("use")
        };
        Reading {
            understood: Some(describe(&intent)),
            intent: Some(intent),
            ..Reading::default()
        }
    }

    fn read_move(&self, argument: &str) -> Reading {
        let exits = self.room.exits_here();
        if is_blank(argument) {
            return Reading::refused(format!("go where? The ways out are: {}", names(&exits)));
        }
        let found = self.resolve(&exits, argument);
        if !found.found() {
            return Reading::refused(cannot_find("way out", argument, &found, &exits));
        }
        intent_reading(Intent {
            destination: found.record,
            ..Intent::new("move")
        })
    }

    fn read_take(&self, argument: &str) -> Reading {
        let here = self.room.items_here();
        if is_blank(argument) {
            return Reading::refused(format!("take what? Lying here: {}", names(&here)));
        }
        let found = self.resolve(&here, argument);
        if !found.found() {
            if let Some(held) = self.resolve(&self.room.items_carried(), argument).record {
                return Reading::refused(crate::refusal::already_carrying(&held));
            }
            return Reading::refused(cannot_find("thing lying here", argument, &found, &here));
        }
        intent_reading(Intent {
            item: found.record,
            ..Intent::new("take")
        })
    }

    fn read_drop(&self, argument: &str) -> Reading {
        let carried = self.room.items_carried();
        if is_blank(argument) {
            return Reading::refused(format!("drop what? Carrying: {}", names(&carried)));
        }
        let found = self.resolve(&carried, argument);
        if !found.found() {
            return Reading::refused(cannot_find(
                "thing you are carrying",
                argument,
                &found,
                &carried,
            ));
        }
        intent_reading(Intent {
            item: found.record,
            ..Intent::new("drop")
        })
    }

    fn read_person(&self, argument: &str, verb: &str, action: &str) -> Reading {
        let cast = self.room.characters_here();
        if is_blank(argument) {
            return Reading::refused(if cast.is_empty() {
                format!("{verb} whom? There is nobody here.")
            } else {
                format!("{verb} whom? Here: {}", names(&cast))
            });
        }
        let found = self.resolve(&cast, argument);
        if !found.found() {
            return Reading::refused(cannot_find("person here", argument, &found, &cast));
        }
        intent_reading(Intent {
            speaker: found.record,
            ..Intent::new(action)
        })
    }

    fn read_talk(&self, argument: &str) -> Reading {
        self.read_person(argument, "talk to", "talk")
    }

    fn read_attack(&self, argument: &str) -> Reading {
        self.read_person(argument, "attack", "attack")
    }

    fn read_throw(&self, argument: &str) -> Reading {
        let throwable = [self.room.items_carried(), self.room.items_here()].concat();
        if is_blank(argument) {
            return Reading::refused(format!(
                "throw what at what? `throw <thing> at <somebody, or a way out>`, and there is: {}",
                names(&throwable)
            ));
        }
        let (thrown, aimed) = match find_word_ci(argument, "at") {
            Some((start, end)) => (
                ruby_strip(&argument[..start]).to_string(),
                ruby_strip(&argument[end..]).to_string(),
            ),
            None => (ruby_strip(argument).to_string(), String::new()),
        };
        if is_blank(&aimed) {
            return Reading::refused(format!(
                "throw {} at what? {}",
                inspect(&thrown),
                self.aim_offer()
            ));
        }
        let found = self.resolve(&throwable, &thrown);
        if !found.found() {
            return Reading::refused(cannot_find(
                "thing in your hands or lying here",
                &thrown,
                &found,
                &throwable,
            ));
        }
        let cast = self.room.characters_here();
        let mut aim = self.resolve(&cast, &aimed);
        if !aim.found() {
            aim = self.resolve(&[cast, self.room.exits_here()].concat(), &aimed);
        }
        let item = found.record.expect("found");
        if !aim.found() {
            return Reading::refused(self.cannot_aim(&aimed, &aim, &item));
        }
        intent_reading(Intent {
            item: Some(item),
            at: aim.record,
            ..Intent::new("throw")
        })
    }

    fn cannot_aim(&self, aimed: &str, aim: &Match, item: &Record) -> String {
        let stays = format!("Nothing was thrown: {}.", stays_put(item));
        if aim.ambiguous() {
            return format!(
                "{} matches more than one thing to throw it at: {}. {stays}",
                inspect(aimed),
                names(&aim.candidates)
            );
        }
        format!(
            "there is nothing called {} to throw it at. {stays} {}",
            inspect(aimed),
            self.aim_offer()
        )
    }

    fn aim_offer(&self) -> String {
        format!(
            "Here with you: {}. The ways out are: {}.",
            names(&self.room.characters_here()),
            names(&self.room.exits_here())
        )
    }

    fn read_reading(&self, argument: &str) -> Reading {
        let readable = [self.room.items_here(), self.room.items_carried()].concat();
        if is_blank(argument) {
            return Reading::refused(format!(
                "read what? Here and in your hands: {}",
                names(&readable)
            ));
        }
        let found = self.resolve(&readable, argument);
        if !found.found() {
            return Reading::refused(cannot_find(
                "thing here or in your hands",
                argument,
                &found,
                &readable,
            ));
        }
        intent_reading(Intent {
            item: found.record,
            ..Intent::new("examine")
        })
    }

    /// The thing the player carries that a line's name lands on, read past
    /// its verb the way [`Grammar::parse`] reads it: what a take that found
    /// nothing lying here was most likely reaching for.
    pub fn carried_named(&self, command: &str) -> Option<Record> {
        let text = normalize_line(&unslashed(command));
        let argument = match self.verb_for(&text) {
            Some(verb) => ruby_strip(chars_after(&text, verb.chars().count())).to_string(),
            None => text,
        };
        self.resolve(&self.room.items_carried(), &argument).record
    }

    /// What a line that opens by leaving ([`cascade::leaving`]) comes to in
    /// this room, or `None` where it names something other than the room
    /// the player stands in, which is the reader's to read. A line opening
    /// with `leave` that also names a thing the player carries could be
    /// either act, so it is [`Leave::Either`].
    pub fn leaving(&self, command: &str) -> Option<Leave> {
        let leaving = cascade::leaving(command)?;
        let here = Record::Place(self.room.here.clone()?);
        let rest = leaving.rest.as_str();
        let names_here = rest.is_empty()
            || HERE_WORDS.contains(&rest)
            || self.resolve(std::slice::from_ref(&here), rest).found();
        if !names_here {
            return None;
        }
        if leaving.leave {
            if let Some(thing) = self.resolve(&self.room.items_carried(), rest).record {
                return Some(Leave::Either(thing));
            }
        }
        let mut exits = self.room.exits_here();
        Some(match exits.len() {
            0 => Leave::Nowhere,
            1 => Leave::Through(exits.remove(0)),
            _ => Leave::Which,
        })
    }

    /// Exact first, then an unambiguous prefix, then an unambiguous fragment
    /// read both ways round -- as typed, then without its leading words.
    fn resolve(&self, records: &[Record], typed: &str) -> Match {
        let mut ambiguous = None;
        for wanted in readings_of(typed) {
            let found = match_once(records, &wanted);
            if found.found() {
                return found;
            }
            if ambiguous.is_none() && found.ambiguous() {
                ambiguous = Some(found);
            }
        }
        ambiguous.unwrap_or_default()
    }
}

fn intent_reading(intent: Intent) -> Reading {
    Reading {
        understood: Some(describe(&intent)),
        intent: Some(intent),
        ..Reading::default()
    }
}

/// `engine_view` for a reading the engine answered out of its own
/// instruments with no intent; `grammar` for everything else.
fn path_for(verb: Option<&str>, reading: &Reading) -> &'static str {
    match verb {
        Some(verb) if ENGINE_VIEW.contains(&verb) && reading.intent.is_none() => "engine_view",
        _ => "grammar",
    }
}

fn read_wound(argument: &str, kind: &str) -> Reading {
    let digits = ruby_strip(argument);
    let well_formed = digits.starts_with(|c: char| ('1'..='9').contains(&c))
        && digits.chars().all(|c| c.is_ascii_digit());
    if !well_formed {
        let typed = if is_blank(argument) {
            "nil".to_string()
        } else {
            inspect(argument)
        };
        return Reading::refused(format!(
            "{kind} how much? `{kind} <n>` takes a whole number of hit points greater than \
             zero, and {typed} is not one"
        ));
    }
    let amount: u64 = digits.parse().unwrap_or(u64::MAX);
    let plural = if amount == 1 { "" } else { "s" };
    Reading {
        wound: Some((kind.to_string(), amount)),
        understood: Some(format!("{kind} -> {amount} hit point{plural}")),
        ..Reading::default()
    }
}

fn read_check(argument: &str) -> Reading {
    let words: Vec<&str> = ruby_strip(argument)
        .split(is_ruby_space)
        .filter(|word| !word.is_empty())
        .collect();
    let abilities = ABILITIES.join(", ");
    let Some(first) = words.first() else {
        return Reading::refused(format!(
            "check what? `check <ability> [penalty]`, and an ability is: {abilities}"
        ));
    };
    let Some(ability) = resolve_ability(Some(first)) else {
        return Reading::refused(format!(
            "{} is not one of the three abilities. There is: {abilities}",
            inspect(first)
        ));
    };
    let penalty = words.get(1);
    if let Some(penalty) = penalty {
        if !penalty.chars().all(|c| c.is_ascii_digit()) {
            return Reading::refused(format!(
                "a penalty is a whole number of points taken off the target, and {} is not one",
                inspect(penalty)
            ));
        }
    }
    let points: u64 = penalty.map_or(0, |p| p.parse().unwrap_or(u64::MAX));
    let noted = if points > 0 {
        format!(" (penalty {points})")
    } else {
        String::new()
    };
    Reading {
        attempt: Some((ability.to_string(), points)),
        understood: Some(format!("check -> {ability}{noted}")),
        ..Reading::default()
    }
}

/// One of the three abilities, exactly or as an unambiguous prefix.
fn resolve_ability(typed: Option<&str>) -> Option<&'static str> {
    let typed = ruby_downcase(typed.unwrap_or(""));
    if let Some(exact) = ABILITIES.iter().find(|a| **a == typed) {
        return Some(exact);
    }
    let matches: Vec<&&str> = ABILITIES.iter().filter(|a| a.starts_with(&typed)).collect();
    (matches.len() == 1).then(|| *matches[0])
}

fn unknown(text: &str) -> Reading {
    let first = split_words(text).first().copied().unwrap_or("");
    Reading {
        refusal: Some(format!(
            "I do not understand {}. The no-model grammar is fixed:",
            inspect(first)
        )),
        help: true,
        ..Reading::default()
    }
}

fn readings_of(typed: &str) -> Vec<String> {
    let wanted = normalize(typed);
    let stripped = without_leading_words(&wanted);
    let mut out = vec![wanted];
    if !out.contains(&stripped) {
        out.push(stripped);
    }
    out.retain(|reading| !reading.is_empty());
    out
}

/// Never down to nothing: a player who typed only "the" named nothing.
fn without_leading_words(wanted: &str) -> String {
    let mut words = split_words(wanted);
    while words.len() > 1 && LEADING_WORDS.contains(&words[0]) {
        words.remove(0);
    }
    words.join(" ")
}

fn match_once(records: &[Record], wanted: &str) -> Match {
    let exact: Vec<Record> = records
        .iter()
        .filter(|record| {
            names_of(record)
                .iter()
                .any(|name| normalize(name) == wanted)
        })
        .cloned()
        .collect();
    if !exact.is_empty() {
        return Match {
            record: exact.first().cloned(),
            candidates: exact,
        };
    }
    let ways: [fn(&str, &str) -> bool; 3] = [
        |name, typed| name.starts_with(typed),
        |name, typed| name.contains(typed),
        |name, typed| contains_word(typed, name),
    ];
    for how in ways {
        let hits: Vec<Record> = records
            .iter()
            .filter(|record| {
                names_of(record)
                    .iter()
                    .any(|name| how(&normalize(name), wanted))
            })
            .cloned()
            .collect();
        match hits.len() {
            0 => continue,
            1 => {
                return Match {
                    record: hits.first().cloned(),
                    candidates: hits,
                }
            }
            _ => {
                return Match {
                    record: None,
                    candidates: hits,
                }
            }
        }
    }
    Match::default()
}

fn cannot_find(kind: &str, typed: &str, found: &Match, records: &[Record]) -> String {
    if found.ambiguous() {
        return format!(
            "{} matches more than one {kind}: {}",
            inspect(typed),
            names(&found.candidates)
        );
    }
    format!(
        "there is no {kind} called {}. There is: {}",
        inspect(typed),
        names(records)
    )
}

fn stays_put(item: &Record) -> String {
    if item.carried() {
        format!("{} stays in your hands", item.definite_name())
    } else {
        format!("{} stays where it is lying", item.definite_name())
    }
}
