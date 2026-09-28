//! The line-reading portions: each case stands in a room written in the
//! file's `worlds` constant (`lib/engine_vectors/room.rb` in the Ruby
//! engine's repository documents its shape), and names records by id.

use crate::check_rooms;
use renderedstep_engine::cascade::{self, Escalation, State, Unavailable};
use renderedstep_engine::grammar::{self, Grammar, Reading, HELP};
use renderedstep_engine::intent::{build_intent, Intent};
use renderedstep_engine::refusal::{Refusal, KINDS, UNCHANGED};
use renderedstep_engine::room::{Exit, Person, Place, Record, Room, Thing, INTENTS, NOTHING};
use renderedstep_engine::slash_menu::{SlashMenu, HINTS, PHYSICAL_HINTS};
use serde_json::{json, Map, Value};

/// A room, and every record in it by id.
struct Built {
    room: Room,
    records: Vec<Record>,
}

impl Built {
    fn record(&self, id: &Value) -> Record {
        let id = id.as_i64().unwrap_or_else(|| panic!("an id, not {id}"));
        self.records
            .iter()
            .find(|record| record.id() == Some(id))
            .unwrap_or_else(|| panic!("no record {id}"))
            .clone()
    }
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

fn place(value: &Value) -> Place {
    Place {
        id: value["id"].as_i64().expect("id"),
        name: text(&value["name"]),
    }
}

fn person(value: &Value) -> Person {
    Person {
        id: value["id"].as_i64().expect("id"),
        fullname: text(&value["fullname"]),
        nickname: value["nickname"].as_str().map(str::to_string),
    }
}

fn thing(value: &Value, carried: bool) -> Thing {
    let mut thing = Thing::new(
        value["id"].as_i64().expect("id"),
        &text(&value["name"]),
        carried,
    );
    if let Some(bulk) = value["bulk"].as_str() {
        thing.bulk = bulk.into();
    }
    if let Some(use_kind) = value["use_kind"].as_str() {
        thing.use_kind = use_kind.into();
    }
    thing.combustible = value["combustible"].as_bool().unwrap_or(false);
    thing.template = value["template"].as_i64();
    thing
}

fn list(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

/// The room named `name` in a `worlds` constant.
fn build(worlds: &Value, name: &Value) -> Built {
    let world = list(worlds)
        .iter()
        .find(|pair| pair[0] == *name)
        .unwrap_or_else(|| panic!("no world {name}"))[1]
        .clone();
    let room = Room {
        protagonist: (!world["protagonist"].is_null()).then(|| person(&world["protagonist"])),
        here: (!world["here"].is_null()).then(|| place(&world["here"])),
        exits: list(&world["exits"])
            .iter()
            .map(|exit| Exit {
                place: place(exit),
                barrier: exit["barrier"].as_str().unwrap_or("open").to_string(),
                edge: exit["id"].as_i64().expect("an exit's id"),
                key: exit["key"].as_i64(),
            })
            .collect(),
        cast: list(&world["cast"]).iter().map(person).collect(),
        lying: list(&world["lying"])
            .iter()
            .map(|t| thing(t, false))
            .collect(),
        carried: list(&world["carried"])
            .iter()
            .map(|t| thing(t, true))
            .collect(),
        over: false,
    };
    let mut records: Vec<Record> = Vec::new();
    records.extend(room.here.clone().map(Record::Place));
    records.extend(room.exits.iter().map(|e| Record::Place(e.place.clone())));
    records.extend(room.protagonist.clone().map(Record::Person));
    records.extend(room.cast.iter().cloned().map(Record::Person));
    records.extend(room.lying.iter().cloned().map(Record::Thing));
    records.extend(room.carried.iter().cloned().map(Record::Thing));
    Built { room, records }
}

fn id_of(record: &Option<Record>) -> Value {
    record
        .as_ref()
        .and_then(Record::id)
        .map_or(Value::Null, Value::from)
}

fn compact(pairs: Vec<(&str, Value)>) -> Value {
    Value::Object(
        pairs
            .into_iter()
            .filter(|(_, value)| !value.is_null())
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn intent_json(intent: Option<&Intent>) -> Value {
    let Some(intent) = intent else {
        return Value::Null;
    };
    compact(vec![
        ("action", json!(intent.action)),
        ("destination", id_of(&intent.destination)),
        ("speaker", id_of(&intent.speaker)),
        ("item", id_of(&intent.item)),
        ("at", id_of(&intent.at)),
        ("also_named", id_of(&intent.also_named)),
        ("unknown_action", json!(intent.unknown_action)),
        (
            "physical",
            json!(intent.physical.as_ref().map(|c| c.token())),
        ),
    ])
}

fn reading_json(reading: Option<&Reading>) -> Value {
    let Some(reading) = reading else {
        return Value::Null;
    };
    compact(vec![
        ("intent", intent_json(reading.intent.as_ref())),
        ("refusal", json!(reading.refusal)),
        (
            "note",
            if reading.help {
                json!("help")
            } else {
                Value::Null
            },
        ),
        ("understood", json!(reading.understood)),
        ("wound", json!(reading.wound)),
        ("attempt", json!(reading.attempt)),
        ("resolved_by", json!(reading.resolved_by)),
    ])
}

fn refusal_json(refusal: Option<&Refusal>) -> Value {
    let Some(refusal) = refusal else {
        return Value::Null;
    };
    compact(vec![
        ("kind", json!(refusal.kind)),
        ("fact", json!(refusal.fact)),
        ("offer", json!(refusal.offer)),
        ("reason", json!(refusal.reason())),
        ("text", json!(refusal.text())),
        ("game_over", json!(refusal.game_over())),
    ])
}

fn pairs(table: &[(&str, &str)]) -> Value {
    Value::Array(table.iter().map(|(k, v)| json!([k, v])).collect())
}

#[test]
fn grammar() {
    check_rooms("grammar", json!({ "help": HELP }), |worlds, input| {
        if let Some(action) = input["action"].as_str() {
            return json!(grammar::word_for(action));
        }
        if input["world"].is_null() {
            let typed = text(&input["typed"]);
            return json!({ "unslashed": grammar::unslashed(&typed),
                           "slashed": grammar::slashed(&typed) });
        }
        let built = build(worlds, &input["world"]);
        let grammar = Grammar::new(&built.room);
        if let Some(token) = input["token"].as_str() {
            let choices = built.room.physical_actions();
            let choice = choices
                .iter()
                .find(|choice| choice.token() == token)
                .expect("an attempt the room offers");
            return json!(grammar.line_for(choice));
        }
        let typed = text(&input["typed"]);
        json!({
            "claims": grammar.claims(&typed),
            "reading_first": reading_json(grammar.reading_first(&typed).as_ref()),
            "parse": reading_json(Some(&grammar.parse(&typed))),
            "engine_view": reading_json(grammar.engine_view_reading(&typed, true).as_ref()),
            "engine_view_offline": reading_json(grammar.engine_view_reading(&typed, false).as_ref()),
        })
    });
}

#[test]
fn grammar_corpus() {
    check_rooms("grammar_corpus", json!({}), |worlds, input| {
        let built = build(worlds, &input["world"]);
        let grammar = Grammar::new(&built.room);
        let typed = text(&input["typed"]);
        let parsed = grammar.parse(&typed);
        let refusal = match &parsed.intent {
            Some(intent) if intent.refused() => {
                let offered = built.room.offered_for(&intent.action);
                Refusal::for_intent(intent, &typed, &offered).map(|r| r.text())
            }
            _ => parsed.refusal.clone(),
        };
        json!({
            "reading_first": reading_json(grammar.reading_first(&typed).as_ref()),
            "parse": reading_json(Some(&parsed)),
            "refusal": refusal,
        })
    });
}

#[test]
fn slash_menu() {
    let tables = json!({ "hints": pairs(HINTS), "physical_hints": pairs(PHYSICAL_HINTS) });
    check_rooms("slash_menu", tables, |worlds, input| {
        let menu = SlashMenu::for_room(&build(worlds, &input["world"]).room);
        json!({
            "verbs": menu.verbs.iter().map(|v| json!({ "word": v.word, "hint": v.hint })).collect::<Vec<_>>(),
            "targets": menu.targets.iter().map(|(w, names)| json!([w, names])).collect::<Vec<_>>(),
        })
    });
}

#[test]
fn classifier_intent() {
    let tables = json!({ "intents": INTENTS, "nothing": NOTHING });
    check_rooms("classifier_intent", tables, |worlds, input| {
        let room = build(worlds, &input["world"]).room;
        let intent = build_intent(
            &room,
            input["intent"].as_str(),
            input["target"].as_str(),
            input["also_named"].as_str(),
            input["thrown_at"].as_str(),
        );
        let refusal = Refusal::for_intent(&intent, "the line", &room.offered_for(&intent.action));
        json!({
            "intent": intent_json(Some(&intent)),
            "refused": intent.refused(),
            "refusal": refusal.map(|r| json!({ "kind": r.kind, "text": r.text() })),
        })
    });
}

/// The provider's whole body for a short-written answer set: a string is a
/// choice, a number a noul, and a question not named answers `nothing` or
/// 0.0.
fn body_for(reply: &Value, questions: &Value) -> Value {
    if reply.get("answers").is_some() {
        return reply.clone();
    }
    let answers: Map<String, Value> = questions
        .as_object()
        .expect("questions")
        .iter()
        .map(|(id, question)| {
            let value = reply.get(id).cloned().unwrap_or_else(|| {
                if question["type"] == "noul" {
                    json!(0.0)
                } else {
                    json!(NOTHING)
                }
            });
            let answer = if value.is_number() {
                json!({ "type": "noul", "noul": value })
            } else {
                json!({ "type": "choice", "choice": value, "confidence": 1.0 })
            };
            (id.clone(), answer)
        })
        .collect();
    json!({ "answers": answers })
}

#[test]
fn cascade() {
    let tables = json!({
        "presence_threshold": cascade::PRESENCE_THRESHOLD,
        "two_name_threshold": cascade::TWO_NAME_THRESHOLD,
        "nothing": NOTHING,
    });
    check_rooms("cascade", tables, |worlds, input| {
        let built = build(worlds, &input["world"]);
        let state = State::new(&built.room, &text(&input["typed"]));
        let questions = cascade::request(&state);
        let reply = &input["answers"];
        if reply.is_null() {
            let asked: Map<String, Value> = questions
                .as_object()
                .expect("questions")
                .iter()
                .map(|(id, question)| {
                    let options: Vec<&String> = question["criteria"]
                        .as_object()
                        .expect("criteria")
                        .keys()
                        .collect();
                    (
                        id.clone(),
                        json!({ "type": question["type"], "options": options }),
                    )
                })
                .collect();
            return json!({ "state": state.to_json(), "questions": asked });
        }
        let body = body_for(reply, &questions);
        let answered = match reply.get("unavailable") {
            Some(message) => Err(Unavailable(text(message))),
            None => Ok(&body),
        };
        let escalation = if input["escalate"] == json!(false) {
            Escalation::Off
        } else {
            Escalation::On
        };
        let outcome = cascade::compose(&state, &questions, answered, escalation);
        json!({
            "path": outcome.path,
            "target_present": outcome.target_present,
            "named_more_than_one": outcome.named_more_than_one,
            "intent": intent_json(outcome.intent.as_ref()),
        })
    });
}

fn intent_of(built: &Built, spec: &Value) -> Intent {
    let slot = |key: &str| (!spec[key].is_null()).then(|| built.record(&spec[key]));
    Intent {
        destination: slot("destination"),
        speaker: slot("speaker"),
        item: slot("item"),
        at: slot("at"),
        also_named: slot("also_named"),
        unknown_action: spec["unknown_action"].as_str().map(str::to_string),
        ..Intent::new(&text(&spec["action"]))
    }
}

#[test]
fn refusal() {
    let tables = json!({ "kinds": KINDS, "unchanged": UNCHANGED });
    check_rooms("refusal", tables, |worlds, input| {
        let built = build(worlds, &input["world"]);
        let typed = text(&input["typed"]);
        let refusal = match input["entry"].as_str() {
            Some("for") => {
                let offered: Vec<Record> = list(&input["offered"])
                    .iter()
                    .map(|id| built.record(id))
                    .collect();
                Refusal::for_intent(&intent_of(&built, &input["intent"]), &typed, &offered)
            }
            Some("unplayable") => Refusal::unplayable(
                &text(&input["intent"]["action"]),
                built.room.protagonist.is_some(),
                built.room.here.is_some(),
                &typed,
            ),
            Some("dead") => {
                let character =
                    (!input["character"].is_null()).then(|| built.record(&input["character"]));
                let person = match &character {
                    Some(Record::Person(person)) => Some(person),
                    _ => None,
                };
                Some(Refusal::dead(&typed, person))
            }
            Some("over") => Some(Refusal::over(
                input["ending"] == "concluded",
                built.room.protagonist.as_ref(),
                &typed,
            )),
            Some("new") => {
                match Refusal::new(&text(&input["kind"]), &typed, &text(&input["fact"]), None) {
                    Ok(refusal) => Some(refusal),
                    Err(error) => return json!({ "error": error.to_string() }),
                }
            }
            other => panic!("no entry {other:?}"),
        };
        refusal_json(refusal.as_ref())
    });
}

#[test]
fn classifier_request() {
    let tables = json!({
        "nothing": NOTHING,
        "examine_edit": [
            "without moving or taking it.",
            "without moving or taking it, or looking around the place in general."
        ],
    });
    check_rooms("classifier_request", tables, |worlds, input| {
        let built = build(worlds, &input["world"]);
        let state = State::new(&built.room, &text(&input["typed"]));
        json!({ "state": state.to_json(), "questions": cascade::request(&state) })
    });
}
