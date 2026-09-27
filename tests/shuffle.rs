//! The story clock's world mechanic: The Lunar Cartographer's nightly
//! doorway shuffle, which no sweep script reaches.
//!
//! `tests/fixtures/shuffle_at_midnight.json` is what the Ruby engine (at the
//! commit `parity/README.md` names) wrote when a new playthrough of that
//! world's sweep copy opened a passage on doorway 1000000007, a scene two
//! hours after the story's start moved the clock past midnight, and the game
//! then played `look` with no model: every doorway, passage, world event and
//! event room afterwards, the mechanic's last run, and the step's dump.

use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::records::Row;
use serde_json::Value;
use std::path::Path;

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn picked(rows: &[Row], columns: &[&str]) -> Vec<Value> {
    rows.iter()
        .map(|row| {
            Value::Object(
                columns
                    .iter()
                    .map(|column| {
                        (
                            column.to_string(),
                            row.get(*column).cloned().unwrap_or(Value::Null),
                        )
                    })
                    .collect(),
            )
        })
        .collect()
}

#[test]
fn the_nightly_shuffle_turns_the_mobile_doorways_as_the_ruby_engine_does() {
    let expected: Value =
        serde_json::from_str(&read("tests/fixtures/shuffle_at_midnight.json")).unwrap();
    let mut engine = open_world(&read("parity/worlds/the-lunar-cartographer.sql")).unwrap();
    let story = engine
        .story_titled(&format!("The Lunar Cartographer{TITLE_SUFFIX}"))
        .unwrap();
    let game = engine.start(story).unwrap();
    let conn = engine.store().connection();
    conn.execute(
        "INSERT INTO playthrough_passages (playthrough_id, location_connection_id, means, opened_at, created_at, updated_at)
         SELECT ?1, 1000000007, 'force', start_time, start_time, start_time FROM stories WHERE id = ?2",
        [game, story],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO scenes (story_id, location_id, previous_scene_id, description, story_timestamp, engine_fallback, is_opening, created_at, updated_at)
         SELECT ?1, p.current_location_id, p.current_scene_id, 'Two hours pass.', datetime(s.start_time, '+2 hours'), 0, 0, s.start_time, s.start_time
         FROM playthroughs p, stories s WHERE p.id = ?2 AND s.id = ?1",
        [story, game],
    )
    .unwrap();

    let outcome = engine.play(game, "look", &mut |_| {}).unwrap();
    let records = engine.store().load().unwrap();
    for (table, columns) in [
        (
            "location_connections",
            &[
                "id",
                "location_id",
                "connected_location_id",
                "barrier",
                "distance",
                "travel_method",
                "time_to_travel",
                "hazard",
                "hazard_die",
                "key_template_id",
            ][..],
        ),
        (
            "playthrough_passages",
            &[
                "id",
                "location_connection_id",
                "playthrough_id",
                "means",
                "opened_at",
            ][..],
        ),
        (
            "world_events",
            &[
                "id",
                "world_mechanic_id",
                "source",
                "occurred_at",
                "summary",
            ][..],
        ),
        (
            "locations_world_events",
            &["location_id", "world_event_id"][..],
        ),
        ("world_mechanics", &["id", "last_run_at"][..]),
    ] {
        assert_eq!(
            Value::Array(picked(records.table(table), columns)),
            expected[table],
            "{table}"
        );
    }
    let exits: Vec<Value> = outcome
        .state
        .exits
        .iter()
        .map(|room| serde_json::json!({ "id": room.id, "name": room.name, "detail": room.detail }))
        .collect();
    assert_eq!(Value::Array(exits), expected["dump"]["exits"]);
}
