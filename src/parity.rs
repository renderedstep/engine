//! The engine sweep, played through this engine: the Ruby engine's stored
//! scripts of typed lines, each step answered with the same state dump the
//! Ruby engine writes (`EngineSweep::Dump`, documented in that repository's
//! `docs/engine-parity.md`), so the two can be compared step for step.
//!
//! A script plays its world's database as the Ruby engine loaded it, ids
//! pinned, and one playthrough per `player` the script names, each started
//! the first time a step is typed into it.

use crate::engine::{Engine, Error};
use crate::outcome::{Inscription, Named, Outcome, Room};
use crate::playthrough::Game;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use yaml_rust2::{Yaml, YamlLoader};

/// Appended to a world's title in the sweep's own copy of it
/// (`EngineSweep::Walk::TITLE_SUFFIX`).
pub const TITLE_SUFFIX: &str = " (engine sweep)";

/// Whose game a step with no `player` is typed into.
pub const DEFAULT_PLAYER: &str = "first";

/// The dump's keys, in the order a divergence is looked for.
pub const KEYS: [&str; 31] = [
    "location",
    "storey",
    "exits",
    "here",
    "carrying",
    "present",
    "foes",
    "inscription",
    "hp",
    "hp_of",
    "abilities",
    "dead",
    "changed",
    "change",
    "refused",
    "offers",
    "understood",
    "resolved_by",
    "note",
    "drifts",
    "blows",
    "hazards",
    "quest",
    "volitions",
    "acts",
    "ending",
    "ending_words",
    "scheduled",
    "fired",
    "elapsed_minutes",
    "shown",
];

/// One step of a script.
#[derive(Clone, Debug)]
pub struct Step {
    pub index: usize,
    pub id: Option<String>,
    pub typed: Option<String>,
    pub player: String,
    /// The step reloads the world file instead of typing a line.
    pub reseed: bool,
    /// A fixed character decision for a conversation.
    pub npc_action: Option<String>,
    /// The step plays a browser submission with fixed provider replies.
    pub browser: bool,
}

impl Step {
    /// How a report names the step (`EngineSweep::Script::Step#label`).
    pub fn label(&self) -> String {
        let mut label = format!("step {}", self.index);
        if let Some(id) = self.id.as_deref().filter(|id| !id.is_empty()) {
            label.push(' ');
            label.push_str(id);
        }
        if self.player != DEFAULT_PLAYER {
            label.push_str(&format!(" ({})", self.player));
        }
        label
    }
}

/// A stored walk: which world, and the steps typed into it.
#[derive(Clone, Debug)]
pub struct Script {
    pub name: String,
    pub story: String,
    pub steps: Vec<Step>,
}

fn scalar(value: &Yaml) -> Option<String> {
    match value {
        Yaml::String(text) | Yaml::Real(text) => Some(text.clone()),
        Yaml::Integer(n) => Some(n.to_string()),
        Yaml::Boolean(flag) => Some(flag.to_string()),
        _ => None,
    }
}

impl Script {
    /// Reads a script from its YAML text; `name` is its file name without
    /// the extension.
    pub fn parse(name: &str, source: &str) -> Result<Script, String> {
        let document = YamlLoader::load_from_str(source)
            .map_err(|error| format!("{name}: {error}"))?
            .into_iter()
            .next()
            .ok_or_else(|| format!("{name}: an empty file"))?;
        let story = scalar(&document["story"]).ok_or_else(|| format!("{name}: no story"))?;
        let rows = document["steps"]
            .as_vec()
            .ok_or_else(|| format!("{name}: \"steps\" is not a list"))?;
        let steps = rows
            .iter()
            .enumerate()
            .map(|(offset, row)| {
                let player = scalar(&row["player"])
                    .filter(|player| !player.trim().is_empty())
                    .unwrap_or_else(|| DEFAULT_PLAYER.to_string());
                Step {
                    index: offset + 1,
                    id: scalar(&row["id"]),
                    typed: scalar(&row["type"]),
                    player,
                    reseed: !row["reseed"].is_badvalue(),
                    npc_action: scalar(&row["npc_action"]),
                    browser: !row["browser"].is_badvalue(),
                }
            })
            .collect();
        Ok(Script {
            name: name.to_string(),
            story,
            steps,
        })
    }

    /// The world fixture's name: the story's title, slugged as the Ruby
    /// engine names its world files (`WorldSeed.slug`).
    pub fn world(&self) -> String {
        slug(&self.story)
    }
}

/// `WorldSeed.slug`: lower case, every run of other characters a hyphen.
pub fn slug(title: &str) -> String {
    let mut slug = String::new();
    let mut gap = false;
    for character in title.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            if gap && !slug.is_empty() {
                slug.push('-');
            }
            gap = false;
            slug.push(character);
        } else {
            gap = true;
        }
    }
    slug
}

/// Opens a world fixture (the SQL text of a database the Ruby engine
/// loaded) as a database of its own, in memory, so nothing played on it is
/// kept.
pub fn open_world(sql: &str) -> Result<Engine, Error> {
    let conn = rusqlite::Connection::open_in_memory()?;
    conn.execute_batch(sql)?;
    Engine::from_connection(conn)
}

/// A script that stopped before its last step: the dumps it wrote, and
/// why it stopped.
#[derive(Debug)]
pub struct Stopped {
    pub dumps: Vec<Value>,
    pub step: String,
    pub error: Error,
}

/// Plays every step of a script on an open world, and returns one dump per
/// step.
pub fn play(engine: &mut Engine, script: &Script) -> Result<Vec<Value>, Stopped> {
    let mut dumps = Vec::new();
    let mut games: HashMap<String, i64> = HashMap::new();
    for step in &script.steps {
        match play_step(engine, script, step, &mut games) {
            Ok(dump) => dumps.push(dump),
            Err(error) => {
                return Err(Stopped {
                    dumps,
                    step: step.label(),
                    error,
                })
            }
        }
    }
    Ok(dumps)
}

fn play_step(
    engine: &mut Engine,
    script: &Script,
    step: &Step,
    games: &mut HashMap<String, i64>,
) -> Result<Value, Error> {
    let story = engine.story_titled(&format!("{}{TITLE_SUFFIX}", script.story))?;
    let playthrough = match games.get(&step.player) {
        Some(game) => *game,
        None => {
            let game = engine.start(story)?;
            games.insert(step.player.clone(), game);
            game
        }
    };
    if step.reseed {
        return Err(Error::Unsupported("reloading the world file".into()));
    }
    if step.browser {
        return Err(Error::Unsupported("a browser submission".into()));
    }
    if step.npc_action.is_some() {
        return Err(Error::Unsupported("a fixed character decision".into()));
    }
    let before = Counts::of(engine, playthrough)?;
    let outcome = engine.play(
        playthrough,
        step.typed.as_deref().unwrap_or_default(),
        &mut |_| {},
    )?;
    let after = Counts::of(engine, playthrough)?;
    Ok(dump(&outcome, &before, &after))
}

/// The rows a step is counted by, before and after it.
struct Counts {
    drifts: i64,
    blows: i64,
    tolls: i64,
    volitions: i64,
    acts: i64,
    story_now: i64,
}

impl Counts {
    fn of(engine: &Engine, playthrough: i64) -> Result<Counts, Error> {
        let conn = engine.store().connection();
        let count = |sql: &str| -> Result<i64, Error> {
            Ok(conn.query_row(sql, [], |row| row.get::<_, i64>(0))?)
        };
        let records = engine.store().load()?;
        Ok(Counts {
            drifts: count("SELECT COUNT(*) FROM playthrough_drifts")?,
            blows: count("SELECT COUNT(*) FROM playthrough_blows")?,
            tolls: count("SELECT COUNT(*) FROM playthrough_tolls")?,
            volitions: count("SELECT COUNT(*) FROM playthrough_volitions")?,
            acts: count("SELECT COUNT(*) FROM playthrough_volitions WHERE status = 'applied'")?,
            story_now: Game::new(&records, playthrough).story_now(),
        })
    }
}

fn room(room: &Room) -> Value {
    json!({ "id": room.id, "name": room.name, "detail": room.detail })
}

fn named(rows: &[Named]) -> Value {
    rows.iter()
        .map(|row| json!({ "id": row.id, "name": row.name }))
        .collect()
}

fn inscriptions(rows: &[Inscription]) -> Value {
    rows.iter()
        .map(|row| json!({ "id": row.id, "name": row.name, "text": row.text }))
        .collect()
}

/// One step's dump (`EngineSweep::Dump#to_h`).
fn dump(outcome: &Outcome, before: &Counts, after: &Counts) -> Value {
    let state = &outcome.state;
    let report = &outcome.report;
    let mut dump = Map::new();
    let mut put = |key: &str, value: Value| {
        dump.insert(key.to_string(), value);
    };
    put(
        "location",
        state.location.as_ref().map_or(Value::Null, room),
    );
    put(
        "storey",
        state
            .location
            .as_ref()
            .and_then(|room| room.storey)
            .map_or(Value::Null, Value::from),
    );
    put("exits", state.exits.iter().map(room).collect());
    put("here", named(&state.here));
    put("carrying", named(&state.carrying));
    put("present", named(&state.present));
    put("foes", named(&state.foes));
    put("inscription", inscriptions(&state.inscription));
    put("hp", state.hp.map_or(Value::Null, Value::from));
    put(
        "hp_of",
        Value::Object(
            state
                .hp_of
                .iter()
                .map(|(name, hp)| (name.clone(), Value::from(*hp)))
                .collect(),
        ),
    );
    put(
        "abilities",
        state.abilities.as_ref().map_or(Value::Null, |abilities| {
            Value::Object(
                abilities
                    .iter()
                    .map(|(name, score)| (name.clone(), score.map_or(Value::Null, Value::from)))
                    .collect(),
            )
        }),
    );
    put("dead", Value::Bool(state.dead));
    put("changed", Value::Bool(report.change.is_some()));
    put("change", json!(report.change));
    put("refused", Value::Bool(report.refusal.is_some()));
    put("offers", json!(report.refusal));
    put("understood", json!(report.understood));
    put("resolved_by", json!(report.resolved_by));
    put(
        "note",
        if report.note.is_empty() {
            Value::Null
        } else {
            Value::from(report.note.join("\n"))
        },
    );
    put("drifts", Value::from(after.drifts - before.drifts));
    put("blows", Value::from(after.blows - before.blows));
    put("hazards", Value::from(after.tolls - before.tolls));
    put(
        "quest",
        Value::Object(
            state
                .quest
                .iter()
                .map(|(position, beat)| (position.to_string(), Value::from(beat.as_str())))
                .collect(),
        ),
    );
    put("volitions", Value::from(after.volitions - before.volitions));
    put("acts", Value::from(after.acts - before.acts));
    put("ending", Value::from(state.ending.as_str()));
    put("ending_words", json!(state.ending_words));
    put("scheduled", Value::from(state.scheduled));
    put("fired", Value::from(state.fired));
    put(
        "elapsed_minutes",
        json!((after.story_now - before.story_now) as f64 / 60.0),
    );
    put("shown", Value::Null);
    Value::Object(dump)
}

/// Equality as Ruby's `==` has it over parsed JSON: numbers by value
/// (`0 == 0.0`), objects whatever their key order.
pub fn same(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_i64(), y.as_i64()) {
            (Some(x), Some(y)) => x == y,
            _ => x.as_f64() == y.as_f64(),
        },
        (Value::Array(x), Value::Array(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(x, y)| same(x, y))
        }
        (Value::Object(x), Value::Object(y)) => {
            x.len() == y.len()
                && x.iter()
                    .all(|(key, value)| y.get(key).is_some_and(|other| same(value, other)))
        }
        _ => a == b,
    }
}

/// The first step at which two engines' dumps disagree, as
/// `EngineSweep::Parity.first_divergence` says it; none when they agree.
pub fn first_divergence(script: &Script, expected: &[Value], actual: &[Value]) -> Option<String> {
    for (index, step) in script.steps.iter().enumerate() {
        let typed = serde_json::to_string(&step.typed).unwrap_or_default();
        let (Some(want), Some(got)) = (expected.get(index), actual.get(index)) else {
            return Some(format!(
                "{}: {} typed {typed} -- the second engine has no dump",
                script.name,
                step.label()
            ));
        };
        let null = Value::Null;
        if let Some(key) = KEYS.iter().find(|key| {
            !same(
                want.get(**key).unwrap_or(&null),
                got.get(**key).unwrap_or(&null),
            )
        }) {
            return Some(format!(
                "{}: {} typed {typed}\n  {key}: {}\n  {}  {} (second engine)",
                script.name,
                step.label(),
                want.get(*key).unwrap_or(&null),
                " ".repeat(key.len()),
                got.get(*key).unwrap_or(&null)
            ));
        }
    }
    if actual.len() > expected.len() {
        return Some(format!(
            "{}: the second engine answered {} step(s) for {}",
            script.name,
            actual.len(),
            expected.len()
        ));
    }
    None
}

/// What checking a directory of scripts against their goldens found.
#[derive(Debug, Default)]
pub struct Checked {
    pub total: usize,
    /// The scripts that agree step for step.
    pub agreed: Vec<String>,
    /// Each other script's first divergence, or why it stopped.
    pub divergences: Vec<String>,
    /// A script named in `PASSING` that diverged, or one that agrees and is
    /// not named there.
    pub failures: Vec<String>,
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}

/// Reads a script file.
pub fn load_script(path: &Path) -> Result<Script, String> {
    let name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("{}: not a script path", path.display()))?;
    Script::parse(name, &read(path)?)
}

/// Opens a script's world from a directory of fixtures.
pub fn open_world_for(worlds: &Path, script: &Script) -> Result<Engine, String> {
    let path = worlds.join(format!("{}.sql", script.world()));
    open_world(&read(&path)?).map_err(|error| format!("{}: {error}", path.display()))
}

/// Plays every script in `dir/scripts` on the worlds in `dir/worlds` and
/// compares each with its golden file in `dir/goldens`. The scripts named
/// in `dir/PASSING` must agree, and every script that agrees must be named
/// there.
pub fn check(dir: &Path) -> Result<Checked, String> {
    let passing: Vec<String> = read(&dir.join("PASSING"))?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect();
    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir.join("scripts"))
        .map_err(|error| error.to_string())?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "yml"))
        .collect();
    paths.sort();

    let mut checked = Checked {
        total: paths.len(),
        ..Checked::default()
    };
    for path in &paths {
        let script = load_script(path)?;
        let golden: Value = serde_json::from_str(&read(
            &dir.join("goldens").join(format!("{}.json", script.name)),
        )?)
        .map_err(|error| format!("{}: {error}", script.name))?;
        let expected = golden_dumps(&golden);
        let mut engine = open_world_for(&dir.join("worlds"), &script)?;
        let divergence = match play(&mut engine, &script) {
            Ok(dumps) => first_divergence(&script, &expected, &dumps),
            Err(stopped) => {
                let played = stopped.dumps.len().min(expected.len());
                let so_far = Script {
                    steps: script.steps[..played].to_vec(),
                    ..script.clone()
                };
                Some(
                    first_divergence(&so_far, &expected[..played], &stopped.dumps).unwrap_or_else(
                        || {
                            format!(
                                "{}: {} stopped: {}",
                                script.name, stopped.step, stopped.error
                            )
                        },
                    ),
                )
            }
        };
        let listed = passing.contains(&script.name);
        match divergence {
            None => {
                if !listed {
                    checked.failures.push(format!(
                        "{} agrees step for step and is not in PASSING: add it",
                        script.name
                    ));
                }
                checked.agreed.push(script.name);
            }
            Some(divergence) => {
                if listed {
                    checked
                        .failures
                        .push(format!("{} is in PASSING and diverged", script.name));
                }
                checked.divergences.push(divergence);
            }
        }
    }
    for name in &passing {
        if !checked.agreed.contains(name)
            && !paths
                .iter()
                .any(|path| path.file_stem().is_some_and(|stem| stem == name.as_str()))
        {
            checked
                .failures
                .push(format!("{name} is in PASSING and there is no such script"));
        }
    }
    Ok(checked)
}

/// A golden file's dumps, in step order.
pub fn golden_dumps(golden: &Value) -> Vec<Value> {
    golden["steps"]
        .as_array()
        .map(|steps| steps.iter().map(|step| step["dump"].clone()).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_a_title_as_the_world_files_are_named() {
        assert_eq!(slug("The Unrecorded Hour"), "the-unrecorded-hour");
        assert_eq!(
            slug("A Clerk With Somewhere To Be"),
            "a-clerk-with-somewhere-to-be"
        );
    }

    #[test]
    fn compares_as_ruby_does() {
        assert!(same(&json!(0), &json!(0.0)));
        assert!(same(&json!({"a": 1, "b": 2}), &json!({"b": 2, "a": 1})));
        assert!(!same(&json!([1, 2]), &json!([2, 1])));
    }
}
