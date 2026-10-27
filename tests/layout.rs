//! The doctor's warning for a place with no layout: the three handmade
//! worlds are laid out as they load, by the rule a generated world's rooms
//! are, and a world laid out by hand is warned only about the places
//! outside its buildings. And laying out the rooms that
//! stand on their own (`Engine::lay_out_rooms`, what the game runs after it
//! pulls): once it has run, no world the sweep plays is warned about any
//! room at all, and running it again does nothing.

use renderedstep_engine::boxes::{Box, Spot};
use renderedstep_engine::engine::Engine;
use renderedstep_engine::layout;
use renderedstep_engine::parity::open_world;
use renderedstep_engine::plan::box_of;
use renderedstep_engine::reach::ROOM_WITHOUT_A_LAYOUT;
use renderedstep_engine::records::{id, int, text, Records};
use std::path::Path;

/// The worlds the sweep plays, by file name.
fn worlds() -> Vec<String> {
    let mut worlds: Vec<String> =
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds"))
            .expect("the world fixtures")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .path()
                    .file_stem()
                    .expect("a name")
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
    worlds.sort();
    worlds
}

fn stories(engine: &Engine) -> Vec<i64> {
    let records = engine.store().load().unwrap();
    records
        .select("stories", |_| true)
        .into_iter()
        .map(id)
        .collect()
}

/// Every row of `table` with a spot, by id.
fn spots(records: &Records, table: &str) -> Vec<(i64, Spot)> {
    records
        .table(table)
        .iter()
        .filter_map(|row| {
            Some((
                id(row),
                Spot {
                    x: int(row, "x")?,
                    y: int(row, "y")?,
                },
            ))
        })
        .collect()
}

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
fn the_three_handmade_worlds_are_laid_out_as_a_generated_world_is() {
    for world in [
        "the-salt-assizes",
        "the-lunar-cartographer",
        "the-unrecorded-hour",
    ] {
        let mut engine = open(world);
        let counted = counts(&engine);
        println!("{world}: {counted:?}");
        assert!(!counted.is_empty(), "{world}");
        for (places, warned) in counted {
            assert!(places > 0, "{world}");
            assert_eq!(warned, 0, "{world}");
        }
        // Their rooms stand on their own and name no sort, so each is
        // sized from the default band, as the loader lays it out.
        let records = engine.store().load().unwrap();
        for story in stories(&engine) {
            for room in records.select("locations", |row| int(row, "story_id") == Some(story)) {
                assert_eq!(
                    box_of(room),
                    Some(layout::own_box(story, id(room), text(room, "kind"))),
                    "{world}"
                );
            }
            assert!(
                engine.lay_out_rooms(story, true).unwrap().is_empty(),
                "{world}"
            );
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

#[test]
fn once_its_rooms_are_laid_out_no_world_is_warned_and_a_second_run_does_nothing() {
    for world in worlds() {
        let mut engine = open(&world);
        for story in stories(&engine) {
            let before = engine.store().load().unwrap();
            let warned = engine.layout_findings(story).unwrap().len();
            let dry = engine.lay_out_rooms(story, true).unwrap();
            assert_eq!(
                engine.store().load().unwrap().table("locations"),
                before.table("locations"),
                "{world}: a dry run writes nothing"
            );
            let laid = engine.lay_out_rooms(story, false).unwrap();
            assert_eq!(laid, dry, "{world}: a dry run says what a run does");
            assert_eq!(laid.len(), warned, "{world}: one room for each warning");
            assert!(engine.layout_findings(story).unwrap().is_empty(), "{world}");
            assert!(
                engine.lay_out_rooms(story, false).unwrap().is_empty(),
                "{world}: a second run finds nothing"
            );

            let after = engine.store().load().unwrap();
            for room in &laid {
                let row = after.find("locations", room.location).unwrap();
                assert_eq!(box_of(row), Some(room.r#box), "{world}");
                assert_eq!((room.r#box.x, room.r#box.y, room.r#box.z), (0, 0, 0));
                let inside = |at: &Spot| {
                    at.x >= 0 && at.y >= 0 && at.x < room.r#box.width && at.y < room.r#box.depth
                };
                assert!(
                    room.items
                        .iter()
                        .chain(&room.people)
                        .all(|(_, at)| inside(at)),
                    "{world}"
                );
            }
            // A spot is only ever added, and every spot that was there stays.
            for table in ["items", "characters"] {
                let had = spots(&before, table);
                let has = spots(&after, table);
                assert!(
                    had.iter().all(|spot| has.contains(spot)),
                    "{world}: {table}"
                );
                let added: usize = laid
                    .iter()
                    .map(|room| {
                        if table == "items" {
                            room.items.len()
                        } else {
                            room.people.len()
                        }
                    })
                    .sum();
                assert_eq!(has.len(), had.len() + added, "{world}: {table}");
            }
            println!("{world}: {} laid out", laid.len());
        }
    }
}

#[test]
fn a_room_inside_a_place_with_no_box_is_left_for_its_place() {
    let mut engine = open("the-ropewalk-at-saltmarsh");
    let story = stories(&engine)[0];
    let laid = engine.lay_out_rooms(story, false).unwrap();
    let names: Vec<&str> = laid.iter().map(|room| room.name.as_str()).collect();
    assert_eq!(names, ["Saltmarsh Green"]);
    let records = engine.store().load().unwrap();
    let footprint = records
        .first("locations", |row| {
            renderedstep_engine::records::string(row, "name") == "Coyle's Ropewalk"
        })
        .unwrap();
    assert_eq!(
        box_of(footprint),
        None::<Box>,
        "a place keeps its footprint"
    );
}
