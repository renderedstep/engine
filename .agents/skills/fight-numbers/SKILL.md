---
name: fight-numbers
description: Choose or check any number that sets how hard the game is — monster level, guard or boss stats, healing and safe-room counts, a floor's minutes, a rest — by running the engine's fight simulator over the engine's own turn loop, never by estimate or by asking a model. Use when a world file, a floor schedule or a combat rule changes such a number.
---

# Fight numbers

A number that decides how hard a fight is gets measured, not guessed. The
fight simulator (`src/bin/fight.rs`, `src/sim/`) plays a matchup many times
through the engine's own turn loop and counts what happened. No model is
asked and no rule is copied, so a rule is in the simulator as soon as it is
in the engine. The README's "Measuring fights" section and the header of
`src/sim/mod.rs` own how it works; this skill is how to use it to settle a
number.

## 1. State the target first

Before any run, write down what the number is for, as a rate: for example
"a careful level-3 player beats this guard most of the time, and a naive one
often dies". Write it in the PR body or the change's notes. A target written
after the result is not a target.

## 2. Write the matchup

A matchup file is `sim/matchups/<name>.yml`, read by `sim::matchup` (its
header in `src/sim/matchup.rs` documents every key). Its keys are a world
file's, so a stat block can be pasted from one, and `rolled` takes a stat
from the engine's own roll for each seed. Start from an existing file:

- `sim/matchups/one-l3-d10.yml` — one foe, attack every line.
- `sim/matchups/careful-vs-naive.yml` — two rolled foes and a healing
  draught, played by the careful player; rerun it with `policy: naive`.
- `sim/matchups/two-l2-d10-flee.yml` — a flee threshold.
- `sim/matchups/levels-grid.yml` — a `sweep:` grid.

## 3. Run it

```sh
cargo run --release --bin fight -- sim/matchups/<file>.yml
cargo run --release --bin fight -- sim/matchups/<file>.yml --replay <seed>
cargo run --release --bin fight -- sim/matchups/<file>.yml --csv
```

Overrides: `--seeds A..B`, `--driver told|mechanics`, `--threads N`,
`--json`, `--csv`, `--fight-table FILE` (`-` for standard output). Anything
that hangs on the story's clock is measured on the `told` driver, the
default (README, "Drivers").

`--replay <seed>` prints one fight line by line. Replay a few seeds of any
result that surprises you before trusting the rates.

### Policies

`sim::policy` (`src/sim/policy.rs`) holds the habits: `attack`, `flee_below`,
`drink_below`, `throw_first`, and the two players the dungeon studies use:

- `naive` drinks when low and otherwise attacks.
- `careful` drinks earlier and leaves once one more round could kill it
  (`src/sim/policy.rs`, the `risky` check in `Policy::choose`).

The thresholds are in `Policy::named` in the same file; read them there rather
than here. Run a difficulty number against both: the careful rate says
whether the fight is fair to a player who plays well, the naive rate says
how punishing it is to one who does not.

### Sweeps and real worlds

- **Grid** (`sim::sweep`, header of `src/sim/sweep.rs`): a `sweep:` key in
  the matchup file plays every combination of its values, one row each. Any
  dotted path a matchup reads can be swept (`player.stats.level`,
  `foes.0.stats.level`, `story.lethality`, `policy.hp`). Sweep the candidate
  number across a range rather than trying one value at a time.
- **Seeded world** (`sim::world`, header of `src/sim/world.rs`): fight a real
  world's own hostiles against its protagonist, without writing the database:

  ```sh
  cargo run --release --bin fight -- --database FILE --story "TITLE" \
    --levels 3..6 sim/matchups/<file>.yml
  ```

  The matchup file beside it sets seeds, carrying and policy.

## 4. Read the result

`sim::tally` (`src/sim/tally.rs`) prints won, died and fled rates, each with
a Wilson 95% interval (`tally::wilson`), then rounds, hit points left,
damage, blows and items used. Judge the target against the interval, not
the point rate: if the interval straddles the target, add seeds
(`--seeds`) until it does not, or say that it does. Two candidate values
whose intervals overlap are not shown to differ.

## 5. Land the number with its command

- The chosen value goes in the world file or the code table that owns it.
  Beside it, in a comment, put the exact command that recomputes it and the
  target it was chosen for. Do not paste the measured rates into the
  comment; the command reproduces them.
- Attach the run to the change: the command, the target, and the output
  (or `--csv`) in the PR body.
- If the change moves fight odds in the engine itself, re-bless the goldens
  in the same PR, as the README's "Rules for changing it" says:
  `cargo run --release --bin fight -- --write sim`, and keep the diff of
  `sim/goldens/*.txt`. CI runs `fight --check sim`.

## What the simulator does not model

The simulator plays one fight in a one-room arena with a room to run to
(`sim::arena`). It does not model:

- **Route choice** — which rooms a player walks through, or which fights
  they take or avoid on a floor.
- **The countdown** — a floor's minutes running out across many fights.
- **Injection** — anything a player types that is not a fight line.

A number that depends on those (a floor's minutes, how many safe rooms or
healing draughts a floor needs, how long a rest should be) cannot be settled
by one matchup. Either build it from fight results with a sweep or the
`--fight-table` export a dungeon-level study reads (header of `src/sim/export.rs`), or, if no study covers
it, flag it to the owner as needing a new simulator feature. Do not fill the
gap with an estimate.

For what difficulty is meant to feel like, the server repository's
`VISION.md` is the reference.
