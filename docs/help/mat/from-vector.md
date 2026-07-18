Calculate pairwise similarity/distance between vectors in input file(s).

Behavior:

* `--mode euclid`: Euclidean distance.
* `--mode euclid --sim`: Euclidean distance converted to similarity.
* `--mode euclid --binary`: binary Euclidean distance (values treated as 0/1).
* `--mode euclid --binary --sim --dis`: binary Euclidean distance to dissimilarity.
* `--mode cosine`: cosine similarity (-1 to 1).
* `--mode cosine --dis`: cosine distance (0 to 2).
* `--mode cosine --binary`: binary cosine similarity.
* `--mode cosine --binary --dis`: binary cosine distance.
* `--mode jaccard --binary`: Jaccard index.
* `--mode jaccard`: weighted Jaccard similarity.

Input:

* One or two vector files in `name<tab>v1<tab>v2<tab>...` format (pure TSV).
* One file: self-comparison (all pairs including diagonal).
* Two files: cross-comparison between the two sets.

Output:

* Three-column TSV: `name1<tab>name2<tab>score`, one row per pair.

Notes:

* `--binary` treats non-zero values as 1 before computing the score.
* `--sim` converts a distance to a similarity; `--dis` converts a similarity to a dissimilarity.
* `--parallel <N>` sets the number of worker threads (default 1).

Examples:

1. Self-compare vectors with binary Jaccard
   `necom mat from-vector vectors.tsv --mode jaccard --binary`

2. Cross-compare two vector sets with cosine distance
   `necom mat from-vector set1.tsv set2.tsv --mode cosine --dis -o out.tsv`
