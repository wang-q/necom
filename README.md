# necom — NWK, CLUST, and MAT Toolkit

[![Build](https://github.com/wang-q/necom/actions/workflows/build.yml/badge.svg)](https://github.com/wang-q/necom/actions)
[![codecov](https://codecov.io/gh/wang-q/necom/branch/master/graph/badge.svg)](https://codecov.io/gh/wang-q/necom)
[![license](https://img.shields.io/github/license/wang-q/necom)](https://github.com/wang-q/necom)
[![Documentation](https://img.shields.io/badge/docs-online-blue)](https://wang-q.github.io/necom/)

`necom` is a command-line toolkit for **clustering**, **distance-matrix processing**, and
**phylogenetic-tree manipulation**.

The name is formed from its three command families — **NWK**, **CLUST**, and **MAT** — with vowels
inserted in alphabetical order. It also echoes the Latin *nexum*("tie" or "bond"), reflecting the
toolkit's focus on connections between clusters, matrix entries, and tree nodes.

## Features

- **Clustering** (`necom clust`): hierarchical (NN-chain), DBSCAN, K-medoids, MCL, connected
  components, plus tree-building algorithms (Neighbor-Joining, UPGMA).
- **Evaluation** (`necom eval`): partition metrics (ARI, AMI, NMI, FMI, Jaccard, etc.), tree
  topology distances (Robinson-Foulds, KF), and branch-support replication.
- **Tree cutting** (`necom cut`): split Newick trees into flat partitions by height, K, root
  distance, clade size, dynamic cut, or hybrid dynamic + PAM; parameter sweeps via `scan-*`.
- **Matrix utilities** (`necom mat`): PHYLIP/pair format conversion, subsetting, pairwise
  comparison (Pearson, Spearman, cosine, Jaccard, MAE), and transformations (log, sqrt, normalize,
  etc.); `from-vector` computes pairwise scores from feature vectors.
- **Tree operations** (`necom nwk`): rerooting, pruning, renaming, subtree extraction, topology
  comparison, statistics, distance, and visualization (SVG, DOT, LaTeX Forest).
- **Pipelines** (`necom pl condense`): integrated workflows such as taxonomic tree condensation.
- **Pipeline-friendly**: reads from `stdin` / writes to `stdout` where possible, with predictable
  output and composable subcommands.
- **Robust**: Rust implementation with a zero-panic policy for malformed inputs.

## Commands

- `necom clust` — `cc`, `dbscan`, `hier`, `k-medoids`, `mcl`, `nj`, `upgma`
- `necom cut` — `simple`, `dynamic`, `hybrid`, `scan-simple`, `scan-dynamic`
- `necom eval` — `compare`, `partition`, `replicate`
- `necom mat` — `compare`, `format`, `from-vector`, `subset`, `to-pair`, `to-phylip`, `transform`
- `necom nwk`
    - Information: `stat`, `label`, `distance`
    - Manipulation: `order`, `prune`, `rename`, `replace`, `reroot`, `subtree`, `topo`
    - Visualization: `comment`, `indent`, `to-dot`, `to-forest`, `to-svg`, `to-tex`
- `necom pl` — `condense`

## Install

Current release: 0.4.1

`necom` requires the Rust nightly toolchain (pinned by `rust-toolchain.toml` for `portable_simd`),
auto-installed by `cargo` on first use:

```bash
cargo install --path . --force
```

## Quick start

After installation, the `necom` binary is available in your `PATH`:

```bash
necom --help
necom clust --help
necom cut --help
necom eval --help
necom mat --help
necom nwk --help
necom pl --help
```

### Examples

```bash
# Hierarchical clustering from a PHYLIP distance matrix
necom clust hier tests/mat/IBPA.phy
# (((((IBPA_ECOLI,IBPA_ESCF3),A0A192CFC5_ECO25):0.0358,IBPA_ECOLI_GA):0.1467,...

# Compare two distance matrices
necom mat compare tests/mat/IBPA.phy tests/mat/IBPA.71.phy
# Method  Score
# pearson 0.935803

# Tree statistics
necom nwk stat tests/newick/catarrhini.nwk
# Type    phylogram
# nodes   19
# leaves  10
# rooted  Yes
# cherries        3
# sackin  36
# colless 8

# Cut a tree into clusters by height
necom cut simple --height 0.05 tests/newick/catarrhini.nwk
# Cercopithecus
# Colobus
# Gorilla
# ...

# Evaluate a partition against ground truth
necom eval partition result.tsv --other truth.tsv

# Condense a tree by taxonomy
necom pl condense --taxon tests/pipeline/strains.taxon.tsv \
    tests/pipeline/minhash.reroot.newick
```

## Documentation

Extended documentation for each command is available in `docs/`:

- [`docs/clust.md`](docs/clust.md) — clustering algorithms
- [`docs/cut.md`](docs/cut.md) — tree cutting
- [`docs/eval.md`](docs/eval.md) — evaluation overview (partition & tree comparison)
- [`docs/eval-partition.md`](docs/eval-partition.md) — partition evaluation deep dive
- [`docs/mat.md`](docs/mat.md) — matrix utilities
- [`docs/nwk.md`](docs/nwk.md) — Newick tree operations
- [`docs/nwk-tex.md`](docs/nwk-tex.md) — LaTeX Forest tree export
- [`docs/pl.md`](docs/pl.md) — integrated pipelines
- [`docs/formats.md`](docs/formats.md) — shared file format conventions

Per-subcommand help text lives under [`docs/help/`](docs/help/) and is also reachable
via `necom <command> <subcommand> --help`. The rendered mdBook site is published at
[https://wang-q.github.io/necom/](https://wang-q.github.io/necom/).

## Author

Qiang Wang [wang-q@outlook.com](mailto:wang-q@outlook.com)

## License

MIT.

Copyright by Qiang Wang. 2024-

