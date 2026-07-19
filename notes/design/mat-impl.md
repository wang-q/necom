# mat 模块内部数据结构

> **实现状态注记**：本文档记录 `necom mat` 与 `necom clust` 内部使用的矩阵数据结构的开发者视角细节（公共 API、死代码等）。三种数据结构的概述、用途与索引公式见 [`docs/mat.md`](../../docs/mat.md)。

## 1. 数据结构概述

三种矩阵数据结构的用途、存储布局与索引公式已迁移到用户文档 [`docs/mat.md`](../../docs/mat.md) 的 **Internal Data Structures** 小节。本文档不再重复，仅保留开发者视角的公共 API 清单与实现注记。

## 2. 公共 API

`pairmat` 模块通过 `mod.rs` 的 `pub use` 暴露以下函数与类型（按职责分组）：

- **数据结构**：
  - `CondensedMatrix`、`NamedMatrix`、`ScoringMatrix<T>`：概述见 [`docs/mat.md`](../../docs/mat.md)。
  - `MatrixView<T>`：只读矩阵视图 trait，`ScoringMatrix` 与 `NamedMatrix` 均实现此接口，供聚类算法统一访问（`mod.rs`）。
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

## 3. 参见

- [docs/mat.md](../../docs/mat.md)：用户面向的矩阵格式说明（PHYLIP / Pairwise）。
- [clust-impl.md](clust-impl.md)：聚类模块实现分析，其中 §1 描述了三种结构在不同命令中的使用。

## 4. 已知死代码

以下公共 API 经全仓库 grep 确认无生产代码引用（仅自身 doc test 或完全无引用），按项目规则保留不删除：

- `NamedMatrix::with_ids(size)`（`named.rs`）：创建数字命名矩阵，无调用方。
- `NamedMatrix::matrix()`（`named.rs`）：返回底层 `CondensedMatrix` 引用，无调用方（调用方使用 `values()` 或 `into_parts()`）。
- `NamedMatrix::index()`（`named.rs`）：返回 `(row, col)` 的线性压缩索引，无调用方。
- `ScoringMatrix::with_size(size)`（`scoring.rs`）：创建指定大小的空矩阵，无调用方（调用方使用 `with_size_and_defaults`）。

注：`NamedMatrix::new_from_values(names, values)` 被 `src/libs/tree_cut/mod.rs` 用于构建树切分距离矩阵，不属于死代码。
