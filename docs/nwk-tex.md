# LaTeX Forest Output

`necom nwk to-forest` and `necom nwk to-tex` convert Newick trees into LaTeX code using the
[Forest](https://ctan.org/pkg/forest) package. This is useful for producing publication-quality
phylogenetic trees directly from the command line.

- `to-forest` emits raw Forest code that can be embedded into an existing LaTeX document.
- `to-tex` generates a complete `.tex` document by merging the Forest code with a built-in template,
  ready for compilation with `xelatex` or `tectonic`.

---
## Core Commands

### `necom nwk to-forest`

Generates raw LaTeX Forest code.

```bash
necom nwk to-forest tree.nwk
```

Use `--bl` to draw a phylogram with branch lengths instead of a cladogram:

```bash
necom nwk to-forest tree.nwk --bl
```

### `necom nwk to-tex`

Generates a complete LaTeX document.

```bash
necom nwk to-tex tree.nwk -o tree.tex
```

Compile the output:

```bash
tectonic tree.tex
# or
latexmk -xelatex tree.tex
```

Use `--forest` to pass through an existing Forest code file instead of a Newick tree:

```bash
necom nwk to-tex forest.tex --forest -o document.tex
```

---
## Style System

Styles are attached to nodes as NHX annotations. The template defines four core visual attributes:

### `dot`

Draws a filled circle at the node.

```text
[&&NHX:dot=red]
```

When no color is given, `dot` is automatically applied to named internal nodes.

### `bar`

Draws a short perpendicular bar on the edge between the node and its parent. Useful for marking
events or character-state changes.

```text
[&&NHX:bar=blue]
```

### `rec`

Draws a background rectangle behind the entire clade rooted at the node.

```text
[&&NHX:rec=LemonChiffon]
```

This is often combined with the soft palette defined in the template.

### `tri`

Draws a triangle to the right of the node, commonly used for collapsed clades or to emphasize a
leaf.

```text
[&&NHX:tri=green]
```

---
## Colors and Global Settings

The built-in template provides a soft, muted palette (for example `ChampagnePink`, `TeaRose`, and
`Celadon`). By default:

- `tier=word` is enabled, aligning all leaf nodes at the same level (cladogram style).
- Branch lines are gray (1 pt).
- Nodes are drawn as small black circles (2 pt).
- A sans-serif font is used.

Font behavior:

- By default, the template uses `Noto Sans`, which is widely available.
- Use `--no-default-style` to keep the template's original `Fira Sans` / `Source Han Sans SC` setup
  instead of injecting the default `Noto Sans` configuration.

---
## Advanced Features

### Phylogram Mode (`--bl`)

When the input tree contains branch lengths, `--bl` produces a phylogram with a automatically
computed scale bar. The scale values (e.g., `0.01`, `0.05`, `1.0`) are chosen based on the tree
height and rendered in the lower-right corner.

### Forest Pass-Through (`--forest`)

`to-tex --forest` allows an externally generated Forest code file to be wrapped in the template.
This is useful when the Forest code has been produced by `to-forest` and then manually adjusted.

### Special Character Handling

LaTeX special characters and Newick conventions are normalized automatically:

- Underscores (`_`) in node names are replaced with spaces.
- LaTeX special characters (`{ } \ # $ % & ~ ^`) in node names, labels, comments, and visualization
  attributes are escaped.

This ensures that the generated Forest code compiles without manual cleanup.

---
## Workflow Example

1. Annotate the tree with visualization attributes:

  ```bash
  necom nwk comment input.nwk --lca A,B --rec TeaRose --label Group1 > annotated.nwk
  ```

2. Generate the LaTeX document:

  ```bash
  necom nwk to-tex annotated.nwk -o output.tex
  ```

3. Compile:

  ```bash
  tectonic output.tex
  ```

---
## Requirements

Compiling `to-tex` output requires a LaTeX installation that includes:

- `fontspec`
- `xeCJK` (for East Asian characters)
- `forest`

[Tectonic](https://tectonic-typesetting.github.io/) is the recommended compiler, but
`latexmk -xelatex` also works.

