//! The words the engine builds, checked without playing a turn: the facts a
//! turn hands the narrator, the frame around them, the classifier's prompt
//! and call over a game's rows, and the grammar's reading of a typed name.

use regex::Regex;
use renderedstep_engine::classifier::{self, Reader};
use renderedstep_engine::facts::{self, Throw};
use renderedstep_engine::grammar::Grammar;
use renderedstep_engine::model::{Answer, Call, Failure, Unavailable};
use renderedstep_engine::records::{Records, Row};
use renderedstep_engine::room::{Place, Record, Room, Thing};
use renderedstep_engine::turn::room_of;
use serde_json::{json, Value};

/// A row from a JSON object.
fn row(value: Value) -> Row {
    value.as_object().expect("a row is an object").clone()
}

/// The thing is named with one article, and named.
fn assert_one_article(text: &str) {
    let doubled = Regex::new(r"(?i)\bthe (a|an|the) ").unwrap();
    assert!(!doubled.is_match(text), "two articles in: {text}");
    assert!(
        text.to_lowercase().contains("the frayed cable tie"),
        "the thing is not named in: {text}"
    );
}

fn tie(name: &str) -> Row {
    row(json!({
        "id": 1,
        "name": name,
        "description": null,
        "readable": true,
        "inscription": "PROPERTY OF DECK 4",
        "bulk": "handy",
    }))
}

/// A thing whose name arrived with its own article is named once in the
/// take, drop and read facts.
#[test]
fn the_take_drop_and_read_facts_name_an_article_bearing_thing_once() {
    let kael = row(json!({ "id": 2, "fullname": "Kael Veyra" }));
    let deck = row(json!({ "id": 3, "name": "Cargo Deck" }));
    for name in ["a frayed cable tie", "the frayed cable tie"] {
        let tie = tie(name);
        assert_one_article(&facts::taken(&tie, Some(&kael), Some(&deck)));
        assert_one_article(&facts::taken(&tie, Some(&kael), None));
        assert_one_article(&facts::dropped(&tie, &deck, Some(&kael)));
        assert_one_article(&facts::dropped(&tie, &deck, None));
        assert_one_article(&facts::dropped_and_broke(&tie, &deck, Some(&kael)));
        assert_one_article(&facts::read(&tie, "PROPERTY OF DECK 4"));
    }
}

/// Every way a throw comes out names an article-bearing thing once, given
/// the bare name a turn hands the throw fact.
#[test]
fn every_throw_fact_names_an_article_bearing_thing_once() {
    let outcomes = [
        Throw::Struck {
            target: "Mira Solis",
            broke: false,
        },
        Throw::Struck {
            target: "Mira Solis",
            broke: true,
        },
        Throw::Thrown {
            into: "The Shaft",
            broke: false,
        },
        Throw::Thrown {
            into: "The Shaft",
            broke: true,
        },
        Throw::ShortOf {
            target: "Mira Solis",
            range: 2,
            distance: 5,
            broke: false,
        },
        Throw::ShortOfTheWayOut {
            into: "The Shaft",
            here: "Cargo Deck",
            range: 1,
            distance: 4,
            broke: true,
        },
        Throw::Immovable,
        Throw::Fumbled { carried: true },
        Throw::Fumbled { carried: false },
    ];
    for name in ["a frayed cable tie", "the frayed cable tie"] {
        let thing = facts::bare_name(&tie(name));
        assert_eq!(thing, "frayed cable tie");
        for outcome in outcomes {
            assert_one_article(&facts::thrown(Some("Kael Veyra"), &thing, "handy", outcome));
            assert_one_article(&facts::thrown(None, &thing, "handy", outcome));
        }
        assert_one_article(&facts::carries(&thing, "handy", 6));
    }
}

/// A turn of a kind with no sentence of its own, and no fact, is framed as
/// a turn with neither: no doing line and no already-happened block.
#[test]
fn an_other_reading_adds_no_doing_sentence_and_no_fact_block() {
    let framed = facts::framing("The market is loud.", "wait", None, Some("other"));
    assert!(!framed.contains("looking more closely"), "{framed}");
    assert!(!framed.contains("ALREADY happened"), "{framed}");
    assert_eq!(
        framed,
        facts::framing("The market is loud.", "wait", None, None)
    );
    let examined = facts::framing("The market is loud.", "look at it", None, Some("examine"));
    assert!(
        examined.contains("looking more closely at something that is here"),
        "{examined}"
    );
}

const PLAYTHROUGH: i64 = 30;
const MARKET: i64 = 10;
const BELFRY: i64 = 11;

/// One game standing in Ashgate Market with nobody else, no way out and
/// nothing to hand; a second room, the Bell of Saint Aravel, is in the
/// story but not reached from here. `items` are this game's things, as
/// `(id, name, location)`: a thing with no location is carried.
fn market(items: &[(i64, &str, Option<i64>)]) -> Records {
    let items: Vec<Value> = items
        .iter()
        .map(|(id, name, location)| {
            json!({
                "id": id,
                "playthrough_id": PLAYTHROUGH,
                "name": name,
                "location_id": location,
                "character_id": null,
                "disposition": "intact",
            })
        })
        .collect();
    Records::from_json(&json!({
        "stories": [{ "id": 1 }],
        "locations": [
            { "id": MARKET, "story_id": 1, "name": "Ashgate Market" },
            { "id": BELFRY, "story_id": 1, "name": "The Bell of Saint Aravel" },
        ],
        "characters": [{
            "id": 20, "story_id": 1, "fullname": "Iri Calder",
            "is_protagonist": true, "location_id": null,
        }],
        "playthroughs": [{
            "id": PLAYTHROUGH, "story_id": 1, "character_id": 20,
            "current_location_id": MARKET, "ended_at": null,
        }],
        "items": items,
    }))
}

/// The classifier's call for `typed` in the market holding `items`.
fn classifier_call(items: &[(i64, &str, Option<i64>)], typed: &str) -> Call {
    let records = market(items);
    let room = room_of(&records, PLAYTHROUGH);
    classifier::call(&records, &room, typed)
}

/// An empty room, an empty floor and empty hands are said in words, and the
/// classifier is asked at temperature zero.
#[test]
fn the_classifier_says_an_empty_room_floor_and_hands_in_words_at_temperature_zero() {
    let call = classifier_call(&[], "go on then");
    for words in [
        "None. The player cannot go anywhere from here.",
        "Nobody. There is no one here to talk to.",
        "Nothing. There is nothing here to pick up.",
        "Nothing. The player is carrying nothing at all.",
    ] {
        assert!(
            call.user.contains(words),
            "{words:?} not in:\n{}",
            call.user
        );
    }
    assert!(call
        .user
        .contains("## Where The Player Is\nAshgate Market\n"));
    assert_eq!(call.temperature, Some(json!(0.0)));
}

/// A thing lying in another room of the story is neither listed nor
/// offered as a target; one lying here is both.
#[test]
fn the_classifier_does_not_offer_a_thing_lying_in_another_room() {
    let call = classifier_call(
        &[
            (40, "Iron Ledger", Some(BELFRY)),
            (41, "Brass Key", Some(MARKET)),
        ],
        "go on then",
    );
    let schema = call.schema.as_ref().expect("an intent schema").to_string();
    assert!(!call.user.contains("Iron Ledger"), "{}", call.user);
    assert!(!schema.contains("Iron Ledger"), "{schema}");
    assert!(call.user.contains("## What Is Lying Here\n- Brass Key\n"));
    assert!(schema.contains("Brass Key"), "{schema}");
}

/// A reader with no System One whose classifier answers one fixed reply.
struct Answering(Value);

impl Reader for Answering {
    fn system_one(&self) -> bool {
        false
    }

    fn questions(&mut self, _state: &Value, _questions: &Value) -> Result<Value, Unavailable> {
        Err(Unavailable("System One is off".into()))
    }

    fn classifier(&mut self, _call: &Call) -> Result<Answer, Failure> {
        Ok(Answer {
            content: self.0.clone(),
            model: None,
        })
    }
}

/// An examine naming a thing that is both lying here and carried resolves
/// to the one lying here.
#[test]
fn an_examine_of_one_name_lying_and_carried_resolves_to_the_one_lying_here() {
    let records = market(&[(50, "folded note", None), (51, "folded note", Some(MARKET))]);
    let room = room_of(&records, PLAYTHROUGH);
    let typed = "look at the folded note";
    let call = classifier::call(&records, &room, typed);
    let mut reader = Answering(json!({ "intent": "examine", "target": "folded note" }));
    let reading = classifier::read(&room, &call, typed, &mut reader).expect("a reading");
    assert_eq!(reading.path, "model");
    assert_eq!(reading.intent.action, "examine");
    let item = reading.intent.item.expect("the note resolves");
    assert_eq!(item.id(), Some(51));
    assert!(!item.carried(), "the note lying here, not the one in hand");
}

/// An office whose floor holds `lying`, as `(id, name)`.
fn office(lying: &[(i64, &str)]) -> Room {
    Room {
        protagonist: None,
        here: Some(Place {
            id: MARKET,
            name: "Ward Office 12".into(),
        }),
        lying: lying
            .iter()
            .map(|(id, name)| Thing::new(*id, name, false))
            .collect(),
        ..Room::default()
    }
}

/// A short name is found only as a whole word: "key" is not inside
/// "monkey".
#[test]
fn a_short_name_is_not_found_inside_a_longer_word() {
    let room = office(&[(60, "key")]);
    let reading = Grammar::new(&room).parse("take the monkey");
    assert!(reading.intent.is_none(), "{reading:?}");
    let refusal = reading.refusal.expect("a refusal");
    assert!(
        refusal.contains("there is no thing lying here"),
        "{refusal}"
    );
}

/// A name that holds a shorter one is taken by the whole name typed.
#[test]
fn a_take_of_the_longer_of_two_overlapping_names_takes_that_one() {
    let room = office(&[(61, "apron"), (62, "copy-room apron")]);
    let reading = Grammar::new(&room).parse("take the copy-room apron");
    let intent = reading.intent.expect("a take");
    assert_eq!(intent.action, "take");
    match intent.item {
        Some(Record::Thing(thing)) => assert_eq!(thing.name, "copy-room apron"),
        other => panic!("took {other:?}"),
    }
}
