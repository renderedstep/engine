//! `Scene::Narrator#prompt_for`: the narrator's request for one typed line,
//! the moment it happens in and what the engine already did; and
//! `Scene::Ending#prompt_for`, the request for a game's last paragraph.
//!
//! The turn asks for it once it has written the line's effect, so the moment
//! is read off the rows as they stand after that write. A caller holding a
//! staged position's rows builds the same request with nothing played.

use crate::data;
use crate::facts;
use crate::model::Call;
use crate::moment::{Handled, Moment};
use crate::playthrough::Game;
use crate::records::Row;

/// The narrator's prompt for `command` in the moment `game` stands in.
pub fn prompt(
    game: Game,
    command: &str,
    fact: Option<&str>,
    doing: Option<&str>,
    handled: Option<Handled>,
) -> String {
    let moment = Moment {
        game,
        handled,
        ending: None,
    };
    facts::framing(&moment.narration_context(true, true), command, fact, doing)
}

/// The whole call: the narrator's instructions and [`prompt`]. It carries no
/// schema, because the narration streams.
pub fn call(
    game: Game,
    command: &str,
    fact: Option<&str>,
    doing: Option<&str>,
    handled: Option<Handled>,
) -> Call {
    Call {
        system: Some(data::narrator_instructions().to_string()),
        ..Call::prompt(prompt(game, command, fact, doing, handled))
    }
}

/// The last paragraph's call, on the line that concluded the story's arc:
/// the ending's instructions and the moment with the outcome `outcome` (a
/// `quest_outcomes` row) this game reached.
pub fn ending_call(game: Game, outcome: &Row) -> Call {
    let moment = Moment {
        game,
        handled: None,
        ending: Some(outcome),
    };
    Call {
        system: Some(data::ending_instructions().to_string()),
        ..Call::prompt(format!(
            "{}\n\nWrite the ending.\n",
            moment.narration_context(true, true)
        ))
    }
}
