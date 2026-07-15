# Change Log

## Unreleased - ReleaseDate

## 0.3.0 - 2026-07-15

### Breaking Changes

*   Renamed project from `pgr` to `necom`.
*   Removed all commands outside the clustering / matrix / phylogeny scope:
    `2bit`, `axt`, `chain`, `dist`, `fa`, `fas`, `fq`, `gff`, `lav`, `maf`, `ms`, `net`, `plot`, `psl`.
*   Reduced `necom pl` to the `condense` pipeline; removed `p2m`, `prefilter`, `trf`, `ir`, `rept`, `ucsc`.

### Changed

*   Moved command implementations from `src/cmd_pgr/` to `src/cmd_necom/`.
*   Reorganized shared libraries under `src/libs/` (`phylo/`, `clust/`, `pairmat/`, etc.).
*   Added `rust-toolchain.toml` pinning nightly Rust (required for `portable_simd`).
*   Renamed `necom clust kmedoids` internal module to `k_medoids`.

### Added

*   `necom nwk to-svg`: SVG export for phylogenetic trees.

### Documentation

*   Rewrote user-facing docs (`docs/*.md`) for `clust`, `clust cut`, `clust eval`, `mat`, `mat transform`, `nwk`, and `pl` to match current CLI options and behavior.
*   Added `docs/pl.md` for the `necom pl condense` pipeline.
*   Reworked `README.md` with the project name origin, a docs index, and the nightly toolchain note.

### Fixed

*   Restored `src/libs/io.rs` after it was accidentally removed during repository history rewriting.

### Repository

*   Rewrote Git history with `git-filter-repo` to remove records of 753 previously deleted files, reducing `.git` size.

## 0.2.0 - 2026-04-05

### New Features

*   `pgr` command-line toolkit for sequence, alignment, clustering, matrix, phylogeny, simulation, and plotting workflows.
*   Commands: `2bit`, `axt`, `chain`, `clust`, `dist`, `fa`, `fas`, `fq`, `gff`, `lav`, `maf`, `mat`, `ms`, `net`, `nwk`, `pl`, `plot`, `psl`.
*   Core libraries for phylogenetics, clustering, sequence I/O, and genomic alignment formats.

## 0.1.0 - 2025-02-08

*   Initial release of `pgr` (version 0.1.0).
