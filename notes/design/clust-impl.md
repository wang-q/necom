# clust 模块实现分析

> **实现状态注记**：本文档记录 `necom clust` 模块的实现分析、优化路线与外部生态对比，从
> `docs/clust.md` 剥离而来。截至 2026-07-21，Phase 1/2/5/6/7 及 Ward/Centroid 平方距离优化、In-place
> 接口已完成；Phase 3/4(Heap) 及 GMM/HDBSCAN/Louvain/Leiden 仍为规划。评估侧（`necom eval`）
> 已实现 `eval partition`（含 `--matrix/--tree/--coords/--other` 目标）、`eval compare`（树拓扑
> RF/WRF/KF）、`eval replicate`（自 `nwk support` 迁移）；多维 `eval tree`（Cophenetic 拟合度、
> 性状纯度等）仍为规划，详见 [eval-planned.md](eval-planned.md)。

## 1. 内存数据布局

根据输入数据的特性和算法需求，`necom` 采用三种不同的内存布局策略。

### 1.1 构树类

- **命令**：`hier`, `upgma`, `nj`
- **输入**：PHYLIP 矩阵 (Dense)
- **加载数据结构**：`NamedMatrix` (内部封装 `CondensedMatrix`)
- **算法接口**：`hier::linkage` / `hier::linkage_with_algo` 已改为泛型，接收任意 `MatrixView<f32>`；
  内部将视图拷贝为 `CondensedMatrix` 后执行 NN-chain / Primitive 算法。`hier` 命令仍通过
  `NamedMatrix::into_parts()` 直接取得 `CondensedMatrix` 并调用 `linkage_inplace`，保持零拷贝。
- **特点**：
    - **全连接/稠密 (Dense)**：存储上三角矩阵，内存占用 $O(N^2)$。
    - **内存瓶颈**：当 $N=100k$ 时，`f32` 矩阵需占用约**18.6 GiB**内存。
      这是单机内存处理全连接矩阵的实用极限。
    - **原因**：PHYLIP 格式本身就是全矩阵格式，且传统构树算法基于全距离矩阵。

### 1.2 扁平聚类

- **命令**：`k-medoids`, `mcl`, `dbscan`
- **输入**：Pair Scores TSV (Sparse-like)
- **加载数据结构**：`NamedMatrix::from_pair_scores` 直接构建底层 `CondensedMatrix`（含对角线向量），
  避免经过 `ScoringMatrix` 造成内存双份。
- **算法接口**：`MatrixView` trait 统一了 `ScoringMatrix` 与 `NamedMatrix` 的只读访问；DBSCAN /
  K-Medoids / MCL 均通过 `&dyn MatrixView<f32>` 类似的泛型接口接收矩阵。
- **特点**：
    - **加载阶段**：`NamedMatrix` 以稠密上三角 `Vec<f32>` 存储，配合 `IndexMap` 名称索引，适合 Pair
      TSV 展开后的大部分场景。
    - **算法阶段**：通过 `MatrixView` 屏蔽底层实现，保持原有 $O(N^2)$ 访问语义。
    - **适用性**：对极稀疏输入，`ScoringMatrix`（`HashMap<usize, f32>`，压缩 key）仍可作为
      `MatrixView` 的实现按需使用。

### 1.3 图连通分量

- **命令**：`cc`
- **输入**：Pair TSV (Graph edges)
- **数据结构**：`petgraph::graphmap::UnGraphMap`
- **特点**：
    - **稀疏图 (Sparse Graph)**：基于邻接表/图结构，内存效率高。
    - **适用性**：专注于图拓扑结构分析，适合超大规模网络。

## 2. DBSCAN 实现特点

`necom clust dbscan` 基于 `MatrixView` 泛型接口做朴素 $O(N^2)$ 距离遍历；`region_query`
对当前输入矩阵做线性扫描。这种设计代码简洁，无需额外空间索引库，并输出代表点对等生物学便利功能。

**当前局限与未来方向**：缺乏空间索引，在大规模（> 1 万点）或高维数据上性能不如基于 BallTree/KDTree
的实现。未来若需处理大规模向量输入，可考虑引入 R-tree/KD-tree 或并行化邻域搜索。

## 3. 层次聚类实现对比

本节说明 `necom` 内部三种构树/层次聚类实现的分工与取舍。

- **UPGMA**：动态维护距离矩阵（HashMap），每次迭代寻找最小值，复杂度 $O(N^3)$。直接输出有根树，
  适合分子钟假设场景。详见 [upgma.rs](../../src/libs/clust/upgma.rs)。
- **NJ**：经典实现，输出有根树（在最后一条边中点定根）。适合一般加性距离。详见
  [nj.rs](../../src/libs/clust/nj.rs) doc comment。
- **`clust hier`**：更底层的通用统计聚类引擎。
    - **输入**：`CondensedMatrix` (压缩上三角矩阵，节省 50% 内存)。
    - **输出**：`Vec<Step>` (Linkage Matrix)，记录合并步骤，不直接生成 Tree 对象。
    - **实现**：已实现**NN-chain**算法，时间复杂度优化至 $O(N^2)$，且对 Ward 方法进行了平方距离优化。
      支持 In-place 操作以减少内存复制。
    - **与 UPGMA 的关系**：`hier` 是更底层的通用计算引擎；但 `upgma` 作为一个独立、
      直观且生物学语义明确的实现将被**长期保留**，作为算法学习和基准参考。

**未来方向**：探索针对超大规模数据的近似算法（如 Representative Strategy 已被推荐）。

## 4. clust hier 关键设计决策

本节记录 `clust hier` 在设计与实现过程中的关键决策，包括从外部实现中借鉴的优化方向和尚在规划中的指标。

1. **Generic Clustering Algorithm (Heap)**:
    - NN-Chain 仅适用于可归约方法（Ward, Average, Complete, Single, Weighted），无法处理 Centroid
      和 Median。未来计划引入基于 Binary Heap 的通用算法，将 Centroid/Median 从 $O(N^3)$ 降至
      $O(N^2 \log N)$。详见 §5.4 Phase 4。
2. **Ward 平方距离优化 (已完成)**:
    - `necom` 采用全程平方距离运算（Internal Squared Euclidean），仅在最终输出时开方。
      这避免了中间步骤的精度损失和 `sqrt` 开销，使得 Ward 方法的性能与 Average 方法完全持平。
3. **生态一致性**:
    - `necom cut` 的设计与 SciPy `fcluster` 的 `criterion='distance'|'maxclust'` 保持概念一致。
    - `necom mat compare` 已提供距离矩阵间相似度指标（Pearson/Spearman/MAE/Cosine/Jaccard/Euclid）。
    - Cophenetic Correlation 仍计划在 `necom eval tree` 中实现，用于量化树对原始距离矩阵的拟合优度。
4. **Optimal Leaf Ordering (OLO)**:
    - 已作为 `necom nwk order --olo` 实现，算法见 [`phylo.md`](phylo.md) §4「已实现」。
5. **Distance Metric Architecture**:
    - 计划参考 `DistanceMetric` 类设计，统一距离计算接口（由 `necom mat` 与 `necom eval partition`
      等命令复用），并在未来支持稀疏距离矩阵计算（Phase 3）。

## 5. clust hier 实现规划与优化分析

### 5.1 核心数据结构优化

- **Heap (堆) - Generic Clustering Algorithm**：
    - 适用所有方法，特别是不可归约的**Centroid**和**Median**。通过 Binary Heap 维护最近邻，
      目标复杂度 $O(N^2 \log N)$。作为 Phase 4 实施。
- **MST (最小生成树)**：
    - 适用**Single Linkage**。Single Linkage 等价于 MST；MST 在稀疏图输入上优势更明显，
      稠密矩阵下当前 NN-Chain 已足够高效。详见 §5.4 Phase 4 第 5 条。

### 5.2 空间与时间复杂度权衡

- **稠密矩阵 (Dense Matrix)**：
    - 现状：`necom` 目前主要处理 PHYLIP 距离矩阵，属于稠密矩阵。
    - 策略：对于 $N < 10,000$，朴素的 $O(N^2)$ 存储和 $O(N^3)$ 计算是可接受的（且利于 SIMD 优化）。
    - 优化：对于更大规模，必须避免全矩阵存储。
- **稀疏/受限连接 (Connectivity Constraints)**：
    - 场景：图像像素聚类或基于 KNN 图的聚类。
    - `necom` 规划：未来可支持从 `pair.tsv`（稀疏边列表）直接构建 Linkage，而不强制转为全距离矩阵，
      从而支持超大规模序列聚类。

### 5.3 现有 Rust 生态参考

`necom` 未引入外部层次聚类 crate；`hier` 的 NN-chain 实现针对 `CondensedMatrix` 与稀疏输入场景自研。
实现过程中参考了 `linfa-hierarchical` 的参数校验（`ParamGuard`）和从 Stepwise Dendrogram 到 Flat
Clusters 的后处理逻辑（`clusters` HashMap 维护）。

### 5.4 阶段性实现路线

#### Phase 1：MVP — **已完成**

基于 `CondensedMatrix` 实现 7 种 Linkage 方法（Single, Complete, Average, Weighted, Centroid,
Median, Ward），采用 Lance-Williams 更新，$O(N^3)$ 时间。

#### Phase 2：性能优化（NN-chain）— **已完成**

对可归约方法（Ward, Average, Complete, Weighted, Single）实现 NN-chain 算法，时间复杂度降至 $O(N^2)
$。`linkage` 自动按方法选择 NN-chain 或 Primitive。配合 Ward 平方距离优化后，Ward 与 Average
性能持平。

**Benchmark Results (Average & Ward):**

| N   | Method  | Primitive $O(N^3)$ | NN-Chain $O(N^2)$ | Speedup |
|-----|---------|--------------------|-------------------|---------|
| 100 | Average | ~300 µs            | ~63 µs            | ~4.7x   |
| 200 | Average | ~2.1 ms            | ~248 µs           | ~8.5x   |
| 400 | Average | ~15.6 ms           | ~975 µs           | ~16x    |
|     |         |                    |                   |         |
| 100 | Ward    | ~315 µs            | ~70 µs            | ~4.5x   |
| 200 | Ward    | ~2.3 ms            | ~266 µs           | ~8.6x   |
| 400 | Ward    | ~15.8 ms           | ~1.0 ms           | ~15.8x  |

#### Phase 3：大规模数据策略（两阶段/代表点）— **推荐**

参见 `docs/clust.md` 中的"大规模数据策略"章节。

#### Phase 4：性能与正确性优化

1. **Generic Clustering Algorithm (Heap)**:
    - 目标：优化**Centroid**和**Median**方法。
    - 方案：引入 Binary Heap 维护最近邻距离，将这两个方法的复杂度从 $O(N^3)$ 降至 $O(N^2 \log N)$。
    - 状态：**未实现**（截至 2026-07-21）。
    - 优先级：中（除非用户有大量 Centroid/Median 聚类需求）。
2. **Ward/Centroid 平方距离优化 (已完成)**.
3. **In-place 接口 (已完成)**.
4. **Chain 循环优化 (已分析，暂不实施)**：Condensed Matrix 顺序访问对缓存友好，引入 ActiveList
   对中小规模数据收益有限，待 profiling 确认后再决定。
5. **MST 算法 (已分析，暂不实施)**：Single Linkage 等价于 MST，但稠密矩阵下 NN-Chain 已足够高效；
   MST 优势主要在稀疏图输入（Phase 3 范畴）。

#### Phase 5：测试覆盖率增强（已完成）

已实现：NN-chain/Primitive 一致性 fuzzing、单调性检查（除 Centroid/Median）、极小输入边界 ($N=0,1,
2$)。

#### Phase 6：基准测试增强（已完成）

已验证 NN-chain 在 $N=1000 \sim 4000$ 范围的 $O(N^2)$ 扩展性，以及 Ward 与 Average 性能曲线重合。
$N=4000$ 耗时约 0.18s。

**最新 Benchmark 数据 (Average & Ward):**

| N    | Primitive $O(N^3)$ | NN-Chain $O(N^2)$ |
|------|--------------------|-------------------|
| 100  | ~0.3 ms            | ~0.06 ms          |
| 400  | ~16 ms             | ~0.9 ms           |
| 1000 | (未测)             | ~5.3 ms           |
| 2000 | (未测)             | ~29.0 ms          |
| 4000 | (未测)             | ~174 ms           |

#### Phase 7：真实分布与效果验证（已完成）

`tests/cli_clust_pipeline.rs` 中的 blobs 集成测试生成 3 个 2D 高斯分布簇，经 `hier --method ward`
与 `cut` 后，用 `eval partition` 对比 ground truth，ARI ≈ 1.0。

## 6. clust hier 内部实现细节

- `ward` 更新采用全程平方距离运算（仅输出时开方），避免了中间步骤的精度损失和开方开销，使得 Ward 与
  Average 方法性能持平。
- 非 NN-chain 方法（Centroid/Median）目前使用朴素 $O(N^3)$ 实现，未来可引入 Heap 优化（详见 §5.4
  Phase 4）。

## 7. 实现路线图

1. **基础图聚类**：已完成 MCL、CC、DBSCAN、K-Medoids。
2. **系统发育构树**：已完成 UPGMA、NJ、Hierarchical Clustering (hier)。
3. **评估体系**：`eval partition` 已完成（从 `clust eval` 迁移；支持 `--matrix`/`--tree`/`--coords`/
   `--other` 多种目标）；`eval compare` 已完成（树拓扑 RF/WRF/KF 距离，自 `nwk compare` 迁移）；
   `eval replicate` 已完成（自 `nwk support` 迁移）。多维 `eval tree`（Cophenetic 拟合度、性状纯度、
   参考树对比等）仍为规划，详见 [eval-planned.md](eval-planned.md)。
4. **向量支持**：已完成。`libs/feature.rs` 提供 `FeatureVector` 基础设施，被
   `necom eval partition --coords`（Davies-Bouldin、Calinski-Harabasz、PBM、Ball-Hall、Xie-Beni、
   Wemmert-Gancarski 等指标）复用。
5. **统计聚类**：引入 GMM 实现，支持 BIC 模型选择（计划中，详见 `docs/clust.md` 的 Planned 章节）。
6. **层次聚类扩展**：实现 HDBSCAN（计划中，详见 §8.1）。
7. **大规模网络社区发现**：实现 Louvain / Leiden 算法（计划中，详见 §8.2）。

## 8. 计划中的算法

> **实现状态注记**：本节记录尚未进入实现阶段、且 `docs/clust.md` 未详细展开的聚类算法规划。GMM
> 的用户面向动机、计划接口与 BIC 模型选择已写在 `docs/clust.md` 的 Planned 章节，此处不再重复。截至
> 2026-07-21，HDBSCAN、Louvain/Leiden 仍为规划，未进入实现阶段。

### 8.1 HDBSCAN

- **原理**：结合层次聚类与 DBSCAN。通过构建基于密度的层次树（Condensed Tree），并根据簇的稳定性
  （Stability）在不同层级自动提取最佳簇，无需全局 $\epsilon$。
- **命令**：`necom clust hdbscan`
- **scikit-learn 对应**：`HDBSCAN`
- **计划内容**：层次化 DBSCAN，无需手动指定 `eps`。
- **价值**：DBSCAN 的现代高级版，**自动适应不同密度的簇**，参数更少且更稳健。

### 8.2 Louvain / Leiden

- **原理**：基于模块度（Modularity）优化的社区发现算法。Louvain 贪心地最大化模块度；Leiden 改进了
  Louvain 的局部合并策略，保证连通性并加速收敛。
- **命令**：(待定)
- **计划内容**：社区发现算法。
- **价值**：比 MCL 更适合**超大规模网络**的层次化结构探索。

## 9. 不引入的算法（Not Recommended / No Plans）

不计划引入的算法及其替代方案已迁移至 `docs/clust.md` 的 "Why Some Common Algorithms Are Not
Provided" 章节，此处不再重复。核心原则：这些算法在大规模生物数据场景下存在限制（如依赖欧氏空间、
计算复杂度过高、或聚焦与当前模块不同的使用场景），`necom` 优先使用 K-Medoids、MCL、DBSCAN、HDBSCAN
（规划）等替代。

