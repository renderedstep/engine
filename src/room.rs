//! The room a playthrough stands in, as plain values: the ways out, who is
//! here, what is lying here and what the party carries. Every line-reading
//! rule reads these four closed sets and nothing else.
//!
//! A record is compared by what it is and its id, the way the Ruby engine's
//! records are, so two things with one name are still two things.

/// A place: the room itself, or the room at the far side of a way out.
#[derive(Clone, Debug)]
pub struct Place {
    pub id: i64,
    pub name: String,
}

/// A person. The nickname is the only second name any record answers to.
#[derive(Clone, Debug)]
pub struct Person {
    pub id: i64,
    pub fullname: String,
    pub nickname: Option<String>,
}

/// A thing, lying on this game's floor or in this game's hands.
#[derive(Clone, Debug)]
pub struct Thing {
    pub id: i64,
    pub name: String,
    /// A key into [`BULK`]; `handy` unless the world says otherwise.
    pub bulk: String,
    /// One of [`USE_KINDS`]; `ordinary` unless the world says otherwise.
    pub use_kind: String,
    pub combustible: bool,
    /// In the party's hands rather than lying in the room.
    pub carried: bool,
}

/// How hard a thing is to pick up and throw: the penalty taken off a
/// thrower's strength, and `None` for a thing that does not move at all.
pub const BULK: &[(&str, Option<i64>)] = &[
    ("light", Some(0)),
    ("handy", Some(2)),
    ("heavy", Some(5)),
    ("immovable", None),
];

pub const USE_KINDS: &[&str] = &[
    "ordinary",
    "food",
    "drink",
    "healing",
    "firestarter",
    "lever",
    "lockpick",
    "key",
];

const CONSUMABLES: &[&str] = &["food", "drink", "healing"];

impl Thing {
    /// A thing with the columns' defaults.
    pub fn new(id: i64, name: &str, carried: bool) -> Thing {
        Thing {
            id,
            name: name.to_string(),
            bulk: "handy".into(),
            use_kind: "ordinary".into(),
            combustible: false,
            carried,
        }
    }

    /// Whether it can leave a pair of hands at all. A bulk outside the table
    /// reads as a thing that does not move.
    pub fn throwable(&self) -> bool {
        BULK.iter()
            .any(|(bulk, penalty)| *bulk == self.bulk && penalty.is_some())
    }

    pub fn consumable(&self) -> bool {
        CONSUMABLES.contains(&self.use_kind.as_str())
    }

    /// The name without an article it arrived with ("a frayed cable tie").
    pub fn bare_name(&self) -> &str {
        strip_leading_article(&self.name)
    }

    /// What an engine sentence calls it.
    pub fn definite_name(&self) -> String {
        format!("the {}", self.bare_name())
    }
}

/// `\A(?:a|an|the)\s+(?=\S)`, case-insensitive: an article, spaces, and
/// something that is not a space after them.
fn strip_leading_article(name: &str) -> &str {
    for article in ["a", "an", "the"] {
        let Some(head) = name.get(..article.len()) else {
            continue;
        };
        if !head.eq_ignore_ascii_case(article) {
            continue;
        }
        let rest = &name[article.len()..];
        let trimmed = rest.trim_start_matches(crate::text::is_ruby_space);
        if trimmed.len() < rest.len() && !trimmed.is_empty() {
            return trimmed;
        }
    }
    name
}

/// A way out of the room: the place beyond it and the doorway's barrier.
/// The doorway carries the place's id.
#[derive(Clone, Debug)]
pub struct Exit {
    pub place: Place,
    /// `open`, or `jammed` until somebody forces or prises it. A keyed
    /// doorway, which needs a key template to open, is not modelled here yet.
    pub barrier: String,
}

/// One physical attempt the room offers, built from this game's own things
/// and doorways. A reader picks one; it never supplies an effect.
#[derive(Clone, Debug)]
pub struct Choice {
    pub kind: String,
    pub item: Option<Thing>,
    pub recipient: Option<Person>,
    pub connection: Option<Exit>,
    pub tool: Option<Thing>,
}

impl PartialEq for Choice {
    fn eq(&self, other: &Choice) -> bool {
        self.token() == other.token()
    }
}

impl Choice {
    /// `use:<kind>:<item>:<recipient>:<doorway>:<tool>`, 0 for a part it
    /// does not have.
    pub fn token(&self) -> String {
        let ids = [
            self.item.as_ref().map(|t| t.id),
            self.recipient.as_ref().map(|p| p.id),
            self.connection.as_ref().map(|e| e.place.id),
            self.tool.as_ref().map(|t| t.id),
        ];
        let mut parts = vec!["use".to_string(), self.kind.clone()];
        parts.extend(ids.iter().map(|id| id.unwrap_or(0).to_string()));
        parts.join(":")
    }

    /// What the attempt acts on: the thing, or the place beyond the doorway.
    pub fn subject(&self) -> Option<Record> {
        if let Some(item) = &self.item {
            return Some(Record::Thing(item.clone()));
        }
        self.connection
            .as_ref()
            .map(|exit| Record::Place(exit.place.clone()))
    }

    /// Every record the attempt binds.
    pub fn records(&self) -> Vec<Record> {
        let mut records = Vec::new();
        if let Some(item) = &self.item {
            records.push(Record::Thing(item.clone()));
        }
        if let Some(person) = &self.recipient {
            records.push(Record::Person(person.clone()));
        }
        if let Some(exit) = &self.connection {
            records.push(Record::Place(exit.place.clone()));
        }
        if let Some(tool) = &self.tool {
            records.push(Record::Thing(tool.clone()));
        }
        records
    }

    /// What follows the attempt's own word in a typed line.
    pub fn argument(&self) -> String {
        if self.kind == "offer" {
            return format!("{} to {}", self.item_name(), self.recipient_name());
        }
        let target = self.subject().map(|s| s.label()).unwrap_or_default();
        match &self.tool {
            Some(tool) => format!("{target} with {}", tool.name),
            None => target,
        }
    }

    /// The attempt in words, as a reader is offered it.
    pub fn name(&self) -> String {
        let place = || {
            self.connection
                .as_ref()
                .map(|e| e.place.name.clone())
                .unwrap_or_default()
        };
        let tool = || {
            self.tool
                .as_ref()
                .map(|t| t.name.clone())
                .unwrap_or_default()
        };
        match self.kind.as_str() {
            "consume" => format!("Consume {}", self.item_name()),
            "offer" => format!(
                "Offer {} to {}; they may refuse",
                self.item_name(),
                self.recipient_name()
            ),
            "burn" => format!("Burn {} with {}", self.item_name(), tool()),
            "unlock" => format!("Unlock the way to {} with {}", place(), tool()),
            "pick" => format!("Pick the lock to {} with {}", place(), tool()),
            "pry" => format!("Pry open the way to {} with {}", place(), tool()),
            "force" => format!("Force open the jammed way to {}", place()),
            _ => String::new(),
        }
    }

    fn item_name(&self) -> String {
        self.item
            .as_ref()
            .map(|t| t.name.clone())
            .unwrap_or_default()
    }

    fn recipient_name(&self) -> String {
        self.recipient
            .as_ref()
            .map(|p| p.fullname.clone())
            .unwrap_or_default()
    }
}

/// Anything a line can name: a place, a person, a thing, or one whole
/// physical attempt.
#[derive(Clone, Debug)]
pub enum Record {
    Place(Place),
    Person(Person),
    Thing(Thing),
    Attempt(Box<Choice>),
}

impl PartialEq for Record {
    fn eq(&self, other: &Record) -> bool {
        match (self, other) {
            (Record::Place(a), Record::Place(b)) => a.id == b.id,
            (Record::Person(a), Record::Person(b)) => a.id == b.id,
            (Record::Thing(a), Record::Thing(b)) => a.id == b.id,
            (Record::Attempt(a), Record::Attempt(b)) => a == b,
            _ => false,
        }
    }
}

impl Record {
    /// The name a record answers to, as the player would type it: a person's
    /// full name, and the name of anything else.
    pub fn label(&self) -> String {
        match self {
            Record::Place(place) => place.name.clone(),
            Record::Person(person) => person.fullname.clone(),
            Record::Thing(thing) => thing.name.clone(),
            Record::Attempt(choice) => choice.name(),
        }
    }

    /// The record's id; an attempt has none of its own.
    pub fn id(&self) -> Option<i64> {
        match self {
            Record::Place(place) => Some(place.id),
            Record::Person(person) => Some(person.id),
            Record::Thing(thing) => Some(thing.id),
            Record::Attempt(_) => None,
        }
    }

    /// What an engine sentence calls it: "the" and its name, less any
    /// article the name arrived with.
    pub fn definite_name(&self) -> String {
        match self {
            Record::Thing(thing) => thing.definite_name(),
            other => format!("the {}", other.label()),
        }
    }

    /// Whether the party is holding it.
    pub fn carried(&self) -> bool {
        matches!(self, Record::Thing(thing) if thing.carried)
    }

    pub fn thing(&self) -> Option<&Thing> {
        match self {
            Record::Thing(thing) => Some(thing),
            _ => None,
        }
    }

    pub fn attempt(&self) -> Option<&Choice> {
        match self {
            Record::Attempt(choice) => Some(choice),
            _ => None,
        }
    }
}

/// The intents a reader may answer, in the order the model's table lists
/// them.
pub const INTENTS: &[&str] = &[
    "move", "talk", "examine", "take", "drop", "attack", "use", "other", "throw",
];

/// The answer for "the line named nothing on any list".
pub const NOTHING: &str = "nothing";

/// A playthrough standing in one room.
#[derive(Clone, Debug, Default)]
pub struct Room {
    /// The player character, or `None` for a story nobody plays.
    pub protagonist: Option<Person>,
    /// Where the playthrough stands, or `None` for nowhere.
    pub here: Option<Place>,
    pub exits: Vec<Exit>,
    /// Who is standing here, the player not among them.
    pub cast: Vec<Person>,
    pub lying: Vec<Thing>,
    pub carried: Vec<Thing>,
    /// The game has ended.
    pub over: bool,
}

impl Room {
    /// The ways out of where the player stands.
    pub fn exits_here(&self) -> Vec<Record> {
        if self.here.is_none() {
            return Vec::new();
        }
        self.exits
            .iter()
            .map(|exit| Record::Place(exit.place.clone()))
            .collect()
    }

    /// Who the player can speak to or strike.
    pub fn characters_here(&self) -> Vec<Record> {
        if self.here.is_none() {
            return Vec::new();
        }
        let protagonist = self.protagonist.as_ref().map(|p| p.id);
        self.cast
            .iter()
            .filter(|person| Some(person.id) != protagonist)
            .map(|person| Record::Person(person.clone()))
            .collect()
    }

    /// What the player can pick up.
    pub fn items_here(&self) -> Vec<Record> {
        if self.here.is_none() {
            return Vec::new();
        }
        self.lying.iter().cloned().map(Record::Thing).collect()
    }

    /// What the player carries.
    pub fn items_carried(&self) -> Vec<Record> {
        self.carried.iter().cloned().map(Record::Thing).collect()
    }

    /// The closed set one action reads against; empty for an action that
    /// reads none.
    pub fn offered_for(&self, action: &str) -> Vec<Record> {
        match action {
            "move" => self.exits_here(),
            "talk" | "attack" => self.characters_here(),
            "take" => self.items_here(),
            "drop" => self.items_carried(),
            "examine" => [self.items_here(), self.items_carried()].concat(),
            "use" => self
                .physical_actions()
                .into_iter()
                .map(|choice| Record::Attempt(Box::new(choice)))
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Every physical attempt this room offers, in the order the engine
    /// lists them: consuming, offering, burning, then each closed doorway.
    pub fn physical_actions(&self) -> Vec<Choice> {
        if self.protagonist.is_none() || self.here.is_none() || self.over {
            return Vec::new();
        }
        let carried = &self.carried;
        let cast: Vec<&Person> = self
            .cast
            .iter()
            .filter(|p| Some(p.id) != self.protagonist.as_ref().map(|q| q.id))
            .collect();
        let choice = |kind: &str| Choice {
            kind: kind.to_string(),
            item: None,
            recipient: None,
            connection: None,
            tool: None,
        };
        let mut result = Vec::new();
        for item in carried.iter().filter(|item| item.consumable()) {
            result.push(Choice {
                item: Some(item.clone()),
                ..choice("consume")
            });
        }
        for item in carried {
            for person in &cast {
                result.push(Choice {
                    item: Some(item.clone()),
                    recipient: Some((*person).clone()),
                    ..choice("offer")
                });
            }
        }
        for tool in carried.iter().filter(|t| t.use_kind == "firestarter") {
            for item in carried.iter().chain(&self.lying).filter(|i| i.combustible) {
                if item.id != tool.id {
                    result.push(Choice {
                        item: Some(item.clone()),
                        tool: Some(tool.clone()),
                        ..choice("burn")
                    });
                }
            }
        }
        for edge in &self.exits {
            if edge.barrier == "jammed" {
                result.push(Choice {
                    connection: Some(edge.clone()),
                    ..choice("force")
                });
                for tool in carried.iter().filter(|t| t.use_kind == "lever") {
                    result.push(Choice {
                        connection: Some(edge.clone()),
                        tool: Some(tool.clone()),
                        ..choice("pry")
                    });
                }
            }
        }
        result
    }
}
