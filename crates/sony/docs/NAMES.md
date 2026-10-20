# Where the names come from

A number is the first four bytes of the SHA-1 over the name followed by a fixed
suffix of sixteen bytes, read little endian. Nothing else goes into it, so the same
name yields the same number everywhere and forever.

```
sony name hash cellHttpUtilParseUri
[SONY] cellHttpUtilParseUri yields 32FAAF58.
```

That direction always works and needs no table at all. The other direction cannot
be computed, because a hash is one way, so it needs a name to try.

## Why a list of names is not a source

`assets/names.txt` is one name to a line and nothing else. It is a
list of candidates in the sense a word list is one for a password, and it is not
where the answers come from.

The answers come from the hash. Every name in that file is put through the
derivation when the dictionary is made, and the number it lands on is the number
the dictionary is keyed by. A name that was invented would simply land somewhere no
executable ever points at, and it could not make a wrong answer, only an unused
row.

This is why the crate writes **no numbers at all**. There is nothing to mistype,
nothing to copy wrongly out of somewhere else and nothing that can drift, because
there is only one side and the other is computed. A test walks all of the dictionary
and sets every entry against its own name.

The candidates were read out of the source of the emulator this repository pins,
which registers every function of it under the name it has. That is where to
look when a name is missing, and adding one is a line in the file.

## What it reaches on the disc's executable

Measured against the findings of one executable, which names its imports by module
and number.

| | |
|---|---|
| distinct numbers the executable imports | 447 |
| of those the dictionary resolves | 447 |

```
sony identifier resolve 9647570B
[SONY] 9647570B is the identifier of sendto.
```

Two names may appear in two modules and yield one number, which three of these
do, and that is the same name twice rather than two names in conflict. Two
different names landing on one number would make a number stop naming one function,
so the dictionary cannot be made at all in that case and names the two that
collided.
