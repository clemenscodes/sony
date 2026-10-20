# The outputs

Every output of the flake is in this table, and a gate refuses a flake and a table
that do not state the same thing. An output that is added and not recorded here
fails, and so does a row that names an output that is gone.

| Output | Kind | What it is |
|---|---|---|
| `rustToolchain` | package | the rust toolchain this repository pins, the one every crate is built with |
| `book` | check | every book builds, names no page that is missing and leaves none out |
| `word` | check | nothing here uses a word this repository refuses |
| `comment` | check | no crate has a comment |
| `coverage-sony` | check | every region, every function and every line of that crate is read by a test |
| `deadnix` | check | no nix binding goes unread |
| `format` | check | every nix file is formatted |
| `interface` | check | no module publishes what another already published, and no glob flattens one |
| `outputs` | check | this table and the flake state the same thing |
| `pipeline` | check | every check of the flake is one a moon task builds |
| `rustfmt` | check | every rust file of the workspace is formatted |
| `signature` | check | no cast outside a conversion, no Result or Response whose name does not begin with try, and no bare bytes answered |
| `statix` | check | no nix file has a known smell |
| `test` | check | no test module keeps a function beside its tests, and no main decides anything |
| `sony` | check | the dictionary still builds and its tests still pass |
| `default` | shell | the development shell, which offers every tool this repository uses |
