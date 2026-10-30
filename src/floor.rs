//! A floor: a place its world names a room count for (a `floors` row),
//! laid out by the engine as exactly that many rooms on one storey, with
//! one of them, never the entry, rolled as the way down. Like any place it
//! is laid out once, on the turn somebody first walks into it, and is world
//! rows from then on: no later turn of any game lays it out again.
//!
//! Every draw comes from the place's own generator (`Roll::INTERIOR`), in
//! this order: the grid's column widths, row depths and serpentine, each
//! room's danger and hazard as it is created, the way-down room, then the
//! extra doors, on a guarded floor with no way down the room its guards
//! stand in, and last its supplies: the safe rooms, then the rooms a
//! healing thing is left in. A floor with none of them draws nothing more,
//! so it is laid out exactly as it was before supplies. A place with no
//! `floors` row is laid out by `interior`, exactly as before.

use crate::interior::{Builder, Layout, Place, WALKING};
use crate::kind;
use crate::parameters::Parameters;
use crate::random::Random;
use crate::roll;

/// The paces a side of a floor's room is given on average when the floor's
/// footprint is sized from its room count.
pub const ROOM_SIDE: i64 = 6;

/// The most rooms a floor is laid out with.
pub const MOST_ROOMS: i64 = 1000;

/// What a floor's row says about its layout.
#[derive(Clone, Copy, Debug, Default)]
pub struct Floor<'a> {
    /// How many rooms the floor is drawn with (`floors.rooms`).
    pub rooms: i64,
    /// `Location::Parameters`' danger word every room's danger is drawn from;
    /// none reads as the table's own fallback, as a missing pick does.
    pub danger: Option<&'a str>,
    /// `Location::Parameters`' hazard word; none is no hazard.
    pub hazard: Option<&'a str>,
    /// Whether the floor has a way down (`floors.down_location_id`).
    pub down: bool,
    /// Whether anybody is posted to the floor (`characters.posted_to_id`).
    pub guarded: bool,
    /// How many rooms are made safe (`floors.safe_rooms`).
    pub safe_rooms: i64,
    /// How many healing things are left about it (`floors.healing_count`).
    pub healing: i64,
}

/// A floor's rooms and doorways, and which room the way down lands on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Laid {
    pub layout: Layout,
    /// The index of the way-down room in `layout.rooms`; never 0, the entry.
    pub way_down: Option<usize>,
    /// The index of the room a guarded floor's guards stand in: the way-down
    /// room, or with no way down a room rolled after every other draw, never
    /// the entry unless it is the only room.
    pub guard_room: Option<usize>,
    /// The indices of the safe rooms, in the order they were drawn: never
    /// the entry, the way-down room or the guards' room. Each is laid out
    /// `safe` with no hazard.
    pub safe: Vec<usize>,
    /// The indices of the rooms a healing thing is left in, one each, in
    /// the order they were drawn: never the entry.
    pub healing: Vec<usize>,
}

/// Up to `count` of `candidates`, each drawn once, in the order drawn. A
/// count of 0 throws no die.
fn draw(mut candidates: Vec<usize>, count: i64, rng: &mut Random) -> Vec<usize> {
    let count = count.clamp(0, candidates.len() as i64) as usize;
    (0..count)
        .map(|_| candidates.remove(roll::one_of(candidates.len(), rng)))
        .collect()
}

/// How many rooms a floor is drawn with: its own count, at least one, and
/// at least two when it has a way down, which is never the entry.
pub fn room_count(floor: &Floor) -> usize {
    let fewest = if floor.down { 2 } else { 1 };
    floor.rooms.clamp(fewest, MOST_ROOMS) as usize
}

/// `[columns, rows]` of the grid a floor of `rooms` rooms is cut into: as
/// square as it goes, the last line short when the count is not a product.
pub fn grid(rooms: usize) -> (i64, i64) {
    let rooms = rooms.max(1) as i64;
    let mut columns = 1;
    while columns * columns < rooms {
        columns += 1;
    }
    (columns, (rooms + columns - 1) / columns)
}

/// The footprint `[width, depth]` a floor of `rooms` rooms is sized at:
/// `ROOM_SIDE` paces a side for every column and row of its grid. The world
/// loader writes it, so the floor is a place before it is laid out.
pub fn footprint(rooms: usize) -> (i64, i64) {
    let (columns, rows) = grid(rooms);
    (columns * ROOM_SIDE, rows * ROOM_SIDE)
}

/// Lays a floor out. `place.footprint` is kept when each side holds the
/// grid at the smallest side a room may have, and is `footprint` otherwise;
/// `place.kind` deals the rooms their sorts. The picks a model gave the
/// place are never read: the floor's own danger and hazard are the
/// parameters, the same throughout.
pub fn lay_out(place: &Place, floor: &Floor) -> Laid {
    let count = room_count(floor);
    let (columns, rows) = grid(count);
    let (width, depth) = place
        .footprint
        .filter(|(width, depth)| {
            *width >= columns * crate::interior::MINIMUM_SIDE
                && *depth >= rows * crate::interior::MINIMUM_SIDE
        })
        .unwrap_or_else(|| footprint(count));
    let parameters = Parameters {
        danger: floor.danger.map(str::to_string),
        hazard: floor.hazard.map(str::to_string),
        ..Parameters::default()
    };
    let mut builder = Builder::new(place);
    let mut boxes = builder.grid(width, depth, 0, columns, rows);
    boxes.truncate(count);
    let rooms: Vec<usize> = boxes
        .into_iter()
        .map(|bounds| builder.create_room(place, bounds, Some(&parameters)))
        .collect();
    for pair in rooms.windows(2) {
        builder.connect(pair[0], pair[1], WALKING);
    }
    let way_down = (floor.down && count > 1).then(|| 1 + roll::one_of(count - 1, &mut builder.rng));
    builder.way_down = way_down;
    builder.open_extra_doors(&rooms);
    builder.close_connectivity();
    let storeys = vec![0; builder.rooms.len()];
    for (room, dealt) in builder
        .rooms
        .iter_mut()
        .zip(kind::deal(place.kind, &storeys))
    {
        room.kind = dealt;
    }
    let guard_room = floor.guarded.then(|| match way_down {
        Some(room) => room,
        None if count > 1 => 1 + roll::one_of(count - 1, &mut builder.rng),
        None => 0,
    });
    let safe = draw(
        (1..count)
            .filter(|room| Some(*room) != way_down && Some(*room) != guard_room)
            .collect(),
        floor.safe_rooms,
        &mut builder.rng,
    );
    for &room in &safe {
        builder.rooms[room].danger = "safe";
        builder.rooms[room].hazard = None;
    }
    let healing = draw((1..count).collect(), floor.healing, &mut builder.rng);
    Laid {
        layout: Layout {
            footprint: (width, depth),
            rooms: builder.rooms,
            edges: builder.edges,
        },
        way_down,
        guard_room,
        safe,
        healing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interior::{self, MAX_EXITS};

    fn place(id: i64, kind: Option<&str>) -> Place<'_> {
        Place {
            story_id: 7,
            id,
            name: "The First Floor",
            clock: 0,
            existing_locations: 3,
            footprint: None,
            kind,
            density: None,
        }
    }

    fn exits(layout: &Layout, room: usize) -> usize {
        layout.edges.iter().filter(|edge| edge.from == room).count()
    }

    fn reachable(layout: &Layout) -> Vec<usize> {
        let mut seen = vec![0];
        let mut next = 0;
        while next < seen.len() {
            let room = seen[next];
            next += 1;
            for edge in layout.edges.iter().filter(|edge| edge.from == room) {
                if !seen.contains(&edge.to) {
                    seen.push(edge.to);
                }
            }
        }
        seen
    }

    /// Every count from one room to two hundred, and some larger, each over
    /// many places: exactly the count, every room reachable from the entry,
    /// the way down never the entry, every doorway both ways, and no room
    /// past the exit cap, the entry and the way-down room each with a
    /// doorway to spare.
    #[test]
    fn a_floor_is_laid_out_to_its_count_and_holds_every_rule() {
        let counts: Vec<i64> = (1..=200).chain([256, 333, 500, 1000]).collect();
        for rooms in counts {
            let places = if rooms > 200 { 2 } else { 12 };
            for id in 1..=places {
                for down in [false, true] {
                    let floor = Floor {
                        rooms,
                        danger: Some("dangerous"),
                        hazard: Some("unlit"),
                        down,
                        guarded: id % 2 == 0,
                        safe_rooms: rooms / 20,
                        healing: rooms / 16,
                    };
                    let laid = lay_out(&place(id * 1009, None), &floor);
                    let layout = &laid.layout;
                    let what = format!("{rooms} rooms, place {}, down {down}", id * 1009);
                    match laid.guard_room {
                        Some(room) => {
                            assert!(floor.guarded, "{what}");
                            assert!(room < layout.rooms.len(), "{what}");
                            assert!(room != 0 || layout.rooms.len() == 1, "{what}");
                            if down {
                                assert_eq!(Some(room), laid.way_down, "{what}");
                            }
                        }
                        None => assert!(!floor.guarded, "{what}"),
                    }
                    let count = layout.rooms.len();
                    let kept_back = 1 + [laid.way_down, laid.guard_room]
                        .iter()
                        .flatten()
                        .filter(|room| **room != 0)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    assert_eq!(
                        laid.safe.len(),
                        (floor.safe_rooms as usize).min(count.saturating_sub(kept_back)),
                        "{what}"
                    );
                    assert_eq!(
                        laid.healing.len(),
                        (floor.healing as usize).min(count - 1),
                        "{what}"
                    );
                    for (drawn, rooms) in [(&laid.safe, "safe"), (&laid.healing, "healing")] {
                        let mut distinct = drawn.clone();
                        distinct.sort_unstable();
                        distinct.dedup();
                        assert_eq!(distinct.len(), drawn.len(), "{what}: {rooms}");
                        assert!(!drawn.contains(&0), "{what}: {rooms} in the entry");
                    }
                    for room in &laid.safe {
                        assert_ne!(Some(*room), laid.way_down, "{what}");
                        assert_ne!(Some(*room), laid.guard_room, "{what}");
                        assert_eq!(layout.rooms[*room].danger, "safe", "{what}");
                        assert_eq!(layout.rooms[*room].hazard, None, "{what}");
                    }
                    assert_eq!(layout.rooms.len(), room_count(&floor), "{what}");
                    let mut reached = reachable(layout);
                    reached.sort_unstable();
                    assert_eq!(
                        reached,
                        (0..layout.rooms.len()).collect::<Vec<_>>(),
                        "{what}"
                    );
                    match laid.way_down {
                        Some(room) => {
                            assert!(down, "{what}");
                            assert_ne!(room, 0, "{what}");
                            assert!(room < layout.rooms.len(), "{what}");
                            assert!(exits(layout, room) < MAX_EXITS, "{what}");
                        }
                        None => assert!(!down, "{what}"),
                    }
                    assert!(exits(layout, 0) < MAX_EXITS, "{what}");
                    for room in 0..layout.rooms.len() {
                        assert!(exits(layout, room) <= MAX_EXITS, "{what}");
                    }
                    for edge in &layout.edges {
                        assert!(
                            layout
                                .edges
                                .iter()
                                .any(|back| back.from == edge.to && back.to == edge.from),
                            "{what}"
                        );
                        assert!(
                            layout.rooms[edge.from]
                                .bounds
                                .shares_a_wall(&layout.rooms[edge.to].bounds),
                            "{what}"
                        );
                    }
                    for room in &layout.rooms {
                        assert_eq!(room.bounds.z, 0, "{what}");
                        assert!(room.bounds.width >= interior::MINIMUM_SIDE, "{what}");
                        assert!(room.bounds.depth >= interior::MINIMUM_SIDE, "{what}");
                    }
                }
            }
        }
    }

    #[test]
    fn a_floor_with_a_way_down_and_too_few_rooms_is_drawn_with_two() {
        let floor = Floor {
            rooms: 1,
            down: true,
            ..Floor::default()
        };
        let laid = lay_out(&place(11, None), &floor);
        assert_eq!(laid.layout.rooms.len(), 2);
        assert_eq!(laid.way_down, Some(1));
        let alone = lay_out(
            &place(11, None),
            &Floor {
                rooms: 0,
                ..Floor::default()
            },
        );
        assert_eq!(alone.layout.rooms.len(), 1);
        assert!(alone.layout.edges.is_empty());
        assert_eq!(alone.way_down, None);
        let huge = Floor {
            rooms: 5000,
            ..Floor::default()
        };
        assert_eq!(room_count(&huge), MOST_ROOMS as usize);
    }

    #[test]
    fn the_same_floor_is_laid_out_the_same_every_time() {
        let floor = Floor {
            rooms: 40,
            danger: Some("uneasy"),
            hazard: Some("flooded"),
            down: true,
            guarded: false,
            safe_rooms: 2,
            healing: 3,
        };
        assert_eq!(
            lay_out(&place(42, Some("warren")), &floor),
            lay_out(&place(42, Some("warren")), &floor)
        );
        assert_ne!(
            lay_out(&place(42, None), &floor).way_down,
            lay_out(&place(43, None), &floor).way_down,
            "another place rolls another way down"
        );
    }

    /// Guards are the last draw: a guarded floor's rooms, doorways and way
    /// down are the unguarded floor's, and its guards stand on the way down,
    /// or with none in a room rolled off the entry.
    #[test]
    fn guards_stand_on_the_way_down_and_move_no_other_draw() {
        for down in [false, true] {
            for id in 1..=40 {
                let floor = Floor {
                    rooms: 9,
                    danger: Some("uneasy"),
                    hazard: None,
                    down,
                    guarded: false,
                    ..Floor::default()
                };
                let open = lay_out(&place(id, Some("warren")), &floor);
                let guarded = lay_out(
                    &place(id, Some("warren")),
                    &Floor {
                        guarded: true,
                        ..floor
                    },
                );
                assert_eq!(open.guard_room, None);
                assert_eq!(guarded.layout, open.layout);
                assert_eq!(guarded.way_down, open.way_down);
                let room = guarded
                    .guard_room
                    .expect("a guarded floor has a guard room");
                assert_ne!(room, 0, "place {id}");
                if down {
                    assert_eq!(Some(room), open.way_down, "place {id}");
                }
            }
        }
        let alone = lay_out(
            &place(5, None),
            &Floor {
                rooms: 1,
                guarded: true,
                ..Floor::default()
            },
        );
        assert_eq!(
            alone.guard_room,
            Some(0),
            "one room is the only room to stand in"
        );
    }

    /// Supplies are the last draws: a stocked floor's rooms, doorways, way
    /// down and guards' room are the bare floor's, but for each safe room's
    /// danger and hazard; and a floor with none draws nothing more.
    #[test]
    fn supplies_are_drawn_last_and_move_no_other_draw() {
        for down in [false, true] {
            for id in 1..=40 {
                let bare = Floor {
                    rooms: 12,
                    danger: Some("dangerous"),
                    hazard: Some("unlit"),
                    down,
                    guarded: id % 3 == 0,
                    ..Floor::default()
                };
                let stocked = Floor {
                    safe_rooms: 2,
                    healing: 3,
                    ..bare
                };
                let open = lay_out(&place(id, Some("warren")), &bare);
                let laid = lay_out(&place(id, Some("warren")), &stocked);
                assert!(open.safe.is_empty() && open.healing.is_empty());
                assert_eq!(laid.way_down, open.way_down, "place {id}");
                assert_eq!(laid.guard_room, open.guard_room, "place {id}");
                assert_eq!(laid.layout.edges, open.layout.edges, "place {id}");
                assert_eq!(laid.safe.len(), 2, "place {id}");
                assert_eq!(laid.healing.len(), 3, "place {id}");
                for (index, (room, was)) in
                    laid.layout.rooms.iter().zip(&open.layout.rooms).enumerate()
                {
                    if laid.safe.contains(&index) {
                        assert_eq!(room.danger, "safe");
                        assert_eq!(room.hazard, None);
                        assert_eq!((&room.name, room.bounds), (&was.name, was.bounds));
                    } else {
                        assert_eq!(room, was, "place {id}, room {index}");
                    }
                }
                assert_eq!(laid, lay_out(&place(id, Some("warren")), &stocked));
            }
        }
        let two = lay_out(
            &place(5, None),
            &Floor {
                rooms: 2,
                down: true,
                safe_rooms: 4,
                healing: 4,
                ..Floor::default()
            },
        );
        assert!(two.safe.is_empty(), "the only other room is the way down");
        assert_eq!(two.healing, vec![1], "one room besides the entry");
    }

    #[test]
    fn the_grid_is_as_square_as_it_goes_and_the_footprint_holds_it() {
        assert_eq!(grid(1), (1, 1));
        assert_eq!(grid(2), (2, 1));
        assert_eq!(grid(5), (3, 2));
        assert_eq!(grid(9), (3, 3));
        assert_eq!(grid(10), (4, 3));
        assert_eq!(grid(160), (13, 13));
        assert_eq!(footprint(160), (78, 78));
        assert_eq!(footprint(8), (18, 18));
        let floor = Floor {
            rooms: 8,
            ..Floor::default()
        };
        let mut kept = place(5, None);
        kept.footprint = Some((30, 24));
        assert_eq!(lay_out(&kept, &floor).layout.footprint, (30, 24));
        kept.footprint = Some((5, 5));
        assert_eq!(
            lay_out(&kept, &floor).layout.footprint,
            (18, 18),
            "a footprint too small for the grid is sized from the count"
        );
    }

    #[test]
    fn every_room_s_danger_and_hazard_come_from_the_floor_s_own_words() {
        let floor = Floor {
            rooms: 160,
            danger: Some("safe"),
            hazard: None,
            down: true,
            guarded: false,
            ..Floor::default()
        };
        let laid = lay_out(&place(9, None), &floor);
        let safe = Parameters::default().danger_for(0);
        for room in &laid.layout.rooms {
            assert!(safe.contains(&room.danger), "{}", room.danger);
            assert_eq!(room.hazard, None);
        }
        let hazardous = Floor {
            hazard: Some("airless"),
            ..floor
        };
        let laid = lay_out(&place(9, None), &hazardous);
        let rooms = &laid.layout.rooms;
        assert!(rooms.iter().any(|room| room.hazard.is_some()));
        assert!(rooms
            .iter()
            .filter_map(|room| room.hazard)
            .all(|hazard| hazard.hazard == "airless"));
    }

    #[test]
    fn a_floor_s_kind_deals_its_rooms_their_sorts() {
        let floor = Floor {
            rooms: 6,
            ..Floor::default()
        };
        let laid = lay_out(&place(3, Some("warren")), &floor);
        assert!(laid
            .layout
            .rooms
            .iter()
            .all(|room| room.kind == Some("cave")));
        let none = lay_out(&place(3, None), &floor);
        assert!(none.layout.rooms.iter().all(|room| room.kind.is_none()));
    }
}
