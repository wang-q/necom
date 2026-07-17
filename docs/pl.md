# necom pl

The `necom pl` module provides integrated pipelines that combine multiple `necom` operations into a single command.

Currently, only one pipeline is implemented:

*   `condense`: Condense monophyletic subtrees based on a taxonomy TSV file.

For command-line options and usage examples, see [`docs/help/pl/condense.md`](help/pl/condense.md).

## Full Example with Test Data

```bash
necom pl condense --taxon tests/pipeline/strains.taxon.tsv \
    tests/pipeline/minhash.reroot.newick
```
