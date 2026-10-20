# sony

The repository of the crate `sony`, being what one platform holder decided, as far as it
can be derived, and the gates that keep it honest. What the crate is, is in
[`crates/sony`](./crates/sony/README.md) and its book.

## Working

Everything runs from the development shell, and every action goes through moon.

```
nix develop
moon run :ci
```

`moon run :ci` runs the checks of the crate and every gate. Each check of the flake is a
task of its own, so a change reruns only what it really touches. A task is named for the
tool it runs, so `cargo/check`, `cargo/clippy`, `cargo/nextest/debug`, `cargo/doc`,
`nix/build` and `nix/build/coverage`.

Tests run under nextest and never under `cargo test`. There are two profiles in
`.config/nextest.toml` and a task for each. `cargo/nextest/debug` stops at the first
failure and prints it at once, which is the loop to run while working.
`cargo/nextest/release` runs every test whatever fails and repeats each failure at the
end, which is what `ci` and the crate gate of the flake run.

```
moon run sony:cargo/nextest/debug
moon run sony:cargo/clippy
moon run sony:mdbook/serve
```

## The gates

Every rule of this repository is a gate of the flake that refuses the tree, and
`docs/OUTPUTS.md` names every one of them.

```
nix flake check                                     every gate
nix build --no-link .#checks.x86_64-linux.word      one of them
moon run workspace:nix/build/word                   the same through moon
```

## Where things are

```
crates    the crate
docs      the outputs of the flake
nix       the flake, one directory per output
.moon     the task definitions every project inherits
.config   what a tool reads, being the nextest profiles
```
