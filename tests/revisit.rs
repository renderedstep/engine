//! The arrival diff ([`revisit`]): a move keeps what the game saw of the room
//! it walked out of, and a move back compares it with the room as it stands,
//! among what the game saw, into a closed list of changes in the engine's
//! words. Played on The Furnished Rooms with people and things added to the
//! Clerk's Study, and the room changed by hand while the game is away.

use renderedstep_engine::engine::{Engine, Error, Submitted};
use renderedstep_engine::model::{Replay, Reply};
use renderedstep_engine::noticed;
use renderedstep_engine::parity::{open_world, TITLE_SUFFIX};
use renderedstep_engine::playthrough::{Game, Mode};
use renderedstep_engine::revisit::{self, Kind, Since};
use serde_json::{json, Value};
use std::path::Path;

const STUDY: i64 = 1_000_000_001;
const CLEARING: i64 = 1_000_000_002;
const STALL: i64 = 1_000_000_003;
const READING_ROOM: i64 = 1_000_000_004;
const DESK: i64 = 1_000_000_001;
/// The study's way out to the Reading Room, taken away before the game
/// starts and put back while it is away.
const TO_READING_ROOM: i64 = 1_000_000_005;
/// The study's way out to Cinder Lane Stall.
const TO_STALL: i64 = 1_000_000_003;

struct Study {
    engine: Engine,
    playthrough: i64,
    story: i64,
}

fn insert(engine: &Engine, table: &str, values: &[(&str, Value)]) -> i64 {
    let row = engine.store().insert(table, values).unwrap();
    row.get("id").and_then(Value::as_i64).unwrap_or_default()
}

/// The Furnished Rooms with three people and four more things in the
/// Clerk's Study, a folded note shut in its desk, and its way out to the Reading Room taken away, then a game
/// started in `mode`.
fn study(mode: Mode) -> Study {
    let sql = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("parity/worlds/the-furnished-rooms.sql"),
    )
    .expect("the world fixture");
    let mut engine = open_world(&sql).unwrap();
    let story = engine
        .story_titled(&format!("The Furnished Rooms{TITLE_SUFFIX}"))
        .unwrap();
    let race: i64 = engine
        .store()
        .connection()
        .query_row("SELECT id FROM races LIMIT 1", [], |row| row.get(0))
        .unwrap();
    for name in ["Tobin Reed", "Wren Hale", "Old Pell"] {
        insert(
            &engine,
            "characters",
            &[
                ("fullname", Value::from(name)),
                ("nickname", Value::from(name.split(' ').next().unwrap())),
                ("story_id", Value::from(story)),
                ("race_id", Value::from(race)),
                ("location_id", Value::from(STUDY)),
                ("level", Value::from(1)),
                ("hit_die", Value::from(8)),
            ],
        );
    }
    for (name, within) in [
        ("ledger", None),
        ("brass button", None),
        ("pewter mug", None),
        ("ink pot", Some((DESK, "on"))),
        ("folded note", Some((DESK, "in"))),
    ] {
        let mut values = vec![
            ("name", Value::from(name)),
            ("bulk", Value::from("handy")),
            ("location_id", Value::from(STUDY)),
            ("description", Value::from(format!("A {name}."))),
        ];
        if let Some((piece, how)) = within {
            values.push(("within_id", Value::from(piece)));
            values.push(("how", Value::from(how)));
        }
        insert(&engine, "items", &values);
    }
    engine
        .store()
        .connection()
        .execute(
            "DELETE FROM location_connections WHERE id = ?1",
            [TO_READING_ROOM],
        )
        .unwrap();
    let playthrough = engine.start_in(story, mode).unwrap();
    Study {
        engine,
        playthrough,
        story,
    }
}

impl Study {
    fn submit(&mut self, line: &str, token: &str, replies: Vec<Reply>) -> Submitted {
        let mut replay = Replay::new(replies);
        let submitted = self
            .engine
            .submit(self.playthrough, line, token, &mut replay, &mut |_| {})
            .unwrap();
        replay.finish().unwrap();
        submitted
    }

    fn since(&self, submitted: &Submitted) -> Option<Since> {
        self.engine
            .since_last_visit(self.playthrough, submitted.turned.scene.expect("a scene"))
            .unwrap()
    }

    /// This game's copy of a thing, by name.
    fn thing(&self, name: &str) -> i64 {
        self.engine
            .store()
            .connection()
            .query_row(
                "SELECT id FROM items WHERE playthrough_id = ?1 AND name = ?2",
                (self.playthrough, name),
                |row| row.get(0),
            )
            .unwrap_or_else(|_| panic!("no copy of {name}"))
    }

    fn person(&self, name: &str) -> i64 {
        self.engine
            .store()
            .connection()
            .query_row(
                "SELECT id FROM characters WHERE fullname = ?1",
                [name],
                |row| row.get(0),
            )
            .unwrap()
    }

    fn execute(&self, sql: &str, values: impl rusqlite::Params) {
        self.engine
            .store()
            .connection()
            .execute(sql, values)
            .unwrap();
    }

    /// Where a person is in this game: their state's room, or the world's.
    fn put(&self, person: i64, room: Option<i64>) {
        self.execute(
            "UPDATE characters SET location_id = ?1 WHERE id = ?2",
            (room, person),
        );
        self.execute(
            "UPDATE playthrough_npc_states SET location_id = ?1 \
             WHERE playthrough_id = ?2 AND character_id = ?3",
            (room, self.playthrough, person),
        );
    }

    /// Story time now, in this game.
    fn now(&self) -> i64 {
        let records = self.engine.store().load().unwrap();
        Game::new(&records, self.playthrough).story_now()
    }
}

/// An event of the world that touched the study at `at`.
fn event(game: &Study, summary: &str, source: &str, at: i64) {
    let event = insert(
        &game.engine,
        "world_events",
        &[
            ("story_id", Value::from(game.story)),
            ("source", Value::from(source)),
            ("occurred_at", Value::from(at)),
            ("summary", Value::from(summary)),
        ],
    );
    insert(
        &game.engine,
        "locations_world_events",
        &[
            ("location_id", Value::from(STUDY)),
            ("world_event_id", Value::from(event)),
        ],
    );
}

/// An arrival writer's answer that holds its request to leaving out `facts`.
fn arrival_reply(facts: &[String]) -> Reply {
    let mut reply = Reply::from_value(&json!({
        "purpose": "arrival",
        "content": { "description": "You come in.", "summary": "You come in." },
    }))
    .unwrap();
    reply.prompt_excludes = facts.to_vec();
    reply
}

fn kinds(since: &Since) -> Vec<(Kind, String)> {
    since
        .changes
        .iter()
        .map(|change| (change.kind, change.fact.clone()))
        .collect()
}

#[test]
fn a_first_visit_has_nothing_to_compare_and_a_return_with_nothing_changed_says_so() {
    let mut game = study(Mode::PlayerNarrates);
    game.submit(noticed::LOOK_LINE, "look", vec![]);
    let out = game.submit("/go The Beech Clearing", "out", vec![]);
    assert_eq!(game.since(&out), None, "the clearing is a first visit");
    let back = game.submit("/go The Clerk's Study", "back", vec![]);
    let since = game.since(&back).expect("a return");
    assert_eq!(since.seen.room, STUDY);
    assert_eq!(since.changes, Vec::new());
    assert_eq!(since.facts(), vec![revisit::UNCHANGED.to_string()]);
    let names: Vec<&str> = since
        .seen
        .things
        .iter()
        .map(|thing| thing.name.as_str())
        .collect();
    for name in ["ledger", "brass button", "pewter mug", "ink pot", "desk"] {
        assert!(names.contains(&name), "{name} was seen: {names:?}");
    }
    // A thing shut away was never noticed, so it is not in the record.
    assert!(!names.contains(&"folded note") && !names.contains(&"lump of coal"));
    let people: Vec<&str> = since
        .seen
        .people
        .iter()
        .map(|who| who.name.as_str())
        .collect();
    assert_eq!(people, ["Tobin Reed", "Wren Hale", "Old Pell"]);
    let ways: Vec<i64> = since.seen.ways_out.iter().map(|way| way.room).collect();
    assert_eq!(ways, [CLEARING, STALL]);
}

#[test]
fn a_return_finds_what_changed_among_what_the_game_saw() {
    let mut game = study(Mode::PlayerNarrates);
    game.submit(noticed::LOOK_LINE, "look", vec![]);
    // An event that happened before the game left is not news on return.
    let left = game.now();
    event(&game, "The clerk swept the hearth.", "loader", left - 60);
    game.submit("/go The Beech Clearing", "out", vec![]);

    // While the game is away.
    let (ledger, ink, button, mug) = (
        game.thing("ledger"),
        game.thing("ink pot"),
        game.thing("brass button"),
        game.thing("pewter mug"),
    );
    let (tobin, wren, pell) = (
        game.person("Tobin Reed"),
        game.person("Wren Hale"),
        game.person("Old Pell"),
    );
    game.execute(
        "UPDATE items SET within_id = ?1, how = 'on' WHERE id = ?2",
        (DESK, ledger),
    );
    game.execute(
        "UPDATE items SET character_id = ?1, location_id = NULL, within_id = NULL, how = NULL \
         WHERE id = ?2",
        (tobin, ink),
    );
    game.execute(
        "UPDATE items SET location_id = ?1 WHERE id = ?2",
        (STALL, button),
    );
    game.execute(
        "UPDATE items SET disposition = 'broken' WHERE id = ?1",
        [mug],
    );
    // The note shut in the desk was never noticed: taking it out is no news.
    let note = game.thing("folded note");
    game.execute(
        "UPDATE items SET location_id = ?1, within_id = NULL, how = NULL WHERE id = ?2",
        (STALL, note),
    );
    game.put(wren, Some(STALL));
    game.execute(
        "UPDATE playthrough_vitals SET hp_current = 0 WHERE playthrough_id = ?1 AND character_id = ?2",
        (game.playthrough, pell),
    );
    let race: i64 = game
        .engine
        .store()
        .connection()
        .query_row(
            "SELECT race_id FROM characters WHERE id = ?1",
            [tobin],
            |row| row.get(0),
        )
        .unwrap();
    insert(
        &game.engine,
        "characters",
        &[
            ("fullname", Value::from("Ash Coyle")),
            ("nickname", Value::from("Ash")),
            ("story_id", Value::from(game.story)),
            ("race_id", Value::from(race)),
            ("location_id", Value::from(STUDY)),
            ("level", Value::from(1)),
            ("hit_die", Value::from(8)),
        ],
    );
    game.execute("DELETE FROM location_connections WHERE id = ?1", [TO_STALL]);
    insert(
        &game.engine,
        "location_connections",
        &[
            ("location_id", Value::from(STUDY)),
            ("connected_location_id", Value::from(READING_ROOM)),
            ("distance", Value::from("adjacent")),
            ("travel_method", Value::from("walking")),
            ("time_to_travel", Value::from("about a minute")),
        ],
    );
    event(&game, "The grate's fire went out.", "loader", left + 60);
    event(
        &game,
        "The Clerk's Study now opens onto The Reading Room instead of Cinder Lane Stall.",
        "world_mechanic",
        left + 60,
    );

    let back = game.submit("/go The Clerk's Study", "back", vec![]);
    let since = game.since(&back).expect("a return");
    assert_eq!(since.seen.at, left);
    assert_eq!(
        kinds(&since),
        vec![
            (
                Kind::Moved,
                "The ledger is on the desk now, not on the floor.".into()
            ),
            (Kind::Gone, "The brass button is gone.".into()),
            (Kind::Broken, "The pewter mug lies broken.".into()),
            (Kind::Held, "Tobin Reed has the ink pot now.".into()),
            (Kind::Left, "Wren Hale is no longer here.".into()),
            (Kind::Died, "Old Pell lies dead.".into()),
            (
                Kind::Came,
                "Ash Coyle is here now, and was not before.".into()
            ),
            (
                Kind::WayGone,
                "The way out to Cinder Lane Stall is gone.".into()
            ),
            (
                Kind::WayNew,
                "There is a way out to The Reading Room that was not there before.".into()
            ),
            (Kind::Event, "The grate's fire went out.".into()),
        ],
        "{since:#?}"
    );
    let subjects: Vec<(&str, i64)> = since
        .changes
        .iter()
        .take(4)
        .map(|change| (change.subject.0.as_str(), change.subject.1))
        .collect();
    assert_eq!(
        subjects,
        [
            ("items", ledger),
            ("items", button),
            ("items", mug),
            ("items", ink)
        ]
    );
    assert_eq!(since.facts().len(), since.changes.len());

    // Walking out keeps what the study holds now, for the next return; the
    // clearing is a return of its own.
    let out = game.submit("/go The Beech Clearing", "out again", vec![]);
    assert_eq!(
        game.since(&out).map(|since| since.seen.room),
        Some(CLEARING)
    );
    let again = game.submit("/go The Clerk's Study", "back again", vec![]);
    assert_eq!(game.since(&again).unwrap().changes, Vec::new());
}

#[test]
fn a_narrated_game_keeps_the_same_record_and_tells_it_to_no_prompt() {
    let mut game = study(Mode::Narrated);
    game.submit("/go The Beech Clearing", "out", vec![arrival_reply(&[])]);
    let ledger = game.thing("ledger");
    game.execute(
        "UPDATE items SET location_id = ?1 WHERE id = ?2",
        (STALL, ledger),
    );
    game.put(game.person("Wren Hale"), None);
    let changed = vec![
        "The ledger is gone.".to_string(),
        "Wren Hale is no longer here.".to_string(),
    ];
    let back = game.submit(
        "/go The Clerk's Study",
        "back",
        vec![arrival_reply(&changed)],
    );
    let since = game.since(&back).expect("a return");
    assert_eq!(since.facts(), changed);
    // A narrated game was shown everything lying there, noticed or not.
    let names: Vec<&str> = since
        .seen
        .things
        .iter()
        .map(|thing| thing.name.as_str())
        .collect();
    assert!(
        names.contains(&"folded note") && names.contains(&"lump of coal"),
        "{names:?}"
    );
    // And nothing is stamped.
    let stamped: i64 = game
        .engine
        .store()
        .connection()
        .query_row(
            "SELECT COUNT(*) FROM items WHERE noticed_at IS NOT NULL",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stamped, 0);
}

#[test]
fn a_resumed_move_keeps_the_record_the_uninterrupted_move_kept() {
    for stop in [revisit::LEFT, revisit::SINCE] {
        let mut game = study(Mode::PlayerNarrates);
        game.submit(noticed::LOOK_LINE, "look", vec![]);
        game.submit("/go The Beech Clearing", "out", vec![]);
        let mut replay = Replay::new(vec![]);
        match game.engine.submit_stopping(
            game.playthrough,
            "/go The Clerk's Study",
            "back",
            &mut replay,
            &mut |_| {},
            Some(stop),
        ) {
            Err(Error::Stopped(step)) => assert_eq!(step, stop),
            other => panic!("expected the move to stop after {stop}, got {other:?}"),
        }
        // Somebody takes the ledger while the worker is down: the resumed
        // move answers what it kept.
        let ledger = game.thing("ledger");
        game.execute(
            "UPDATE items SET location_id = ?1 WHERE id = ?2",
            (STALL, ledger),
        );
        let back = game.submit("/go The Clerk's Study", "back", vec![]);
        let since = game.since(&back).expect("a return");
        if stop == revisit::SINCE {
            assert_eq!(since.changes, Vec::new(), "stopped after {stop}");
        } else {
            assert_eq!(
                since.facts(),
                ["The ledger is gone."],
                "stopped after {stop}"
            );
        }
    }
}
