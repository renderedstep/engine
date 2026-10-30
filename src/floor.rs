//! A floor: a place its world names a room count for (a `floors` row),
//! laid out by the engine as exactly that many rooms on one storey, with
//! one of them, never the entry, rolled as the way down. Like any place it
//! is laid out once, on the turn somebody first walks into it, and is world
//! rows from then on: no later turn of any game lays it out again.
//!
//! Every draw comes from the place's own generator (`Roll::INTERIOR`), in
//! this order: the grid's column widths, row depths and serpentine, each
//! room's danger and hazard as it is created, the way-down room, then the
//! extra doors. A place with no `floors` row is laid out by `interior`,
//! exactly as before.

use crate::interior::{Builder, Layout, Place, WALKING};
use crate::kind;
use crate::parameters::Parameters;
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
}

/// A floor's rooms and doorways, and which room the way down lands on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Laid {
    pub layout: Layout,
    /// The index of the way-down room in `layout.rooms`; never 0, the entry.
    pub way_down: Option<usize>,
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
    Laid {
        layout: Layout {
            footprint: (width, depth),
            rooms: builder.rooms,
            edges: builder.edges,
        },
        way_down,
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
                    };
                    let laid = lay_out(&place(id * 1009, None), &floor);
                    let layout = &laid.layout;
                    let what = format!("{rooms} rooms, place {}, down {down}", id * 1009);
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
