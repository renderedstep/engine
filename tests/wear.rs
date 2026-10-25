//! Where a thing is worn and whether it is: the doctor finds every world
//! the sweep walks clean, a game's copy of a worn thing brings both columns
//! along, the doctor names what is out of line, and nothing a model reads
//! moves when somebody is dressed.

use renderedstep_engine::dialogue;
use renderedstep_engine::engine::Engine;
use renderedstep_engine::narration;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::{Game, Mode};
use renderedstep_engine::records::{flag, id, int, text, Records};
use renderedstep_engine::wear;
use std::path::Path;

fn world_sql(world: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("parity/worlds/{world}.sql")),
    )
    .expect("the world fixture")
}

fn stories(engine: &Engine) -> Vec<i64> {
    let records = engine.store().load().unwrap();
    records
        .select("stories", |_| true)
        .into_iter()
        .map(id)
        .collect()
}

/// The Salt Assizes with its people dressed by hand: Coraith Vell in an
/// oilskin coat over court black, Ammon Brace (in the room the game starts
/// in) in a brass-badged cap, and `more` after them.
fn salt_assizes(dressed: bool, more: &str) -> Engine {
    let row = |id: i64, name: &str, character: i64, place: &str| {
        let (wear, worn) = if dressed {
            (format!("'{place}'"), 1)
        } else {
            ("NULL".to_string(), 0)
        };
        format!(
            "INSERT INTO items (id, name, character_id, wear, worn, created_at, updated_at) \
             VALUES ({id}, '{name}', {character}, {wear}, {worn}, '2026-10-25 00:00:00', \
             '2026-10-25 00:00:00');\n"
        )
    };
    let sql = format!(
        "{}{}{}{}{more}",
        world_sql("the-salt-assizes"),
        row(1000000010, "oilskin coat", 1000000001, "outer"),
        row(1000000011, "court black", 1000000001, "body"),
        row(1000000012, "brass-badged cap", 1000000002, "head"),
    );
    open_world(&sql).unwrap()
}

fn start(engine: &mut Engine, mode: Mode) -> i64 {
    let story = engine
        .story_titled(&format!("The Salt Assizes{TITLE_SUFFIX}"))
        .unwrap();
    engine.start_in(story, mode).unwrap()
}

/// What a narrator, a character's own context and the character pass are
/// handed for a game standing in its opening room.
fn model_reads(records: &Records, playthrough: i64) -> Vec<String> {
    let game = Game::new(records, playthrough);
    let mut read = vec![narration::prompt(
        Game::new(records, playthrough),
        "look around",
        None,
        None,
        None,
    )];
    for person in game.cast_in(game.current_location()) {
        if flag(person, "is_protagonist") {
            continue;
        }
        read.push(dialogue::character_request(&game, person, "Good morning.").to_string());
    }
    read
}

#[test]
fn the_doctor_finds_every_world_the_sweep_walks_clean() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds");
    let mut worlds: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            let stem = path.file_stem()?.to_string_lossy().into_owned();
            (path.extension()? == "sql").then_some(stem)
        })
        .collect();
    worlds.sort();
    for handmade in [
        "the-salt-assizes",
        "the-lunar-cartographer",
        "the-unrecorded-hour",
    ] {
        assert!(worlds.iter().any(|world| world == handmade), "{handmade}");
    }
    for world in &worlds {
        let engine = open_world(&world_sql(world)).unwrap();
        for story in stories(&engine) {
            assert_eq!(
                engine.wear_findings(story).unwrap(),
                Vec::new(),
                "{world}: story #{story}"
            );
        }
    }
}

#[test]
fn a_game_copies_what_its_people_wear_and_the_doctor_finds_nothing() {
    let mut engine = salt_assizes(true, "");
    let playthrough = start(&mut engine, Mode::Narrated);
    let records = engine.store().load().unwrap();
    let copy = |template: i64| {
        records
            .first("items", |item| {
                int(item, "playthrough_id") == Some(playthrough)
                    && int(item, "template_id") == Some(template)
            })
            .unwrap_or_else(|| panic!("no copy of #{template}"))
    };
    for (template, place, holder) in [
        (1000000010, "outer", None),
        (1000000011, "body", None),
        (1000000012, "head", Some(1000000002)),
    ] {
        let row = copy(template);
        assert_eq!(text(row, "wear"), Some(place));
        assert!(wear::worn(row));
        assert_eq!(int(row, "character_id"), holder);
        assert_eq!(int(row, "location_id"), None);
    }
    for story in stories(&engine) {
        assert_eq!(engine.wear_findings(story).unwrap(), Vec::new());
    }
}

#[test]
fn the_doctor_names_a_worn_record_out_of_line() {
    let more = "INSERT INTO items (id, name, character_id, wear, worn, created_at, updated_at) \
                VALUES (1000000013, 'tricorn', 1000000002, 'head', 1, '2026-10-25 00:00:00', \
                '2026-10-25 00:00:00');\n\
                INSERT INTO items (id, name, location_id, wear, worn, created_at, updated_at) \
                VALUES (1000000014, 'apron', 1000000001, 'waist', 1, '2026-10-25 00:00:00', \
                '2026-10-25 00:00:00');\n\
                UPDATE items SET worn = 1 WHERE id = 1000000003;\n\
                UPDATE items SET wear = 'tail' WHERE id = 1000000002;\n";
    let engine = salt_assizes(true, more);
    let story = stories(&engine)[0];
    let found: Vec<(&str, i64)> = engine
        .wear_findings(story)
        .unwrap()
        .iter()
        .map(|finding| (finding.code, finding.subject.1))
        .collect();
    assert_eq!(
        found,
        [
            (wear::NO_SUCH_PLACE, 1000000002),
            (wear::UNWEARABLE, 1000000003),
            (wear::PLACE_TAKEN, 1000000013),
            (wear::NOT_HELD, 1000000014),
        ]
    );
}

#[test]
fn nothing_a_model_reads_moves_when_somebody_is_dressed() {
    for mode in [Mode::Narrated, Mode::PlayerNarrates] {
        let mut read = Vec::new();
        for dressed in [false, true] {
            let mut engine = salt_assizes(dressed, "");
            let playthrough = start(&mut engine, mode);
            read.push(model_reads(&engine.store().load().unwrap(), playthrough));
        }
        assert!(read[0].len() > 1, "nobody to speak to");
        assert_eq!(read[0], read[1]);
    }
}
