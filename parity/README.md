# The engine sweep

The Ruby engine's stored scripts of typed lines, and what it wrote after
every step of each, copied from
[renderedstep/server](https://github.com/renderedstep/server) at commit
`f15d07b1bfff79cda40cb59e0269c8b38c6592f1`, and unchanged at `b9407f4499c319748aab1ba2a5f118bf401f1901`,
the commit whose runner CI plays them through and whose schema the worlds
are built from. That repository's
`docs/engine-parity.md` documents the dump and both command contracts.

| Directory | Source | What it is |
| --- | --- | --- |
| `scripts/` | `lib/engine_sweep/scripts/*.yml`, less their comments and `why:` notes | each walk: its world, the lines typed and what each step expects |
| `goldens/` | `test/engine_parity/*.json` | the Ruby engine's dump after every step of each |
| `worlds/` | produced from `lib/engine_sweep/worlds/` and `db/seeds/worlds/` (below) | each world as the Ruby engine loads it for a walk, as SQL text |
| `records/` | produced from the Ruby engine by `records.rb` (below) | for some scripts, every request each step handed a provider and every row the walk wrote |
| `records.rb` | this repository | the runner script that produces `records/` |
| `PASSING` | this repository | the scripts this engine plays exactly as the Ruby engine does |
| `PASSING_WITH_RUNNER` | this repository | the scripts that agree only when the Ruby engine's runner plays them: those with `reseed:` steps, and those whose browser steps assert `shown` |
| `runner.sh` | this repository | plays both lists through the Ruby engine's runner in its shared-database mode |

Three scripts are ahead of that commit: `a-room-written-while-the-arc-waits`,
`an-ending-kept-when-the-narrator-fails` and
`a-deadline-places-what-the-arc-waits-for`, with the worlds
`a-bell-nobody-has-rung` and `a-yard-before-the-winter` they walk. They
were written in this repository and played by the Ruby engine at the
commit above, with two changes to its sweep that they need: a browser
reply may answer the `ending` and `inscription` passes, and the people a
declared realization brings into the story join the walk's reference
wherever they stand, not only in the room it wrote. Both are proposed to
the game repository with the scripts; once it has them, a refresh copies
them from there like the rest.

Never edit the scripts or the goldens by hand. The scripts are re-emitted
by Ruby's YAML library with the comments and the `why:` notes (which no
engine reads) taken out:

```sh
ruby -ryaml -e 'd = YAML.safe_load_file(ARGV[0]); d.delete("why"); d["steps"].each { |s| s.delete("why") }; print YAML.dump(d)' <script>
```

## The worlds

A walk plays its own copy of a world: the world file loaded by
`WorldSeed::Loader` under the title with `EngineSweep::Walk::TITLE_SUFFIX`,
with every table's id counter pinned at `EngineSweep::Walk::ID_BASE`, so the
same script gets the same ids, and so the same dice, on any database. Each
file here is that database as `sqlite3 <database> .dump` writes it: the whole
schema at the version `store::SCHEMA_VERSION` names, and the world, before
any playthrough exists. A script's world is the file named after its story's
title (`WorldSeed.slug`).

To produce one, in a scratch copy of the Ruby engine's repository (never a
working checkout) with both provider keys unset, load the schema into an
empty database (`RAILS_ENV=test bin/rails db:schema:load`), copy it once per
world, and against each copy run, with `DATABASE_URL=sqlite3:<copy>`:

```ruby
# bin/rails runner load_world.rb <a script that walks the world>
script = EngineSweep.scripts.find { |s| s.name == ARGV.fetch(0) }
walk = EngineSweep::Walk.new(script)
EngineSweep.without_a_model do
  walk.send(:pin_ids!)
  walk.send(:load_world!)
end
```

then `sqlite3 <copy> .dump > worlds/<slug>.sql`. The `created_at` and
`updated_at` columns carry the time it was run; nothing the engine plays
reads them.

## The records

A dump says where the player stands and what they hold; it does not say
what the models were asked, or every row a step wrote. For a script with a
file in `records/`, the check also holds each step to the Ruby engine's
requests (every provider call a browser step made, with its instructions,
its line and its schema, and every System One state and question set) and
to its rows: for each table `parity::WRITTEN_TABLES` names, every row that
is not in the world as it was loaded, and every loaded row that is gone,
compared column for column less `created_at` and `updated_at`. A script
agrees only when its dumps, its requests and its rows all do.

To produce one, in the same scratch copy with its test database's schema
loaded and both provider keys unset:

```sh
RAILS_ENV=test bin/rails runner <this repository>/parity/records.rb <output directory> <script>...
```

and copy `records/<script>.json` from the output directory. It also writes
the script's golden file, which is the one `bin/rails engine:parity`
writes.

## Refreshing

Copy `scripts/` and `goldens/` from a newer commit, rebuild `worlds/` as
above, update the commit at the top, and run `cargo test`. A script that
stops agreeing is a rule the Ruby engine changed; port the change. A new
migration changes the schema version. The engine still opens the worlds
when the migration leaves every table it touches as `store::SHAPE` records
them; otherwise it refuses them until whatever the migration changed is
ported. Either way, move `store::SCHEMA_VERSION` to the new version and
rebuild the worlds, and when the shape changed, replace `src/store/shape.txt`
with the shape the failing
`every_world_has_the_shape_the_engine_is_written_against` test prints.

## What this engine cannot play yet

A step with `browser:` plays a submission through `Engine::submit` with the
step's fixed provider replies (`model::Replay`), and fails when a call comes
out of order, a reply is left over, or a prompt misses what its reply says it
must include or leave out. A browser step this engine cannot play yet (see
the crate README) stops its script with `Error::Unsupported`, and so does a
step with `reseed:`, which reloads the world file and needs the world loader.
`cargo run --release --bin parity -- --check parity` names every other
script's first divergence. A browser step that asserts `shown` agrees only
through the runner, which renders the notices.

The world loader stays in Ruby. A script with `reseed:` steps is played in
the shared-database mode instead (`--database`, `--player` and `ENGINE_STEP`,
see the crate README): the Ruby engine's runner owns the database, plays
each `reseed:` step with `WorldSeed::Loader`, and asks this engine for each
typed step in turn. `parity/runner.sh <text-adventure checkout> <parity binary>`
does that for every script in `PASSING` and `PASSING_WITH_RUNNER`, and CI
runs it on Linux against the commit named at the top.
