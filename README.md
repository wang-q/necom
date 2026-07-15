# necom - Clustering, Matrix, and Phylogeny Toolkit

[![Build](https://github.com/wang-q/necom/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/necom/actions)
[![codecov](https://codecov.io/gh/wang-q/necom/branch/master/graph/badge.svg)](https://codecov.io/gh/wang-q/necom)
[![license](https://img.shields.io/github/license/wang-q/necom)](https://github.com//wang-q/necom)

`necom` is a command-line toolkit for clustering, distance-matrix processing,
phylogenetic-tree manipulation, and related workflows.

It is designed as a practical companion for day-to-day phylogenetics and
clustering tasks, with a focus on:

- Clustering algorithms and evaluation (`necom clust`)
- Distance-matrix utilities (`necom mat`)
- Newick-tree operations (`necom nwk`)
- Pipeline-friendly behavior (stdin/stdout where possible, predictable output,
  composable subcommands)
- Performance and robustness (Rust implementation, zero-panic policy for
  malformed inputs)

High-level capabilities include:

- Clustering & trees: distance/matrix processing, multiple clustering
  algorithms, tree cutting, tree comparison, rerooting, pruning, and
  visualization
- Pipelines: integrated workflows such as taxonomic tree condensation
  (`necom pl condense`)

## Install

Current release: 0.2.0

```bash
cargo install --path . --force #--offline

# test
cargo test -- --test-threads=1
```

## Usage

After installation, the `necom` binary should be available in your `PATH`:

```bash
necom --help
necom clust --help
necom mat --help
necom nwk --help
```

## Examples

Below are a few quick examples to get started:

```bash
# Hierarchical clustering from a PHYLIP distance matrix
necom clust hier tests/mat/IBPA.phy

# Compare two distance matrices
necom mat compare tests/mat/IBPA.phy tests/mat/IBPA.71.phy

# Tree statistics
necom nwk stat tests/newick/catarrhini.nwk

# Condense a tree by taxonomy
necom pl condense --taxon tests/pipeline/strains.taxon.tsv \
    tests/pipeline/minhash.reroot.newick
```

Extended documentation for each command is available in `docs/`.

## Author

Qiang Wang <wang-q@outlook.com>

## License

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2024-
