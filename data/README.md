# Vendored engine data

These YAML files are copied byte for byte from the Ruby engine's repository,
[calebl/text-adventure](https://github.com/calebl/text-adventure), directory
`config/engine/`, at commit `659ca6ad5d8246f2fd7edee4bbd1e728e8148f59`. They
are the one copy of their strings in this crate: `src/data.rs` compiles them
in and reads them, and no string in them is retyped anywhere else.

| File | Source | Read by |
| --- | --- | --- |
| `playthrough/grammar.yml` | `config/engine/playthrough/grammar.yml` | `grammar`: the fixed grammar's verb table |
| `playthrough/classifier/request.yml` | `config/engine/playthrough/classifier/request.yml` | `cascade`: the wording of a System One request |
| `scene/generator.yml` | `config/engine/scene/generator.yml` | `arrival`: the arrival writer's instructions |
| `location/generator.yml` | `config/engine/location/generator.yml` | `realization`: the room writer's instructions |
| `character/desires.yml` | `config/engine/character/desires.yml` | `realization`: the desire block's extra lines for a roomful |

Their comments name files in the Ruby repository, where the reasons for each
value are written.

Never edit them by hand. To refresh them, copy the files from that directory
at a newer commit, update the commit above, and run `cargo test`: the
`grammar`, `cascade` and `kept_requests` vector portions read through them.

Some wording the builders send still lives in the Ruby classes rather than in
`config/engine/` (the schema descriptions, the narration and character
contexts, most of the room writer's and the character pass's prompts). This
crate writes those strings in the module that ports each class, and the
vectors hold both copies to the same bytes; when the Ruby engine moves one
into a data file, it moves here too.
