//! `Playthrough::Moment`: what the prompts are told about the moment the
//! player is standing in, built out of the records and nothing else.

use crate::ledger;
use crate::memory::{self, CONCLUSIONS};
use crate::plan::Plan;
use crate::playthrough::{got_clear, Game};
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::text::{is_blank, is_ruby_space, presence, ruby_strip, truncate};

/// How many characters of what a character already concluded a prompt may
/// carry.
pub const CONCLUSIONS_BUDGET: usize = 400;

/// And of their attributed recollections.
pub const MEMORIES_BUDGET: usize = 1_200;

/// The rows' sentences that fit the budget, in the order they were ranked,
/// then read back in id order.
fn under_memory_budget(
    rows: &[&Row],
    mut budget: usize,
    sentence: impl Fn(&Row) -> String,
) -> Vec<String> {
    let mut kept: Vec<(i64, String)> = Vec::new();
    for row in rows {
        let text = sentence(row);
        let length = text.chars().count();
        if is_blank(&text) || length > budget {
            continue;
        }
        budget -= length;
        kept.push((id(row), text));
    }
    kept.sort_by_key(|(id, _)| *id);
    kept.into_iter().map(|(_, text)| text).collect()
}

/// `#conclusions`: what this character decided, on the exchanges the chat no
/// longer replays.
pub fn conclusions(
    game: &Game,
    character: &Row,
    replayed: i64,
    query: Option<&str>,
) -> Vec<String> {
    let rows = memory::recall(game, character, query, replayed, CONCLUSIONS);
    under_memory_budget(&rows, CONCLUSIONS_BUDGET, memory::resolution)
}

/// `#recollections`: the same exchanges, attributed.
pub fn recollections(
    game: &Game,
    character: &Row,
    replayed: i64,
    query: Option<&str>,
) -> Vec<String> {
    let rows = memory::recall(game, character, query, replayed, CONCLUSIONS);
    under_memory_budget(&rows, MEMORIES_BUDGET, |row| {
        memory::recollection(game, row)
    })
}

/// How much of the player's history the narrator is given, in characters,
/// and over how many scenes (`Playthrough::RECAP_BUDGET`, `RECAP_SCENES`).
pub const RECAP_BUDGET: usize = 600;
pub const RECAP_SCENES: usize = 12;

/// Which way the one item a turn moved went (`Moment::Handled`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Taken,
    Dropped,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Direction::Taken => "taken",
            Direction::Dropped => "dropped",
        }
    }

    pub fn note(self) -> &'static str {
        match self {
            Direction::Taken => "picked up just now, on this turn",
            Direction::Dropped => "put down just now, on this turn",
        }
    }
}

/// The item a turn moved, marked where it now stands.
#[derive(Clone, Copy, Debug)]
pub struct Handled {
    pub item: i64,
    pub direction: Direction,
}

/// One moment of one game.
pub struct Moment<'a> {
    pub game: Game<'a>,
    pub handled: Option<Handled>,
    /// The `quest_outcomes` row this game has just reached, on the one pass
    /// that tells the story's ending.
    pub ending: Option<&'a Row>,
}

/// `Moment#one_toll`: one toll, and the facts about it a paragraph must not
/// contradict.
pub fn one_toll(game: &Game, toll: &Row) -> String {
    let place = game.where_it_was(toll);
    let who = string(
        game.character(int(toll, "character_id").unwrap()),
        "fullname",
    );
    if got_clear(toll) {
        return format!("{who} got clear of {place} and lost nothing.");
    }
    let damage = int(toll, "damage").unwrap();
    let fate = if int(toll, "hp_after") == Some(0) {
        "dead: that was what killed them"
    } else {
        "alive"
    };
    format!(
        "{place} cost {who} {damage} hit point{} -- {}. {who} is {fate}.",
        if damage == 1 { "" } else { "s" },
        game.toll_words(toll)
    )
}

/// `Scene.recap_line`: a scene's summary, or its first sentence.
pub fn recap_line(scene: &Row) -> Option<String> {
    if let Some(summary) = presence(text(scene, "summary")) {
        return Some(ruby_strip(summary).to_string());
    }
    let description = ruby_strip(string(scene, "description"));
    let first = first_sentence(description);
    let line = truncate(first, 200);
    (!is_blank(&line)).then_some(line)
}

/// The text up to the first ASCII whitespace run that follows ".", "!" or
/// "?" (`split(/(?<=[.!?])\s+/).first`).
fn first_sentence(text: &str) -> &str {
    let mut previous = None;
    for (at, c) in text.char_indices() {
        if is_ruby_space(c) && matches!(previous, Some('.' | '!' | '?')) {
            return &text[..at];
        }
        previous = Some(c);
    }
    text
}

impl<'a> Moment<'a> {
    pub fn new(game: Game<'a>) -> Moment<'a> {
        Moment {
            game,
            handled: None,
            ending: None,
        }
    }

    fn records(&self) -> &'a Records {
        self.game.records
    }

    fn location(&self) -> Option<&'a Row> {
        self.game.current_location()
    }

    fn protagonist(&self) -> Option<&'a Row> {
        self.game.protagonist()
    }

    /// `#narration_context(plan:, arc:)`.
    pub fn narration_context(&self, plan: bool, arc: bool) -> String {
        let story = self.game.story();
        let mut parts = vec![
            format!(
                "Story: {} ({})",
                string(story, "title"),
                string(story, "genre")
            ),
            format!("Premise: {}", string(story, "summary")),
        ];
        if let Some(location) = self.location() {
            parts.push(format!(
                "The player is in {}: {}",
                string(location, "name"),
                string(location, "description")
            ));
            let exits: Vec<&str> = self
                .game
                .exits()
                .iter()
                .map(|way| string(way, "name"))
                .collect();
            let exits = exits.join(", ");
            parts.push(format!(
                "Ways out of here: {}. There are no others.",
                if exits.is_empty() { "none" } else { &exits }
            ));
            if plan {
                if let Some(facts) = Plan::of(self.records(), location) {
                    parts.push(facts.to_prompt());
                }
            }
        }
        if let Some(protagonist) = self.protagonist() {
            parts.push(format!(
                "The player is {}.",
                string(protagonist, "fullname")
            ));
            if let Some(condition) = self.game.vitals_for(protagonist) {
                parts.push(format!(
                    "{} is {}.",
                    string(protagonist, "fullname"),
                    condition.in_words()
                ));
            }
        }
        if arc {
            if let Some(beat) = self.next_beat() {
                parts.push(format!("The story is asking for: {beat}"));
            }
            if let Some(ending) = self.ending {
                parts.push(format!(
                    "The story has ended: {}",
                    string(ending, "summary")
                ));
            }
        }
        let others = self.others();
        parts.push(if others.is_empty() {
            "Nobody else is here.".into()
        } else {
            format!("Also here: {}. Nobody else is present.", name_list(&others))
        });
        if self.ending.is_some() {
            parts.extend(self.dead_here());
        }
        parts.extend(self.conditions_of_others());
        parts.extend(self.struck_fact());
        parts.extend(self.toll_fact());
        parts.extend(self.volition_fact());
        // A fixed piece is not a thing lying here: it is told on a line of its
        // own, only when the room has one, so a room with none is asked for in
        // exactly the words it was before any room was furnished.
        let (fixed, loose): (Vec<&Row>, Vec<&Row>) = self
            .game
            .items_lying_in(self.location())
            .into_iter()
            .partition(|item| text(item, "tier") == Some(crate::kit::FIXTURE));
        if !fixed.is_empty() {
            parts.push(format!(
                "Fixed here, and not takeable: {}.",
                self.item_list(&fixed)
            ));
        }
        let floor = self.item_list(&loose);
        parts.push(format!(
            "Lying here, and takeable: {}.",
            if floor.is_empty() { "nothing" } else { &floor }
        ));
        let carried = self.item_list(&self.game.carried());
        parts.push(format!(
            "The player is carrying: {}.",
            if carried.is_empty() {
                "nothing"
            } else {
                &carried
            }
        ));
        if let Some(previous) = self.what_just_happened() {
            parts.push(format!(
                "What just happened: {}",
                string(previous, "description")
            ));
        }
        if let Some(recap) = self.recap() {
            parts.push(format!("Earlier, in order:\n{recap}"));
        }
        parts.join("\n\n")
    }

    /// `#character_context(character, replayed:, query:)`.
    pub fn character_context(&self, character: &Row, replayed: i64, query: Option<&str>) -> String {
        let mut lines = Vec::new();
        if let Some(location) = self.location() {
            lines.push(format!("Where you are: {}.", string(location, "name")));
        }
        lines.push(format!(
            "The time is about {}.",
            time_of_day(self.game.story_now())
        ));
        let company: Vec<&Row> = self
            .others()
            .into_iter()
            .filter(|other| id(other) != id(character))
            .collect();
        if !company.is_empty() {
            lines.push(format!(
                "Also here, besides the two of you: {}.",
                name_list(&company)
            ));
        }
        lines.extend(self.personal_facts(character));
        if self.witnessed_current_scene(character) {
            lines.extend(self.last_attempt());
            lines.extend(self.last_reading());
        }
        let remembered = recollections(&self.game, character, replayed, query);
        if let (false, Some(protagonist)) = (remembered.is_empty(), self.protagonist()) {
            let listed: Vec<String> = remembered
                .iter()
                .map(|sentence| format!("- {sentence}"))
                .collect();
            lines.push(format!(
                "Your recollections of earlier exchanges with {}. \
                 These are what you heard and believed then, not independent proof of the speaker's claims:\n{}",
                string(protagonist, "fullname"),
                listed.join("\n")
            ));
        }
        lines.join("\n")
    }

    /// `#personal_facts`: what this person's own body and eyes hold.
    pub fn personal_facts(&self, character: &Row) -> Vec<String> {
        let cast = self.game.cast_in(self.location());
        if !cast.iter().any(|who| id(who) == id(character)) {
            return Vec::new();
        }
        let mut lines = Vec::new();
        if let Some(own) = self.game.vitals_for(character) {
            lines.push(format!("Your own condition: {}.", own.in_words()));
        }
        let Some(protagonist) = self.protagonist() else {
            return lines;
        };
        let player = string(protagonist, "fullname");
        if self
            .game
            .foes_in(self.location())
            .iter()
            .any(|who| id(who) == id(character))
        {
            lines.push(format!("You are currently fighting {player}."));
        } else if self.game.ceasefire_with(id(character)) {
            lines.push(format!(
                "You have a ceasefire with {player}; it still holds."
            ));
        }
        lines.extend(ledger::recall(&self.game, character, self.location()));
        lines
    }

    fn witnessed_current_scene(&self, character: &Row) -> bool {
        let Some(scene) = self.game.current_scene() else {
            return false;
        };
        let (scene, who) = (Some(id(scene)), Some(id(character)));
        self.records()
            .first("characters_scenes", |row| {
                int(row, "scene_id") == scene && int(row, "character_id") == who
            })
            .is_some()
    }

    fn others(&self) -> Vec<&'a Row> {
        if self.location().is_none() {
            return Vec::new();
        }
        let player = self.protagonist().map(id);
        self.game
            .cast_in(self.location())
            .into_iter()
            .filter(|who| Some(id(who)) != player)
            .collect()
    }

    /// The dead lying in this room, on the ending's pass alone: the last
    /// paragraph is written once with nothing after it to correct it, and
    /// the room's own description and the ending's sentence may still speak
    /// of them standing. A nickname goes in brackets, because that is what
    /// the world's prose may call them. Never the player, who is "you" and
    /// has a condition line of their own. The arrival's sentence, otherwise.
    fn dead_here(&self) -> Option<String> {
        let location = self.location()?;
        let player = self.protagonist().map(id);
        let dead: Vec<String> = self
            .game
            .characters_located_in(location)
            .into_iter()
            .filter(|who| Some(id(who)) != player)
            .filter(|who| self.game.vitals_for(who).is_some_and(|c| c.dead()))
            .map(|who| {
                let name = string(who, "fullname");
                match presence(text(who, "nickname")).map(ruby_strip) {
                    Some(nickname) if !is_blank(nickname) && nickname != name => {
                        format!("{name} ({nickname})")
                    }
                    _ => name.to_string(),
                }
            })
            .collect();
        if dead.is_empty() {
            return None;
        }
        Some(format!(
            "Dead here: {}. They cannot speak or act.",
            dead.join(", ")
        ))
    }

    fn conditions_of_others(&self) -> Vec<String> {
        let fighting: Vec<i64> = self
            .game
            .foes_in(self.location())
            .iter()
            .map(|who| id(who))
            .collect();
        self.others()
            .iter()
            .filter_map(|person| {
                let state = self.game.vitals_for(person)?;
                let foe = fighting.contains(&id(person));
                if state.unhurt() && !foe {
                    return None;
                }
                Some(format!(
                    "{} is {}{}.",
                    string(person, "fullname"),
                    state.in_words(),
                    if foe { " and is fighting you" } else { "" }
                ))
            })
            .collect()
    }

    fn struck_fact(&self) -> Option<String> {
        let blows: Vec<&Row> = self
            .game
            .own("playthrough_blows")
            .into_iter()
            .filter(|blow| int(blow, "scene_id").is_none())
            .collect();
        if blows.is_empty() {
            return None;
        }
        let told: Vec<String> = blows
            .iter()
            .map(|blow| {
                let name = |column: &str| {
                    string(self.game.character(int(blow, column).unwrap()), "fullname")
                };
                let damage = int(blow, "damage").unwrap();
                let fate = if int(blow, "hp_after") == Some(0) {
                    "dead: that was the blow that killed them"
                } else {
                    "alive"
                };
                format!(
                    "{} struck {} for {damage} hit point{}. {} is {fate}.",
                    name("attacker_id"),
                    name("target_id"),
                    if damage == 1 { "" } else { "s" },
                    name("target_id")
                )
            })
            .collect();
        Some(format!(
            "Blows landed, recorded by the game: {} Those are the numbers and they do not change. \
             Do not decide who lives, who dies, or how much anything hurt.",
            told.join(" ")
        ))
    }

    fn volition_fact(&self) -> Option<String> {
        let acts: Vec<&str> = self
            .game
            .own("playthrough_volitions")
            .into_iter()
            .filter(|row| int(row, "scene_id").is_none() && text(row, "status") == Some("applied"))
            .map(|row| string(row, "fact"))
            .collect();
        if acts.is_empty() {
            return None;
        }
        Some(format!(
            "What else happened here, recorded by the game: {}",
            acts.join(" ")
        ))
    }

    fn toll_fact(&self) -> Option<String> {
        let tolls: Vec<&Row> = self
            .game
            .own("playthrough_tolls")
            .into_iter()
            .filter(|toll| int(toll, "scene_id").is_none())
            .collect();
        if tolls.is_empty() {
            return None;
        }
        let told: Vec<String> = tolls
            .iter()
            .map(|toll| one_toll(&self.game, toll))
            .collect();
        Some(format!(
            "The place itself, recorded by the game: {} Those are the numbers and they do not change. \
             Do not decide how much anything hurt, and do not write a wound the game did not record.",
            told.join(" ")
        ))
    }

    fn last_attempt(&self) -> Option<String> {
        let protagonist = self.protagonist()?;
        let typed = ruby_strip(
            self.game
                .current_scene()
                .map(|s| string(s, "typed"))
                .unwrap_or(""),
        );
        if is_blank(typed) {
            return None;
        }
        Some(format!(
            "What {} did a moment ago: \"{}\"",
            string(protagonist, "fullname"),
            truncate(typed, 200)
        ))
    }

    fn last_reading(&self) -> Option<String> {
        let scene = self.game.current_scene()?;
        if text(scene, "resolved_action") != Some("examine")
            || text(scene, "acted_on_type") != Some("Item")
        {
            return None;
        }
        let item = self.records().find("items", int(scene, "acted_on_id")?)?;
        if !flag(item, "readable") || is_blank(string(item, "inscription")) {
            return None;
        }
        let reader = self
            .protagonist()
            .map(|who| string(who, "fullname"))
            .unwrap_or("The person you are speaking to");
        Some(format!(
            "What {reader} was reading a moment ago: {}, which says, word for word: \"{}\"",
            definite_name(string(item, "name")),
            string(item, "inscription")
        ))
    }

    fn what_just_happened(&self) -> Option<&'a Row> {
        let scene = self.game.current_scene()?;
        if self.ending.is_none() {
            return Some(scene);
        }
        int(scene, "previous_scene_id").and_then(|id| self.records().find("scenes", id))
    }

    /// `Playthrough::Arc#next_step`'s summary: the main open arc's first
    /// step by position this game has not reached.
    pub fn next_beat(&self) -> Option<String> {
        let story = Some(self.game.story_id());
        let arc = self.records().first("quests", |quest| {
            int(quest, "story_id") == story
                && text(quest, "status") == Some("open")
                && int(quest, "parent_quest_id").is_none()
        })?;
        let reached: Vec<i64> = self
            .game
            .own("playthrough_beats")
            .iter()
            .filter_map(|beat| int(beat, "quest_step_id"))
            .collect();
        let mut steps = self.records().select("quest_steps", |step| {
            int(step, "quest_id") == Some(id(arc)) && !reached.contains(&id(step))
        });
        steps.sort_by_key(|step| int(step, "position"));
        steps
            .first()
            .map(|step| string(step, "summary").to_string())
    }

    /// `Playthrough#recap`: the turns before this one, newest kept first,
    /// inside the budget.
    fn recap(&self) -> Option<String> {
        let mut chain = self.game.scene_chain();
        if let Some(current) = self.game.current_scene() {
            if let Some(at) = chain.iter().position(|scene| id(scene) == id(current)) {
                chain.truncate(at);
            }
        }
        let start = chain.len().saturating_sub(RECAP_SCENES);
        let mut room = RECAP_BUDGET;
        let mut dropped = 0;
        let mut lines = Vec::new();
        for scene in chain[start..].iter().rev() {
            let Some(line) = recap_line(scene) else {
                continue;
            };
            let length = line.chars().count();
            if length > room {
                dropped += 1;
                continue;
            }
            room -= length;
            lines.push(line);
        }
        if lines.is_empty() {
            return None;
        }
        lines.reverse();
        let text = lines.join("\n");
        Some(if dropped > 0 {
            format!(
                "({dropped} earlier turn{} left out)\n{text}",
                if dropped == 1 { "" } else { "s" }
            )
        } else {
            text
        })
    }

    fn item_list(&self, items: &[&Row]) -> String {
        items
            .iter()
            .map(|item| match self.handled {
                Some(handled) if handled.item == id(item) => {
                    format!("{} ({})", string(item, "name"), handled.direction.note())
                }
                _ => string(item, "name").to_string(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn name_list(people: &[&Row]) -> String {
    people
        .iter()
        .map(|person| match presence(text(person, "nickname")) {
            Some(nickname) => format!("{} ({nickname})", string(person, "fullname")),
            None => string(person, "fullname").to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `Item#definite_name`: "the" and the name without its own article.
pub fn definite_name(name: &str) -> String {
    let lowered = name.to_ascii_lowercase();
    for article in ["a", "an", "the"] {
        if lowered.starts_with(article) {
            let rest = &name[article.len()..];
            let trimmed = rest.trim_start_matches(is_ruby_space);
            if trimmed.len() < rest.len() && !trimmed.is_empty() {
                return format!("the {trimmed}");
            }
        }
    }
    format!("the {name}")
}

/// `strftime("%-l %P")` in UTC: the hour on a twelve-hour clock and "am" or
/// "pm".
pub fn time_of_day(at: i64) -> String {
    let hour = at.rem_euclid(86_400) / 3_600;
    let twelve = match hour % 12 {
        0 => 12,
        h => h,
    };
    format!("{twelve} {}", if hour < 12 { "am" } else { "pm" })
}
