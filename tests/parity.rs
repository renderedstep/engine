//! The engine sweep: every stored script of the game, played through this
//! engine and compared, step for step, with its golden dumps (`parity/`, see
//! its README), and the writer that keeps those goldens.

use renderedstep_engine::parity;
use std::path::Path;

#[test]
fn the_sweep_agrees_with_its_goldens() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity");
    let checked = parity::check(&dir).expect("the parity directory reads");
    for divergence in &checked.divergences {
        println!("{divergence}\n");
    }
    println!(
        "sweep: {} of {} script(s) agree step for step",
        checked.agreed.len(),
        checked.total
    );
    assert!(
        checked.failures.is_empty(),
        "{}",
        checked.failures.join("\n")
    );
}

/// The shared-database mode: each passing script's world in a database file,
/// opened afresh for every typed step as the Ruby engine's runner does,
/// agrees with the goldens exactly as the whole-script mode does.
#[test]
fn one_step_at_a_time_on_a_kept_database_agrees_too() {
    use renderedstep_engine::engine::Engine;
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity");
    let passing = std::fs::read_to_string(dir.join("PASSING")).unwrap();
    for name in passing
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let script = parity::load_script(&dir.join("scripts").join(format!("{name}.yml"))).unwrap();
        let world =
            std::fs::read_to_string(dir.join("worlds").join(format!("{}.sql", script.world())))
                .unwrap();
        let file = std::env::temp_dir().join(format!(
            "renderedstep-engine-step-{name}-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&file);
        rusqlite::Connection::open(&file)
            .and_then(|conn| conn.execute_batch(&world))
            .unwrap();
        let dumps: Vec<serde_json::Value> = script
            .steps
            .iter()
            .map(|step| {
                let mut engine = Engine::open(&file).unwrap();
                parity::play_one(&mut engine, &script, step.index, &step.player)
                    .unwrap_or_else(|error| panic!("{name}: {}: {error}", step.label()))
            })
            .collect();
        let _ = std::fs::remove_file(&file);
        let golden: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(dir.join("goldens").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        let divergence = parity::first_divergence(&script, &parity::golden_dumps(&golden), &dumps);
        assert!(divergence.is_none(), "{}", divergence.unwrap_or_default());
    }
}

/// Every golden is exactly what `parity --write` renders for its dumps, so
/// a rewrite moves only the steps whose dumps moved.
#[test]
fn every_golden_is_rendered_as_the_writer_renders_it() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity");
    let mut goldens: Vec<_> = std::fs::read_dir(dir.join("goldens"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    goldens.sort();
    assert!(!goldens.is_empty());
    for path in goldens {
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();
        let script = parity::load_script(&dir.join("scripts").join(format!("{name}.yml"))).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let golden: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(
            parity::render_golden(&script, &parity::golden_dumps(&golden)),
            text,
            "{name}: the writer would render this golden differently"
        );
    }
}

/// Writing the goldens with no rule changed writes nothing: every file is
/// left byte for byte as it was, the records files included.
#[test]
fn writing_the_goldens_again_changes_nothing() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("parity");
    let copy =
        std::env::temp_dir().join(format!("renderedstep-parity-write-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&copy);
    for sub in ["scripts", "goldens", "records", "worlds"] {
        std::fs::create_dir_all(copy.join(sub)).unwrap();
        for entry in std::fs::read_dir(source.join(sub)).unwrap() {
            let path = entry.unwrap().path();
            std::fs::copy(&path, copy.join(sub).join(path.file_name().unwrap())).unwrap();
        }
    }
    for file in ["PASSING", "PASSING_WITH_RUNNER"] {
        std::fs::copy(source.join(file), copy.join(file)).unwrap();
    }
    let written = parity::write(&copy).expect("the parity directory writes");
    for sub in ["goldens", "records"] {
        for entry in std::fs::read_dir(source.join(sub)).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap();
            assert_eq!(
                std::fs::read(&path).unwrap(),
                std::fs::read(copy.join(sub).join(name)).unwrap(),
                "{sub}/{} was written again with nothing changed",
                name.to_string_lossy()
            );
        }
    }
    let _ = std::fs::remove_dir_all(&copy);
    assert!(written.goldens.is_empty(), "{:?}", written.goldens);
    assert!(written.records.is_empty(), "{:?}", written.records);
    assert!(written.stopped.is_empty(), "{:?}", written.stopped);
    let mut runner = parity::runner_only(&source).unwrap();
    runner.sort();
    let mut left = written.runner.clone();
    left.sort();
    assert_eq!(left, runner);
}
