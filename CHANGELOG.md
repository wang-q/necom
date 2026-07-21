# Change Log

## Unreleased - ReleaseDate

### Added

*   `nwk order --olo`: Optimal Leaf Ordering via distance-matrix dynamic
    programming.
*   `clust scan-dbscan` subcommand and `--min-pct` cluster filter flag for
    `dbscan`/`kmedoids`/`mcl`/`scan-dbscan`.
*   Davies-Bouldin index in `eval partition`.
*   SciPy parity test suite for clustering and phylogenetic tree construction.

### Fixed

*   NaN safety: replaced `partial_cmp().unwrap_or(Equal)` with `total_cmp`
    across `clust`, `cut`, `phylo`, and `eval` modules.
*   `pairmat` log/normalize now propagate NaN and handle non-finite diagonals
    correctly; `eval partition` rejects non-finite coordinate input.
*   `cut scan-*` defers output headers until first valid dispatch.

## 0.4.0 - 2026-07-20

### Breaking Changes

*   Promoted `necom clust cut` to a top-level `necom cut` command with five
    subcommands (`simple`, `dynamic`, `hybrid`, `scan-simple`,
    `scan-dynamic`). The `--method`/`--threshold` interface was replaced with
    dedicated flags (e.g. `--k`, `--height`, `--max-clade`) and `--scan` was
    renamed to `--range`.
*   Migrated `necom clust eval`, `nwk compare`, and `nwk support` into a new
    top-level `necom eval` command with `partition`, `compare`, and
    `replicate` subcommands.
*   Migrated test fixtures and partition input files from comma-separated to
    pure TSV format.
*   Reorganized shared libraries: `tree_cut` → `cut`, clustering evaluation
    extracted to `libs/eval`, feature vectors to `libs/feature`,
    `absolute_path` relocated to `libs/io`.

### Added

*   `necom eval` command suite (`compare`, `partition`, `replicate`) and
    `necom cut` command suite (five cutting modes with parameter sweep
    variants).
*   `necom mat from-vector` subcommand for pairwise similarity/distance
    scoring between feature vectors with optional binarization and parallel
    execution.
*   `necom nwk cmp` (later renamed `compare`) subcommand for tree topology
    comparison.
*   PHYLIP lower-triangular matrix format without diagonal values, with
    automatic layout detection.
*   SIMD-accelerated linear algebra helpers in `libs/linalg` (euclidean,
    cosine, jaccard, Pearson, Spearman, MAE) and `MatrixView` trait unifying
    `ScoringMatrix` / `NamedMatrix` access for clustering algorithms.
*   Centralized markdown help docs under `docs/help/` loaded via
    `include_str!`, plus mdBook online documentation with GitHub Pages CI.
*   Node selection filters (`-n`/`-l`/`-x`) and edge length statistics in
    `nwk distance` / `nwk stat`; NHX value escaping for `:=;,` and
    LaTeX/DOT/SVG special-character escaping.
*   `rustfmt.toml` and `rust-toolchain.toml` pinning nightly Rust for
    `portable_simd`.

### Changed

*   Reorganized `nwk` subcommand groups (`info`/`ops`/`viz` →
    `Information`/`Manipulation`/`Visualization`).
*   Switched pairwise matrix storage to compact upper-triangular indexing and
    refactored `par_run_pairs` to use a local `rayon::ThreadPool`, reducing
    peak memory and allowing repeated invocations in one process.
*   Replaced `crossbeam` with `crossbeam-channel`; removed unused dependencies
    (`itertools`, `intspan`, `flate2`).
*   Newick output now consistently omits zero, non-finite, and negative branch
    lengths; `Node::finite_length()` normalizes length lookups.
*   `ScoringMatrix::set` silently ignores out-of-bounds writes (matching
    `NamedMatrix`); `CondensedMatrix::from_vec` returns `Result` instead of
    panicking.
*   Hierarchical `NN-chain` preserves original merge order and falls back to
    O(N³) for non-reducible linkage methods (centroid, median); DBSCAN output
    sorted by cluster ID for determinism.
*   `mat to-pair` and `mat compare` emit `NA` instead of raw `NaN`/`Inf`;
    `mat subset` defaults to 6 decimal places.
*   `read_lines` propagates IO errors instead of silently truncating;
    `Tree::remove_degree_two_nodes`, `prune_nodes`, `collapse_node`, and
    `set_root` now return `Result`.
*   Reworked tree traversal to iterative stack-based postorder to avoid stack
    overflow on deep trees.

### Fixed

*   Prevented output file truncation across all `clust`, `cut`, `eval`,
    `mat`, `nwk`, and `pl` subcommands: writers are now opened only after
    input loading and validation succeed.
*   Eliminated non-deterministic output in `cut single-linkage`, `hybrid cut`,
    and `scan-simple` caused by `HashMap` iteration order.
*   Replaced panicking `unwrap`/`expect` calls with `anyhow` error
    propagation across clustering, cut, and phylo modules.
*   Fixed `pl condense` stdin double-consumption by buffering stdin to a temp
    file.
*   Added finite-value validation for floating-point parameters (`--eps`,
    `--inflation`, `--support`, `--same`, `--missing`, `--max-tree-height`,
    `--max-pam-dist`) to reject `NaN`/`Inf`/negative values.
*   Corrected asymmetric `homogeneity`/`completeness` calculation (swapped
    denominators) and clamped negative branch lengths to `0` in NJ/UPGMA
    output for non-ultrametric input.
*   Fixed `tau_score` tie handling (exact equality instead of epsilon),
    PHYLIP strict-mode name truncation (char boundaries), `pairmat` normalize
    transform (negative diagonals via `NEG_INFINITY`), and `eval replicate`
    silently skipping unnamed leaves.
*   Fixed `dynamic` tree cut node height caching (all nodes, not just root)
    and support filter sentinel (finite `1e20` instead of `f64::INFINITY`).
*   Added leaf set consistency validation in `eval compare`, duplicate sample
    detection in `eval partition`, and range validation for `cut scan`.
*   Fixed `partition` parser over-skipping lines starting with `Group` or
    `Threshold`; fixed `read_lines` silently discarding failed reads.

### Documentation

*   Rewrote user-facing docs for all command groups to match the new
    structure; added `docs/formats/` and `docs/help/` centralized references.
*   Added design notes under `notes/design/`; replaced `ignore` code fences
    with runnable `rust` doc tests.
*   Standardized "clade" terminology in place of "monophyletic group".

### Repository

*   Removed unused dependencies (`itertools`, `intspan`, `flate2`); switched
    `crossbeam` → `crossbeam-channel`.
*   Added mdBook + GitHub Pages CI workflow, `rustfmt.toml`, and
    `rust-toolchain.toml`.

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

*   Rewrote user-facing docs (`docs/*.md`) for `clust`, `cut`, `clust eval`, `mat`, `mat transform`, `nwk`, and `pl` to match current CLI options and behavior.
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
