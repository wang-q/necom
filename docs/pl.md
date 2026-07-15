# necom pl

The `necom pl` module provides integrated pipelines that combine multiple `necom` operations into a single command.

Currently, only one pipeline is implemented:

*   `condense`: Condense monophyletic subtrees based on a taxonomy TSV file.

## condense

Condenses subtrees based on taxonomy.

```bash
necom pl condense [OPTIONS] --taxon <taxon.tsv> <infile>
```

### Arguments

*   `<infile>`: Input Newick filename. Use `stdin` for standard input.

### Options

*   `--taxon <taxon.tsv>`: Path to the taxonomy TSV file (required).
*   `--rank <N>`: Column index(es) to use for grouping (1-based). Can be specified multiple times. Default: `2`.
*   `--map`: Write a mapping file `condensed.tsv` showing original node names and their condensed labels.
*   `-o, --outfile <outfile>`: Output filename. Use `stdout` for screen output.

### Input Formats

*   **Newick tree** (`<infile>`): A tree whose leaf labels match the first column of the taxonomy file.
*   **Taxonomy TSV** (`--taxon`): A tab-separated file without header, containing at least two columns:
    *   Column 1: node name (must match leaf labels in the Newick file).
    *   Column 2+: taxonomic terms (e.g., species, genus, family).

Monophyletic subtrees whose leaves share the same taxonomic term at the selected rank are collapsed into a single node named `{term}||{count}`, where `{count}` is the number of leaves in the condensed group. Condensed nodes carry `member=<count>` and `tri=white` comments for visualization.

### Examples

```bash
# Condense by species (2nd column)
necom pl condense --taxon taxon.tsv tree.nwk

# Condense by genus (3rd column)
necom pl condense --taxon taxon.tsv --rank 3 tree.nwk

# Condense by multiple ranks
necom pl condense --taxon taxon.tsv --rank 2 --rank 3 tree.nwk

# Output mapping file
necom pl condense --taxon taxon.tsv --map tree.nwk -o condensed.nwk
```

### Full Example with Test Data

```bash
necom pl condense --taxon tests/pipeline/strains.taxon.tsv \
    tests/pipeline/minhash.reroot.newick
```
