# Golden vectors

These JSON files are copied verbatim from the Ruby engine's repository,
[renderedstep/server](https://github.com/renderedstep/server), directory
`test/engine_vectors/`, at commit `659ca6ad5d8246f2fd7edee4bbd1e728e8148f59`.
Each holds one portion of the engine's pure rules as cases of named inputs and
the exact output the Ruby code gives for them. `docs/engine-vectors.md` in that
repository documents the format (version 1).

The dice and geometry portions were first copied at
`49e5388ea768dbe75814157177f4a51912cc56a1`, and the line-reading portions
(`grammar`, `grammar_corpus`, `slash_menu`, `classifier_intent`, `cascade`,
`refusal`) at `7d989d7eb3ac8f71f117c77e1cd0573a7674baed`; all of them are
byte-identical at the commit above. Those stand in rooms written in each
file's `worlds` constant; `lib/engine_vectors/room.rb` in that repository
documents a room's shape.

The commit above added the request-building portions (`classifier_request`,
`volition_request`, `moment`, `ledger`, `memory`, `plan`, `request_identity`,
`kept_requests`). Most of their cases hold every row the database held as a
`records` input; `lib/engine_vectors/records.rb` in that repository documents
the shape. Two `kept_requests` dialogue cases need what the rows do not hold:
the line the player typed and the character's stored answer. Those are in
`tests/fixtures/dialogue_replay.json`, copied from the same commit, which
names the files they come from.

Never edit them by hand. To refresh them, in the Ruby engine's repository:

```sh
bin/rails engine:vectors
```

then copy `test/engine_vectors/*.json` here, update the commit above, and run
`cargo test`. A case that fails is a rule the Ruby engine
changed; port the change. A new format version fails every portion until the
harness in `tests/vectors/main.rs` learns to read it.
