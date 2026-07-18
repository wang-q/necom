Add comments to node(s) in a Newick file.

Input:

* A Newick tree file.

Notes:

* Comments are stored in an NHX-like format (`:key=value`).
* For named nodes, use `--node`.
* For unnamed internal nodes, use `--lca` with two comma-separated names, e.g. `--lca A,B`.
* Use `--string` to add a free-form string stored under the `string` property.
* Visualization options:
    * `--color`, `--label`, and `--comment-text` each take one argument.
    * `--dot`, `--bar`, `--rec`, and `--tri` take zero or one argument.
* Predefined colors for `--color`, `--dot`, and `--bar`:
    * `red` {RGB}{188,36,46}
    * `black` {RGB}{26,25,25}
    * `grey` {RGB}{129,130,132}
    * `green` {RGB}{32,128,108}
    * `purple` {RGB}{160,90,150}
* Background rectangle colors for `--rec`:
    * `LemonChiffon` {RGB}{251,248,204}
    * `ChampagnePink` {RGB}{253,228,207}
    * `TeaRose` {RGB}{255,207,210}
    * `PinkLavender` {RGB}{241,192,232}
    * `Mauve` {RGB}{207,186,240}
    * `JordyBlue` {RGB}{163,196,243}
    * `NonPhotoBlue` {RGB}{144,219,244}
    * `ElectricBlue` {RGB}{142,236,245}
    * `Aquamarine` {RGB}{152,245,225}
    * `Celadon` {RGB}{185,251,192}
* `--tri` places a triangle at the end of the branch (default color: `white`).
* `--remove <REGEX>` scans all nodes and removes parts of comments matching the regex.

Examples:

1. Add a string comment to a node
   `necom nwk comment tree.nwk --node A --string "bootstrap 90" -o out.nwk`

2. Add a colored dot to an internal node via LCA
   `necom nwk comment tree.nwk --lca A,B --dot red -o out.nwk`
