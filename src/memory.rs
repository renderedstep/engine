//! `Playthrough::Memory`: which of one character's earlier exchanges with the
//! player come back into a prompt, ranked by the words of the present subject,
//! and the sentences they come back as.

use crate::playthrough::Game;
use crate::records::{id, int, string, text, Row};
use crate::text::{is_blank, presence, ruby_downcase, ruby_strip, ruby_sum, truncate};
use std::cmp::Ordering;
use std::collections::HashMap;

/// Words too common to rank an exchange by.
pub const STOP_WORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "been", "but", "by", "did", "do", "does", "for",
    "from", "had", "has", "have", "he", "her", "him", "his", "how", "i", "if", "in", "is", "it",
    "its", "me", "my", "of", "on", "or", "our", "she", "so", "that", "the", "their", "them",
    "they", "this", "to", "was", "we", "were", "what", "when", "where", "which", "who", "why",
    "will", "with", "would", "you", "your",
];

/// How many distinct older recollections are considered for a prompt
/// (`Playthrough::Moment::CONCLUSIONS`).
pub const CONCLUSIONS: usize = 6;

/// `#resolution`: what they concluded, or what they said.
pub fn resolution(row: &Row) -> String {
    let resolved = presence(text(row, "inner_resolution")).or(text(row, "action"));
    ruby_strip(resolved.unwrap_or("")).to_string()
}

/// `#recollection`: the exchange, attributed, in one paragraph.
pub fn recollection(game: &Game, row: &Row) -> String {
    let mut pieces = Vec::new();
    let typed = string(row, "user_input");
    if !is_blank(typed) {
        let speaker = game
            .protagonist()
            .map(|who| string(who, "fullname").to_string())
            .unwrap_or_else(|| "the speaker".into());
        pieces.push(format!(
            "You heard {speaker} say: \"{}\"",
            truncate(typed, 100)
        ));
    }
    let action = string(row, "action");
    let resolved = resolution(row);
    if !is_blank(action) && ruby_strip(action) != resolved {
        pieces.push(format!(
            "You remember responding: \"{}\"",
            truncate(action, 200)
        ));
    }
    pieces.push(format!("You then concluded: \"{resolved}\""));
    if text(row, "action_status") == Some("applied") && !is_blank(string(row, "action_fact")) {
        pieces.push(format!(
            "The recorded result was: {}",
            string(row, "action_fact")
        ));
    }
    pieces.join(" ")
}

fn words(text: &str) -> Vec<String> {
    let lowered = ruby_downcase(text);
    lowered
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.chars().count() >= 3 && !STOP_WORDS.contains(term))
        .map(str::to_string)
        .collect()
}

fn unique(terms: Vec<String>) -> Vec<String> {
    let mut seen = Vec::new();
    for term in terms {
        if !seen.contains(&term) {
            seen.push(term);
        }
    }
    seen
}

fn rarity(frequency: usize, total: usize) -> f64 {
    (1.0 + total as f64 / frequency as f64).ln()
}

/// The character's exchanges on this game's scene chain, oldest first.
fn exchanges<'a>(game: &Game<'a>, character: &Row) -> Vec<&'a Row> {
    let is_player = game
        .protagonist()
        .is_some_and(|who| id(who) == id(character));
    if int(character, "story_id") != Some(game.story_id()) || is_player {
        return Vec::new();
    }
    let chain: Vec<i64> = game.scene_chain().iter().map(|scene| id(scene)).collect();
    let who = Some(id(character));
    game.records.select("interactions", |row| {
        int(row, "character_id") == who
            && int(row, "scene_id").is_some_and(|scene| chain.contains(&scene))
    })
}

/// `#recall(query:, replayed:, limit:)`: the rows, best first.
pub fn recall<'a>(
    game: &Game<'a>,
    character: &Row,
    query: Option<&str>,
    replayed: i64,
    limit: usize,
) -> Vec<&'a Row> {
    let mut rows = exchanges(game, character);
    let count = replayed.max(0) as usize;
    if count > 0 {
        rows.truncate(rows.len().saturating_sub(count));
    }
    let key = |row: &'a Row| {
        (
            ruby_strip(&ruby_downcase(string(row, "user_input"))).to_string(),
            ruby_strip(&ruby_downcase(string(row, "action"))).to_string(),
            ruby_downcase(&resolution(row)),
            (text(row, "action_status") == Some("applied")).then(|| text(row, "action_fact")),
        )
    };
    let mut seen = Vec::new();
    let mut kept: Vec<&'a Row> = Vec::new();
    for row in rows.iter().rev() {
        let k = key(row);
        if !seen.contains(&k) {
            seen.push(k);
            kept.push(row);
        }
    }
    kept.reverse();
    let rows = kept;
    if rows.is_empty() {
        return Vec::new();
    }
    let terms: HashMap<i64, Vec<String>> = rows
        .iter()
        .map(|row| {
            let joined = [
                string(row, "user_input").to_string(),
                string(row, "action").to_string(),
                resolution(row),
                string(row, "action_fact").to_string(),
            ]
            .join(" ");
            (id(row), unique(words(&joined)))
        })
        .collect();
    let mut frequencies: HashMap<&str, usize> = HashMap::new();
    for row in &rows {
        for term in &terms[&id(row)] {
            *frequencies.entry(term.as_str()).or_default() += 1;
        }
    }
    let held: Vec<String> = game
        .items_held_by(character)
        .iter()
        .map(|item| string(item, "name").to_string())
        .collect();
    let subject = std::iter::once(query.unwrap_or("").to_string())
        .chain(held)
        .collect::<Vec<_>>()
        .join(" ");
    let subjects = unique(words(&subject));
    let total = rows.len();
    let score = |row: &Row| {
        let vocabulary = &terms[&id(row)];
        let relevant = ruby_sum(
            vocabulary
                .iter()
                .filter(|term| subjects.contains(term))
                .map(|term| rarity(frequencies[term.as_str()], total)),
        );
        let distinct = ruby_sum(
            vocabulary
                .iter()
                .map(|term| rarity(frequencies[term.as_str()], total)),
        ) / vocabulary.len().max(1) as f64;
        (-relevant, -distinct, -id(row))
    };
    let mut ranked: Vec<(f64, f64, i64, &'a Row)> = rows
        .iter()
        .map(|row| {
            let (a, b, c) = score(row);
            (a, b, c, *row)
        })
        .collect();
    ranked.sort_by(|x, y| {
        x.0.partial_cmp(&y.0)
            .unwrap_or(Ordering::Equal)
            .then(x.1.partial_cmp(&y.1).unwrap_or(Ordering::Equal))
            .then(x.2.cmp(&y.2))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|entry| entry.3)
        .collect()
}
