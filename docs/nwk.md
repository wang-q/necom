# necom nwk

`necom nwk` provides a complete set of tools for processing **Newick**-format phylogenetic trees,
covering information extraction, structural manipulation, and visualization.

## Subcommands

Subcommands are grouped into three categories:

- **Information**: Extract information or statistics from trees.
    - `distance`: Compute distances between nodes.
    - `label`: Extract labels/names from the tree.
    - `stat`: Print tree statistics (node count, leaf count, balance indices, etc.).
- **Manipulation**: Modify tree structure.
    - `order`: Reorder nodes (ladderize, alphabetical).
    - `prune`: Remove nodes (and their descendants/ancestors).
    - `rename`: Rename specified nodes.
    - `replace`: Batch replace names or comments from a file.
    - `reroot`: Reroot the tree.
    - `subtree`: Extract a specified clade/subtree.
    - `topo`: Modify topology (remove branch lengths or labels).
- **Visualization**: Format and visualize output.
    - `comment`: Add visualization comments.
    - `indent`: Format Newick with indentation.
    - `to-dot`: Convert to Graphviz DOT format.
    - `to-forest`: Convert to LaTeX Forest code.
    - `to-svg`: Convert to SVG format.
    - `to-tex`: Generate a complete LaTeX document.

---
## Information Commands

### distance

Compute distances between nodes or generate a distance matrix. Use `--mode` to select root, parent,
pairwise, lca, or phylip output.

### label

Extract labels (names) from the tree.

### stat

Print tree statistics.

---
## Manipulation Commands

### order

Sort children of each node without changing topology.

### prune

Remove selected nodes from the tree.

### rename

Rename specified nodes.

### replace

Batch replace node names or append annotations from a TSV file. Use `--mode` to choose label, taxid,
species, or asis replacement.

### reroot

Reroot the tree. The default is midpoint rooting; use `-n` to specify the ingroup, rerooting on the
edge leading to the LCA of the specified nodes.

### subtree

Extract the subtree rooted at the LCA of selected nodes, or condense it into a single node with
`--condense`.

### topo

Modify tree topology and properties, such as removing branch lengths or labels.

---
## Visualization Commands

### comment

Add visualization comments or attributes to selected nodes.

### indent

Format a Newick tree with indentation or compact output.

### to-dot

Convert to Graphviz DOT format.

### to-forest

Convert to LaTeX Forest code.

### to-svg

Convert to SVG format. Draws a phylogram when branch lengths exist; otherwise draws a cladogram.

### to-tex

Wrap a Newick tree in a complete LaTeX document.

---
## Label Handling

- Unquoted labels are trimmed: leading and trailing whitespace is removed. Internal whitespace is
  preserved only when the label is quoted with single or double quotes.
- Labels containing Newick reserved characters (`( ) : ; , [ ]`) or whitespace must be quoted to
  round-trip correctly.
- Single quotes inside single-quoted labels are escaped by doubling (`''`), and double quotes
  inside double-quoted labels are escaped by doubling (`""`).
- Name-based selection filters (`-n`, `-l`, `-x`, `--lca`) generally log a warning and continue
  when a requested name is not found, rather than abort. This applies to commands such as `rename`,
  `subtree`, `comment`, and `prune`.
- `order --name-list` is an exception: entries in the name list that are not found among the leaf
  names cause the command to fail with an error listing the missing names.
- `reroot` is also an exception: if none of the names specified with `--node` are found, it reports
  an error and exits, because there is no valid target for rerooting.

---
## Branch Length Handling

`necom nwk` treats non-finite branch lengths (`NaN`, positive/negative infinity), negative values,
and zero values as `0.0` during computation and visualization. On input, such values are normalized
to `None` (no length annotation); on output, `None` and zero (`0.0`) lengths are omitted so that
cladograms remain unannotated. This applies to:

- Statistics (`stat`) and distance calculations (`distance`).
- Tree operations such as `reroot`, collapse, and insert_parent.
- Visualization (`to-svg`, `to-dot`, `to-forest`, `to-tex`).

This normalization prevents invalid values from polluting sums, maxima, or distance computations.
Note that input files themselves are not modified; normalization occurs only during internal
computation. If strict branch-length validation is needed, clean the data in the input first.

### Length detection and distance threshold

- `stat` counts an edge as "with length" only when its branch length is a positive finite value.
  Missing, zero, and non-finite lengths are reported as "without length".
- `distance` in `root`, `parent`, `pairwise`, and `lca` modes uses the sum of branch lengths only
  when its absolute value exceeds `1e-9`; otherwise it falls back to the topological edge count.
  This avoids treating near-zero floating-point values as meaningful distances.
- `distance --mode phylip` uses a tree-wide decision: if any edge in the tree carries a positive
  finite length, the full matrix is computed from branch lengths; otherwise it falls back to
  topological edge counts for all pairs.

---
## Planned Subcommands

- `condense`: Tree condensation functionality is currently provided by
  `necom nwk subtree --condense`; no standalone `condense` subcommand is planned at this time.
- `match`, `ed`, `gen`, `duration`: Mapped from `newick_utils` but not yet implemented in
  `necom nwk`; no concrete plan at this time.
- Tree evaluation (geometric, taxonomic, phylogenetic, trait consistency) is planned as a top-level
  command `necom eval`. `nwk compare` and `nwk support` have been migrated to `necom eval compare`
  and `necom eval replicate`; see [`docs/eval.md`](eval.md). The `eval tree` subcommand remains in
  design — see [`notes/design/eval-planned.md`](../notes/design/eval-planned.md).

