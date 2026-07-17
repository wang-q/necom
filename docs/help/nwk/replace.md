Replace node names or append annotations in a Newick file using a TSV file.

Input:

* A Newick tree file.
* A TSV file with 2 or more columns: `<original_name> <replacement> [additional_annotations...]`.

Notes:

* The behavior of the 2nd column (`<replacement>`) depends on `--mode`:
    * `label` (default): replaces the node name. An empty string removes the name.
    * `taxid`: appends as NCBI TaxID (`:T=<replacement>`) in NHX.
    * `species`: appends as species name (`:S=<replacement>`) in NHX.
    * `asis`: appends as comments/properties. Values containing `=` are parsed as `key=value` pairs; bare values are stored as keys with empty values.
* Columns 3+ are always appended to the node's comments/properties. Key-value pairs (e.g., `color=red`) are stored as properties; simple tags (e.g., `highlight`) are stored as keys with empty values.

Examples:

1. Basic renaming of nodes
   `necom nwk replace input.nwk --replace-tsv names.tsv > output.nwk`

2. Add species and color annotations
   `necom nwk replace input.nwk --replace-tsv annotations.tsv --mode species`

3. Remove node names (2nd column is empty)
   `necom nwk replace input.nwk --replace-tsv remove.tsv`
