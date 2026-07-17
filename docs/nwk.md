# necom nwk

`necom nwk` provides a complete set of tools for processing **Newick**-format phylogenetic trees, covering information extraction, structural manipulation, and visualization.

## Subcommands

Subcommands are grouped into three categories:

*   **Information**: Extract information or statistics from trees.
    *   `cmp`: Compare trees (RF, WRF, KF distances).
    *   `distance`: Compute distances between nodes.
    *   `label`: Extract labels/names from the tree.
    *   `stat`: Print tree statistics (node count, leaf count, balance indices, etc.).
    *   `support`: Assign bootstrap support values.
*   **Manipulation**: Modify tree structure.
    *   `order`: Reorder nodes (ladderize, alphabetical).
    *   `prune`: Remove nodes (and their descendants/ancestors).
    *   `rename`: Rename specified nodes.
    *   `replace`: Batch replace names or comments from a file.
    *   `reroot`: Reroot the tree.
    *   `subtree`: Extract a specified clade/subtree.
    *   `topo`: Modify topology (remove branch lengths or labels).
*   **Visualization**: Format and visualize output.
    *   `comment`: Add visualization comments.
    *   `indent`: Format Newick with indentation.
    *   `to-dot`: Convert to Graphviz DOT format.
    *   `to-forest`: Convert to LaTeX Forest code.
    *   `to-svg`: Convert to SVG format.
    *   `to-tex`: Generate a complete LaTeX document.

---

## Information Commands

### cmp

Compare trees using Robinson-Foulds (RF) distance and its variants.

```bash
necom nwk cmp [OPTIONS] <infile> [compare_file]
```

*   `[compare_file]`: Optional second tree file. If omitted, all trees in `<infile>` are compared pairwise.
*   `--include-trivial`: Include trivial splits (single-leaf branches) in WRF/KF calculations.
*   Output columns: `Tree1`, `Tree2`, `RF_Dist`, `WRF_Dist`, `KF_Dist`.

> **RF (Robinson-Foulds)**: Measures topological difference based on the number of splits that differ between two trees. Smaller values indicate greater similarity; 0 means identical. WRF is weighted RF, and KF is the Kuhner-Felsenstein branch-score distance.

### distance

Compute distances between nodes or generate a distance matrix.

```bash
necom nwk distance [OPTIONS] <infile>
```

*   If the input file contains multiple trees, only the first tree is processed.
*   `--mode <mode>`: Computation mode.
    *   `root` (default): Distance to the root.
    *   `parent`: Distance to the parent.
    *   `pairwise`: Pairwise distances among all leaves.
    *   `lca`: Distance to the lowest common ancestor (LCA).
    *   `phylip`: Output a PHYLIP-format distance matrix for selected nodes.
*   `-I`: Ignore internal nodes.
*   `-L`: Ignore leaf nodes.

### label

Extract labels (names) from the tree.

```bash
necom nwk label [OPTIONS] <infile>
```

*   `-I`: Skip internal node labels.
*   `-L`: Skip leaf node labels.
*   `-n <name>` / `-l <file>` / `-x <regex>`: Filter nodes by name, list, or regex.
*   `-D`: Include descendants of selected nodes.
*   `-M`: Clade check (output only when the selected nodes form a monophyletic group with at least two nodes).
*   `--tab`: Output tab-separated values (single line).
*   `-c <col>` / `--extra-column <col>`: Append extra columns (`dup`, `taxid`, `species`, `full`).
*   `--root`: Output only the root node label. When `--root` is given, other selection options (`-I`, `-L`, `-n`, `-x`, etc.) are ignored.

### stat

Print tree statistics.

```bash
necom nwk stat [OPTIONS] <infile>
```

*   `--style <col|line>`: Output format (key-value pairs or TSV lines).
*   Statistics include: type (cladogram/phylogram/neither), node count, leaf count, rooted status, dichotomies, leaf labels, internal labels, cherries, Sackin index, Colless index.

### support

Assign bootstrap support values to a target tree based on replicate trees.

```bash
necom nwk support [OPTIONS] <target> <replicates>
```

*   `-p, --percent`: Output support values as percentages (0–100).
*   Overwrites existing internal node labels in the target tree.

---

## Manipulation Commands

### order

Sort children of each node (rotate branches) without changing topology.

```bash
necom nwk order [OPTIONS] <infile>
```

*   `--num-descendants` / `--num-descendants-rev`: Sort by number of descendants (ladderize).
*   `--alphanumeric` / `--alphanumeric-rev`: Sort labels alphanumerically.
*   `--name-list <file>`: Sort by a given name list; applied before other sort options.
*   `--deladderize` (`--dl`): Alternate sort direction at each level.
*   If no sort option is provided, children are sorted in ascending alphanumeric order by default.

When both `--alphanumeric` and `--num-descendants` are enabled, labels are sorted alphabetically first, then stably ladderized by number of descendants; the final order is primarily by number of descendants, with alphabetical order used as a tie-breaker within groups of the same size.

### prune

Remove nodes from the tree.

```bash
necom nwk prune [OPTIONS] <infile>
```

*   `-n <name>` / `-l <file>` / `-x <regex>`: Select nodes to remove.
*   `-i, --invert`: Invert selection (keep selected nodes, remove the rest).
*   `-D`: Include descendants.

### rename

Rename specified nodes.

```bash
necom nwk rename [OPTIONS] <infile>
```

*   `-n <name>`: Select node by name.
*   `-l <name1,name2>`: Select node by LCA of two names.
*   `--rename <new_name>`: New name(s), which must correspond one-to-one with `-n` or `-l` arguments.

### replace

Batch replace node names or comments using a TSV file.

```bash
necom nwk replace [OPTIONS] --replace-tsv <replace.tsv> <infile>
```

*   `--replace-tsv <replace.tsv>`: Tab-separated file with format `Original <TAB> Replacement [TAB Extra...]`.
*   `-I, --internal`: Skip internal labels.
*   `-L, --leaf`: Skip leaf labels.
*   `--mode <mode>`:
    *   `label` (default): Replace node names.
    *   `taxid`: Add as NCBI TaxID (`:T=`).
    *   `species`: Add as species name (`:S=`).
    *   `asis`: Append directly as comment/attribute. Values containing `=` are parsed as `key=value`; bare values are treated as keys with empty values.

### reroot

Reroot the tree.

```bash
necom nwk reroot [OPTIONS] <infile>
```

*   If the input file contains multiple trees, only the first tree is processed.
*   (Default): Root at the midpoint of the longest branch.
*   `-n <node>`: Root on the edge leading to the LCA of specified nodes (outgroup rooting).
*   `-l, --lax`: Lax mode (if the LCA is already the root, use its complement).
*   `-d, --deroot`: Deroot (produce a multifurcating root).
*   `--support-as-labels`: Treat support values as labels (rerooting will move labels).
*   The default midpoint rerooting is a no-op when the tree has no positive branch lengths (cladogram).

### subtree

Extract the subtree rooted at the LCA of selected nodes.

```bash
necom nwk subtree [OPTIONS] <infile>
```

*   `-n` / `-l` / `-x`: Select nodes.
*   `-D, --descendants`: Include all descendants of selected internal nodes.
*   `-M`: Clade check (output only when selected nodes form a monophyletic group with at least two nodes).
*   `-c <N>`: Context (expand upward by N levels). Default: `0`.
*   `-C, --condense <name>`: Condense the subtree to a single node.
    *   The new node gets `member=<count>` and `tri=white` comments.
    *   `<count>` is the number of named nodes matched by `-n/-l/-x`, including descendants expanded by `-D`.

### topo

Modify tree topology and properties.

```bash
necom nwk topo [OPTIONS] <infile>
```

*   By default removes branch lengths and comments, keeping only topology.
*   `-b, --bl`: Preserve branch lengths.
*   `-c, --comment`: Preserve comments.
*   `-I`: Remove internal labels.
*   `-L`: Remove leaf labels.

---

## Visualization Commands

### comment

Add comments to nodes for visualization.

```bash
necom nwk comment [OPTIONS] <infile>
```

*   `-n` / `-l`: Select nodes by name or name-list file.
*   `--lca <A,B>`: Select an internal node by the lowest common ancestor (LCA) of two comma-separated names. Can be specified multiple times.
*   `--string <str>`: Add a free-text string comment (stored as the `string` attribute).
*   `--color`, `--label`, `--comment-text`: Add text attributes (`--comment-text` stores as the `comment` attribute, used for visualization).
*   `--dot`, `--bar`, `--rec`, `--tri`: Add shape attributes (used by `to-tex` / `to-forest`).
*   `--remove <regex>`: Remove comments matching the regex.
*   `-o, --outfile <file>`: Output filename; `[stdout]` for screen output.

### indent

Format a Newick tree with indentation or compactly.

```bash
necom nwk indent [OPTIONS] <infile>
```

*   `--text <str>`: Indentation string (default `"  "`).
*   `-c, --compact`: Compact output (single line).
*   `-o, --outfile <file>`: Output filename; `[stdout]` for screen output.

### to-dot

Convert a Newick tree to Graphviz DOT format.

```bash
necom nwk to-dot [OPTIONS] <infile>
```

*   `-o, --outfile <file>`: Output filename; `[stdout]` for screen output.

### to-forest

Convert a Newick tree to raw LaTeX Forest code.

```bash
necom nwk to-forest [OPTIONS] <infile>
```

*   If the input file contains multiple trees, only the first tree is processed.
*   `-b, --bl`: Include branch lengths.
*   `-o, --outfile <file>`: Output filename; `[stdout] for screen output.
*   LaTeX special characters (`{ } \ # $ % & ~ ^`) and underscores in node names, labels, and comments are escaped or normalized for safe Forest output.

### to-svg

Convert a Newick tree to SVG format for visualization.

```bash
necom nwk to-svg [OPTIONS] <infile>
```

*   If the input file contains multiple trees, only the first tree is processed.
*   Draws a phylogram if branch lengths exist, otherwise a cladogram.
*   `-w, --width <N>`: Drawing-area width used as the branch-length scaling baseline in pixels (default 800). The final SVG width also includes margins for leaf labels.
*   `-v, --vskip <N>`: Vertical spacing between leaf nodes in pixels (default 20).
*   `-o, --outfile <file>`: Output filename; `[stdout]` for screen output.

### to-tex

Wrap a Newick tree in a complete LaTeX document (based on `to-forest`).

```bash
necom nwk to-tex [OPTIONS] <infile>
```

*   If the input file contains multiple trees, only the first tree is processed (unless `--forest` is used, in which case the input is passed through verbatim).
*   `-b, --bl`: Draw a phylogram (with branch lengths).
*   `--forest`: Input is already Forest code.
*   `--no-default-style`: Skip default style definitions.
*   `-o, --outfile <file>`: Output filename; `[stdout] for screen output.
*   LaTeX special characters (`{ } \ # $ % & ~ ^`) and underscores in node names, labels, and comments are escaped or normalized for safe LaTeX compilation.

> LaTeX output requires a locally installed `xelatex` or an engine such as `tectonic` to compile. By default the embedded style overrides the template fonts with `Noto Sans`; use `--no-default-style` to keep the template's own font setup (`Fira Sans` / `Source Han Sans SC`).

---

## Label Handling

*   Unquoted labels are trimmed: leading and trailing whitespace is removed. Internal whitespace is preserved only when the label is quoted with single or double quotes.
*   Labels containing Newick reserved characters (`( ) : ; , [ ]`) or whitespace must be quoted to round-trip correctly.
*   Single quotes inside single-quoted labels are escaped by doubling (`''`), and double quotes inside double-quoted labels are escaped by doubling (`""`).

---

## Branch Length Handling

`necom nwk` treats non-finite branch lengths (`NaN`, positive/negative infinity), negative values, and zero values as `0.0` during computation and visualization. On input, such values are normalized to `None` (no length annotation); on output, `None` and zero (`0.0`) lengths are omitted so that cladograms remain unannotated. This applies to:

*   Statistics (`stat`) and distance calculations (`distance`).
*   Tree comparison (`cmp`), including weighted Robinson-Foulds and Kuhner-Felsenstein distances.
*   Tree operations such as `reroot`, collapse, and insert_parent.
*   Visualization (`to-svg`, `to-dot`, `to-forest`, `to-tex`).

This normalization prevents invalid values from polluting sums, maxima, or distance computations. Note that input files themselves are not modified; normalization occurs only during internal computation. If strict branch-length validation is needed, clean the data in the input first.

### Length detection and distance threshold

*   `stat` counts an edge as "with length" only when its branch length is a positive finite value. Missing, zero, and non-finite lengths are reported as "without length".
*   `distance` uses the sum of branch lengths only when its absolute value exceeds `1e-9`; otherwise it falls back to the topological edge count. This avoids treating near-zero floating-point values as meaningful distances.

---

## Planned Subcommands

*   `condense`: Tree condensation functionality is currently provided by `necom nwk subtree --condense`; no standalone `condense` subcommand is planned at this time.
*   `eval`: Multi-dimensional tree evaluation framework (geometric, taxonomic, phylogenetic, trait consistency). Related metrics are referenced in the `necom clust eval` and `necom clust cut` documentation.
