# Golden vectors

Each of these JSON files holds one portion of the engine's pure rules as cases
of named inputs and the exact output for them. `docs/engine-vectors.md` in the
game repository, [renderedstep/server](https://github.com/renderedstep/server),
documents the format (version 1).

**Two owners.** Most portions are rules the game's Ruby code still runs (world
creation, seeding, repair, the doctor, the benches), so the game exports them
from that code, and these are verbatim copies of its `test/engine_vectors/` at
commit `f121cd54da57673b17497cf7c94e156c002dedc8`, the last to change any of
them. The portions named in `ENGINE_OWNED` are this crate's: no Ruby code runs
them any more but the game's Ruby turn loop, which no player plays, so the game
no longer exports them. The five line-reading portions (`grammar`,
`grammar_corpus`, `slash_menu`, `classifier_intent`, `refusal`) joined
`shuffle_connections` there once the game read its panels, verbs and slash
menu off this crate (`Engine::glance`). They are blessed here from this crate's answers, as a
reviewed diff, and the game vendors them at the commit it pins, checked byte for
byte there (`bin/rails engine:vendored`):

```sh
BLESS=1 cargo test --test vectors   # rewrite each engine-owned portion from this crate
```

A bless rewrites only the outputs and constants that moved, so an unchanged
case keeps its bytes; a new case is added to the file's `cases` with any
output and blessed. A portion joins `ENGINE_OWNED` only once the game's Ruby
copy of its rule runs nowhere but that loop.

The dice and geometry portions were first copied at
`49e5388ea768dbe75814157177f4a51912cc56a1`, and the line-reading portions
(`grammar`, `grammar_corpus`, `slash_menu`, `classifier_intent`, `cascade`,
`refusal`) at `7d989d7eb3ac8f71f117c77e1cd0573a7674baed`; all of them are
byte-identical at the commit above, but for `grammar_corpus`, which gained the
lines the labelled classifier corpus gained there. Those stand in rooms written in each
file's `worlds` constant; `lib/engine_vectors/room.rb` in that repository
documents a room's shape.

Commit `659ca6ad5d8246f2fd7edee4bbd1e728e8148f59` added the request-building portions (`classifier_request`,
`volition_request`, `moment`, `ledger`, `memory`, `plan`, `request_identity`,
`kept_requests`). Most of their cases hold every row the database held as a
`records` input; `lib/engine_vectors/records.rb` in that repository documents
the shape. Since the game's benches build their requests through this crate,
the builders the game's Ruby code no longer has are this crate's portions:
`cascade`, `classifier_request`, `volition_request`, `moment`, `ledger`,
`memory`, and `dialogue_requests`, the dialogue bench's cases that
`kept_requests` held until then. `kept_requests` keeps the arrival and room
writer's cases, whose Ruby builders still write a world's rooms and its
opening arrival. Two `dialogue_requests` cases need what the rows do not
hold: the line the player typed and the character's stored answer. Those are
in `tests/fixtures/dialogue_replay.json`, which names the files they come
from.

Never edit the game's portions by hand. To refresh them, in the game's
repository:

```sh
bin/rails engine:vectors
```

then copy `test/engine_vectors/*.json` here (the engine-owned files there are
already this repository's), update the commit above, and run `cargo test`. A
case that fails is a rule the game's Ruby code changed; port the change. A new
format version fails every portion until the harness in `tests/vectors/main.rs`
learns to read it.
