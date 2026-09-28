//! The engine's data files (`data/`, see its README): the words a model is
//! handed and the tables a rule reads. They are the one copy of those
//! strings, compiled in: nothing here retypes them, and the game reads the
//! same bytes through [`files`] rather than keeping a copy of its own.

use std::sync::OnceLock;
use yaml_rust2::{Yaml, YamlLoader};

const GRAMMAR: &str = include_str!("../data/playthrough/grammar.yml");
const REQUEST: &str = include_str!("../data/playthrough/classifier/request.yml");

/// Every data file, by its path under `data/` without `.yml`, and its
/// text exactly as compiled in: what a host that reads the same words
/// (the game's `EngineData`) reads.
pub fn files() -> [(&'static str, &'static str); 13] {
    [
        ("character/desires", DESIRES),
        ("item/inscriber", INSCRIBER),
        ("item/kits", KITS),
        ("location/generator", LOCATION_GENERATOR),
        ("location/kind", LOCATION_KIND),
        ("physics", PHYSICS),
        ("playthrough/classifier", CLASSIFIER),
        ("playthrough/classifier/request", REQUEST),
        ("playthrough/grammar", GRAMMAR),
        ("playthrough/volition/speech", SPEECH),
        ("playthrough/volition/weights", WEIGHTS),
        ("scene/ending", ENDING),
        ("scene/generator", SCENE_GENERATOR),
        ("scene/narrator", NARRATOR),
    ]
}

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
    narrator_doings()[intent].as_str()
}

/// `Scene::Narrator::DOING.keys`: the kinds of turn that have a sentence, in
/// file order.
pub fn narrator_doing_intents() -> Vec<&'static str> {
    narrator_doings()
        .as_hash()
        .map(|doing| doing.keys().filter_map(Yaml::as_str).collect())
        .unwrap_or_default()
}

fn narrator_doings() -> &'static Yaml {
    static FILE: OnceLock<Yaml> = OnceLock::new();
    &file_of(&FILE, "scene/narrator.yml", NARRATOR)["doing"]
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

const SPEECH: &str = include_str!("../data/playthrough/volition/speech.yml");

/// A pair of values, one for an ordinary turn and one for an arrival.
pub struct PerTurn {
    pub turn: i64,
    pub arrival: i64,
}

/// `data/playthrough/volition/speech.yml`: the speech die's shapes, the
/// weight of saying nothing, the cooldown, how many may speak at once, and
/// each pursuit's weight per shape, in file order.
pub struct Speech {
    pub shapes: Vec<String>,
    pub silent: PerTurn,
    pub cooldown_turns: i64,
    pub max_speakers: PerTurn,
    pub table: Vec<(String, Vec<(String, i64)>)>,
}

pub fn speech() -> &'static Speech {
    static SPEECH_TABLE: OnceLock<Speech> = OnceLock::new();
    SPEECH_TABLE.get_or_init(|| {
        let file = load("speech.yml", SPEECH);
        let whole = |value: &Yaml, what: &str| {
            value
                .as_i64()
                .unwrap_or_else(|| panic!("{what} is not a whole number"))
        };
        let per_turn = |value: &Yaml, what: &str| PerTurn {
            turn: whole(&value["turn"], what),
            arrival: whole(&value["arrival"], what),
        };
        let shapes: Vec<String> = file["shapes"]
            .as_vec()
            .expect("a list of shapes")
            .iter()
            .map(|shape| string(shape, "a shape"))
            .collect();
        let table = file["table"]
            .as_hash()
            .expect("a table of pursuits")
            .iter()
            .map(|(pursuit, row)| {
                let row: Vec<(String, i64)> = row
                    .as_hash()
                    .expect("a pursuit's weights")
                    .iter()
                    .map(|(shape, weight)| (string(shape, "a shape"), whole(weight, "a weight")))
                    .collect();
                assert!(
                    row.iter().map(|(shape, _)| shape).eq(shapes.iter()),
                    "a pursuit weighs every shape, in order"
                );
                (string(pursuit, "a pursuit"), row)
            })
            .collect();
        Speech {
            silent: per_turn(&file["silent"], "silent"),
            cooldown_turns: whole(&file["cooldown_turns"], "cooldown_turns"),
            max_speakers: per_turn(&file["max_speakers"], "max_speakers"),
            shapes,
            table,
        }
    })
}

const PHYSICS: &str = include_str!("../data/physics.yml");

/// `data/physics.yml`: the physics and item tables. Each table keeps its
/// file order; a `None` is a row the file leaves empty on purpose (a thing
/// that does not move, a thing that never breaks).
// The range tables have no reader yet.
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

const LOCATION_KIND: &str = include_str!("../data/location/kind.yml");

/// `data/location/kind.yml`: what sort of place a room may be, how
/// cluttered, and which rooms a building of each sort has.
pub struct LocationKind {
    pub kinds: Vec<String>,
    pub densities: Vec<String>,
    pub buildings: Vec<(String, Building)>,
}

/// One building's rooms: the one you walk in at, then the bands the rest are
/// dealt from, in the order ground, above, below.
pub struct Building {
    pub entry: String,
    pub bands: [Vec<String>; 3],
}

fn strings(value: &Yaml, what: &str) -> Vec<String> {
    value
        .as_vec()
        .unwrap_or_else(|| panic!("{what} is not a list"))
        .iter()
        .map(|word| string(word, what))
        .collect()
}

pub fn location_kind() -> &'static LocationKind {
    static KIND: OnceLock<LocationKind> = OnceLock::new();
    KIND.get_or_init(|| {
        let file = load("location/kind.yml", LOCATION_KIND);
        let buildings = file["buildings"]
            .as_hash()
            .expect("buildings is a map")
            .iter()
            .map(|(name, plan)| {
                let band = |key: &str| strings(&plan[key], key);
                (
                    string(name, "a building"),
                    Building {
                        entry: string(&plan["entry"], "entry"),
                        bands: [band("ground"), band("above"), band("below")],
                    },
                )
            })
            .collect();
        LocationKind {
            kinds: strings(&file["kinds"], "kinds"),
            densities: strings(&file["densities"], "densities"),
            buildings,
        }
    })
}

const KITS: &str = include_str!("../data/item/kits.yml");

/// `data/item/kits.yml`: what stands in a room of each sort, what lies on or
/// in it, and what small stuff lies about, each table in file order.
#[derive(Debug)]
pub struct Kits {
    pub kit_die: i64,
    pub visible: usize,
    pub density: Vec<(String, Vec<i64>)>,
    pub kinds: Vec<(String, Kit)>,
    pub pieces: Vec<(String, String)>,
    pub holding: Vec<(String, String)>,
    pub pools: Vec<(String, Pool)>,
    pub readable: Vec<String>,
    pub use_kinds: Vec<(String, String)>,
    pub combustible: Vec<String>,
}

/// One sort of place: each piece and its share of the die, and the small
/// stuff that may lie about.
#[derive(Debug)]
pub struct Kit {
    pub pieces: Vec<(String, i64)>,
    pub loose: Vec<String>,
}

/// What lies on or in a piece: how many, one of the list drawn, and the names
/// they are drawn from.
#[derive(Debug)]
pub struct Pool {
    pub count: Vec<i64>,
    pub from: Vec<String>,
}

/// The file's shape: every top-level key, and nothing else.
const KITS_KEYS: &[&str] = &[
    "kit_die",
    "visible",
    "density",
    "kinds",
    "pieces",
    "holding",
    "pools",
    "readable",
    "use_kinds",
    "combustible",
];

pub fn kits() -> &'static Kits {
    static TABLES: OnceLock<Kits> = OnceLock::new();
    TABLES.get_or_init(|| read_kits(KITS))
}

fn integers(value: &Yaml, what: &str) -> Vec<i64> {
    value
        .as_vec()
        .unwrap_or_else(|| panic!("{what} is not a list"))
        .iter()
        .map(|n| {
            n.as_i64()
                .unwrap_or_else(|| panic!("{what} holds a non-integer"))
        })
        .collect()
}

fn entries<'y>(value: &'y Yaml, what: &str) -> Vec<(String, &'y Yaml)> {
    value
        .as_hash()
        .unwrap_or_else(|| panic!("{what} is not a map"))
        .iter()
        .map(|(key, value)| (string(key, what), value))
        .collect()
}

fn read_kits(source: &str) -> Kits {
    let file = load("item/kits.yml", source);
    let keys: Vec<String> = entries(&file, "item/kits.yml")
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    assert_eq!(keys, KITS_KEYS, "item/kits.yml: its keys");
    let number = |key: &str| {
        file[key]
            .as_i64()
            .unwrap_or_else(|| panic!("item/kits.yml: {key} is not an integer"))
    };
    let kits = Kits {
        kit_die: number("kit_die"),
        visible: usize::try_from(number("visible")).expect("item/kits.yml: visible"),
        density: entries(&file["density"], "density")
            .into_iter()
            .map(|(word, band)| (word, integers(band, "a density")))
            .collect(),
        kinds: entries(&file["kinds"], "kinds")
            .into_iter()
            .map(|(kind, kit)| {
                let pieces = entries(&kit["pieces"], "a kind's pieces")
                    .into_iter()
                    .map(|(piece, share)| (piece, share.as_i64().expect("a share is an integer")))
                    .collect();
                (
                    kind,
                    Kit {
                        pieces,
                        loose: strings(&kit["loose"], "a kind's loose"),
                    },
                )
            })
            .collect(),
        pieces: pairs(&file["pieces"], "pieces"),
        holding: pairs(&file["holding"], "holding"),
        pools: entries(&file["pools"], "pools")
            .into_iter()
            .map(|(name, pool)| {
                (
                    name,
                    Pool {
                        count: integers(&pool["count"], "a pool's count"),
                        from: strings(&pool["from"], "a pool's names"),
                    },
                )
            })
            .collect(),
        readable: strings(&file["readable"], "readable"),
        use_kinds: pairs(&file["use_kinds"], "use_kinds"),
        combustible: strings(&file["combustible"], "combustible"),
    };
    check_kits(&kits);
    kits
}

/// `Item::Kit.checked!`: a table that cannot be right fails as it is read.
fn check_kits(kits: &Kits) {
    let words = [
        "nothing", "top", "hollow", "closed", "light", "handy", "heavy",
    ];
    let piece = |name: &str| kits.pieces.iter().find(|(p, _)| p == name).map(|(_, w)| w);
    for (name, word) in &kits.pieces {
        assert!(
            words.contains(&word.as_str()),
            "item/kits.yml: {name} is {word:?}"
        );
    }
    for (kind, kit) in &kits.kinds {
        assert!(
            location_kind().kinds.contains(kind),
            "item/kits.yml: {kind} is not a kind"
        );
        for (name, _) in &kit.pieces {
            assert!(
                piece(name).is_some(),
                "item/kits.yml: {kind} lists {name}, which is no piece"
            );
        }
    }
    for (name, pool) in &kits.holding {
        assert!(
            kits.pools.iter().any(|(p, _)| p == pool),
            "item/kits.yml: {name} holds from {pool}, which is no pool"
        );
        assert!(
            piece(name).is_some_and(|w| ["top", "hollow", "closed"].contains(&w.as_str())),
            "item/kits.yml: {name} holds things and is not fixed"
        );
    }
    for (name, word) in &kits.pieces {
        if ["top", "hollow"].contains(&word.as_str()) {
            assert!(
                kits.holding.iter().any(|(p, _)| p == name),
                "item/kits.yml: {name} is {word} and holds nothing"
            );
        }
    }
    let drawn = kits
        .pools
        .iter()
        .flat_map(|(_, pool)| &pool.from)
        .chain(kits.kinds.iter().flat_map(|(_, kit)| &kit.loose));
    for name in drawn {
        assert!(
            piece(name).is_none(),
            "item/kits.yml: {name} is both a piece and a thing"
        );
    }
    for (word, _) in &kits.density {
        assert!(
            location_kind().densities.contains(word),
            "item/kits.yml: density {word} is not a density"
        );
    }
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

    /// Every YAML file under `data/` is one `files` hands out, byte for
    /// byte, so a file added here cannot be missing from what the game reads.
    #[test]
    fn files_are_every_data_file_exactly() {
        fn walk(
            directory: &std::path::Path,
            root: &std::path::Path,
            found: &mut Vec<(String, String)>,
        ) {
            for entry in std::fs::read_dir(directory).expect("data/") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    walk(&path, root, found);
                } else if path.extension().is_some_and(|extension| extension == "yml") {
                    let name = path
                        .strip_prefix(root)
                        .expect("under data/")
                        .with_extension("")
                        .to_string_lossy()
                        .replace('\\', "/");
                    found.push((name, std::fs::read_to_string(&path).expect("a file")));
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
        let mut found = Vec::new();
        walk(&root, &root, &mut found);
        found.sort();
        let mut listed: Vec<(String, String)> = files()
            .iter()
            .map(|(name, text)| (name.to_string(), text.to_string()))
            .collect();
        listed.sort();
        assert_eq!(listed, found);
    }
}
