Compare trees using Robinson-Foulds (RF) distance and its variants.

Input:

* One file: compares all trees in the file against each other (pairwise).
* Two files: compares each tree in the first file against each tree in the second file.

Output:

* TSV with columns `Tree1`, `Tree2`, `RF_Dist`, `WRF_Dist`, `KF_Dist`.
* Tree IDs are 1-based indices of the trees in the input files.

Notes:

* Metrics:
    * `RF`: Robinson-Foulds distance (topological difference).
    * `WRF`: Weighted Robinson-Foulds distance (branch length difference). Trivial splits (single-leaf branches) are excluded by default.
    * `KF`: Kuhner-Felsenstein (Branch Score) distance. Trivial splits are excluded by default.
* Use `--include-trivial` to include single-leaf splits in WRF/KF calculations.
* Single-file pairwise mode requires at least 2 trees; with fewer, the command errors out with a clear message.

Examples:

1. Compare all trees in a file
   `necom eval compare trees.nwk`

2. Compare trees between two files
   `necom eval compare set1.nwk set2.nwk`
