//! The doctor's warning for a place with no layout: every place of the three
//! handmade worlds has none yet, and a world laid out by hand is warned only
//! about the places outside its buildings.

use renderedstep_engine::engine::Engine;
use renderedstep_engine::parity::open_world;
use renderedstep_engine::reach::ROOM_WITHOUT_A_LAYOUT;
use renderedstep_engine::records::{id, int};
use std::path::Path;

fn open(world: &str) -> Engine {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("parity/worlds/{world}.sql")),
    )
    .expect("the world fixture");
    open_world(&sql).unwrap()
}

/// Per story of a world: how many places it has and how many the doctor
/// warns have no layout.
fn counts(engine: &Engine) -> Vec<(usize, usize)> {
    let records = engine.store().load().unwrap();
    records
        .select("stories", |_| true)
        .into_iter()
        .map(|story| {
            let places = records
                .select("locations", |row| int(row, "story_id") == Some(id(story)))
                .len();
            let found = engine.layout_findings(id(story)).unwrap();
            assert!(found
                .iter()
                .all(|finding| finding.code == ROOM_WITHOUT_A_LAYOUT
                    && finding.severity == "warning"));
            (places, found.len())
        })
        .collect()
}

#[test]
fn every_place_of_the_three_handmade_worlds_is_without_a_layout() {
    for world in [
        "the-salt-assizes",
        "the-lunar-cartographer",
        "the-unrecorded-hour",
    ] {
        let counted = counts(&open(world));
        println!("{world}: {counted:?}");
        assert!(!counted.is_empty(), "{world}");
        for (places, warned) in counted {
            assert!(places > 0, "{world}");
            assert_eq!(warned, places, "{world}");
        }
    }
}

#[test]
fn a_world_laid_out_by_hand_is_warned_only_about_what_is_outside() {
    for world in [
        "the-pottery-on-mill-lane",
        "the-quay-house",
        "the-landing-stair",
    ] {
        let counted = counts(&open(world));
        println!("{world}: {counted:?}");
        for (places, warned) in counted {
            assert!(warned < places, "{world}");
        }
    }
}
