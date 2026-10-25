//! Breaking a thing apart (`tear`, `smash`; [`crate::divide`]): the check
//! its material takes, if any, and then the thing ended and its pieces
//! begun.
//!
//! THE CHECK IS ONE ROLL, WHERE THE MATERIAL TAKES ONE. The ability the
//! material names, less what it takes off, on one d20 thrown out of a
//! generator keyed on the thing's own row at story time (`Roll::TEAR`,
//! `Roll::SMASH`), and kept like every other check, so the turn's receipts
//! carry it. A failed check is a played turn with nothing changed: the
//! thing is whole, and the next attempt is thrown at a later story time.
//!
//! THE THING ENDS AND ITS PIECES BEGIN WHERE IT WAS. It is spent as a burned
//! thing is, `divided`, in no place; one row of this game's own is written
//! where it was -- in the player's hands, or lying where it lay -- with
//! nothing the world's template could bring back. In a game the player
//! narrates, the pieces are noticed as they appear.

use super::{check_ability, Mechanics};
use crate::divide::{self, DIVIDED, SMASH};
use crate::engine::Error;
use crate::records::{id, int, string, text, Row};
use crate::roll;
use crate::room::Choice;
use serde_json::Value;

impl Mechanics<'_> {
    /// `tear` and `smash`: the check, and the thing ended and its pieces
    /// begun, or whole.
    pub(super) fn divide(&mut self, choice: &Choice) -> Result<(&'static str, String), Error> {
        let Some(thing) = &choice.item else {
            return Err(Error::Database("a break apart with no thing".into()));
        };
        let row = self.row("items", thing.id)?;
        let kind = choice.kind.as_str();
        let Some(act) = text(&row, "material").and_then(|material| divide::act(material, kind))
        else {
            return Err(Error::Database(format!(
                "{kind} on a thing of no such material"
            )));
        };
        let attempt = crate::text::ruby_downcase(&choice.name());
        let at = self.story_now();
        let check = match &act.check {
            None => None,
            Some((ability, penalty)) => {
                let Some(who) = self.player() else {
                    return Ok((
                        "failed",
                        format!("You tried to {attempt}. Nobody is playing, so nothing changed."),
                    ));
                };
                let kind_roll = if kind == SMASH {
                    roll::SMASH
                } else {
                    roll::TEAR
                };
                let mut rng = self.generator(at, thing.id, kind_roll);
                let Some(check) = self.thrown(check_ability(&who, ability, *penalty, &mut rng))
                else {
                    return Ok((
                        "failed",
                        format!(
                            "You tried to {attempt}. {} has no abilities, so there is no {ability} \
                             to do it with; nothing changed.",
                            string(&who, "fullname")
                        ),
                    ));
                };
                if !check.passed() {
                    return Ok((
                        "failed",
                        format!(
                            "You tried to {attempt}. {check}. It held together; nothing changed."
                        ),
                    ));
                }
                Some(check)
            }
        };
        let pieces = self.leave_pieces(&row, kind, at)?;
        let carried = int(&row, "location_id").is_none() && int(&row, "character_id").is_none();
        let rolled = check.map_or(String::new(), |check| format!(" {check}."));
        let pieces = crate::facts::definite_name(&pieces);
        let whereabouts = if carried {
            format!("you are holding {pieces}")
        } else {
            format!("{pieces} lie where it was")
        };
        Ok((
            "applied",
            format!(
                "You {} {}.{rolled} It is gone; {whereabouts}.",
                if kind == SMASH {
                    "smashed"
                } else {
                    "tore apart"
                },
                crate::facts::definite_name(&row),
            ),
        ))
    }

    /// The source spent as `divided`, and the row of what it leaves written
    /// where it was, in this game's own layer: the row written.
    fn leave_pieces(&mut self, source: &Row, kind: &str, at: i64) -> Result<Row, Error> {
        let Some(mut values) = divide::pieces(source, kind) else {
            return Err(Error::Database(format!("{kind} leaves nothing")));
        };
        for column in ["location_id", "character_id", "x", "y", "within_id", "how"] {
            values.push((column, source.get(column).cloned().unwrap_or(Value::Null)));
        }
        values.push(("playthrough_id", Value::from(self.playthrough)));
        if self.game().player_narrates() {
            values.push(("noticed_at", Value::from(at)));
        }
        self.spend(id(source), DIVIDED)?;
        self.insert("items", values)
    }
}
