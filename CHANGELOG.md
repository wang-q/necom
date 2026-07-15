# Change Log

## Unreleased - ReleaseDate

### Documentation

*   Added `docs/pl.md` to document the `necom pl condense` pipeline.
*   Updated `docs/clust.md` with complete CLI options and defaults for `cc`, `dbscan`, `k-medoids`, `mcl`, `upgma`, `nj`, and `hier`.
*   Fixed `docs/mat.md` and `docs/mat-transform.md` to use the correct `--max-val` flag for `necom mat transform`.
*   Updated `docs/clust-cut.md` with common options (`--format`, `--rep`, `--deep`, `--support`, `--scan`, `--stats-out`) and corrected a scan-range example.
*   Updated `docs/nwk.md` to note commands that process only the first tree in a multi-tree file.
*   Added missing `--lca` option to `docs/nwk.md` (`necom nwk comment`) and noted the default value of `--context` in the `subtree` section.
*   Corrected another stale `--max` reference in `docs/clust.md` (`clust hier` similarity conversion example) to `--max-val`.
*   Clarified input format support in `docs/clust-eval.md` (single mode defaults to `pair`; batch mode requires `long`).
*   Fixed `--scan` placeholder wording in `docs/clust-cut.md`.
*   Supplemented `docs/mat.md` with `--method` defaults/multi-value support for `mat compare` and the default `--format` for `mat format`.

## 0.2.0 - 2026-04-05

### New Features

#### Clustering and Phylogenetic Analysis

* **`necom clust`** - Clustering operations (9 subcommands)
  * `hier`: Hierarchical clustering (NN-chain algorithm)
  * `nj`: Neighbor-Joining tree construction
  * `upgma`: UPGMA tree construction
  * `cc`: Connected components clustering
  * `cut`: Tree cutting for cluster extraction
  * `dbscan`: DBSCAN clustering
  * `k-medoids`: K-medoids clustering
  * `mcl`: Markov Clustering (MCL)
  * `eval`: Cluster evaluation metrics

* **`necom mat`** - Matrix operations (6 subcommands)
  * `compare`: Compare distance matrices
  * `format`: Format matrix files
  * `subset`: Create matrix subset
  * `to-pair`: Convert to pair format
  * `to-phylip`: Convert to PHYLIP format
  * `transform`: Matrix transformations

* **`necom nwk`** - Newick tree manipulation and visualization (17 subcommands)
  * `stat`: Tree statistics
  * `label`: Label tree nodes
  * `distance`: Calculate pairwise distances
  * `support`: Branch support operations
  * `order`: Order tree nodes
  * `prune`: Prune tree branches
  * `rename`: Rename tree nodes
  * `replace`: Replace node information
  * `reroot`: Reroot the tree
  * `subtree`: Extract subtrees
  * `topo`: Topological operations
  * `comment`: Add comments to trees
  * `indent`: Indent/format tree files
  * `to-dot`: Convert to Graphviz DOT format
  * `to-forest`: Convert to forest representation
  * `to-tex`: Convert to LaTeX/TikZ format
  * `cmp`: Compare trees

#### Pipelines

* **`necom pl`** - Integrated pipelines
  * `condense`: Condense subtrees based on taxonomy

### Core Libraries

* **`src/libs/phylo/`** - Phylogenetic analysis core library
  * Tree structure definitions and traversals
  * Tree I/O operations
  * Tree statistics (`stat.rs`)
  * Tree manipulation algorithms (sorting, rerooting)

* **`src/libs/clust/`** - Clustering algorithm implementations
  * `hier.rs`: Hierarchical clustering with NN-chain algorithm
  * `dbscan.rs`: DBSCAN implementation
  * `mcl.rs`: Markov Clustering implementation
  * `k_medoids.rs`: K-medoids implementation

* **`src/libs/io.rs`** - I/O utilities

### Technical Features

* **Rust Implementation**: High-performance, memory-safe implementation
* **Parallel Computing**: Rayon-based parallelism for performance-critical operations
* **Zero Panic Policy**: Robust error handling for malformed inputs
* **Pipeline-Friendly**: stdin/stdout support where possible, predictable outputs
* **Comprehensive CLI**: Built with clap for excellent command-line experience
* **Testing**: Extensive integration tests using assert_cmd

### Dependencies

* **CLI**: clap 4.5.28
* **Error Handling**: anyhow 1.0.93
* **Parallelism**: rayon 1.10.0
* **Parsing**: nom 8.0.0, regex 1.11.1
* **Data Structures**: petgraph 0.7.1, indexmap 2.13.0

## 0.1.0 - 2025-02-08

* New binary `necom`
