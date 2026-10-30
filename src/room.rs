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
    /// A key into `bulk` in `data/physics.yml`; `handy` unless the world says otherwise.
    pub bulk: String,
    /// One of [`USE_KINDS`]; `ordinary` unless the world says otherwise.
    pub use_kind: String,
    pub combustible: bool,
    /// In the party's hands rather than lying in the room.
    pub carried: bool,
    /// The world's own row this is a game's copy of, where it is one.
    pub template: Option<i64>,
    /// What a fixed piece holds (`Item::HOLDS`); none for a portable thing.
    pub holds: Option<String>,
    /// This game has searched it (`items.searched_at`).
    pub searched: bool,
    /// The fixed piece a part is attached to, by id and name; none for
    /// anything that is not a part.
    pub piece: Option<(i64, String)>,
    /// How hard a part holds on (`hold_penalty` in `data/item/kits.yml`).
    pub hold: Option<String>,
    /// The place on a body it is worn on (`items.wear`); none for a thing
    /// nobody wears.
    pub wear: Option<String>,
    /// The party is wearing it (`items.worn`): a kind of carried.
    pub worn: bool,
    /// What it is made of (`items.material`); none where that is unknown.
    pub material: Option<String>,
    /// Fixed in place (`items.tier` `fixture`): it is never broken apart.
    pub fixture: bool,
    /// A step of an open arc this game has not reached names it, so it is
    /// neither broken apart nor burned ([`crate::divide`]).
    pub bound: bool,
}

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

/// The attempt that lets time pass where the player stands: waiting,
/// resting, idling, thinking or praying. It binds no record and is always
/// within reach, so every room offers it, last.
pub const WAIT: &str = "wait";

/// The attempt that puts a thing on: one the party carries, or one lying
/// here that can be picked up, which the same act picks up first.
pub const WEAR: &str = "wear";

/// The attempt that takes a worn thing off. It stays in the party's hands.
pub const REMOVE: &str = "remove";

/// The attempts no reader is handed until the readers are retrained on
/// them (the owner's calls of 2026-10-25): each is reached through the
/// grammar alone.
pub const GRAMMAR_ONLY: &[&str] = &[
    WEAR,
    REMOVE,
    WAIT,
    crate::divide::TEAR,
    crate::divide::SMASH,
];

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
            template: None,
            holds: None,
            searched: false,
            piece: None,
            hold: None,
            wear: None,
            worn: false,
            material: None,
            fixture: false,
            bound: false,
        }
    }

    /// The place on a body it would be worn on, where that is one of the
    /// places `data/item/wear.yml` lists.
    pub fn wear_place(&self) -> Option<&str> {
        self.wear
            .as_deref()
            .filter(|place| crate::wear::is_place(place))
    }

    /// Whether a search would find anything this game has not looked in: a
    /// closed piece, or a hollow one, nobody in this game has searched.
    pub fn searchable(&self) -> bool {
        !self.searched && matches!(self.holds.as_deref(), Some("closed" | "hollow"))
    }

    /// Whether it is shut and could be opened: a closed piece this game has
    /// not searched.
    pub fn openable(&self) -> bool {
        !self.searched && self.holds.as_deref() == Some("closed")
    }

    /// Whether it can leave a pair of hands at all. A bulk outside the table
    /// reads as a thing that does not move.
    pub fn throwable(&self) -> bool {
        crate::data::physics()
            .bulk
            .iter()
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

    /// What a sentence in the engine's own words calls it
    /// ([`crate::moment::own_name`]).
    pub fn own_name(&self) -> String {
        crate::moment::own_name(&self.name)
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
#[derive(Clone, Debug)]
pub struct Exit {
    pub place: Place,
    /// `open` for this game; otherwise `jammed` until somebody forces or
    /// prises it, or `keyed` until somebody unlocks it or picks its lock.
    pub barrier: String,
    /// The doorway's own id.
    pub edge: i64,
    /// The key template a keyed doorway opens to.
    pub key: Option<i64>,
}

/// A way out onto a floor that has fallen in this game, or that floor
/// itself: never offered, and a line that names it is refused in the
/// engine's words ([`crate::refusal::fallen`]).
#[derive(Clone, Debug)]
pub struct Fallen {
    pub place: Place,
    /// The fallen floor's name.
    pub floor: String,
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
            self.connection.as_ref().map(|e| e.edge),
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
        if self.kind == "break" {
            return format!("{} off {}", self.item_name(), self.piece_name());
        }
        if self.kind == WAIT {
            return String::new();
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
            crate::divide::TEAR => format!("Tear {}", self.item_definite()),
            crate::divide::SMASH => format!("Smash {}", self.item_definite()),
            "unlock" => format!("Unlock the way to {} with {}", place(), tool()),
            "pick" => format!("Pick the lock to {} with {}", place(), tool()),
            "pry" => format!("Pry open the way to {} with {}", place(), tool()),
            "force" => format!("Force open the jammed way to {}", place()),
            "search" => format!("Search {}", self.item_definite()),
            "open" => format!("Open {}", self.item_definite()),
            "break" => format!(
                "Break {} off the {}",
                self.item_definite(),
                self.piece_name()
            ),
            WAIT => "Wait and let time pass".into(),
            WEAR => format!("Put on {}", self.item_definite()),
            REMOVE => format!("Take off {}", self.item_definite()),
            _ => String::new(),
        }
    }

    /// The fixed piece a `break` would take its part off, as a record a
    /// line can name.
    pub fn piece(&self) -> Option<Record> {
        let (id, name) = self.item.as_ref()?.piece.clone()?;
        Some(Record::Thing(Thing::new(id, &name, false)))
    }

    fn piece_name(&self) -> String {
        self.item
            .as_ref()
            .and_then(|t| t.piece.as_ref())
            .map(|(_, name)| strip_leading_article(name).to_string())
            .unwrap_or_default()
    }

    fn item_definite(&self) -> String {
        self.item
            .as_ref()
            .map(Thing::definite_name)
            .unwrap_or_default()
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
    /// The ways out of here onto a floor that has fallen in this game, and
    /// each such floor, kept out of [`Room::exits`].
    pub fallen: Vec<Fallen>,
    /// Who is standing here, the player not among them.
    pub cast: Vec<Person>,
    pub lying: Vec<Thing>,
    pub carried: Vec<Thing>,
    /// What is attached to the pieces lying here and could be broken off
    /// them. In no closed set of its own: [`Room::physical_actions`] offers
    /// a `break` for each sort.
    pub parts: Vec<Thing>,
    /// The player character's abilities and their scores, empty for a body
    /// with none: what a check an attempt would take is held against.
    pub abilities: Vec<(String, i64)>,
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

    /// The ways out onto a fallen floor, as records a line could name.
    pub fn fallen_here(&self) -> Vec<Record> {
        self.fallen
            .iter()
            .map(|fallen| Record::Place(fallen.place.clone()))
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

    /// The physical attempts a reader is handed: every one the room offers
    /// but the [`GRAMMAR_ONLY`] ones (`wear`, `remove`, `wait`, `tear` and
    /// `smash`), which join the readers' lists with their retrain (the
    /// owner's calls of 2026-10-25). Until then they are reached through the
    /// grammar alone, and every reader request stays as it was.
    pub fn reader_actions(&self) -> Vec<Choice> {
        let mut choices = self.physical_actions();
        choices.retain(|choice| !GRAMMAR_ONLY.contains(&choice.kind.as_str()));
        choices
    }

    /// The closed set a reader is handed for one action: [`Room::offered_for`],
    /// less the [`GRAMMAR_ONLY`] attempts ([`Room::reader_actions`]).
    pub fn reader_offered_for(&self, action: &str) -> Vec<Record> {
        if action == "use" {
            return self
                .reader_actions()
                .into_iter()
                .map(|choice| Record::Attempt(Box::new(choice)))
                .collect();
        }
        self.offered_for(action)
    }

    /// Every physical attempt this room offers, in the order the engine
    /// lists them: consuming, offering, burning, tearing and smashing each
    /// thing that may be broken apart ([`crate::divide::offered`]), in the
    /// player's hands and then lying here, each closed doorway in the
    /// order the doorways were written, then each fixed piece here a search
    /// would find something new in, as a `search`, and a shut one as an
    /// `open` beside it, in the order the pieces lie, then a `break` for each
    /// sort of part still attached to a piece lying here, in the order the
    /// pieces lie: one attempt for a tree's branches however many it has
    /// left, on the first of them; then a `wear` for each thing carried,
    /// then lying here, that could be put on now ([`Room::wear_refused`]),
    /// and a `remove` for each worn thing that could be taken off now
    /// ([`Room::remove_refused`]); and last, a `wait`, which needs nothing
    /// and so is always offered.
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
            for item in carried
                .iter()
                .chain(&self.lying)
                .filter(|i| i.combustible && !i.bound)
            {
                if item.id != tool.id {
                    result.push(Choice {
                        item: Some(item.clone()),
                        tool: Some(tool.clone()),
                        ..choice("burn")
                    });
                }
            }
        }
        result.extend(
            self.breaking_apart(|kind, thing| crate::divide::offered(kind, thing, &self.abilities)),
        );
        let mut doorways: Vec<&Exit> = self.exits.iter().collect();
        doorways.sort_by_key(|edge| edge.edge);
        for edge in doorways {
            if edge.barrier == "keyed" {
                for tool in carried {
                    let kind = if tool.use_kind == "key"
                        && tool.template.is_some()
                        && tool.template == edge.key
                    {
                        "unlock"
                    } else if tool.use_kind == "lockpick" {
                        "pick"
                    } else {
                        continue;
                    };
                    result.push(Choice {
                        connection: Some(edge.clone()),
                        tool: Some(tool.clone()),
                        ..choice(kind)
                    });
                }
            }
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
        for piece in self.lying.iter().filter(|thing| thing.searchable()) {
            result.push(Choice {
                item: Some(piece.clone()),
                ..choice("search")
            });
            if piece.openable() {
                result.push(Choice {
                    item: Some(piece.clone()),
                    ..choice("open")
                });
            }
        }
        let mut breaks: Vec<Choice> = Vec::new();
        for piece in &self.lying {
            for part in &self.parts {
                if part.piece.as_ref().map(|(id, _)| *id) != Some(piece.id) {
                    continue;
                }
                let offered = breaks.iter().any(|choice| {
                    choice
                        .item
                        .as_ref()
                        .is_some_and(|other| other.piece == part.piece && other.name == part.name)
                });
                if !offered {
                    breaks.push(Choice {
                        item: Some(part.clone()),
                        ..choice("break")
                    });
                }
            }
        }
        result.extend(breaks);
        for item in carried.iter().chain(&self.lying) {
            if self.wear_refused(item).is_none() {
                result.push(Choice {
                    item: Some(item.clone()),
                    ..choice(WEAR)
                });
            }
        }
        for item in carried.iter().filter(|item| item.worn) {
            if self.remove_refused(item).is_none() {
                result.push(Choice {
                    item: Some(item.clone()),
                    ..choice(REMOVE)
                });
            }
        }
        result.push(choice(WAIT));
        result
    }

    /// What the party wears on a place, if anything.
    pub fn worn_on(&self, place: &str) -> Option<&Thing> {
        self.carried
            .iter()
            .find(|item| item.worn && item.wear_place() == Some(place))
    }

    /// Why a thing carried or lying here cannot be put on now, or `None`
    /// where it can: it is worn nowhere, it is worn already, it is lying
    /// here and cannot be picked up, its place is taken, or the place it
    /// lies under is worn over ([`crate::wear::under`]).
    pub fn wear_refused(&self, item: &Thing) -> Option<Dressing<'_>> {
        let Some(place) = item.wear_place() else {
            return Some(Dressing::Unwearable);
        };
        if !item.carried && !item.throwable() {
            return Some(Dressing::Unwearable);
        }
        if item.worn {
            return Some(Dressing::Worn);
        }
        if let Some(other) = self.worn_on(place) {
            return Some(Dressing::Taken(other));
        }
        crate::wear::under(place)
            .and_then(|above| self.worn_on(above))
            .map(Dressing::Over)
    }

    /// Why a thing carried cannot be taken off now, or `None` where it can:
    /// it is not worn, or the place it lies under is worn over.
    pub fn remove_refused(&self, item: &Thing) -> Option<Dressing<'_>> {
        if !item.worn {
            return Some(Dressing::NotWorn);
        }
        item.wear_place()
            .and_then(crate::wear::under)
            .and_then(|above| self.worn_on(above))
            .map(Dressing::Over)
    }

    /// A `tear` or a `smash` of each thing in the player's hands, then of
    /// each lying here, where `keep` lets it be.
    fn breaking_apart(&self, keep: impl Fn(&str, &Thing) -> bool) -> Vec<Choice> {
        let mut result = Vec::new();
        for thing in self.carried.iter().chain(&self.lying) {
            for kind in crate::divide::KINDS {
                if keep(kind, thing) {
                    result.push(Choice {
                        kind: kind.to_string(),
                        item: Some(thing.clone()),
                        recipient: None,
                        connection: None,
                        tool: None,
                    });
                }
            }
        }
        result
    }

    /// What the beat guard keeps off the list ([`crate::divide`]): each
    /// `tear`, `smash` or `burn` the room would offer if no open beat of the
    /// story were bound to its thing, of a thing one is. A line asking for
    /// one of these is refused in the engine's words.
    pub fn guarded_actions(&self) -> Vec<Choice> {
        if self.protagonist.is_none() || self.here.is_none() || self.over {
            return Vec::new();
        }
        let mut result = Vec::new();
        for tool in self.carried.iter().filter(|t| t.use_kind == "firestarter") {
            for item in self
                .carried
                .iter()
                .chain(&self.lying)
                .filter(|i| i.combustible && i.bound && i.id != tool.id)
            {
                result.push(Choice {
                    kind: "burn".into(),
                    item: Some(item.clone()),
                    recipient: None,
                    connection: None,
                    tool: Some(tool.clone()),
                });
            }
        }
        result.extend(self.breaking_apart(|kind, thing| {
            thing.bound && crate::divide::possible(kind, thing, &self.abilities)
        }));
        result
    }
}

/// Why a `wear` or a `remove` is not offered on a thing.
#[derive(Clone, Copy, Debug)]
pub enum Dressing<'a> {
    /// It names no place on a body, or it is lying here and cannot be
    /// picked up.
    Unwearable,
    /// It is worn already.
    Worn,
    /// It is not worn.
    NotWorn,
    /// Another thing is worn on its place.
    Taken(&'a Thing),
    /// Its place lies under one this thing is worn on.
    Over(&'a Thing),
}
