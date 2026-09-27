//! What the play box can complete after a slash: the words the grammar
//! resolves, each physical attempt's own word, and after each the names it
//! can be followed by. It invents nothing -- every name is one the grammar
//! and a model would both be able to bind.

use crate::grammar::RESOLVING;
use crate::room::Room;

/// Each attempt's word, and a line about it for the menu row.
pub const PHYSICAL_HINTS: &[(&str, &str)] = &[
    ("consume", "eat or drink something you carry"),
    ("offer", "offer something; its recipient may refuse"),
    ("burn", "burn a combustible thing with a firestarter"),
    ("unlock", "open a locked passage with its key"),
    ("pick", "try a lock with lockpicks"),
    ("pry", "try a jammed passage with a lever"),
    ("force", "try to force a jammed passage"),
];

/// A line about each resolving word.
pub const HINTS: &[(&str, &str)] = &[
    ("go", "a way out of here"),
    ("talk", "somebody standing here"),
    ("take", "something lying here"),
    ("drop", "something you are carrying"),
    ("inspect", "something here or in your hands"),
    (
        "attack",
        "strike somebody standing here -- your first blow lands now, and they answer",
    ),
];

fn hint(table: &[(&str, &'static str)], word: &str) -> Option<&'static str> {
    table.iter().find(|(w, _)| *w == word).map(|(_, h)| *h)
}

/// One word the box offers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verb {
    pub word: String,
    pub hint: Option<String>,
}

/// The whole menu for one room.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlashMenu {
    pub verbs: Vec<Verb>,
    /// Each word with the names it completes to, in order.
    pub targets: Vec<(String, Vec<String>)>,
}

fn uniq(names: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in names {
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

impl SlashMenu {
    pub fn for_room(room: &Room) -> SlashMenu {
        let mut kinds: Vec<String> = Vec::new();
        let choices = room.physical_actions();
        for choice in &choices {
            if !kinds.contains(&choice.kind) {
                kinds.push(choice.kind.clone());
            }
        }
        let mut verbs: Vec<Verb> = RESOLVING
            .iter()
            .map(|(word, _)| Verb {
                word: word.to_string(),
                hint: hint(HINTS, word).map(str::to_string),
            })
            .collect();
        verbs.extend(kinds.iter().map(|kind| {
            Verb {
                word: kind.clone(),
                hint: Some(
                    hint(PHYSICAL_HINTS, kind)
                        .unwrap_or_else(|| panic!("no hint for {kind}"))
                        .to_string(),
                ),
            }
        }));
        let mut targets: Vec<(String, Vec<String>)> = RESOLVING
            .iter()
            .map(|(word, action)| {
                let names = room
                    .offered_for(action)
                    .into_iter()
                    .map(|record| record.label())
                    .filter(|name| !crate::text::is_blank(name));
                (word.to_string(), uniq(names))
            })
            .collect();
        targets.extend(kinds.iter().map(|kind| {
            let arguments = choices
                .iter()
                .filter(|choice| &choice.kind == kind)
                .map(|choice| choice.argument());
            (kind.clone(), uniq(arguments))
        }));
        SlashMenu { verbs, targets }
    }
}
