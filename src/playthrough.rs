//! One game read out of its rows: who is in a room, how much is left of a
//! body, who is fighting the party, and the sentences the engine writes about
//! a blow and a toll.
//!
//! Each reader is the Ruby `Playthrough` method of the same name
//! (`app/models/playthrough.rb`), asked of a [`Records`] rather than of a
//! database, so it returns rows in the order that query returns them.

use crate::physics;
use crate::records::{flag, id, int, string, text, Records, Row};

/// A room's own hazards, and the words the engine says of each
/// (`Location::HAZARDS`).
pub const ROOM_HAZARDS: [(&str, &str); 4] = [
    ("flooded", "the water takes your legs"),
    ("unlit", "you go down in the dark"),
    ("silent", "the quiet gets into you"),
    ("airless", "there is nothing here to breathe"),
];

/// A doorway's hazards (`LocationConnection::HAZARDS`).
pub const DOORWAY_HAZARDS: [(&str, &str); 2] = [
    ("drop", "the way down is further than it looks from the top"),
    ("undertow", "the water pulls at you the whole way across"),
];

/// Whether a character has a level and a hit die (`Character#stat_block?`).
pub fn stat_block(character: &Row) -> bool {
    int(character, "level").is_some() && int(character, "hit_die").is_some()
}

/// `Character#max_hp`, or none without a stat block: the world's own, at the
/// level the character's row holds. A game's player may have risen past it
/// ([`Game::max_hp`]).
pub fn max_hp(character: &Row) -> Option<i64> {
    Some(hit_points(
        int(character, "level")?,
        int(character, "hit_die")?,
    ))
}

/// The hit points a body of this level and hit die can take.
pub fn hit_points(level: i64, hit_die: i64) -> i64 {
    hit_die + (level - 1) * (hit_die.div_euclid(2) + 1)
}

/// `Character::LEVELS.max`: no floor raises a body past it.
pub const HIGHEST_LEVEL: i64 = 20;

/// The `floors` row of a floor, or of a room laid out inside one: the
/// floor's own, or its parent's.
pub fn floor_of<'a>(records: &'a Records, location: &Row) -> Option<&'a Row> {
    let places = [Some(id(location)), int(location, "parent_location_id")];
    records.first(crate::store::FLOORS, |floor| {
        places.contains(&int(floor, "location_id"))
    })
}

/// `Playthrough::EndNotice#reason`: why a finished game is over, off records.
/// An ending row means the story concluded, a protagonist at zero means death,
/// and neither means the game stopped with no reason recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    Concluded,
    Died,
    Unrecorded,
}

/// `Playthrough::Vitals::Condition`: how much is left of one body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Condition {
    pub hp: i64,
    pub max: i64,
}

impl Condition {
    pub fn dead(&self) -> bool {
        self.hp <= 0
    }

    pub fn unhurt(&self) -> bool {
        self.hp >= self.max
    }

    pub fn badly_hurt(&self) -> bool {
        !self.dead() && self.hp * 2 <= self.max
    }

    pub fn in_words(&self) -> String {
        if self.dead() {
            return "dead".into();
        }
        if self.unhurt() {
            return "unhurt".into();
        }
        let hurt = if self.badly_hurt() {
            "badly hurt"
        } else {
            "hurt"
        };
        format!("{hurt} ({} of {})", self.hp, self.max)
    }
}

/// A floor the player stands on that falls in this game, and when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Countdown {
    /// The floor's name.
    pub name: String,
    /// Story time it gives way, in seconds since the epoch.
    pub falls_at: i64,
    /// Whole minutes until then, a part of a minute counted as one.
    pub minutes_left: i64,
}

impl Countdown {
    /// What the engine says of it, to the player only: "The First Floor
    /// falls in 1 hour 5 minutes.", and under an hour "The First Floor
    /// falls in 25 minutes."
    pub fn in_words(&self) -> String {
        let (hours, minutes) = (self.minutes_left / 60, self.minutes_left % 60);
        let unit = |n: i64, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
        let left = if hours == 0 {
            unit(minutes, "minute")
        } else {
            format!("{} {}", unit(hours, "hour"), unit(minutes, "minute"))
        };
        format!("{} falls in {left}.", self.name)
    }
}

/// How a game is told (`playthroughs.mode`), chosen when it starts and kept
/// on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// The player types each act, and the narrator writes what happened.
    #[default]
    Narrated,
    /// The game picks each act, the engine tells it in its own words, and
    /// the player writes the paragraph. No narrator is asked.
    PlayerNarrates,
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Narrated, Mode::PlayerNarrates];

    /// The column's value.
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Narrated => "narrated",
            Mode::PlayerNarrates => "player_narrates",
        }
    }

    /// The mode a column value names; none for one this engine does not know.
    pub fn parse(value: &str) -> Option<Mode> {
        Mode::ALL.into_iter().find(|mode| mode.as_str() == value)
    }
}

/// One playthrough and every row it can read.
#[derive(Clone, Copy)]
pub struct Game<'a> {
    pub records: &'a Records,
    pub row: &'a Row,
}

impl<'a> Game<'a> {
    pub fn new(records: &'a Records, playthrough: i64) -> Game<'a> {
        let row = records
            .find("playthroughs", playthrough)
            .unwrap_or_else(|| panic!("no playthrough {playthrough}"));
        Game { records, row }
    }

    pub fn id(&self) -> i64 {
        id(self.row)
    }

    pub fn story_id(&self) -> i64 {
        int(self.row, "story_id").expect("a playthrough's story")
    }

    pub fn story(&self) -> &'a Row {
        self.records
            .find("stories", self.story_id())
            .expect("the playthrough's story")
    }

    pub fn character(&self, id: i64) -> &'a Row {
        self.records
            .find("characters", id)
            .unwrap_or_else(|| panic!("no character {id}"))
    }

    pub fn location(&self, id: i64) -> &'a Row {
        self.records
            .find("locations", id)
            .unwrap_or_else(|| panic!("no location {id}"))
    }

    /// How this game is told. A value this engine does not know is told as
    /// narrated, as every game was before the column.
    pub fn mode(&self) -> Mode {
        text(self.row, "mode")
            .and_then(Mode::parse)
            .unwrap_or_default()
    }

    /// The game picks the acts and the player writes the paragraphs.
    pub fn player_narrates(&self) -> bool {
        self.mode() == Mode::PlayerNarrates
    }

    /// The player (`playthrough.character`).
    pub fn protagonist(&self) -> Option<&'a Row> {
        int(self.row, "character_id").map(|id| self.character(id))
    }

    /// `Playthrough::EndNotice#reason`, asked of a game that is over.
    pub fn ended(&self) -> Ended {
        if !self.own("playthrough_endings").is_empty() {
            return Ended::Concluded;
        }
        let dead = self
            .protagonist()
            .and_then(|who| self.vitals_for(who))
            .is_some_and(|condition| condition.dead());
        if dead {
            Ended::Died
        } else {
            Ended::Unrecorded
        }
    }

    pub fn current_location(&self) -> Option<&'a Row> {
        int(self.row, "current_location_id").map(|id| self.location(id))
    }

    pub fn current_scene(&self) -> Option<&'a Row> {
        int(self.row, "current_scene_id").and_then(|id| self.records.find("scenes", id))
    }

    /// The rows of a table that belong to this game, in id order.
    pub fn own(&self, table: &str) -> Vec<&'a Row> {
        let game = self.id();
        self.records
            .select(table, |row| int(row, "playthrough_id") == Some(game))
    }

    /// The scenes from the first to the current one (`#scene_chain`).
    pub fn scene_chain(&self) -> Vec<&'a Row> {
        let mut scenes = Vec::new();
        let mut scene = self.current_scene();
        while let Some(row) = scene {
            scenes.insert(0, row);
            scene = int(row, "previous_scene_id").and_then(|id| self.records.find("scenes", id));
        }
        scenes
    }

    /// Story time now (`#story_now`): the current scene's, or the story's
    /// clock.
    pub fn story_now(&self) -> i64 {
        self.story_time().expect("a story with a clock")
    }

    /// [`Game::story_now`], or none for a story with no scene and no start
    /// time, which has no clock at all.
    pub fn story_time(&self) -> Option<i64> {
        if let Some(at) = self
            .current_scene()
            .and_then(|scene| int(scene, "story_timestamp"))
        {
            return Some(at);
        }
        let story = self.story_id();
        self.records
            .select("scenes", |scene| int(scene, "story_id") == Some(story))
            .iter()
            .filter_map(|scene| int(scene, "story_timestamp"))
            .max()
            .or_else(|| int(self.story(), "start_time"))
    }

    fn vitals_row(&self, character: i64) -> Option<&'a Row> {
        self.own("playthrough_vitals")
            .into_iter()
            .find(|row| int(row, "character_id") == Some(character))
    }

    fn npc_state(&self, character: i64) -> Option<&'a Row> {
        self.own("playthrough_npc_states")
            .into_iter()
            .find(|row| int(row, "character_id") == Some(character))
    }

    /// The floors this game has reached, in the order it reached them.
    pub fn floors_reached(&self) -> Vec<&'a Row> {
        self.own(crate::store::PLAYTHROUGH_FLOORS)
    }

    /// The level a body has in this game by story time `at` (none: now).
    /// For the player it is the level of the highest floor this game has
    /// reached by then that raised one (`playthrough_floors.level`); for
    /// anybody else, and for a player no floor has raised, it is the
    /// world's (`characters.level`).
    pub fn level_at(&self, character: &Row, at: Option<i64>) -> Option<i64> {
        let world = int(character, "level")?;
        if int(self.row, "character_id") != Some(id(character)) {
            return Some(world);
        }
        let raised = self
            .floors_reached()
            .into_iter()
            .filter(|row| at.is_none_or(|at| int(row, "arrived_at").is_some_and(|then| then <= at)))
            .filter_map(|row| int(row, "level"))
            .max();
        Some(raised.unwrap_or(world))
    }

    /// The level a body has in this game now.
    pub fn level(&self, character: &Row) -> Option<i64> {
        self.level_at(character, None)
    }

    /// The hit points a body can take in this game now, or none without a
    /// stat block.
    pub fn max_hp(&self, character: &Row) -> Option<i64> {
        self.max_hp_at(character, None)
    }

    /// The hit points a body could take in this game at story time `at`:
    /// what a blow or a toll of then is said against.
    pub fn max_hp_at(&self, character: &Row, at: Option<i64>) -> Option<i64> {
        Some(hit_points(
            self.level_at(character, at)?,
            int(character, "hit_die")?,
        ))
    }

    /// What the engine says of a floor that raised the player's level, in
    /// its own words and to the player only: "Reaching The Second Floor
    /// raised Odile to level 5: 28 hit points, and healed to full." None for
    /// a floor that raised nothing.
    pub fn floor_to_s(&self, reached: &Row) -> Option<String> {
        let level = int(reached, "level")?;
        let player = self.protagonist()?;
        let floor = self.location(int(reached, "location_id")?);
        Some(format!(
            "Reaching {} raised {} to level {level}: {} hit points, and healed to full.",
            string(floor, "name"),
            string(player, "fullname"),
            hit_points(level, int(player, "hit_die")?)
        ))
    }

    /// This game's row for the floor a room is on (the floor itself, or a
    /// room laid out inside it), or none for a room on no floor and a floor
    /// this game has not reached.
    pub fn reached(&self, location: &Row) -> Option<&'a Row> {
        let place = int(floor_of(self.records, location)?, "location_id");
        self.floors_reached()
            .into_iter()
            .find(|row| int(row, "location_id") == place)
    }

    /// The story time a floor this game has reached gives way in it: its
    /// countdown (`floors.minutes`) from the time this game first stood on
    /// it. None for a floor with no countdown.
    pub fn falls_at(&self, reached: &Row) -> Option<i64> {
        let place = int(reached, "location_id");
        let floor = self.records.first(crate::store::FLOORS, |floor| {
            int(floor, "location_id") == place
        })?;
        Some(int(reached, "arrived_at")? + int(floor, "minutes")? * 60)
    }

    /// Whether a room is on a floor that has fallen in this game
    /// (`playthrough_floors.fell_at`): nobody on it is alive in this game,
    /// and no way out leads onto it. The world's rows are as they were, and
    /// every other game sees the floor whole.
    pub fn fallen(&self, location: &Row) -> bool {
        self.reached(location)
            .is_some_and(|row| int(row, "fell_at").is_some())
    }

    /// The floor the player stands on and how long it has left, while it
    /// stands: none off a floor, on a floor with no countdown, and once it
    /// has fallen.
    pub fn countdown(&self) -> Option<Countdown> {
        let here = self.current_location()?;
        let row = self.reached(here)?;
        if int(row, "fell_at").is_some() {
            return None;
        }
        let falls_at = self.falls_at(row)?;
        let left = (falls_at - self.story_time()?).max(0);
        Some(Countdown {
            name: string(self.location(int(row, "location_id")?), "name").to_string(),
            falls_at,
            minutes_left: (left + 59).div_euclid(60),
        })
    }

    /// What the engine says of a floor that gave way under the player, in
    /// its own words and to the player only: "The First Floor gave way at
    /// 8:25 am, and Odile fell with it." None unless the game ended with the
    /// player on a fallen floor; the death statement follows it.
    pub fn collapse_to_s(&self) -> Option<String> {
        int(self.row, "ended_at")?;
        let row = self.reached(self.current_location()?)?;
        let fell = int(row, "fell_at")?;
        Some(format!(
            "{} gave way at {}, and {} fell with it.",
            string(self.location(int(row, "location_id")?), "name"),
            crate::moment::clock_time(fell),
            string(self.protagonist()?, "fullname")
        ))
    }

    /// `#vitals_for`: none for somebody with no stat block.
    pub fn vitals_for(&self, character: &Row) -> Option<Condition> {
        let max = self.max_hp(character)?;
        let hp = self
            .vitals_row(id(character))
            .and_then(|row| int(row, "hp_current"))
            .unwrap_or(max);
        Some(Condition { hp, max })
    }

    /// `Playthrough::NpcState#ceasefire_holds?`.
    fn ceasefire_holds(&self, state: &Row) -> bool {
        if !flag(state, "ceasefire") {
            return false;
        }
        let Some(after) = int(state, "peace_after_blow_id") else {
            return true;
        };
        let attacker = int(self.row, "character_id");
        let target = int(state, "character_id");
        !self.own("playthrough_blows").iter().any(|blow| {
            int(blow, "attacker_id") == attacker
                && int(blow, "target_id") == target
                && id(blow) > after
        })
    }

    /// `#provoked?`: this game picked a fight with this body, and no
    /// ceasefire with it holds.
    pub fn provoked(&self, character: i64) -> bool {
        !self.ceasefire_with(character)
            && self
                .vitals_row(character)
                .is_some_and(|row| int(row, "provoked_at").is_some())
    }

    /// Whether this character's ceasefire with the player holds.
    pub fn ceasefire_with(&self, character: i64) -> bool {
        self.npc_state(character)
            .is_some_and(|state| self.ceasefire_holds(state))
    }

    /// `Scene::Generator.characters_present` without a playthrough: the
    /// protagonist, the companions and whoever the world puts in the room.
    fn characters_present(&self, location: &Row) -> Vec<&'a Row> {
        let story = int(location, "story_id");
        let characters = self
            .records
            .select("characters", |row| int(row, "story_id") == story);
        let mut present: Vec<&'a Row> = Vec::new();
        let mut add = |row: &'a Row| {
            if !present.iter().any(|seen| id(seen) == id(row)) {
                present.push(row);
            }
        };
        if let Some(protagonist) = characters.iter().find(|row| flag(row, "is_protagonist")) {
            add(protagonist);
        }
        for row in characters.iter().filter(|row| flag(row, "is_companion")) {
            add(row);
        }
        let here = id(location);
        for row in self
            .records
            .select("characters", |row| int(row, "location_id") == Some(here))
        {
            add(row);
        }
        present
    }

    /// `#characters_located_in`.
    pub fn characters_located_in(&self, location: &Row) -> Vec<&'a Row> {
        if int(location, "story_id") != Some(self.story_id()) {
            return Vec::new();
        }
        let states = self.own("playthrough_npc_states");
        let overridden: Vec<i64> = states
            .iter()
            .filter_map(|row| int(row, "character_id"))
            .collect();
        let here = id(location);
        let mut people: Vec<&'a Row> = self
            .characters_present(location)
            .into_iter()
            .filter(|who| !overridden.contains(&id(who)))
            .collect();
        for state in states
            .iter()
            .filter(|row| int(row, "location_id") == Some(here))
        {
            let who = self.character(int(state, "character_id").expect("a state's character"));
            if !people.iter().any(|seen| id(seen) == id(who)) {
                people.push(who);
            }
        }
        people.sort_by_key(|who| id(who));
        people
    }

    /// `#cast_in`: the living in a room. Nobody on a floor that has fallen
    /// in this game is alive in it.
    pub fn cast_in(&self, location: Option<&Row>) -> Vec<&'a Row> {
        let Some(location) = location.filter(|room| !self.fallen(room)) else {
            return Vec::new();
        };
        self.characters_located_in(location)
            .into_iter()
            .filter(|who| !self.vitals_for(who).is_some_and(|c| c.dead()))
            .collect()
    }

    /// `#foes_in`: who in a room is fighting the party.
    pub fn foes_in(&self, location: Option<&Row>) -> Vec<&'a Row> {
        self.cast_in(location)
            .into_iter()
            .filter(|who| !flag(who, "is_protagonist") && !flag(who, "is_companion"))
            .filter(|who| {
                let provoked = self
                    .vitals_row(id(who))
                    .is_some_and(|row| int(row, "provoked_at").is_some());
                (flag(who, "hostile") || provoked) && !self.ceasefire_with(id(who))
            })
            .collect()
    }

    /// `#followers`: the people walking with the party out of its room.
    pub fn followers(&self) -> Vec<&'a Row> {
        let Some(here) = self.current_location() else {
            return Vec::new();
        };
        let foes: Vec<i64> = self.foes_in(Some(here)).iter().map(|who| id(who)).collect();
        self.own("playthrough_npc_states")
            .into_iter()
            .filter(|row| flag(row, "following") && int(row, "location_id") == Some(id(here)))
            .map(|row| self.character(int(row, "character_id").expect("a state's character")))
            .filter(|who| {
                !self.vitals_for(who).is_some_and(|c| c.dead()) && !foes.contains(&id(who))
            })
            .collect()
    }

    /// `#cast_on_arrival`: who a room holds once the party walks in.
    pub fn cast_on_arrival(&self, location: &Row) -> Vec<&'a Row> {
        let mut people = self.cast_in(Some(location));
        for who in self.followers() {
            if !people.iter().any(|seen| id(seen) == id(who)) {
                people.push(who);
            }
        }
        people.sort_by_key(|who| id(who));
        people
    }

    /// This game's intact items, in id order.
    fn items(&self, keep: impl Fn(&Row) -> bool) -> Vec<&'a Row> {
        self.own("items")
            .into_iter()
            .filter(|item| text(item, "disposition") == Some("intact") && keep(item))
            .collect()
    }

    /// `#carried`: what the party has in its hands.
    pub fn carried(&self) -> Vec<&'a Row> {
        self.items(|item| int(item, "character_id").is_none() && int(item, "location_id").is_none())
    }

    /// What the party is wearing, of what it carries, head to foot
    /// (`data/item/wear.yml`'s places), a place no body has last.
    pub fn worn(&self) -> Vec<&'a Row> {
        let mut worn: Vec<&'a Row> = self
            .carried()
            .into_iter()
            .filter(|item| crate::wear::worn(item))
            .collect();
        let places = crate::wear::places();
        worn.sort_by_key(|item| {
            crate::wear::place_of(item)
                .and_then(|place| places.iter().position(|known| known == place))
                .unwrap_or(places.len())
        });
        worn
    }

    /// `#items_lying_in`.
    pub fn items_lying_in(&self, location: Option<&Row>) -> Vec<&'a Row> {
        let Some(location) = location else {
            return Vec::new();
        };
        let here = id(location);
        self.items(|item| {
            int(item, "location_id") == Some(here)
                && int(item, "character_id").is_none()
                && text(item, "tier") != Some(crate::kit::PART)
        })
    }

    /// What is attached to the fixed pieces of a room and could be broken
    /// off them, in id order. A part is not a thing lying there: it is in no
    /// list a line is read against but the break attempts.
    pub fn parts_in(&self, location: Option<&Row>) -> Vec<&'a Row> {
        let Some(location) = location else {
            return Vec::new();
        };
        let here = id(location);
        self.items(|item| {
            int(item, "location_id") == Some(here)
                && int(item, "character_id").is_none()
                && text(item, "tier") == Some(crate::kit::PART)
        })
    }

    /// What the player has noticed lying in a room ([`crate::noticed`]): in
    /// a game the player narrates, the things this game stamped; in any
    /// other, everything [`Game::items_lying_in`] holds.
    pub fn items_noticed_in(&self, location: Option<&Row>) -> Vec<&'a Row> {
        let lying = self.items_lying_in(location);
        if !self.player_narrates() {
            return lying;
        }
        lying
            .into_iter()
            .filter(|item| crate::noticed::stamped(item))
            .collect()
    }

    /// `#items_held_by`.
    pub fn items_held_by(&self, character: &Row) -> Vec<&'a Row> {
        let who = id(character);
        self.items(|item| int(item, "character_id") == Some(who))
    }

    /// `#exits`: the rooms the current room's doorways lead to, by id, less
    /// any on a floor that has fallen in this game.
    pub fn exits(&self) -> Vec<&'a Row> {
        let Some(here) = self.current_location() else {
            return Vec::new();
        };
        let here = id(here);
        let mut rooms: Vec<&'a Row> = self
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id") == Some(here)
            })
            .iter()
            .map(|edge| self.location(int(edge, "connected_location_id").expect("a far end")))
            .filter(|room| !self.fallen(room))
            .collect();
        rooms.sort_by_key(|room| id(room));
        rooms
    }

    /// `Playthrough::Toll#to_s`.
    pub fn toll_to_s(&self, toll: &Row) -> String {
        let who = string(
            self.character(int(toll, "character_id").expect("a toll's body")),
            "fullname",
        );
        let hazard = string(toll, "hazard");
        let where_it_was = self.where_it_was(toll);
        if got_clear(toll) {
            return format!("{who} got clear of {hazard} on {where_it_was}");
        }
        let damage = int(toll, "damage").expect("damage");
        let condition = Condition {
            hp: int(toll, "hp_after").expect("hp_after"),
            max: self
                .max_hp_at(
                    self.character(int(toll, "character_id").unwrap()),
                    int(toll, "story_timestamp"),
                )
                .expect("a stat block"),
        };
        format!(
            "{hazard} on {where_it_was} cost {who} {damage} hit point{} ({}); {who} is {}",
            if damage == 1 { "" } else { "s" },
            self.toll_words(toll),
            condition.in_words()
        )
    }

    /// `Playthrough::Toll#words`, and for a fall, how far it was and whether
    /// a landing halved it.
    pub fn toll_words(&self, toll: &Row) -> String {
        if text(toll, "hazard") != Some(physics::FALL) {
            return toll_words(toll);
        }
        let storeys = int(toll, "location_connection_id")
            .and_then(|edge| self.records.find("location_connections", edge))
            .and_then(|edge| {
                physics::storeys(
                    crate::plan::storey(self.location(int(edge, "location_id")?)),
                    crate::plan::storey(self.location(int(edge, "connected_location_id")?)),
                )
            });
        physics::words(
            storeys,
            flag(toll, "saved"),
            int(toll, "damage").unwrap_or_default(),
        )
    }

    /// `Playthrough::Toll#where_it_was`.
    pub fn where_it_was(&self, toll: &Row) -> String {
        let edge = int(toll, "location_connection_id")
            .and_then(|edge| self.records.find("location_connections", edge));
        match edge {
            None => string(
                self.location(int(toll, "location_id").expect("a toll's room")),
                "name",
            )
            .to_string(),
            Some(edge) => format!(
                "the way from {} into {}",
                string(self.location(int(edge, "location_id").unwrap()), "name"),
                string(
                    self.location(int(edge, "connected_location_id").unwrap()),
                    "name"
                )
            ),
        }
    }
}

/// Whether a toll cost nothing because the save was made. A fall's save
/// only halves it, so a saved fall that still cost something is a cost.
pub fn got_clear(toll: &Row) -> bool {
    flag(toll, "saved")
        && (text(toll, "hazard") != Some(physics::FALL) || int(toll, "damage") == Some(0))
}

/// `Playthrough::Toll#words`: the catalogue's sentence, or the bare key.
pub fn toll_words(toll: &Row) -> String {
    let hazard = string(toll, "hazard");
    let tables: [&[(&str, &str)]; 2] = if int(toll, "location_connection_id").is_some() {
        [&DOORWAY_HAZARDS, &ROOM_HAZARDS]
    } else {
        [&ROOM_HAZARDS, &DOORWAY_HAZARDS]
    };
    tables
        .iter()
        .find_map(|table| table.iter().find(|(key, _)| *key == hazard))
        .map(|(_, words)| words.to_string())
        .unwrap_or_else(|| hazard.to_string())
}
