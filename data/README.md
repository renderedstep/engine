# Vendored engine data

These YAML files are copied byte for byte from the Ruby engine's repository,
[renderedstep/server](https://github.com/renderedstep/server), directory
`config/engine/`, at commit `659ca6ad5d8246f2fd7edee4bbd1e728e8148f59` (`playthrough/volition/weights.yml`
at `f15d07b1bfff79cda40cb59e0269c8b38c6592f1`, where the others are unchanged). They
are the one copy of their strings in this crate: `src/data.rs` compiles them
in and reads them, and no string in them is retyped anywhere else.

| File | Source | Read by |
| --- | --- | --- |
| `playthrough/grammar.yml` | `config/engine/playthrough/grammar.yml` | `grammar`: the fixed grammar's verb table |
| `playthrough/classifier/request.yml` | `config/engine/playthrough/classifier/request.yml` | `cascade`: the wording of a System One request |
| `scene/generator.yml` | `config/engine/scene/generator.yml` | `arrival`: the arrival writer's instructions |
| `playthrough/classifier.yml` | `config/engine/playthrough/classifier.yml` (at `b4c9921c008ac1e459486318324aec66672a9366`) | `turn`: the classifier's instructions |
| `scene/narrator.yml` | `config/engine/scene/narrator.yml` (at `b4c9921c008ac1e459486318324aec66672a9366`) | `turn`: the narrator's instructions and what each kind of turn is |
| `location/generator.yml` | `config/engine/location/generator.yml` | `realization`: the room writer's instructions |
| `character/desires.yml` | `config/engine/character/desires.yml` | `realization`: the desire block's extra lines for a roomful |
| `scene/ending.yml` | `config/engine/scene/ending.yml` (at `b9407f4499c319748aab1ba2a5f118bf401f1901`) | `turn`: the instructions the last paragraph of a game is written under |
| `item/inscriber.yml` | `config/engine/item/inscriber.yml` (at `b9407f4499c319748aab1ba2a5f118bf401f1901`) | `turn`: the instructions the words on a readable thing are written under |
| `playthrough/volition/weights.yml` | `config/engine/playthrough/volition/weights.yml` | `turn`: how heavily each person's pursuit weighs each act on volition's die |

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
