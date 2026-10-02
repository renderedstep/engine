//! Where in a room one thing stands (`Location::Spot` and `Location::Placement`).

use crate::boxes::{Box, Spot};
use crate::random::Random;
use crate::roll::{self, Seed};

/// `Spot.inside`: a cell of the box, x drawn before y.
pub fn inside(room: &Box, rng: &mut Random) -> Spot {
    let x = room.x + roll::one_of(room.width as usize, rng) as i64;
    let y = room.y + roll::one_of(room.depth as usize, rng) as i64;
    Spot { x, y }
}

/// Which table a placed record comes from; each has its own kind of roll,
/// because their ids collide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Record {
    Item(i64),
    Character(i64),
}

/// The playthrough a placement belongs to, and its story time in seconds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Game {
    pub playthrough_id: i64,
    pub story_now: i64,
}

/// `Placement.in_the_world` (no game) or `Placement.in_a_game`. A room with
/// no box places nothing.
pub fn place(
    story_id: i64,
    room: Option<&Box>,
    record: Record,
    game: Option<Game>,
) -> Option<Spot> {
    let room = room?;
    let (kind, id) = match record {
        Record::Item(id) => (roll::ITEM_POSITION, id),
        Record::Character(id) => (roll::CHARACTER_POSITION, id),
    };
    let game = game.unwrap_or(Game {
        playthrough_id: 0,
        story_now: 0,
    });
    let mut rng = Seed {
        story: story_id.into(),
        playthrough: game.playthrough_id.into(),
        at: game.story_now.into(),
        sequence: id.into(),
        kind,
    }
    .generator();
    Some(inside(room, &mut rng))
}

/// `Placement.along`: where a thing thrown from `from` at `to` comes down
/// when it carries only `paces`, along the line between them, rounded
/// towards where it was thrown from. The one spot that is computed rather
/// than rolled. A throw that carries as far as `to` lands there.
pub fn along(from: Spot, to: Spot, paces: i64) -> Spot {
    let distance = crate::physics::paces(from, to);
    if paces >= distance {
        return to;
    }
    let step = |start: i64, end: i64| start + (end - start) * paces.max(0) / distance;
    Spot {
        x: step(from.x, to.x),
        y: step(from.y, to.y),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_throw_lands_its_paces_along_the_line_and_a_long_one_where_it_was_aimed() {
        let from = Spot { x: 1, y: 1 };
        let to = Spot { x: 7, y: 3 };
        assert_eq!(along(from, to, 4), Spot { x: 4, y: 2 });
        assert_eq!(along(from, to, 3), Spot { x: 3, y: 1 });
        assert_eq!(along(from, to, 8), to);
        assert_eq!(along(from, to, 20), to);
        assert_eq!(along(to, from, 4), Spot { x: 4, y: 2 });
        assert_eq!(along(from, from, 3), from);
    }
}
