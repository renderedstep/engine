//! Where the engine puts what an overdue quest step needs: the anchor search
//! of `Quest::Deadline`, over a story's rooms and doorways given as values.
//!
//! What a step needs is grown in ground nobody has written yet, never bolted
//! onto a room the world has already written (`way_on`): the engine does not
//! change the world after it is generated, it only takes the world into
//! account as it generates more.

use std::collections::{BTreeMap, VecDeque};

pub const GRACE_ROOMS: i64 = 3;
/// `Location::ExitsSchema::MAX_EXITS`: a room with this many doorways out
/// takes no more.
pub const MAX_EXITS: usize = 4;

/// One room of the story, as the search reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Room {
    pub id: i64,
    pub realized: bool,
    /// The storey, or `None` for a room with no position (read as 0).
    pub z: Option<i64>,
    /// A place with a footprint whose rooms are already laid out.
    pub laid_out: bool,
    /// A place with a footprint and no box of its own, laid out into rooms
    /// already or on its first entry: nobody stands in it, and a doorway to
    /// it is moved onto one of its rooms when it is laid out.
    pub place: bool,
}

/// `Quest::Deadline#hops`: a breadth-first walk from the lowest-id realized
/// room over doorways taken both ways, neighbours in id order. Empty when no
/// room is realized.
pub fn hops(rooms: &[Room], connections: &[(i64, i64)]) -> BTreeMap<i64, i64> {
    let Some(root) = rooms.iter().filter(|r| r.realized).map(|r| r.id).min() else {
        return BTreeMap::new();
    };
    let mut adjacency: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for &(from, to) in connections {
        adjacency.entry(from).or_default().push(to);
        adjacency.entry(to).or_default().push(from);
    }
    for neighbours in adjacency.values_mut() {
        neighbours.sort_unstable();
        neighbours.dedup();
    }
    let mut seen = BTreeMap::from([(root, 0)]);
    let mut queue = VecDeque::from([root]);
    while let Some(id) = queue.pop_front() {
        let here = seen[&id];
        for &neighbour in adjacency.get(&id).map_or(&[][..], Vec::as_slice) {
            seen.entry(neighbour).or_insert_with(|| {
                queue.push_back(neighbour);
                here + 1
            });
        }
    }
    seen
}

/// `Quest::Deadline#anchor`: among reached rooms that are not laid out and
/// have fewer than `MAX_EXITS` doorways out, the lowest storey, then the
/// farthest, then the lowest id.
///
/// This is the game's rule as its vector still states it; the engine no
/// longer grows anything off the room it names, which may be one the world
/// has already written (`way_on`).
pub fn anchor(rooms: &[Room], connections: &[(i64, i64)]) -> Option<i64> {
    deepest(rooms, connections, |room| !room.laid_out)
}

/// Where an overdue step's target is grown from, as the room `written`
/// finishes its writing: a way on the world already has into ground nobody
/// has written, so the target is generated there rather than bolted onto a
/// room already written.
///
/// Among reached rooms nobody has written yet that are not places and have
/// fewer than `MAX_EXITS` doorways out, the lowest storey, then the
/// farthest, then the lowest id. With none, `written` itself, the next room
/// written: its writing is finishing in this same turn and nobody has stood
/// in it written yet, so what is grown off it is part of that writing. It
/// too must be reached, not a place, and have a doorway to spare. None when
/// neither will take it: the step waits for the next writing.
pub fn way_on(rooms: &[Room], connections: &[(i64, i64)], written: i64) -> Option<i64> {
    deepest(rooms, connections, |room| !room.realized && !room.place)
        .or_else(|| deepest(rooms, connections, |room| room.id == written && !room.place))
}

/// Among reached rooms that `fits` and have fewer than `MAX_EXITS` doorways
/// out, the least `[z (None as 0), -hops, id]`.
fn deepest(
    rooms: &[Room],
    connections: &[(i64, i64)],
    fits: impl Fn(&Room) -> bool,
) -> Option<i64> {
    let hops = hops(rooms, connections);
    let exits = |id: i64| connections.iter().filter(|(from, _)| *from == id).count();
    rooms
        .iter()
        .filter(|room| hops.contains_key(&room.id))
        .filter(|room| fits(room) && exits(room.id) < MAX_EXITS)
        .min_by_key(|room| (room.z.unwrap_or(0), -hops[&room.id], room.id))
        .map(|room| room.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn room(id: i64, realized: bool) -> Room {
        Room {
            id,
            realized,
            z: None,
            laid_out: false,
            place: false,
        }
    }

    fn both(pairs: &[(i64, i64)]) -> Vec<(i64, i64)> {
        pairs.iter().flat_map(|&(a, b)| [(a, b), (b, a)]).collect()
    }

    /// The yard, its kitchen and the loft over it written; the cold store
    /// just written; the wash house and the byre off the yard still stubs.
    fn the_yard() -> (Vec<Room>, Vec<(i64, i64)>) {
        let rooms = vec![
            room(1, true),
            room(2, true),
            room(3, true),
            room(4, true),
            room(5, false),
            room(6, false),
        ];
        (rooms, both(&[(1, 2), (2, 3), (1, 4), (1, 5), (1, 6)]))
    }

    #[test]
    fn the_old_anchor_is_a_room_already_written() {
        let (rooms, connections) = the_yard();
        assert_eq!(anchor(&rooms, &connections), Some(3));
    }

    #[test]
    fn a_way_on_is_a_room_nobody_has_written() {
        let (rooms, connections) = the_yard();
        assert_eq!(way_on(&rooms, &connections, 4), Some(5));
    }

    #[test]
    fn the_deepest_unwritten_room_is_the_way_on() {
        let (mut rooms, mut connections) = the_yard();
        rooms.push(Room {
            z: Some(-1),
            ..room(7, false)
        });
        connections.extend(both(&[(6, 7)]));
        assert_eq!(way_on(&rooms, &connections, 4), Some(7));
    }

    #[test]
    fn a_place_is_never_the_way_on() {
        let (mut rooms, connections) = the_yard();
        for room in rooms.iter_mut().filter(|room| room.id >= 5) {
            room.place = true;
        }
        assert_eq!(way_on(&rooms, &connections, 4), Some(4));
    }

    #[test]
    fn with_nothing_unwritten_the_room_being_written_is_the_way_on() {
        let (mut rooms, connections) = the_yard();
        for room in &mut rooms {
            room.realized = true;
        }
        assert_eq!(way_on(&rooms, &connections, 4), Some(4));
        assert_eq!(way_on(&rooms, &connections, 2), Some(2));
    }

    #[test]
    fn an_unwritten_room_with_no_doorway_to_spare_is_passed_over() {
        let (mut rooms, mut connections) = the_yard();
        rooms.extend((8..=10).map(|id| room(id, true)));
        connections.extend((8..=10).map(|id| (5, id)));
        assert_eq!(way_on(&rooms, &connections, 4), Some(6));
    }

    #[test]
    fn nothing_takes_it_when_the_room_being_written_is_full_and_nothing_is_unwritten() {
        let (mut rooms, mut connections) = the_yard();
        for room in &mut rooms {
            room.realized = true;
        }
        rooms.extend((8..=10).map(|id| room(id, true)));
        connections.extend(both(&[(4, 8), (4, 9), (4, 10)]));
        assert_eq!(way_on(&rooms, &connections, 4), None);
    }
}
