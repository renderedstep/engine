# Vendored engine data

These YAML files are copied byte for byte from the Ruby engine's repository,
[calebl/text-adventure](https://github.com/calebl/text-adventure), directory
`config/engine/`, at commit `7d989d7eb3ac8f71f117c77e1cd0573a7674baed`. They
are the one copy of their strings in this crate: `src/data.rs` compiles them
in and reads them, and no string in them is retyped anywhere else.

| File | Source | Read by |
| --- | --- | --- |
| `playthrough/grammar.yml` | `config/engine/playthrough/grammar.yml` | `grammar`: the fixed grammar's verb table |
| `playthrough/classifier/request.yml` | `config/engine/playthrough/classifier/request.yml` | `cascade`: the wording of a System One request |

Their comments name files in the Ruby repository, where the reasons for each
value are written.

Never edit them by hand. To refresh them, copy the files from that directory
at a newer commit, update the commit above, and run `cargo test`: the
`grammar` and `cascade` vector portions read through them.
