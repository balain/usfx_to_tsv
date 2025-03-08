# usfx_to_tsv

Convert [USFX](https://ebible.org/usfx/) formatted files to TSV

## Why?

To prepare the files for importing into a database more easily.

## Dependencies
- [quick-xml](https://crates.io/crates/quick-xml)

## Setup
1. Install quick_xml crate (`cargo add quick-xml`)
1. Copy the source XML file to `./xml/source.xml`
1. `cargo run main.rs > output.tsv`
1. Import `output.tsv` into your database

## Output Format

- One verse per line
- Tab delimited
- Fields:
  - Book (abbreviated)
  - Chapter (number)
  - Verse (number)
  - Text (string)

## Future
- [ ] Add comments
- [ ] Implement command line arguments
- [ ] Rename the main file

## Resources
- [USFX home page](https://ebible.org/usfx/)
