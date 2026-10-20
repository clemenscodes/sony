# sony

What one platform holder decided, as opposed to what is universally true.

It is not a format and it depends on no port of `xbreak`, so it could be lifted into
another project unchanged. It knows nothing about a disc and nothing about a Ghidra
project.

It is these things. The first is the identity of every function a module exports.
A module table in an executable names that function by a number rather than by its
name, and this crate turns the one into the other.

The second is the signed executable. What the disc writes is not a program, it is a
container with the program inside it encrypted in pieces, and this crate takes it apart
into the executable a disassembler can read. That is set out in
`docs/DECRYPTION.md`.

The third is the archive. Almost everything else on the disc is inside one, packed
and listed in a table of contents, and this crate reads the files out of one and
writes them back into one. That is set out in `docs/ARCHIVE.md`.

The fourth is the package. What a shop sells is one file with everything of a purchase
inside it, encrypted under one key every such file shares, and this crate opens one and
puts what is in it under a tree. That is set out in `docs/PACKAGE.md`.

The fifth is the license. A purchase comes with sixteen bytes, and those sixteen yield
the klicensee that opens what the purchase brought. That is set out in
`docs/LICENSE.md`.

The sixth is the protected file. What a package brings of real value is opened by a
klicensee and by nothing else, and this crate measures the one against the other. That
is set out in `docs/EDATA.md`.

Only what can be derived enters. A number that follows from a name is a fact
anybody can recompute, and that is why the table in here needs no source outside
itself. A number that was merely assigned, such as a syscall number, follows from
nothing and would only be a claim somebody wrote down elsewhere, so it stays out.

```
sony name hash cellHttpUtilParseUri
sony identifier resolve 32FAAF58
```

It is a library and a command. The library publishes the concepts `archive`,
`content_identifier`, `dictionary`, `edata`, `executable`, `identifier`, `license`,
`name` and `package`, and every one of them publishes its own `Error`. The command is
one person asking one question. The adapters `sony_archive`, `sony_executable`,
`sony_license` and `sony_package` take the library and translate what it reads into the
concepts of `xbreak`.
