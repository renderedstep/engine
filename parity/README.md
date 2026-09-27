# The engine sweep

The game's stored scripts of typed lines, and the state after every step of
each: this engine's goldens. The game,
[renderedstep/server](https://github.com/renderedstep/server), keeps the
scripts and vendors the goldens at the commit it pins, checked byte for byte
there (`bin/rails engine:vendored`); its `docs/engine-parity.md` documents
the dump, both command contracts and how a change moves through both
repositories.

| Directory | What it is | Who writes it |
| --- | --- | --- |
| `scripts/` | each walk: its world, the lines typed and what each step expects | copied from the game's `lib/engine_sweep/scripts/*.yml`, less their comments and `why:` notes |
| `goldens/` | the dump after every step of each script | this engine: `parity --write`, and `runner.sh --write` for the runner's scripts |
| `records/` | for some scripts, every request each step handed a provider and every row the walk wrote | this engine: `parity --write` |
| `worlds/` | each world as the game's loader loads it for a walk, as SQL text | the game's loader (below) |
| `records.rb` | the runner script that produced the first `records/`, from the game's Ruby loop | kept for reference |
| `PASSING` | the scripts this engine plays whole, on its own | this repository |
| `PASSING_WITH_RUNNER` | the scripts only the game's runner plays whole: those with `reseed:` steps, and those whose browser steps assert `shown` | this repository |
| `runner.sh` | plays both lists through the game's runner in its shared-database mode, against `goldens/`, or writes the runner's goldens | this repository |

The goldens were written by the game's Ruby turn loop until this engine took
them over, and every one was unchanged when it did: the game's commit tagged
`ruby-reference-final` is where every turn moved to this engine, and its
goldens are the ones here. From then on a golden changes only as a reviewed
diff in a pull request here.

Never edit the scripts or the goldens by hand. The scripts are re-emitted
by Ruby's YAML library with the comments and the `why:` notes (which no
engine reads) taken out:

```sh
ruby -ryaml -e 'd = YAML.safe_load_file(ARGV[0]); d.delete("why"); d["steps"].each { |s| s.delete("why") }; print YAML.dump(d)' <script>
```

## Changing a rule

1. Change the rule, with its tests, and add or change the script that walks
   it (in the game's repository too, with its `why:` notes).
2. Write the goldens again, and read the diff: every key that moved is a
   behaviour change, and the pull request says which and why.

   ```sh
   cargo run --release --bin parity -- --write parity
   parity/runner.sh --write <server checkout> target/release/parity
   ```

   `parity --write` rewrites a golden only when its dumps moved, and a records
   file only when a request or a row did, so an unchanged file keeps its bytes.
   `runner.sh --write` writes the scripts in `PASSING_WITH_RUNNER`, through a
   server checkout with its test schema loaded.
3. CI plays every script against these goldens, whole and through the game's
   runner, and runs the game's `engine:rust_gates` with the extension built
   from this checkout: the goldens as the game's Ruby reads the rows, the
   invariants, and what the game's doctor and audit say after each script
   (`test/engine_parity/<script>.checks.json` there). `SERVER_COMMIT` in
   `.github/workflows/ci.yml` names the game commit it runs against; a change
   that moves what the doctor or the audit says, or that needs a new script
   or a migration, points it at a game branch that carries them.
4. Once this merges, the game's pull request moves the pin and copies the
   goldens, the scripts and any vector portion this engine owns.

## The worlds

A walk plays its own copy of a world: the world file loaded by
`WorldSeed::Loader` under the title with `EngineSweep::Walk::TITLE_SUFFIX`,
with every table's id counter pinned at `EngineSweep::Walk::ID_BASE`, so the
same script gets the same ids, and so the same dice, on any database. Each
file here is that database as `sqlite3 <database> .dump` writes it: the whole
schema at the version `store::SCHEMA_VERSION` names, and the world, before
any playthrough exists. A script's world is the file named after its story's
title (`WorldSeed.slug`).

To produce one, in a scratch copy of the game's repository (never a
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
file in `records/`, the check also holds each step to its requests (every
provider call a browser step made, with its instructions, its line and its
schema, and every System One state and question set) and to its rows: for
each table `parity::WRITTEN_TABLES` names, every row that is not in the
world as it was loaded, and every loaded row that is gone, compared column
for column less `created_at` and `updated_at`. A script agrees only when its
dumps, its requests and its rows all do.

The first records came from the game's Ruby loop, through `records.rb`;
`parity --write` writes them now, for a script that already has one. To give
a script records, create an empty `records/<script>.json` (`{"steps": []}`)
and write.

## Refreshing the worlds

A new migration changes the schema version. The engine still opens the
worlds when the migration leaves every table it touches as `store::SHAPE`
records them; otherwise it refuses them until whatever the migration changed
is ported. Either way, move `store::SCHEMA_VERSION` to the new version and
rebuild the worlds as above, and when the shape changed, replace
`src/store/shape.txt` with the shape the failing
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
see the crate README): the game's runner owns the database, plays each
`reseed:` step with `WorldSeed::Loader`, and asks this engine for each typed
step in turn. `parity/runner.sh <server checkout> <parity binary>` does that
for every script in `PASSING` and `PASSING_WITH_RUNNER`, against `goldens/`,
and CI runs it on Linux against the game commit `SERVER_COMMIT` names.
