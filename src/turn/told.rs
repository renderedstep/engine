//! One typed line played the way every front end plays it
//! (`Playthrough::Turn` under `Playthrough::Session`): accepted as a
//! submission, read, refused or played, told in prose by the models, and
//! answered by the world.
//!
//! It asks through [`Models`], so the same turn plays against the live
//! models or against fixed replies. Every effect is written with its journal
//! receipt in one short transaction, and a model answer is kept only after
//! its call returns; nothing holds a transaction open across a call. When a
//! call for prose fails after an effect was written, the turn keeps the
//! engine's own words in its place and still finishes: the world answers and
//! the clock moves, and nobody gets a free turn out of a failed paragraph.

use super::Mechanics;
use crate::arrival::Arrival;
use crate::command::{self, encode, Journal, Produced};
use crate::data;
use crate::dialogue::sanitize;
use crate::engine::Error;
use crate::grammar::{self, Grammar};
use crate::intent::Intent;
use crate::model::{Agent, Book, Call, Failure, Filed, Models};
use crate::moment::{Direction, Handled, Moment};
use crate::playthrough::Game;
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::refusal::Refusal;
use crate::room::{Choice, Record};
use crate::store::Store;
use crate::text::{is_blank, presence, upcase_first};
use serde_json::{json, Value};

mod realize;

/// `Scene::TURN_MINUTES["action"]`, in seconds.
const ACTION_SECONDS: i64 = 5 * 60;

/// How a submission ended.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Turned {
    /// The scene the turn is answered with, if it wrote one.
    pub scene: Option<i64>,
    /// The engine's refusal, for a line it would not play.
    pub refusal: Option<Refusal>,
    /// The model answered with crisis resources, and the engine's words
    /// stand in their place (`Playthrough::SafetyNotice`).
    pub safety_notice: bool,
    /// The prose fell back on the engine's words because there is no model
    /// to ask (`Playthrough::SetupNotice`).
    pub setup: bool,
}

/// What an arrival is written from: the request, the story time it
/// happens at, the cast, the facts on the way in and the untold tolls.
type ArrivalInputs = (Value, i64, Vec<i64>, Vec<String>, Vec<i64>);

/// A scene a turn wrote, and what the turn knows about it beside the row.
#[derive(Clone, Debug, PartialEq)]
struct Told {
    id: i64,
    /// The tolls its paragraph told, where it did not tell them all.
    tolls: Option<Vec<i64>>,
    safety: bool,
    setup: bool,
}

impl Told {
    fn plain(id: i64) -> Told {
        Told {
            id,
            tolls: None,
            safety: false,
            setup: false,
        }
    }

    fn encoded(&self) -> Value {
        encode::scene(self.id, self.tolls.as_deref(), self.safety, self.setup)
    }
}

enum Played {
    Scene(Told),
    Refused(Refusal),
    Nothing,
}

/// A turn in hand.
pub struct Turn<'s, 'm> {
    m: Mechanics<'s>,
    models: &'m mut dyn Models,
    on_chunk: &'m mut dyn FnMut(&str),
    journal: Option<Journal>,
    safety: bool,
}

fn model(failure: Failure) -> Error {
    Error::Model(failure)
}

/// A value the journal keeps, and the scalar it came from.
fn nil() -> Value {
    Value::Null
}

impl<'s, 'm> Turn<'s, 'm> {
    pub fn new(
        store: &'s Store,
        playthrough: i64,
        models: &'m mut dyn Models,
        on_chunk: &'m mut dyn FnMut(&str),
    ) -> Result<Turn<'s, 'm>, Error> {
        Ok(Turn {
            m: Mechanics::new(store, playthrough)?,
            models,
            on_chunk,
            journal: None,
            safety: false,
        })
    }

    pub fn records(&self) -> &Records {
        &self.m.records
    }

    fn filed(&self, purpose: &str) -> Filed {
        Filed {
            purpose: purpose.to_string(),
            playthrough: Some(self.m.playthrough),
            character: None,
            player: int(self.m.game().row, "player_id"),
        }
    }

    // --- the journal -------------------------------------------------------

    /// `Journal.commit`: `body`'s writes and the receipt saying they happened,
    /// in one transaction, or neither.
    fn commit<T>(
        &mut self,
        key: &str,
        encoded: impl FnOnce(&T) -> Value,
        body: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.refuse_to_resume(key)?;
        self.m.store.savepoint()?;
        let result = body(self).and_then(|value| {
            let receipt = encoded(&value);
            if let Some(journal) = self.journal.as_mut() {
                journal.save(self.m.store, &mut self.m.records, key, receipt)?;
            }
            Ok(value)
        });
        match result {
            Ok(value) => {
                self.m.store.release()?;
                Ok(value)
            }
            Err(error) => {
                self.m.store.rollback_to();
                Err(error)
            }
        }
    }

    /// `Journal.remember`: `body` runs with no transaction open, and what it
    /// answered is kept afterwards.
    fn remember<T>(
        &mut self,
        key: &str,
        encoded: impl FnOnce(&T) -> Value,
        body: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        self.refuse_to_resume(key)?;
        let value = body(self)?;
        let receipt = encoded(&value);
        if let Some(journal) = self.journal.as_mut() {
            journal.save(self.m.store, &mut self.m.records, key, receipt)?;
        }
        Ok(value)
    }

    fn refuse_to_resume(&self, key: &str) -> Result<(), Error> {
        if self
            .journal
            .as_ref()
            .is_some_and(|journal| journal.saved(key))
        {
            return Err(Error::Unsupported(format!(
                "resuming a turn from its journal (the {key} step)"
            )));
        }
        Ok(())
    }

    // --- the submission ----------------------------------------------------

    /// `Playthrough::Turn#play` with a request token: this game's lines
    /// accepted before this one and still owed a finish are played first, in
    /// the order they were accepted, and then this one. A line already
    /// played is answered with what it produced, and plays nothing again.
    pub fn play(&mut self, line: &str, token: &str) -> Result<Turned, Error> {
        let store = self.m.store;
        let mine = command::accept(store, &mut self.m.records, self.m.playthrough, line, token)?;
        if command::overtaken(&self.m.records, &mine) {
            return Ok(turned(command::produced(&mine), false));
        }
        let mut outcome = Turned::default();
        for row in command::accepted_up_to(&self.m.records, &mine) {
            let played = self.take_turn(id(&row))?;
            if id(&row) == id(&mine) {
                outcome = played;
            }
        }
        Ok(outcome)
    }

    /// `#take_turn` through `Playthrough::Command#execute!`.
    fn take_turn(&mut self, submission: i64) -> Result<Turned, Error> {
        self.safety = false;
        let row = self.m.row("playthrough_commands", submission)?;
        let status = text(&row, "status").unwrap_or("pending").to_string();
        if status == "completed" {
            return Ok(turned(command::produced(&row), false));
        }
        if status == "running" && !command::recoverable(&row) {
            return Err(Error::Interrupted);
        }
        if status == "failed" && !command::recoverable(&row) {
            if text(&row, "error_kind") == Some("crisis") {
                return Err(model(Failure::Crisis(
                    "a crisis answer was suppressed".into(),
                )));
            }
            return Err(Error::PreviouslyFailed);
        }
        let journal = match row.get("journal").filter(|j| j.get("version").is_some()) {
            Some(journal) => journal.clone(),
            None => json!({ "version": 1, "steps": {} }),
        };
        self.m.update(
            "playthrough_commands",
            submission,
            vec![("status", Value::from("running")), ("journal", journal)],
        )?;
        self.journal = Some(Journal::open(
            &self.m.row("playthrough_commands", submission)?,
        ));
        let line = string(&row, "command").to_string();
        let result = self.remember("outcome", played_encoded, |turn| turn.play_serially(&line));
        self.journal = None;
        match result {
            Ok(played) => {
                let mut values = vec![
                    ("status", Value::from("completed")),
                    ("error_kind", Value::Null),
                ];
                let mut crisis = false;
                match &played {
                    Played::Scene(told) => {
                        values.push(("result_scene_id", Value::from(told.id)));
                        if told.setup {
                            values.push(("error_kind", Value::from("setup")));
                        }
                        if told.safety || self.safety {
                            crisis = true;
                            values.push(("error_kind", Value::from("crisis")));
                        }
                    }
                    Played::Refused(refusal) => {
                        values.push(("refusal", command::refusal_columns(refusal)));
                    }
                    Played::Nothing => {}
                }
                self.m.update("playthrough_commands", submission, values)?;
                let row = self.m.row("playthrough_commands", submission)?;
                Ok(turned(command::produced(&row), crisis))
            }
            Err(error) => {
                let crisis = matches!(&error, Error::Model(failure) if failure.crisis());
                self.m.update(
                    "playthrough_commands",
                    submission,
                    vec![
                        ("status", Value::from("failed")),
                        (
                            "error_kind",
                            Value::from(if crisis { "crisis" } else { "error" }),
                        ),
                    ],
                )?;
                Err(error)
            }
        }
    }

    // --- the turn ----------------------------------------------------------

    /// `#play_serially`.
    fn play_serially(&mut self, command: &str) -> Result<Played, Error> {
        let ended = self.commit(
            "already_over",
            |r: &Option<Refusal>| r.as_ref().map_or(nil(), encode::refusal),
            |turn| Ok(turn.m.over().then(|| turn.m.over_refusal(command))),
        )?;
        if let Some(refusal) = ended {
            return Ok(Played::Refused(refusal));
        }
        self.commit("world_clock", |_| nil(), |turn| turn.m.catch_up_world())?;
        self.commit(
            "starting_room",
            |_| nil(),
            |turn| {
                let here = turn.m.here();
                turn.m.snapshot_room(here.as_ref())
            },
        )?;

        let typed = grammar::unslashed(command);
        let (intent, resolved_by) =
            self.remember("intent", intent_encoded, |turn| turn.read_line(command))?;

        let refusal = self.commit(
            "refusal",
            |r: &Option<Refusal>| r.as_ref().map_or(nil(), encode::refusal),
            |turn| {
                let room = turn.m.room();
                Ok(turn.m.refusal_for(&intent, &typed, &room))
            },
        )?;
        if let Some(refusal) = refusal {
            return Ok(Played::Refused(refusal));
        }

        let from = self.commit(
            "origin",
            |here: &Option<Row>| {
                here.as_ref()
                    .map_or(nil(), |room| encode::record("locations", id(room)))
            },
            |turn| Ok(turn.m.here()),
        )?;
        let round = self.commit(
            "round",
            |round: &i64| Value::from(*round),
            |turn| Ok(turn.m.next_round()),
        )?;
        self.m.round = round;

        let scene = if let Some(choice) = &intent.physical {
            self.use_physical(choice, &typed)?
        } else if let Some(Record::Place(place)) = &intent.destination {
            Some(self.move_to(place.id)?)
        } else if let Some(Record::Person(person)) = &intent.speaker {
            if intent.action == "attack" {
                let target = person.id;
                self.commit(
                    "attack",
                    |_| nil(),
                    |turn| turn.m.attack(target, None).map(|_| ()),
                )?;
                None
            } else {
                return Err(Error::Unsupported(
                    "a conversation through the models".into(),
                ));
            }
        } else if let Some(Record::Thing(thing)) = &intent.item {
            match intent.action.as_str() {
                "throw" => return Err(Error::Unsupported("a throw through the models".into())),
                "examine" => self.read_item(thing.id, &typed)?,
                "drop" => self.drop_item(thing.id, &typed)?,
                _ => self.take_item(thing.id, &typed)?,
            }
        } else {
            self.narrate(&typed, None, Some(intent.action.as_str()), None, None)?
        };

        self.commit(
            "scene_facts",
            |_| nil(),
            |turn| {
                if let Some(told) = &scene {
                    turn.scene_facts(told.id, &typed, &resolved_by, &intent)?;
                }
                Ok(())
            },
        )?;
        self.safety = self.safety || scene.as_ref().is_some_and(|told| told.safety);
        self.commit(
            "told_tolls",
            |n: &i64| Value::from(*n),
            |turn| turn.claim_tolls(scene.as_ref()),
        )?;
        self.commit(
            "told_volitions",
            |n: &i64| Value::from(*n),
            |turn| turn.claim_volitions(scene.as_ref()),
        )?;
        self.commit(
            "riposte",
            |_| nil(),
            |turn| turn.m.riposte(from.as_ref()).map(|_| ()),
        )?;
        self.commit(
            "volition",
            |_| nil(),
            |turn| turn.m.volitions(from.as_ref()).map(|_| ()),
        )?;
        self.commit(
            "room_hazard",
            |_| nil(),
            |turn| match &from {
                Some(room) => turn.m.standing(room, "every_turn"),
                None => Ok(()),
            },
        )?;
        let endings_before = self.m.game().own("playthrough_endings").len();
        self.commit("arc", |_| nil(), |turn| turn.m.run_arc().map(|_| ()))?;
        if self.m.game().own("playthrough_endings").len() > endings_before {
            return Err(Error::Unsupported(
                "telling an ending through the models".into(),
            ));
        }
        let closing = self.commit(
            "fight_closed",
            |closed: &Option<i64>| closed.map_or(nil(), |scene| encode::record("scenes", scene)),
            |turn| {
                Ok(turn
                    .m
                    .close_fight()?
                    .and_then(|_| int(turn.m.game().row, "current_scene_id")))
            },
        )?;

        let outcome = match (scene, closing) {
            (Some(told), _) => Played::Scene(told),
            (None, Some(closed)) => Played::Scene(Told::plain(closed)),
            (None, None) => Played::Nothing,
        };
        Ok(match outcome {
            Played::Scene(mut told) => {
                told.safety = told.safety || self.safety;
                Played::Scene(told)
            }
            other => other,
        })
    }

    /// `#read_line`: the grammar reads a slashed line first; everything it
    /// cannot place goes to the classifier.
    fn read_line(&mut self, command: &str) -> Result<(Intent, String), Error> {
        let room = self.m.room();
        let reading = Grammar::new(&room).reading_first(command);
        if let Some(reading) = reading {
            if reading.resolved() || reading.intent.as_ref().is_some_and(|i| i.action == "use") {
                return Ok((
                    reading.intent.expect("a resolved reading"),
                    "grammar".into(),
                ));
            }
        }
        Err(Error::Unsupported(
            "reading a line through the classifier".into(),
        ))
    }

    /// The columns only this place has on every branch: the line, who was
    /// there, which reader answered and what the turn did to what.
    fn scene_facts(
        &mut self,
        scene: i64,
        typed: &str,
        resolved_by: &str,
        intent: &Intent,
    ) -> Result<(), Error> {
        let (kind, subject) = match intent.subject() {
            Some(Record::Place(place)) => (Value::from("Location"), Value::from(place.id)),
            Some(Record::Person(person)) => (Value::from("Character"), Value::from(person.id)),
            Some(Record::Thing(thing)) => (Value::from("Item"), Value::from(thing.id)),
            _ => (Value::Null, Value::Null),
        };
        self.m.update(
            "scenes",
            scene,
            vec![
                ("typed", Value::from(typed)),
                ("resolved_by", Value::from(resolved_by)),
                ("resolved_action", Value::from(intent.action.as_str())),
                ("acted_on_type", kind),
                ("acted_on_id", subject),
            ],
        )?;
        let row = self.m.row("scenes", scene)?;
        let here = int(&row, "location_id")
            .and_then(|room| self.m.records.find("locations", room).cloned())
            .or_else(|| self.m.here());
        let cast: Vec<i64> = match &here {
            Some(room) => self
                .m
                .game()
                .cast_in(Some(room))
                .iter()
                .map(|who| id(who))
                .collect(),
            None => Vec::new(),
        };
        self.cast_scene(scene, &cast)
    }

    /// Sets who a scene has in it, keeping the rows already right.
    fn cast_scene(&mut self, scene: i64, cast: &[i64]) -> Result<(), Error> {
        let have: Vec<i64> = self
            .m
            .records
            .select("characters_scenes", |row| {
                int(row, "scene_id") == Some(scene)
            })
            .iter()
            .filter_map(|row| int(row, "character_id"))
            .collect();
        for who in have.iter().filter(|who| !cast.contains(who)) {
            self.m.store.connection().execute(
                "DELETE FROM characters_scenes WHERE scene_id = ?1 AND character_id = ?2",
                [scene, *who],
            )?;
            let mut rows = self.m.records.table("characters_scenes").to_vec();
            rows.retain(|row| {
                !(int(row, "scene_id") == Some(scene) && int(row, "character_id") == Some(*who))
            });
            self.m.records.set_table("characters_scenes", rows);
        }
        for who in cast.iter().filter(|who| !have.contains(who)) {
            self.m.insert(
                "characters_scenes",
                vec![
                    ("character_id", Value::from(*who)),
                    ("scene_id", Value::from(scene)),
                ],
            )?;
        }
        Ok(())
    }

    /// `#claim_tolls!`: the tolls the scene's paragraph told are told.
    fn claim_tolls(&mut self, scene: Option<&Told>) -> Result<i64, Error> {
        let Some(scene) = scene else {
            return Ok(0);
        };
        let tolls: Vec<i64> = self
            .m
            .game()
            .own("playthrough_tolls")
            .into_iter()
            .filter(|toll| int(toll, "scene_id").is_none())
            .map(id)
            .filter(|toll| scene.tolls.as_ref().is_none_or(|told| told.contains(toll)))
            .collect();
        for toll in &tolls {
            self.m.update(
                "playthrough_tolls",
                *toll,
                vec![("scene_id", Value::from(scene.id))],
            )?;
        }
        Ok(tolls.len() as i64)
    }

    /// `#claim_volitions!`.
    fn claim_volitions(&mut self, scene: Option<&Told>) -> Result<i64, Error> {
        let Some(scene) = scene else {
            return Ok(0);
        };
        let acts: Vec<i64> = self
            .m
            .game()
            .own("playthrough_volitions")
            .into_iter()
            .filter(|act| int(act, "scene_id").is_none())
            .map(id)
            .collect();
        for act in &acts {
            self.m.update(
                "playthrough_volitions",
                *act,
                vec![("scene_id", Value::from(scene.id))],
            )?;
        }
        Ok(acts.len() as i64)
    }

    // --- the branches ------------------------------------------------------

    fn use_physical(&mut self, choice: &Choice, command: &str) -> Result<Option<Told>, Error> {
        if choice.kind == "offer" {
            return Err(Error::Unsupported("an offer through the models".into()));
        }
        let (_, fact) = self.commit(
            "physical_effect",
            |(status, fact): &(&'static str, String)| {
                encode::data(
                    "Playthrough::PhysicalAction::Result",
                    vec![
                        ("status", Value::from(*status)),
                        ("fact", Value::from(fact.as_str())),
                    ],
                )
            },
            |turn| turn.m.apply_physical(choice),
        )?;
        self.narrate(command, Some(fact.clone()), Some("use"), None, Some(fact))
    }

    fn take_item(&mut self, item: i64, command: &str) -> Result<Option<Told>, Error> {
        let row = self.m.row("items", item)?;
        let from = self.m.here();
        let taker = self.m.player();
        self.commit(
            "take",
            |_| Value::Bool(true),
            |turn| turn.m.take(item, None).map(|_| ()),
        )?;
        let fact = taken_fact(&row, taker.as_ref(), from.as_ref());
        self.narrate(
            command,
            Some(fact),
            None,
            Some(Handled {
                item,
                direction: Direction::Taken,
            }),
            Some(format!("You pick up {}.", definite_name(&row))),
        )
    }

    fn drop_item(&mut self, item: i64, command: &str) -> Result<Option<Told>, Error> {
        let row = self.m.row("items", item)?;
        let here = self
            .m
            .here()
            .ok_or_else(|| Error::Database("a drop with nowhere to stand".into()))?;
        let dropper = self.m.player();
        self.commit(
            "drop",
            |_| Value::Bool(true),
            |turn| turn.m.drop(item, None).map(|_| ()),
        )?;
        let fact = dropped_fact(&row, &here, dropper.as_ref());
        self.narrate(
            command,
            Some(fact),
            None,
            Some(Handled {
                item,
                direction: Direction::Dropped,
            }),
            Some(format!(
                "You put down {} in {}.",
                definite_name(&row),
                string(&here, "name")
            )),
        )
    }

    /// `#read_item`: what is written on a thing, out of the records.
    fn read_item(&mut self, item: i64, command: &str) -> Result<Option<Told>, Error> {
        let row = self.m.row("items", item)?;
        if !flag(&row, "readable") {
            return self.narrate(command, None, None, None, None);
        }
        let Some(words) = presence(text(&row, "inscription")).map(str::to_string) else {
            return Err(Error::Unsupported(
                "writing an inscription through the models".into(),
            ));
        };
        let fact = format!(
            "{} has writing on it. {}",
            upcase_first(&definite_name(&row)),
            written_words_fact(&words)
        );
        let fallback = format!("On {} you read: {words}", definite_name(&row));
        self.narrate(command, Some(fact), None, None, Some(fallback))
    }

    /// `#move_to`: the room realized, walked into at its way in, paid for,
    /// and the arrival told.
    fn move_to(&mut self, destination: i64) -> Result<Told, Error> {
        let mut realizers = Vec::new();
        realizers.extend(self.realize(destination)?);
        let entry = self.m.way_in(destination);
        if entry != destination {
            realizers.extend(self.realize(entry)?);
        }
        let destination = entry;
        let room = self.m.row("locations", destination)?;
        if text(&room, "detail_level") != Some("realized") {
            return Err(Error::Database(format!(
                "{} was not written out, so there is nowhere to stand",
                string(&room, "name")
            )));
        }

        self.commit(
            "destination_snapshot",
            |_| nil(),
            |turn| turn.m.snapshot_room(Some(&room)),
        )?;
        self.commit(
            "arrival_cost",
            |_| nil(),
            |turn| {
                let from = turn.m.here();
                turn.m.on_arrival(&room, from.as_ref())
            },
        )?;

        let told = match self.arrive(destination) {
            Ok(told) => told,
            Err(Error::Model(failure)) => self.arrive_without_prose(destination, Some(failure))?,
            Err(other) => return Err(other),
        };
        self.commit("moved", Told::encoded, |turn| {
            turn.m.stand_the_party_in(destination)?;
            turn.m.update(
                "playthroughs",
                turn.m.playthrough,
                vec![("current_scene_id", Value::from(told.id))],
            )?;
            Ok(told.clone())
        })?;
        for agent in &realizers {
            let mut book = Book {
                store: self.m.store,
                records: &mut self.m.records,
            };
            self.models.attribute(&mut book, agent, told.id)?;
        }
        let description = string(&self.m.row("scenes", told.id)?, "description").to_string();
        (self.on_chunk)(&description);
        Ok(told)
    }

    /// What the arrival writer is told, and what the scene it writes keeps:
    /// the request, the time, the cast, the facts and the untold tolls.
    fn arrival(&self, destination: i64) -> Result<ArrivalInputs, Error> {
        let records = &self.m.records;
        let location = records
            .find("locations", destination)
            .ok_or_else(|| Error::Database(format!("locations {destination} is not there")))?;
        let game = Game::new(records, self.m.playthrough);
        let arrival = Arrival {
            records,
            location,
            previous_scene: game.current_scene(),
            game: Some(game),
            opening: false,
        };
        let cast: Vec<i64> = arrival
            .characters_present()
            .iter()
            .map(|who| id(who))
            .collect();
        let facts = arrival.facts().unwrap_or_default();
        let tolls: Vec<i64> = game
            .own("playthrough_tolls")
            .into_iter()
            .filter(|toll| int(toll, "scene_id").is_none())
            .map(id)
            .collect();
        Ok((
            arrival.request(),
            arrival.story_timestamp().round() as i64,
            cast,
            facts,
            tolls,
        ))
    }

    /// `Scene::Generator#generate!`.
    fn arrive(&mut self, destination: i64) -> Result<Told, Error> {
        let (request, at, cast, facts, tolls) = self.arrival(destination)?;
        let mut agent = Agent::new(self.filed("arrival"));
        let call = Call::from_request(&request);
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        let answer = self
            .models
            .ask(&mut book, &mut agent, &call, None, None)
            .map_err(model)?;
        let field = |key: &str| sanitize(answer.content[key].as_str().unwrap_or_default());
        let (description, summary) = (field("description"), field("summary"));
        let told = self.commit("arrival", Told::encoded, |turn| {
            let scene = turn.persist_arrival(
                destination,
                &description,
                &summary,
                at,
                &cast,
                &facts,
                false,
            )?;
            Ok(Told {
                tolls: Some(tolls),
                ..Told::plain(scene)
            })
        })?;
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        self.models.attribute(&mut book, &agent, told.id)?;
        Ok(told)
    }

    /// `Scene::Generator#fallback!`: the arrival in the engine's own words.
    fn arrive_without_prose(
        &mut self,
        destination: i64,
        failure: Option<Failure>,
    ) -> Result<Told, Error> {
        let (_, at, cast, facts, tolls) = self.arrival(destination)?;
        let name = string(&self.m.row("locations", destination)?, "name").to_string();
        let description = std::iter::once(format!("You arrive at {name}."))
            .chain(facts.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ");
        let safety = failure.as_ref().is_some_and(Failure::crisis);
        let setup = failure == Some(Failure::NoModel);
        self.commit("arrival", Told::encoded, |turn| {
            let scene = turn.persist_arrival(
                destination,
                &description,
                &description,
                at,
                &cast,
                &facts,
                true,
            )?;
            Ok(Told {
                id: scene,
                tolls: Some(tolls),
                safety,
                setup,
            })
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn persist_arrival(
        &mut self,
        destination: i64,
        description: &str,
        summary: &str,
        at: i64,
        cast: &[i64],
        facts: &[String],
        fallback: bool,
    ) -> Result<i64, Error> {
        let mut values = vec![
            ("location_id", Value::from(destination)),
            ("description", Value::from(description)),
            ("summary", Value::from(summary)),
            (
                "engine_fact",
                if facts.is_empty() {
                    Value::Null
                } else {
                    Value::from(facts.join("\n"))
                },
            ),
            ("is_opening", Value::Bool(false)),
            ("story_timestamp", Value::from(at)),
        ];
        if fallback {
            values.push(("engine_fallback", Value::Bool(true)));
        }
        self.m.write_scene(values, cast)
    }

    /// `Scene::Narrator#narrate`: prose for the line, streamed as it comes,
    /// or the engine's own sentence when there is one and the call failed.
    fn narrate(
        &mut self,
        command: &str,
        fact: Option<String>,
        doing: Option<&str>,
        handled: Option<Handled>,
        fallback: Option<String>,
    ) -> Result<Option<Told>, Error> {
        let prompt = self.narration_prompt(command, fact.as_deref(), doing, handled);
        let call = Call {
            system: Some(data::narrator_instructions().to_string()),
            ..Call::prompt(prompt)
        };
        let mut agent = Agent::new(self.filed("narration"));
        let asked = {
            let on_chunk = &mut *self.on_chunk;
            let mut book = Book {
                store: self.m.store,
                records: &mut self.m.records,
            };
            self.models
                .ask(&mut book, &mut agent, &call, None, Some(on_chunk))
        };
        let (mut text, mut failed) = match asked {
            Ok(answer) => (answer.text().to_string(), None),
            Err(failure) => match &fallback {
                Some(words) => (words.clone(), Some(failure)),
                None => return Err(model(failure)),
            },
        };
        if is_blank(&text) {
            match &fallback {
                Some(words) => {
                    text = words.clone();
                    failed = Some(Failure::Rejected("Narration was blank".into()));
                }
                None => return Err(model(Failure::Rejected("Narration was blank".into()))),
            }
        }
        let used_fallback = failed.is_some();
        let safety = failed.as_ref().is_some_and(Failure::crisis);
        let setup = failed == Some(Failure::NoModel);
        let told = self.commit(
            "narrated",
            |told: &Told| told.encoded(),
            |turn| {
                let here = turn.m.here().map(|room| id(&room));
                let at = turn.m.story_now() + ACTION_SECONDS;
                let mut values = vec![
                    ("location_id", here.map_or(Value::Null, Value::from)),
                    ("description", Value::from(text.as_str())),
                    (
                        "engine_fact",
                        fact.as_deref().map_or(Value::Null, Value::from),
                    ),
                    ("engine_fallback", Value::Bool(used_fallback)),
                    ("story_timestamp", Value::from(at)),
                ];
                values.retain(|(column, value)| !(*column == "location_id" && value.is_null()));
                let scene = turn.m.write_scene(values, &[])?;
                turn.m.update(
                    "playthroughs",
                    turn.m.playthrough,
                    vec![("current_scene_id", Value::from(scene))],
                )?;
                Ok(Told {
                    id: scene,
                    tolls: used_fallback.then(Vec::new),
                    safety,
                    setup,
                })
            },
        )?;
        let mut book = Book {
            store: self.m.store,
            records: &mut self.m.records,
        };
        self.models.attribute(&mut book, &agent, told.id)?;
        Ok(Some(told))
    }

    /// `Scene::Narrator#prompt_for`.
    fn narration_prompt(
        &self,
        command: &str,
        fact: Option<&str>,
        doing: Option<&str>,
        handled: Option<Handled>,
    ) -> String {
        let moment = Moment {
            game: self.m.game(),
            handled,
            ending: None,
        };
        let context = moment.narration_context(true, true);
        let fact = match fact.filter(|fact| !is_blank(fact)) {
            Some(fact) => format!(
                "\nWhat has ALREADY happened, recorded by the game: {fact}\nNarrate it as done. Do not contradict it and do not undo it.\n"
            ),
            None => String::new(),
        };
        let doing = match doing.and_then(data::narrator_doing) {
            Some(sentence) => format!("\n{sentence}\n"),
            None => String::new(),
        };
        format!(
            "{context}\n{fact}\n{doing}\nThe player types: {command}\n\nNarrate what happens.\n"
        )
    }
}

fn turned(produced: Produced, crisis: bool) -> Turned {
    match produced {
        Produced::Scene {
            id,
            crisis: kept,
            setup,
        } => Turned {
            scene: Some(id),
            refusal: None,
            safety_notice: kept || crisis,
            setup,
        },
        Produced::Refused(refusal) => Turned {
            refusal: Some(refusal),
            ..Turned::default()
        },
        Produced::Nothing => Turned::default(),
    }
}

fn played_encoded(played: &Played) -> Value {
    match played {
        Played::Scene(told) => told.encoded(),
        Played::Refused(refusal) => encode::refusal(refusal),
        Played::Nothing => Value::Null,
    }
}

fn record_encoded(record: &Option<Record>) -> Value {
    match record {
        None => Value::Null,
        Some(Record::Place(place)) => encode::record("locations", place.id),
        Some(Record::Person(person)) => encode::record("characters", person.id),
        Some(Record::Thing(thing)) => encode::record("items", thing.id),
        Some(Record::Attempt(choice)) => choice_encoded(choice),
    }
}

fn choice_encoded(choice: &Choice) -> Value {
    let thing = |thing: &Option<crate::room::Thing>| {
        thing
            .as_ref()
            .map_or(Value::Null, |t| encode::record("items", t.id))
    };
    encode::data(
        "Playthrough::PhysicalAction::Choice",
        vec![
            ("kind", Value::from(choice.kind.as_str())),
            ("item", thing(&choice.item)),
            (
                "recipient",
                choice
                    .recipient
                    .as_ref()
                    .map_or(Value::Null, |p| encode::record("characters", p.id)),
            ),
            (
                "connection",
                choice.connection.as_ref().map_or(Value::Null, |exit| {
                    encode::record("location_connections", exit.edge)
                }),
            ),
            ("tool", thing(&choice.tool)),
        ],
    )
}

/// The reading, as `Playthrough::Classifier::Intent` and the reader's name.
fn intent_encoded((intent, resolved_by): &(Intent, String)) -> Value {
    encode::array(vec![
        encode::data(
            "Playthrough::Classifier::Intent",
            vec![
                ("action", encode::symbol(&intent.action)),
                ("destination", record_encoded(&intent.destination)),
                ("speaker", record_encoded(&intent.speaker)),
                ("item", record_encoded(&intent.item)),
                ("at", record_encoded(&intent.at)),
                ("also_named", record_encoded(&intent.also_named)),
                (
                    "unknown_action",
                    intent
                        .unknown_action
                        .as_deref()
                        .map_or(Value::Null, Value::from),
                ),
                (
                    "physical",
                    intent.physical.as_ref().map_or(Value::Null, choice_encoded),
                ),
            ],
        ),
        Value::from(resolved_by.as_str()),
    ])
}

/// `Item#bare_name` and `#definite_name`: "the" in front of the name, with
/// any article the name arrived with taken off first.
fn definite_name(item: &Row) -> String {
    crate::moment::definite_name(string(item, "name"))
}

fn bare_name(item: &Row) -> String {
    let name = string(item, "name");
    let definite = crate::moment::definite_name(name);
    definite
        .strip_prefix("the ")
        .unwrap_or(&definite)
        .to_string()
}

fn written_words_fact(words: &str) -> String {
    format!(
        "This is exactly what is written on it, word for word: \"{words}\" -- those are \
         the words on it, and they do not change between readings. Quote them as they \
         are; do not add to them, and do not write different ones."
    )
}

/// `Playthrough::Turn#taken_fact`.
fn taken_fact(item: &Row, taker: Option<&Row>, from: Option<&Row>) -> String {
    let lying = match from {
        Some(room) => format!("in {}", string(room, "name")),
        None => "in this room".into(),
    };
    let description = match presence(text(item, "description")) {
        Some(description) => format!(" -- {description}"),
        None => String::new(),
    };
    let inscribed = match presence(text(item, "inscription")).filter(|_| flag(item, "readable")) {
        Some(words) => format!(" {}", written_words_fact(words)),
        None => String::new(),
    };
    format!(
        "ON THIS TURN, and not before it, {} picked {} up. Until this turn it was NOT in their \
         hands at all: it was lying {lying}. Now they are carrying it{description}. The picking up \
         is what has just happened and it is what to narrate. Do not write it as something they \
         already had, already held, or turn out to be holding. {} is the only thing that moved: \
         nothing else was lifted, opened, drawn out or taken into anybody's hands.{inscribed}",
        taker.map_or("", |who| string(who, "fullname")),
        definite_name(item),
        upcase_first(&definite_name(item)),
    )
}

/// `Playthrough::Turn#dropped_fact`.
fn dropped_fact(item: &Row, here: &Row, dropper: Option<&Row>) -> String {
    format!(
        "ON THIS TURN, and not before it, {} put the {} down. Until this turn it WAS in their \
         hands: it is no longer carried, and it is now lying in {}, where it stays until somebody \
         picks it up. The putting down is what has just happened and it is what to narrate. Do \
         not write them picking it up or finding it. {} is the only thing that moved: nothing \
         else was lifted, opened, drawn out or taken into anybody's hands.",
        dropper.map_or("The party", |who| string(who, "fullname")),
        bare_name(item),
        string(here, "name"),
        upcase_first(&definite_name(item)),
    )
}
