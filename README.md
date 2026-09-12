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

Pass `--sort` to get the output alphabetically (case-insensitive) instead:

```
./target/release/name-formatter --sort names.txt
```

Pass `--sample N` to draw N random names instead of printing the whole
cleaned list. Sampling is weighted by how often each name occurred in the
source list before deduplication, so a name that showed up ten times in a
scraped list is roughly ten times as likely to be picked as one that showed
up once:

```
./target/release/name-formatter --sample 5 names.txt
```

Sampling is with replacement, so the same name can come up more than once in
a single run, and each run draws a different random sequence.

## Known limitations

Title-casing capitalizes after spaces, hyphens, and apostrophes, and knows
a curated list of common Mc/Mac surnames ("mcdonald" becomes "McDonald",
"mackenzie" becomes "MacKenzie"). Anything not on that list falls back to
plain title-casing, so an uncommon Mc/Mac surname will come out as e.g.
"Mcintire" instead of "McIntire" until it's added to the list.

## License

MIT, see LICENSE.
