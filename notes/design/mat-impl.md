# mat 模块内部数据结构

> **实现状态注记**：本文档记录 `necom mat` 与 `necom clust` 内部使用的三种矩阵数据结构，从 `docs/mat.md` 迁移而来，供开发者参考。

## 1. ScoringMatrix

- **用途**：稀疏或按需计算的评分/距离矩阵。
- **底层存储**：`HashMap<(usize, usize), T>`。
- **特点**：稀疏存储，仅保留显式设置的值；支持对角线和 non-diagonal 的默认值；逻辑对称（`get(i,j)` 等价于 `get(j,i)`）。

## 2. CondensedMatrix

- **用途**：高效层次聚类（如 `clust hier`），支持较大规模数据。
- **底层存储**：`Vec<f32>`，仅存上三角（不含对角线），内存占用 $N(N-1)/2$。
- **索引映射**：对于 $(i, j)$ 且 $i < j$ → $k = N \cdot i - i(i+1)/2 + (j - i - 1)$。
- **特点**：强制对称，对角线假定为 0，不存储名称映射，纯数值计算。

## 3. NamedMatrix

- **用途**：带行列名的稠密距离矩阵（如 `PHYLIP` 的内存表示）。
- **底层存储**：`IndexMap`（名称索引）+ `CondensedMatrix` + 可选对角向量。
- **特点**：组合包装器，通过名称索引访问底层 `CondensedMatrix`；支持可选对角存储（`mat transform --normalize` 需要）；$N=10{,}000$ 时约 200MB。

## 4. 参见

- [docs/mat.md](../../docs/mat.md)：用户面向的矩阵格式说明（PHYLIP / Pairwise）。
- [clust-impl.md](clust-impl.md)：聚类模块实现分析，其中 §1 描述了三种结构在不同命令中的使用。
