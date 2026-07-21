# clust dbscan 规划

> **实现状态注记**：本文档列出 `necom clust dbscan` 尚未实现的规划功能（`--scan`/`--opt-eps`/`--min-pct` 等）。当前已实现基本的 `--eps`/`--min-points` 密度聚类，以及代表点选择（`--rep medoid|first`）。
> 截至 2026-07-21，`--scan`、`--opt-eps`、`--min-pct` 仍未实现。距离矩阵版 Silhouette 与 Davies–Bouldin 已由 `necom eval partition --matrix` / `--tree` 提供（坐标版在 `eval partition --coords` 中可用）。

## 1. 扫描与评分

- 新增 `--scan <start>,<end>,<steps>`：参考 `cut --scan` 风格，对主参数 `eps` 进行扫描，输出 TSV 摘要表（不绘图）
- 评分列（可选）：
  - `Silhouette`（距离矩阵版，复用 `libs::eval::silhouette_score`）
  - `DBIndex`（Davies–Bouldin 距离矩阵版，待实现）
- 建议输出列：
  - `Epsilon`, `Clusters`, `Noise`, `Silhouette`, `DBIndex`
- 自动选优（可选）：`--opt-eps {silhouette|max-clusters|min-noise}`，在扫描后直接选点并输出该分区

**复用已有指标**：扫描功能实现时应优先复用 `libs::eval` 中的距离矩阵指标（Silhouette、Dunn、C-index 等），避免在 `clust` 内部重复实现。

**用法规划**：
`necom clust dbscan ... --scan <start>,<end>,<steps>`
（注：扫描仅针对方法的**主阈值参数**，此处为 `eps`。例如 `min_points` 保持固定为用户指定值或默认值）

**输出指标表（示例）**：
| Epsilon | Clusters | Noise | Silhouette | DBIndex |
| :--- | :--- | :--- | :--- | :--- |
| 0.10 | 25 | 12 | 0.42 | 1.85 |
| 0.12 | 18 | 20 | 0.47 | 1.72 |
| ... | ... | ... | ... | ... |

## 2. 代表点与比例参数

- 代表点（`cluster`/`pair` 通用）：已实现 `--rep medoid|first`。`medoid` 为簇内平均距离最小点；`first` 为首个发现的点。
- 新增 `--min-pct <0..1>`：按样本比例折算为 `min_points`，与 `--min-points` 二选一（规划中）

## 3. 评分实现（距离矩阵版本）

- Silhouette
  - 对每个点 i：簇内平均距离 `a(i)`；到其它簇的最小平均距离 `b(i)`；`s(i) = (b-a)/max(a,b)`；总体取平均。
  - 状态：已实现于 `libs::eval::silhouette_score`，可通过 `necom eval partition --matrix` 使用。
- DBIndex（Davies–Bouldin）
  - 坐标版已实现于 `libs::eval::davies_bouldin_score`（`eval partition --coords`）；距离矩阵版已实现于 `libs::eval::distance::davies_bouldin_score`（`eval partition --matrix` / `--tree`）。
  - **距离矩阵版实现方案**：
    1. 在 `libs/eval/distance.rs` 中新增 `davies_bouldin_score(partition, dist_mat)`。
    2. 以 **medoid** 替代 centroid：对每个簇，选取簇内平均距离最小的样本作为代表点。
    3. `scatter_i` = 簇内所有点到 medoid_i 的平均距离。
    4. `d(c_i, c_j)` = medoid_i 与 medoid_j 之间的距离。
    5. `R_ij = (scatter_i + scatter_j) / d(c_i, c_j)`，取每个簇的最大 `R_ij` 后平均。
    6. 边界：若 `d(c_i, c_j) == 0` 且 scatter 之和为 0，则 `R_ij = 0`；否则使用大数代理（与坐标版 `DB_INFINITY_PROXY` 一致）。
    7. 复杂度：`O(K · n_k²)` 计算 medoid + `O(K²)` 计算 medoid 间距离。
  - **命名冲突处理**：`coordinates.rs` 中已存在同名 `davies_bouldin_score`。`distance.rs` 中的实现不在 `mod.rs` 顶层 re-export，改由 `format.rs` 通过全路径 `super::distance::davies_bouldin_score` 调用；`DISTANCE_METRIC_NAMES` 增加 `davies_bouldin` 列。
  - 用途：供 `scan-dbscan` 评分列，同时让 `eval partition --matrix` / `--tree` 支持 DBI。
- 位置建议：新增距离矩阵版 DBIndex 应放入 `libs::eval`，供扫描与 `eval partition --matrix` 复用。

## 4. 互操作与职责分离

- 算法侧（本命令）：负责 DBSCAN 聚类与扫描 TSV 输出
- 评估侧（`eval partition`）：外部有效性（`ARI/AMI/V-Measure`）与内部有效性（`Silhouette/Dunn/C-index` 等）均已支持；扫描评分应优先复用这些实现
- 与树工具协作：不直接涉及 `nwk`，但输出的 `cluster/pair` 可用于后续评估或可视化

## 5. 性能与边界

- 复杂度
  - DBSCAN：从距离矩阵出发，整体约 `O(N^2)`；扫描为 `steps × O(N^2)`
  - 评分计算也需 `O(N^2)`（平均距离/中心距）
- 建议
  - 缩小 `start,end`：可据距离分布的分位数设范围（如 `p10..p90`）
  - 合理的 `steps`（默认 100），规模较大时降低分辨率
  - 清晰文档提示中大型数据的计算成本

## 6. 测试计划

- 单元测试
  - 距离矩阵版 Silhouette/DBIndex 的正确性（小矩阵、可手算）
  - `--min-pct` 与 `--min-points` 的互斥与折算
  - 噪声计数与聚类数的统计正确性
- 集成测试
  - `--scan` 输出 TSV 字段与排序一致性
  - `--opt-eps silhouette` 在简单数据上的选点合理性
- Fuzz
  - 随机小矩阵，验证扫描输出与聚类稳定性

## 7. 使用示例（规划）

```bash
# 1) 基本聚类（pairwise 距离输入）
necom clust dbscan pairs.tsv --eps 0.15 --min-points 3 -o clusters.tsv

# 2) 扫描 eps 并输出评分曲线（TSV）
necom clust dbscan pairs.tsv --scan 0.05,0.5,100 -o scan.tsv

# 3) 自动基于 Silhouette 选优 eps 并直接输出分区
necom clust dbscan pairs.tsv --scan 0.05,0.5,100 --opt-eps silhouette -o best.tsv

# 4) 使用比例表达 min_points
necom clust dbscan pairs.tsv --eps 0.15 --min-pct 0.02 -o clusters.tsv

# 5) 输出 pair 格式，便于后续评估
necom clust dbscan pairs.tsv --eps 0.15 --min-points 3 --format pair -o pairs.out.tsv
```
