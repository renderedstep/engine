# renderedstep-engine

The text-adventure engine's pure rules in Rust: the dice, the geometry,
reading a typed line against the room the player stands in, and building the
requests the engine hands a model. Every function
takes values and returns values. Nothing here touches a database, the
network, the clock or an async runtime. Its two dependencies are
`serde_json`, for the System One request and answers, and `yaml-rust2`, for
the engine data it compiles in (`data/`, see its README).

It reproduces the Ruby engine exactly, roll for roll, and is checked against
the golden vectors that engine exports (`vectors/`, see its README).

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

## Licence

MIT OR Apache-2.0, at your option, see `LICENSE-MIT` and `LICENSE-APACHE`.
