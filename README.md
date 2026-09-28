# renderedstep-engine

The text-adventure engine in Rust: the dice, the geometry, reading a typed
line against the room the player stands in, building the requests the engine
hands a model, the model client that sends them, and a turn loop that plays a
line with no model at all over the game's own SQLite database. The rules take
values and return values; only the turn loop (`store`, `turn`, `outcome`,
`engine`) touches a database, only the model client (`model`) touches the
network, and nothing needs an async runtime. Its dependencies are
`serde_json`, for the System One request and answers and the rows a rule
reads, `yaml-rust2`, for the engine data it compiles in (`data/`, see its
README), `sha2`, for request digests, `rusqlite` with SQLite bundled, for the
database, `regex`, for the refusal detector, and `ureq` with rustls, for
HTTPS.

It was written to reproduce the game's Ruby engine exactly, roll for roll,
and now owns the game's behaviour: it is checked against the golden vectors
(`vectors/`, see its README), most of which the game still exports from the
rules its Ruby code runs, and against the engine sweep, the game's stored
scripts of typed lines with the state after every step (`parity/`, see its
README). The goldens of the sweep are this repository's own: a change to a
rule rewrites them here, and the game vendors them at the commit it pins.

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
| `kind` | `Location::Kind` | what sort of place a room may be and how cluttered, and the sort each room of a building is dealt |
| `kit` | `Item::Kit` | what stands in a room of a kind and what lies about in it, rolled on the room's name from `data/item/kits.yml` |
| `shuffle_connections` | `WorldMechanic::ShuffleConnections` | how a shuffle rearranges the mobile rooms' doorways |
| `world_mechanic` | `WorldMechanic` | the cadence boundaries a mechanic has to run for |
| `deadline` | `Quest::Deadline` | the hops walk and the anchor room an overdue step is placed from |
| `cast` | `Character::Registry`, `Character::Generator` | the seeded race, age, sex and background of new people |
| `physics` | none: it was written here | a fall through a doorway: the storeys it drops, the dice the world's gravity throws for them, the save that halves them, and what the toll says it was; and whether a thing that came down on a floor broke: its fragility, how it came down and the floor's surface, as a share of one die |
| `room` | `Playthrough::Classifier`'s closed sets, `Playthrough::PhysicalAction#choices` | the ways out, the cast, the floor, the hands and the physical attempts of one room |
| `grammar` | `Playthrough::Grammar` | a typed line read without a model: a slashed line claimed, names resolved, refusals written |
| `intent` | `Playthrough::Classifier::Intent`, `Playthrough::Classifier#build_intent` | what a line was read as, whether it is refused, and a model's answer resolved to records |
| `refusal` | `Playthrough::Refusal` (with `DeathNotice` and `StoryOverNotice` sentences) | what the engine says when it will not play a line |
| `slash_menu` | `Playthrough::SlashMenu` | the words offered after a slash and what each completes to |
| `cascade` | `Playthrough::Classifier::Cascade`, `::State`, `::Request`, `SystemOneAgent::Answers` | the System One request for a line, and recorded answers composed into an intent or escalated. It sends nothing |
| `classifier` | `Playthrough::Classifier#classify` | a line read over a room's records: System One first where it is on, then the classifier model's call, through whatever answers the two (`classifier::Reader`) |
| `records` | the rows a builder reads | every row of every table, handed in as values; a query is a filter over a list |
| `playthrough` | `Playthrough`'s readers, `Playthrough::Vitals::Condition`, `Playthrough::Toll#to_s` | who is in a room, how much is left of a body, who is fighting the party |
| `ledger` | `Playthrough::Ledger` | what one person saw happen in one game, inside both of its bounds |
| `memory` | `Playthrough::Memory` | which earlier exchanges come back into a prompt, ranked with Ruby's float arithmetic |
| `plan` | `Location::Plan` | a room's size, storey and ways out, said in sentences |
| `moment` | `Playthrough::Moment` | the narration context and a character's context for the moment the player stands in |
| `narration` | `Scene::Narrator#prompt_for` | the narrator's request for a typed line, what the engine already did and the moment it happens in |
| `volition` | `Playthrough::Volition#choices`, `::State`, `::SystemOne#request` | the acts a person may take and the System One request that asks which; what they may say unasked, and the speech die thrown over it |
| `schemas` | the `RubyLLM::Schema` classes a request carries | each schema's `to_json_schema` output, key for key |
| `identity` | `Eval::RequestIdentity` | the canonical form of a set of requests and its 16-hex digest |
| `arrival` | `Scene::Generator`, `Scene::ArrivalContext` | the arrival writer's request for walking into a place |
| `realization` | `Location::Generator`, `Character::Desires.instructions` | the room writer's detail and exits requests |
| `dialogue` | `InteractionAgent`, `Character#interaction_instructions`, `Playthrough::NpcAction` | the character pass and narrator pass of one exchange |
| `clock` | Rails' datetime columns | a stored time as whole seconds since the epoch, and back |
| `store` | the schema `db/schema.rb` describes | the database on a connection of its own: the schema version and the shape of every table it touches checked, every table the loop reads loaded as records, a row inserted or updated |
| `turn` | `Playthrough::Mechanics` with `model: false`, the `Playthrough::Turn` writers it calls, `PhysicalAction`, `NpcAction`, `Riposte`, `Volition`, `Hazards`, `Arc`, `Fight` | one typed line read, refused or played, and the world's answer: who speaks up unasked, foes, volition's die, hazards, the arc and its ending, and the scene that closes a fight |
| `outcome` | `Playthrough::Mechanics::State` | what a turn left behind, read off the records |
| `glance` | `Playthrough::Glance`, `Playthrough::Availability`, `Playthrough::SlashMenu` | what a front end's panels show between turns: the room, its ways out, who and what is here, which verbs are open at what (each target one the turn plays, each closed verb closed in the refusal's words), the slash menu and the next beat |
| `facts`, `prompt_version` | `Scene::Narrator#prompt_for`, `Playthrough::Turn`'s `_fact` builders, `Playthrough::PromptVersion::Scaffold` | what a turn hands the narrator as already done, and that scaffold rendered against fixed placeholders for the game's prompt version to digest |
| `command` | `Playthrough::Command`, `Playthrough::Command::Journal` | a submitted line and its token, the order lines were accepted in, and the receipts a turn writes as it goes |
| `turn::Turn` | `Playthrough::Turn#play` with a request token, `Playthrough::Classifier#classify`, `InteractionAgent`, `Scene::Narrator`, `Scene::Generator`, `Location::Generator#realize!` with `Item::Kit#furnish!`, `Item::Registry`, `Character::Registry`, `Location::RoomName` and `Location::Interior.lay_out!`, `Quest::Binder`, `Quest::Deadline`, `Scene::Ending`, `Item::Inscriber` | a submitted line read, refused or played through the models, told in prose or in the engine's own words, and answered by the world |
| `engine` | `Playthrough::Session`'s place at the switch | a line in, the outcome out, one transaction per line, every failure a value |
| `parity` | `EngineSweep::Walk`, `EngineSweep::Dump`, `EngineSweep::Parity` | a sweep script played through this engine, dumped step by step and compared |
| `model` | `BaseAgent`, `BaseAgent::Refusal`, `SystemOneAgent`, RubyLLM's OpenRouter provider and its `chats`/`messages` receipts, `EngineSweep::BrowserTurn`'s fixed replies | where a model call goes, the body it sends, whether an answer is kept, the model rotation, and what a call leaves in the database |

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
  SQLite connection and checks its schema. A database whose newest
  migration is older than `store::SCHEMA_VERSION` is refused with
  `Error::SchemaMismatch`. One at a newer migration is opened only when
  every table the engine reads or writes still has the columns, indexes,
  foreign keys and triggers `store::SHAPE` records, and nothing new points
  into one of them; otherwise it is refused with `Error::SchemaChanged`,
  which lists each difference. A migration that only adds a table the engine
  never touches is therefore accepted; one that changes a table it does
  needs the change ported before the engine will open that database.
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

`Engine::start` begins a playthrough the way the browser does,
`Engine::read` returns the records with nothing played, `Engine::glance`
returns what a front end's panels show between turns and writes nothing, and
`Engine::play_deciding` plays a line that talks to somebody with a fixed
decision standing in for the answer a model would give, which is how the
engine sweep plays its conversations.

`Engine::submit_fixed` plays a submitted line with a fixed reading
(`turn::Fixed`: an action and the name of its target) wherever the
classifier would have been asked, and asks its models for everything else:
a bench that measures what a turn does after its reading plays its cases
this way. The request builders take rows rather than a database, so a
caller holding a staged position's rows builds any request the turn
would send, `turn::room_of` reads the room the grammar and the classifier
read, and `classifier::read` reads a line as a turn does.

`Engine::submit` plays a line the way every front end does
(`Playthrough::Session#play` with a request token), asking a `model::Models`
for its prose: `model::Live` over the player's route, or `model::Replay`.
The line is accepted into the game's submission queue first
(`playthrough_commands`); lines accepted earlier and still owed a finish
play before it, and a second delivery of the same token and line hands back
what the first produced and plays nothing. There is no transaction around a
submitted line: each effect is committed with its journal receipt as the
turn reaches it, and no transaction is open while a model is asked, so a
failure keeps what was committed before it, as the Ruby engine does. A call
for prose that fails after an effect was written is answered with the
engine's own words, and the turn still finishes. `Engine::accept` queues a
line without playing it.

The journal is also how an interrupted turn is finished. Each step's
receipt holds the value the step answered, written as the Ruby engine
writes it (`Playthrough::Command::Journal#encode`), and a step the journal
already holds answers that value again without running: a take committed
before a worker died is not taken twice, a crossing already paid for is not
paid again, and a narration already written is not asked for again. A
submission left running is finished by the next delivery of any later
line, before that line plays. `Engine::submit_stopping` stops a turn right
after a named step commits, as a killed worker stops, and comes back as
`Error::Stopped` with the submission still running; the engine sweep plays
its interrupted workers this way.

Reading a readable thing that has no words yet writes them first
(`Item::Inscriber`): one call, kept on the game's copy and on the world's
own before the narrator is told them, so every later reading, in any game,
reads the same words and asks for nothing. Offering a thing to somebody is
a conversation with one more choice in it: the person may accept what the
player holds out, and only the engine moves it.

A line with no slash, and a slashed line the grammar cannot place, is read
by System One first where it is on (`cascade`), and by the classifier model
when System One is off, escalates or is unavailable
(`Playthrough::Classifier#classify`); a reach that found nothing and a line
that named two things are counted as the Ruby engine counts them. Talking
to somebody is `InteractionAgent`'s two calls: the person answers in their
own conversation with this game, picked up again on the next talk, and
picks one action the engine offered; the engine applies it, and the
narrator writes the exchange from the reaction and the engine's receipt.

Walking into a room nobody has written writes it first
(`Location::Generator#realize!`): one call for its description, lore,
things and people, whom the engine has already drawn, and one for its ways
out, with the first answer kept on the room as a checkpoint so a room whose
second call failed is picked up there, never paid for twice. What the
answers propose is admitted, not obeyed (`Item::Registry`,
`Character::Registry`, the exit rules), and a building is laid out into
rooms as it is written and walked into at its entry. A place, a person or a
thing a room's writing admits is taken by any step of the story's arc that
was waiting for its name (`Quest::Binder`). Once the story has written more
rooms than its grace, a room finishing its writing places the arc's first
waiting step itself (`Quest::Deadline`): a place laid out two levels down
off the deepest room the party can reach, a person in the deepest room of
a place they can reach, or a thing on that room's floor.

On the line that concludes the story's arc, the narrator writes the last
paragraph over the closing scene the arc already wrote (`Scene::Ending`).
A failed call, and a paragraph that stops mid-sentence, leave the engine's
own sentence standing on that scene.

A Ruby binding (magnus) is the next consumer and is not built yet. It is a
thin layer over this surface: open an `Engine` on the app's database path,
call `play` with a block that receives each chunk, turn the `Outcome` into a
Hash, and raise one Ruby exception class per `Error` variant.

## Model calls

Every call goes to OpenRouter by one of two routes, and the engine takes the
route as one value, `model::Route`:

- `Direct { key }`: the player's own OpenRouter key, sent to
  `https://openrouter.ai` and nowhere else;
- `Relay { base_url, token }`: the owner's relay, which mirrors OpenRouter's
  paths under its base and holds his key under a monthly cap per player, so
  the request body is the same on both routes and only the base and the
  bearer differ;
- `None`: no model access, and the offline path only.

`Route::pick` applies the precedence (the player's own key, then a relay
invitation, unless the player asked for one of them) and never falls back
from one to the other. A key or token is a `model::Secret`, which has no
`Display` and a redacted `Debug`; a failure quotes the provider's message,
never the request.

`model::Live` sends a call under `BaseAgent`'s policy: the models in
`REMOTE_MODEL_IDS` order, up to `MAX_ATTEMPTS`; a refusal, an ignored
schema, a provider failure or the caller's own check asks the next model in
the same context; a crisis answer is suppressed and a refused credential
fails, neither rotated past. It writes the conversation to `chats` and
`messages`, and a `ruby_llm_usages` row per attempt, as RubyLLM does, so the
Ruby app reads the receipts of a call this engine made. System One goes over
the same route to OpenRouter's decisions path, or to TypeSafe directly with
a TypeSafe key.

`model::Replay` answers each call with the next of a list of fixed replies
tagged by purpose, and checks what each prompt includes and leaves out: the
engine sweep's browser steps play this way, and nothing is sent anywhere.

`tests/model.rs` holds the client to what RubyLLM sent for the same calls
(`tests/fixtures/rubyllm_wire.json`, captured by a server on the machine
that answered with fixed replies), the refusal detector to the Ruby
engine's flags over its corpus (`tests/fixtures/refusal_corpus.json`), a
stream to what it shows of that corpus before the answer is judged, and
both routes to the no-leak contract over a real socket on this machine. The
kept requests of `vectors/kept_requests.json` are sent through the live
client too (`tests/vectors/builders.rs`), and go out exactly as stored.

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
tables, and refuses a file whose format version it does not know. The
portions this crate owns (`vectors/ENGINE_OWNED`) are blessed rather than
checked when `BLESS` is set:

```sh
BLESS=1 cargo test --test vectors   # rewrite the engine-owned portions from this crate
```

`tests/parity.rs` plays every sweep script in `parity/scripts` and compares
each step's dump with its golden file, and, for a script with a file in
`parity/records`, every request it made and every row it wrote with that
file's, printing each other script's first divergence and a pass count. The
binary does the same, and writes the goldens again once a rule has moved
them:

```sh
cargo run --release --bin parity -- --check parity
cargo run --release --bin parity -- --write parity   # then review the diff
```

and plays one script for the game repository's runner, which diffs it
against a directory of goldens:

```sh
ENGINE="target/release/parity --worlds $PWD/parity/worlds" GOLDENS=$PWD/parity/goldens bin/rails engine:parity_diff
```

In the shared-database mode the game repository's runner prepares the world in a
database file of its own, plays each `reseed:` step itself, and asks for one
typed step at a time:

```sh
ENGINE_STEP=3 target/release/parity --database walk.sqlite3 --player first script.yml
```

It prints that step's one dump, with `shown` null for the runner to fill.
Each player's playthrough is the story's playthrough in the order the
players first appear in the script, started when it has none yet.

## Licence

MIT OR Apache-2.0, at your option, see `LICENSE-MIT` and `LICENSE-APACHE`.
