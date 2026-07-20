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
    * `--dot`, `--bar`, `--rec`, and `--tri` take zero or one argument. When no value is supplied they use their default color (see below); any other value is passed through as the color directly.
* Defaults when no color is given:
    * `--dot` / `--bar`: `black`
    * `--rec`: `LemonChiffon`
    * `--tri`: `white`
* Predefined colors for `--color`, `--dot`, and `--bar`:
    * `red` (188,36,46)
    * `black` (26,25,25)
    * `grey` (129,130,132)
    * `brown` (121,37,0)
    * `green` (32,128,108)
    * `purple` (160,90,150)
    * `blue` (0,103,149)
* Predefined background rectangle colors for `--rec`:
    * `LemonChiffon` (251,248,204)
    * `ChampagnePink` (253,228,207)
    * `TeaRose` (255,207,210)
    * `PinkLavender` (241,192,232)
    * `Mauve` (207,186,240)
    * `JordyBlue` (163,196,243)
    * `NonPhotoBlue` (144,219,244)
    * `ElectricBlue` (142,236,245)
    * `Aquamarine` (152,245,225)
    * `Celadon` (185,251,192)
* `--remove <REGEX>` scans all nodes and removes whole property entries whose serialized `key=value` (or bare `key`) matches the regex.

Examples:

1. Add a string comment to a node
   `necom nwk comment tree.nwk --node A --string "bootstrap 90" -o out.nwk`

2. Add a colored dot to an internal node via LCA
   `necom nwk comment tree.nwk --lca A,B --dot red -o out.nwk`
