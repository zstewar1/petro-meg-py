# Petroglyph Meg file library

MEGA files are a format used by Petroglyph for various games including Star Wars: Empire
at War, Universe at War: Earth Assault, Guardians of Graxia, Rise of Immortals, Grey Goo,
and Great War: Western Front.

This repo provides a Python Library for working with them. It is a wrapper around a Rust
library that does the same thing ([crate][pm-cio], [repo][cm-repo]).

[pm-cio]: https://crates.io/crates/petro-meg
[cm-repo]: https://github.com/zstewar1/petro-meg

The parsing and encoding is based on the format descriptions provided
[here](https://modtools.petrolution.net/docs/MegFileFormat) by some of the Modders who
first enabled Empire at War modding.

## Why?

Q: *You're like 20 years late to this party. Why make this now?*

Mostly for fun. Also because the [MEGA file
editor](https://github.com/GlyphXTools/meg-editor/) produced by Mike Lankamp is a C# GUI
intended for Windows and I'm a Linux dev who likes CLI tools.

There are some tweaks I like to apply to *all* projectiles in Empire at War (e.g. making
*all* lasers faster), and I tend to write scripts to do that rather than hand editing the
XML config files. By making this a library, I can write a script that directly extracts
and edits files from the game's MEGA files without needing to provide it a pre-extracted
bundle of XML "source" files to work with.

## How do I use it?

For reading MEGA files, there is a single function, `read_meg(mega_file, /, version=None,
key=None, iv=None)`. This takes a single file-like input (must be in binary mode) and
parses the headers. It has optional arguments for `version` (1, 2, or 3), and the key and
initial vector used for reading encrypted MEGA files. If using encryption Key and IV must
both be bytes and must both have len 16. `read_meg` returns a list of `FileEntry` objects.

`FileEntry` has two properties, `name` and `size` which tell the file name and number of
bytes in the file. It provides a `read` method which attempts to extract the contents of
the file as a `bytes` object. `read` only works if the MEGA file is still open. It will
seek to the start of the file contents and read a length matching the file size. If you
prefer to get a range over the file and extract it yourself, you can instead access the
`start` and `end` properties.

Entry names use the 'MegPath' type, which is a case-insensitive path name. We restrict
MEGA paths to ASCII-7, so it can always be converted to `str`. It also implements equality
and order comparisons with `str`, though be aware that e.g. sorting a list with a mix of
`str` and `MegPath` can be problematic since `MegPath` will be case-insensitive when
comparing to itself or str, but str will be case-sensitive. Don't mix them; the
comparisons with `str` are provided only for convenience. `MegPath` can be constructed
from a `str`. Some validation is applied, like no charcters not valid in Windows file
names are allowed, and paths cannot be rooted or have empty segments.

For building, there is a dict-like `MegBuilder`. The constructor takes a version argument,
which must be 1, 2, or 3, and an optional pair of key and initial vector for encryption if
you want to enable encryption for V3. The constructor also takes an `entries` argument to
initialize the builder from a dictionary. It always uses MegPath for the keys. You can
insert any values that are either `bytes`/`bytearray` or are readable and seekable files.
It can then be updated like a dict. Any `str` key passed will automatically be converted
to `MegPath` for you. Once you have all the files added, you can call `build` to convert
to a MEGA file. `build` takes one optional argument, an open binary file to write to. If
you don't provide a file, `build` will instead build the file as `bytes` and return that.
