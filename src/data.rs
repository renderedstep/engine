//! The data files the Ruby engine reads its verb table and its System One
//! wording from, vendored under `data/` byte for byte (see `data/README.md`).
//! They are the one copy of those strings: nothing here retypes them.

use std::sync::OnceLock;
use yaml_rust2::{Yaml, YamlLoader};

const GRAMMAR: &str = include_str!("../data/playthrough/grammar.yml");
const REQUEST: &str = include_str!("../data/playthrough/classifier/request.yml");

fn load(name: &str, text: &str) -> Yaml {
    YamlLoader::load_from_str(text)
        .unwrap_or_else(|error| panic!("{name}: {error}"))
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{name}: no document"))
}

fn string(value: &Yaml, what: &str) -> String {
    value
        .as_str()
        .unwrap_or_else(|| panic!("{what} is not a string"))
        .to_string()
}

/// A map's entries in file order, as strings.
fn pairs(value: &Yaml, what: &str) -> Vec<(String, String)> {
    value
        .as_hash()
        .unwrap_or_else(|| panic!("{what} is not a map"))
        .iter()
        .map(|(key, value)| (string(key, what), string(value, what)))
        .collect()
}

/// The fixed grammar's verb table: each word a player may type, and the verb
/// it is read as, in file order.
pub fn verbs() -> &'static [(String, String)] {
    static VERBS: OnceLock<Vec<(String, String)>> = OnceLock::new();
    VERBS.get_or_init(|| pairs(&load("grammar.yml", GRAMMAR)["verbs"], "verbs"))
}

/// One target question's premise and its `nothing` criterion.
#[derive(Clone, Debug)]
pub struct Target {
    pub action: String,
    pub premise: String,
    pub nothing: String,
}

/// The wording of a System One request, as the Ruby engine sends it.
#[derive(Clone, Debug)]
pub struct RequestTexts {
    pub intent_instructions: String,
    pub intent_criteria: Vec<(String, String)>,
    /// Still carrying its `%{nothing}` placeholders.
    pub target_instructions: String,
    pub targets: Vec<Target>,
    pub also_named_instructions: String,
    pub also_named_nothing: String,
    pub named_more_than_one_instructions: String,
    pub named_more_than_one_criteria: Vec<(String, String)>,
    pub target_present_instructions: String,
    pub target_present_criteria: Vec<(String, String)>,
}

pub fn request_texts() -> &'static RequestTexts {
    static TEXTS: OnceLock<RequestTexts> = OnceLock::new();
    TEXTS.get_or_init(|| {
        let doc = load("request.yml", REQUEST);
        let text = |key: &str| string(&doc[key], key);
        let targets = doc["targets"]
            .as_hash()
            .expect("targets is a map")
            .iter()
            .map(|(action, target)| Target {
                action: string(action, "targets"),
                premise: string(&target["premise"], "premise"),
                nothing: string(&target["nothing"], "nothing"),
            })
            .collect();
        RequestTexts {
            intent_instructions: text("intent_instructions"),
            intent_criteria: pairs(&doc["intent_criteria"], "intent_criteria"),
            target_instructions: text("target_instructions"),
            targets,
            also_named_instructions: text("also_named_instructions"),
            also_named_nothing: text("also_named_nothing"),
            named_more_than_one_instructions: text("named_more_than_one_instructions"),
            named_more_than_one_criteria: pairs(
                &doc["named_more_than_one_criteria"],
                "named_more_than_one_criteria",
            ),
            target_present_instructions: text("target_present_instructions"),
            target_present_criteria: pairs(
                &doc["target_present_criteria"],
                "target_present_criteria",
            ),
        }
    })
}

const SCENE_GENERATOR: &str = include_str!("../data/scene/generator.yml");
const LOCATION_GENERATOR: &str = include_str!("../data/location/generator.yml");
const DESIRES: &str = include_str!("../data/character/desires.yml");
const NARRATOR: &str = include_str!("../data/scene/narrator.yml");
const CLASSIFIER: &str = include_str!("../data/playthrough/classifier.yml");
const ENDING: &str = include_str!("../data/scene/ending.yml");
const INSCRIBER: &str = include_str!("../data/item/inscriber.yml");

/// `config/engine/scene/ending.yml`: the instructions the last paragraph of
/// a game is written under (`Scene::Ending::INSTRUCTIONS`).
pub fn ending_instructions() -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(&FILE, "scene/ending.yml", ENDING, "instructions")
}

/// `config/engine/item/inscriber.yml`: the instructions the words on a
/// readable thing are written under (`Item::Inscriber::INSTRUCTIONS`).
pub fn inscriber_instructions() -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(&FILE, "item/inscriber.yml", INSCRIBER, "instructions")
}

/// `config/engine/playthrough/classifier.yml`: the classifier's
/// instructions.
pub fn classifier_instructions() -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(
        &FILE,
        "playthrough/classifier.yml",
        CLASSIFIER,
        "instructions",
    )
}

/// One string of a vendored file, by key.
fn text_of(file: &'static OnceLock<Yaml>, name: &str, source: &str, key: &str) -> &'static str {
    file.get_or_init(|| load(name, source))[key]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: {key} is not a string"))
}

/// `config/engine/scene/generator.yml`: the arrival writer's words.
pub fn scene_generator(key: &str) -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(&FILE, "scene/generator.yml", SCENE_GENERATOR, key)
}

/// `config/engine/location/generator.yml`: the room writer's words.
pub fn location_generator(key: &str) -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(&FILE, "location/generator.yml", LOCATION_GENERATOR, key)
}

/// `config/engine/character/desires.yml`: the desire writer's words.
pub fn desires(key: &str) -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(&FILE, "character/desires.yml", DESIRES, key)
}

/// `config/engine/scene/narrator.yml`: the narrator's instructions.
pub fn narrator_instructions() -> &'static str {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    text_of(&FILE, "scene/narrator.yml", NARRATOR, "instructions")
}

/// `Scene::Narrator::DOING`: what the narrator is told a kind of turn is, for
/// the kinds that have a sentence.
pub fn narrator_doing(intent: &str) -> Option<&'static str> {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    file_of(&FILE, "scene/narrator.yml", NARRATOR)["doing"][intent].as_str()
}

fn file_of(file: &'static OnceLock<Yaml>, name: &str, source: &str) -> &'static Yaml {
    file.get_or_init(|| load(name, source))
}

const WEIGHTS: &str = include_str!("../data/playthrough/volition/weights.yml");

/// `Playthrough::Volition::Weights`: the weight every act starts from, and
/// each pursuit's extra weight per act shape, in file order.
pub struct Weights {
    pub base: i64,
    pub table: Vec<(String, Vec<(String, i64)>)>,
}

pub fn weights() -> &'static Weights {
    static WEIGHTS_TABLE: OnceLock<Weights> = OnceLock::new();
    WEIGHTS_TABLE.get_or_init(|| {
        let file = load("weights.yml", WEIGHTS);
        let base = file["base"].as_i64().expect("a base weight");
        let table = file["table"]
            .as_hash()
            .expect("a table of pursuits")
            .iter()
            .map(|(pursuit, row)| {
                let row = row
                    .as_hash()
                    .expect("a pursuit's weights")
                    .iter()
                    .map(|(shape, weight)| {
                        (
                            string(shape, "a shape"),
                            weight.as_i64().expect("a whole weight"),
                        )
                    })
                    .collect();
                (string(pursuit, "a pursuit"), row)
            })
            .collect();
        Weights { base, table }
    })
}

const PHYSICS: &str = include_str!("../data/physics.yml");

/// `data/physics.yml`: the physics and item tables. Each table keeps its
/// file order; a `None` is a row the file leaves empty on purpose (a thing
/// that does not move, a thing that never breaks).
// The fall, breakage and range tables have no reader yet.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Physics {
    pub bulk: Vec<(String, Option<i64>)>,
    pub thrown_damage: Vec<(String, i64)>,
    pub gravity: Vec<(String, i64)>,
    pub fall_die: i64,
    pub fall_save: String,
    pub throw_range: Vec<(String, i64)>,
    pub throw_reach_per_strength: i64,
    pub throw_reach_cap: i64,
    pub fragility: Vec<(String, Option<i64>)>,
    pub break_die: i64,
    pub height_step: Vec<(String, i64)>,
    pub surface: Vec<(String, i64)>,
}

/// The file's shape: every top-level key, and nothing else.
const PHYSICS_KEYS: &[&str] = &[
    "bulk",
    "thrown_damage",
    "gravity",
    "fall_die",
    "fall_save",
    "throw_range",
    "throw_reach_per_strength",
    "throw_reach_cap",
    "fragility",
    "break_die",
    "height_step",
    "surface",
];

pub fn physics() -> &'static Physics {
    static TABLES: OnceLock<Physics> = OnceLock::new();
    TABLES.get_or_init(|| read_physics(PHYSICS))
}

fn read_physics(source: &str) -> Physics {
    let file = load("physics.yml", source);
    let keys: Vec<String> = file
        .as_hash()
        .expect("physics.yml: a map")
        .keys()
        .map(|key| string(key, "physics.yml"))
        .collect();
    assert_eq!(keys, PHYSICS_KEYS, "physics.yml: its keys, in order");
    let table = |key: &str, empty: bool| -> Vec<(String, Option<i64>)> {
        let rows: Vec<(String, Option<i64>)> = file[key]
            .as_hash()
            .unwrap_or_else(|| panic!("physics.yml: {key} is not a map"))
            .iter()
            .map(|(row, value)| {
                let row = string(row, key);
                let value = match value {
                    Yaml::Integer(value) => Some(*value),
                    Yaml::Null if empty => None,
                    _ => panic!("physics.yml: {key}.{row} is not a whole number"),
                };
                (row, value)
            })
            .collect();
        assert!(!rows.is_empty(), "physics.yml: {key} is empty");
        rows
    };
    let whole = |key: &str| -> Vec<(String, i64)> {
        table(key, false)
            .into_iter()
            .map(|(row, value)| (row, value.expect("a whole number")))
            .collect()
    };
    let number = |key: &str| {
        file[key]
            .as_i64()
            .unwrap_or_else(|| panic!("physics.yml: {key} is not a whole number"))
    };
    let physics = Physics {
        bulk: table("bulk", true),
        thrown_damage: whole("thrown_damage"),
        gravity: whole("gravity"),
        fall_die: number("fall_die"),
        fall_save: string(&file["fall_save"], "fall_save"),
        throw_range: whole("throw_range"),
        throw_reach_per_strength: number("throw_reach_per_strength"),
        throw_reach_cap: number("throw_reach_cap"),
        fragility: table("fragility", true),
        break_die: number("break_die"),
        height_step: whole("height_step"),
        surface: whole("surface"),
    };
    // A thing that moves strikes for a die and flies a range; one that does
    // not has neither.
    let moving: Vec<&str> = physics
        .bulk
        .iter()
        .filter(|(_, penalty)| penalty.is_some())
        .map(|(bulk, _)| bulk.as_str())
        .collect();
    assert_eq!(
        rows(&physics.thrown_damage),
        moving,
        "physics.yml: thrown_damage"
    );
    assert_eq!(
        rows(&physics.throw_range),
        moving,
        "physics.yml: throw_range"
    );
    for die in [physics.fall_die, physics.break_die]
        .into_iter()
        .chain(physics.thrown_damage.iter().map(|(_, die)| *die))
    {
        assert!(die > 0, "physics.yml: a die of {die} sides");
    }
    physics
}

/// A table's rows, without their values.
fn rows(rows: &[(String, i64)]) -> Vec<&str> {
    rows.iter().map(|(row, _)| row.as_str()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physics_loads_with_the_throw_tables_the_engine_has_always_played() {
        let physics = physics();
        let bulk: Vec<(&str, Option<i64>)> = physics
            .bulk
            .iter()
            .map(|(bulk, penalty)| (bulk.as_str(), *penalty))
            .collect();
        assert_eq!(
            bulk,
            [
                ("light", Some(0)),
                ("handy", Some(2)),
                ("heavy", Some(5)),
                ("immovable", None)
            ]
        );
        let damage: Vec<(&str, i64)> = physics
            .thrown_damage
            .iter()
            .map(|(bulk, die)| (bulk.as_str(), *die))
            .collect();
        assert_eq!(damage, [("light", 4), ("handy", 6), ("heavy", 8)]);
        assert_eq!(physics.fall_save, "dexterity");
        assert_eq!(physics.fragility[0], ("sturdy".to_string(), None));
    }

    #[test]
    #[should_panic(expected = "its keys")]
    fn physics_refuses_a_key_it_does_not_declare() {
        read_physics(&format!("{PHYSICS}wind: 3\n"));
    }

    #[test]
    #[should_panic(expected = "thrown_damage")]
    fn physics_refuses_a_moving_bulk_with_no_damage_die() {
        read_physics(&PHYSICS.replace("  heavy: 8\n", ""));
    }
}
