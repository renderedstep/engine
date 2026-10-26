//! A layout for every room that stands on its own: a street, a square, a
//! field, a one-room shop -- any room with no building around it.
//!
//! ITS OWN PLANE. A room inside a place is a box in that place's plane
//! (`boxes`, `interior`). A room with no parent is read in a plane of its
//! own, so its box sits at the origin, `0, 0` on storey `0`, and is never
//! measured against another room's: two rooms that stand on their own share
//! no plane, and a storey is compared only between rooms of one place
//! (`plan::storey`).
//!
//! ITS SIZE. Width then depth, each rolled from its sort's band
//! (`data/location/size.yml`) out of a generator keyed on the room's own row
//! (`roll::ROOM_SIZE`). A room with no sort, or one the table does not know,
//! takes the default band. No model sets a number.
//!
//! ITS WAYS OUT. Each way out leaves by one side, north, east, south or
//! west, rolled on the doorway's own row (`roll::WAY_OUT_SIDE`) every time
//! it is read and never kept. The cell beside it is that side's middle
//! cell, the western or northern of two. Inside a building a doorway stays
//! on the wall two rooms share.
//!
//! ROOMS THAT EXIST. [`lay_out`] is what each room of a story that stands
//! on its own and has no layout yet is given, with a spot for each of the
//! world's own things and people in it that has none, as `Placement` rolls
//! one in the world; `Engine::lay_out_rooms` writes it. A room with a parent
//! and no box is left: it is read in its parent's plane, which nothing here
//! sizes.
//!
//! Nothing a model reads names any of it: `plan::Plan::of` needs a parent,
//! so neither the narrator nor the room writer hears of these layouts.

use crate::boxes::{Box, Spot, EAST, NORTH, SOUTH, WALLS, WEST};
use crate::data::{self, Sides};
use crate::plan::interior;
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::roll::{self, Seed};
use crate::spot;

/// The bands a room of this sort is sized from.
pub fn sides(kind: Option<&str>) -> Sides {
    let table = data::location_size();
    kind.and_then(|kind| {
        table
            .kinds
            .iter()
            .find(|(sort, _)| sort == kind)
            .map(|(_, sides)| *sides)
    })
    .unwrap_or(table.default)
}

/// `[width, depth]` of a room that stands on its own, each drawn from its
/// sort's band, width first, on the room's own row.
pub fn size(story: i64, room: i64, kind: Option<&str>) -> (i64, i64) {
    let sides = sides(kind);
    let mut rng = Seed {
        story: story.into(),
        sequence: room.into(),
        kind: roll::ROOM_SIZE,
        ..Seed::default()
    }
    .generator();
    let mut draw =
        |(low, high): (i64, i64)| low + roll::one_of((high - low + 1) as usize, &mut rng) as i64;
    let width = draw(sides.width);
    let depth = draw(sides.depth);
    (width, depth)
}

/// The box of a room that stands on its own: at the origin of its own
/// plane, on storey 0, sized by [`size`].
pub fn own_box(story: i64, room: i64, kind: Option<&str>) -> Box {
    let (width, depth) = size(story, room, kind);
    Box {
        x: 0,
        y: 0,
        z: 0,
        width,
        depth,
    }
}

/// Whether a room stands on its own: no place around it.
pub fn stands_alone(room: &Row) -> bool {
    int(room, "parent_location_id").is_none()
}

/// Whether a room stands on its own and has no layout yet: neither a box
/// nor a footprint (a footprint with no box is a place, whose rooms are laid
/// out inside it instead).
pub fn wants_one(room: &Row) -> bool {
    stands_alone(room) && !interior(room)
}

/// Which side of a room that stands on its own the way out `way` (its
/// `location_connections` row) leaves by.
pub fn side(story: i64, way: i64) -> &'static str {
    let mut rng = Seed {
        story: story.into(),
        sequence: way.into(),
        kind: roll::WAY_OUT_SIDE,
        ..Seed::default()
    }
    .generator();
    WALLS[roll::one_of(WALLS.len(), &mut rng)]
}

/// The cell of `room` beside its `side`: that side's middle cell, the
/// western or northern of two.
///
/// # Panics
/// When `side` is not one of `boxes::WALLS`.
pub fn beside(room: &Box, side: &str) -> Spot {
    let across = room.x + (room.width - 1).div_euclid(2);
    let down = room.y + (room.depth - 1).div_euclid(2);
    match side {
        NORTH => Spot {
            x: across,
            y: room.y,
        },
        SOUTH => Spot {
            x: across,
            y: room.y + room.depth - 1,
        },
        WEST => Spot { x: room.x, y: down },
        EAST => Spot {
            x: room.x + room.width - 1,
            y: down,
        },
        other => panic!("no side {other}"),
    }
}

/// Where a thing thrown at the way out `way` of a room that stands on its
/// own leaves it: the cell beside the side the way leaves by.
pub fn by_the_way_out(story: i64, room: &Box, way: i64) -> Spot {
    beside(room, side(story, way))
}

/// One room given a layout, and a spot for each of the world's own things
/// and people in it that had none, by id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaidOut {
    pub location: i64,
    pub name: String,
    pub r#box: Box,
    pub items: Vec<(i64, Spot)>,
    pub people: Vec<(i64, Spot)>,
}

/// Every room of `story` that stands on its own and has no layout yet, in
/// id order, as it would be laid out: its box from [`own_box`], and a spot,
/// as `Placement` rolls one in the world, for each of the world's own
/// things lying loose on its floor and each person in it but the
/// protagonist, whose spot is never kept. A thing on or in a piece keeps no
/// spot of its own, and a game's own copies are left to the game. Rolls
/// nothing that is already there, so asking again after it is written finds
/// nothing.
pub fn lay_out(records: &Records, story: i64) -> Vec<LaidOut> {
    let unplaced = |row: &Row| int(row, "x").is_none() || int(row, "y").is_none();
    records
        .select("locations", |row| {
            int(row, "story_id") == Some(story) && wants_one(row)
        })
        .into_iter()
        .map(|room| {
            let here = id(room);
            let r#box = own_box(story, here, text(room, "kind"));
            let items = records
                .select("items", |item| {
                    int(item, "location_id") == Some(here)
                        && int(item, "playthrough_id").is_none()
                        && int(item, "character_id").is_none()
                        && int(item, "within_id").is_none()
                        && unplaced(item)
                })
                .into_iter()
                .filter_map(|item| {
                    spot::place(story, Some(&r#box), spot::Record::Item(id(item)), None)
                        .map(|at| (id(item), at))
                })
                .collect();
            let people = records
                .select("characters", |person| {
                    int(person, "location_id") == Some(here)
                        && !flag(person, "is_protagonist")
                        && unplaced(person)
                })
                .into_iter()
                .filter_map(|person| {
                    spot::place(
                        story,
                        Some(&r#box),
                        spot::Record::Character(id(person)),
                        None,
                    )
                    .map(|at| (id(person), at))
                })
                .collect();
            LaidOut {
                location: here,
                name: string(room, "name").to_string(),
                r#box,
                items,
                people,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_size_falls_inside_its_band() {
        for (kind, sides) in &data::location_size().kinds {
            for room in 1..40 {
                let (width, depth) = size(7, room, Some(kind));
                assert!((sides.width.0..=sides.width.1).contains(&width), "{kind}");
                assert!((sides.depth.0..=sides.depth.1).contains(&depth), "{kind}");
            }
        }
    }

    #[test]
    fn a_room_with_no_sort_or_an_unknown_one_takes_the_default_band() {
        let default = data::location_size().default;
        assert_eq!(sides(None), default);
        assert_eq!(sides(Some("a sort nobody wrote")), default);
        assert_ne!(sides(Some("field")), default);
    }

    #[test]
    fn the_same_room_is_always_the_same_size() {
        assert_eq!(size(3, 11, Some("street")), size(3, 11, Some("street")));
        assert_eq!(
            own_box(3, 11, Some("street")),
            Box {
                x: 0,
                y: 0,
                z: 0,
                width: size(3, 11, Some("street")).0,
                depth: size(3, 11, Some("street")).1,
            }
        );
    }

    #[test]
    fn a_side_is_one_of_the_four_and_the_same_on_every_read() {
        for way in 1..20 {
            let side = side(5, way);
            assert!(WALLS.contains(&side));
            assert_eq!(side, super::side(5, way));
        }
    }

    #[test]
    fn the_cell_beside_a_side_is_its_middle_the_western_or_northern_of_two() {
        let room = Box {
            x: 0,
            y: 0,
            z: 0,
            width: 6,
            depth: 5,
        };
        assert_eq!(beside(&room, NORTH), Spot { x: 2, y: 0 });
        assert_eq!(beside(&room, SOUTH), Spot { x: 2, y: 4 });
        assert_eq!(beside(&room, WEST), Spot { x: 0, y: 2 });
        assert_eq!(beside(&room, EAST), Spot { x: 5, y: 2 });
    }

    #[test]
    fn a_room_that_stands_on_its_own_is_on_no_storey_of_any_place() {
        let row = |value: serde_json::Value| value.as_object().unwrap().clone();
        let alone = row(serde_json::json!({ "x": 0, "y": 0, "z": 0, "width": 8, "depth": 8 }));
        let upstairs = row(serde_json::json!({
            "parent_location_id": 3, "x": 0, "y": 0, "z": 1, "width": 4, "depth": 4
        }));
        assert!(stands_alone(&alone) && !wants_one(&alone));
        assert_eq!(crate::plan::storey(&alone), None);
        assert_eq!(crate::plan::storey(&upstairs), Some(1));
        assert_eq!(
            crate::physics::storeys(crate::plan::storey(&upstairs), crate::plan::storey(&alone)),
            None,
            "no fall is measured between two planes"
        );
    }
}
