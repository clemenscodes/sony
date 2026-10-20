# Running it

Every action goes through moon.

```
moon run sony:cargo/nextest/debug
moon run sony:cargo/build/release
moon run sony:nix/build
```

The run tasks this crate would inherit are excluded in its `moon.yml`, so no moon task
runs the binary.

The concepts are the name a function was given and the number a module table
writes for it.

```
sony name hash <NAME>
sony identifier resolve <IDENTIFIER>
```

Hashing cannot fail on a name, because a hash is defined for every text. It fails on
a writing that is no name at all, being empty or made of anything other than
letters, digits and the underscore.

Resolving can fail. A writing that is not eight upper case hexadecimal digits
is no identifier, and an identifier no name in the dictionary yields is named back
so that it is clear the number was read and not found.

```
[SONY] 32faaf58 is no identifier.
[SONY] no name of this dictionary yields DEADBEEF.
```

The status is one when anything failed and zero when nothing did.

## The builds

```
nix build .#sony
nix build .#sony-musl
nix build .#sony-windows
```

All of them are gates, so a target that is claimed and no longer builds is loud the
day it stops. The host build also runs the tests, because `doCheck` is on for it
and off for the musl build.

## Without nix

The image builds on the shared toolchain image and keeps nothing of it, being the
closure of one binary on `scratch`.

```
moon run sony:docker/build
docker run --rm sony identifier resolve 32FAAF58
```

The names are compiled into the binary, so the image needs nothing beside it and no
volume has to be mounted for it to answer.
