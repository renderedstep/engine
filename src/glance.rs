//! What a front end's side panels show for the room the player stands in,
//! and which verbs are open from there (`Playthrough::Glance` and
//! `Playthrough::Availability`): the room and its ways out, the people here
//! and how each of them is, what is lying here, what the player carries, the
//! player's condition, the story's next beat, what the play box completes
//! after a slash, and the read-out a line leaves.
//!
//! THE TARGETS ARE THE ENGINE'S, FILTERED BY THE ENGINE'S OWN CHECK. Each
//! verb's candidates are the closed set a line reads against
//! ([`Room::offered_for`]), kept only where the turn would play an intent
//! naming them ([`Mechanics::refusal_for`]). So a target listed here is one
//! the turn accepts, by construction rather than by a second copy of its
//! rules. The reason a verb is closed is the refusal's own words.
//!
//! IN A GAME THE PLAYER NARRATES, THE PANELS ARE WHAT IT HAS NOTICED. The
//! room those closed sets come from holds only noticed things
//! ([`crate::noticed`]), so every list here does too, and `examine` offers a
//! look around the room for as long as one would show something.
//!
//! It reads records only: it writes nothing, calls no model and rolls no
//! die.

use crate::grammar::{self, Grammar};
use crate::intent::{slot_for, Intent};
use crate::noticed;
use crate::outcome::State;
use crate::playthrough::{Game, Mode};
use crate::records::{flag, id, int, string, text, Records, Row};
use crate::refusal::{self, Refusal};
use crate::room::{Choice, Record, Room};
use crate::slash_menu::SlashMenu;
use crate::turn::{person_of, Mechanics, ABILITIES};

/// The verbs a panel lists, in order: every intent a line can resolve to,
/// less `other`, which names nothing (`Playthrough::Availability::VERBS`).
/// The engine's own instruments are not verbs in the fiction.
pub const VERBS: &[&str] = &[
    "move", "talk", "examine", "take", "drop", "attack", "throw", "use",
];

/// Why a verb that reads two closed sets, or reads no set a refusal names,
/// is closed (`Playthrough::Availability::REASONS`).
pub const REASONS: &[(&str, &str)] = &[
    (
        "examine",
        "There is nothing here or in your hands to look at closely.",
    ),
    (
        "throw",
        "There is nothing you can lift and nothing to throw it at.",
    ),
];

/// The room the player stands in, and the place it is a room of.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Here {
    pub id: i64,
    pub name: String,
    /// The containing place's name, for a room that has been placed in one.
    pub within: Option<String>,
    /// For a safe room (`locations.sanctuary`), the engine's line saying
    /// so, [`SAFE_ROOM`]. No model is told it.
    pub safe: Option<String>,
}

/// The room panel's line for a safe room.
pub const SAFE_ROOM: &str = "A safe room: nothing hostile is born here, and resting here heals.";

/// A way out: the room beyond, whether it is written yet, and whether this
/// game may cross it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Exit {
    pub id: i64,
    pub name: String,
    pub written: bool,
    pub open: bool,
}

/// Somebody standing here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub id: i64,
    pub name: String,
    /// How much is left of them, in words; none for somebody with no stat
    /// block.
    pub condition: Option<String>,
    /// Fighting the party now.
    pub foe: bool,
    /// This game picked the fight.
    pub provoked: bool,
}

/// A thing, by id and name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Thing {
    pub id: i64,
    pub name: String,
    /// The fixture it lies on or in, for a thing lying here on one.
    pub on: Option<String>,
    /// Story time when a game the player narrates noticed it; none in any
    /// other game.
    pub noticed_at: Option<i64>,
}

/// Something fixed in place here: what it holds, whether it is shut, and
/// what lies on or in it, in the order the lists are written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fixture {
    pub id: i64,
    pub name: String,
    /// `Item::HOLDS`: `nothing`, `top`, `hollow` or `closed`.
    pub holds: String,
    /// `shut` for a closed fixture, `open` once this game has searched it,
    /// none for one with no inside.
    pub state: Option<String>,
    /// Whether this game has looked inside a closed fixture; none for one
    /// with no inside.
    pub searched: Option<bool>,
    pub on: Vec<String>,
    /// Story time when a game the player narrates noticed it; none in any
    /// other game.
    pub noticed_at: Option<i64>,
}

/// How much the room shows, and how much of it nobody has looked in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// The fixtures and the things lying here, together.
    pub visible: usize,
    /// The closed fixtures this game has not searched.
    pub unsearched: usize,
    /// In a game the player narrates, the things in plain sight here it
    /// has not noticed yet, which a look would show; 0 in any other game.
    pub unnoticed: usize,
}

/// One target a verb can take: a record by id and the name it answers to,
/// or, for `use`, one whole attempt by its token, or, for `examine` in a game
/// the player narrates, a look around the room it stands in (kind `look`,
/// named for the room).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub id: Option<i64>,
    pub name: String,
    /// An attempt's `use:` token.
    pub token: Option<String>,
    /// An attempt's own word, or `look` for a look around the room.
    pub kind: Option<String>,
    /// The line that plays this attempt, or none where no line the grammar
    /// reads plays this one rather than another.
    pub line: Option<String>,
}

/// One verb. `reason` is none exactly when it is available; `aims` is set
/// only for `throw`, the one verb that names two records; `word` is what
/// follows the slash for it, none for `use`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verb {
    pub name: String,
    pub targets: Vec<Target>,
    pub aims: Option<Vec<Target>>,
    pub reason: Option<String>,
    pub word: Option<String>,
}

impl Verb {
    pub fn available(&self) -> bool {
        self.reason.is_none()
    }
}

/// The question a front end asks once every goal of the story is met:
/// whether to end it there. Its answers are `/end` and "Not yet"
/// ([`crate::engine::Engine::put_off_ending`]); any other line is "Not yet"
/// too.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EndingOffer {
    /// "You have done everything <the arc> asked for:".
    pub heading: String,
    /// What the story asked for, beat by beat in order.
    pub goals: Vec<String>,
    pub question: String,
    /// The answer that ends the story, and the line it plays.
    pub end: String,
    pub end_line: String,
    pub not_yet: String,
}

pub const ENDING_QUESTION: &str = "End the story here?";
pub const END_THE_STORY: &str = "End the story";
pub const NOT_YET: &str = "Not yet";

impl EndingOffer {
    fn of(arc: &Row, goals: Vec<String>) -> EndingOffer {
        EndingOffer {
            heading: format!(
                "You have done everything {} asked for:",
                string(arc, "title")
            ),
            goals,
            question: ENDING_QUESTION.to_string(),
            end: END_THE_STORY.to_string(),
            end_line: "/end".to_string(),
            not_yet: NOT_YET.to_string(),
        }
    }
}

/// The floor the player stands on, while it stands and has a countdown
/// (`floors.minutes`). No model is told any of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Floor {
    pub name: String,
    /// Story time it gives way in this game, in seconds since the epoch.
    pub falls_at: i64,
    /// Whole minutes until then, a part of a minute counted as one.
    pub minutes_left: i64,
    /// The engine's line for it: "The First Floor falls in 25 minutes."
    pub countdown: String,
}

/// Everything a front end reads between turns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Glance {
    pub here: Option<Here>,
    /// In the order the turn offers them, as are the other lists here.
    pub exits: Vec<Exit>,
    pub people: Vec<Person>,
    /// What stands here fixed in place, never among `lying_here`.
    pub fixtures: Vec<Fixture>,
    /// What lies here and may be picked up: on the floor, or on or in a
    /// fixture.
    pub lying_here: Vec<Thing>,
    pub counts: Counts,
    pub carrying: Vec<Thing>,
    /// The player's condition in words; none without a stat block.
    pub condition: Option<String>,
    /// The world's numbers for the player: the stat block, the abilities,
    /// each only where the world has it.
    pub sheet: Vec<String>,
    /// The sentence the narrator is told the story is asking for.
    pub next_beat: Option<String>,
    /// Story time now, in seconds since the epoch.
    pub story_time: Option<i64>,
    /// The floor the player stands on and when it falls, while it stands.
    pub floor: Option<Floor>,
    /// For a game that ended with the player on a floor that gave way, the
    /// engine's sentence for it, which the death statement (`ended`)
    /// follows: "The First Floor gave way at 8:25 am, and Odile fell with
    /// it."
    pub collapse: Option<String>,
    /// Every goal is met and the ending decided: `/end` ends the story on
    /// it, whether or not it is still offered.
    pub ending_due: bool,
    /// The question to ask while the ending is due and the player has
    /// neither taken it nor played on past it; asked once.
    pub ending_offer: Option<EndingOffer>,
    pub over: bool,
    /// Why the game stopped, for a game that has.
    pub ended: Option<String>,
    pub verbs: Vec<Verb>,
    pub slash_menu: SlashMenu,
    /// The read-out a console prints (`Playthrough::Mechanics::State`).
    pub state: State,
    /// How the game is told.
    pub mode: Mode,
    /// In a game the player narrates, the scene waiting for the player's
    /// paragraph (`narrates::waiting`).
    pub waiting: Option<i64>,
}

impl Glance {
    /// Reads one playthrough's glance off the records the loop holds.
    pub fn read(mechanics: &Mechanics) -> Glance {
        let records = mechanics.records();
        let game = Game::new(records, mechanics.playthrough());
        let room = mechanics.room();
        let location = game.current_location();
        let foes: Vec<i64> = game.foes_in(location).into_iter().map(id).collect();
        let over = mechanics.over();

        let exits = if location.is_some() {
            room.exits
                .iter()
                .map(|exit| Exit {
                    id: exit.place.id,
                    name: exit.place.name.clone(),
                    written: records
                        .find("locations", exit.place.id)
                        .is_some_and(|far| text(far, "detail_level") != Some("stub")),
                    open: exit.barrier == "open",
                })
                .collect()
        } else {
            Vec::new()
        };
        let people = room
            .characters_here()
            .iter()
            .filter_map(|record| records.find("characters", record.id()?))
            .map(|who| Person {
                id: id(who),
                name: string(who, "fullname").to_string(),
                condition: game.vitals_for(who).map(|condition| condition.in_words()),
                foe: foes.contains(&id(who)),
                provoked: game.provoked(id(who)),
            })
            .collect();
        let row = |id: i64| records.find("items", id);
        let fixed =
            |id: i64| row(id).and_then(|item| text(item, "tier")) == Some(crate::kit::FIXTURE);
        let things = |listed: Vec<Record>| -> Vec<Thing> {
            listed
                .iter()
                .filter_map(Record::thing)
                .filter(|thing| !fixed(thing.id))
                .map(|thing| Thing {
                    id: thing.id,
                    name: thing.name.clone(),
                    on: row(thing.id)
                        .and_then(|item| int(item, "within_id"))
                        .and_then(row)
                        .map(|within| string(within, "name").to_string()),
                    noticed_at: row(thing.id).and_then(|item| int(item, "noticed_at")),
                })
                .collect()
        };
        let lying_here = things(room.items_here());
        let fixtures: Vec<Fixture> = room
            .items_here()
            .iter()
            .filter_map(Record::thing)
            .filter_map(|thing| row(thing.id).filter(|_| fixed(thing.id)))
            .map(|item| {
                let holds = text(item, "holds").unwrap_or("nothing").to_string();
                let closed = holds == "closed";
                let searched = int(item, "searched_at").is_some();
                Fixture {
                    id: id(item),
                    name: string(item, "name").to_string(),
                    on: lying_here
                        .iter()
                        .filter(|thing| {
                            row(thing.id).and_then(|t| int(t, "within_id")) == Some(id(item))
                        })
                        .map(|thing| thing.name.clone())
                        .collect(),
                    holds,
                    state: closed.then(|| if searched { "open" } else { "shut" }.to_string()),
                    searched: closed.then_some(searched),
                    noticed_at: int(item, "noticed_at"),
                }
            })
            .collect();
        let counts = Counts {
            visible: fixtures.len() + lying_here.len(),
            unsearched: fixtures
                .iter()
                .filter(|f| f.searched == Some(false))
                .count(),
            unnoticed: noticed::unnoticed_in(&game, location).len(),
        };
        let player = game.protagonist();

        Glance {
            here: location.map(|room| Here {
                id: id(room),
                name: string(room, "name").to_string(),
                within: containing_place(records, room),
                safe: flag(room, "sanctuary").then(|| SAFE_ROOM.to_string()),
            }),
            exits,
            people,
            fixtures,
            lying_here,
            counts,
            carrying: things(room.items_carried()),
            condition: player
                .and_then(|who| game.vitals_for(who))
                .map(|condition| condition.in_words()),
            sheet: player.map(|who| sheet(&game, who)).unwrap_or_default(),
            next_beat: crate::moment::Moment::new(game).next_beat(),
            story_time: game.story_time(),
            floor: game.countdown().map(|countdown| Floor {
                countdown: countdown.in_words(),
                name: countdown.name,
                falls_at: countdown.falls_at,
                minutes_left: countdown.minutes_left,
            }),
            collapse: game.collapse_to_s(),
            ending_due: !over && mechanics.due_ending().is_some(),
            ending_offer: ending_offer(mechanics, &game),
            over,
            ended: over.then(|| over_refusal(&game).fact),
            verbs: verbs(mechanics, &room, &game),
            slash_menu: SlashMenu::for_room(&room),
            state: State::read(records, mechanics.playthrough()),
            mode: game.mode(),
            waiting: crate::narrates::waiting(records, mechanics.playthrough()),
        }
    }
}

/// The question a due ending asks, while it is still offered.
fn ending_offer(mechanics: &Mechanics, game: &Game) -> Option<EndingOffer> {
    if !mechanics.ending_offered() {
        return None;
    }
    let due = mechanics.due_ending()?;
    let outcome = game
        .records
        .find("quest_outcomes", int(&due, "quest_outcome_id")?)?;
    let arc = game.records.find("quests", int(outcome, "quest_id")?)?;
    let goals = crate::outcome::steps(game.records, arc)
        .into_iter()
        .map(|step| string(step, "summary").to_string())
        .collect();
    Some(EndingOffer::of(arc, goals))
}

/// `Location#containing_place`: the parent of a room that has been placed.
fn containing_place(records: &Records, room: &Row) -> Option<String> {
    crate::plan::box_of(room)?;
    let parent = records.find("locations", int(room, "parent_location_id")?)?;
    Some(string(parent, "name").to_string())
}

/// `Playthrough::Mechanics::State#sheet`: each half of the sheet the world
/// has, and nothing for a half it does not. The level is this game's, which
/// a floor may have raised past the world's.
fn sheet(game: &Game, character: &Row) -> Vec<String> {
    let mut sheet = Vec::new();
    if crate::playthrough::stat_block(character) {
        sheet.push(format!(
            "level {}, d{}",
            game.level(character).unwrap_or_default(),
            int(character, "hit_die").unwrap_or_default()
        ));
    }
    let abilities: Option<Vec<String>> = ABILITIES
        .iter()
        .map(|ability| int(character, ability).map(|score| format!("{ability} {score}")))
        .collect();
    if let Some(abilities) = abilities {
        sheet.push(abilities.join(" "));
    }
    sheet
}

/// `Playthrough::EndNotice#sentence`, as a refusal of a line typed into the
/// finished game says it.
fn over_refusal(game: &Game) -> Refusal {
    let character = int(game.row, "character_id")
        .and_then(|who| game.records.find("characters", who))
        .map(person_of);
    Refusal::over(game.ended(), character.as_ref(), "")
}

fn verbs(mechanics: &Mechanics, room: &Room, game: &Game) -> Vec<Verb> {
    let grammar = Grammar::new(room);
    VERBS
        .iter()
        .map(|&name| {
            let word = grammar::word_for(name);
            if mechanics.over() {
                return Verb {
                    name: name.to_string(),
                    targets: Vec::new(),
                    aims: (name == "throw").then(Vec::new),
                    reason: Some(over_refusal(game).fact),
                    word,
                };
            }
            let accepted = |intent: &Intent| {
                !intent.refused() && mechanics.refusal_for(intent, "", room).is_none()
            };
            let (targets, aims) = match name {
                "throw" => {
                    let things = [room.items_carried(), room.items_here()].concat();
                    let aims = [room.characters_here(), room.exits_here()].concat();
                    let mut kept_things: Vec<Record> = Vec::new();
                    let mut kept_aims: Vec<Record> = Vec::new();
                    for thing in &things {
                        for aim in &aims {
                            let intent = Intent {
                                item: Some(thing.clone()),
                                at: Some(aim.clone()),
                                ..Intent::new("throw")
                            };
                            if accepted(&intent) {
                                if !kept_things.contains(thing) {
                                    kept_things.push(thing.clone());
                                }
                                if !kept_aims.contains(aim) {
                                    kept_aims.push(aim.clone());
                                }
                            }
                        }
                    }
                    (kept_things, Some(kept_aims))
                }
                "use" => {
                    let choices = room
                        .offered_for("use")
                        .into_iter()
                        .filter(|record| {
                            let physical = record.attempt().cloned();
                            accepted(&Intent {
                                physical,
                                ..Intent::new("use")
                            })
                        })
                        .collect();
                    (choices, None)
                }
                _ => {
                    let slot = slot_for(name).expect("every single-slot verb has a slot");
                    let targets = room
                        .offered_for(name)
                        .into_iter()
                        .filter(|record| {
                            accepted(&Intent::new(name).with(slot, Some(record.clone())))
                        })
                        .collect();
                    (targets, None)
                }
            };
            let mut targets: Vec<Target> = targets
                .iter()
                .map(|record| target(record, &grammar))
                .collect();
            if name == "examine" {
                targets.extend(look(game));
            }
            let reason = targets.is_empty().then(|| blocked(name, game));
            Verb {
                name: name.to_string(),
                targets,
                aims: aims.map(|aims| aims.iter().map(|record| target(record, &grammar)).collect()),
                reason,
                word,
            }
        })
        .collect()
}

/// In a game the player narrates, a look around the room it stands in, for
/// as long as a look would show something there.
fn look(game: &Game) -> Option<Target> {
    let here = game.current_location();
    noticed::worth_a_look(game, here).then(|| Target {
        id: None,
        name: here.map_or("", |room| string(room, "name")).to_string(),
        token: None,
        kind: Some(noticed::LOOK.to_string()),
        line: Some(noticed::LOOK_LINE.to_string()),
    })
}

fn target(record: &Record, grammar: &Grammar) -> Target {
    let attempt: Option<&Choice> = record.attempt();
    Target {
        id: record.id(),
        name: record.label(),
        token: attempt.map(Choice::token),
        kind: attempt.map(|choice| choice.kind.clone()),
        line: attempt.and_then(|choice| grammar.line_for(choice)),
    }
}

/// `Playthrough::Availability#blocked`: what the game cannot do at all,
/// else what the empty closed set says, else the verb's own reason.
fn blocked(name: &str, game: &Game) -> String {
    let unplayable = Refusal::unplayable(
        name,
        int(game.row, "character_id").is_some(),
        game.current_location().is_some(),
        "",
    );
    if let Some(unplayable) = unplayable {
        return unplayable.fact;
    }
    if let Some(sentence) = refusal::empty(name) {
        return sentence.to_string();
    }
    REASONS
        .iter()
        .find(|(verb, _)| *verb == name)
        .map(|(_, reason)| reason.to_string())
        .unwrap_or_else(|| panic!("no reason for {name}"))
}
