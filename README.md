# nn – *Normal Notes* 

Created as a means to jot down daily TODO's with a simple file system that's out of the way, and easy to search.

---

## Features

- Opens or creates a note for **today’s date** by default
- Search through all your notes
- Delete a note by date
- List all your notes
- Extract and list all **#tags** you've used
- Uses your favorite `$EDITOR` (defaults to `nano`)
- Configurable storage path via `~/.notes_cli/config.toml`
- Notes live in `~/.notes_cli/notes` by default, but configure wherever

---

## Installation

```bash
git clone https://github.com/euprhasiologist/nn
cd nn
cargo install --path .
# I'll get to cargo dist in a minute
```

## Usage

```
# create or edit an already created note for today's date
nn

# list all notes
nn list
```
All the info below.

```
A normal notes tool

Usage: nn [COMMAND]

Commands:
  delete  Delete a note
  list    List all notes
  search  Search all notes for a string
  tags    Show all tags used in notes
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
