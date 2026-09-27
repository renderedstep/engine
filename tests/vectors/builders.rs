//! The request-building portions: each case holds every row the database
//! held (`records`, written as `lib/engine_vectors/records.rb` in the Ruby
//! engine's repository describes) and the output a builder made of them.

use crate::check;
use renderedstep_engine::arrival::Arrival;
use renderedstep_engine::dialogue;
use renderedstep_engine::moment::{Direction, Handled, Moment};
use renderedstep_engine::plan::{self, Plan};
use renderedstep_engine::playthrough::Game;
use renderedstep_engine::realization::{self, Realization, Slot};
use renderedstep_engine::records::{id, Records};
use renderedstep_engine::roll::{self, Seed};
use renderedstep_engine::{cast, danger, population, text};
use renderedstep_engine::{identity, ledger, memory, moment, schemas, volition};
use serde_json::{json, Value};

fn int(value: &Value) -> i64 {
    value
        .as_i64()
        .unwrap_or_else(|| panic!("an integer, not {value}"))
}

fn records(input: &Value) -> Records {
    Records::from_json(&input["records"])
}

#[test]
fn ledger() {
    let tables = json!({ "rows": ledger::ROWS, "budget": ledger::BUDGET });
    check("ledger", tables, |input| {
        let records = records(input);
        let game = Game::new(&records, int(&input["playthrough"]));
        let story = game.story_id();
        let rooms: Vec<_> = records.select("locations", |row| row["story_id"] == story);
        let mut out = Vec::new();
        for character in records.select("characters", |row| row["story_id"] == story) {
            for room in std::iter::once(None).chain(rooms.iter().map(|room| Some(*room))) {
                out.push(json!({
                    "character": id(character),
                    "location": room.map(id),
                    "recall": ledger::recall(&game, character, room),
                }));
            }
        }
        Value::Array(out)
    });
}

#[test]
fn memory() {
    let tables = json!({
        "stop_words": memory::STOP_WORDS,
        "conclusions": memory::CONCLUSIONS,
        "conclusions_budget": moment::CONCLUSIONS_BUDGET,
        "memories_budget": moment::MEMORIES_BUDGET,
    });
    check("memory", tables, |input| {
        let records = records(input);
        let game = Game::new(&records, int(&input["playthrough"]));
        let character = game.character(int(&input["character"]));
        let query = input["query"].as_str();
        let replayed = int(&input["replayed"]);
        let recall: Vec<Value> =
            memory::recall(&game, character, query, replayed, memory::CONCLUSIONS)
                .into_iter()
                .map(|row| {
                    json!({
                        "id": id(row),
                        "resolution": memory::resolution(row),
                        "recollection": memory::recollection(&game, row),
                    })
                })
                .collect();
        json!({
            "recall": recall,
            "conclusions": moment::conclusions(&game, character, replayed, query),
            "recollections": moment::recollections(&game, character, replayed, query),
        })
    });
}

#[test]
fn plan() {
    check("plan", json!({ "way_out": plan::WAY_OUT }), |input| {
        let records = records(input);
        let rows = records.table("locations");
        let story = rows.first().map(|row| row["story_id"].clone());
        let entries: Vec<Value> = rows
            .iter()
            .filter(|row| Some(&row["story_id"]) == story.as_ref())
            .map(|room| {
                let plan = Plan::of(&records, room).map(|plan| {
                    json!({ "to_h": plan.to_json(), "sentences": plan.sentences(), "prompt": plan.to_prompt() })
                });
                json!({ "location": id(room), "plan": plan })
            })
            .collect();
        Value::Array(entries)
    });
}

#[test]
fn volition_request() {
    let tables = json!({
        "serves": volition::SERVES,
        "pressure_threshold": volition::PRESSURE_THRESHOLD,
    });
    check("volition_request", tables, |input| {
        let records = records(input);
        let playthrough = records
            .table("playthroughs")
            .first()
            .expect("a playthrough");
        let game = Game::new(&records, id(playthrough));
        let characters: Vec<_> = input["characters"]
            .as_array()
            .expect("characters")
            .iter()
            .map(|who| game.character(int(who)))
            .collect();
        let location = game.location(int(&input["location"]));
        volition::request(&game, &characters, location, input["line"].as_str())
    });
}

#[test]
fn moment() {
    let tables = json!({
        "replayed": 2,
        "conclusions": memory::CONCLUSIONS,
        "conclusions_budget": moment::CONCLUSIONS_BUDGET,
        "memories_budget": moment::MEMORIES_BUDGET,
        "recap_budget": moment::RECAP_BUDGET,
        "recap_scenes": moment::RECAP_SCENES,
    });
    check("moment", tables, |input| {
        let records = records(input);
        let game = Game::new(&records, int(&input["playthrough"]));
        let replayed = int(&input["replayed"]);
        let moment = Moment::new(game);
        let location = game.current_location();
        let mut handled = Vec::new();
        if let Some(item) = game.carried().first() {
            handled.push((id(item), Direction::Taken));
        }
        if let Some(item) = game.items_lying_in(location).first() {
            handled.push((id(item), Direction::Dropped));
        }
        let handled: Vec<Value> = handled
            .into_iter()
            .map(|(item, direction)| {
                let marked = Moment { handled: Some(Handled { item, direction }), ..Moment::new(game) };
                json!({ "item": item, "direction": direction.name(), "narration": marked.narration_context(true, true) })
            })
            .collect();
        let story = game.story_id();
        let quests: Vec<i64> = records
            .select("quests", |row| row["story_id"] == story)
            .iter()
            .map(|row| id(row))
            .collect();
        let endings: Vec<Value> = records
            .select("quest_outcomes", |row| {
                quests.iter().any(|quest| row["quest_id"] == *quest)
            })
            .into_iter()
            .map(|outcome| {
                let ended = Moment {
                    ending: Some(outcome),
                    ..Moment::new(game)
                };
                json!({ "outcome": id(outcome), "narration": ended.narration_context(true, true) })
            })
            .collect();
        let player = game.protagonist().map(id);
        let characters: Vec<Value> = game
            .cast_in(location)
            .into_iter()
            .filter(|who| Some(id(who)) != player)
            .map(|who| {
                json!({
                    "character": id(who),
                    "context": moment.character_context(who, replayed, None),
                    "personal_facts": moment.personal_facts(who),
                })
            })
            .collect();
        json!({
            "narration": moment.narration_context(true, true),
            "narration_bare": moment.narration_context(false, false),
            "handled": handled,
            "endings": endings,
            "characters": characters,
        })
    });
}

/// A schema by its Ruby class name, built by the class method a case names.
fn schema(input: &Value) -> Value {
    let args = input["args"].as_array().cloned().unwrap_or_default();
    let texts: Vec<&str> = args.iter().filter_map(Value::as_str).collect();
    match (
        input["schema"].as_str().expect("a schema"),
        input["for"].as_str(),
    ) {
        ("Scene::Schema", None) => schemas::scene(),
        ("Story::Schema", None) => schemas::story(),
        ("Character::Schema", None) => schemas::character(),
        ("Character::DesireWriter::Schema", None) => schemas::desire_writer(),
        ("Interaction::Schema", None) => schemas::interaction(),
        ("Location::PlaceSchema", None) => schemas::location_place(),
        ("Location::DetailSchema", None) => schemas::detail(0),
        ("Location::ExitsSchema", None) => schemas::location_exits(),
        ("Item::InscriptionSchema", None) => schemas::item_inscription(),
        ("Quest::Schema", None) => schemas::quest(),
        ("Universe::PhysicalSchema", None) => schemas::universe_physical(),
        ("Universe::SocietalSchema", None) => schemas::universe_societal(),
        ("Playthrough::IntentSchema", Some("for")) => schemas::intent(&texts),
        ("Interaction::Schema", Some("with_actions")) => schemas::interaction_with_actions(&texts),
        ("Location::DetailSchema", Some("for_people")) => schemas::detail(int(&args[0])),
        (name, built_by) => panic!("no schema {name} {built_by:?}"),
    }
}

#[test]
fn request_identity() {
    let tables = json!({ "request_identity_version": identity::VERSION });
    check("request_identity", tables, |input| {
        if input.get("schema").is_some() {
            return schema(input);
        }
        let requests = &input["requests"];
        let mut output = json!({
            "canonical": identity::canonical_text(requests),
            "digest": identity::digest(requests),
        });
        // The two designated sets also record the identity their task prints:
        // the classifier's, whose first request is the corpus's first line,
        // and the prompt set's, which asks for an examine.
        let fields = output.as_object_mut().unwrap();
        if requests.get("attack-brace-lunge").is_some() {
            fields.insert("identity".into(), identity::classifier_identity(requests));
        } else if requests.get("examine").is_some() {
            fields.insert("identity".into(), identity::of(requests));
        }
        output
    });
}

/// The arrival stage's request: the game walks from where it stands into the
/// other room, or, with no game, the story opens in its one room.
fn arrival(records: &Records) -> Value {
    let game = records
        .table("playthroughs")
        .first()
        .map(|row| Game::new(records, id(row)));
    let origin = game.and_then(|game| game.current_location()).map(id);
    let location = records
        .table("locations")
        .iter()
        .find(|room| Some(id(room)) != origin)
        .expect("the room arrived in");
    Arrival {
        records,
        location,
        previous_scene: game.and_then(|game| game.current_scene()),
        game,
        opening: game.is_none(),
    }
    .request()
}

/// What a dialogue case needs beyond its rows: the line the player typed
/// and the character's stored answer (`tests/fixtures/dialogue_replay.json`).
fn replayed(case: &str) -> (String, Value) {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/dialogue_replay.json");
    let fixture: Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("the replay fixture"))
            .expect("JSON");
    let entry = fixture["cases"]
        .as_array()
        .expect("cases")
        .iter()
        .find(|entry| entry["id"] == case)
        .unwrap_or_else(|| panic!("no replay for {case}"))
        .clone();
    (
        entry["line"].as_str().unwrap().to_string(),
        entry["answer"].clone(),
    )
}

/// The dialogue bench's two requests: the character pass, and then the
/// narrator pass over the stored answer, whose receipt is read off the rows
/// the character pass was built from.
fn dialogue_request(records: &Records, input: &Value) -> Value {
    let game = Game::new(
        records,
        id(records.table("playthroughs").first().expect("a game")),
    );
    let chat = records
        .first("chats", |chat| chat["purpose"] == "character")
        .expect("the character's chat");
    let character = game.character(int(&chat["character_id"]));
    let case = input["id"].as_str().expect("a case id");
    let (line, answer) = replayed(case);
    if int(&input["call"]) == 0 {
        return dialogue::character_request(&game, character, &line);
    }
    let before = crate::load("kept_requests")
        .cases
        .into_iter()
        .find(|other| other["input"]["id"] == case && other["input"]["call"] == 0)
        .expect("the character pass");
    let before_records = Records::from_json(&before["input"]["records"]);
    let before_game = Game::new(&before_records, game.id());
    let choice = answer["engine_action"].as_str().unwrap_or(dialogue::NONE);
    let fact = dialogue::receipt(&before_game, before_game.character(id(character)), choice);
    let reaction = answer.as_object().expect("an answer").clone();
    dialogue::narrator_request(&game, character, &line, &reaction, &fact)
}

#[test]
fn kept_requests() {
    check("kept_requests", json!({ "dialogue_cases": 3 }), |input| {
        let records = records(input);
        match input["set"].as_str().expect("a set") {
            "arrival-branches" => arrival(&records),
            set if set.starts_with("realization-") => realization(&records, &input["id"]),
            set if set.starts_with("desires-dialogue-") => dialogue_request(&records, input),
            other => panic!("no builder for the kept set {other}"),
        }
    });
}

/// The realization bench's request for the room a case stages: the one with
/// a written detail waiting for its ways out, or the stub it peopled.
fn realization(records: &Records, case: &Value) -> Value {
    let pending =
        |row: &&renderedstep_engine::records::Row| !row["generation_checkpoint"].is_null();
    let location = records
        .table("locations")
        .iter()
        .find(pending)
        .or_else(|| {
            let staged: Vec<_> = records
                .table("locations")
                .iter()
                .filter(|row| row["detail_level"] == "stub" && !row["population"].is_null())
                .collect();
            assert_eq!(staged.len(), 1, "one staged room");
            staged.first().copied()
        })
        .expect("the room being written");
    let slots = staged_cast(records, location, case.as_str().expect("a case id"));
    let chat = records.table("chats").last();
    identity::canonical(
        &Realization {
            records,
            location,
            slots,
        }
        .request(chat),
    )
}

/// The people the bench stages for a room (`Eval::Realization::Branches#seed_cast!`):
/// drawn as the registry would, from a generator seeded with the world's and
/// the case's names.
fn staged_cast(
    records: &Records,
    location: &renderedstep_engine::records::Row,
    case: &str,
) -> Vec<Slot> {
    let story = records
        .find("stories", int(&location["story_id"]))
        .expect("the room's story");
    let world = story["title"].as_str().expect("a title");
    let (_, count) = population::for_room(
        location["name"].as_str().unwrap(),
        location["population"].as_str(),
    );
    let (room, in_world) = realization::room_for_people(records, location);
    let drawn = count.min(room).min(in_world);
    let mut rng = Seed {
        story: text::crc32(world.as_bytes()).into(),
        sequence: text::crc32(case.as_bytes()).into(),
        kind: roll::POPULATION,
        ..Seed::default()
    }
    .generator();
    let universe = story["universe_id"].clone();
    let races: Vec<_> = records.select("races", |race| race["universe_id"] == universe);
    (0..drawn)
        .map(|_| {
            let monstrous =
                danger::monstrous(location["danger"].as_str().unwrap_or("safe"), &mut rng);
            let mut pool: Vec<_> = races
                .iter()
                .filter(|race| race["monstrous"] == monstrous)
                .copied()
                .collect();
            if pool.is_empty() {
                pool = races.clone();
            }
            pool.sort_by_key(|race| race["name"].as_str().unwrap().to_string());
            let race = pool[roll::one_of(pool.len(), &mut rng)]["name"]
                .as_str()
                .unwrap()
                .to_string();
            let age = rng.range(cast::NPC_AGES.0, cast::NPC_AGES.1);
            let sex = cast::SEXES[roll::one_of(cast::SEXES.len(), &mut rng)].to_string();
            Slot { race, age, sex }
        })
        .collect()
}

/// A transport that keeps what it was sent and answers every call with an
/// answer that satisfies any schema the kept sets carry.
#[derive(Default)]
struct Kept {
    sent: Vec<Value>,
}

impl renderedstep_engine::model::http::Transport for Kept {
    fn post(
        &mut self,
        _endpoint: &renderedstep_engine::model::route::Endpoint,
        body: &Value,
        _options: &renderedstep_engine::model::http::Options,
        _on_line: Option<&mut (dyn FnMut(&str) + '_)>,
    ) -> Result<renderedstep_engine::model::http::Posted, renderedstep_engine::model::http::Unreached>
    {
        self.sent.push(body.clone());
        let required = body["response_format"]["json_schema"]["schema"]["required"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let content = if required.is_empty() {
            json!("You look around.")
        } else {
            let fields: serde_json::Map<String, Value> = required
                .iter()
                .map(|key| (key.as_str().unwrap().to_string(), json!("x")))
                .collect();
            json!(Value::Object(fields).to_string())
        };
        Ok(renderedstep_engine::model::http::Posted {
            status: 200,
            body: json!({"choices": [{"message": {"content": content}, "finish_reason": "stop"}]})
                .to_string(),
        })
    }
}

/// Every kept request, sent through the live client: the body that goes out
/// carries exactly the request the kept set stores, on either route.
#[test]
fn kept_requests_on_the_live_path() {
    use renderedstep_engine::model::{Agent, Book, Call, Filed, Live, Models, Route, Secret};
    let vectors = crate::load("kept_requests");
    let engine = renderedstep_engine::parity::open_world(include_str!(
        "../../parity/worlds/the-quay-house.sql"
    ))
    .expect("a database for the receipts");
    let mut checked = 0;
    for case in &vectors.cases {
        let input = crate::with_shared_records(&case["input"], &vectors.cases);
        let records = records(&input);
        let request = match input["set"].as_str().expect("a set") {
            "arrival-branches" => arrival(&records),
            set if set.starts_with("realization-") => realization(&records, &input["id"]),
            _ => dialogue_request(&records, &input),
        };
        assert_eq!(request, case["output"], "{}", case["name"]);
        for route in [
            Route::Direct {
                key: Secret::new("own"),
            },
            Route::Relay {
                base_url: "https://relay.example/relay/openrouter".into(),
                token: Secret::new("t"),
            },
        ] {
            let mut live = Live::with_transport(route, Kept::default());
            let mut agent = Agent::new(Filed {
                purpose: "kept".into(),
                ..Filed::default()
            });
            let mut mirror = Records::default();
            let mut book = Book {
                store: engine.store(),
                records: &mut mirror,
            };
            live.ask(
                &mut book,
                &mut agent,
                &Call::from_request(&request),
                None,
                None,
            )
            .unwrap_or_else(|failure| panic!("{}: {failure}", case["name"]));
            let body = &live.transport().sent[0];
            let messages = body["messages"].as_array().expect("messages");
            let mut expected = Vec::new();
            if let Some(system) = request["system"].as_str() {
                expected.push(json!({ "role": "developer", "content": system }));
            }
            for message in request["history"].as_array().expect("history") {
                if message["role"] == "system" {
                    assert_eq!(
                        message["content"], request["system"],
                        "{}: one set of instructions",
                        case["name"]
                    );
                } else {
                    expected.push(message.clone());
                }
            }
            expected.push(json!({ "role": "user", "content": request["user"] }));
            assert_eq!(messages, &expected, "{}", case["name"]);
            if request["schema"].is_null() {
                assert!(body.get("response_format").is_none(), "{}", case["name"]);
            } else {
                let sent = &body["response_format"]["json_schema"];
                let mut stored = request["schema"]["schema"].clone();
                let strict = stored.as_object_mut().unwrap().remove("strict").unwrap();
                assert_eq!(sent["schema"], stored, "{}", case["name"]);
                assert_eq!(sent["strict"], strict, "{}", case["name"]);
                assert_eq!(
                    sent["name"],
                    renderedstep_engine::model::wire::schema_name(
                        request["schema"]["name"].as_str().unwrap()
                    ),
                    "{}",
                    case["name"]
                );
            }
            checked += 1;
        }
    }
    println!(
        "kept_requests on the live path: {checked} of {checked} sends carry the stored request"
    );
}

/// `rake eval:realization_digest`'s branch requests, built here: every
/// staged case of the realization bench, digested as the bench names them.
/// The Ruby engine prints `4536c9cd93c39e2a` at the commit `parity/README.md`
/// names, and the stored baseline carries the same identity.
#[test]
fn realization_branch_requests_digest_as_the_bench_names_them() {
    let vectors = crate::load("kept_requests");
    let mut requests = serde_json::Map::new();
    for case in &vectors.cases {
        let input = crate::with_shared_records(&case["input"], &vectors.cases);
        if !input["set"]
            .as_str()
            .unwrap_or_default()
            .starts_with("realization-")
        {
            continue;
        }
        let records = records(&input);
        let id = input["id"].as_str().expect("a case id").to_string();
        requests.insert(id, realization(&records, &input["id"]));
    }
    let identity = identity::of(&Value::Object(requests.clone()));
    println!(
        "realization branch requests: {} cases, identity {identity}",
        requests.len()
    );
    assert_eq!(
        identity,
        json!({ "version": 1, "digest": "4536c9cd93c39e2a" })
    );
}
