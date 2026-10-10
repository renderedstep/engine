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
//! on a return, what has changed since the game last left the room;
//! what the turn noticed for the first time;
//! what the people there did; what the world took; who was hit; and how the
//! story ended, where it did.
//!
//! THE REQUEST IS BUILT AND NEVER SENT. A turn the game chose builds the
//! request a narrator would have been sent for it, at the point in the turn
//! a narrated game sends it, keeps it in the turn's journal
//! ([`WOULD_ASK`], and [`WOULD_END`] for a turn that closed the story), and
//! sends nothing. The paragraph's row carries those requests and the
//! narrator's prompt digest beside the player's words, so each paragraph is
//! a reference written against the very request a model would have answered.
//! The row also says what the player let it be used for ([`Consent`]).

use crate::model::Call;
use crate::playthrough::Game;
use crate::prompt_version;
use crate::records::{id, int, string, text, Records, Row};
use crate::turn::chooser;
use serde_json::{json, Value};

/// Who wrote a paragraph (`playthrough_paragraphs.author`).
pub const PLAYER: &str = "player";

/// The journal step a chosen turn keeps the request for its act in: the
/// narrator's, or the arrival writer's on a turn that walked into a room.
pub const WOULD_ASK: &str = "would_ask";

/// The journal step a chosen turn that closed the story keeps the request
/// for the game's last paragraph in.
pub const WOULD_END: &str = "would_end";

/// What the player let their paragraph be used for
/// (`playthrough_paragraphs.consent`). Nothing but the game, unless the
/// caller says otherwise.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Consent {
    /// Kept for the game alone, and in no set.
    #[default]
    None,
    /// The owner's own paragraph, which they keep as reference material for
    /// the narrator in a set of their own that is never published.
    Owner,
}

impl Consent {
    pub fn as_str(self) -> &'static str {
        match self {
            Consent::None => "none",
            Consent::Owner => "owner",
        }
    }

    pub fn parse(word: &str) -> Option<Consent> {
        match word {
            "none" => Some(Consent::None),
            "owner" => Some(Consent::Owner),
            _ => None,
        }
    }
}

/// A request a narrator would have been sent for a chosen turn, built and
/// not sent: which pass it is for (`narration`, `arrival` or `ending`) and
/// the call as a builder states it (`{system, user, schema, history}`).
#[derive(Clone, Debug, PartialEq)]
pub struct WouldAsk {
    pub purpose: String,
    pub request: Value,
}

impl WouldAsk {
    pub fn new(purpose: &str, call: &Call) -> WouldAsk {
        WouldAsk {
            purpose: purpose.to_string(),
            request: call.to_request(),
        }
    }

    /// The prompt digest of the pass it stands in for
    /// ([`prompt_version::digest`]), as a narrated turn's verdict records
    /// it.
    pub fn prompt_digest(&self) -> Option<String> {
        prompt_version::digest(&self.purpose, self.request["system"].as_str())
    }

    /// As the paragraph's row keeps it: `{purpose, prompt_digest, request}`.
    pub fn to_json(&self) -> Value {
        json!({
            "purpose": self.purpose,
            "prompt_digest": self.prompt_digest(),
            "request": self.request,
        })
    }

    /// As a journal step keeps it: the JSON text of its purpose and request.
    pub fn to_journal(&self) -> Value {
        Value::String(json!({ "purpose": self.purpose, "request": self.request }).to_string())
    }

    /// The inverse of [`WouldAsk::to_journal`].
    pub fn from_journal(value: &Value) -> Option<WouldAsk> {
        let kept: Value = serde_json::from_str(value.as_str()?).ok()?;
        Some(WouldAsk {
            purpose: kept["purpose"].as_str()?.to_string(),
            request: kept.get("request")?.clone(),
        })
    }
}

/// The requests a narrator would have been sent for the turn the game chose
/// that `scene` answered, in the order a narrated game sends them: the act's,
/// then the ending's where the turn closed the story. Empty for any other
/// scene, and for a turn played before the game kept them.
pub fn would_ask(records: &Records, playthrough: i64, scene: i64) -> Vec<WouldAsk> {
    let Some(command) = records.first("playthrough_commands", |row| {
        int(row, "playthrough_id") == Some(playthrough)
            && int(row, "result_scene_id") == Some(scene)
            && text(row, "command") == Some(chooser::LINE)
    }) else {
        return Vec::new();
    };
    let steps = &command["journal"]["steps"];
    [WOULD_ASK, WOULD_END]
        .iter()
        .filter_map(|step| WouldAsk::from_journal(&steps[*step]))
        .collect()
}

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
    /// What the turn noticed first, in the order it did: on the way in, on a
    /// look, and on the time it spent in the room ([`crate::noticed`]).
    pub noticed: Vec<String>,
    /// On a walk back into a room this game left before, what has changed
    /// there since among what it saw, one fact a line, or
    /// [`crate::revisit::UNCHANGED`] when nothing has ([`crate::revisit`]).
    /// Empty on any other turn.
    pub changed: Vec<String>,
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
        noticed: crate::noticed::names(
            records,
            &crate::noticed::noticed_on(records, playthrough, scene),
        ),
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
        card.changed = crate::revisit::since_on(records, playthrough, scene)
            .map(|since| since.facts())
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
