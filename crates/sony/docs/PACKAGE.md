# The package

What a shop sells, and how the files come out of one.

## What a shop sells

A package is one file with the ending `pkg`. It is a header, a metadata of entries, and
behind those the contents, being a table of what is in the package and the files
themselves. Everything from the contents offset onward is encrypted and nothing in front
of it is.

The name of a package file means nothing. It is a scramble of letters and it differs from
one purchase to the next. What names the package is the content identifier at 0x30 of
its header, and that is also what pairs it with the license beside it.

An archive is opened to read what is in it. A package is opened to put what is in it
somewhere. That is why the two are two concepts here and not one.

## The header

The first 0x80 bytes. A file that does not begin with `7F504B47` is no package, and a
file shorter than 0x80 bytes fails before anything else is read.

| Offset | Bytes | What is there |
|---|---|---|
| 0x00 | 4 | The magic, `7F504B47` |
| 0x04 | 2 | The release, `0x8000` for a package a shop sold |
| 0x06 | 2 | The platform, one for the PlayStation 3 |
| 0x08 | 4 | Where the metadata begins |
| 0x0C | 4 | How many entries the metadata has |
| 0x10 | 4 | How long the metadata is |
| 0x14 | 4 | How many records the file table has |
| 0x18 | 8 | How long the package is |
| 0x20 | 8 | Where the contents begin |
| 0x28 | 8 | How long the contents are |
| 0x30 | 0x30 | The content identifier, letters and zeroes |
| 0x60 | 0x10 | A digest |
| 0x70 | 0x10 | The vector the cipher of the contents starts from |

A release other than the one a shop sold is an error, and so is a platform other than the
PlayStation 3, because this repository has no file of either and would be guessing.

## The metadata

Behind the header, at the offset the header names. It is a list of entries, each one four
bytes of kind, four bytes of length, and that many bytes. This repository reads as many
entries as the header counts and fails on a package whose entries reach past the metadata.

The three packs of this repository each state eight entries, and the eight are the same
eight in all three.

| Kind | Bytes | What all three state |
|---|---|---|
| 1 | 4 | `00000002` |
| 2 | 4 | `00000004` |
| 3 | 4 | `00000008` |
| 4 | 8 | The length of the contents, to the byte, the same number the header states at 0x28 |
| 5 | 4 | `16200100` |
| 7 | 0x18 | Eight zero bytes and sixteen that differ from pack to pack |
| 8 | 8 | `0200000001000000` |
| 9 | 8 | Zero |

What a kind means is not stated here, because nothing in this repository has measured it.
The one thing measured is that the entry of kind four is the length of the contents in all
three packs.

The eight entries take 0x80 bytes and end at 0x140, and the metadata the header declares
ends at 0x180. The 0x40 bytes between them are no entry. They are not zero and they differ
from pack to pack. What they are is unknown, and RPCS3 reads them nowhere either.

None of the three packs states an entry that names an installation directory, so this
repository writes no code for one.

## How the contents come out from under the key

The contents are encrypted under AES-128 in a counter mode. The counter is the sixteen
bytes the header states at 0x70 and it advances as one full number with the high byte
first, once per sixteen bytes of the contents. The key is the same for every package a
shop sold, it is four lines long, and it comes out of
`rpcs3/Crypto/key_vault.h` where RPCS3 keeps it as `PKG_AES_KEY`. The key of a package
that never reached a shop and the key of a store demonstration unit are absent, because
no file of this repository needs either.

## The file table

The plaintext of the contents begins with the file table, one record of 0x20 bytes per
record the header counts.

| Offset | Bytes | What is there |
|---|---|---|
| 0x00 | 4 | Where the name begins, counted from the contents |
| 0x04 | 4 | How many letters the name has |
| 0x08 | 8 | Where the file begins, counted from the contents |
| 0x10 | 8 | How long the file is |
| 0x18 | 4 | The kind, and flags in the bytes in front of it |
| 0x1C | 4 | Four bytes that are zero in every record read so far |

The kinds this repository knows are 1, 2, 3, 4 and 9. Four means a directory and yields no
file. Every other one of them is a file. A kind outside those is an error rather than
something passed through.

A package has no checksum over its plaintext and RPCS3 verifies none, so the file table is
the only signal there is that the key fit. A record is therefore an error where its name is
longer than 256 letters, where its name or its file reaches past the contents, where its
name and its file begin at one offset, or where its name is no path a file can have.

The path is measured while the table is read and not while the files come out. A package
that exists therefore names every file it keeps with a path, and opening one can no longer
fail halfway, with some files already on the disc and the rest behind a name nobody can
use.

## Building one

This crate also writes a package, which is the same format in the other direction. A list
of files becomes the table, the names behind it and the files behind those, the header is
built from the count and the length of that, and the contents go out under the same key
and counter they come in under. A package built this way and read back yields the files it
was built from, which is the property a test pins.

The vector is zero in a package this repository builds. The key of a package is the same
for every package a shop sold, so a counter of its own buys nothing, and a number nobody
has to remember is the honest one.

Writing one is what makes opening one provable. Before it there was no package in this
repository other than the three purchased packs, and nothing that reads a package could be
measured without one of them.

## Where it fails

| Error | When |
|---|---|
| Length | The file ends before the header, or before an offset the header names |
| Magic | The file does not begin with `7F504B47` |
| Release | The release is not the one a shop sold |
| Platform | The platform is not the PlayStation 3 |
| Content Identifier | A byte of it is neither a letter nor zero |
| Metadata | The entries reach past the metadata |
| Record | The 0x20 bytes of a record did not read, or its kind is one this repository does not know |
| Name | A name is longer than 256 letters |
| Contents | A record names a name or a file that reaches past the contents |
| Offset | A record names its name and its file from one offset |
| File Path | A name is no letters, or it is a way out of the tree it belongs in |

## What was measured

Three purchased packages, read by this crate alone. Each states eight metadata entries.

Every one of the three states the release a shop sold, the PlayStation 3, a metadata at
0xC0 of 0xC0 bytes, and contents from 0x180. The package length each one states is its file
length to the byte, so none of them continues into a further file.

Twenty eight files came out of the three. Every one of them begins with a magic this
repository or the disc already knows, which is what shows the key and the counter were
right, because a wrong one of either yields noise and noise begins with nothing.

| Begins with | What it is | How many |
|---|---|---|
| `89504E47` | A PNG | 18 |
| `00505346` | A parameter file | 3 |
| `50534152` | An archive of Sony's, version 1.3 | 2 |
| `4E504400` | A protected file | 5 |

The ends were measured too, because a counter that drifts is right at the start and wrong
afterwards. Every PNG of the three packages ends with `49454E44AE426082`, being the last
chunk of a PNG and its own checksum.

The two archives are the reason a package is as large as it is. This crate already reads
an archive of that kind.

The five protected files all state the same header, which `docs/EDATA.md` covers. What a
package unlocks is not inside them. They are the unlocking.

```
xbreak assets check --tree <packages> --manifest <manifest>
```

measures a tree of files that came out of packages against the tags a manifest pins.
