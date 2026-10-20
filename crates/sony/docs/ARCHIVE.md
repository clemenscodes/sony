# The archive

What a PlayStation archive is, and how the files inside it are found.

## What is on the disc

The disc writes twelve of them under `build/main`, being `pak22.psarc`, `text1.psarc`,
`actor27.psarc`, `dc1.psarc`, `sound1/banks.psarc`, `sound1/music.psarc`,
`sound1/streams.psarc` and one `sound1/speech` archive each for English, Polish,
Portuguese, Russian and Spanish, and the store serves more of them. Each begins with the
four bytes `PSAR` and each is one file made of many. Nothing in it is encrypted. It is
packed, and the packing is zlib.

An archive has five parts, one after the other, in this order, and the last of them is
empty in ten of the twelve archives of this disc.

| Part | Where it begins | What is there |
|---|---|---|
| Header | 0 | Thirty two bytes that state how the rest is laid out. |
| Records | 32 | One record per file, thirty bytes each. |
| Block lengths | behind the records | How long every packed block is, two bytes each. |
| Blocks | where the header states the table of contents ends | The bytes of every file, packed. |
| Remnant | behind the last block | Bytes that no record and no block reaches. |

An archive writes every number with the high byte first.

## The header

| Offset | Bytes | What is there |
|---|---|---|
| 0x00 | 4 | The magic, `PSAR`. |
| 0x04 | 2 | The major version. |
| 0x06 | 2 | The minor version. |
| 0x08 | 4 | The name of the packing, `zlib`. |
| 0x0C | 4 | How long the table of contents is, being where the blocks begin. |
| 0x10 | 4 | How long one record is. |
| 0x14 | 4 | How many records there are. |
| 0x18 | 4 | How long one block is when it is unpacked. |
| 0x1C | 4 | The flags. |

The table of contents is the header, the records and the block lengths together, so
the number at `0x0C` is where the first block begins.

## One record

Thirty bytes, and every archive on this disc writes thirty.

| Offset in the record | Bytes | What is there |
|---|---|---|
| 0x00 | 16 | The MD5 digest of the file path. |
| 0x10 | 4 | Which block the file begins at. |
| 0x14 | 5 | How long the file is when it is unpacked. |
| 0x19 | 5 | Where the first block of the file begins in the archive. |

Two of these fields are five bytes and not four or eight. Five bytes reach 1099511627775,
and `pak22.psarc` is 5596444529 bytes, so four would not reach the end of it and eight
would be three bytes of zero in every record of every archive Sony ever wrote.

## The block lengths

Behind the last record comes one number per block, and every one of them is two bytes
long. The length follows from how long an unpacked block is. A packed block is never
longer than an unpacked one, an unpacked one is 65536 bytes here, and a length of zero
means that the block is an unpacked one. So the numbers to write are 0 to 65535 and
two bytes are exactly enough.

How many blocks there are is nowhere in the archive. It follows from the arithmetic.

```
blocks = (length of the table of contents - 32 - records * 30) / 2
```

## The remnant

Where the blocks end is where the archive is read out, and two archives of this disc go
on past it. What is behind the last block is the remnant.

| Archive | Where the last block ends | Bytes behind it |
|---|---|---|
| text1.psarc | 2212439 | 73942 |
| pak22.psarc | 5590777738 | 5666791 |

The other ten end exactly where their last block ends.

The remnant is no padding. Of the 73942 bytes behind the last block of `text1.psarc`,
73667 are not zero, a zlib stream begins 53834 bytes into it and yields 48254
bytes of subtitle text, and those bytes are nowhere among what the records of the
archive yield.

Every record of both archives begins exactly where the block lengths in front of its
first block put it, and the last record of both yields exactly as many bytes as it
states, so the remnant is behind the archive rather than missing from it.

A reader keeps the remnant because an archive that was read goes back out byte for
byte, and a writer puts it back behind the last block. An archive packed from files has
none.

## The manifest

The first record is not a file of the archive's contents. It is the manifest, and it is
the paths of all the other files, one to a line, in the order of the records behind
it. Its own path is nowhere, which is why its digest is sixteen zero bytes.

The record at index one is the file the first line of the manifest names, and so on.

## How a file is read out

1. Take the record. It names which block the file begins at and where that block is.
2. Read the block lengths from that block onward. Any other length is how many bytes the
   block is.
3. Go on to the next block until as many bytes have come out as the record states the file
   is long.

A file of no bytes has no block at all, so nothing is read.

Two different questions decide what a block is, and they are asked of two different
things.

**How long is it.** The block lengths answer that, and a length of zero is not an empty
block. A packed block is never longer than an unpacked one, so the one length that
does not fit in two bytes is 65536 itself, and zero is what a writer puts there for
it. A length of zero therefore means a block that did not pack, and the reader takes
65536 bytes.

**Is it packed.** The block's own first byte answers that, and the block lengths cannot.
A packer that made a block longer by packing it writes the block as it is, at its own
true length, and that length is neither zero nor anything else a reader could tell from
the table. So a block is inflated where its first byte is `0x78`, which is what zlib
writes in front of a stream of a window of 32768 bytes, and taken as it is where it is
not.

## How an archive is made

1. Sort nothing. The order of the records is the order of the manifest and the order of
   the manifest is what a reader gets back.
2. Write the manifest first, being every file path one to a line, and give it the
   digest of sixteen zero bytes.
3. Cut every file into pieces of 65536 bytes and pack each piece on its own. Where the
   packed piece is not shorter than the piece, write the piece as it is.
4. Write the length of every block in the order of the blocks.
5. Write one record per file with the digest of its path, the block it begins at, how
   long it is unpacked and where its first block begins.
6. Write the header last, because it states how long the table of contents is and that is
   only known once the records and the block lengths are there.
7. Write the remnant behind the last block, where the archive was read from one that has
   one.

## Where it fails

| Error | When |
|---|---|
| Unarchived | The file does not begin with `PSAR`. |
| Unversioned | The version is not one this repository has read. |
| Unpackable | The packing is not `zlib`. |
| Truncated | The archive ends before a record, a block length or a block it names. |
| Unpacked | A block did not come back out of its zlib packing. |
| Unlisted | The manifest names a different number of files than there are records behind it. |
| Misnamed | The digest a record gives is not the digest of the file path the manifest gives it. |
| Displaced | The offset a record gives is not where the block it names begins. |
| Unpathed | A line of the manifest is no path from the root of the archive to a file. |

The packing is four letters and Sony defined two of them. Every archive measured here
states `zlib`. An archive that writes `lzma` fails by name rather than being read as though it
were packed some other way, because a file unpacked the wrong way is a wrong file that
nothing downstream would notice.

## What the crate calls these

The words of this page are the words of the code.

| The word | The type | What it is made of |
|---|---|---|
| Archive | `Archive` | a header, the records, the blocks and the remnant |
| Header | `Header` | the thirty two bytes in front |
| Archive version | `Version` | a major version and a minor version |
| Packer | `Packer` | the four letters that name what packed the blocks |
| Record | `Record` | a digest, a first block, an unpacked length and an offset |
| Digest | `Digest` | the sixteen bytes MD5 yields over a file path |
| Block | `Block` | the bytes of one block, exactly as the archive gives them |
| Remnant | `Blob` | the bytes of the archive behind its last block |
| Manifest | `Manifest` | the file paths, one to a line, in the order of the records |
| File path | `FilePath` | the way from the root of the archive to one file, made of letters |
| File | `File` | one file path and the bytes that belong to it |
| Files | `Files` | every file of an archive, which is what comes out and what goes in |
| Cursor | `Cursor` | an archive read file by file, one at a time |

`Archive` is the disc. It exists for every file that is laid out as this page states, even
for one whose records and whose block lengths disagree about where a file begins, because
that is what such a file states about itself.

`Files` is the one whose structure cannot be wrong. A file is a file path and its bytes
and nothing else, there is no offset in it to be wrong and no length to disagree with
anything, and that is why extracting yields it and why packing takes it.

A file path is not a file name. `build/main/pak22/net.pak` is the way from the
root of the archive to the file, and the name of that file is `net`, without the way
in front of it and without the `.pak` behind it. An archive has no directories to
walk, so the way appears in full on every line of the manifest, and it is
those letters, all of them, that the digest is taken over.

## What was measured

Every number here was read out of the archives on this disc and out of the one the store
serves. Nothing in this page was taken from another repository.

| Archive | Version | Packing | One record | One block | Flags | Records | Table of contents |
|---|---|---|---|---|---|---|---|
| dc1.psarc | 1.3 | zlib | 30 | 65536 | 0 | 178 | 6416 |
| text1.psarc | 1.3 | zlib | 30 | 65536 | 0 | 52 | 1794 |
| actor27.psarc | 1.3 | zlib | 30 | 65536 | 0 | 1312 | 132186 |
| pak22.psarc | 1.3 | zlib | 30 | 65536 | 0 | 782 | 274000 |
| net-patch9.bin.psarc | 1.4 | zlib | 30 | 65536 | 0 | 2 | 100 |

The smallest of them is small enough to read by hand, and it proves all of the layout at
once. `net-patch9.bin.psarc` is 40135 bytes and is one file.

```
header        50 53 41 52 00 01 00 04 7A 6C 69 62 00 00 00 64
              00 00 00 1E 00 00 00 02 00 01 00 00 00 00 00 00

record 0      00000000000000000000000000000000 00000000 000000000E 0000000064
record 1      1AA7B23FF54BF508C1D86EAA35340A14 00000001 000002AFF8 0000000072

block lengths 000E 483C 25BD 2E5C
```

The manifest is 14 bytes at 100 and reads `net-patch9.bin`, which is 14 characters. Its
block is 14 bytes long and begins with `6E`, so it is not packed and is taken as it is.

The MD5 digest of `net-patch9.bin` is `1AA7B23FF54BF508C1D86EAA35340A14`, which is what
record 1 gives.

The three arithmetics all close exactly.

```
100 + 14 + 18492 + 9661 + 11868 = 40135, which is the length of the file
65536 + 65536 + 45048 = 176120, which is what record 1 states the file is
32 + 2 * 30 + 4 * 2 = 100, which is what the header states the table of contents is
```

All twelve archives of the disc were read in and put back out, and all twelve came
back byte for byte.
