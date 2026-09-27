//! One typed line played with no model at all: `Playthrough::Mechanics` in
//! its `model: false` mode, over the game's own database.
//!
//! The line is read by the fixed grammar ([`crate::grammar`]) against the
//! room the player stands in, refused ([`crate::refusal`]) or played, and
//! then the world answers exactly where the Ruby engine lets it: every foe in
//! the room the turn began in strikes, everybody else there takes a turn on
//! volition's die, the place itself takes its toll, the story's arc is read,
//! and a fight that has ended is closed with one scene.
//!
//! Every write goes to the database and to the turn's copy of the rows in the
//! same step, so what a later rule reads is what the database now says.

use crate::engine::Error;
use crate::grammar::{self, Grammar, Reading};
use crate::intent::Intent;
use crate::playthrough::{max_hp, stat_block, Condition, Game};
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::refusal::Refusal;
use crate::roll::{self, Seed};
use crate::room::{Choice, Exit, Person, Place, Record, Room, Thing};
use crate::spot;
use crate::store::Store;
use crate::{data, outcome, plan, shuffle_connections, volition, world_mechanic};
use serde_json::Value;

mod told;
pub use told::{Turn, Turned};

/// `Character::ABILITIES`, in order: a check's sequence is its place here.
pub const ABILITIES: [&str; 3] = ["strength", "dexterity", "will"];

/// `Character::CHECK_DIE`.
pub const CHECK_DIE: i64 = 20;

/// `Scene::TURN_MINUTES["action"]`: one beat in a room, in story minutes.
const ACTION_MINUTES: i64 = 5;

/// `Playthrough::Turn::SEQUENCE_OFFSET`: a blow's sequence starts past every
/// ability check's.
const SEQUENCE_OFFSET: i64 = ABILITIES.len() as i64 + 1;

/// `Location::HAZARDS`: a room's hazard, the ability that saves against it,
/// and when it is paid.
const ROOM_HAZARDS: [(&str, Option<&str>, &str); 4] = [
    ("flooded", Some("strength"), "on_arrival"),
    ("unlit", Some("dexterity"), "on_arrival"),
    ("silent", Some("will"), "every_turn"),
    ("airless", None, "every_turn"),
];

/// `LocationConnection::HAZARDS`: a doorway's hazard and its save.
const DOORWAY_HAZARDS: [(&str, &str); 2] = [("drop", "dexterity"), ("undertow", "strength")];

/// `Item::HEALING_POINTS`: what a healing thing mends when it is consumed.
const HEALING_POINTS: i64 = 8;

/// What a refused line left behind, said only in this mode
/// (`Playthrough::Mechanics::ROW_WRITTEN`).
fn row_written(kind: &str) -> Option<&'static str> {
    match kind {
        "unresolved" => Some("A Playthrough::Drift row was written."),
        "named_more_than_one" => Some("A Playthrough::Overreach row was written."),
        _ => None,
    }
}

/// `Character::Check`: one d20 against one ability.
#[derive(Clone, Debug)]
pub struct Check {
    pub ability: String,
    pub score: i64,
    pub penalty: i64,
    pub die: Option<i64>,
}

impl Check {
    pub fn target(&self) -> i64 {
        self.score - self.penalty
    }

    pub fn impossible(&self) -> bool {
        self.target() <= 0
    }

    pub fn passed(&self) -> bool {
        self.die.is_some_and(|die| die <= self.target())
    }
}

impl std::fmt::Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.die {
            None => write!(
                f,
                "{} -> {} - {} = {}, IMPOSSIBLE (no roll)",
                self.ability,
                self.score,
                self.penalty,
                self.target()
            ),
            Some(die) => write!(
                f,
                "{} -> d{CHECK_DIE}({die}) <= {} {}",
                self.ability,
                self.target(),
                if self.passed() { "PASS" } else { "FAIL" }
            ),
        }
    }
}

/// `Character#check`: none for a body with no abilities; no die is thrown
/// at a target of zero or less.
pub fn check_ability(
    character: &Row,
    ability: &str,
    penalty: i64,
    rng: &mut crate::random::Random,
) -> Option<Check> {
    if !ABILITIES.iter().all(|a| int(character, a).is_some()) {
        return None;
    }
    let score = int(character, ability)?;
    let target = score - penalty;
    let die = (target > 0).then(|| roll::die(CHECK_DIE, rng));
    Some(Check {
        ability: ability.to_string(),
        score,
        penalty,
        die,
    })
}

/// One line and what it did (`Playthrough::Mechanics::Report`, less the
/// read-out, which [`crate::outcome`] reads off the records afterwards).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub understood: Option<String>,
    pub change: Option<String>,
    pub refusal: Option<String>,
    pub note: Vec<String>,
    pub resolved_by: Option<String>,
}

impl Report {
    fn change(line: String, understood: Option<String>) -> Report {
        Report {
            understood,
            change: Some(line),
            ..Report::default()
        }
    }

    fn refuse(reason: String, understood: Option<String>) -> Report {
        Report {
            understood,
            refusal: Some(reason),
            ..Report::default()
        }
    }

    fn read(note: Vec<String>, understood: Option<String>) -> Report {
        Report {
            understood,
            note,
            ..Report::default()
        }
    }
}

/// One volition's result (`Playthrough::Volition::Result`).
struct Volition {
    status: String,
    fact: String,
}

/// One game on one database, for the length of one line.
pub struct Mechanics<'s> {
    store: &'s Store,
    records: Records,
    playthrough: i64,
    engine_refused: bool,
    round: i64,
    tolls_before: i64,
    decision: Option<String>,
    decided: bool,
}

impl<'s> Mechanics<'s> {
    /// Reads every row the loop needs, fresh.
    pub fn new(store: &'s Store, playthrough: i64) -> Result<Mechanics<'s>, Error> {
        let records = store.load()?;
        if records.find("playthroughs", playthrough).is_none() {
            return Err(Error::NoSuchPlaythrough(playthrough));
        }
        Ok(Mechanics {
            store,
            records,
            playthrough,
            engine_refused: false,
            round: 1,
            tolls_before: 0,
            decision: None,
            decided: false,
        })
    }

    pub fn records(&self) -> &Records {
        &self.records
    }

    pub fn playthrough(&self) -> i64 {
        self.playthrough
    }

    fn game(&self) -> Game<'_> {
        Game::new(&self.records, self.playthrough)
    }

    fn row(&self, table: &str, id: i64) -> Result<Row, Error> {
        self.records
            .find(table, id)
            .cloned()
            .ok_or_else(|| Error::Database(format!("{table} {id} is not there")))
    }

    fn insert(&mut self, table: &str, values: Vec<(&str, Value)>) -> Result<Row, Error> {
        let row = self.store.insert(table, &values)?;
        self.records.push(table, row.clone());
        Ok(row)
    }

    fn update(&mut self, table: &str, id: i64, values: Vec<(&str, Value)>) -> Result<(), Error> {
        let written = self.store.update(table, id, &values)?;
        for (column, value) in written {
            self.records.set(table, id, &column, value);
        }
        Ok(())
    }

    fn story_id(&self) -> i64 {
        self.game().story_id()
    }

    fn story_now(&self) -> i64 {
        self.game().story_now()
    }

    /// `Story#clock`: the latest scene's story time, or the story's start.
    fn clock(&self) -> i64 {
        let story = self.story_id();
        self.records
            .select("scenes", |scene| int(scene, "story_id") == Some(story))
            .iter()
            .filter_map(|scene| int(scene, "story_timestamp"))
            .max()
            .or_else(|| int(self.game().story(), "start_time"))
            .unwrap_or_default()
    }

    fn player(&self) -> Option<Row> {
        self.game().protagonist().cloned()
    }

    fn player_id(&self) -> Option<i64> {
        int(self.game().row, "character_id")
    }

    fn here(&self) -> Option<Row> {
        self.game().current_location().cloned()
    }

    pub fn over(&self) -> bool {
        int(self.game().row, "ended_at").is_some()
    }

    fn generator(&self, at: i64, sequence: i64, kind: i128) -> crate::random::Random {
        Seed {
            story: self.story_id().into(),
            playthrough: self.playthrough.into(),
            at: at.into(),
            sequence: sequence.into(),
            kind,
        }
        .generator()
    }

    // --- the line --------------------------------------------------------

    /// Plays one typed line with a fixed decision standing in for the
    /// model's answer on a conversation: one of the person's own choices
    /// (`none`, `give:<name or id>`, `follow`, `stop_following`,
    /// `ceasefire`), applied by the same effect writer a model's answer
    /// goes through. A line that is not a conversation is an error.
    pub fn run_deciding(&mut self, command: &str, decision: &str) -> Result<Report, Error> {
        self.decision = Some(decision.to_string());
        self.decided = false;
        let report = self.run(command);
        self.decision = None;
        match report {
            Ok(_) if !self.decided => Err(Error::Database(
                "a character decision needs a line that talks to somebody".into(),
            )),
            other => other,
        }
    }

    /// Plays one typed line.
    pub fn run(&mut self, command: &str) -> Result<Report, Error> {
        self.engine_refused = false;
        if self.over() {
            let refusal = self.over_refusal(command);
            return Ok(Report::refuse(refusal.reason(), None));
        }

        self.catch_up_world()?;
        let here = self.here();
        self.snapshot_room(here.as_ref())?;

        let room = self.room();
        let reading = Grammar::new(&room).parse(command);

        let from = self.here();
        self.round = self.next_round();
        let game = self.playthrough;
        self.tolls_before = self
            .records
            .select("playthrough_tolls", |row| {
                int(row, "playthrough_id") == Some(game)
            })
            .iter()
            .map(|row| id(row))
            .max()
            .unwrap_or(0);

        let report = if let Some(refusal) = &reading.refusal {
            Report {
                note: note_of(&reading),
                ..Report::refuse(refusal.clone(), reading.understood.clone())
            }
        } else if let Some((kind, amount)) = &reading.wound {
            self.wound(kind, *amount as i64, reading.understood.clone())?
        } else if let Some((ability, penalty)) = &reading.attempt {
            self.attempt(ability, *penalty as i64, reading.understood.clone())?
        } else if let Some(intent) = &reading.intent {
            self.act(intent, command, &room, reading.understood.clone())?
        } else {
            Report::read(note_of(&reading), None)
        };

        let report = self.answered_by_the_world(report, &reading, from.as_ref())?;
        Ok(Report {
            resolved_by: reading.resolved_by.clone(),
            ..report
        })
    }

    /// `Playthrough::Refusal.over`, out of `Playthrough::EndNotice`.
    fn over_refusal(&self, command: &str) -> Refusal {
        let game = self.playthrough;
        let concluded = self
            .records
            .first("playthrough_endings", |row| {
                int(row, "playthrough_id") == Some(game)
            })
            .is_some();
        let person = self.player().map(|row| person_of(&row));
        Refusal::over(concluded, person.as_ref(), command)
    }

    /// The room as the grammar reads it: the classifier's four closed sets.
    pub fn room(&self) -> Room {
        let game = self.game();
        let here = game.current_location();
        let story_protagonist = self.story_protagonist();
        let exits = match here {
            None => Vec::new(),
            Some(room) => self
                .exits_of(room)
                .into_iter()
                .map(|(edge, far)| Exit {
                    place: Place {
                        id: id(&far),
                        name: string(&far, "name").to_string(),
                    },
                    barrier: if self.open_for(&edge) {
                        "open".to_string()
                    } else {
                        string(&edge, "barrier").to_string()
                    },
                    edge: id(&edge),
                    key: int(&edge, "key_template_id"),
                })
                .collect(),
        };
        Room {
            protagonist: game.protagonist().map(person_of),
            here: here.map(|room| Place {
                id: id(room),
                name: string(room, "name").to_string(),
            }),
            exits,
            cast: game
                .cast_in(here)
                .into_iter()
                .filter(|who| Some(id(who)) != story_protagonist)
                .map(person_of)
                .collect(),
            lying: game
                .items_lying_in(here)
                .into_iter()
                .map(|item| thing_of(item, false))
                .collect(),
            carried: game
                .carried()
                .into_iter()
                .map(|item| thing_of(item, true))
                .collect(),
            over: self.over(),
        }
    }

    fn story_protagonist(&self) -> Option<i64> {
        let story = self.story_id();
        self.records
            .first("characters", |row| {
                int(row, "story_id") == Some(story) && flag(row, "is_protagonist")
            })
            .map(id)
    }

    /// A room's doorways and the rooms beyond them, by the far room's id.
    fn exits_of(&self, room: &Row) -> Vec<(Row, Row)> {
        let here = id(room);
        let mut ways: Vec<(Row, Row)> = self
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id") == Some(here)
            })
            .into_iter()
            .filter_map(|edge| {
                let far = self
                    .records
                    .find("locations", int(edge, "connected_location_id")?)?;
                Some((edge.clone(), far.clone()))
            })
            .collect();
        ways.sort_by_key(|(_, far)| id(far));
        ways
    }

    /// `LocationConnection.walked`: the doorway from one room into another.
    fn walked(&self, origin: Option<&Row>, destination: Option<&Row>) -> Option<Row> {
        let (origin, destination) = (id(origin?), id(destination?));
        self.records
            .first("location_connections", |edge| {
                int(edge, "location_id") == Some(origin)
                    && int(edge, "connected_location_id") == Some(destination)
            })
            .cloned()
    }

    /// `LocationConnection#open_for?`.
    fn open_for(&self, edge: &Row) -> bool {
        let story = Some(self.story_id());
        let in_story = |column: &str| {
            int(edge, column)
                .and_then(|room| self.records.find("locations", room))
                .is_some_and(|room| int(room, "story_id") == story)
        };
        if !in_story("location_id") || !in_story("connected_location_id") {
            return false;
        }
        let game = self.playthrough;
        text(edge, "barrier") == Some("open")
            || self
                .records
                .first("playthrough_passages", |row| {
                    int(row, "location_connection_id") == Some(id(edge))
                        && int(row, "playthrough_id") == Some(game)
                })
                .is_some()
    }

    // --- acting on it ------------------------------------------------------

    fn act(
        &mut self,
        intent: &Intent,
        command: &str,
        room: &Room,
        understood: Option<String>,
    ) -> Result<Report, Error> {
        if let Some(refusal) = self.refusal_for(intent, command, room) {
            self.engine_refused = true;
            let reason = [
                Some(refusal.reason()),
                row_written(&refusal.kind).map(str::to_string),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
            return Ok(Report::refuse(reason, understood));
        }

        if let Some(choice) = &intent.physical {
            return self.physical(choice, understood);
        }
        if let Some(Record::Place(destination)) = &intent.destination {
            return self.stand_in(destination.id, understood);
        }
        if let Some(Record::Person(person)) = &intent.speaker {
            if intent.action == "attack" {
                return self.attack(person.id, understood);
            }
            return self.talk(person.id, understood);
        }
        if let Some(Record::Thing(thing)) = &intent.item {
            return match intent.action.as_str() {
                "examine" => self.recite(thing.id, understood),
                "throw" => self.throw_it(thing.id, intent.at.as_ref(), understood),
                "drop" => self.drop(thing.id, understood),
                _ => self.take(thing.id, understood),
            };
        }
        Ok(Report::refuse(
            format!(
                "`{}` does not move anything: it is answered in prose, and this mode writes \
                 none. Nothing changed.",
                intent.action
            ),
            understood,
        ))
    }

    /// `Playthrough::Turn#refusal_for`: the reading first, the game and its
    /// closed doorways second.
    fn refusal_for(&self, intent: &Intent, command: &str, room: &Room) -> Option<Refusal> {
        if intent.refused() {
            return Refusal::for_intent(intent, command, &room.offered_for(&intent.action));
        }
        let destination = match (&intent.destination, &intent.at) {
            (Some(Record::Place(place)), _) => Some(place.id),
            (None, Some(Record::Place(place))) if intent.is_throw() => Some(place.id),
            _ => None,
        };
        if let Some(destination) = destination {
            let here = self.here();
            let far = self.records.find("locations", destination).cloned();
            if let Some(edge) = self.walked(here.as_ref(), far.as_ref()) {
                if !self.open_for(&edge) {
                    let fact = format!(
                        "The way to {} is {}. Open it before crossing or throwing anything \
                         through it.",
                        far.as_ref().map_or("", |room| string(room, "name")),
                        if text(&edge, "barrier") == Some("keyed") {
                            "locked"
                        } else {
                            "jammed"
                        }
                    );
                    return Refusal::new("unplayable", command, &fact, None).ok();
                }
            }
        }
        Refusal::unplayable(
            &intent.action,
            self.player_id().is_some(),
            self.here().is_some(),
            command,
        )
    }

    /// The move with no model: the row moves and the room stays unwritten.
    fn stand_in(&mut self, destination: i64, understood: Option<String>) -> Result<Report, Error> {
        let from = self.here();
        let destination = self.way_in(destination);
        let room = self.row("locations", destination)?;
        self.snapshot_room(Some(&room))?;
        self.on_arrival(&room, from.as_ref())?;
        self.stand_the_party_in(destination)?;
        let stub = text(&room, "detail_level") == Some("stub");
        Ok(Report::change(
            format!(
                "moved: {} -> {}{}",
                from.as_ref().map_or("nowhere", |room| string(room, "name")),
                string(&room, "name"),
                if stub {
                    " (a stub -- nobody has written this room, and no-model mode cannot)"
                } else {
                    ""
                }
            ),
            understood,
        ))
    }

    /// `Location::Interior.way_in`: a laid-out place is walked into at its
    /// entry room, and anything else at itself.
    fn way_in(&self, location: i64) -> i64 {
        let Some(place) = self.records.find("locations", location) else {
            return location;
        };
        let interior = int(place, "width").is_some() && int(place, "depth").is_some();
        let placed = plan::box_of(place).is_some();
        if !interior || placed {
            return location;
        }
        self.records
            .first("locations", |room| {
                int(room, "parent_location_id") == Some(location)
            })
            .map_or(location, id)
    }

    /// `Playthrough::Turn#stand_in!`: the followers walk with the party, and
    /// the party stands in the room.
    fn stand_the_party_in(&mut self, destination: i64) -> Result<(), Error> {
        let followers: Vec<i64> = {
            let game = self.game();
            let followers: Vec<i64> = game.followers().iter().map(|who| id(who)).collect();
            game.own("playthrough_npc_states")
                .into_iter()
                .filter(|row| int(row, "character_id").is_some_and(|who| followers.contains(&who)))
                .map(id)
                .collect()
        };
        for row in followers {
            self.update(
                "playthrough_npc_states",
                row,
                vec![("location_id", Value::from(destination))],
            )?;
        }
        self.update(
            "playthroughs",
            self.playthrough,
            vec![("current_location_id", Value::from(destination))],
        )
    }

    fn take(&mut self, item: i64, understood: Option<String>) -> Result<Report, Error> {
        let row = self.row("items", item)?;
        let was = int(&row, "location_id").and_then(|room| self.records.find("locations", room));
        let was = was.map_or("nowhere".to_string(), |room| {
            string(room, "name").to_string()
        });
        self.update(
            "items",
            item,
            vec![
                ("playthrough_id", Value::from(self.playthrough)),
                ("character_id", Value::Null),
                ("location_id", Value::Null),
                ("x", Value::Null),
                ("y", Value::Null),
            ],
        )?;
        let taker = self
            .player()
            .map(|who| string(&who, "fullname").to_string())
            .unwrap_or_default();
        Ok(Report::change(
            format!(
                "took: {} (was lying in {was}, now carried by {taker})",
                string(&row, "name")
            ),
            understood,
        ))
    }

    fn drop(&mut self, item: i64, understood: Option<String>) -> Result<Report, Error> {
        let row = self.row("items", item)?;
        let here = self
            .here()
            .ok_or_else(|| Error::Database("a drop with nowhere to stand".into()))?;
        self.put_down(item, &here)?;
        let dropper = self.player().map_or("the party".to_string(), |who| {
            string(&who, "fullname").to_string()
        });
        Ok(Report::change(
            format!(
                "dropped: {} (was carried by {dropper}, now lying in {})",
                string(&row, "name"),
                string(&here, "name")
            ),
            understood,
        ))
    }

    /// `Playthrough::Turn#put_down!`: this game's copy lies in a room, at a
    /// spot rolled for this game.
    fn put_down(&mut self, item: i64, room: &Row) -> Result<(), Error> {
        let spot = self.spot_in_a_game(room, item);
        self.update(
            "items",
            item,
            vec![
                ("playthrough_id", Value::from(self.playthrough)),
                ("character_id", Value::Null),
                ("location_id", Value::from(id(room))),
                ("x", spot.map_or(Value::Null, |spot| Value::from(spot.x))),
                ("y", spot.map_or(Value::Null, |spot| Value::from(spot.y))),
            ],
        )
    }

    /// `Location::Placement.in_a_game` for a thing.
    fn spot_in_a_game(&self, room: &Row, item: i64) -> Option<crate::boxes::Spot> {
        spot::place(
            int(room, "story_id").unwrap_or_default(),
            plan::box_of(room).as_ref(),
            spot::Record::Item(item),
            Some(spot::Game {
                playthrough_id: self.playthrough,
                story_now: self.story_now(),
            }),
        )
    }

    /// What is written on a thing, out of the records.
    fn recite(&mut self, item: i64, understood: Option<String>) -> Result<Report, Error> {
        let row = self.row("items", item)?;
        let definite = thing_of(&row, false).definite_name();
        if !flag(&row, "readable") {
            return Ok(Report::refuse(
                format!(
                    "there is nothing written on {definite}. Looking at something that has no \
                     writing on it is prose, and this mode writes none. Nothing changed."
                ),
                understood,
            ));
        }
        let Some(words) = text(&row, "inscription").filter(|words| !crate::text::is_blank(words))
        else {
            return Ok(Report::refuse(
                format!(
                    "{definite} has writing on it and the records do not hold the words yet. \
                     Writing them down is one model call (Item::Inscriber) and this mode makes \
                     none here. Read it in the browser once and it is a record from then on."
                ),
                understood,
            ));
        };
        Ok(Report::read(
            vec![
                format!("reads: {}", string(&row, "name")),
                format!("  {words}"),
            ],
            understood,
        ))
    }

    /// Swinging at somebody, through the engine's own writer. A body with
    /// no stat block is refused here, and the world still answers.
    fn attack(&mut self, target: i64, understood: Option<String>) -> Result<Report, Error> {
        let Some(who) = self.player() else {
            return Ok(Report::refuse(
                "this playthrough has no protagonist, so there is nobody to swing".into(),
                understood,
            ));
        };
        if !stat_block(&who) {
            return Ok(Report::refuse(
                format!(
                    "{} has no stat block, so there is no hit die to hit with. \
                     `rake game:backfill_stat_blocks` rolls one, offline",
                    string(&who, "fullname")
                ),
                understood,
            ));
        }
        let target = self.row("characters", target)?;
        let here = self
            .here()
            .ok_or_else(|| Error::Database("a blow with nowhere to stand".into()))?;
        match self.strike(&who, &target, &here, None)? {
            Some(blow) => Ok(Report::change(format!("struck: {blow}"), understood)),
            None => Ok(Report::refuse(
                format!(
                    "{} has no stat block, so there is no body to hurt. \
                     `rake game:backfill_stat_blocks` rolls one, offline",
                    string(&target, "fullname")
                ),
                understood,
            )),
        }
    }

    /// Throwing something, through `Playthrough::Turn#throw_item!`: a
    /// strength check less the thing's bulk, and where it lands. A failed
    /// lift is a played turn with nothing moved, not a refusal.
    fn throw_it(
        &mut self,
        item: i64,
        at: Option<&Record>,
        understood: Option<String>,
    ) -> Result<Report, Error> {
        let (Some(thrower), Some(at)) = (self.player(), at) else {
            return Err(Error::Database("a throw with no thrower or no aim".into()));
        };
        let row = self.row("items", item)?;
        let name = string(&row, "name").to_string();
        let target = at.label();
        let thrown_die = match text(&row, "bulk") {
            Some("light") => Some((0, 4)),
            Some("handy") => Some((2, 6)),
            Some("heavy") => Some((5, 8)),
            _ => None,
        };
        let Some((penalty, die)) = thrown_die else {
            let attempt = format!("threw: {name} at {target}");
            return Ok(Report::read(
                vec![
                    attempt,
                    "it does not move for anybody, so no die was thrown".into(),
                ],
                understood,
            ));
        };
        let mut rng = self.generator(self.story_now(), item, roll::THROW);
        let Some(check) = check_ability(&thrower, "strength", penalty, &mut rng) else {
            return Ok(Report::refuse(
                format!(
                    "{} has no abilities, so there is no strength to throw with. \
                     `rake game:backfill_stat_blocks` rolls them, offline",
                    string(&thrower, "fullname")
                ),
                understood,
            ));
        };
        let attempt = format!("threw: {name} at {target} -- {check}");
        if !check.passed() {
            return Ok(Report::read(
                vec![
                    attempt,
                    "the lift failed: nothing was thrown and no row moved".into(),
                ],
                understood,
            ));
        }
        let landed = match at {
            Record::Person(person) => {
                let here = self
                    .here()
                    .ok_or_else(|| Error::Database("a throw with nowhere to stand".into()))?;
                self.put_down(item, &here)?;
                let damage = roll::die(die, &mut rng);
                let target_row = self.row("characters", person.id)?;
                let blow = self.strike(&thrower, &target_row, &here, Some(damage))?;
                let game = self.playthrough;
                let struck = blow.and_then(|_| {
                    self.records
                        .table("playthrough_blows")
                        .iter()
                        .rev()
                        .find(|row| int(row, "playthrough_id") == Some(game))
                        .cloned()
                });
                match struck {
                    Some(blow) => {
                        let condition = Condition {
                            hp: int(&blow, "hp_after").unwrap_or_default(),
                            max: max_hp(&target_row).unwrap_or_default(),
                        };
                        format!(
                            "it hit {who} for {} and is lying at their feet; {who} is {}",
                            int(&blow, "damage").unwrap_or_default(),
                            condition.in_words(),
                            who = person.fullname
                        )
                    }
                    None => format!(
                        "it hit {} and is lying at their feet; there is no stat block to hurt",
                        person.fullname
                    ),
                }
            }
            Record::Place(place) => {
                let room = self.row("locations", place.id)?;
                self.put_down(item, &room)?;
                format!("it went through and is lying in {}", place.name)
            }
            _ => {
                return Err(Error::Database(
                    "a throw at something that is not an aim".into(),
                ))
            }
        };
        Ok(Report {
            understood,
            change: Some(landed),
            note: vec![attempt],
            ..Report::default()
        })
    }

    /// A physical attempt through `Playthrough::PhysicalAction#apply!`. A
    /// failed check is a played turn with nothing changed; an attempt the
    /// room no longer offers is refused, and the world does not answer.
    fn physical(&mut self, choice: &Choice, understood: Option<String>) -> Result<Report, Error> {
        if choice.kind == "offer" {
            let recipient = choice.recipient.as_ref().map_or(0, |person| person.id);
            return self.talk(recipient, understood);
        }
        let (status, fact) = self.apply_physical(choice)?;
        match status {
            "applied" => Ok(Report::change(fact, understood)),
            "failed" => Ok(Report::read(vec![fact], understood)),
            _ => {
                self.engine_refused = true;
                Ok(Report::refuse(fact, understood))
            }
        }
    }

    /// `Playthrough::PhysicalAction#apply!`: the attempt, if the room still
    /// offers it, and what happened, as its status (`applied`, `failed` or
    /// `rejected`) and the fact the engine states.
    pub(crate) fn apply_physical(
        &mut self,
        choice: &Choice,
    ) -> Result<(&'static str, String), Error> {
        let offered = self.room().physical_actions();
        let Some(choice) = offered.into_iter().find(|offer| offer == choice) else {
            return Ok((
                "rejected",
                "That physical action is unavailable. No item or passage changed.".into(),
            ));
        };
        let item = choice.item.as_ref().map(|thing| thing.id);
        match choice.kind.as_str() {
            "consume" => {
                let item = item.unwrap_or_default();
                let row = self.row("items", item)?;
                let healing = if text(&row, "use_kind") == Some("healing") {
                    HEALING_POINTS
                } else {
                    0
                };
                let player = self.player();
                let before = player
                    .as_ref()
                    .and_then(|who| self.condition_of(who))
                    .map(|c| c.hp);
                if before.is_none() && healing > 0 {
                    return Ok((
                        "rejected",
                        "Your condition is unavailable, so the healing item was not consumed."
                            .into(),
                    ));
                }
                if let Some(who) = &player {
                    self.mend(who, healing)?;
                }
                let gained = match (
                    before,
                    player.as_ref().and_then(|who| self.condition_of(who)),
                ) {
                    (Some(before), Some(after)) => after.hp - before,
                    _ => 0,
                };
                self.spend(item, "consumed")?;
                Ok((
                    "applied",
                    format!(
                        "You consumed {}. It is gone from your possessions. You recovered \
                         {gained} hit points.",
                        string(&row, "name")
                    ),
                ))
            }
            "burn" => {
                let item = item.unwrap_or_default();
                let name = string(&self.row("items", item)?, "name").to_string();
                self.spend(item, "burned")?;
                let tool = choice
                    .tool
                    .as_ref()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                Ok((
                    "applied",
                    format!(
                        "You burned {name} using {tool}. The burned item is gone; you still \
                         carry {tool}."
                    ),
                ))
            }
            "unlock" | "pick" | "pry" | "force" => self.open_passage(&choice),
            _ => Ok((
                "rejected",
                "The item remains in your hands until its recipient accepts it.".into(),
            )),
        }
    }

    /// `PhysicalAction#spend!`: a thing used up leaves every place.
    fn spend(&mut self, item: i64, disposition: &str) -> Result<(), Error> {
        self.update(
            "items",
            item,
            vec![
                ("disposition", Value::from(disposition)),
                ("character_id", Value::Null),
                ("location_id", Value::Null),
                ("x", Value::Null),
                ("y", Value::Null),
            ],
        )
    }

    /// `PhysicalAction#open_passage!`: a key opens a lock outright; picking,
    /// prising and forcing each take a check. The way opens both ways, for
    /// this game only, and nobody crosses it.
    fn open_passage(&mut self, choice: &Choice) -> Result<(&'static str, String), Error> {
        let Some(edge) = &choice.connection else {
            return Err(Error::Database("a passage with no doorway".into()));
        };
        let check = if choice.kind == "unlock" {
            None
        } else {
            let ability = if choice.kind == "pick" {
                "dexterity"
            } else {
                "strength"
            };
            let penalty = if choice.kind == "force" { 4 } else { 0 };
            match self.player() {
                Some(who) => self.check(&who, ability, penalty),
                None => None,
            }
        };
        if choice.kind != "unlock" && !check.as_ref().is_some_and(Check::passed) {
            let told = check.as_ref().map_or(String::new(), ToString::to_string);
            return Ok((
                "failed",
                format!(
                    "You tried to {}. {told}. The way remains closed; you stay here.",
                    crate::text::ruby_downcase(&choice.name())
                ),
            ));
        }
        let means = match choice.kind.as_str() {
            "unlock" => "key",
            "pick" => "lockpick",
            "pry" => "lever",
            _ => "force",
        };
        let forward = self.row("location_connections", edge.edge)?;
        let reverse = self
            .records
            .first("location_connections", |row| {
                int(row, "location_id") == int(&forward, "connected_location_id")
                    && int(row, "connected_location_id") == int(&forward, "location_id")
            })
            .cloned();
        let game = self.playthrough;
        for doorway in [Some(forward), reverse].into_iter().flatten() {
            let opened = self
                .records
                .first("playthrough_passages", |row| {
                    int(row, "playthrough_id") == Some(game)
                        && int(row, "location_connection_id") == Some(id(&doorway))
                })
                .is_some();
            if opened {
                continue;
            }
            self.insert(
                "playthrough_passages",
                vec![
                    ("playthrough_id", Value::from(game)),
                    ("location_connection_id", Value::from(id(&doorway))),
                    ("opened_at", Value::from(self.story_now())),
                    ("means", Value::from(means)),
                    (
                        "opened_by_item_id",
                        choice
                            .tool
                            .as_ref()
                            .map_or(Value::Null, |tool| Value::from(tool.id)),
                    ),
                ],
            )?;
        }
        let here = self
            .here()
            .map_or(String::new(), |room| string(&room, "name").to_string());
        Ok((
            "applied",
            format!(
                "You opened the way to {}.{} You have not crossed it; you remain in {here}.",
                edge.place.name,
                check.map_or(String::new(), |check| format!(" {check}."))
            ),
        ))
    }

    /// Talking is prose, and this mode writes none, unless a fixed decision
    /// stands in for the person's answer.
    fn talk(&mut self, character: i64, understood: Option<String>) -> Result<Report, Error> {
        let person = self.row("characters", character)?;
        if let Some(decision) = self.decision.clone() {
            self.decided = true;
            return self.decide_for(&person, &decision, understood);
        }
        Ok(Report::refuse(
            format!(
                "{} is here and the classifier resolved them, but talking is prose and \
                 this mode writes none. Nothing changed. Play the browser game to speak to \
                 somebody.",
                string(&person, "fullname")
            ),
            understood,
        ))
    }

    /// `Playthrough::NpcAction#choices`: token => sentence, in order.
    fn npc_choices(&self, character: &Row) -> Vec<(String, String)> {
        let mut available = vec![(
            "none".to_string(),
            "Speak without transferring anything or changing an agreement.".to_string(),
        )];
        let game = self.game();
        let here = game.current_location();
        let present = !self.over()
            && Some(id(character)) != self.player_id()
            && int(character, "story_id") == Some(self.story_id())
            && game
                .cast_in(here)
                .iter()
                .any(|who| id(who) == id(character));
        if !present {
            return available;
        }
        let player = self
            .player()
            .map(|who| string(&who, "fullname").to_string());
        let party = player.clone().unwrap_or_else(|| "the player".to_string());
        if let Some(player) = &player {
            for item in game.items_held_by(character) {
                available.push((
                    format!("give:{}", id(item)),
                    format!("Give {} to {player}.", string(item, "name")),
                ));
            }
        }
        let foe = game
            .foes_in(here)
            .iter()
            .any(|who| id(who) == id(character));
        let state = game
            .own("playthrough_npc_states")
            .into_iter()
            .find(|row| int(row, "character_id") == Some(id(character)));
        let following = match state {
            Some(row) => flag(row, "following"),
            None => flag(character, "is_companion"),
        };
        if foe {
            available.push((
                "ceasefire".into(),
                format!("Stop fighting {party}; another attack can break the truce."),
            ));
        } else if following {
            available.push((
                "stop_following".into(),
                format!(
                    "Stay in {} when the player leaves.",
                    here.map_or("", |room| string(room, "name"))
                ),
            ));
        } else {
            available.push((
                "follow".into(),
                format!("Accompany {party} when they leave this room."),
            ));
        }
        available
    }

    /// `EngineSweep::Conversation#talk` and `Playthrough::NpcAction#apply!`:
    /// the decision applied, then reported as a change, a refusal or a note.
    fn decide_for(
        &mut self,
        character: &Row,
        decision: &str,
        understood: Option<String>,
    ) -> Result<Report, Error> {
        let who = string(character, "fullname").to_string();
        let mut choice = decision.to_string();
        if let Some(name) = decision.strip_prefix("give:") {
            if let Some(item) = self
                .game()
                .items_held_by(character)
                .into_iter()
                .find(|item| string(item, "name") == name)
            {
                choice = format!("give:{}", id(item));
            }
        }
        if choice == "none" {
            return Ok(Report::read(
                vec![format!(
                    "{who} changes no possessions, travel agreement or ceasefire."
                )],
                understood,
            ));
        }
        if !self
            .npc_choices(character)
            .iter()
            .any(|(token, _)| *token == choice)
        {
            return Ok(Report::refuse(
                "The proposed action was rejected: it is unavailable. No possessions, travel \
                 agreement or ceasefire changed."
                    .into(),
                understood,
            ));
        }
        let party = self.player().map_or("the player".to_string(), |row| {
            string(&row, "fullname").to_string()
        });
        let here = self.here();
        let here_id = here.as_ref().map(id);
        let here_name = here
            .as_ref()
            .map_or(String::new(), |room| string(room, "name").to_string());
        let fact = match choice.split_once(':') {
            Some(("give", item)) => {
                let item = item.parse::<i64>().unwrap_or_default();
                let name = string(&self.row("items", item)?, "name").to_string();
                self.update(
                    "items",
                    item,
                    vec![
                        ("character_id", Value::Null),
                        ("location_id", Value::Null),
                        ("x", Value::Null),
                        ("y", Value::Null),
                    ],
                )?;
                format!("{who} gave {name} to {party}; the player now carries it.")
            }
            _ => {
                let at = self.location_of(character).or(here_id);
                let state = self.npc_state(character, at)?;
                match choice.as_str() {
                    "follow" => {
                        self.update(
                            "playthrough_npc_states",
                            id(&state),
                            vec![
                                ("following", Value::Bool(true)),
                                ("location_id", here_id.map_or(Value::Null, Value::from)),
                            ],
                        )?;
                        format!("{who} is now accompanying {party} and will travel with them.")
                    }
                    "stop_following" => {
                        self.update(
                            "playthrough_npc_states",
                            id(&state),
                            vec![
                                ("following", Value::Bool(false)),
                                ("location_id", here_id.map_or(Value::Null, Value::from)),
                            ],
                        )?;
                        format!("{who} stopped accompanying the player and remains in {here_name}.")
                    }
                    _ => {
                        let player = self.player_id();
                        let target = id(character);
                        let last = self
                            .game()
                            .own("playthrough_blows")
                            .into_iter()
                            .filter(|blow| {
                                int(blow, "attacker_id") == player
                                    && player.is_some()
                                    && int(blow, "target_id") == Some(target)
                            })
                            .map(id)
                            .max()
                            .unwrap_or(0);
                        self.update(
                            "playthrough_npc_states",
                            id(&state),
                            vec![
                                ("ceasefire", Value::Bool(true)),
                                ("peace_after_blow_id", Value::from(last)),
                            ],
                        )?;
                        format!(
                            "{who} stopped fighting the player. The ceasefire holds unless the \
                             player attacks again."
                        )
                    }
                }
            }
        };
        Ok(Report::change(fact, understood))
    }

    /// Hurting or mending the player, through the engine's own writers.
    fn wound(
        &mut self,
        kind: &str,
        amount: i64,
        understood: Option<String>,
    ) -> Result<Report, Error> {
        let Some(who) = self.player() else {
            return Ok(Report::refuse(
                format!("this playthrough has no protagonist, so there is no body to {kind}"),
                understood,
            ));
        };
        let fullname = string(&who, "fullname").to_string();
        let Some(before) = self.game().vitals_for(&who) else {
            return Ok(Report::refuse(
                format!(
                    "{fullname} has no stat block, so there is nothing to {kind}. \
                     `rake game:backfill_stat_blocks` rolls one, offline"
                ),
                understood,
            ));
        };
        let harm = kind == "harm";
        let after = if harm {
            self.harm(&who, amount)?
        } else {
            self.mend(&who, amount)?
        }
        .expect("a body with a stat block has a condition");
        Ok(Report::change(
            format!(
                "{}: {fullname} {} -> {}{}{}",
                if harm { "harmed" } else { "mended" },
                before.in_words(),
                after.in_words(),
                if self.over() {
                    ". THIS PLAYTHROUGH IS OVER -- every line from here is refused"
                } else {
                    ""
                },
                if after.dead() && !harm {
                    " (nothing changed: death is terminal and a mend never raises the dead)"
                } else {
                    ""
                }
            ),
            understood,
        ))
    }

    /// One d20 against one of the player's abilities. It writes nothing.
    fn attempt(
        &mut self,
        ability: &str,
        penalty: i64,
        understood: Option<String>,
    ) -> Result<Report, Error> {
        let Some(who) = self.player() else {
            return Ok(Report::refuse(
                "this playthrough has no protagonist, so there is nobody to check".into(),
                understood,
            ));
        };
        let fullname = string(&who, "fullname").to_string();
        let Some(result) = self.check(&who, ability, penalty) else {
            return Ok(Report::refuse(
                format!(
                    "{fullname} has no abilities, so there is nothing to check. \
                     `rake game:backfill_stat_blocks` rolls them, offline"
                ),
                understood,
            ));
        };
        if result.impossible() {
            return Ok(Report::refuse(
                format!(
                    "{fullname} cannot do it at all: {ability} {} less a penalty of {} is {}, \
                     and no d{CHECK_DIE} comes up that low. No die was thrown.",
                    result.score,
                    result.penalty,
                    result.target()
                ),
                understood,
            ));
        }
        Ok(Report::read(vec![format!("check {result}")], understood))
    }

    /// `Playthrough::Turn#check`: seeded off the story's clock and the
    /// ability's place in the list.
    fn check(&self, character: &Row, ability: &str, penalty: i64) -> Option<Check> {
        let sequence = ABILITIES
            .iter()
            .position(|a| *a == ability)
            .map_or(0, |index| index as i64 + 1);
        let mut rng = self.generator(self.clock(), sequence, 0);
        check_ability(character, ability, penalty, &mut rng)
    }

    // --- bodies --------------------------------------------------------------

    /// `Playthrough::Vitals.instantiate!`: this game's row for a body, made
    /// at full health the first time. None for a body with no stat block.
    fn vitals_row(&mut self, character: &Row) -> Result<Option<Row>, Error> {
        if !stat_block(character) {
            return Ok(None);
        }
        let game = self.playthrough;
        let who = id(character);
        if let Some(row) = self.records.first("playthrough_vitals", |row| {
            int(row, "playthrough_id") == Some(game) && int(row, "character_id") == Some(who)
        }) {
            return Ok(Some(row.clone()));
        }
        let row = self.insert(
            "playthrough_vitals",
            vec![
                ("playthrough_id", Value::from(game)),
                ("character_id", Value::from(who)),
                ("hp_current", Value::from(max_hp(character))),
            ],
        )?;
        Ok(Some(row))
    }

    fn condition_of(&self, character: &Row) -> Option<Condition> {
        self.game().vitals_for(character)
    }

    /// `Playthrough::Turn#harm!`: the last hit point, the body letting go of
    /// what it held, and the game's end if it was the player's.
    fn harm(&mut self, character: &Row, amount: i64) -> Result<Option<Condition>, Error> {
        let Some(row) = self.vitals_row(character)? else {
            return Ok(None);
        };
        let hp = int(&row, "hp_current").unwrap_or_default();
        let after = (hp - amount).max(0);
        if after != hp {
            self.update(
                "playthrough_vitals",
                id(&row),
                vec![("hp_current", Value::from(after))],
            )?;
        }
        if after <= 0 {
            self.keep_npc_body(character)?;
            self.spill(character)?;
            if Some(id(character)) == self.player_id() {
                self.end_game(self.story_now())?;
            }
        }
        Ok(self.condition_of(character))
    }

    /// `Playthrough::Turn#mend!`: a mend never raises the dead.
    fn mend(&mut self, character: &Row, amount: i64) -> Result<Option<Condition>, Error> {
        let Some(row) = self.vitals_row(character)? else {
            return Ok(None);
        };
        let hp = int(&row, "hp_current").unwrap_or_default();
        if hp <= 0 {
            return Ok(self.condition_of(character));
        }
        let after = (hp + amount).min(max_hp(character).unwrap_or(hp));
        if after != hp {
            self.update(
                "playthrough_vitals",
                id(&row),
                vec![("hp_current", Value::from(after))],
            )?;
        }
        Ok(self.condition_of(character))
    }

    /// `Playthrough#end!`.
    fn end_game(&mut self, at: i64) -> Result<(), Error> {
        if self.over() {
            return Ok(());
        }
        self.update(
            "playthroughs",
            self.playthrough,
            vec![("ended_at", Value::from(at))],
        )
    }

    /// `Playthrough#location_of`.
    fn location_of(&self, character: &Row) -> Option<i64> {
        if int(character, "story_id") != Some(self.story_id()) {
            return None;
        }
        let game = self.playthrough;
        let who = id(character);
        if let Some(state) = self.records.first("playthrough_npc_states", |row| {
            int(row, "playthrough_id") == Some(game) && int(row, "character_id") == Some(who)
        }) {
            return int(state, "location_id");
        }
        if Some(who) == self.player_id() || flag(character, "is_companion") {
            return int(self.game().row, "current_location_id");
        }
        int(character, "location_id")
    }

    /// `Playthrough#keep_npc_body!`: a body stays where it fell, in this game.
    fn keep_npc_body(&mut self, character: &Row) -> Result<(), Error> {
        if Some(id(character)) == self.player_id() || flag(character, "is_protagonist") {
            return Ok(());
        }
        let location = self.location_of(character);
        let row = self.npc_state(character, location)?;
        self.update(
            "playthrough_npc_states",
            id(&row),
            vec![
                ("location_id", location.map_or(Value::Null, Value::from)),
                ("following", Value::Bool(false)),
            ],
        )
    }

    /// This game's row for somebody, made where they stand the first time
    /// (`find_or_create_by!`).
    fn npc_state(&mut self, character: &Row, location: Option<i64>) -> Result<Row, Error> {
        let game = self.playthrough;
        let who = id(character);
        if let Some(row) = self.records.first("playthrough_npc_states", |row| {
            int(row, "playthrough_id") == Some(game) && int(row, "character_id") == Some(who)
        }) {
            return Ok(row.clone());
        }
        self.insert(
            "playthrough_npc_states",
            vec![
                ("playthrough_id", Value::from(game)),
                ("character_id", Value::from(who)),
                ("location_id", location.map_or(Value::Null, Value::from)),
            ],
        )
    }

    /// `Playthrough::Turn#spill!`: what a body held lies where it fell.
    fn spill(&mut self, character: &Row) -> Result<(), Error> {
        let Some(room) = self.location_of(character) else {
            return Ok(());
        };
        let room = self.row("locations", room)?;
        let held: Vec<i64> = self
            .game()
            .items_held_by(character)
            .iter()
            .map(|item| id(item))
            .collect();
        for item in held {
            self.put_down(item, &room)?;
        }
        Ok(())
    }

    // --- the world's turn ----------------------------------------------------

    /// `Story#catch_up_world!`: the mechanics that have come due, then every
    /// scheduled event the clock has passed.
    fn catch_up_world(&mut self) -> Result<(), Error> {
        let now = self.clock();
        let story = self.story_id();
        let start = self.start_time();
        let mechanics: Vec<Row> = self
            .records
            .select("world_mechanics", |row| int(row, "story_id") == Some(story))
            .into_iter()
            .cloned()
            .collect();
        for mechanic in mechanics {
            let Some(cadence) = text(&mechanic, "cadence").and_then(world_mechanic::cadence) else {
                continue;
            };
            let Some(from) = int(&mechanic, "last_run_at").or(start) else {
                continue;
            };
            for at in cadence.pending_boundaries(from, now) {
                if text(&mechanic, "kind") != Some("shuffle_connections") {
                    return Err(Error::Unsupported(format!(
                        "the world mechanic {}",
                        string(&mechanic, "kind")
                    )));
                }
                self.shuffle_connections(&mechanic, at)?;
                self.update(
                    "world_mechanics",
                    id(&mechanic),
                    vec![("last_run_at", Value::from(at))],
                )?;
            }
        }
        let mut due: Vec<(i64, i64)> = self
            .records
            .select("world_events", |row| {
                int(row, "story_id") == Some(story)
                    && int(row, "fired_at").is_none()
                    && int(row, "scheduled_for").is_some_and(|at| at <= now)
            })
            .iter()
            .map(|row| (int(row, "scheduled_for").unwrap_or_default(), id(row)))
            .collect();
        due.sort();
        for (_, event) in due {
            self.update("world_events", event, vec![("fired_at", Value::from(now))])?;
        }
        Ok(())
    }

    /// `WorldMechanic::ShuffleConnections#run!`: the mobile rooms' doorways
    /// onto anchored rooms turned, each doorway keeping its own state and
    /// this world's games keeping the passages they opened, and one event
    /// saying what moved.
    fn shuffle_connections(&mut self, mechanic: &Row, at: i64) -> Result<(), Error> {
        let story = self.story_id();
        let mut locations: Vec<(i64, bool, String)> = self
            .records
            .select("locations", |row| int(row, "story_id") == Some(story))
            .iter()
            .map(|row| {
                (
                    id(row),
                    flag(row, "mobile"),
                    string(row, "name").to_string(),
                )
            })
            .collect();
        locations.sort_by_key(|(room, _, _)| *room);
        let ids: Vec<i64> = locations.iter().map(|(room, _, _)| *room).collect();
        let doorways: Vec<Row> = self
            .records
            .select("location_connections", |row| {
                int(row, "location_id").is_some_and(|room| ids.contains(&room))
            })
            .into_iter()
            .cloned()
            .collect();
        let graph = shuffle_connections::Graph {
            story_id: story,
            locations: locations
                .iter()
                .map(|(room, mobile, _)| (*room, *mobile))
                .collect(),
            connections: doorways
                .iter()
                .map(|row| {
                    (
                        int(row, "location_id").unwrap_or_default(),
                        int(row, "connected_location_id").unwrap_or_default(),
                    )
                })
                .collect(),
        };
        let edges = graph.anchor_edges();
        if edges.len() < 2 {
            return Ok(());
        }
        let Some(arrangement) = graph.choose_arrangement(&edges, at) else {
            return Ok(());
        };
        let rows: Vec<Row> = edges
            .iter()
            .map(|&(from, to)| {
                doorways
                    .iter()
                    .find(|row| {
                        int(row, "location_id") == Some(from)
                            && int(row, "connected_location_id") == Some(to)
                    })
                    .cloned()
                    .expect("an anchor edge is a doorway")
            })
            .collect();
        let moves: Vec<(Row, i64, i64)> = rows
            .into_iter()
            .zip(arrangement)
            .filter_map(|(edge, to)| {
                let from = int(&edge, "connected_location_id")?;
                (from != to).then_some((edge, from, to))
            })
            .collect();
        if moves.is_empty() {
            return Ok(());
        }

        let states: Vec<[Doorway; 2]> = moves
            .iter()
            .map(|(edge, _, _)| self.doorway_state(edge))
            .collect();
        for (edge, from, _) in &moves {
            let room = int(edge, "location_id").unwrap_or_default();
            self.remove_edge(room, *from)?;
        }
        for ((edge, _, to), state) in moves.iter().zip(states) {
            let room = int(edge, "location_id").unwrap_or_default();
            for ((a, b), doorway) in [(room, *to), (*to, room)].into_iter().zip(state) {
                let mut values = doorway.values;
                let time = travel_time(&values);
                values.push(("location_id", Value::from(a)));
                values.push(("connected_location_id", Value::from(b)));
                values.push(("time_to_travel", time));
                let written = id(&self.insert("location_connections", values)?);
                for mut opening in doorway.openings {
                    opening.push(("location_connection_id", Value::from(written)));
                    self.insert("playthrough_passages", opening)?;
                }
            }
        }

        let name = |room: i64| {
            locations
                .iter()
                .find(|(id, _, _)| *id == room)
                .map_or("somewhere unrecorded".to_string(), |(_, _, name)| {
                    name.clone()
                })
        };
        let summary = moves
            .iter()
            .map(|(edge, from, to)| {
                format!(
                    "{} now opens onto {} instead of {}.",
                    name(int(edge, "location_id").unwrap_or_default()),
                    name(*to),
                    name(*from)
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let event = self.insert(
            "world_events",
            vec![
                ("world_mechanic_id", Value::from(id(mechanic))),
                ("story_id", Value::from(story)),
                ("source", Value::from("world_mechanic")),
                ("occurred_at", Value::from(at)),
                ("summary", Value::from(summary)),
            ],
        )?;
        let mut touched: Vec<i64> = moves
            .iter()
            .flat_map(|(edge, from, to)| [int(edge, "location_id").unwrap_or_default(), *from, *to])
            .collect();
        touched.sort();
        touched.dedup();
        for room in touched {
            self.insert(
                "locations_world_events",
                vec![
                    ("location_id", Value::from(room)),
                    ("world_event_id", Value::from(id(&event))),
                ],
            )?;
        }
        Ok(())
    }

    /// `ShuffleConnections#doorway_state`: what a doorway and its way back
    /// carry, and the passages every game opened through them. A way back
    /// that was never written is made from the doorway, with no hazard.
    fn doorway_state(&self, edge: &Row) -> [Doorway; 2] {
        let reverse = self
            .records
            .first("location_connections", |row| {
                int(row, "location_id") == int(edge, "connected_location_id")
                    && int(row, "connected_location_id") == int(edge, "location_id")
            })
            .cloned();
        [Some(edge.clone()), reverse].map(|row| {
            let written = row.is_some();
            let source = row.unwrap_or_else(|| edge.clone());
            let mut columns: Vec<&str> =
                vec!["distance", "travel_method", "barrier", "key_template_id"];
            if written {
                columns.extend(["id", "created_at", "hazard", "hazard_die"]);
            }
            let values = columns
                .into_iter()
                .map(|column| (column, source.get(column).cloned().unwrap_or(Value::Null)))
                .collect();
            let openings = self
                .records
                .select("playthrough_passages", |passage| {
                    int(passage, "location_connection_id") == Some(id(&source))
                })
                .into_iter()
                .map(|passage| {
                    PASSAGE_COLUMNS
                        .iter()
                        .filter(|column| written || **column != "id")
                        .map(|column| {
                            (
                                *column,
                                passage.get(*column).cloned().unwrap_or(Value::Null),
                            )
                        })
                        .collect()
                })
                .collect();
            Doorway { values, openings }
        })
    }

    /// `ShuffleConnections#remove_edge`: a doorway both ways gone, its tolls
    /// kept with no doorway, its passages with it.
    fn remove_edge(&mut self, room: i64, far: i64) -> Result<(), Error> {
        let rows: Vec<i64> = self
            .records
            .select("location_connections", |row| {
                let pair = (int(row, "location_id"), int(row, "connected_location_id"));
                pair == (Some(room), Some(far)) || pair == (Some(far), Some(room))
            })
            .into_iter()
            .map(id)
            .collect();
        for row in rows {
            let tolls: Vec<i64> = self
                .records
                .select("playthrough_tolls", |toll| {
                    int(toll, "location_connection_id") == Some(row)
                })
                .into_iter()
                .map(id)
                .collect();
            for toll in tolls {
                self.update(
                    "playthrough_tolls",
                    toll,
                    vec![("location_connection_id", Value::Null)],
                )?;
            }
            let passages: Vec<i64> = self
                .records
                .select("playthrough_passages", |passage| {
                    int(passage, "location_connection_id") == Some(row)
                })
                .into_iter()
                .map(id)
                .collect();
            for passage in passages {
                self.records.remove("playthrough_passages", passage);
            }
            self.store.delete("location_connections", row)?;
            self.records.remove("location_connections", row);
        }
        Ok(())
    }

    /// `Playthrough::Fight#next_round`.
    fn next_round(&self) -> i64 {
        self.game()
            .own("playthrough_blows")
            .into_iter()
            .filter(|blow| int(blow, "scene_id").is_none())
            .filter_map(|blow| int(blow, "round"))
            .max()
            .unwrap_or(0)
            + 1
    }

    /// `Playthrough::Mechanics#answered_by_the_world`.
    fn answered_by_the_world(
        &mut self,
        report: Report,
        reading: &Reading,
        from: Option<&Row>,
    ) -> Result<Report, Error> {
        let Some(intent) = &reading.intent else {
            return Ok(report);
        };
        if intent.refused() || self.engine_refused {
            return Ok(report);
        }

        let blows = self.riposte(from)?;
        let volitions = self.volitions(from)?;
        if let Some(room) = from {
            self.standing(room, "every_turn")?;
        }
        let beats = self.run_arc()?;
        let closing = self.close_fight()?;
        let tolls_before = self.tolls_before;
        let taken: Vec<String> = {
            let view = self.game();
            view.own("playthrough_tolls")
                .into_iter()
                .filter(|toll| id(toll) > tolls_before)
                .map(|toll| view.toll_to_s(toll))
                .collect()
        };
        let ended = if self.over() {
            self.ending_of_the_arc()
        } else {
            None
        };
        let acts: Vec<&Volition> = volitions.iter().filter(|v| v.status == "applied").collect();
        if blows.is_empty()
            && closing.is_none()
            && taken.is_empty()
            && beats.is_empty()
            && ended.is_none()
            && acts.is_empty()
        {
            return Ok(report);
        }

        let mut notes = report.note.clone();
        notes.extend(blows.iter().map(|blow| format!("answered: {blow}")));
        notes.extend(acts.iter().map(|act| format!("someone here: {}", act.fact)));
        notes.extend(taken.iter().map(|toll| format!("the world: {toll}")));
        notes.extend(beats.iter().map(|beat| format!("the story: {beat}")));
        if let Some(closing) = closing {
            notes.push(format!("the fight is over: {closing}"));
        }
        if let Some(ended) = ended {
            notes.push(format!("the story is over: {ended}"));
        }
        Ok(Report {
            note: notes,
            ..report
        })
    }

    /// `Playthrough::Riposte#run!`: every live foe in the room the turn
    /// began in strikes the player once.
    fn riposte(&mut self, location: Option<&Row>) -> Result<Vec<String>, Error> {
        let (Some(location), Some(target)) = (location, self.player()) else {
            return Ok(Vec::new());
        };
        if self.over() {
            return Ok(Vec::new());
        }
        let foes: Vec<Row> = self
            .game()
            .foes_in(Some(location))
            .into_iter()
            .cloned()
            .collect();
        let mut blows = Vec::new();
        for foe in foes {
            if self.over() {
                break;
            }
            if let Some(blow) = self.strike(&foe, &target, location, None)? {
                blows.push(blow);
            }
        }
        Ok(blows)
    }

    /// `Playthrough::Turn#strike!`: one blow, seeded off the story's time
    /// and how many blows this game has seen. None when either body has no
    /// stat block. Returns the blow as the engine says it.
    fn strike(
        &mut self,
        attacker: &Row,
        target: &Row,
        room: &Row,
        damage: Option<i64>,
    ) -> Result<Option<String>, Error> {
        if !stat_block(attacker) {
            return Ok(None);
        }
        let sequence = SEQUENCE_OFFSET + self.game().own("playthrough_blows").len() as i64;
        let damage = damage.unwrap_or_else(|| {
            let mut rng = self.generator(self.story_now(), sequence, 0);
            roll::die(int(attacker, "hit_die").unwrap_or_default(), &mut rng)
        });
        let Some(after) = self.harm(target, damage)? else {
            return Ok(None);
        };
        self.provoke(target)?;
        let round = self.round;
        self.insert(
            "playthrough_blows",
            vec![
                ("playthrough_id", Value::from(self.playthrough)),
                ("attacker_id", Value::from(id(attacker))),
                ("target_id", Value::from(id(target))),
                ("location_id", Value::from(id(room))),
                ("damage", Value::from(damage)),
                ("hp_after", Value::from(after.hp)),
                ("round", Value::from(round)),
                ("sequence", Value::from(sequence)),
                ("story_timestamp", Value::from(self.story_now())),
            ],
        )?;
        Ok(Some(blow_to_s(
            string(attacker, "fullname"),
            target,
            damage,
            after.hp,
            round,
        )))
    }

    /// `Playthrough::Turn#provoke!`: this game remembers who was struck.
    fn provoke(&mut self, character: &Row) -> Result<(), Error> {
        let Some(row) = self.vitals_row(character)? else {
            return Ok(());
        };
        if int(&row, "provoked_at").is_none() {
            self.update(
                "playthrough_vitals",
                id(&row),
                vec![("provoked_at", Value::from(self.story_now()))],
            )?;
        }
        Ok(())
    }

    /// `Playthrough::Volition.run!`: everybody else in the room the turn
    /// began in takes a turn on volition's die. Nothing here asks a model,
    /// so every decision is the die's.
    fn volitions(&mut self, location: Option<&Row>) -> Result<Vec<Volition>, Error> {
        let Some(location) = location else {
            return Ok(Vec::new());
        };
        if self.over() {
            return Ok(Vec::new());
        }
        let cast: Vec<Row> = {
            let game = self.game();
            let fighting: Vec<i64> = game
                .foes_in(Some(location))
                .iter()
                .map(|who| id(who))
                .collect();
            let player = self.player_id();
            let mut cast: Vec<&Row> = game
                .cast_in(Some(location))
                .into_iter()
                .filter(|who| {
                    Some(id(who)) != player
                        && !flag(who, "is_protagonist")
                        && !fighting.contains(&id(who))
                        && weights_for(text(who, "desire_pursuit")).is_some()
                })
                .collect();
            cast.sort_by_key(|who| id(who));
            cast.into_iter().cloned().collect()
        };
        let mut results = Vec::new();
        for who in cast {
            if let Some(result) = self.decide(&who, location)? {
                results.push(result);
            }
        }
        Ok(results)
    }

    /// `Playthrough::Volition#decide!`: one weighted draw over the acts on
    /// offer, seeded off the story's time and who is choosing.
    fn decide(&mut self, character: &Row, location: &Row) -> Result<Option<Volition>, Error> {
        let Some(row) = weights_for(text(character, "desire_pursuit")) else {
            return Ok(None);
        };
        let choices = volition::choices(&self.game(), character, location);
        if choices.is_empty() {
            return Ok(None);
        }
        let base = data::weights().base;
        let weights: Vec<i64> = choices
            .iter()
            .map(|(token, _)| base + weight_of(row, shape_of(token)))
            .collect();
        let mut rng = self.generator(self.story_now(), id(character), roll::VOLITION);
        let chosen = choices[roll::weighted_one_of(choices.len(), &weights, &mut rng)]
            .0
            .clone();
        self.apply_volition(character, location, &chosen).map(Some)
    }

    /// `Playthrough::Volition#apply!`: the act, if it is still on offer, and
    /// its record either way.
    fn apply_volition(
        &mut self,
        character: &Row,
        location: &Row,
        chosen: &str,
    ) -> Result<Volition, Error> {
        let who = string(character, "fullname").to_string();
        let here = string(location, "name").to_string();
        if chosen == volition::WAIT {
            let fact = format!("{who} stayed in {here} and changed nothing.");
            return self.record_volition(character, location, chosen, "none", fact, "none");
        }
        let offered = volition::choices(&self.game(), character, location);
        if !offered.iter().any(|(token, _)| token == chosen) {
            let fact = format!(
                "{who} was going to act and could not: the act is no longer available. \
                 Nothing moved."
            );
            return self.record_volition(character, location, chosen, "rejected", fact, "none");
        }
        let player = self
            .player()
            .map(|row| string(&row, "fullname").to_string())
            .unwrap_or_default();
        let (shape, target) = match chosen.split_once(':') {
            Some((shape, target)) => (shape, target.parse::<i64>().ok()),
            None => (chosen, None),
        };
        let fact = match (shape, target) {
            ("move", Some(way)) => {
                let destination = self.row("locations", way)?;
                let state = self.volition_state(character, location)?;
                self.update(
                    "playthrough_npc_states",
                    id(&state),
                    vec![
                        ("location_id", Value::from(way)),
                        ("following", Value::Bool(false)),
                    ],
                )?;
                format!(
                    "{who} walked out of {here} to {} and is no longer in {here}.",
                    string(&destination, "name")
                )
            }
            ("take", Some(item)) => {
                let name = string(&self.row("items", item)?, "name").to_string();
                self.update(
                    "items",
                    item,
                    vec![
                        ("character_id", Value::from(id(character))),
                        ("location_id", Value::Null),
                        ("x", Value::Null),
                        ("y", Value::Null),
                    ],
                )?;
                format!("{who} picked up {name} in {here} and now holds it.")
            }
            ("give", Some(item)) => {
                let name = string(&self.row("items", item)?, "name").to_string();
                self.update(
                    "items",
                    item,
                    vec![
                        ("character_id", Value::Null),
                        ("location_id", Value::Null),
                        ("x", Value::Null),
                        ("y", Value::Null),
                    ],
                )?;
                format!("{who} handed {name} to {player}; the player now carries it.")
            }
            ("follow", None) => {
                let state = self.volition_state(character, location)?;
                let party = int(self.game().row, "current_location_id");
                self.update(
                    "playthrough_npc_states",
                    id(&state),
                    vec![
                        ("following", Value::Bool(true)),
                        ("location_id", party.map_or(Value::Null, Value::from)),
                    ],
                )?;
                format!("{who} decided to go with {player} and will travel with them.")
            }
            ("stop_following", None) => {
                let state = self.volition_state(character, location)?;
                self.update(
                    "playthrough_npc_states",
                    id(&state),
                    vec![
                        ("following", Value::Bool(false)),
                        ("location_id", Value::from(id(location))),
                    ],
                )?;
                format!("{who} stopped accompanying the player and remains in {here}.")
            }
            _ => return Err(Error::Database(format!("volition offered {chosen}"))),
        };
        let serves = weights_for(text(character, "desire_pursuit"))
            .map_or(0, |row| weight_of(row, shape))
            > 0;
        self.record_volition(
            character,
            location,
            chosen,
            "applied",
            fact,
            if serves { "conscious" } else { "none" },
        )
    }

    /// `Volition#state!`: this game's row for somebody, made where the game
    /// last saw them.
    fn volition_state(&mut self, character: &Row, location: &Row) -> Result<Row, Error> {
        let at = self.location_of(character).or(Some(id(location)));
        self.npc_state(character, at)
    }

    fn record_volition(
        &mut self,
        character: &Row,
        location: &Row,
        chosen: &str,
        status: &str,
        fact: String,
        serves: &str,
    ) -> Result<Volition, Error> {
        self.insert(
            "playthrough_volitions",
            vec![
                ("playthrough_id", Value::from(self.playthrough)),
                ("character_id", Value::from(id(character))),
                ("location_id", Value::from(id(location))),
                ("chosen", Value::from(chosen)),
                ("status", Value::from(status)),
                ("fact", Value::from(fact.clone())),
                ("serves", Value::from(serves)),
                ("round", Value::from(self.round)),
                ("decided_by", Value::from("die")),
            ],
        )?;
        Ok(Volition {
            status: status.to_string(),
            fact,
        })
    }

    /// `Playthrough::Hazards#on_arrival!`: the doorway walked, then the room.
    fn on_arrival(&mut self, destination: &Row, from: Option<&Row>) -> Result<(), Error> {
        if let Some(edge) = self.walked(from, Some(destination)) {
            let save = text(&edge, "hazard")
                .and_then(|hazard| DOORWAY_HAZARDS.iter().find(|(key, _)| *key == hazard));
            if let (Some((hazard, save)), Some(die)) = (save, int(&edge, "hazard_die")) {
                self.take_toll(hazard, die, Some(save), destination, Some(id(&edge)))?;
            }
        }
        self.standing(destination, "on_arrival")
    }

    /// `Playthrough::Hazards#standing`: a room's own hazard, at its moment.
    fn standing(&mut self, room: &Row, moment: &str) -> Result<(), Error> {
        let entry = text(room, "hazard")
            .and_then(|hazard| ROOM_HAZARDS.iter().find(|(key, _, _)| *key == hazard));
        let (Some((hazard, save, when)), Some(die)) = (entry, int(room, "hazard_die")) else {
            return Ok(());
        };
        if *when != moment {
            return Ok(());
        }
        self.take_toll(hazard, die, *save, room, None)
    }

    /// `Playthrough::Hazards#take!`: a save if the hazard allows one, the die
    /// if it fails, and the toll either way.
    fn take_toll(
        &mut self,
        hazard: &str,
        die: i64,
        save: Option<&str>,
        room: &Row,
        connection: Option<i64>,
    ) -> Result<(), Error> {
        let Some(who) = self.player() else {
            return Ok(());
        };
        if self.over() {
            return Ok(());
        }
        let sequence = -(self.game().own("playthrough_tolls").len() as i64 + 1);
        let mut rng = self.generator(self.story_now(), sequence, 0);
        let check = save.and_then(|save| check_ability(&who, save, 0, &mut rng));
        let saved = check.as_ref().is_some_and(Check::passed);
        let damage = if saved { 0 } else { roll::die(die, &mut rng) };
        let Some(after) = self.harm(&who, damage)? else {
            return Ok(());
        };
        self.insert(
            "playthrough_tolls",
            vec![
                ("playthrough_id", Value::from(self.playthrough)),
                ("character_id", Value::from(id(&who))),
                ("location_id", Value::from(id(room))),
                (
                    "location_connection_id",
                    connection.map_or(Value::Null, Value::from),
                ),
                ("hazard", Value::from(hazard)),
                ("saved", Value::Bool(saved)),
                ("damage", Value::from(damage)),
                ("hp_after", Value::from(after.hp)),
                ("sequence", Value::from(sequence)),
                ("story_timestamp", Value::from(self.story_now())),
            ],
        )?;
        Ok(())
    }

    /// The story's open arcs, lowest id first (`Playthrough::Arc#quests`).
    fn open_arcs(&self) -> Vec<Row> {
        let story = self.story_id();
        self.records
            .select("quests", |row| {
                int(row, "story_id") == Some(story) && text(row, "status") == Some("open")
            })
            .into_iter()
            .cloned()
            .collect()
    }

    /// `Playthrough::Arc#main_arc`: the open arc with no parent.
    fn open_main_arc(&self) -> Option<Row> {
        self.open_arcs()
            .into_iter()
            .find(|quest| int(quest, "parent_quest_id").is_none())
    }

    fn reached_steps(&self) -> Vec<i64> {
        self.game()
            .own("playthrough_beats")
            .iter()
            .filter_map(|beat| int(beat, "quest_step_id"))
            .collect()
    }

    /// `Quest#next_step_for`: the lowest beat this game has not reached.
    fn next_step(&self, quest: &Row) -> Option<Row> {
        let reached = self.reached_steps();
        outcome::steps(&self.records, quest)
            .into_iter()
            .filter(|step| !reached.contains(&id(step)))
            .min_by_key(|step| int(step, "position"))
            .cloned()
    }

    fn start_time(&self) -> Option<i64> {
        int(self.game().story(), "start_time")
    }

    /// `Playthrough::Arc#reached?`.
    fn beat_reached(&self, step: &Row) -> bool {
        if text(step, "trigger_kind") == Some("time_passed") {
            return match (self.start_time(), int(step, "minutes")) {
                (Some(start), Some(minutes)) => self.story_now() >= start + minutes * 60,
                _ => false,
            };
        }
        let Some(target) = int(step, "target_id") else {
            return false;
        };
        match text(step, "trigger_kind") {
            Some("reach_location") => {
                text(step, "target_type") == Some("Location")
                    && self.records.find("locations", target).is_some()
                    && int(self.game().row, "current_location_id") == Some(self.way_in(target))
            }
            Some("speak_to") => {
                let Some(scene) = int(self.game().row, "current_scene_id") else {
                    return false;
                };
                self.records
                    .first("interactions", |row| {
                        int(row, "scene_id") == Some(scene)
                            && int(row, "character_id") == Some(target)
                    })
                    .is_some()
            }
            Some("hold_item") => self
                .game()
                .carried()
                .iter()
                .any(|item| int(item, "template_id") == Some(target)),
            _ => false,
        }
    }

    /// `Playthrough::Arc#run!`: the beats this line reached, by summary,
    /// and the ending if the main arc is finished.
    fn run_arc(&mut self) -> Result<Vec<String>, Error> {
        let quests = self.open_arcs();
        if quests.is_empty() {
            return Ok(Vec::new());
        }
        if self.over() {
            self.record_failure()?;
            return Ok(Vec::new());
        }
        let mut reached = self.reached_steps();
        let mut beats = Vec::new();
        for quest in &quests {
            let steps: Vec<Row> = outcome::steps(&self.records, quest)
                .into_iter()
                .cloned()
                .collect();
            for step in steps {
                if reached.contains(&id(&step)) || !self.beat_reached(&step) {
                    continue;
                }
                self.insert(
                    "playthrough_beats",
                    vec![
                        ("playthrough_id", Value::from(self.playthrough)),
                        ("quest_step_id", Value::from(id(&step))),
                        ("reached_at", Value::from(self.story_now())),
                    ],
                )?;
                reached.push(id(&step));
                beats.push(string(&step, "summary").to_string());
            }
        }
        self.conclude()?;
        Ok(beats)
    }

    /// `Playthrough::Arc#satisfies?`.
    fn satisfies(&self, outcome: &Row) -> bool {
        match text(outcome, "condition") {
            Some("slower_than") => match (self.start_time(), int(outcome, "minutes")) {
                (Some(start), Some(minutes)) => self.story_now() > start + minutes * 60,
                _ => false,
            },
            Some("out_of_order") => {
                let Some(quest) =
                    int(outcome, "quest_id").and_then(|quest| self.records.find("quests", quest))
                else {
                    return false;
                };
                let steps = outcome::steps(&self.records, quest);
                let mut beats: Vec<(i64, i64, i64)> = self
                    .game()
                    .own("playthrough_beats")
                    .into_iter()
                    .filter_map(|beat| {
                        let step = steps
                            .iter()
                            .find(|step| Some(id(step)) == int(beat, "quest_step_id"))?;
                        Some((
                            int(beat, "reached_at").unwrap_or_default(),
                            id(beat),
                            int(step, "position").unwrap_or_default(),
                        ))
                    })
                    .collect();
                beats.sort();
                let positions: Vec<i64> = beats.iter().map(|(_, _, position)| *position).collect();
                let mut sorted = positions.clone();
                sorted.sort();
                positions != sorted
            }
            _ => false,
        }
    }

    /// `Playthrough::Arc#conclude!`: the main arc finished, its ending
    /// recorded, the closing scene written and the game ended.
    fn conclude(&mut self) -> Result<(), Error> {
        let Some(arc) = self.open_main_arc() else {
            return Ok(());
        };
        if self.over()
            || outcome::steps(&self.records, &arc).is_empty()
            || self.next_step(&arc).is_some()
        {
            return Ok(());
        }
        let quest = id(&arc);
        let outcomes: Vec<Row> = self
            .records
            .select("quest_outcomes", |row| int(row, "quest_id") == Some(quest))
            .into_iter()
            .cloned()
            .collect();
        let reached = outcomes
            .iter()
            .filter(|outcome| text(outcome, "condition").is_some_and(|c| !c.is_empty()))
            .find(|outcome| self.satisfies(outcome))
            .or_else(|| outcomes.iter().find(|outcome| flag(outcome, "is_default")))
            .or_else(|| outcomes.first())
            .cloned();
        let Some(reached) = reached else {
            return Ok(());
        };
        let at = self.story_now();
        let story = self.story_id();
        self.insert(
            "playthrough_endings",
            vec![
                ("playthrough_id", Value::from(self.playthrough)),
                ("quest_outcome_id", Value::from(id(&reached))),
                ("reached_at", Value::from(at)),
            ],
        )?;
        if !self.quest_event_recorded() {
            self.insert(
                "world_events",
                vec![
                    ("story_id", Value::from(story)),
                    ("playthrough_id", Value::from(self.playthrough)),
                    ("source", Value::from("quest")),
                    ("occurred_at", Value::from(at)),
                    (
                        "summary",
                        Value::from(format!(
                            "{} was finished ({}): {}",
                            string(&arc, "title"),
                            string(&reached, "name"),
                            string(&reached, "summary")
                        )),
                    ),
                ],
            )?;
        }
        let ramification = (
            int(&reached, "ramification_minutes"),
            text(&reached, "ramification_summary").filter(|words| !crate::text::is_blank(words)),
        );
        if let (Some(minutes), Some(summary)) = ramification {
            self.insert(
                "world_events",
                vec![
                    ("story_id", Value::from(story)),
                    ("source", Value::from("quest")),
                    ("occurred_at", Value::from(at)),
                    ("scheduled_for", Value::from(at + minutes * 60)),
                    ("summary", Value::from(summary)),
                ],
            )?;
        }
        let summary = string(&reached, "summary").to_string();
        let here = int(self.game().row, "current_location_id");
        let scene = self.write_scene(
            vec![
                ("location_id", here.map_or(Value::Null, Value::from)),
                ("description", Value::from(summary.clone())),
                ("summary", Value::from(summary.clone())),
                ("engine_fact", Value::from(summary)),
                ("story_timestamp", Value::from(at)),
                ("resolved_action", Value::from("conclude")),
            ],
            &[],
        )?;
        self.update(
            "playthroughs",
            self.playthrough,
            vec![("current_scene_id", Value::from(scene))],
        )?;
        self.end_game(at)
    }

    fn quest_event_recorded(&self) -> bool {
        let story = self.story_id();
        let game = self.playthrough;
        self.records
            .first("world_events", |row| {
                int(row, "story_id") == Some(story)
                    && int(row, "playthrough_id") == Some(game)
                    && text(row, "source") == Some("quest")
            })
            .is_some()
    }

    /// `Playthrough::Arc#record_failure!`: a game that ended short of its
    /// arc's ending says where it stopped, once.
    fn record_failure(&mut self) -> Result<(), Error> {
        let Some(arc) = self.open_main_arc() else {
            return Ok(());
        };
        if outcome::steps(&self.records, &arc).is_empty()
            || self.ending_of_the_arc().is_some()
            || self.quest_event_recorded()
        {
            return Ok(());
        }
        let progress = match self.next_step(&arc) {
            None => "every beat was reached and the ending was never written".to_string(),
            Some(step) => format!("it stopped at {}", string(&step, "summary")),
        };
        let at = int(self.game().row, "ended_at").unwrap_or_else(|| self.story_now());
        self.insert(
            "world_events",
            vec![
                ("story_id", Value::from(self.story_id())),
                ("playthrough_id", Value::from(self.playthrough)),
                ("source", Value::from("quest")),
                ("occurred_at", Value::from(at)),
                (
                    "summary",
                    Value::from(format!(
                        "{} was left unfinished: {progress}.",
                        string(&arc, "title")
                    )),
                ),
            ],
        )?;
        Ok(())
    }

    /// `Playthrough::Arc#ending`: this game's ending of the open main arc,
    /// as its outcome's own words (`Playthrough::Ending#to_s`).
    fn ending_of_the_arc(&self) -> Option<String> {
        let quest = id(&self.open_main_arc()?);
        self.game()
            .own("playthrough_endings")
            .into_iter()
            .find_map(|ending| {
                let outcome = self
                    .records
                    .find("quest_outcomes", int(ending, "quest_outcome_id")?)?;
                (int(outcome, "quest_id") == Some(quest))
                    .then(|| string(outcome, "summary").to_string())
            })
    }

    /// Writes a scene after the current one, with the people in it, and
    /// stamps the room's visit (`Scene#mark_location_visit`). Returns its id.
    fn write_scene(&mut self, mut values: Vec<(&str, Value)>, cast: &[i64]) -> Result<i64, Error> {
        let previous = int(self.game().row, "current_scene_id");
        let at = values
            .iter()
            .find(|(column, _)| *column == "story_timestamp")
            .and_then(|(_, value)| value.as_i64());
        let room = values
            .iter()
            .find(|(column, _)| *column == "location_id")
            .and_then(|(_, value)| value.as_i64());
        values.push(("story_id", Value::from(self.story_id())));
        values.push((
            "previous_scene_id",
            previous.map_or(Value::Null, Value::from),
        ));
        let scene = id(&self.insert("scenes", values)?);
        for who in cast {
            self.insert(
                "characters_scenes",
                vec![
                    ("character_id", Value::from(*who)),
                    ("scene_id", Value::from(scene)),
                ],
            )?;
        }
        if let (Some(room), Some(at)) = (room, at) {
            self.update(
                "locations",
                room,
                vec![("last_protagonist_visit", Value::from(at))],
            )?;
        }
        Ok(scene)
    }

    /// `Playthrough::Fight#close!`: a fight that has ended is closed with
    /// one scene carrying what it cost. Returns the scene's description.
    fn close_fight(&mut self) -> Result<Option<String>, Error> {
        let blows: Vec<Row> = self
            .game()
            .own("playthrough_blows")
            .into_iter()
            .filter(|blow| int(blow, "scene_id").is_none())
            .cloned()
            .collect();
        let Some(first) = blows.first() else {
            return Ok(None);
        };
        let here = self.row("locations", int(first, "location_id").unwrap_or_default())?;
        let party_here = int(self.game().row, "current_location_id") == Some(id(&here));
        let over = self.over() || !party_here || self.game().foes_in(Some(&here)).is_empty();
        if !over {
            return Ok(None);
        }
        let now = self.here().unwrap_or_else(|| here.clone());
        let mut rounds: Vec<i64> = Vec::new();
        for round in blows.iter().filter_map(|blow| int(blow, "round")) {
            if !rounds.contains(&round) {
                rounds.push(round);
            }
        }
        let rounds = rounds.len() as i64;
        let party = self.player_id();
        let opponent = blows
            .iter()
            .rev()
            .find(|blow| int(blow, "attacker_id") == party && party.is_some())
            .and_then(|blow| int(blow, "target_id"))
            .or_else(|| blows.last().and_then(|blow| int(blow, "attacker_id")));
        let ending = self.fight_ending(&blows, party_here);
        let told: Vec<String> = blows
            .iter()
            .map(|blow| {
                let attacker = self
                    .records
                    .find("characters", int(blow, "attacker_id").unwrap_or_default())
                    .map_or("", |row| string(row, "fullname"))
                    .to_string();
                let target = self
                    .records
                    .find("characters", int(blow, "target_id").unwrap_or_default())
                    .cloned()
                    .unwrap_or_default();
                blow_to_s(
                    &attacker,
                    &target,
                    int(blow, "damage").unwrap_or_default(),
                    int(blow, "hp_after").unwrap_or_default(),
                    int(blow, "round").unwrap_or_default(),
                )
            })
            .collect();
        let name = string(&here, "name").to_string();
        let description = [
            Some(format!(
                "The fight in {name} is over after {}.",
                count(rounds, "round")
            )),
            ending.clone(),
            Some(format!(
                "{} landed: {}.",
                count(blows.len() as i64, "blow"),
                told.join("; ")
            )),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
        let summary = format!(
            "A fight in {name}: {} over {}. {}",
            count(blows.len() as i64, "blow"),
            count(rounds, "round"),
            ending.unwrap_or_default()
        );
        let cast: Vec<i64> = self
            .game()
            .cast_in(Some(&now))
            .iter()
            .map(|who| id(who))
            .collect();
        let at = self.story_now() + ACTION_MINUTES * 60 * rounds;
        let scene = self.write_scene(
            vec![
                ("location_id", Value::from(id(&now))),
                ("description", Value::from(description.clone())),
                ("summary", Value::from(summary)),
                ("story_timestamp", Value::from(at)),
                ("resolved_action", Value::from("attack")),
                (
                    "acted_on_type",
                    opponent.map_or(Value::Null, |_| Value::from("Character")),
                ),
                ("acted_on_id", opponent.map_or(Value::Null, Value::from)),
            ],
            &cast,
        )?;
        for blow in &blows {
            self.update(
                "playthrough_blows",
                id(blow),
                vec![("scene_id", Value::from(scene))],
            )?;
        }
        self.update(
            "playthroughs",
            self.playthrough,
            vec![("current_scene_id", Value::from(scene))],
        )?;
        Ok(Some(description))
    }

    /// `Playthrough::Fight#ending`: who died, or why nobody did.
    fn fight_ending(&self, blows: &[Row], party_here: bool) -> Option<String> {
        let party = self.player();
        let mut people: Vec<i64> = party.iter().map(id).collect();
        for blow in blows {
            for column in ["attacker_id", "target_id"] {
                if let Some(who) = int(blow, column) {
                    if !people.contains(&who) {
                        people.push(who);
                    }
                }
            }
        }
        let dead: Vec<Row> = people
            .into_iter()
            .filter_map(|who| self.records.find("characters", who).cloned())
            .filter(|who| self.condition_of(who).is_some_and(|c| c.dead()))
            .collect();
        if let Some(party) = &party {
            if dead.iter().any(|who| id(who) == id(party)) {
                return Some(format!("{} is dead.", string(party, "fullname")));
            }
        }
        let names: Vec<String> = dead
            .iter()
            .map(|who| string(who, "fullname").to_string())
            .collect();
        match names.len() {
            0 if party_here => Some("Nobody was killed: the fighting stopped.".into()),
            0 => Some("Nobody was killed: the party is no longer standing in it.".into()),
            1 => Some(format!("{} is dead.", names[0])),
            _ => Some(format!("{} are dead.", to_sentence(&names))),
        }
    }

    // --- the snapshot --------------------------------------------------------

    /// `Playthrough::Snapshot#of_the_party!`: this game's copies of what the
    /// protagonist starts out carrying, and the protagonist's body.
    pub fn snapshot_party(&mut self) -> Result<(), Error> {
        let Some(protagonist) = self.story_protagonist() else {
            return Ok(());
        };
        let templates = self.templates_held_by(protagonist);
        self.copy_all(templates.into_iter().map(|t| (t, Into::Hands)).collect())?;
        let who = self.row("characters", protagonist)?;
        self.vitals_row(&who)?;
        Ok(())
    }

    /// `Playthrough::Snapshot#of_the_room!`: this game's copies of what lies
    /// in a room and what its people hold, and a body for each of them.
    pub fn snapshot_room(&mut self, location: Option<&Row>) -> Result<(), Error> {
        let Some(location) = location else {
            return Ok(());
        };
        let here = id(location);
        let mut wanted: Vec<(i64, Into)> = self
            .records
            .select("items", |item| {
                int(item, "location_id") == Some(here)
                    && int(item, "character_id").is_none()
                    && int(item, "playthrough_id").is_none()
                    && text(item, "disposition") == Some("intact")
            })
            .iter()
            .map(|item| (id(item), Into::Room(here)))
            .collect();
        let people: Vec<(i64, bool)> = self
            .game()
            .characters_located_in(location)
            .iter()
            .map(|who| (id(who), flag(who, "is_protagonist")))
            .collect();
        for (person, protagonist) in people {
            let into = if protagonist {
                Into::Hands
            } else {
                Into::Person(person)
            };
            wanted.extend(
                self.templates_held_by(person)
                    .into_iter()
                    .map(|t| (t, into)),
            );
        }
        self.copy_all(wanted)?;
        let bodies: Vec<Row> = self
            .records
            .select("characters", |who| int(who, "location_id") == Some(here))
            .into_iter()
            .cloned()
            .collect();
        for who in bodies {
            self.vitals_row(&who)?;
        }
        Ok(())
    }

    fn templates_held_by(&self, character: i64) -> Vec<i64> {
        self.records
            .select("items", |item| {
                int(item, "character_id") == Some(character)
                    && int(item, "playthrough_id").is_none()
                    && text(item, "disposition") == Some("intact")
            })
            .iter()
            .map(|item| id(item))
            .collect()
    }

    fn copy_all(&mut self, candidates: Vec<(i64, Into)>) -> Result<(), Error> {
        let game = self.playthrough;
        let mut copied: Vec<i64> = self
            .records
            .select("items", |item| int(item, "playthrough_id") == Some(game))
            .iter()
            .filter_map(|item| int(item, "template_id"))
            .collect();
        for (template, into) in candidates {
            if copied.contains(&template) {
                continue;
            }
            copied.push(template);
            let source = self.row("items", template)?;
            let mut values: Vec<(&str, Value)> = source
                .iter()
                .filter(|(column, _)| !NOT_COPIED.contains(&column.as_str()))
                .map(|(column, value)| (column_name(column), value.clone()))
                .collect();
            values.push(("playthrough_id", Value::from(game)));
            values.push(("template_id", Value::from(template)));
            match into {
                Into::Hands => {}
                Into::Room(room) => values.push(("location_id", Value::from(room))),
                Into::Person(who) => values.push(("character_id", Value::from(who))),
            }
            self.insert("items", values)?;
        }
        Ok(())
    }
}

/// One side of a doorway as a shuffle rewrites it: its columns, and the
/// passages opened through it.
struct Doorway {
    values: Vec<(&'static str, Value)>,
    openings: Vec<Vec<(&'static str, Value)>>,
}

/// A passage's columns, less its doorway.
const PASSAGE_COLUMNS: [&str; 7] = [
    "id",
    "playthrough_id",
    "means",
    "opened_at",
    "opened_by_item_id",
    "created_at",
    "updated_at",
];

/// `LocationConnection#derive_time_to_travel`: a doorway's journey in words.
fn travel_time(values: &[(&str, Value)]) -> Value {
    let column = |name: &str| {
        values
            .iter()
            .find(|(column, _)| *column == name)
            .and_then(|(_, value)| value.as_str())
    };
    let (Some(distance), Some(method)) = (column("distance"), column("travel_method")) else {
        return Value::Null;
    };
    let Some(minutes) = crate::arrival::travel_minutes(distance, method) else {
        return Value::Null;
    };
    Value::from(if minutes < 1.0 {
        "under a minute".to_string()
    } else if minutes < 2.0 {
        "about a minute".to_string()
    } else if minutes < 60.0 {
        format!("about {} minutes", minutes.round() as i64)
    } else if minutes < 120.0 {
        "about an hour".to_string()
    } else if minutes < 1440.0 {
        format!("about {} hours", (minutes / 60.0).round() as i64)
    } else if minutes < 2880.0 {
        "about a day".to_string()
    } else {
        format!("about {} days", (minutes / 1440.0).round() as i64)
    })
}

/// Where a copied thing goes.
#[derive(Clone, Copy)]
enum Into {
    Hands,
    Room(i64),
    Person(i64),
}

/// `Item::NOT_COPIED`.
const NOT_COPIED: [&str; 7] = [
    "character_id",
    "location_id",
    "id",
    "playthrough_id",
    "template_id",
    "created_at",
    "updated_at",
];

/// A column name as a `'static` string, for the columns an item copies.
fn column_name(column: &str) -> &'static str {
    const COLUMNS: [&str; 11] = [
        "bulk",
        "combustible",
        "description",
        "disposition",
        "inscription",
        "name",
        "properties",
        "readable",
        "use_kind",
        "x",
        "y",
    ];
    COLUMNS
        .iter()
        .find(|name| **name == column)
        .copied()
        .unwrap_or_else(|| panic!("items has a column {column} this engine does not copy"))
}

/// The help text, where a reading carries it.
fn note_of(reading: &Reading) -> Vec<String> {
    if reading.help {
        grammar::HELP.iter().map(|line| line.to_string()).collect()
    } else {
        Vec::new()
    }
}

/// A person as the grammar reads them.
pub fn person_of(row: &Row) -> Person {
    Person {
        id: id(row),
        fullname: string(row, "fullname").to_string(),
        nickname: text(row, "nickname")
            .filter(|name| !crate::text::is_blank(name))
            .map(str::to_string),
    }
}

/// A thing as the grammar reads it.
pub fn thing_of(row: &Row, carried: bool) -> Thing {
    Thing {
        id: id(row),
        name: string(row, "name").to_string(),
        bulk: text(row, "bulk").unwrap_or("handy").to_string(),
        use_kind: text(row, "use_kind").unwrap_or("ordinary").to_string(),
        combustible: flag(row, "combustible"),
        carried,
        template: int(row, "template_id"),
    }
}

/// A pursuit's row of weights, or none for a pursuit the table lacks
/// (`Volition::Weights.row_for`).
fn weights_for(pursuit: Option<&str>) -> Option<&'static [(String, i64)]> {
    let pursuit = pursuit?;
    data::weights()
        .table
        .iter()
        .find(|(name, _)| name == pursuit)
        .map(|(_, row)| row.as_slice())
}

/// `"1 round"`, `"2 rounds"`.
fn count(number: i64, noun: &str) -> String {
    format!("{number} {noun}{}", if number == 1 { "" } else { "s" })
}

/// `Array#to_sentence`: `a, b, and c`.
fn to_sentence(words: &[String]) -> String {
    match words {
        [] => String::new(),
        [one] => one.clone(),
        [one, two] => format!("{one} and {two}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

/// `Playthrough::Volition.shape_of`: `move:12` is a `move`.
fn shape_of(token: &str) -> &str {
    token.split(':').next().unwrap_or_default()
}

fn weight_of(row: &[(String, i64)], shape: &str) -> i64 {
    row.iter()
        .find(|(name, _)| name == shape)
        .map_or(0, |(_, weight)| *weight)
}

/// `Playthrough::Blow#to_s`.
fn blow_to_s(attacker: &str, target: &Row, damage: i64, hp_after: i64, round: i64) -> String {
    let condition = Condition {
        hp: hp_after,
        max: max_hp(target).unwrap_or_default(),
    };
    format!(
        "{attacker} hit {} for {damage} (round {round}); {} is {}",
        string(target, "fullname"),
        string(target, "fullname"),
        condition.in_words()
    )
}
