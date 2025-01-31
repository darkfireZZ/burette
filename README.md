
# burette

`burette` is a very simple command line program to store & manage books (and
possibly other documents).

In addition, `burette` also stores the following metadata for each document:
 - Title
 - Authors
 - ISBNs
 - DOI (see <https://www.doi.org/> for more information)

In the future, `burette` may feature a search function to search the metadata
of the documents and maybe even a full-text search.
However, these features are not implemented as of now.

`burette` is written primarily for my personal use, but I'm making it public in
case someone else finds it useful.
It might lack features that you would expect from a program like this, but I'm
open to suggestions and contributions.

## Installation

To install `burette`, you need to have `cargo` installed.
If you don't have `cargo` installed, you can install it by following the
instructions at
<https://doc.rust-lang.org/cargo/getting-started/installation.html>.

After you have `cargo` installed, you can install `burette` by running the
following command:

```sh
cargo install --git https://github.com/darkfireZZ/burette --tag v0.1.0
```

## Usage

The following is an incomplete list of subcommands that `burette` supports.
There are many more useful subcommands that are not listed here.
You can see the full list of subcommands by running `burette --help`.

### Creating a new library

First, you need to create a new library
```sh
burette new
```
If you don't like the default location of the library (`~/.book-store/`), you
can specify a different location using the `--library` flag.
In this case, you will need to use the `--library` flag with every subcommand
that you run.

### Adding, listing and removing documents

To add a document to the library, you can use the `add` subcommand.
```sh
burette add <path-to-document>
```
You will then be prompted to enter the metadata of the document.

`burette list` lists all the documents in the library along with their SHA-256
hashes.

And finally, removing a document from the library is as simple as running
```sh
burette remove <sha256-hash-of-document>
```

### Retrieving documents

To retrieve a document from the library, you can use the `get` subcommand.
```sh
burette get <sha256-hash-of-document>
```

This will place the document in the current directory with some default name.
If you want to specify a different name or location, you can use the `--output`
flag.

## Inner Workings

All the files used by `burette` are stored in a single directory called the
"library".
By default, the library is located at `~/.book-store/`.
The library is of the following structure:

```
.book-store/
    burette_version
    index.json
    documents/
        <document1>
        <document2>
        <document3>
        ...
```

- `burette_version` contains the version of `burette` that created the library.
- `index.json` contains the metadata of all the documents in the library.
- `documents/` is the directory where the actual documents are stored.
  The documents are named after their SHA-256 hash.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md)

## License

```plaintext
This is free and unencumbered software released into the public domain.

Anyone is free to copy, modify, publish, use, compile, sell, or
distribute this software, either in source code form or as a compiled
binary, for any purpose, commercial or non-commercial, and by any
means.

In jurisdictions that recognize copyright laws, the author or authors
of this software dedicate any and all copyright interest in the
software to the public domain. We make this dedication for the benefit
of the public at large and to the detriment of our heirs and
successors. We intend this dedication to be an overt act of
relinquishment in perpetuity of all present and future rights to this
software under copyright law.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY CLAIM, DAMAGES OR
OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE,
ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.

For more information, please refer to <https://unlicense.org/>
```
