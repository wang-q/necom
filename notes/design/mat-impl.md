# mat 模块内部数据结构

> **实现状态注记**：本文档记录 `necom mat` 与 `necom clust` 内部使用的三种矩阵数据结构，从 `docs/mat.md` 迁移而来，供开发者参考。

## 1. ScoringMatrix

- **用途**：稀疏或按需计算的评分/距离矩阵。
- **底层存储**：`HashMap<usize, T>`，key 为上三角（含对角线）的压缩线性索引 `i * N - i(i-1)/2 + (j-i)`（`i <= j`）。
- **特点**：稀疏存储，仅保留显式设置的值；单 `usize` key 比 `(usize, usize)` tuple 更省内存；支持对角线和 non-diagonal 的默认值；逻辑对称（`get(i,j)` 等价于 `get(j,i)`）。

## 2. CondensedMatrix

- **用途**：高效层次聚类（如 `clust hier`），支持较大规模数据。
- **底层存储**：`Vec<f32>`，仅存上三角（不含对角线），内存占用 $N(N-1)/2$。
- **索引映射**：对于 $(i, j)$ 且 $i < j$ → $k = N \cdot i - i(i+1)/2 + (j - i - 1)$。
- **特点**：强制对称，对角线假定为 0，不存储名称映射，纯数值计算。

## 3. NamedMatrix

- **用途**：带行列名的稠密距离矩阵（如 `PHYLIP` 的内存表示）。
- **底层存储**：`IndexMap`（名称索引）+ `CondensedMatrix` + 可选对角向量。
- **特点**：组合包装器，通过名称索引访问底层 `CondensedMatrix`；支持可选对角存储（`mat transform --normalize` 需要）；$N=10{,}000$ 时约 200MB。

## 4. 公共 API

`pairmat` 模块通过 `mod.rs` 的 `pub use` 暴露以下函数与类型（按职责分组）：

- **数据结构**：
  - `CondensedMatrix`、`NamedMatrix`、`ScoringMatrix<T>`：见 §1-§3。
  - `get_condensed_index(size, row, col)`：上三角线性索引计算（`condensed.rs`）。

- **I/O 与格式**：
  - `MatrixFormat`：`Full` / `Lower` / `Strict` 三种 PHYLIP 输出变体（`output.rs`）。
  - `MatrixFormat::from_mode(s)`：从字符串（`"full"` / `"lower"` / `"strict"`）解析。
  - `write_phylip_matrix(m, fmt, precision, writer)`：按指定格式写出 PHYLIP 矩阵。
  - `write_subset(m, names, precision, writer)`：按给定名称子集写出子矩阵，返回缺失名称列表；`precision` 控制小数位数。
  - `extract_common_lower_triangle(m1, m2)`：抽取两个矩阵公共名称的下三角值对，供 `mat compare` 使用。
  - `NamedMatrix::from_relaxed_phylip(infile)`：从 Relaxed PHYLIP 文件加载。
  - `NamedMatrix::from_pair_scores(infile, same, missing)`：从 Pairwise TSV 直接构建底层 `CondensedMatrix`（自对角默认 `same`，缺失对默认 `missing`），避免经过 `ScoringMatrix` 造成内存双份。

- **变换**：
  - `transform_matrix(matrix, method, max_val, scale, offset, normalize)`：元素级数学变换（`transform.rs`），支持 `linear` / `inv-linear` / `log` / `exp` / `square` / `sqrt`，可选按对角线归一化。

## 5. 参见

- [docs/mat.md](../../docs/mat.md)：用户面向的矩阵格式说明（PHYLIP / Pairwise）。
- [clust-impl.md](clust-impl.md)：聚类模块实现分析，其中 §1 描述了三种结构在不同命令中的使用。

## 6. 已知死代码

以下公共 API 经全仓库 grep 确认无生产代码引用（仅自身 doc test 或完全无引用），按项目规则保留不删除：

- `NamedMatrix::with_ids(size)`（`named.rs`）：创建数字命名矩阵，无调用方。
- `NamedMatrix::matrix()`（`named.rs`）：返回底层 `CondensedMatrix` 引用，无调用方（调用方使用 `values()` 或 `into_parts()`）。
- `NamedMatrix::new_from_values(names, values)`（`named.rs`）：从名称和上三角值创建，无调用方（调用方使用 `from_relaxed_phylip` 或 `from_pair_scores`）。
- `ScoringMatrix::with_size(size)`（`scoring.rs`）：创建指定大小的空矩阵，无调用方（调用方使用 `with_size_and_defaults`）。
