# Engine data

These YAML files are the one home of the words a model is handed and the
tables a rule reads. `src/data.rs` compiles them in and reads them, and no
string in them is retyped anywhere else. They were first copied from the
game repository, [renderedstep/server](https://github.com/renderedstep/server),
directory `config/engine/`; they are owned here now, and the game keeps no
copy of them: it reads these same bytes through its extension, which is
built against the commit it pins (`data::files`).

| File | Read by |
| --- | --- |
| `playthrough/grammar.yml` | `grammar`: the fixed grammar's verb table |
| `playthrough/classifier/request.yml` | `cascade`: the wording of a System One request |
| `playthrough/classifier.yml` | `classifier`: the classifier's instructions |
| `scene/generator.yml` | `arrival`: the arrival writer's instructions |
| `scene/narrator.yml` | `narration`: the narrator's instructions and what each kind of turn is |
| `scene/ending.yml` | `turn`: the instructions the last paragraph of a game is written under |
| `location/generator.yml` | `realization`: the room writer's instructions |
| `character/desires.yml` | `realization`: the desire block's extra lines for a roomful |
| `item/inscriber.yml` | `turn`: the instructions the words on a readable thing are written under |
| `playthrough/volition/weights.yml` | `turn`: how heavily each person's pursuit weighs each act on volition's die |
| `playthrough/volition/speech.yml` | `turn`: how often each person's pursuit speaks up unasked on the speech die, and what bounds it |
| `physics.yml` | `room`, `turn`: the bulk and thrown-damage tables a throw reads |
| `location/kind.yml` | `kind`: what sort of place a room may be, how cluttered, and the sort each room of a building is dealt; `schemas` offers the first two to the exits call and the buildings to the place call |
| `item/kits.yml` | `kit`: what stands in a room of each sort, what lies on or in each fixed piece in plain view, and the small stuff each density lies about; `turn` furnishes a room from it as the room is realized, and `realization` states what it wrote |

Their comments name files in the game repository, where the reasons for
each value were first written.

Edit them here, and run `cargo test`: the `grammar`, `cascade` and
`kept_requests` vector portions read through them, and a portion this
engine owns is blessed again (`vectors/README.md`). A change to a file the
game's own code also reads -- world creation writes rooms and their opening
arrivals with `location/generator.yml`, `location/kind.yml`, `item/kits.yml`,
`character/desires.yml` and `scene/generator.yml` -- reaches it when the game moves its pin, and moves
the digests its benches print; the game's `rake eval:prompt_digest`,
`eval:classifier_digest` and `eval:realization_digest` are how a change is
seen there.

Some wording the builders send lives in the module that builds the request
rather than in a data file (the schema descriptions, the narration and
character contexts, most of the room writer's and the character pass's
prompts), because its shape is code.

`physics.yml` has always been this engine's: `src/data.rs`
(`physics()`) declares its shape and refuses any other. Its `bulk` and
`thrown_damage` tables are what a throw reads (`room`, `turn`), and its
`gravity`, `fall_die` and `fall_save` what a fall reads (`physics`, `turn`),
and its `fragility`, `break_die`, `height_step` and `surface` what a break
reads (`physics`, `turn`), and its `throw_range`, `throw_reach_per_strength`
and `throw_reach_cap` what a throw's range reads (`physics`, `turn`). Edit it
here, run `cargo test`, and bless the `physics`, `breakage` and `range`
vector portions (`vectors/README.md`).
