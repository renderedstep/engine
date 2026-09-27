# renderedstep-engine

The text-adventure engine in Rust: the dice, the geometry, reading a typed
line against the room the player stands in, building the requests the engine
hands a model, and a turn loop that plays a line with no model at all over the
game's own SQLite database. The rules take values and return values; only the
turn loop (`store`, `turn`, `outcome`, `engine`) touches a database, and
nothing touches the network or an async runtime. Its dependencies are
`serde_json`, for the System One request and answers and the rows a rule
reads, `yaml-rust2`, for the engine data it compiles in (`data/`, see its
README), `sha2`, for request digests, and `rusqlite` with SQLite bundled, for
the database.

It reproduces the Ruby engine exactly, roll for roll, and is checked against
the golden vectors that engine exports (`vectors/`, see its README) and
against the engine sweep, the Ruby engine's stored scripts of typed lines
with the state it wrote after every step (`parity/`, see its README).

| Module | Ruby original | What it answers |
| --- | --- | --- |
| `random` | Ruby's `Random` | MT19937 seeded as `Random.new` seeds it; bounded draws and `Array#shuffle` |
| `roll` | `Roll` | the five-part seed, a die, a pool, one of a list, a weighted pick |
| `stat_block` | `Character::StatBlock` | a body's level, hit die and abilities |
| `spot` | `Location::Spot`, `Location::Placement` | where in a room a thing or a person stands |
| `population` | `Location::Population`, `WorldSeed.natural_key` | how many people a room holds, seeded from a CRC32 of its name |
| `text` | Ruby string behaviour | `downcase` without the final-sigma rule, ASCII `strip` and `\s`, `inspect`, CRC32 |
| `danger` | `Location::Danger` (the pure half) | a new room's danger, monstrous throws, a building's danger and hazards |
| `parameters` | `Location::Parameters` | a building's picks resolved against its tables, kept in key order |
| `boxes` | `Location::Box` | walls, shared ground, bearings and distances between rooms |
| `interior` | `Location::Interior` | a building's rooms and doorways, returned rather than written |
| `shuffle_connections` | `WorldMechanic::ShuffleConnections` | how a shuffle rearranges the mobile rooms' doorways |
| `world_mechanic` | `WorldMechanic` | the cadence boundaries a mechanic has to run for |
| `deadline` | `Quest::Deadline` | the hops walk and the anchor room an overdue step is placed from |
| `cast` | `Character::Registry`, `Character::Generator` | the seeded race, age, sex and background of new people |
| `room` | `Playthrough::Classifier`'s closed sets, `Playthrough::PhysicalAction#choices` | the ways out, the cast, the floor, the hands and the physical attempts of one room |
| `grammar` | `Playthrough::Grammar` | a typed line read without a model: a slashed line claimed, names resolved, refusals written |
| `intent` | `Playthrough::Classifier::Intent`, `Playthrough::Classifier#build_intent` | what a line was read as, whether it is refused, and a model's answer resolved to records |
| `refusal` | `Playthrough::Refusal` (with `DeathNotice` and `StoryOverNotice` sentences) | what the engine says when it will not play a line |
| `slash_menu` | `Playthrough::SlashMenu` | the words offered after a slash and what each completes to |
| `cascade` | `Playthrough::Classifier::Cascade`, `::State`, `::Request`, `SystemOneAgent::Answers` | the System One request for a line, and recorded answers composed into an intent or escalated. It sends nothing |
| `records` | the rows a builder reads | every row of every table, handed in as values; a query is a filter over a list |
| `playthrough` | `Playthrough`'s readers, `Playthrough::Vitals::Condition`, `Playthrough::Toll#to_s` | who is in a room, how much is left of a body, who is fighting the party |
| `ledger` | `Playthrough::Ledger` | what one person saw happen in one game, inside both of its bounds |
| `memory` | `Playthrough::Memory` | which earlier exchanges come back into a prompt, ranked with Ruby's float arithmetic |
| `plan` | `Location::Plan` | a room's size, storey and ways out, said in sentences |
| `moment` | `Playthrough::Moment` | the narration context and a character's context for the moment the player stands in |
| `volition` | `Playthrough::Volition#choices`, `::State`, `::SystemOne#request` | the acts a person may take and the System One request that asks which |
| `schemas` | the `RubyLLM::Schema` classes a request carries | each schema's `to_json_schema` output, key for key |
| `identity` | `Eval::RequestIdentity` | the canonical form of a set of requests and its 16-hex digest |
| `arrival` | `Scene::Generator`, `Scene::ArrivalContext` | the arrival writer's request for walking into a place |
| `realization` | `Location::Generator`, `Character::Desires.instructions` | the room writer's detail and exits requests |
| `dialogue` | `InteractionAgent`, `Character#interaction_instructions`, `Playthrough::NpcAction` | the character pass and narrator pass of one exchange |
| `clock` | Rails' datetime columns | a stored time as whole seconds since the epoch, and back |
| `store` | the schema `db/schema.rb` describes | the database on a connection of its own: the schema version checked, every table the loop reads loaded as records, a row inserted or updated |
| `turn` | `Playthrough::Mechanics` with `model: false`, and the `Playthrough::Turn` writers it calls | one typed line read, refused or played, and the world's answer: foes, volition's die, hazards, the arc and the fight |
| `outcome` | `Playthrough::Mechanics::State` | what a turn left behind, read off the records |
| `engine` | `Playthrough::Session`'s place at the switch | a line in, the outcome out, one transaction per line, every failure a value |
| `parity` | `EngineSweep::Walk`, `EngineSweep::Dump`, `EngineSweep::Parity` | a sweep script played through this engine, dumped step by step and compared |

## Playing a turn

```rust
use renderedstep_engine::engine::Engine;

let mut engine = Engine::open(Path::new("storage/development.sqlite3"))?;
let outcome = engine.play(playthrough_id, "take the ward stamp", &mut |chunk| {
    print!("{chunk}");
})?;
println!("{:?} {:?}", outcome.report.change, outcome.state.carrying);
```

`Engine` is the whole surface a host calls, and it is shaped for the switch,
where the Rails app hands each whole turn to this engine in-process:

- **Its own connection.** `Engine::open` opens the database file on a new
  SQLite connection and refuses it with `Error::SchemaMismatch` unless its
  newest migration is `store::SCHEMA_VERSION`. The app keeps
  `bin/rails generate migration`; a new migration needs this constant, and
  whatever it changes, ported before the engine will open that database.
- **One transaction per line.** `play` takes SQLite's write lock
  (`BEGIN IMMEDIATE`), plays the line, and commits. The caller must hold no
  transaction on the same database while it runs. Anything that fails rolls
  the whole line back.
- **A line and a chunk callback in, the structured outcome out.** The
  `Outcome` is the report (`understood`, `change`, `refusal`, `note`,
  `resolved_by`) and the records the line left (`outcome::State`: the room,
  its exits, what lies here and is carried, who is present and fighting, hit
  points, the arc, the ending, what the clock owes). `on_chunk` receives
  narrated prose as it is written; this engine plays with no model, so it
  narrates nothing and never calls it.
- **Errors as values, never a panic across the boundary.** Every public call
  returns `Result<_, engine::Error>`. A panic inside a call is caught there
  (`catch_unwind`), the line's writes are rolled back, and it comes back as
  `Error::Panicked`. A line that reaches a rule this engine does not play yet
  comes back as `Error::Unsupported`, also rolled back.

`Engine::start` begins a playthrough the way the browser does, and
`Engine::read` returns the records with nothing played.

A Ruby binding (magnus) is the next consumer and is not built yet. It is a
thin layer over this surface: open an `Engine` on the app's database path,
call `play` with a block that receives each chunk, turn the `Outcome` into a
Hash, and raise one Ruby exception class per `Error` variant.

## Rules for changing it

- Draw only from `random::Random`. Any other generator rolls different dice.
- Ruby's integer division floors: use `div_euclid`, never `/`, on anything
  that can be negative.
- Tables that Ruby keeps in a `Hash` are arrays of pairs here, so their order
  survives.

## Testing

```sh
cargo test -- --nocapture
```

`tests/vectors/` runs every case of every vector file and prints a pass count
per portion. An answer is compared as written, so key order counts as well as
values. It also checks each file's `constants` against this crate's
tables, and refuses a file whose format version it does not know.

`tests/parity.rs` plays every sweep script in `parity/scripts` and compares
each step's dump with the Ruby engine's golden file, printing each other
script's first divergence and a pass count. The binary does the same:

```sh
cargo run --release --bin parity -- --check parity
```

and plays one script for the Ruby engine's own runner, which diffs it against
the goldens there:

```sh
ENGINE="target/release/parity --worlds $PWD/parity/worlds" bin/rails engine:parity_diff
```

## Licence

MIT OR Apache-2.0, at your option, see `LICENSE-MIT` and `LICENSE-APACHE`.
