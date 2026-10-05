//! The guards on a moving world's shuffle: a night with fewer than two
//! doorways it may turn, or with nothing mobile at all, writes nothing; and
//! no night picks an arrangement that leaves the world in two pieces.
//!
//! A Lock in the Moving City turns its two outer doorways on the hour, and
//! its story starts five minutes before one.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::random::Random;
use renderedstep_engine::shuffle_connections::{Edge, Graph, SEED_STORY_MULTIPLIER};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Every doorway of a world, as `(from, to)`, in order.
fn doorways(engine: &Engine) -> Vec<Edge> {
    let conn = engine.store().connection();
    let mut statement = conn
        .prepare(
            "SELECT location_id, connected_location_id FROM location_connections
             ORDER BY location_id, connected_location_id",
        )
        .unwrap();
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn count(engine: &Engine, query: &str) -> i64 {
    engine
        .store()
        .connection()
        .query_row(query, [], |row| row.get(0))
        .unwrap()
}

/// The moving city with `sql` run over it, a game in it whose clock is
/// moved ten minutes on, past the hour, and one line played: how many
/// doorways lead from a mobile room to an anchored one, every doorway before
/// and after, and the world events written.
fn past_the_hour(sql: &str) -> (i64, Vec<Edge>, Vec<Edge>, i64) {
    let world = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/a-lock-in-the-moving-city.sql"),
    )
    .unwrap();
    let mut engine = open_world(&format!("{world}{sql}")).unwrap();
    let story = engine
        .story_titled(&format!("A Lock in the Moving City{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    let turnable = count(
        &engine,
        "SELECT COUNT(*) FROM location_connections c
         JOIN locations f ON f.id = c.location_id JOIN locations t ON t.id = c.connected_location_id
         WHERE f.mobile AND NOT t.mobile",
    );
    let before = doorways(&engine);
    let conn = engine.store().connection();
    conn.execute(
        "INSERT INTO scenes (story_id, location_id, previous_scene_id, description, story_timestamp, engine_fallback, is_opening, created_at, updated_at)
         SELECT ?1, p.current_location_id, p.current_scene_id, 'Ten minutes pass.', datetime(s.start_time, '+10 minutes'), 0, 0, s.start_time, s.start_time
         FROM playthroughs p, stories s WHERE p.id = ?2 AND s.id = ?1",
        [story, game],
    )
    .unwrap();
    conn.execute(
        "UPDATE playthroughs SET current_scene_id = last_insert_rowid() WHERE id = ?1",
        [game],
    )
    .unwrap();
    engine.play(game, "look", &mut |_| {}).unwrap();
    assert_eq!(
        count(
            &engine,
            "SELECT COUNT(*) FROM world_mechanics WHERE last_run_at IS NOT NULL"
        ),
        1,
        "the hour came due"
    );
    let events = count(&engine, "SELECT COUNT(*) FROM world_events");
    (turnable, before, doorways(&engine), events)
}

/// The hour turns the city's two outer doorways; with one of them gone, or
/// with every room anchored, the same hour writes nothing.
#[test]
fn a_world_with_fewer_than_two_doorways_to_turn_or_nothing_mobile_does_not_move() {
    let (turnable, before, after, events) = past_the_hour("");
    assert_eq!(turnable, 2);
    assert_ne!(before, after, "the two outer doorways turned");
    assert_eq!(events, 1);

    let one_doorway = "DELETE FROM location_connections WHERE id IN (1000000005, 1000000006);";
    let (turnable, before, after, events) = past_the_hour(one_doorway);
    assert_eq!(turnable, 1);
    assert_eq!(before, after);
    assert_eq!(events, 0);

    let (turnable, before, after, events) = past_the_hour("UPDATE locations SET mobile = 0;");
    assert_eq!(turnable, 0);
    assert_eq!(before, after);
    assert_eq!(events, 0);
}

/// Whether every room is reached from the first through `doorways`.
fn whole(rooms: &[i64], doorways: &[Edge]) -> bool {
    let mut adjacency: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for &(a, b) in doorways {
        adjacency.entry(a).or_default().push(b);
        adjacency.entry(b).or_default().push(a);
    }
    let mut reached = BTreeSet::from([rooms[0]]);
    let mut frontier = vec![rooms[0]];
    while let Some(room) = frontier.pop() {
        for &next in adjacency.get(&room).map_or(&[][..], Vec::as_slice) {
            if reached.insert(next) {
                frontier.push(next);
            }
        }
    }
    reached.len() == rooms.len()
}

/// Three mobile rooms A, B and C and three anchored ones X, Y and Z, joined
/// A-B, A-X, B-Y, C-Z and Z-Y: turning the three doorways out to X, Y and
/// Z can strand C with X. Every night for a fortnight the arrangement
/// chosen keeps the world whole, and on some of those nights the first
/// arrangement drawn would not have.
#[test]
fn a_night_that_would_split_the_world_in_two_is_refused() {
    let [a, b, c, x, y, z] = [1, 2, 3, 4, 5, 6];
    let rooms = [a, b, c, x, y, z];
    let pairs = [(a, b), (a, x), (b, y), (c, z), (z, y)];
    let graph = Graph {
        story_id: 1_000_000_001,
        locations: rooms.iter().map(|&room| (room, room <= c)).collect(),
        connections: pairs
            .iter()
            .flat_map(|&(from, to)| [(from, to), (to, from)])
            .collect(),
    };
    let edges = graph.anchor_edges();
    assert_eq!(edges, [(a, x), (b, y), (c, z)]);
    let unturned: Vec<Edge> = graph
        .connections
        .iter()
        .copied()
        .filter(|edge| !edges.contains(edge) && !edges.contains(&(edge.1, edge.0)))
        .collect();
    let turned_to = |arrangement: &[i64]| -> Vec<Edge> {
        let mut doorways = unturned.clone();
        doorways.extend(
            edges
                .iter()
                .zip(arrangement)
                .map(|(edge, &to)| (edge.0, to)),
        );
        doorways
    };

    let midnight = 1_788_220_800; // 2026-09-01 00:00 UTC
    let mut refused = 0;
    for night in 0..14 {
        let at = midnight + night * 86_400;
        let mut first: Vec<i64> = edges.iter().map(|edge| edge.1).collect();
        Random::new(i128::from(graph.story_id) * SEED_STORY_MULTIPLIER + i128::from(at))
            .shuffle(&mut first);
        if !whole(&rooms, &turned_to(&first)) {
            refused += 1;
        }
        if let Some(arrangement) = graph.choose_arrangement(&edges, at) {
            assert!(
                whole(&rooms, &turned_to(&arrangement)),
                "night {night} split the world in two: {arrangement:?}"
            );
        }
    }
    assert!(refused > 0, "no night drew a splitting arrangement first");
}
