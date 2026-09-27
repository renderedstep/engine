//! Runs every case of every golden vector file in `vectors/` against this
//! crate, and fails on any case whose answer differs from the file's.
//!
//! Each portion checks its `constants` too, so a Ruby table that changed
//! under an unchanged case list still fails here.
//!
//! A portion this engine owns (`vectors/ENGINE_OWNED`) is blessed instead
//! when `BLESS` is set: every case's output and the constants are written
//! from this crate's answers, and a case or table that still agrees is left
//! byte for byte as it was, so the diff is exactly what the change moved.
//! `BLESS=1 cargo test --test vectors`, then review the diff. Every other
//! portion is exported by the game repository and never blessed here.

mod builders;
mod lines;
mod portions;

use serde_json::{json, Value};
use std::path::PathBuf;

/// The format versions this harness knows how to read.
const KNOWN_VERSIONS: &[u64] = &[1];

pub struct Vectors {
    pub constants: Value,
    pub cases: Vec<Value>,
}

/// Reads one portion's file and refuses one it cannot read faithfully.
pub fn load(portion: &str) -> Vectors {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vectors")
        .join(format!("{portion}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
    let document: Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("parsing {}: {error}", path.display()));
    check_header(&document, portion).unwrap_or_else(|problem| panic!("{portion}.json: {problem}"));
    Vectors {
        constants: document["constants"].clone(),
        cases: document["cases"].as_array().expect("cases").clone(),
    }
}

/// The header every file carries. An unknown version fails rather than being
/// read as if it were a known one.
pub fn check_header(document: &Value, portion: &str) -> Result<(), String> {
    if document["format"] != "engine-vectors" {
        return Err(format!(
            "not an engine-vectors file: {}",
            document["format"]
        ));
    }
    let version = document["version"]
        .as_u64()
        .ok_or_else(|| format!("no version: {}", document["version"]))?;
    if !KNOWN_VERSIONS.contains(&version) {
        return Err(format!(
            "format version {version} is not one this harness knows ({KNOWN_VERSIONS:?})"
        ));
    }
    if document["portion"] != portion {
        return Err(format!("portion is {}, not {portion}", document["portion"]));
    }
    if !document["cases"].is_array() {
        return Err("no cases".into());
    }
    Ok(())
}

/// The portions this engine owns: `vectors/ENGINE_OWNED`, one per line.
pub fn engine_owned() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vectors")
        .join("ENGINE_OWNED");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn blessing() -> bool {
    std::env::var_os("BLESS").is_some_and(|value| !value.is_empty())
}

/// A vector file as the game repository's `EngineVectors.render` writes
/// it: each header field on a line of its own, then one case per line.
pub fn render(document: &Value) -> String {
    let fields = document.as_object().expect("a vector document");
    let mut lines = vec!["{".to_string()];
    for (key, value) in fields.iter().filter(|(key, _)| *key != "cases") {
        lines.push(format!("  {}: {},", Value::from(key.as_str()), value));
    }
    lines.push("  \"cases\": [".to_string());
    let cases = fields["cases"].as_array().expect("cases");
    for (index, case) in cases.iter().enumerate() {
        let comma = if index + 1 == cases.len() { "" } else { "," };
        lines.push(format!("    {case}{comma}"));
    }
    lines.push("  ]".to_string());
    lines.push("}".to_string());
    format!("{}\n", lines.join("\n"))
}

/// Writes an engine-owned portion's file from this crate's answers.
fn bless(portion: &str, expected_constants: Value, answer: impl Fn(&Value) -> Value) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vectors")
        .join(format!("{portion}.json"));
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
    let mut document: Value = serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("parsing {}: {error}", path.display()));
    check_header(&document, portion).unwrap_or_else(|problem| panic!("{portion}.json: {problem}"));
    let cases = document["cases"].as_array().expect("cases").clone();
    let mut moved = 0;
    let blessed: Vec<Value> = cases
        .iter()
        .map(|case| {
            let got = answer(&with_shared_records(&case["input"], &cases));
            if serde_json::to_string(&got).ok() == serde_json::to_string(&case["output"]).ok() {
                return case.clone();
            }
            moved += 1;
            let mut case = case.clone();
            case["output"] = got;
            case
        })
        .collect();
    if document["constants"] != expected_constants {
        document["constants"] = expected_constants;
    }
    document["cases"] = Value::Array(blessed);
    std::fs::write(&path, render(&document))
        .unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
    println!(
        "{portion}: blessed, {moved} of {} case(s) moved",
        cases.len()
    );
}

/// Runs every case through `answer` and fails with the first few mismatches.
pub fn check(portion: &str, expected_constants: Value, answer: impl Fn(&Value) -> Value) {
    if blessing() && engine_owned().iter().any(|owned| owned == portion) {
        return bless(portion, expected_constants, answer);
    }
    let vectors = load(portion);
    assert_eq!(
        vectors.constants, expected_constants,
        "{portion}: the constants differ from this crate's tables"
    );
    let mut failures = Vec::new();
    for case in &vectors.cases {
        let input = with_shared_records(&case["input"], &vectors.cases);
        let got = answer(&input);
        // Compared as written, so key order counts as well as values.
        if serde_json::to_string(&got).ok() != serde_json::to_string(&case["output"]).ok() {
            failures.push(format!(
                "  {}\n    input:    {}\n    expected: {}\n    got:      {}",
                case["name"], case["input"], case["output"], got
            ));
        }
    }
    let total = vectors.cases.len();
    println!("{portion}: {}/{total} cases pass", total - failures.len());
    assert!(
        failures.is_empty(),
        "{portion}: {} of {total} cases differ; the first:\n{}",
        failures.len(),
        failures
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A case that names another in `records_of` shares its rows: the input is
/// handed on with that case's `records` in their place.
fn with_shared_records(input: &Value, cases: &[Value]) -> Value {
    let Some(name) = input.get("records_of") else {
        return input.clone();
    };
    let owner = cases
        .iter()
        .find(|case| case["name"] == *name)
        .unwrap_or_else(|| panic!("records_of names no case: {name}"));
    let mut input = input.clone();
    let fields = input.as_object_mut().expect("an input object");
    fields.remove("records_of");
    fields.insert("records".into(), owner["input"]["records"].clone());
    input
}

/// Runs every case of a portion that stands in rooms: `answer` gets the
/// file's `worlds` constant with each case's input. The portion's other
/// constants must equal `tables`, this crate's own.
pub fn check_rooms(portion: &str, tables: Value, answer: impl Fn(&Value, &Value) -> Value) {
    let vectors = load(portion);
    let worlds = vectors.constants["worlds"].clone();
    let mut expected = serde_json::Map::new();
    expected.insert("worlds".into(), worlds.clone());
    expected.extend(tables.as_object().expect("tables").clone());
    check(portion, Value::Object(expected), |input| {
        answer(&worlds, input)
    });
}

#[test]
fn an_unknown_format_version_is_refused() {
    let document =
        json!({ "format": "engine-vectors", "version": 2, "portion": "roll", "cases": [] });
    let problem = check_header(&document, "roll").unwrap_err();
    assert!(problem.contains("version 2"), "{problem}");
}

#[test]
fn a_file_that_is_not_vectors_is_refused() {
    let document =
        json!({ "format": "something-else", "version": 1, "portion": "roll", "cases": [] });
    assert!(check_header(&document, "roll").is_err());
}

/// Every portion this engine owns is a portion, and its file is exactly what
/// blessing renders, so a bless with no rule changed writes nothing new.
#[test]
fn every_engine_owned_portion_is_rendered_as_blessing_renders_it() {
    let owned = engine_owned();
    assert!(!owned.is_empty(), "vectors/ENGINE_OWNED names no portion");
    for portion in owned {
        assert!(
            portions::NAMES.contains(&portion.as_str()),
            "{portion} is not a portion"
        );
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vectors")
            .join(format!("{portion}.json"));
        let text = std::fs::read_to_string(&path).expect("the portion's file");
        let document: Value = serde_json::from_str(&text).expect("a vector document");
        assert_eq!(
            render(&document),
            text,
            "{portion}: blessing would render it differently"
        );
    }
}

#[test]
fn every_file_in_the_directory_has_a_portion() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vectors");
    let mut files: Vec<String> = std::fs::read_dir(directory)
        .expect("vectors directory")
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            name.strip_suffix(".json").map(str::to_string)
        })
        .collect();
    files.sort();
    let mut known: Vec<String> = portions::NAMES.iter().map(|n| n.to_string()).collect();
    known.sort();
    assert_eq!(
        files, known,
        "a vector file with no test, or a test with no file"
    );
}
