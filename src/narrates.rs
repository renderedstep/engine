//! A game the player narrates (`playthroughs.mode = player_narrates`): what
//! the player writes from, which scene waits for their paragraph, and the
//! paragraph kept beside the scene.
//!
//! THE PARAGRAPH IS PROSE AND NOTHING ELSE. It is kept in
//! `playthrough_paragraphs` with its author, never in the scene, and no rule
//! and no prompt reads it: the scene keeps the engine's own words, and a
//! later prompt is told the engine's fact for the turn
//! (`moment::Moment::narration_context`). The game checks the paragraph
//! against the turn's facts on its side and keeps what it found in `audit`.
//!
//! THE CARD IS THE ENGINE'S PLAYER-FACING WORDS, not the narrator's facts:
//! what the protagonist did, as the scene says it; what the arrival found;
//! what the people there did; what the world took; who was hit; and how the
//! story ended, where it did.

use crate::playthrough::Game;
use crate::records::{id, int, string, text, Records, Row};
use crate::turn::chooser;

/// Who wrote a paragraph (`playthrough_paragraphs.author`).
pub const PLAYER: &str = "player";

/// The actions a scene the arc closed the story on carries.
const CLOSING: [&str; 2] = ["conclude", "ending"];

/// What the player writes a turn's paragraph from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactsCard {
    /// The scene the paragraph is for: the one the turn was answered with.
    pub scene: i64,
    /// What the protagonist did, in the engine's words.
    pub act: String,
    /// On an arrival, what the records said on the way in, one fact a line.
    pub found: Vec<String>,
    /// What the people there did, each in the words its row keeps.
    pub people: Vec<String>,
    /// What a hazard or a doorway took from somebody.
    pub tolls: Vec<String>,
    /// Who was hit, and how they are.
    pub wounds: Vec<String>,
    /// The story's closing words, where the turn ended it.
    pub ending: Option<String>,
}

/// The scenes this game was answered with on the turns the game chose, in
/// the order they were played.
pub fn chosen_scenes(records: &Records, playthrough: i64) -> Vec<i64> {
    records
        .select("playthrough_commands", |row| {
            int(row, "playthrough_id") == Some(playthrough)
                && text(row, "command") == Some(chooser::LINE)
                && text(row, "status") == Some("completed")
        })
        .iter()
        .filter_map(|row| int(row, "result_scene_id"))
        .collect()
}

/// The player's paragraph for a scene, if they wrote one.
pub fn paragraph(records: &Records, scene: i64) -> Option<&Row> {
    records.first("playthrough_paragraphs", |row| {
        int(row, "scene_id") == Some(scene) && text(row, "author") == Some(PLAYER)
    })
}

/// The scene waiting for the player's paragraph: the latest turn the game
/// chose, until a paragraph is written for it. None in a game the player
/// does not narrate.
pub fn waiting(records: &Records, playthrough: i64) -> Option<i64> {
    if !Game::new(records, playthrough).player_narrates() {
        return None;
    }
    chosen_scenes(records, playthrough)
        .last()
        .copied()
        .filter(|scene| paragraph(records, *scene).is_none())
}

/// The card for a scene a turn the game chose was answered with; none for
/// any other scene.
pub fn card(records: &Records, playthrough: i64, scene: i64) -> Option<FactsCard> {
    if !chosen_scenes(records, playthrough).contains(&scene) {
        return None;
    }
    let game = Game::new(records, playthrough);
    let answered = records.find("scenes", scene)?;
    // A turn that ended the story is answered with the closing scene, which
    // the arc wrote after the act's own.
    let (acted, ending) = if CLOSING.contains(&text(answered, "resolved_action").unwrap_or("")) {
        let act =
            int(answered, "previous_scene_id").and_then(|before| records.find("scenes", before));
        (act, Some(string(answered, "description").to_string()))
    } else {
        (Some(answered), None)
    };
    let mut card = FactsCard {
        scene,
        ending,
        ..FactsCard::default()
    };
    let Some(acted) = acted else {
        return Some(card);
    };
    let told_here = |table: &str| -> Vec<&Row> {
        game.own(table)
            .into_iter()
            .filter(|row| int(row, "scene_id") == Some(id(acted)))
            .collect()
    };
    if text(acted, "resolved_action") == Some("move") {
        let room = int(acted, "location_id").and_then(|room| records.find("locations", room));
        card.act = format!(
            "You arrive at {}.",
            room.map_or("", |room| string(room, "name"))
        );
        card.found = text(acted, "engine_fact")
            .map(|facts| facts.lines().map(str::to_string).collect())
            .unwrap_or_default();
    } else {
        card.act = string(acted, "description").to_string();
    }
    card.people = told_here("playthrough_volitions")
        .into_iter()
        .filter(|row| text(row, "status") == Some("applied"))
        .map(|row| string(row, "fact").to_string())
        .collect();
    card.tolls = told_here("playthrough_tolls")
        .into_iter()
        .map(|toll| game.toll_to_s(toll))
        .collect();
    card.wounds = told_here("playthrough_blows")
        .into_iter()
        .filter_map(|blow| {
            let attacker = records.find("characters", int(blow, "attacker_id")?)?;
            let target = records.find("characters", int(blow, "target_id")?)?;
            Some(crate::turn::blow_to_s(
                string(attacker, "fullname"),
                target,
                int(blow, "damage")?,
                int(blow, "hp_after")?,
                int(blow, "round")?,
            ))
        })
        .collect();
    Some(card)
}
