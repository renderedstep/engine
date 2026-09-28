//! RubyLLM's `to_json_schema` output for every schema a request carries,
//! reproduced key for key: `Schema#to_json_schema` writes `{name,
//! description, schema}`, and a property's keys come in the order
//! ruby_llm-schema's builders write them, with empty ones left out.
//!
//! Each schema below is the Ruby class of the same name, field for field.

use crate::{interior, kind, parameters, population};
use serde_json::{json, Map, Value};

/// A table's keys, in its order.
fn keys<K: AsRef<str>, V>(table: &[(K, V)]) -> Vec<&str> {
    table.iter().map(|(key, _)| key.as_ref()).collect()
}

/// One property of an object, and whether the object requires it.
pub struct Field {
    name: String,
    schema: Value,
    required: bool,
}

pub fn field(name: &str, schema: impl Into<Value>) -> Field {
    Field {
        name: name.into(),
        schema: schema.into(),
        required: true,
    }
}

pub fn optional(name: &str, schema: impl Into<Value>) -> Field {
    Field {
        name: name.into(),
        schema: schema.into(),
        required: false,
    }
}

/// A string property (`string_schema`): type, enum, description, minLength,
/// maxLength.
pub struct Text {
    description: String,
    choices: Option<Vec<String>>,
    max: Option<i64>,
}

pub fn string(description: &str) -> Text {
    Text {
        description: description.into(),
        choices: None,
        max: None,
    }
}

impl Text {
    pub fn choices<S: AsRef<str>>(mut self, choices: &[S]) -> Text {
        self.choices = Some(choices.iter().map(|c| c.as_ref().to_string()).collect());
        self
    }

    pub fn max(mut self, max: i64) -> Text {
        self.max = Some(max);
        self
    }
}

impl From<Text> for Value {
    fn from(text: Text) -> Value {
        let mut schema = Map::new();
        schema.insert("type".into(), json!("string"));
        if let Some(choices) = text.choices {
            schema.insert("enum".into(), json!(choices));
        }
        schema.insert("description".into(), json!(text.description));
        if let Some(max) = text.max {
            schema.insert("maxLength".into(), json!(max));
        }
        Value::Object(schema)
    }
}

pub fn boolean(description: &str) -> Value {
    json!({ "type": "boolean", "description": description })
}

/// An array property (`array_schema`): type, description, items, minItems,
/// maxItems.
pub struct List {
    description: String,
    items: Value,
    min: Option<i64>,
    max: Option<i64>,
}

pub fn array(description: &str, items: Value) -> List {
    List {
        description: description.into(),
        items,
        min: None,
        max: None,
    }
}

impl List {
    pub fn min_items(mut self, min: i64) -> List {
        self.min = Some(min);
        self
    }

    pub fn max_items(mut self, max: i64) -> List {
        self.max = Some(max);
        self
    }
}

impl From<List> for Value {
    fn from(list: List) -> Value {
        let mut schema = Map::new();
        schema.insert("type".into(), json!("array"));
        schema.insert("description".into(), json!(list.description));
        schema.insert("items".into(), list.items);
        if let Some(min) = list.min {
            schema.insert("minItems".into(), json!(min));
        }
        if let Some(max) = list.max {
            schema.insert("maxItems".into(), json!(max));
        }
        Value::Object(schema)
    }
}

fn members(fields: &[Field]) -> (Map<String, Value>, Vec<String>) {
    let properties = fields
        .iter()
        .map(|f| (f.name.clone(), f.schema.clone()))
        .collect();
    let required = fields
        .iter()
        .filter(|f| f.required)
        .map(|f| f.name.clone())
        .collect();
    (properties, required)
}

/// An object property (`object_schema` with a block): type, properties,
/// required, additionalProperties, description.
pub fn object(description: Option<&str>, fields: Vec<Field>) -> Value {
    let (properties, required) = members(&fields);
    let mut schema = Map::new();
    schema.insert("type".into(), json!("object"));
    schema.insert("properties".into(), Value::Object(properties));
    schema.insert("required".into(), json!(required));
    schema.insert("additionalProperties".into(), json!(false));
    if let Some(description) = description {
        schema.insert("description".into(), json!(description));
    }
    Value::Object(schema)
}

/// `Schema.new.to_json_schema` for a schema of this name and these fields.
pub fn to_json_schema(name: &str, fields: Vec<Field>) -> Value {
    let (properties, required) = members(&fields);
    json!({
        "name": name,
        "description": null,
        "schema": {
            "type": "object",
            "properties": properties,
            "required": required,
            "additionalProperties": false,
            "strict": true,
        },
    })
}

/// `Character::PURSUIT_NAMES`: the shapes a desire is pursued in.
pub const PURSUITS: [&str; 7] = [
    "keep", "obtain", "reach", "attend", "avoid", "withhold", "offer",
];

/// `Character::DESIRE_LIMIT`.
pub const DESIRE_LIMIT: i64 = 220;

/// `Character::Desires::CONSCIOUS`.
pub const CONSCIOUS: &str = "The future, standing, relationship, legacy, place or way of life they knowingly organize their choices around and would name if asked what they want from their life. Make it specific to this person and world. Do not substitute the clue, deadline, delivery, payment or room they are dealing with today. One sentence, third person, by name.";

/// `Character::Desires::UNCONSCIOUS`.
pub const UNCONSCIOUS: &str = "The deeper reward their choices have pursued for years and they would deny: the recognition, absolution, dependence, belonging, power or intimacy beneath the conscious account. It explains a repeated pattern in the backstory and pulls at an angle to the conscious desire, not merely today's hidden motive. One sentence, third person, by name.";

/// `Character::Desires::RECOGNIZED`.
pub const RECOGNIZED: &str = "The duty, oath, debt, craft, family burden or bodily discipline they believe they must keep over years whether they want to or not. It predates today's assignment, survives it, and can repeatedly obstruct the conscious desire. One sentence, third person, by name.";

/// `Character::Desires::UNRECOGNIZED`.
pub const UNRECOGNIZED: &str = "The enduring change, truth or relationship their life requires and their repeated pattern prevents them from seeing. A reader can infer it from the backstory. It cannot be completed by one confession, realization or errand: it demands a new way of choosing across the story. If they never move toward it, pursuit of the conscious desire ruins them. It must not be the conscious desire said twice. One sentence, third person, by name.";

/// `Character::Desires::DESIRE_PURSUIT`.
pub const DESIRE_PURSUIT: &str = "Which listed ENGINE ACT could repeatedly advance or protect the CONSCIOUS DESIRE in ordinary rooms. The label is the next-step expression of the larger desire, not its timescale.";

/// `Character::Desires::NEED_PURSUIT`.
pub const NEED_PURSUIT: &str = "Which listed ENGINE ACT could repeatedly move them toward the UNRECOGNIZED NEED in ordinary rooms. It may match desire_pursuit, but a different label should reflect a real conflict rather than manufactured variety.";

/// `Scene::Schema`.
pub fn scene() -> Value {
    to_json_schema(
        "Scene::Schema",
        vec![
            field(
                "description",
                string("What the player experiences as they arrive here, right now. Second person, present tense. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "summary",
                string("What happened in this moment, for the game engine rather than the player. Third person. One sentence.").max(200),
            ),
        ],
    )
}

/// `Story::Schema`.
pub fn story() -> Value {
    to_json_schema(
        "Story::Schema",
        vec![
            field(
                "title",
                string("The title of the story. 2 to 5 words, no subtitle.").max(80),
            ),
            field(
                "genre",
                string("The genre of the story, e.g. 'gothic horror', 'space western'. 1 to 4 words.").max(60),
            ),
            field(
                "preface",
                string("The opening text shown to the player before they take their first action. Second person, addressed as 'you'. One or two paragraphs, 5 to 9 sentences.").max(1800),
            ),
            field(
                "summary",
                string("A summary of the situation the story opens on, written for the game engine rather than the player. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "opening_location_name",
                string("The name of the place the preface opens in, as a player would refer to it. 1 to 4 words, no article.").max(60),
            ),
            field(
                "opening_location_teaser",
                string("A one-line glimpse of that place, the kind a narrator gives before you walk in. Exactly one sentence.").max(160),
            ),
        ],
    )
}

/// `Character::Schema`.
pub fn character() -> Value {
    to_json_schema(
        "Character::Schema",
        vec![
            field(
                "fullname",
                string("Full name of the character. 2 to 4 words.").max(60),
            ),
            field(
                "nickname",
                string("A short nickname the character is known by. 1 or 2 words.").max(30),
            ),
            field(
                "personality",
                string("The character's temperament and how they treat others. 2 to 3 sentences.").max(400),
            ),
            field(
                "appearance",
                string("What the character looks like. 2 to 3 sentences.").max(400),
            ),
            field(
                "likes",
                string("What the character enjoys. A comma separated list of 3 to 5 items.").max(200),
            ),
            field(
                "dislikes",
                string("What the character cannot stand. A comma separated list of 3 to 5 items.").max(200),
            ),
            field(
                "fears",
                string("What the character is afraid of. A comma separated list of 2 to 4 items.").max(200),
            ),
            field(
                "backstory",
                string("The character's life before the story, their motivations and their goals. One paragraph, 4 to 6 sentences.").max(1200),
            ),
            field(
                "conscious_desire",
                string(CONSCIOUS).max(220),
            ),
            field(
                "unconscious_desire",
                string(UNCONSCIOUS).max(220),
            ),
            field(
                "recognized_need",
                string(RECOGNIZED).max(220),
            ),
            field(
                "unrecognized_need",
                string(UNRECOGNIZED).max(220),
            ),
            field(
                "desire_pursuit",
                string(DESIRE_PURSUIT).choices(&PURSUITS),
            ),
            field(
                "need_pursuit",
                string(NEED_PURSUIT).choices(&PURSUITS),
            ),
        ],
    )
}

/// `Character::DesireWriter::Schema`.
pub fn desire_writer() -> Value {
    to_json_schema(
        "Character::DesireWriter::Schema",
        vec![
            field("conscious_desire", string(CONSCIOUS).max(220)),
            field("unconscious_desire", string(UNCONSCIOUS).max(220)),
            field("recognized_need", string(RECOGNIZED).max(220)),
            field("unrecognized_need", string(UNRECOGNIZED).max(220)),
            field("desire_pursuit", string(DESIRE_PURSUIT).choices(&PURSUITS)),
            field("need_pursuit", string(NEED_PURSUIT).choices(&PURSUITS)),
        ],
    )
}

/// `Location::PlaceSchema`.
pub fn location_place() -> Value {
    let mut fields = prose_fields();
    fields.push(optional(
        "place_kind",
        string("What sort of place with rooms this is -- a building, a ship, a station -- as the game will lay out and furnish its rooms: pick the closest. The game decides which rooms it has and what stands in each; you describe it from outside and at its way in.").choices(&kind::buildings()),
    ));
    fields.push(optional(
                "parameters",
                object(
                    Some("What kind of building this is. Every field is optional; leave one out and the game takes the quietest option. Answer for the place you have just described."),
                    vec![
                        optional(
                            "storeys_above",
                            string("How far up it goes, counting the ground floor. Most buildings are one floor.").choices(&keys(&parameters::STOREYS_ABOVE)),
                        ),
                        optional(
                            "storeys_below",
                            string("How far down it goes below the ground floor. Most buildings have nothing under them.").choices(&keys(&parameters::STOREYS_BELOW)),
                        ),
                        optional(
                            "danger",
                            string("How likely a room of it is to hold something that means the player harm. Most buildings are safe.").choices(&keys(&parameters::DANGER)),
                        ),
                        optional(
                            "gradient",
                            string("Whether that gets worse in one direction. Only say so when the place itself makes it true -- a cellar that gets worse the further down you go, a tower that gets worse the higher you climb.").choices(&keys(&parameters::GRADIENT)),
                        ),
                        optional(
                            "hazard",
                            string("What standing in its rooms does to a person, if anything. NONE is the right answer for almost every building: this is not atmosphere, it is a place that takes hit points off whoever walks through it.").choices(&parameters::HAZARDS),
                        ),
                    ],
                ),
            ));
    to_json_schema("Location::PlaceSchema", fields)
}

/// `Location::ExitsSchema`.
pub fn location_exits() -> Value {
    to_json_schema(
        "Location::ExitsSchema",
        vec![
            field(
                "exits",
                array(
                    "The places a player can reach directly from here.",
                    object(
                        None,
                        vec![
                            field(
                                "name",
                                string("The name of the place this exit leads to, as a player would refer to it. 1 to 4 words, no article.").max(60),
                            ),
                            field(
                                "teaser",
                                string("A one-line glimpse of what lies that way, enough to make the player choose it. Exactly one sentence.").max(160),
                            ),
                            field(
                                "distance",
                                string("How far it is. Pick the closest of these; the exact wording does not matter.").choices(&keys(&interior::DISTANCES)),
                            ),
                            field(
                                "travel_method",
                                string("How the player covers that ground. Pick the closest of these. It must read correctly in both directions, because the way back is the same edge.").choices(&["walking", "taking stairs", "climbing", "crawling", "swimming", "rowing", "riding"]),
                            ),
                            optional(
                                "inside",
                                string("NO INSIDE for almost everything you name. Anything else here makes the game build a whole floor plan of rooms there and send the player walking through them, so answer otherwise ONLY for a place that is a BUILDING somebody goes in at a door: an inn, a keep, a counting house, a warren. NO INSIDE for a road, a shore, a clearing, a bridge, a square, a cave mouth, a stair, a courtyard -- and for a room, an office or a hall, which are already somewhere you stand. When it really is a building, pick the size it would really be.").choices(&keys(&parameters::INSIDE)),
                            ),
                            field(
                                "population",
                                string("How many people are in that place. Pick the closest of these words. It is a word and never a number: the engine decides how many people that is.").choices(&keys(&population::BANDS)),
                            ),
                            field(
                                "kind",
                                string("What sort of place that is, as the game will furnish it. Pick the closest of these words; the game decides what stands in a place of that kind and tells you room by room. It is a word and never a list of furniture.").choices(kind::kinds()),
                            ),
                            field(
                                "density",
                                string("How much small stuff is lying about in it: 'sparse' for somewhere kept or empty, 'lived-in' for most places, 'cluttered' for somewhere nobody tidies. A word; the game rolls the count.").choices(kind::densities()),
                            ),
                        ],
                    ),
                )
                .min_items(1)
                .max_items(4),
            ),
        ],
    )
}

/// `Item::InscriptionSchema`.
pub fn item_inscription() -> Value {
    to_json_schema(
        "Item::InscriptionSchema",
        vec![
            field(
                "inscription",
                string("The words written on this thing, exactly as they appear on it, as the player would read them. Not a description of the object and not a narration of reading it -- the text itself. Keep the register and the period of the world. It may be a few words, a line, or a short paragraph.").max(400),
            ),
        ],
    )
}

/// `Quest::Schema`.
pub fn quest() -> Value {
    to_json_schema(
        "Quest::Schema",
        vec![
            field(
                "title",
                string("What this story's quest is called, as a chronicler would name it. 2 to 5 words, no subtitle.").max(80),
            ),
            field(
                "premise",
                string("What the quest is, in one sentence, written for the game engine rather than the player.").max(400),
            ),
            field(
                "steps",
                array(
                    "The beats of the quest, in the order they would most naturally happen. Each one names ONE thing the world must eventually contain. Do not describe how the player gets there and do not invent a route between them.",
                    object(
                        None,
                        vec![
                            field(
                                "summary",
                                string("What this beat asks of the player, in one short line. This is the only sentence about the quest the narrator is ever shown, so write it as a thing to do rather than as a thing that happens.").max(160),
                            ),
                            field(
                                "trigger",
                                string("What kind of thing this beat needs. reach_location: the player has to stand somewhere. speak_to: the player has to talk to somebody. hold_item: the player has to be carrying something.").choices(&["reach_location", "speak_to", "hold_item"]),
                            ),
                            field(
                                "target",
                                string("The NAME of that place, person or thing, as a player would refer to it. It does not exist yet and you are not creating it -- you are saying what the world must come to contain. A place: 1 to 4 words, no article. A person: their full name. A thing: what somebody would call it picking it up.").max(60),
                            ),
                            field(
                                "teaser",
                                string("One sentence about that place, person or thing -- the glimpse a narrator gives before you walk in, or the one line somebody would say about a person. It is what the world is handed if it has to build the thing itself.").max(160),
                            ),
                        ],
                    ),
                )
                .min_items(3)
                .max_items(5),
            ),
            field(
                "outcomes",
                array(
                    "The ways this quest can end. Exactly one of them is the ending the world is built toward; the rest are other ways it could go.",
                    object(
                        None,
                        vec![
                            field(
                                "name",
                                string("A short label for this ending, lower case, hyphens for spaces. 1 to 3 words.").max(40),
                            ),
                            field(
                                "summary",
                                string("The ending itself, in one sentence, in the past tense -- what the player reads when the story closes on it.").max(300),
                            ),
                            field(
                                "is_default",
                                boolean("True for the ONE ending this world is built toward. False for every other."),
                            ),
                        ],
                    ),
                )
                .min_items(2)
                .max_items(4),
            ),
        ],
    )
}

/// `Universe::PhysicalSchema`.
pub fn universe_physical() -> Value {
    to_json_schema(
        "Universe::PhysicalSchema",
        vec![
            field(
                "physics",
                string("How physics works here, including any magic or supernatural forces and their limits. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "technology",
                string("The level and character of technology available. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "weapons",
                string("The weapons that exist and who carries them. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "geographies",
                string("The notable terrain, regions and landmarks of this world. One paragraph, 3 to 5 sentences.").max(900),
            ),
        ],
    )
}

/// `Universe::SocietalSchema`.
pub fn universe_societal() -> Value {
    to_json_schema(
        "Universe::SocietalSchema",
        vec![
            field(
                "races",
                array(
                    "The peoples and species that inhabit this world.",
                    object(
                        None,
                        vec![
                            field(
                                "name",
                                string("The name of this people, as they are known in this world. 1 to 3 words.").max(40),
                            ),
                            field(
                                "description",
                                string("What sets this people apart -- appearance, temperament, standing in the world. 2 to 3 sentences.").max(500),
                            ),
                        ],
                    ),
                )
                .min_items(3)
                .max_items(6),
            ),
            field(
                "civilizations",
                string("The major civilizations, factions and settlements. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "history",
                string("The defining historical events that shaped the present. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "economics",
                string("How wealth, trade and scarcity work here. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "politics",
                string("Who holds power, and the tensions between them. One paragraph, 3 to 5 sentences.").max(900),
            ),
            field(
                "religion",
                string("The beliefs, deities and rituals of this world. One paragraph, 3 to 5 sentences.").max(900),
            ),
        ],
    )
}

/// `Location::DetailSchema::PROSE_FIELDS`: the room's description and lore,
/// shared with `Location::PlaceSchema`.
fn prose_fields() -> Vec<Field> {
    vec![
    field(
        "description",
        string("What the player sees, hears and smells standing in this place right now. Describe THIS place only -- not what neighbours it, not what is visible out of a window or across the way, because the world around it can move. Second person. One paragraph, 4 to 6 sentences.").max(1200),
    ),
    field(
        "lore",
        string("What this place is, who made it and what happened here. Written for the game engine rather than the player. One paragraph, 3 to 5 sentences.").max(900),
    ),
    ]
}

/// `Interaction::Schema`'s six fields.
fn interaction_fields() -> Vec<Field> {
    vec![
    field(
        "pre_thought",
        string("What you thought immediately in response to what was just said or done to you. One sentence, at most 320 characters.").max(320),
    ),
    field(
        "pre_feeling",
        string("What you felt immediately in response to what was just said or done to you. Two or three words, comma separated, at most 120 characters.").max(120),
    ),
    field(
        "action",
        string("What you did and said in response. Speech goes inside quotes. One or two sentences, at most 480 characters.").max(480),
    ),
    field(
        "post_feeling",
        string("What you felt after you took the action. Two or three words, comma separated, at most 120 characters.").max(120),
    ),
    field(
        "post_thought",
        string("What you thought after you took the action. One sentence, at most 320 characters.").max(320),
    ),
    field(
        "inner_resolution",
        string("What you decided to do as a result of this exchange. One sentence, at most 320 characters.").max(320),
    ),
    ]
}

/// `Interaction::Schema`.
pub fn interaction() -> Value {
    to_json_schema("Interaction::Schema", interaction_fields())
}

/// `Interaction::Schema.with_actions(choices)`: the six fields and the
/// engine action a reply may take.
pub fn interaction_with_actions<S: AsRef<str>>(choices: &[S]) -> Value {
    let mut fields = interaction_fields();
    fields.push(field(
        "engine_action",
        string("One immediate action token from the offered list, or none. Choose according to your own motivations; requests need not be accepted.")
            .choices(choices),
    ));
    to_json_schema("character_interaction", fields)
}

/// `Playthrough::IntentSchema::INTENTS`.
pub const INTENTS: [&str; 9] = [
    "move", "talk", "examine", "take", "drop", "attack", "use", "other", "throw",
];

/// `Playthrough::IntentSchema.for(targets)`: the targets stripped, blanks and
/// repeats dropped, and `nothing` last.
pub fn intent(targets: &[&str]) -> Value {
    let mut choices: Vec<String> = Vec::new();
    for target in targets {
        let target = crate::text::ruby_strip(target);
        if !target.is_empty() && !choices.iter().any(|seen| seen == target) {
            choices.push(target.to_string());
        }
    }
    choices.push(NOTHING.to_string());
    to_json_schema(
        "player_intent",
        vec![
        field(
            "intent",
            string("What the player is trying to do. Pick the closest.").choices(&INTENTS),
        ),
        field(
            "target",
            string("What they aimed it at, copied exactly from the lists you were given: a way out for `move`, a person for `talk` or `attack`, a thing lying here for `take`, a thing they are carrying for `drop`, and for `examine` a thing on either of those two lists. Answer `nothing` for anything else, or when they named something that is not on those lists.").choices(&choices),
        ),
        field(
            "also_named",
            string("One more thing on those lists that the player named in the SAME line and that `target` is not already pointing at, copied exactly -- as in \"take the index and the apron\". One line does one thing, so nothing here is acted on; naming it is only how the game says what it is leaving undone. Answer `nothing` when they named one thing or none, which is usual.").choices(&choices),
        ),
        field(
            "thrown_at",
            string("Only for `throw`: who or which way out the thrown thing was aimed at, copied exactly from the people here or the ways out. Answer `nothing` for every other intent, and for a throw aimed at anything not on those two lists -- a wall, a machine, the room.").choices(&choices),
        ),

        ],
    )
}

/// The word every closed choice ends with.
pub const NOTHING: &str = "nothing";

/// `Location::Population::MOST`: the most people a room is written with.
pub const MOST_PEOPLE: i64 = 3;

/// `Location::DetailSchema.for_people(wanted)`: a room, what lies loose in
/// it, and exactly `wanted` people, or up to `MOST_PEOPLE` when none are
/// asked for.
pub fn detail(wanted: i64) -> Value {
    let person = object(
    None,
    vec![
        field(
            "fullname",
            string("Their full name, as a player would type it to speak to them. 2 or 3 words.").max(60),
        ),
        field(
            "nickname",
            string("What they are called to their face. 1 or 2 words.").max(30),
        ),
        field(
            "appearance",
            string("What somebody walking in sees of them, consistent with the room you just wrote and with the race and age you were given for them. One or two sentences.").max(300),
        ),
        field(
            "personality",
            string("How they behave and how they treat a stranger. One or two sentences.").max(300),
        ),
        field(
            "backstory",
            string("The life history that brought them here, the repeated choices it taught them, and why today's presence presses on an enduring aim. Third person, by name. Two or three sentences.").max(450),
        ),
        field(
            "likes",
            string("A comma separated list of 2 or 3 things they enjoy.").max(160),
        ),
        field(
            "dislikes",
            string("A comma separated list of 2 or 3 things they cannot stand.").max(160),
        ),
        field(
            "fears",
            string("A comma separated list of 1 or 2 things they are afraid of.").max(160),
        ),
        field(
            "conscious_desire",
            string(CONSCIOUS).max(180),
        ),
        field(
            "unconscious_desire",
            string(UNCONSCIOUS).max(180),
        ),
        field(
            "recognized_need",
            string(RECOGNIZED).max(180),
        ),
        field(
            "unrecognized_need",
            string(UNRECOGNIZED).max(180),
        ),
        field(
            "desire_pursuit",
            string(DESIRE_PURSUIT).choices(&PURSUITS),
        ),
        field(
            "need_pursuit",
            string(NEED_PURSUIT).choices(&PURSUITS),
        ),
    ],
);
    let described = match wanted {
        1 => "The person who is in this place right now. Write the one the instructions describe.".to_string(),
        n if n > 1 => format!(
            "The {n} people who are in this place right now. Write all {n} the instructions describe, in the order they are given there."
        ),
        _ => "People who are in this place right now.".to_string(),
    };
    let people = array(&described, person);
    let people = if wanted > 0 {
        field("people", people.min_items(wanted).max_items(wanted))
    } else {
        optional("people", people.max_items(MOST_PEOPLE))
    };
    let mut fields = prose_fields();
    fields.extend([
    optional(
        "name",
        string("What this room is called -- ONLY when the instructions above ask you to name it. Leave this out entirely otherwise. A short noun phrase a player would type to walk into it, carrying the article English wants on it: \"the counting room\". Never the name of the building it is in, never a name this story has already given to a room, a person or a thing, and never a comma.").max(60),
    ),
    optional(
        "items",
        array(
            "Portable things lying loose in this place that a player could pick up and carry away. Empty is the right answer for most rooms.",
            object(
                None,
                vec![
                    field(
                        "name",
                        string("What the thing is called, as a player would type it to pick it up. A short noun phrase, 1 to 4 words, lower case unless it is a proper name. Never the name of a person or of a place.").max(60),
                    ),
                    field(
                        "description",
                        string("What it is and what state it is in, consistent with the description of the room you just wrote. One or two sentences.").max(400),
                    ),
                    field(
                        "use_kind",
                        string("The engine's physical profile: ordinary unless it is food, a drink, a healing dose, a firestarter, a prying lever, lockpicks, or a key. Choose healing only for an actual restorative dose allowed by this world. Profiles supply behavior from code, never from the description.").choices(&["ordinary", "food", "drink", "healing", "firestarter", "lever", "lockpick", "key"]),
                    ),
                    field(
                        "combustible",
                        boolean("True only if this portable thing can be destroyed by an ordinary carried firestarter: dry paper, cloth or wood. False for stone, metal, liquids, or anything uncertain."),
                    ),
                    field(
                        "readable",
                        boolean("True only if this thing has WRITING on it that a player could read: a note, a letter, a label, a docket, a page, a sign, an inscription. False for everything else, which is most things."),
                    ),
                    optional(
                        "inscription",
                        string("The words written on it, exactly as they appear, and only when `readable` is true. Write what is actually on the thing -- what a player would read off it -- not a description of it. A few words, a line, or a few short lines; well under the limit, and finished rather than trailing off. Leave this out entirely when nothing is written on it.").max(400),
                    ),
                ],
            ),
        )
        .max_items(3),
    ),
        people,
    ]);
    to_json_schema("Location::DetailSchema", fields)
}
