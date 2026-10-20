# The license

What a purchase comes with, and the sixteen bytes it yields.

## What a purchase comes with

A license is one file with the ending `rap` and it is exactly sixteen bytes. A file of
any other length is no license, and it yields an error.

Its name is the content identifier of the package it belongs to. The package file name
is a scramble and means nothing, so the license is what names the pair.

## What comes out of it

A klicensee, being sixteen bytes. It comes out of the license alone and out of nothing
else, so the same license always yields the same klicensee.

Three steps, and none of them is a choice.

The sixteen bytes are decrypted once under AES-128 with a key that is the same for every
license. RPCS3 does that as a cipher block chaining with a vector of zeroes, which over a
single block is the same thing as the electronic code book this crate uses.

Then five rounds, and each round is three passes over the sixteen bytes. The first
exclusive ors a mask into every byte. The second, going backwards, exclusive ors into
each byte the one before it in the order. The third subtracts a second table from every
byte, with a borrow that runs from one byte into the next.

Every one of the three passes goes in an order that a table of sixteen positions names,
and that order is a permutation, so no position is worked twice and none is left out.

The key and the three tables are the ones RPCS3 keeps in `rpcs3/Crypto/key_vault.h` as
`RAP_KEY`, `RAP_PBOX`, `RAP_E1` and `RAP_E2`, and the rounds are `rap_to_rif` in
`rpcs3/Crypto/key_vault.cpp`.

## What a klicensee is for

It opens a protected file, and that is the only use this repository has for one. What
that means is set out in `docs/EDATA.md`.

## Where it fails

| Error | When |
|---|---|
| Length | The file is not sixteen bytes |

There is nothing else that can fail. Every sixteen bytes yield a klicensee, and whether that
klicensee is the right one is a question only a protected file can answer.

## What was measured

The three licenses of this repository each yield a klicensee, and each of those
klicensees opens every protected file of its own pack while opening none of another
pack's. That is in `docs/EDATA.md` with the numbers.

The klicensees themselves are absent from this repository on purpose. They are what a
purchase unlocks, they are derivable from the licenses in a moment, and a page that
repeats them buys nothing.

```
xbreak license resolve --path <the rap file>
```
