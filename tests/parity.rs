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
