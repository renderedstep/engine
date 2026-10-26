//! What is within a person's reach in a laid-out room, and where they stand
//! when they walk up to something or step back from it; and what the game's
//! doctor reports about a room nobody can stand anywhere in.
//!
//! ARM'S LENGTH. Within reach is your own cell and the eight around it. A
//! spot is a cell of the room's box in the same plane the box is read in
//! (`spot::inside`), so two spots of one room measure against each other
//! directly. No die is thrown anywhere here.
//!
//! WALKING UP. Going to a thing, a person or a way out puts you in the
//! nearest free cell within arm's length of it, the nearest to where you
//! stood; a cell somebody or something else takes is not free, and the
//! cell you stand in always is.
//!
//! STEPPING BACK. Stepping back puts you two paces farther from what you
//! step back from, inside the walls; with no room behind you it is refused.
//!
//! Nothing plays any of this yet: no act reads it, and no spot is written
//! by it. A room with no layout has no cells, so nothing here can be asked
//! there; the doctor counts such rooms (`room_without_a_layout`), as a
//! warning that changes nothing.

use crate::boxes::{Box, Spot};
use crate::noticed::Finding;
use crate::physics::paces;
use crate::plan::{box_of, interior};
use crate::records::{id, int, string, Records};

/// The doctor's code for a room the party can stand in with no layout.
pub const ROOM_WITHOUT_A_LAYOUT: &str = "room_without_a_layout";

/// How many paces a step back puts between you and what you step back
/// from, beyond what was there.
pub const STEP_BACK_PACES: i64 = 2;

/// Why a step back is not taken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    /// Every cell two paces farther off lies outside the walls.
    NoRoomBehind,
}

/// Whether `to` is within arm's length of `from`: the same cell or one of
/// the eight around it.
pub fn within_reach(from: Spot, to: Spot) -> bool {
    (from.x - to.x).abs() <= 1 && (from.y - to.y).abs() <= 1
}

/// Whether a spot is a cell of the room.
pub fn inside(room: &Box, spot: Spot) -> bool {
    spot.x >= room.x
        && spot.x < room.x + room.width
        && spot.y >= room.y
        && spot.y < room.y + room.depth
}

/// Every cell of the room, north to south and west to east within a row.
fn cells(room: &Box) -> impl Iterator<Item = Spot> + '_ {
    (room.y..room.y + room.depth)
        .flat_map(move |y| (room.x..room.x + room.width).map(move |x| Spot { x, y }))
}

/// Where you stand after walking up to `target` from `from`: the free cell
/// of the room within arm's length of it nearest `from`, ties going to the
/// first north to south, then west to east. `taken` are the cells others
/// stand or lie in; `target`'s own cell is never free, and `from` always
/// is, so one already within reach stays put. None when every cell within
/// reach of it is taken or outside the walls.
pub fn nearest_free(room: &Box, target: Spot, from: Spot, taken: &[Spot]) -> Option<Spot> {
    cells(room)
        .filter(|&cell| within_reach(cell, target) && cell != target)
        .filter(|&cell| cell == from || !taken.contains(&cell))
        .min_by_key(|&cell| paces(cell, from))
}

/// The nearest of `people` within arm's length of `from`, as its index: what
/// a step back with nothing named steps back from. Ties go to the first.
pub fn nearest_within_reach(from: Spot, people: &[Spot]) -> Option<usize> {
    people
        .iter()
        .enumerate()
        .filter(|(_, &at)| within_reach(from, at))
        .min_by_key(|(_, &at)| paces(from, at))
        .map(|(index, _)| index)
}

/// Where you stand after stepping back from `away` at `from`: a cell inside
/// the walls two paces from where you stood and two paces farther from
/// `away`. The straightest wins, the one whose larger axis from `away` is
/// largest, then the first north to south, west to east.
pub fn step_back(room: &Box, from: Spot, away: Spot) -> Result<Spot, Refused> {
    let apart = paces(from, away);
    let straightness = |cell: Spot| (cell.x - away.x).abs().max((cell.y - away.y).abs());
    cells(room)
        .filter(|&cell| {
            paces(cell, from) == STEP_BACK_PACES && paces(cell, away) == apart + STEP_BACK_PACES
        })
        .min_by_key(|&cell| -straightness(cell))
        .ok_or(Refused::NoRoomBehind)
}

/// Every place of a story the party could stand in that has no layout, so
/// that no spot in it can be measured: neither a box of its own nor a
/// footprint whose rooms are walked into instead. A warning only: nothing
/// plays differently for it.
pub fn findings(records: &Records, story: i64) -> Vec<Finding> {
    records
        .select("locations", |row| {
            int(row, "story_id") == Some(story) && box_of(row).is_none() && !interior(row)
        })
        .into_iter()
        .map(|room| Finding {
            code: ROOM_WITHOUT_A_LAYOUT,
            severity: "warning",
            message: format!(
                "{:?} has no layout: nobody can be given a spot in it, so what is within reach \
                 there cannot be measured",
                string(room, "name")
            ),
            remedy: "manual",
            subject: ("locations", id(room)),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room() -> Box {
        Box {
            x: 2,
            y: 3,
            z: 0,
            width: 5,
            depth: 4,
        }
    }

    fn at(x: i64, y: i64) -> Spot {
        Spot { x, y }
    }

    #[test]
    fn arms_length_is_your_cell_and_the_eight_around_it() {
        let here = at(4, 4);
        for dx in -1..=1 {
            for dy in -1..=1 {
                assert!(within_reach(here, at(4 + dx, 4 + dy)));
            }
        }
        assert!(!within_reach(here, at(6, 4)));
        assert!(!within_reach(here, at(4, 2)));
        assert!(!within_reach(here, at(6, 6)));
    }

    #[test]
    fn walking_up_stops_in_the_nearest_free_cell_beside_it() {
        assert_eq!(
            nearest_free(&room(), at(6, 6), at(2, 3), &[]),
            Some(at(5, 5))
        );
        assert_eq!(
            nearest_free(&room(), at(6, 6), at(2, 3), &[at(5, 5)]),
            Some(at(6, 5))
        );
        assert_eq!(
            nearest_free(&room(), at(5, 4), at(4, 4), &[]),
            Some(at(4, 4))
        );
        assert_eq!(
            nearest_free(&room(), at(5, 4), at(4, 4), &[at(4, 4)]),
            Some(at(4, 4))
        );
    }

    #[test]
    fn walking_up_to_something_hemmed_in_finds_nowhere() {
        let corner = at(2, 3);
        let taken = [at(3, 3), at(2, 4), at(3, 4)];
        assert_eq!(nearest_free(&room(), corner, at(6, 6), &taken), None);
    }

    #[test]
    fn a_step_back_goes_straight_away_two_paces() {
        assert_eq!(step_back(&room(), at(4, 5), at(4, 6)), Ok(at(4, 3)));
        assert_eq!(step_back(&room(), at(4, 4), at(3, 4)), Ok(at(6, 4)));
    }

    #[test]
    fn a_step_back_with_the_wall_straight_behind_goes_aside() {
        assert_eq!(step_back(&room(), at(4, 4), at(4, 5)), Ok(at(3, 3)));
    }

    #[test]
    fn a_step_back_against_a_wall_turns_aside_or_is_refused() {
        assert_eq!(step_back(&room(), at(2, 4), at(3, 4)), Ok(at(2, 6)));
        assert_eq!(
            step_back(&room(), at(2, 3), at(3, 4)),
            Err(Refused::NoRoomBehind)
        );
    }

    #[test]
    fn nothing_named_steps_back_from_the_nearest_within_reach() {
        let people = [at(6, 6), at(5, 5), at(4, 5)];
        assert_eq!(nearest_within_reach(at(4, 4), &people), Some(2));
        assert_eq!(nearest_within_reach(at(2, 3), &people), None);
    }
}
