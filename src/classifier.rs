//! `Playthrough::Classifier#classify`, over a room of plain records: a line
//! the grammar did not claim, read by System One first where it is on, and
//! by the classifier model when System One is off, escalates, or is
//! unavailable.
//!
//! System One is asked ten typed questions over the room's closed sets and
//! the engine composes the answers into a reading
//! ([`crate::cascade::compose`]). The model is asked for one intent and one
//! target out of the same sets, at temperature zero, and its answer is
//! resolved to records the same way ([`crate::intent::build_intent`]).
//!
//! Where the classifier has the local reader ([`Reader::local`]), it is the
//! only reader: what it reads is played, and what it cannot read is refused
//! in the engine's words ([`read_locally`]). The model call is not made.
//!
//! A turn reads its line through this ([`crate::turn::Turn`]), and so does a
//! caller holding a staged position's rows: whatever answers the two calls
//! is a [`Reader`], so the reading is the turn's own either way. Nothing here
//! writes: a turn counts a reach that found nothing and a line that named
//! two things itself.

use crate::cascade::{self, Escalation};
use crate::data;
use crate::intent::{build_intent, Intent, Unread};
use crate::model::{Answer, Call, Failure, Unavailable};
use crate::records::{string, Records};
use crate::room::Room;
use crate::schemas;
use serde_json::Value;

/// `Playthrough::Classifier::MODEL_PATHS`: the readings a model call made.
pub const MODEL_PATHS: [&str; 3] = ["model", "typed_model_escalated", "typed_model_unavailable"];

/// `scenes.resolved_by` for a line the local reader was unsure of, played on
/// its own reading all the same: never a model call's.
pub const LOCAL_UNSURE: &str = "typed_model_unsure";

/// `scenes.resolved_by` for a free-typed throw the local reader left unread.
pub const LOCAL_THROW: &str = "typed_model_throw";

/// `scenes.resolved_by` for a free-typed line that puts a thing on or takes
/// it off ([`cascade::dressed`]): no reader was asked.
pub const DRESS: &str = "typed_model_dress";

/// `scenes.resolved_by` for a line refused because the local reader is down
/// or not set up.
pub const LOCAL_DOWN: &str = "typed_model_down";

/// What answers the two calls a line is read through.
pub trait Reader {
    /// Whether System One is asked first (`SystemOneAgent.configured?`).
    fn system_one(&self) -> bool;

    /// Whether the line is the local reader's alone ([`read`]).
    fn local(&self) -> bool {
        false
    }

    /// System One's typed questions about `state`: the answer payload.
    fn questions(&mut self, state: &Value, questions: &Value) -> Result<Value, Unavailable>;

    /// The classifier model's call.
    fn classifier(&mut self, call: &Call) -> Result<Answer, Failure>;
}

/// One line, read.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub intent: Intent,
    /// Which reader answered (`scenes.resolved_by`).
    pub path: &'static str,
    /// System One's two readings, where it was asked and answered them.
    pub target_present: Option<f64>,
    pub named_more_than_one: Option<f64>,
}

/// Reads `typed` in `room`: System One first where it is on, then the model
/// call `call` where System One did not compose an intent.
///
/// The local reader's line never reaches the model call ([`read_locally`]).
/// A line that puts a thing on or takes it off is asked of no reader
/// ([`DRESS`]).
pub fn read(
    room: &Room,
    call: &Call,
    typed: &str,
    reader: &mut dyn Reader,
) -> Result<Reading, Failure> {
    if reader.local() {
        return Ok(read_locally(room, typed, reader));
    }
    if cascade::dressed(room, typed) {
        return Ok(unread(Unread::Dress, DRESS, None, None));
    }
    let (composed, path, target_present, named_more_than_one) = if reader.system_one() {
        let state = cascade::State::new(room, typed);
        let questions = cascade::request(&state);
        let reply = reader
            .questions(&state.to_json(), &questions)
            .map_err(|unavailable| cascade::Unavailable(unavailable.0));
        let outcome = cascade::compose(
            &state,
            &questions,
            reply.as_ref().map_err(Clone::clone),
            Escalation::On,
        );
        (
            outcome.intent,
            outcome.path,
            outcome.target_present,
            outcome.named_more_than_one,
        )
    } else {
        (None, "model", None, None)
    };
    let intent = match composed {
        Some(intent) => intent,
        None => {
            let answer = reader.classifier(call)?;
            let field = |key: &str| {
                answer
                    .content
                    .get(key)
                    .and_then(Value::as_str)
                    .map(str::to_string)
            };
            build_intent(
                room,
                field("intent").as_deref(),
                field("target").as_deref(),
                field("also_named").as_deref(),
                field("thrown_at").as_deref(),
            )
        }
    };
    Ok(Reading {
        intent,
        path,
        target_present,
        named_more_than_one,
    })
}

/// The local reader's reading: the presence gate on its own
/// ([`Escalation::Off`]), so a line it is unsure of is played on its own
/// reading ([`LOCAL_UNSURE`]), and a target it reads as absent is refused
/// with the list of what is here. Three lines it does not read at all: a
/// free-typed throw, which is asked nothing and pointed at `/throw`; a line
/// that puts a thing on or takes it off, asked nothing and pointed at
/// `/wear` and `/remove`; and any line while the reader is down or answers
/// nothing it can compose.
pub fn read_locally(room: &Room, typed: &str, reader: &mut dyn Reader) -> Reading {
    if cascade::thrown(typed) {
        return unread(Unread::Throw, LOCAL_THROW, None, None);
    }
    if cascade::dressed(room, typed) {
        return unread(Unread::Dress, DRESS, None, None);
    }
    let state = cascade::State::new(room, typed);
    let questions = cascade::request(&state);
    let reply = reader
        .questions(&state.to_json(), &questions)
        .map_err(|unavailable| cascade::Unavailable(unavailable.0));
    let outcome = cascade::compose(
        &state,
        &questions,
        reply.as_ref().map_err(Clone::clone),
        Escalation::Off,
    );
    let (presence, two_name) = (outcome.target_present, outcome.named_more_than_one);
    let Some(intent) = outcome.intent else {
        return unread(Unread::ReaderDown, LOCAL_DOWN, presence, two_name);
    };
    let unsure = presence.is_some_and(|p| p < cascade::PRESENCE_THRESHOLD)
        || two_name.is_some_and(|p| p >= cascade::TWO_NAME_THRESHOLD);
    Reading {
        intent,
        path: if unsure { LOCAL_UNSURE } else { outcome.path },
        target_present: presence,
        named_more_than_one: two_name,
    }
}

/// A line left unread, refused in the engine's own words.
fn unread(
    unread: Unread,
    path: &'static str,
    target_present: Option<f64>,
    named_more_than_one: Option<f64>,
) -> Reading {
    Reading {
        intent: Intent {
            unread: Some(unread),
            ..Intent::new("other")
        },
        path,
        target_present,
        named_more_than_one,
    }
}

/// The classifier model's call for `typed` in `room`: its instructions, the
/// room's closed sets as the prompt, and the intent schema whose target is
/// one of them.
pub fn call(records: &Records, room: &Room, typed: &str) -> Call {
    let mut targets: Vec<String> = Vec::new();
    targets.extend(room.exits.iter().map(|exit| exit.place.name.clone()));
    for person in &room.cast {
        targets.push(person.fullname.clone());
        targets.extend(person.nickname.clone());
    }
    targets.extend(room.lying.iter().map(|thing| thing.name.clone()));
    targets.extend(room.carried.iter().map(|thing| thing.name.clone()));
    targets.extend(room.reader_actions().iter().map(|choice| choice.token()));
    let names: Vec<&str> = targets.iter().map(String::as_str).collect();
    Call {
        system: Some(data::classifier_instructions().to_string()),
        schema: Some(schemas::intent(&names)),
        ..Call::prompt(command_prompt(records, room, typed))
    }
    .with_temperature(0.0)
}

/// `#command_prompt`.
fn command_prompt(records: &Records, room: &Room, typed: &str) -> String {
    let here = room
        .here
        .as_ref()
        .map_or("Nowhere in particular.".to_string(), |place| {
            place.name.clone()
        });
    let exits: Vec<String> = room
        .exits
        .iter()
        .map(|exit| {
            let note = records
                .find("location_connections", exit.edge)
                .map(|edge| {
                    let barrier = match exit.barrier.as_str() {
                        "open" => String::new(),
                        "keyed" => "; locked, open before crossing".into(),
                        _ => "; jammed, open before crossing".into(),
                    };
                    format!(
                        " ({}, {}{barrier})",
                        string(edge, "distance"),
                        string(edge, "travel_method")
                    )
                })
                .unwrap_or_default();
            format!("- {}{note}", exit.place.name)
        })
        .collect();
    let list = |lines: Vec<String>, empty: &str| {
        if lines.is_empty() {
            empty.to_string()
        } else {
            lines.join("\n")
        }
    };
    let cast = room
        .cast
        .iter()
        .map(
            |person| match person.nickname.as_deref().filter(|n| !n.trim().is_empty()) {
                Some(nickname) => format!("- {} ({nickname})", person.fullname),
                None => format!("- {}", person.fullname),
            },
        )
        .collect();
    let things = |things: &[crate::room::Thing]| {
        things
            .iter()
            .map(|thing| format!("- {}", thing.name))
            .collect()
    };
    let physical: Vec<String> = room
        .reader_actions()
        .iter()
        .map(|choice| format!("{}: {}", choice.token(), choice.name()))
        .collect();
    format!(
        "## Where The Player Is\n{here}\n\n## Ways Out\n{}\n\n## Who Is Here\n{}\n\n## What Is Lying Here\n{}\n\n\
         ## What The Player Is Carrying\n{}\n\n## Physical Actions (token: one attempt)\n{}\n\n## The Player Types\n{typed}\n",
        list(exits, "None. The player cannot go anywhere from here."),
        list(cast, "Nobody. There is no one here to talk to."),
        list(things(&room.lying), "Nothing. There is nothing here to pick up."),
        list(things(&room.carried), "Nothing. The player is carrying nothing at all."),
        list(physical, "None are available."),
    )
}
