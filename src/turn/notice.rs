//! The tiers of what a game the player narrates notices, as writes: each
//! stamps `noticed_at` with story time on this game's copy of a thing or its
//! state for a person, and answers the things it noticed, in the order it
//! noticed them ([`crate::noticed`] says what each tier holds). In a game
//! told any other way each writes nothing and answers nothing.

use super::Mechanics;
use crate::engine::Error;
use crate::noticed::{self, concealed, stamped, ON_ARRIVAL};
use crate::records::{id, int, string, text, Row};
use crate::roll;
use serde_json::Value;
use std::cmp::Reverse;

impl Mechanics<'_> {
    fn narrated(&self) -> bool {
        !self.game().player_narrates()
    }

    fn stamp(&mut self, table: &str, rows: &[i64], at: i64) -> Result<(), Error> {
        for row in rows {
            self.update(table, *row, vec![("noticed_at", Value::from(at))])?;
        }
        Ok(())
    }

    /// What is always noticed: what the player carries, and this game's
    /// state for each person in the room it stands in.
    pub(crate) fn notice_at_hand(&mut self, room: Option<&Row>) -> Result<(), Error> {
        if self.narrated() {
            return Ok(());
        }
        let at = self.story_now();
        let game = self.game();
        let carried: Vec<i64> = game
            .carried()
            .into_iter()
            .filter(|item| !stamped(item))
            .map(id)
            .collect();
        let present: Vec<i64> = game.cast_in(room).into_iter().map(id).collect();
        let states: Vec<i64> = game
            .own("playthrough_npc_states")
            .into_iter()
            .filter(|state| !stamped(state))
            .filter(|state| int(state, "character_id").is_some_and(|who| present.contains(&who)))
            .map(id)
            .collect();
        self.stamp("items", &carried, at)?;
        self.stamp("playthrough_npc_states", &states, at)
    }

    /// Walking into `room`: every fixed piece in plain sight, and a rolled
    /// few loose things, larger bulk first, while the room has room to show
    /// them; and what is always noticed.
    pub(crate) fn notice_on_arrival(&mut self, room: &Row) -> Result<Vec<i64>, Error> {
        if self.narrated() {
            return Ok(Vec::new());
        }
        let at = self.story_now();
        let game = self.game();
        let lying: Vec<&Row> = game
            .items_lying_in(Some(room))
            .into_iter()
            .filter(|item| !stamped(item) && !concealed(item))
            .collect();
        let fixed = |item: &Row| text(item, "tier") == Some(crate::kit::FIXTURE);
        let (fixtures, mut loose): (Vec<&Row>, Vec<&Row>) =
            lying.into_iter().partition(|item| fixed(item));
        let mut seen: Vec<i64> = fixtures.into_iter().map(id).collect();
        loose.sort_by_key(|item| (Reverse(noticed::bulk_rank(string(item, "bulk"))), id(item)));
        let left = noticed::room_left(&game, Some(room)).saturating_sub(seen.len());
        if !loose.is_empty() && left > 0 {
            let mut rng = self.generator(at, id(room), roll::NOTICE);
            let few = roll::die(ON_ARRIVAL, &mut rng) as usize;
            seen.extend(loose.iter().take(few.min(left)).map(|item| id(item)));
        }
        self.stamp("items", &seen, at)?;
        self.notice_at_hand(Some(room))?;
        Ok(seen)
    }

    /// A look around the room the player stands in: everything in plain
    /// sight not yet noticed, in id order, while the room has room to show
    /// it.
    pub(crate) fn notice_on_look(&mut self) -> Result<Vec<i64>, Error> {
        if self.narrated() {
            return Ok(Vec::new());
        }
        let at = self.story_now();
        let here = self.here();
        let game = self.game();
        let left = noticed::room_left(&game, here.as_ref());
        let seen: Vec<i64> = noticed::unnoticed_in(&game, here.as_ref())
            .into_iter()
            .take(left)
            .map(id)
            .collect();
        self.stamp("items", &seen, at)?;
        self.notice_at_hand(here.as_ref())?;
        Ok(seen)
    }

    /// A played turn that did not leave the room: one more thing in plain
    /// sight, by `Roll::NOTICE` for the submission `sequence`; and what is
    /// always noticed.
    pub(crate) fn notice_in_time(&mut self, sequence: i64) -> Result<Vec<i64>, Error> {
        if self.narrated() {
            return Ok(Vec::new());
        }
        let at = self.story_now();
        let here = self.here();
        let game = self.game();
        let unnoticed: Vec<i64> = noticed::unnoticed_in(&game, here.as_ref())
            .into_iter()
            .map(id)
            .collect();
        let mut seen = Vec::new();
        if !unnoticed.is_empty() && noticed::room_left(&game, here.as_ref()) > 0 {
            let mut rng = self.generator(at, sequence, roll::NOTICE);
            seen.push(unnoticed[roll::one_of(unnoticed.len(), &mut rng)]);
        }
        self.stamp("items", &seen, at)?;
        self.notice_at_hand(here.as_ref())?;
        Ok(seen)
    }
}
