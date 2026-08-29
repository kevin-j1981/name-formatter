# name-formatter

Cleans up messy name lists so they're usable as input for a random name
generator.

## The problem

If you've ever built a random name generator, the generator itself is the
easy part. The hard part is the source list: a scraped surname list from a
census export, a pasted spreadsheet column of first names, a fantasy name
list copied out of a forum post. These lists are never clean. You get:

- inconsistent case (`MARY`, `mary`, `Mary`)
- stray whitespace and tabs from a pasted spreadsheet
- a mix of delimiters in the same file (commas here, newlines there)
- exact duplicates that differ only in case
- junk entries that aren't names at all (`007`, empty lines, stray quotes)

Feeding that straight into a name generator means the generator's output is
only as clean as its worst input line. `name-formatter` is a small tool that
sits between "list I found" and "list I can actually sample from."

## Usage

Build it with cargo (standard library only, nothing to fetch):

```
cargo build --release
```

Run it against a file:

```
./target/release/name-formatter names.txt
```

Or pipe messy input through stdin:

```
$ printf "  mary jane , MARY JANE\nBob O'brien\n007\n" | ./target/release/name-formatter
Mary Jane
Bob O'Brien
```

Notice what happened: the duplicate (differing only in case) was dropped,
whitespace was collapsed, everything was title-cased, the apostrophe name
was capitalized correctly, and the numeric junk entry was filtered out
entirely.

The output is one cleaned, deduplicated name per line, in the order each
name first appeared, ready to be sampled from by whatever generator you're
building on top.

## Known limitations

Title-casing is a simple algorithm (capitalize after spaces, hyphens, and
apostrophes) with no dictionary of exceptions, so it doesn't know that
"mcdonald" should become "McDonald" rather than "Mcdonald". See the roadmap
for planned fixes.

## License

MIT, see LICENSE.
