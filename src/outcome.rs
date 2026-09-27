//! What a turn left behind, read off the records once it is over
//! (`Playthrough::Mechanics::State`): where the player stands, what leads out,
//! what lies here and is carried, who is present and who is fighting, how
//! much is left of everybody, where the story's arc stands, and what the
//! story's clock still owes.

use crate::playthrough::Game;
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::turn::{Report, ABILITIES};

/// A room, by id, name and whether it is written yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Room {
    pub id: i64,
    pub name: String,
    /// `stub` or `realized`.
    pub detail: String,
    /// Its storey, for a room with a box.
    pub storey: Option<i64>,
}

/// A thing or a person, by id and name (a person's full name).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Named {
    pub id: i64,
    pub name: String,
}

/// Something in reach that has writing on it or could; `text` is none for a
/// readable thing left blank.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Inscription {
    pub id: i64,
    pub name: String,
    pub text: Option<String>,
}

/// The records after a turn, for one playthrough.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub location: Option<Room>,
    /// Every set below is sorted by name, then id.
    pub exits: Vec<Room>,
    pub here: Vec<Named>,
    pub carrying: Vec<Named>,
    pub present: Vec<Named>,
    pub foes: Vec<Named>,
    pub inscription: Vec<Inscription>,
    /// The player's hit points, for a player with a stat block.
    pub hp: Option<i64>,
    /// Hit points of everybody present with a stat block, by full name.
    pub hp_of: Vec<(String, i64)>,
    /// The player's three abilities, in order.
    pub abilities: Option<Vec<(String, Option<i64>)>>,
    /// The game is over.
    pub dead: bool,
    /// Each beat of the main arc by position: `unbound`, `bound` or
    /// `reached`.
    pub quest: Vec<(i64, String)>,
    /// The reached ending's name, or `none`.
    pub ending: String,
    /// The closing scene's words, once the game has ended with one.
    pub ending_words: Option<String>,
    /// Events the clock owes this game that have not happened yet.
    pub scheduled: i64,
    /// Scheduled events that have happened.
    pub fired: i64,
}

/// A line's report and the records it left behind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub report: Report,
    pub state: State,
}

impl Eq for Report {}

fn room(location: &Row) -> Room {
    Room {
        id: id(location),
        name: string(location, "name").to_string(),
        detail: string(location, "detail_level").to_string(),
        storey: int(location, "z"),
    }
}

fn named(rows: &[&Row], column: &str) -> Vec<Named> {
    let mut named: Vec<Named> = rows
        .iter()
        .map(|row| Named {
            id: id(row),
            name: string(row, column).to_string(),
        })
        .collect();
    named.sort_by(|a, b| (&a.name, a.id).cmp(&(&b.name, b.id)));
    named
}

impl State {
    /// Reads one playthrough's state off the records.
    pub fn read(records: &Records, playthrough: i64) -> State {
        let game = Game::new(records, playthrough);
        let here = game.current_location();
        let story = game.story_id();
        let story_protagonist = records
            .first("characters", |row| {
                int(row, "story_id") == Some(story) && flag(row, "is_protagonist")
            })
            .map(id);
        let present: Vec<&Row> = game
            .cast_in(here)
            .into_iter()
            .filter(|who| Some(id(who)) != story_protagonist)
            .collect();
        let lying = game.items_lying_in(here);
        let carried = game.carried();

        let mut exits: Vec<Room> = if here.is_some() {
            game.exits().into_iter().map(room).collect()
        } else {
            Vec::new()
        };
        exits.sort_by(|a, b| (&a.name, a.id).cmp(&(&b.name, b.id)));

        let mut inscription: Vec<Inscription> = lying
            .iter()
            .chain(carried.iter())
            .filter(|item| {
                flag(item, "readable")
                    || text(item, "inscription").is_some_and(|words| !crate::text::is_blank(words))
            })
            .map(|item| Inscription {
                id: id(item),
                name: string(item, "name").to_string(),
                text: text(item, "inscription")
                    .filter(|words| !crate::text::is_blank(words))
                    .map(str::to_string),
            })
            .collect();
        inscription.sort_by(|a, b| (&a.name, a.id).cmp(&(&b.name, b.id)));

        let mut hp_of: Vec<(String, i64)> = present
            .iter()
            .filter_map(|who| {
                game.vitals_for(who)
                    .map(|condition| (string(who, "fullname").to_string(), condition.hp))
            })
            .collect();
        hp_of.sort();

        let player = game.protagonist();
        State {
            location: here.map(room),
            exits,
            here: named(&lying, "name"),
            carrying: named(&carried, "name"),
            present: named(&present, "fullname"),
            foes: named(&game.foes_in(here), "fullname"),
            inscription,
            hp: player
                .and_then(|who| game.vitals_for(who))
                .map(|condition| condition.hp),
            hp_of,
            abilities: player.map(|who| {
                ABILITIES
                    .iter()
                    .map(|ability| (ability.to_string(), int(who, ability)))
                    .collect()
            }),
            dead: int(game.row, "ended_at").is_some(),
            quest: arc_state(&game),
            ending: ending(&game).unwrap_or_else(|| "none".to_string()),
            ending_words: game
                .current_scene()
                .filter(|scene| {
                    matches!(text(scene, "resolved_action"), Some("conclude" | "ending"))
                })
                .map(|scene| string(scene, "description").to_string()),
            scheduled: events(&game, |event| {
                int(event, "scheduled_for").is_some() && int(event, "fired_at").is_none()
            }),
            fired: events(&game, |event| int(event, "fired_at").is_some()),
        }
    }
}

/// `Quest.main_arc`: the story's arc with no parent, lowest id first.
pub fn main_arc<'a>(game: &Game<'a>) -> Option<&'a Row> {
    let story = Some(game.story_id());
    game.records.first("quests", |quest| {
        int(quest, "story_id") == story && int(quest, "parent_quest_id").is_none()
    })
}

/// A quest's steps, by position.
pub fn steps<'a>(records: &'a Records, quest: &Row) -> Vec<&'a Row> {
    let quest = id(quest);
    let mut steps = records.select("quest_steps", |step| int(step, "quest_id") == Some(quest));
    steps.sort_by_key(|step| (int(step, "position"), id(step)));
    steps
}

/// `Playthrough::Mechanics#arc_state`, less the summaries.
fn arc_state(game: &Game) -> Vec<(i64, String)> {
    let Some(quest) = main_arc(game) else {
        return Vec::new();
    };
    let reached: Vec<i64> = game
        .own("playthrough_beats")
        .iter()
        .filter_map(|beat| int(beat, "quest_step_id"))
        .collect();
    steps(game.records, quest)
        .into_iter()
        .map(|step| {
            let state = if reached.contains(&id(step)) {
                "reached"
            } else if int(step, "target_id").is_some()
                || text(step, "trigger_kind") == Some("time_passed")
            {
                "bound"
            } else {
                "unbound"
            };
            (int(step, "position").unwrap_or_default(), state.to_string())
        })
        .collect()
}

/// `Playthrough::Arc#ending`: this game's ending of the main arc.
pub fn ending_row<'a>(game: &Game<'a>) -> Option<(&'a Row, &'a Row)> {
    let quest = id(main_arc(game)?);
    game.own("playthrough_endings")
        .into_iter()
        .find_map(|ending| {
            let outcome = game
                .records
                .find("quest_outcomes", int(ending, "quest_outcome_id")?)?;
            (int(outcome, "quest_id") == Some(quest)).then_some((ending, outcome))
        })
}

fn ending(game: &Game) -> Option<String> {
    ending_row(game).map(|(_, outcome)| string(outcome, "name").to_string())
}

/// `WorldEvent.for_a_game`, counted: the world's events and this game's own.
fn events(game: &Game, keep: impl Fn(&Row) -> bool) -> i64 {
    let story = Some(game.story_id());
    let own = game.id();
    game.records
        .select("world_events", |event| {
            int(event, "story_id") == story
                && int(event, "playthrough_id").is_none_or(|of| of == own)
                && keep(event)
        })
        .len() as i64
}
