# necom — NWK, CLUST, and MAT Toolkit

[![Build](https://github.com/wang-q/necom/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/necom/actions)
[![codecov](https://codecov.io/gh/wang-q/necom/branch/master/graph/badge.svg)](https://codecov.io/gh/wang-q/necom)
[![license](https://img.shields.io/github/license/wang-q/necom)](https://github.com/wang-q/necom)
[![Documentation](https://img.shields.io/badge/docs-online-blue)](https://wang-q.github.io/necom/)

`necom` is a command-line toolkit for **clustering**, **distance-matrix processing**, and **phylogenetic-tree
manipulation**.

The name is formed from its three command families — **NWK**, **CLUST**, and **MAT** — with vowels inserted in
alphabetical order. It also echoes the Latin *nexum* (“tie” or “bond”), reflecting the toolkit’s focus on connections
between clusters, matrix entries, and tree nodes.

## Features

- **Clustering** (`necom clust`): hierarchical clustering, DBSCAN, K-medoids, MCL, connected components.
- **Evaluation** (`necom eval`): partition and tree comparison metrics, branch support assignment.
- **Tree cutting** (`necom cut`): split Newick trees into flat partitions by height, diameter, dynamic cut, and other
  criteria.
- **Matrix utilities** (`necom mat`): format conversion, subsetting, comparison, and mathematical transformations for
  distance matrices.
- **Tree operations** (`necom nwk`): rerooting, pruning, renaming, subtree extraction, topology comparison, statistics,
  and visualization.
- **Pipelines** (`necom pl condense`): integrated workflows such as taxonomic tree condensation.
- **Pipeline-friendly**: reads from `stdin`/writes to `stdout` where possible, with predictable output and composable
  subcommands.
- **Robust**: Rust implementation with a zero-panic policy for malformed inputs.

## Install

Current release: 0.3.0

`necom` uses unstable Rust features (notably `portable_simd`), so a **nightly** toolchain is required:

```bash
rustup toolchain install nightly
rustup run nightly cargo install --path . --force
```

## Test

```bash
rustup run nightly cargo test
```

## Quick start

After installation, the `necom` binary is available in your `PATH`:

```bash
necom --help
necom clust --help
necom mat --help
necom nwk --help
```

### Examples

```bash
# Hierarchical clustering from a PHYLIP distance matrix
necom clust hier tests/mat/IBPA.phy

# Compare two distance matrices
necom mat compare tests/mat/IBPA.phy tests/mat/IBPA.71.phy

# Tree statistics
necom nwk stat tests/newick/catarrhini.nwk

# Cut a tree into clusters by height
necom cut --height 0.5 tests/newick/catarrhini.nwk

# Evaluate a partition against ground truth
necom eval partition result.tsv --other truth.tsv

# Condense a tree by taxonomy
necom pl condense --taxon tests/pipeline/strains.taxon.tsv \
    tests/pipeline/minhash.reroot.newick
```

## Documentation

Extended documentation for each command is available in `docs/`:

- [`docs/clust.md`](clust.md) — clustering algorithms
- [`docs/cut.md`](cut.md) — tree cutting
- [`docs/mat.md`](mat.md) — matrix utilities
- [`docs/nwk.md`](nwk.md) — Newick tree operations
- [`docs/eval.md`](eval.md) — evaluation overview (partition & tree comparison)
- [`docs/eval-partition.md`](eval-partition.md) — partition evaluation deep dive
- [`docs/pl.md`](pl.md) — integrated pipelines
- [`docs/formats.md`](formats.md) — shared file format conventions

## Author

Qiang Wang <wang-q@outlook.com>

## License

MIT.

Copyright by Qiang Wang.

Written by Qiang Wang <wang-q@outlook.com>, 2024-
