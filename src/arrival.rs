//! `Scene::Generator`'s arrival request: the moment a player walks into a
//! place, as the prompt the arrival writer is sent. It builds the request and
//! sends nothing.

use crate::data;
use crate::moment::one_toll;
use crate::playthrough::Game;
use crate::records::{flag, id, int, string, Records, Row};
use crate::schemas;
use serde_json::{json, Value};

/// How many minutes a doorway takes, by its distance
/// (`LocationConnection::DISTANCES`).
pub const DISTANCES: [(&str, i64); 5] = [
    ("adjacent", 1),
    ("a short walk", 5),
    ("across the district", 20),
    ("a long journey", 120),
    ("days away", 2880),
];

/// And by how it is covered, as a factor on walking
/// (`LocationConnection::TRAVEL_METHODS`).
pub const TRAVEL_METHODS: [(&str, f64); 7] = [
    ("walking", 1.0),
    ("taking stairs", 1.5),
    ("climbing", 3.0),
    ("crawling", 4.0),
    ("swimming", 2.5),
    ("rowing", 0.8),
    ("riding", 0.4),
];

/// `LocationConnection.travel_minutes`.
pub fn travel_minutes(distance: &str, method: &str) -> Option<f64> {
    let base = DISTANCES.iter().find(|(key, _)| *key == distance)?.1;
    let factor = TRAVEL_METHODS.iter().find(|(key, _)| *key == method)?.1;
    Some(base as f64 * factor)
}

/// Universe fields as a prompt reads them (`Universe#prompt_details`), for
/// one audience's list of fields.
pub fn prompt_details(records: &Records, universe: &Row, fields: &[&str]) -> String {
    let races: Vec<&Row> = records.select("races", |race| {
        int(race, "universe_id") == Some(id(universe))
    });
    let lines: Vec<String> = fields
        .iter()
        .map(|field| match *field {
            "races" => format!(
                "races:\n{}",
                races
                    .iter()
                    .map(|race| format!(
                        "{} -- {}",
                        string(race, "name"),
                        string(race, "description")
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            "race_names" => format!(
                "races: {}",
                races
                    .iter()
                    .map(|race| string(race, "name"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            other => format!("{other}: {}", string(universe, other)),
        })
        .collect();
    lines.join("\n") + "\n"
}

/// Rails' `distance_of_time_in_words` for a span of seconds, up to a year.
pub fn distance_of_time_in_words(seconds: f64) -> String {
    let minutes = (seconds.abs() / 60.0).round() as i64;
    match minutes {
        0 => "less than a minute".into(),
        1 => "1 minute".into(),
        2..=44 => format!("{minutes} minutes"),
        45..=89 => "about 1 hour".into(),
        90..=1439 => format!("about {} hours", (minutes as f64 / 60.0).round() as i64),
        1440..=2519 => "1 day".into(),
        2520..=43199 => format!("{} days", (minutes as f64 / 1440.0).round() as i64),
        43200..=86399 => format!(
            "about {} month{}",
            (minutes as f64 / 43200.0).round() as i64,
            {
                if (minutes as f64 / 43200.0).round() as i64 == 1 {
                    ""
                } else {
                    "s"
                }
            }
        ),
        86400..=525599 => format!("{} months", (minutes as f64 / 43200.0).round() as i64),
        _ => unimplemented!("a span of a year or more"),
    }
}

/// The arrival being written: which room, from which scene, in which game.
pub struct Arrival<'a> {
    pub records: &'a Records,
    pub location: &'a Row,
    pub previous_scene: Option<&'a Row>,
    /// None for an arrival no game is standing in, such as a story's opening.
    pub game: Option<Game<'a>>,
    pub opening: bool,
}

impl<'a> Arrival<'a> {
    fn story(&self) -> &'a Row {
        self.records
            .find("stories", int(self.location, "story_id").unwrap())
            .expect("the room's story")
    }

    fn story_protagonist(&self) -> Option<&'a Row> {
        let story = int(self.location, "story_id");
        self.records.first("characters", |row| {
            int(row, "story_id") == story && flag(row, "is_protagonist")
        })
    }

    fn origin(&self) -> Option<&'a Row> {
        match self.game {
            Some(game) => game.current_location(),
            None => self
                .previous_scene
                .and_then(|scene| int(scene, "location_id"))
                .and_then(|id| self.records.find("locations", id)),
        }
    }

    /// `#story_timestamp`: when the player arrives.
    pub fn story_timestamp(&self) -> f64 {
        let story = self.story();
        if self.opening {
            return int(story, "start_time").unwrap() as f64;
        }
        let Some(previous) = self.previous_scene else {
            let at = self
                .records
                .select("scenes", |scene| int(scene, "story_id") == Some(id(story)))
                .iter()
                .filter_map(|scene| int(scene, "story_timestamp"))
                .max()
                .or_else(|| int(story, "start_time"));
            return at.unwrap() as f64;
        };
        let origin = self.origin().map(id);
        let edge = self.records.first("location_connections", |edge| {
            int(edge, "location_id") == origin
                && int(edge, "connected_location_id") == Some(id(self.location))
        });
        let adjacent = DISTANCES[0].1 as f64;
        let minutes = edge
            .and_then(|edge| {
                travel_minutes(string(edge, "distance"), string(edge, "travel_method"))
            })
            .unwrap_or(adjacent);
        int(previous, "story_timestamp").unwrap() as f64 + minutes * 60.0
    }

    /// `#characters_present`: the cast the arrival introduces.
    pub fn characters_present(&self) -> Vec<&'a Row> {
        if let Some(game) = self.game {
            return game.cast_on_arrival(self.location);
        }
        let story = int(self.location, "story_id");
        let mut present: Vec<&'a Row> = Vec::new();
        let mut add = |row: &'a Row| {
            if !present.iter().any(|seen| id(seen) == id(row)) {
                present.push(row);
            }
        };
        if let Some(protagonist) = self.story_protagonist() {
            add(protagonist);
        }
        for row in self.records.select("characters", |row| {
            int(row, "story_id") == story && flag(row, "is_companion")
        }) {
            add(row);
        }
        for row in self.records.select("characters", |row| {
            int(row, "location_id") == Some(id(self.location))
        }) {
            add(row);
        }
        present
    }

    fn cast_list(&self, cast: &[&Row]) -> String {
        let protagonist = self.story_protagonist().map(id);
        let lines: Vec<String> = cast
            .iter()
            .map(|character| {
                let race = int(character, "race_id")
                    .and_then(|race| self.records.find("races", race))
                    .map(|race| string(race, "name"))
                    .unwrap_or("");
                let line = format!(
                    "{} ({}), {race}",
                    string(character, "fullname"),
                    string(character, "nickname")
                );
                if Some(id(character)) == protagonist {
                    format!("{line} -- the player, the one arriving")
                } else {
                    line
                }
            })
            .collect();
        if !lines.is_empty() {
            return lines.join("\n");
        }
        if self.game.is_some() {
            "Nobody is alive here.".into()
        } else {
            "Nobody but the player.".into()
        }
    }

    fn exit_names(&self) -> String {
        let here = Some(id(self.location));
        let names: Vec<&str> = self
            .records
            .select("location_connections", |edge| {
                int(edge, "location_id") == here
            })
            .iter()
            .map(|edge| {
                string(
                    self.records
                        .find("locations", int(edge, "connected_location_id").unwrap())
                        .unwrap(),
                    "name",
                )
            })
            .collect();
        if names.is_empty() {
            "None written yet.".into()
        } else {
            names.join(", ")
        }
    }

    fn lead_in(&self) -> String {
        let Some(previous) = self.previous_scene else {
            return "Nothing. This is where the story opens.".into();
        };
        let coming_from = match self.origin() {
            Some(from) if id(from) != id(self.location) => {
                format!("The player has come from {}. ", string(from, "name"))
            }
            _ => String::new(),
        };
        let summary = crate::text::presence(crate::records::text(previous, "summary"))
            .unwrap_or_else(|| string(previous, "description"));
        format!("{coming_from}{summary}")
    }

    fn arrival_instructions(&self, at: f64) -> String {
        match int(self.location, "last_protagonist_visit") {
            None => data::scene_generator("arrival_first").to_string(),
            Some(visit) => data::scene_generator("arrival_returning")
                .replace("%{elapsed}", &distance_of_time_in_words(at - visit as f64)),
        }
    }

    /// `Scene::ArrivalContext#facts`: what the records say on the way in.
    pub fn facts(&self) -> Option<Vec<String>> {
        let game = self.game?;
        let mut parts = Vec::new();
        let player = game.protagonist();
        if let Some(condition) = player.and_then(|who| game.vitals_for(who)) {
            parts.push(format!("You are {}.", condition.in_words()));
        }
        let names = |people: &[&Row]| {
            people
                .iter()
                .map(|who| string(who, "fullname"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let others: Vec<&Row> = game
            .cast_on_arrival(self.location)
            .into_iter()
            .filter(|who| Some(id(who)) != player.map(id))
            .collect();
        parts.push(if others.is_empty() {
            "Nobody else is alive here.".into()
        } else {
            format!("Also here: {}. Nobody else is alive here.", names(&others))
        });
        for person in &others {
            if let Some(condition) = game.vitals_for(person).filter(|c| !c.unhurt()) {
                parts.push(format!(
                    "{} is {}.",
                    string(person, "fullname"),
                    condition.in_words()
                ));
            }
        }
        let dead: Vec<&Row> = game
            .characters_located_in(self.location)
            .into_iter()
            .filter(|who| game.vitals_for(who).is_some_and(|c| c.dead()))
            .collect();
        if !dead.is_empty() {
            parts.push(format!(
                "Dead here: {}. They cannot speak or act.",
                names(&dead)
            ));
        }
        let items = |items: Vec<&Row>| {
            let names: Vec<&str> = items.iter().map(|item| string(item, "name")).collect();
            if names.is_empty() {
                "nothing".to_string()
            } else {
                names.join(", ")
            }
        };
        parts.push(format!(
            "Lying here: {}.",
            items(game.items_lying_in(Some(self.location)))
        ));
        parts.push(format!("You are carrying: {}.", items(game.carried())));
        for toll in game
            .own("playthrough_tolls")
            .into_iter()
            .filter(|toll| int(toll, "scene_id").is_none())
        {
            parts.push(one_toll(&game, toll));
        }
        Some(parts)
    }

    /// `#arrival_prompt`.
    pub fn prompt(&self) -> String {
        let story = self.story();
        let universe = self
            .records
            .find("universes", int(story, "universe_id").unwrap())
            .expect("the story's universe");
        let at = self.story_timestamp();
        let mut prompt = format!(
            "## Universe Details\n{}\n## Story Details\ntitle: {}\ngenre: {}\nsummary: {}\n\n\
             ## The Place\nname: {}\ndescription: {}\nlore: {}\nways out: {}\n\n\
             ## Who Is Here\n{}\n\n## Just Before This\n{}\n\n## Instructions\n{}\n\
             - Address the player as \"you\", in the present tense\n\
             - One paragraph. Do not re-describe the place item by item -- the\n  \
             description above is already what is here, and your job is the moment\n  \
             of coming into it\n\
             - Anyone listed above is here; write them as already present, not as\n  \
             arriving. Do not add a person who is not on that list\n\
             - Do not name a way out that is not on the list above\n\
             - Respect the stated length of each field\n",
            prompt_details(self.records, universe, &["physics", "technology"]),
            string(story, "title"),
            string(story, "genre"),
            string(story, "summary"),
            string(self.location, "name"),
            string(self.location, "description"),
            string(self.location, "lore"),
            self.exit_names(),
            self.cast_list(&self.characters_present()),
            self.lead_in(),
            self.arrival_instructions(at),
        );
        if let Some(facts) = self.facts() {
            prompt.push_str(&format!(
                "\n## Current State On Arrival\n\
                 The place description is the world's original account. These current records\n\
                 take precedence over its claims about people, items and wounds. Narrate the\n\
                 recorded crossing result as part of this arrival.\n{}\n",
                facts.join("\n")
            ));
        }
        prompt
    }

    /// The request as a kept set stores it: `{system, user, schema, history}`.
    pub fn request(&self) -> Value {
        json!({
            "system": data::scene_generator("system_prompt"),
            "user": self.prompt(),
            "schema": schemas::scene(),
            "history": [],
        })
    }
}
