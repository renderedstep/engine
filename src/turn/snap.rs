//! Breaking a part off a fixed piece (`Playthrough::PhysicalAction`'s
//! `break`): a tree's branch, a wall's loose stone, attached to the piece
//! since the room was furnished (`kit::roll`).
//!
//! THE ATTEMPT IS ONE CHECK. Strength, less what the part's hold takes off
//! it (`hold_penalty` in `data/item/kits.yml`), on one d20 thrown out of a
//! generator keyed on the part's own row at story time (`Roll::SNAP`), the
//! throw's seed with a kind of its own. The check is kept like every other,
//! so the turn's receipts carry it. A failed check is a played turn with
//! nothing moved: the part stays attached, and the next attempt on it is
//! thrown at a later story time.
//!
//! A PART THAT COMES AWAY IS IN THE HAND. Its row is the thing now: portable,
//! attached to nothing, carried, with the bulk and fragility it was born
//! with. A part that can break (brittle, fragile) then throws the break die
//! as a thing put down on this floor would (`physics::share` at
//! `Landing::Dropped`, `Roll::BREAK`), and when it comes up the part came
//! away in pieces and is spent: the hand holds nothing new.

use super::{check_ability, Landing, Mechanics};
use crate::engine::Error;
use crate::kit;
use crate::physics;
use crate::records::{int, string, text};
use crate::roll;
use crate::room::Choice;
use serde_json::Value;

impl Mechanics<'_> {
    /// `break`: the check, and the part in the hand, in pieces, or still
    /// attached.
    pub(super) fn snap(&mut self, choice: &Choice) -> Result<(&'static str, String), Error> {
        let Some(part) = &choice.item else {
            return Err(Error::Database("a break with no part".into()));
        };
        let row = self.row("items", part.id)?;
        let attempt = crate::text::ruby_downcase(&choice.name());
        let Some(who) = self.player() else {
            return Ok((
                "failed",
                format!("You tried to {attempt}. Nobody is playing, so nothing came away."),
            ));
        };
        let at = self.story_now();
        let mut rng = self.generator(at, part.id, roll::SNAP);
        let penalty = kit::hold_penalty(text(&row, "hold"));
        let Some(check) = self.thrown(check_ability(&who, "strength", penalty, &mut rng)) else {
            return Ok((
                "failed",
                format!(
                    "You tried to {attempt}. {} has no abilities, so there is no strength to \
                     break it with; nothing came away.",
                    string(&who, "fullname")
                ),
            ));
        };
        if !check.passed() {
            return Ok((
                "failed",
                format!("You tried to {attempt}. {check}. It held; nothing came away."),
            ));
        }
        let mut values = vec![
            ("tier", Value::from(kit::PORTABLE)),
            ("within_id", Value::Null),
            ("how", Value::Null),
            ("hold", Value::Null),
            ("location_id", Value::Null),
            ("character_id", Value::Null),
            ("x", Value::Null),
            ("y", Value::Null),
            (
                "description",
                Value::from(kit::plain_description(string(&row, "name"))),
            ),
        ];
        if self.game().player_narrates() && !crate::noticed::stamped(&row) {
            values.push(("noticed_at", Value::from(at)));
        }
        self.update("items", part.id, values)?;
        let surface = self
            .here()
            .and_then(|room| text(&room, "surface").map(str::to_string));
        let share = physics::share(
            text(&row, "fragility"),
            Landing::Dropped,
            surface.as_deref(),
        )
        .filter(|_| int(&row, "playthrough_id").is_some());
        let rolled = share.map(|share| {
            let mut rng = self.generator(at, part.id, roll::BREAK);
            physics::break_roll(share, &mut rng)
        });
        let done = attempt.replacen("break", "broke", 1);
        if rolled.is_some_and(|rolled| rolled.broke) {
            self.spend(part.id, physics::BROKEN)?;
            return Ok((
                "applied",
                format!(
                    "You {done}. {check}. It came away in pieces; you are holding nothing new."
                ),
            ));
        }
        Ok((
            "applied",
            format!("You {done}. {check}. You are holding it now."),
        ))
    }
}
