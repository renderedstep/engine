//! The engine sweep: every stored script of the Ruby engine, played through
//! this engine and compared, step for step, with the dumps the Ruby engine
//! wrote (`parity/`, see its README).

use renderedstep_engine::parity;
use std::path::Path;

#[test]
fn the_sweep_agrees_with_the_ruby_engine() {
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
