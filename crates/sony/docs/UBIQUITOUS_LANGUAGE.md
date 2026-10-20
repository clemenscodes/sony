# The ubiquitous language

| Word | What it means |
|---|---|
| Act | To do what an action says. |
| Action | One thing the command line does with a concept. |
| Alignment | A count of bytes that an offset is a multiple of. |
| And | What joins the key and the vector the metadata of a signed executable keeps together. |
| Archive | One file of Sony's that is many files in one. Nothing in it is encrypted, it is packed, and what a reader gets out of it are the files that went in. |
| Archive Flags | The four bytes at the end of an archive header. Every archive read so far writes zero and this repository knows no other value. |
| Archive Root | Where an archive begins. Every file path is written from there and none of them from the root of a disc. |
| Archive Version | Which version of the archive format an archive was written to, being a major version and a minor version. |
| Backslash | The \ character |
| Blob | Bytes as many as the file happens to be. An archive, a file inside it, a block and a signed executable are each a blob, because none of them is a count the format settles in advance. What is read off a disc is a blob until a field inside it states otherwise. |
| Block | One piece of an archive, packed on its own. Unpacked it is sixty five thousand five hundred and thirty six bytes, and the last one of a file is whatever is left. |
| Byte | One of the bytes something is made of. |
| Ceiling | The smallest value a given field length cannot write, being two hundred fifty six raised to that many bytes. A header grows its length field until the block length is below it. |
| Checksum | The four bytes a name yields, being the first four bytes of what SHA1 yields over the name and the name suffix, written back to front. A checksum is a kind of thing and never a doing, and what its bytes are for is comparing and not counting, so the one question they answer is whether two names are one name. An identifier is made of one and is not one. |
| Chunk | As much of a file as goes into one block before it is packed. |
| Console | The machine Sony built and a program of the PlayStation 3 runs on, as opposed to the handhelds that read a package of their own kind. |
| Content Identifier | What a package and a license both name, being a publisher code, the title identifier and the name of the content. The file name of a package is scrambled and says nothing about what is in it, so this is the only thing that pairs the two files. |
| Control Information | The part of a signed executable that is not the executable itself, among it the license. |
| Count | How many of something there are. |
| Cursor | An archive, its manifest and which file of it is next in turn, so that the files come out one after the other in the order the manifest writes them. |
| Decrypt | To take a signed executable apart into the executable inside it. |
| Dictionary | Every name this repository knows, kept under the number it yields. It is made of the names beside it and no two names in it yield one number. |
| Difference | The byte that is left once one byte is taken from another, wrapping below zero, as a klicensee is worked out. |
| Digest | The sixteen bytes MD5 yields over a file path, which is what a record writes in place of the path itself. |
| Disc | The disc a program was pressed on, as opposed to what the store serves. |
| Done | The outcome of an action that was done. |
| Edata | What Sony calls a file that only a license opens. It begins with the letters NPD and a zero, it names the content it belongs to, and a hash in its header says whether a klicensee is the right one for it. |
| Elf | The layout an executable is written in. Its own header says where the programs and the sections of the executable begin. |
| Entry | One name together with the number it yields. |
| Executable | The program the console runs, as it is once the signed executable was taken apart. |
| Extended Header | Where the header of a signed executable goes on beyond what every signed executable has, and names the program type. |
| Field | One part a record or a signed header is made of, written with the high byte first and as many bytes wide as that part is. |
| Field Length | How many bytes one field is written with. A block length is written with two bytes and a length in a record with five. |
| File | One file inside an archive, being its path and the bytes that belong to it. |
| File Path | The whole way from the root of an archive to one file. It is not a file name, because the name of a file is neither the way in front of it nor the ending behind it, and an archive has no directories to walk, so the way is written out in full. |
| First Block | Which block of the archive a file begins at. |
| Fit | A keyset fits a signed executable when it was made for that revision and that program type. |
| Flag | One bit of a field whose bits each mean a thing of their own, so such a field is read bit by bit and never as one number. |
| Free License | A license anybody may use, whose key is the same on every console. |
| Has | The start of a question a block or a section responds to with yes or no, such as whether it is packed or whether a key belongs to it. |
| Hash | To compute the identifier a name yields, which needs no dictionary. |
| Header | The bytes at the front of something that say how the rest of it is laid out. The signed executable has one, the metadata has one, and the executable has one. |
| Hexadecimal | Written to base sixteen, in upper case. A writing in lower case is no identifier, which two tests pin on the way in and on the way back. |
| Identifier | The number a module table writes in place of a name. It comes from the name alone, it is four bytes long and it is written with eight hexadecimal digits in upper case. |
| Index | Which one of many, counted from zero. A record, a block and a file of an archive are each reached by one. |
| Installation Directory | Where the console puts what a package brought, being one directory the metadata of that package names. |
| Integer | That many bytes read as one whole value without a sign, the high byte first. Sony writes them two, four, five and eight bytes long, and the five byte one is why an offset into an archive of ten gigabytes fits. |
| Is | The start of a question a value responds to with yes or no, such as whether a digest is nameless or a section is a program. |
| Key | The bytes a cipher is keyed with. |
| Key Revision | Which generation of keys a signed executable was made with. |
| Keyset | One key and one vector, together with the revision and the program type they were made for. |
| Klicensee | The bytes that unlock the license of a signed executable. |
| Last | The end of a range, such as the last block of a record, the last letter a content identifier may use or the last path of a license. |
| Length | How many bytes something is. A header is thirty two, a section is forty eight, an identifier is four. |
| Letter | One character of a text. An identifier is written with eight of them. |
| License | What a signed executable states about who may run it. Without it the metadata stays encrypted. |
| Line | One line of a text. The names are written one to a line, and the command line is what the process was started with. |
| Magic | The bytes a file begins with, by which it says what it is. A signed executable begins with SCE and nothing else does. |
| Major Version | The first half of a version, which is one for every archive read so far. |
| Manifest | The first record of an archive, which is no file of the archive's contents. It is the path of every other file, one to a line, in the order of the records behind it. |
| Mask | Bytes that are exclusive ored into a value of the same length. Applying a mask twice yields what there was before it. |
| Metadata | What a signed executable says about its own sections and about the keys they need. It is encrypted itself and is the first thing that has to be decrypted. |
| Metadata Information | The bytes in front of the metadata, which give the key and the vector the metadata itself is encrypted with. |
| Minor Version | The second half of a version. The archives of the disc state three and the one the store serves states four. |
| Mix | To exclusive or every byte of a value into the byte worked before it in the order, so that a change in one byte reaches every byte behind it. |
| Name | The name a function was given in the module that exports it. It is written with letters, digits and the underscore, and the case it was written in is part of it. |
| Name Suffix | The sixteen bytes Sony puts behind a name before the digest is computed. Without them a name yields another number. |
| Nameless | A text with no name in it at all, which is no dictionary. |
| Npdrm | What Sony calls a signed executable with a license in it. |
| Offset | How far from the front of a file something begins. |
| Order | The order in which the sixteen bytes of a license are worked, one place for each turn, so that no place is worked twice and none is left out. |
| Outcome | What a run of the command line ends with, and the status the process exits with. |
| Package | One file of Sony's that a shop sells and a console installs. It is a header, a table of what is in it and the files themselves, and everything from the data offset onward is encrypted. An archive is opened to read what is in it and a package is opened to put what is in it somewhere, which is why the two are not one thing. |
| Packer | What an archive names as having packed its blocks, written as four letters. Every archive of this disc names zlib. |
| Packing | What an archive states its blocks were packed with, being four letters. Every archive measured here states zlib. |
| Packing Effort | How hard the packer tries. It changes how long an archive is and never what comes out of it. |
| Padding | The bytes that fill a place nothing was written into. In the information of a signed executable they are all zero, which is how a key that does not fit is found. |
| Path Separator | The one letter between two steps of a file path, which is the forward slash and never the one the other platform writes. |
| Platform | Which console a package was made for. The PlayStation 3 states one and the handhelds state two, and the two are not opened the same way. Every package of this repository states the PlayStation 3. |
| Program | One piece of the executable that the console loads into memory. |
| Program Type | What kind of thing was signed, which the extended header names. An NPDRM one is eight and an application is four. |
| Read | Of the entries of a dictionary, those taken from the file so far. |
| Reality | The count of bytes there are, as an error of a length states it. |
| Record | What the table of contents of an archive says about one file, being the digest of its path, which block it begins at, how long it is unpacked and where its first block begins. |
| Release | What a package states about where it came from, being either one a shop sold or one that never left the studio. Which of the two it is decides how the data of the package is encrypted. Every package of this repository states the one a shop sold. |
| Remnant | Every byte of an archive behind its last block. No record and no block of the archive reaches it, and it is part of the archive all the same, because an archive that was read must be written back byte for byte. Two of the twelve archives of this disc have one and the other ten end where their last block ends. |
| Report | What an action yields when it was done. |
| Request | What a person wrote on the command line for one action to act on, being the concept that action is under. |
| Resolve | To find the name an identifier means, which needs the dictionary. |
| Revision | Which firmware a signed executable was made for, two bytes in its header, and what decides which key of the vault fits it. |
| Round | One pass of a transformation that is done again and again over the same value. A license yields its klicensee after five. |
| Section | One piece of the signed executable that the metadata describes, saying where it begins, how long it is, which key decrypts it and whether it was compressed. |
| Section Kind | What a section says it is. A section of a program says so here. |
| Signed Executable | A file as Sony ships it. It begins with SCE, an executable that was encrypted is inside it, and nothing in it runs until it is decrypted. |
| Sony | The company that made the PlayStation 3 and decided how a module names what it exports. What Sony decided is true for every program on that console and for no program on another. |
| Store | Where a program fetches what it was not pressed with. |
| Subtract | To take a table of bytes out of a value of the same length, one byte at a time, with a borrow that runs into the byte worked next. |
| Subtrahend | The bytes that are subtracted from a value of the same length, one byte at a time, with a borrow that runs into the byte worked next. |
| Table of Contents | The header, the records and the block lengths together, being everything of an archive in front of its first block. |
| Text | The characters something was written with, kept exactly as written. |
| Title Identifier | The title a package belongs to, being four letters and five digits. |
| Unknown | An identifier that no name in the dictionary yields. |
| Unpack | To take a block back out of its packing, or a whole file out of the blocks it was cut into. |
| Uppercase | Of an identifier, that its letters are upper case, as it has to be spelled. |
| Vault | Every keyset this repository knows. A signed executable is decrypted with the one keyset that fits it and with no other. |
| Vector | The bytes a cipher starts from, beside the key. |
| Version | Which writing of a format a file is, being a major and a minor part of two bytes each in an archive header. |
| Word | Four bytes, which is how an archive header writes every part of itself. What those four bytes mean is a second question, so a word says nothing about it. |
| Write | To put the sections of a signed executable into the executable they decrypt to. |
| Zero | That every byte is zero. A digest of nothing but zero bytes is what an archive writes for the record that has no name, which is the manifest itself, so asking a digest whether it is zero is how the nameless record is found. |
