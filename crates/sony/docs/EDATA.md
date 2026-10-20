# The protected file

What a package brings that only a license opens.

## What it is

A file with the ending `edat`. It begins with `NPD\0`, it names the content it belongs
to, and a hash in its header shows whether the klicensee offered for it is the right one.

A package brings such files.

## The header

The first 0x100 bytes. This crate reads these parts of it and fails on a file that states
anything else, because a file of another kind was never measured here and would be
guessed at.

| Offset | Bytes | What is there | What this crate accepts |
|---|---|---|---|
| 0x00 | 4 | The magic | `4E504400`, being `NPD\0` |
| 0x04 | 4 | The version | 2 |
| 0x08 | 4 | Which kind of license opens it | 2, being a license file |
| 0x10 | 0x30 | The content identifier | Letters and zeroes |
| 0x80 | 4 | The flags | 12 |
| 0x84 | 4 | How long a block is | Any |
| 0x88 | 8 | How long the file inside is | Any |
| 0xA0 | 0x10 | The hash over the 0xA0 bytes in front of it | What the klicensee yields |

## How a klicensee is measured against it

The hash at 0xA0 is an AES-CMAC over the 0xA0 bytes in front of it. The key of that
CMAC is not the klicensee itself. The flags state 12, and the bit 8 of those flags means
the key is wrapped, so the key is the klicensee decrypted under AES-128 with the key
RPCS3 keeps as `EDAT_KEY_0` in `rpcs3/Crypto/key_vault.h`.

Where the hash this crate computes is the hash the file states, the license that yielded
the klicensee is the license of that file. Where it differs, it is not, and the file
fails.

RPCS3 has that same check in `check_header_hash` and calls it from nowhere. The line
above it states the verification it belongs to slows loading by a factor of twenty, so it
was commented out. This repository is not an emulator and has the time.

A protected file is therefore only ever made out of a blob and a klicensee together. It
cannot come about unopened, and there is no way to ask one whether it was opened,
because one that failed never became one.

## Where it fails

| Error | When |
|---|---|
| Magic | The file does not begin with `4E504400` |
| Length | The file is shorter than 0x100 bytes, or ends before its hash |
| Version | The version is not 2 |
| License | The kind of license is not 2 |
| Flags | The flags are not 12 |
| Content Identifier | A byte of it is neither a letter nor zero |
| Klicensee | The hash of the file is not the one this klicensee yields |

## What was measured

Three packages brought five protected files and they are alike in every part of the
header.

| What the header states | Value, in all five |
|---|---|
| Version | 2 |
| Kind of license | 2 |
| Flags | 12 |
| Block length | 16384 |
| File length | 0 |

**A file length of zero means there is nothing inside.** Each of the five is 272 bytes,
being the header and sixteen bytes more, and no block of content at all. What a package
unlocks is not inside them. They are the unlocking, and a program reads one to find out
whether this console may have the content.

That is also why no block is decrypted here and why Sony's own unpacker is absent. The
flag that states a packed block is the bit 1, no file of these packages states it, and
what was never measured is not built.

Each of the five was measured against the license of its own package and against another.
The license of one package opens none of another package's, which was run on purpose and
responded with

```
[SONY] the hash of this protected file is not the one this klicensee yields, so this klicensee does not open it.
```

Five out of five open and the wrong license opens none. That is what shows the license
this repository reads, the klicensee it computes and the hash it measures are each
right, because none of the three can be wrong on its own and still come out here.

```
xbreak edata verify --path <the edat file> --license <the rap file>
```
