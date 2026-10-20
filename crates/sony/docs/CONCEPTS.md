# What it is made of

The crate is laid out by layer. Every module is
private, and a layer publishes the types that cross out of it in its own `mod.rs`,
which `docs/RUST_STYLE.md` sets out in full.

```
core                  the domain
  name                the text a function was given
  identifier          the number a name yields, and what a module table writes
  dictionary          every name that is known, kept under the number it yields
    entry             one name and the identifier that follows from it
  content_identifier  the forty eight letters that name what a package brought
  executable          the program the console runs
    signed            the trait a file as Sony ships it is read through
      executable      the container on the disc, and how it comes apart
        header        where every part of the container begins
        license       who may run it, and the key that follows from that
        metadata      the sections, and the keys they need
          section     one piece of the container, and what has to be done to it
        vault         every keyset this repository knows
          keyset      one key and one vector, and what they fit
  archive             one file of Sony's that is made of many
    header            how the rest of the archive is laid out
      version         which version of the format it follows
      packer          what its blocks were packed with
    record            what the table of contents states about one file
      digest          the sixteen bytes MD5 yields over a file path
    block             one piece of the archive, packed on its own
    manifest          the path of every file, one to a line
    file_path         the way from the root of the archive to one file
    files             every file of an archive, which is what comes out and goes in
      file            one path and the bytes that belong to it
    cursor            an archive read file by file
  license             the sixteen bytes a purchase comes with
    klicensee         the sixteen bytes those yield
  package             one file a shop sells, with everything of a purchase inside it
    header            how the rest of the package is laid out
      platform        which console it was made for
      release         which release it was made for
    metadata          the entries that follow the header
      entry           one entry, its length and the bytes that belong to it
    record            what the table of contents states about one file
      kind            whether a record is a directory or a file
    file              one path and the bytes that belong to it
    file_path         the way from the root of the package to one file
    cursor            a package read file by file
  edata               the protected file a package brings
    header            the content it names, and the lengths it states

application           what the presentation drives
  command_line        the concepts clap knows, and the streams it writes to
    action            what an action is, and every action there is
      name
        hash          the number a name yields
          report      the report it responds with
      identifier
        resolve       the name a number means
          report      the report it responds with
    service           sony, which owns the dictionary and responds from it
    outcome           what the process exits with
```

There is no `infra` and no port. Nothing here is read from the world, because the
names are compiled in, which is the same reason `src/contexts` in
`crates/vocabulary` has no adapter either.

`core` publishes these concepts.

```text
sony::archive::Archive
sony::archive::Cursor
sony::archive::FilePath
sony::archive::files::Files
sony::archive::files::File
sony::content_identifier::ContentIdentifier
sony::dictionary::Dictionary
sony::edata::Edata
sony::executable::Executable
sony::executable::signed::Signed
sony::executable::signed::SignedExecutable
sony::identifier::Identifier
sony::license::License
sony::license::Klicensee
sony::name::Name
sony::package::Package
sony::package::Cursor
sony::package::File
sony::package::FilePath
```

The concepts leave as modules rather than as a flat list of their types, so a caller
that writes `archive::files::File` shows which concept it came out of without opening
anything. `docs/RUST_STYLE.md` sets out the rule under `A layer publishes concepts,
not a heap of types`, and this crate is the worked example there, because it did it
the other way first. Every concept publishes the `Error` it fails with, and there is
no bare error at the edge of the crate.

`files` is a module of its own and `file_path` is not, because `File`
lives two below `archive` and a module publishes no further than its own child, while
`FilePath` lives one below and a module opened around one type of
its own name adds nothing.

`Signed` is a trait and `SignedExecutable` is the file it is implemented for. The
trait states what can be done with a file as Sony ships it and the struct is the file,
so a caller that never decrypts never names the trait at all.

`Entry` stays inside `dictionary`, because a caller that already has a name and a
number has no use for the pair. Everything under `SignedExecutable` stays there too,
because a caller puts in a file and takes out an executable and never sees a keyset or
a section.

Of the archive a caller sees the forms and what one of them is made of, being an
`Archive` and the `Files` it is made of, and it sees `Cursor` because an archive of five
gigabytes is read one file at a time and never all at once. `Header`, `Record`,
`Digest`, `Block` and `Manifest` stay inside, because they are how an archive is laid
out and not what a caller wants out of it.

## Where the targets divide

`src/lib.rs` declares `core` and publishes those concepts again,
and nowhere else in this repository does a name appear twice.
`docs/RUST_STYLE.md` sets out the rule and the reason, being that `lib.rs` is the edge
of the crate rather than a module inside it. `src/main.rs` declares `application`
alone and reaches the domain through `sony::`, so the command a person types crosses
the same edge every other consumer crosses.

Because those concepts leave the crate, `core` opens them with `pub` rather than
`pub(crate)`, and what they publish is `pub` for the same reason.

The library declares `doctest = false`. A crate of this repository has no
comments, which the `comment` gate rejects, so it has no example inside one
either. Running rustdoc over a library that can have no doc test is work that can
only fail, and it did, because the split standard library this devshell links needs
a sysroot that rustdoc was never given.

A `Dictionary` writes itself as one number and one name to a line.

## Why a name and a number are kept beside the dictionary and not inside it

`sony name hash` needs no dictionary at all. A number follows from a name by a hash
and by nothing else, so neither concept is a part of the dictionary. The dictionary
is a collection of pairs of them, which is why it is the one that names the other
two and not the other way round.

Where a concept really is only ever a part of another, it lives inside it, and
`entry` is that case here.

## What follows from the rules

A `Dictionary` cannot be made empty. There is no `Default`, it is made from a text
of names, and a text that yields none of them fails, so a dictionary that
answers nothing is not a state this type can be in. `TryFrom<()>` is that same
constructor over the names compiled in beside it, which is what lets the empty case
be proven by a test at all.

An `Identifier` is derived from a `Name` and is nothing else, so a dictionary
entry has no identity beside the two values it is made of and is a value object rather
than an entity.

A number goes out and comes back. `Display` writes eight upper case
hexadecimal digits and `TryFrom<&str>` accepts exactly that and nothing
else, so what a report shows is what the next command line takes.
