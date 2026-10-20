# The decryption

What a signed executable is, and how the executable comes back out of it.

## What is on the disc

The disc writes `EBOOT.BIN`. It begins with the four bytes `SCE\0` and it is not a
program. It is a container, and the ELF the console runs is inside it, encrypted in
pieces and described by a metadata that is itself encrypted.

The container has four parts.

| Part | What is there |
|---|---|
| Header | Where every other part begins, which key revision was used and which program type this is. |
| Control | What the file states about who may run it, among it the license. |
| Metadata | The sections, and the keys and vectors those sections need. |
| Sections | The pieces of the executable, each one encrypted and some of them compressed. |

The header is read at fixed offsets and nothing about it is guessed.

| Offset | What is there |
|---|---|
| 0x00 | The magic, four bytes, `SCE\0`. |
| 0x08 | The key revision. |
| 0x0C | Where the metadata begins. |
| 0x10 | How long the header is, being where the metadata ends. |
| 0x18 | How long the executable will be once it is put back together. |
| 0x30 | Where the ELF header of the executable begins inside the container. |
| 0x38 | Where the program headers begin. |
| 0x40 | Where the section headers begin. |
| 0x58 | Where the control begins. |
| 0x60 | How long the control is. |
| 0x7C | The program type. |

## The way in

The parts are peeled off in this order, and each one needs the one before it.

1. The header is read. A file that does not begin with `SCE\0` fails as
   unsigned, and a file that ends before an offset it names fails as truncated.

2. The license is read out of the control. The control is a list of entries, each
   one four bytes of kind and four bytes of length. The entry of kind three is the
   one the license is in, it begins with `NPD\0`, and eight bytes further in
   comes the license type. Where that type is three the license is free. Where the
   control writes no such entry the file has no license at all, which is the
   ordinary case for a program that is not an NPDRM one.

3. The keyset is looked up in the vault by the key revision and the program type.
   Where the vault has none for that pair the file fails as unkeyed, and
   nothing is attempted with a key that does not fit.

4. The information is decrypted. It is the sixty four bytes behind the header of the
   metadata. Where the license is free it is first taken out from under AES-128-CBC
   with the klicensee, which is itself the sixteen published bytes taken out from
   under AES-128-ECB with the sixteen bytes Sony keys them with. Then it comes out
   from under AES-256-CBC with the key and the vector of the keyset. What comes out
   is the key at its first sixteen bytes and the vector at its bytes thirty
   two to forty eight.

5. The metadata is decrypted with that key and that vector under AES-128-CTR. It
   then names how many sections there are, how many keys there are, where each
   section begins, how long it is, which of those keys it needs and whether it was
   compressed.

## How the executable is put back together

A buffer of the length the header names is made, and these things go into it.

The ELF header, sixty four bytes, straight out of the container.

The program headers, each fifty six bytes, at the offset the ELF header names.

Every section that is of a program and is encrypted. Its bytes come out from under
AES-128-CTR with the key and the vector the metadata names for it, and where the
section states it was compressed they are then taken out of their zlib packing. They
go in at the offset the matching program header names, cut to the length that
header gives.

The section headers, straight out of the container, at the offset the ELF header
names for them.

## Where it fails

| Error | When |
|---|---|
| Unsigned | The file does not begin with `SCE\0`. |
| Truncated | The file ends before an offset or a length inside it. |
| Unkeyed | The vault has no keyset for that key revision and that program type. |
| Unlicensed | The license the control names is not the free one. |
| Undecrypted | The padding of the information did not come out zero, or a section names a key that is not in the metadata. |
| Unpacked | A compressed section did not come back out of its zlib packing. |

Undecrypted is what a wrong key looks like. Sixteen bytes at the middle of
the information and sixteen at its end are zero in every signed file, so a key that
does not fit is caught there and never later.

## Where the keys come from

The keysets are the ones RPCS3 keeps in `rpcs3/Crypto/key_vault.cpp`, read out of
that file and put into `core/executable/signed/executable/vault`. The ones this repository needs
are here.

| Program type | Key revision | What it decrypts |
|---|---|---|
| 8, NPDRM | 1 | A patched executable. |
| 4, application | 1 | An executable as the disc pressed it. |

A keyset is found by the key revision and the program type and by nothing else,
which is what RPCS3 does as well. The version a key vault writes beside a keyset is
not read for either of these two types, so it is not on this page.

The published bytes of the free license are the same two the emulator uses, being
the klicensee every free title is licensed with and the key that unwraps it.

## What was measured

Two executables, one of each program type above, were decrypted by this crate alone,
with no emulator in the way, and each one came out byte for byte the file RPCS3 had
decrypted beforehand. Both are key revision one.

```
xbreak executable decrypt --path EBOOT.BIN --cleartext EBOOT.elf
```
