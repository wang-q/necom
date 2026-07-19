# clust 模块实现分析

> **实现状态注记**：本文档记录 `necom clust` 模块的实现分析、优化路线与外部生态对比，从 `docs/clust.md` 剥离而来。
> 截至 2026-07-19，Phase 1/2/5/6/7 及 Ward/Centroid 平方距离优化、In-place 接口已完成；Phase 3/4(Heap) 及 GMM/HDBSCAN/Louvain/Leiden 仍为规划。

## 1. 内存数据布局

根据输入数据的特性和算法需求，`necom` 采用三种不同的内存布局策略。

### 1.1 构树类
- **命令**：`hier`, `upgma`, `nj`
- **输入**：PHYLIP 矩阵 (Dense)
- **加载数据结构**：`NamedMatrix` (内部封装 `CondensedMatrix`)
- **算法接口**：`hier::linkage` / `hier::linkage_with_algo` 已改为泛型，接收任意 `MatrixView<f32>`；内部将视图拷贝为 `CondensedMatrix` 后执行 NN-chain / Primitive 算法。`hier` 命令仍通过 `NamedMatrix::into_parts()` 直接取得 `CondensedMatrix` 并调用 `linkage_inplace`，保持零拷贝。
- **特点**：
  - **全连接/稠密 (Dense)**：存储上三角矩阵，内存占用 $O(N^2)$。
  - **内存瓶颈**：当 $N=100k$ 时，`f32` 矩阵需占用约 **18.6 GiB** 内存。这是单机内存处理全连接矩阵的实用极限。
  - **原因**：PHYLIP 格式本身就是全矩阵格式，且传统构树算法基于全距离矩阵。

### 1.2 扁平聚类
- **命令**：`k-medoids`, `mcl`, `dbscan`
- **输入**：Pair Scores TSV (Sparse-like)
- **加载数据结构**：`NamedMatrix::from_pair_scores` 直接构建底层 `CondensedMatrix`（含对角线向量），避免经过 `ScoringMatrix` 造成内存双份。
- **算法接口**：`MatrixView` trait 统一了 `ScoringMatrix` 与 `NamedMatrix` 的只读访问；DBSCAN / K-Medoids / MCL 均通过 `&dyn MatrixView<f32>` 类似的泛型接口接收矩阵。
- **特点**：
  - **加载阶段**：`NamedMatrix` 以稠密上三角 `Vec<f32>` 存储，配合 `IndexMap` 名称索引，适合 Pair TSV 展开后的大部分场景。
  - **算法阶段**：通过 `MatrixView` 屏蔽底层实现，保持原有 $O(N^2)$ 访问语义。
  - **适用性**：对极稀疏输入，`ScoringMatrix`（`HashMap<usize, f32>`，压缩 key）仍可作为 `MatrixView` 的实现按需使用。

### 1.3 图连通分量
- **命令**：`cc`
- **输入**：Pair TSV (Graph edges)
- **数据结构**：`petgraph::graphmap::UnGraphMap`
- **特点**：
  - **稀疏图 (Sparse Graph)**：基于邻接表/图结构，内存效率高。
  - **适用性**：专注于图拓扑结构分析，适合超大规模网络。

## 2. DBSCAN 实现对比

- **scikit-learn 实现**：
  - **核心**：使用 `NearestNeighbors` 模块（基于 BallTree/KDTree）加速邻域查询。
  - **优化**：支持稀疏矩阵；通过 `n_jobs` 并行化；核心逻辑部分使用 Cython 加速。
  - **适用性**：能处理数百万量级的数据（如果维度不高）。
- **necom 实现**：
  - **核心**：基于 `ScoringMatrix` 的朴素 $O(N^2)$ 距离遍历；`region_query` 为线性扫描。
  - **特点**：代码简洁，无需额外空间索引库；输出包含"代表点对"等生物学便利功能。
  - **局限**：缺乏空间索引，在大规模（>1万点）或高维数据上性能不如 sklearn。
- **未来方向**：对于大规模向量输入，需引入空间索引（如 R-tree/KD-tree）或并行化邻域搜索。

## 3. 层次聚类 (UPGMA / NJ vs Agglomerative)

- **scikit-learn (AgglomerativeClustering)**：
  - **定位**：通用统计聚类，输出 Linkage Matrix（$N-1$ 次合并记录）。
  - **优化**：
    - 使用 **MST (最小生成树)** 加速 Single Linkage ($O(N^2)$)。
    - 使用 **Heap (堆)** 结构加速 Ward/Average/Complete Linkage 的最近邻查找。
  - **输出**：不直接生成 Newick 树，需转换。
- **necom (UPGMA / NJ)**：
  - **定位**：生物系统发育构树，直接输出 **Tree** 对象和 **Newick** 格式。
  - **实现**：
    - **UPGMA**：动态维护距离矩阵（HashMap），每次迭代寻找最小值，复杂度 $O(N^3)$。
    - **NJ**：经典的 Neighbor-Joining 实现，计算净发散度与 Q 矩阵，通过最后一条边的中点定根输出有根树（见 [nj.rs](../../src/libs/clust/nj.rs) doc comment）。
  - **优势**：逻辑直观，原生支持生物学所需的枝长计算与树操作。
  - **局限**：未采用 Heap 优化，在大规模数据（>5000 序列）上速度慢于优化过的 Linkage 算法。
- **necom (clust hier)**：
  - **定位**：通用统计聚类底层引擎，类似 SciPy/scikit-learn。
  - **输入**：`CondensedMatrix` (压缩上三角矩阵，节省 50% 内存)。
  - **输出**：`Vec<Step>` (Linkage Matrix)，记录合并步骤，不直接生成 Tree 对象。
  - **实现**：已实现 **NN-chain** 算法，时间复杂度优化至 $O(N^2)$，且对 Ward 方法进行了平方距离优化。支持 In-place 操作以减少内存复制。
  - **与 UPGMA 的关系**：`hier` 是更底层的通用计算引擎；但 `upgma` 作为一个独立、直观且生物学语义明确的实现将被**长期保留**，作为算法学习和基准参考。
- **未来方向**：探索针对超大规模数据的近似算法（如 Representative Strategy 已被推荐）。

## 4. clust hier SciPy 实现借鉴

通过深入分析 `scipy.cluster.hierarchy` 源码（基于 Cython 的高性能实现），`necom` 吸收了以下关键设计思想：

1.  **Generic Clustering Algorithm (Heap 优化)**:
    - **背景**: NN-Chain 算法仅适用于可归约方法（Ward, Average, Complete, Single, Weighted），无法处理 Centroid 和 Median。
    - **SciPy 方案**: 在 `fast_linkage` (`_hierarchy.pyx`) 中实现了 Daniel Müllner (2011) 的算法。该算法结合 `neighbor` 数组和 Binary Heap，将所有方法的复杂度统一优化至 $O(N^2 \log N)$ 甚至 $O(N^2)$。
    - **necom 借鉴**: 目前 `necom` 对 Centroid/Median 使用 $O(N^3)$ 朴素实现。未来计划移植该 Heap 算法，消除性能短板。

2.  **Ward 方法的数值稳定性与效率**:
    - **SciPy 实现**: `ward` 更新公式在内部计算时涉及平方和开方（`sqrt`），这在大量迭代中可能积累浮点误差，且计算开销较大。
    - **necom 优化**: `necom` 采用全程平方距离运算（Internal Squared Euclidean），仅在最终输出时开方。这避免了中间步骤的精度损失和 `sqrt` 开销，使得 Ward 方法的性能与 Average 方法完全持平（而在许多其他库中 Ward 通常更慢）。

3.  **生态一致性**:
    - **Flat Clustering**: `necom cut` 的设计与 SciPy `fcluster` 的 `criterion='distance'|'maxclust'` 保持概念一致。
    - **Cophenetic Correlation**: 确认将 `cophenet` 引入 `necom eval tree` [计划中]，作为衡量树对原始距离矩阵拟合优度的核心指标。

4.  **Optimal Leaf Ordering (OLO)**:
    - **背景**: 标准层次聚类算法生成的树，左右子树的顺序是任意的。这导致在绘制热图（Heatmap）时，相似的行/列可能不相邻，视觉效果杂乱。
    - **SciPy 方案**: `scipy.cluster.hierarchy.optimal_leaf_ordering`。
    - **算法**: Bar-Joseph et al. (2001) 的动态规划算法。在不改变树拓扑结构的前提下，通过旋转内部节点，最小化相邻叶子之间的距离之和。
    - **necom 借鉴**: 计划在 `necom nwk order` 中实现此功能（`--olo` 或 `--optimal`），作为聚类后的标准优化步骤，显著提升下游可视化（外部工具）的效果。

5.  **Cophenetic Correlation Coefficient**:
    - **背景**: 如何量化生成的树是否真实反映了原始距离矩阵？
    - **SciPy 方案**: `scipy.cluster.hierarchy.cophenet`。
    - **原理**: 计算树上两点间的距离（cophenetic distance，即最近共同祖先的高度）与原始距离矩阵之间的 Pearson 相关系数。
    - **necom 借鉴**: 在 `necom eval tree` [计划中] 中实现此指标，帮助用户评估不同 Linkage 方法（如 UPGMA vs Ward）对数据的拟合优度。

6.  **Distance Metric Architecture**:
    - **背景**: SciPy/Scikit-learn 的距离计算模块架构清晰，支持稀疏矩阵和多种度量。
    - **necom 借鉴**: 计划参考 `DistanceMetric` 类设计，统一距离计算接口（由 `necom mat` 与 `necom eval partition` 等命令复用），并在未来支持稀疏距离矩阵计算（Phase 3）。

## 5. clust hier 实现规划与优化分析

### 5.1 核心数据结构优化
- **Heap (堆) - Generic Clustering Algorithm**:
  - 适用：所有方法，特别是 **Centroid** 和 **Median**（不可归约，无法使用 NN-chain）。
  - 原理：维护一个距离最近邻的优先队列。这是 Daniel Müllner (2011) 提出的 "Generic Clustering Algorithm"。
  - SciPy 参考：`fast_linkage` in `_hierarchy.pyx`。
  - `necom` 规划：作为 Phase 4 的一部分，替换目前的 Primitive $O(N^3)$ 实现，统一所有方法的性能基线。
- **MST (最小生成树)**:
  - 适用：**Single Linkage** (最近邻)。
  - 原理：Single Linkage 聚类等价于求最小生成树（MST）。使用 Prim 或 Kruskal 算法可在 $O(N^2)$ (稠密) 或 $O(E \log E)$ (稀疏) 内完成，显著快于通用 Linkage 的 $O(N^3)$。
  - `scikit-learn` 参考：`sklearn/cluster/_agglomerative.py` 中的 `_single_linkage_tree` 函数。
- **Union-Find (并查集)**：
  - 配合 MST 使用，用于快速合并簇和标记标签。

### 5.2 空间与时间复杂度权衡
- **稠密矩阵 (Dense Matrix)**：
  - 现状：`necom` 目前主要处理 PHYLIP 距离矩阵，属于稠密矩阵。
  - 策略：对于 $N < 10,000$，朴素的 $O(N^2)$ 存储和 $O(N^3)$ 计算是可接受的（且利于 SIMD 优化）。
  - 优化：对于更大规模，必须避免全矩阵存储。
- **稀疏/受限连接 (Connectivity Constraints)**：
  - 场景：图像像素聚类或基于 KNN 图的聚类。
  - `scikit-learn` 特性：支持 `connectivity` 参数（稀疏矩阵），限制只有相邻节点可以合并。这能将复杂度从 $O(N^3)$ 降至 $O(N \log N)$ 甚至 $O(N)$。
  - `necom` 规划：未来可支持从 `pair.tsv`（稀疏边列表）直接构建 Linkage，而不强制转为全距离矩阵，从而支持超大规模序列聚类。

### 5.3 现有 Rust 生态参考
- **kodama**（[GitHub](https://github.com/m4b/kodama)）：
  - 实现了现代层次聚类算法（NN-chain），性能对标 `fastcluster`。
  - 核心接口 `linkage` 接受 Condensed Matrix（上三角压缩），输出 Stepwise Dendrogram。
  - 提供了完整的 `Method` 枚举（Single, Complete, Average, Ward 等）。
  - **决策**：`necom` 将参考 `kodama` 的 NN-chain 算法实现自己的逻辑，保持对核心数据结构的完全控制（如适配稀疏输入）。
  - **价值**：参考 `kodama` 的测试用例和基准测试思路来验证 `necom` 实现的正确性与性能。
- **linfa-hierarchical**（[GitHub](https://github.com/rust-ml/linfa)）：
  - 提供了符合 `linfa` 生态的 `Transformer` 接口。
  - 内部直接调用 `kodama`，并增加了对 Similarity Kernel 的支持（自动转为 Distance）。
  - **借鉴**：参考其清晰的参数校验（`ParamGuard`）和从 Stepwise Dendrogram 到 Flat Clusters 的后处理逻辑（`src/lib.rs` 中的 `clusters` HashMap 维护）。

### 5.4 阶段性实现路线

#### Phase 1：MVP — **已完成**
- **状态**：已在 `src/libs/clust/hier.rs` 中实现，并集成到 CLI `src/cmd_necom/clust/hier.rs`。
- **特性**：
  - 实现了基于 `CondensedMatrix`（上三角压缩矩阵）的存储，节省 50% 内存。
  - 实现了通用的 Lance-Williams 更新公式，支持 7 种 Linkage 方法：
    - `Single`, `Complete`, `Average` (UPGMA), `Weighted` (WPGMA), `Centroid` (UPGMC), `Median` (WPGMC), `Ward` (Ward's D2)。
  - 复杂度：$O(N^3)$ 时间，$O(N^2)$ 空间。
  - 验证：单元测试覆盖了核心算法，集成测试覆盖了 CLI 功能。

#### Phase 2：性能优化（NN-chain）— **已完成**
- **状态**：已在 `src/libs/clust/hier.rs` 中实现 NN-chain 算法。
- **特性**：
  - **算法**：NN-chain (Nearest-neighbor chain) 算法。
  - **适用性**：Ward, Average, Complete, Weighted (空间具有可还原性/Reducibility)。
  - **复杂度**：时间复杂度优化至 $O(N^2)$。
  - **自动调度**：`linkage` 函数自动根据 Method 选择最佳算法（Reducible 方法用 NN-chain，其它用 Primitive）。
  - **验证**：
    - 单元测试验证了 NN-chain 与 Primitive 算法输出的一致性（包括 ID 映射和拓扑）。
    - 基准测试证明了显著的性能提升。

**Benchmark Results (Average & Ward):**

| N | Method | Primitive $O(N^3)$ | NN-Chain $O(N^2)$ | Speedup |
|---|---|---|---|---|
| 100 | Average | ~300 µs | ~63 µs | ~4.7x |
| 200 | Average | ~2.1 ms | ~248 µs | ~8.5x |
| 400 | Average | ~15.6 ms | ~975 µs | ~16x |
| | | | | |
| 100 | Ward | ~315 µs | ~70 µs | ~4.5x |
| 200 | Ward | ~2.3 ms | ~266 µs | ~8.6x |
| 400 | Ward | ~15.8 ms | ~1.0 ms | ~15.8x |

*注：Ward Linkage 在优化后（平方距离更新）性能与 Average Linkage 几乎持平。*

#### Phase 3：大规模数据策略（两阶段/代表点）— **推荐**
参见 `docs/clust.md` 中的"大规模数据策略"章节。

#### Phase 4：性能与正确性优化
通过分析 `kodama`、`scikit-learn` 和 `scipy` 实现，确定以下优化方向：
1.  **Generic Clustering Algorithm (Heap)**:
    - 目标：优化 **Centroid** 和 **Median** 方法。
    - 方案：参考 SciPy 的 `fast_linkage` 实现（基于 Müllner 2011），引入 Binary Heap 维护最近邻距离。这将把这两个方法的复杂度从 $O(N^3)$ 降至 $O(N^2 \log N)$。
    - 状态：**未实现**（截至 2026-07-19）。
    - 优先级：中（除非用户有大量 Centroid/Median 聚类需求）。
2.  **Ward/Centroid 平方距离优化 (已完成)**:
    - 改进：在算法开始时一次性将距离矩阵平方，使用简化版 Lance-Williams 更新，仅在输出时开方。
    - 效果：消除了每次迭代中的 `sqrt` 调用，使得 Ward Linkage 的性能与 Average Linkage 持平（基准测试证实）。
3.  **In-place 接口 (已完成)**:
    - 引入 `linkage_inplace`，允许消耗输入的 `CondensedMatrix`（避免克隆），节省 $O(N^2)$ 内存复制开销。
4.  **Chain 循环优化 (已分析)**:
    - 分析：`kodama` 使用了高效的 `ActiveList` (双向链表) 来跳过非活跃节点。
    - 结论：虽然这能将寻找最近邻的复杂度从 $O(N)$ 降至 $O(K)$，但鉴于 Condensed Matrix 的顺序访问对 CPU 缓存非常友好，且当前实现在 $N=400$ 时仅需 ~1ms，引入链表的跳跃访问可能收益有限甚至负优化（对于中小规模数据）。
    - 决策：暂不实施，直至 profiling 显示 NN 搜索成为显著瓶颈。
5.  **MST 算法 (已分析)**:
    - 分析：`scikit-learn` 和 `kodama` 对 Single Linkage 使用 MST 算法。
    - 结论：对于稠密矩阵，NN-Chain 和 Prim MST 都是 $O(N^2)$。当前的 NN-Chain 实现通用且足够高效。MST 主要优势在于处理稀疏图输入（Phase 3 范畴）。
    - 决策：维持现状。

#### Phase 5：测试覆盖率增强（已完成）
参考 `kodama` 和 `scikit-learn` 的测试策略，已增加以下测试以提升稳健性：
1.  **Fuzzing / Randomized Testing (Kodama)**:
    - 目标：验证 NN-Chain 算法与 Primitive 算法在大量随机输入下的一致性。
    - 状态：已实现 `test_nn_chain_fuzzing`，循环测试 20 个不同大小（$N=10 \sim 105$）的随机矩阵，验证输出 Step 数量和合并距离的一致性（包括 Ward 方法）。
2.  **Monotonicity Check (Sklearn)**:
    - 目标：验证生成的 Dendrogram 是否满足单调性（除了 Centroid/Median 方法）。
    - 状态：已实现 `test_monotonicity`，断言所有单调方法的 `steps[i].distance <= steps[i+1].distance`。
3.  **Edge Cases (Kodama)**:
    - 目标：验证极小输入的处理 ($N=0, 1, 2$)。
    - 状态：已实现 `test_edge_cases`，确保空输入或单点输入正确返回空结果。

#### Phase 6：基准测试增强（已完成）
参考 `kodama` 和 `scikit-learn` 的基准测试策略，已实施了以下测试：
1.  **多尺度性能曲线 (Scalability)**:
    - 验证了 NN-Chain 算法在 $N=1000 \sim 4000$ 范围内的 $O(N^2)$ 扩展性。
    - **Ward** 与 **Average** 的性能曲线几乎重合，证明了平方距离优化的有效性。
    - $N=4000$ 时耗时约 0.18s，推算 $N=20000$ 时约需 5s，完全可接受。
2.  **方法间对比 (Method Comparison)**:
    - 在 $N=1000$ 时，所有方法（Single, Complete, Average, Weighted, Ward）的耗时高度一致（~6.0ms）。
    - 表明核心算法框架的效率主导了计算，具体距离公式的差异对性能影响微乎其微。

**最新 Benchmark 数据 (Average & Ward):**

| N | Primitive $O(N^3)$ | NN-Chain $O(N^2)$ |
|---|---|---|
| 100 | ~0.3 ms | ~0.06 ms |
| 400 | ~16 ms | ~0.9 ms |
| 1000 | (未测) | ~5.3 ms |
| 2000 | (未测) | ~29.0 ms |
| 4000 | (未测) | ~174 ms |

#### Phase 7：真实分布与效果验证（已完成）
参考 `linfa-hierarchical` 的 `test_blobs` 测试，已增加集成测试验证算法的统计有效性：
1.  **真实分布测试 (Blobs Test)**:
    - 目标：验证算法能否正确聚类具有明显几何结构的合成数据（Statistical Correctness）。
    - 实现：`tests/cli_clust_pipeline.rs` 中的 `test_clust_pipeline_full` 生成 3 个 2D 高斯分布簇（每簇 10 个点），计算欧氏距离矩阵后依次执行 `necom mat to-phylip`、`necom clust hier --method ward`、`necom cut simple tree.nwk --k 3`，最后用 `necom eval partition` 与 ground truth 对比。
    - 验证结果：ARI ≈ 1.0，确认 Ward linkage 能将不同 blob 的点正确分到不同主分支。
2.  **输入预处理文档 (已完成)**:
    - 目标：澄清输入要求。
    - 状态：已在 `docs/mat.md` 的 `necom mat transform` 小节和 `docs/clust.md` 的 Hierarchical Clustering 详细说明中更新，并增强了 `necom mat transform` 功能（支持对角线归一化），确保用户能正确地将相似度转换为距离。

## 6. clust hier 内部实现细节（与 SciPy 对比）

- SciPy 的 `ward` 更新公式在内部进行平方和开方（`sqrt`）；`necom` 采用全程平方距离运算（仅输出时开方），避免了中间步骤的精度损失和开方开销，理论上更高效。
- SciPy 的 `fast_linkage` 使用了 Heap 优化；`necom` 目前对非 NN-chain 方法使用朴素实现，未来可借鉴此优化。

## 7. 实现路线图

1. **基础图聚类**：已完成 MCL、CC、DBSCAN、K-Medoids。
2. **系统发育构树**：已完成 UPGMA、NJ、Hierarchical Clustering (hier)。
3. **评估体系**：`eval partition` 已完成（从 `clust eval` 迁移）；树评估功能已纳入 `necom eval` 统一设计（见 [eval-planned.md](eval-planned.md)），尚未实现。
4. **向量支持**：已完成。`libs/feature.rs` 提供 `FeatureVector` 基础设施，被 `necom eval partition --coords`（Davies-Bouldin 指标）等内部评估逻辑复用。
5. **统计聚类**：引入 GMM 实现，支持 BIC 模型选择（计划中，详见 §8.1）。
6. **层次聚类扩展**：实现 HDBSCAN（计划中，详见 §8.2）。
7. **大规模网络社区发现**：实现 Louvain / Leiden 算法（计划中，详见 §8.3）。

## 8. 计划中的算法

> **实现状态注记**：本节列出尚未实现的聚类算法规划。当前 `necom clust` 已实现 hier/dbscan/mcl/k-medoids/cc/nj/upgma + cut/eval。截至 2026-07-19，GMM、HDBSCAN、Louvain/Leiden 仍为规划，未进入实现阶段。

### 8.1 GMM (Gaussian Mixture Models)

- **原理**：假设数据由 $K$ 个高斯分布混合而成。使用 EM（期望最大化）算法迭代估计每个高斯分量的参数（均值、协方差）及每个样本属于各分量的后验概率。
- **命令**：`necom clust gmm`
- **计划内容**：支持**软聚类**（概率输出）与 **BIC** 模型选择。
- **价值**：适合**椭球形簇**与密度估计，解决 K-Means 仅适应球形簇的限制；BIC 可辅助确定最佳 K。

### 8.2 HDBSCAN

- **原理**：结合层次聚类与 DBSCAN。通过构建基于密度的层次树（Condensed Tree），并根据簇的稳定性（Stability）在不同层级自动提取最佳簇，无需全局 $\epsilon$。
- **命令**：`necom clust hdbscan`
- **scikit-learn 对应**：`HDBSCAN`
- **计划内容**：层次化 DBSCAN，无需手动指定 `eps`。
- **价值**：DBSCAN 的现代高级版，**自动适应不同密度的簇**，参数更少且更稳健。

### 8.3 Louvain / Leiden

- **原理**：基于模块度（Modularity）优化的社区发现算法。Louvain 贪心地最大化模块度；Leiden 改进了 Louvain 的局部合并策略，保证连通性并加速收敛。
- **命令**：(待定)
- **计划内容**：社区发现算法。
- **价值**：比 MCL 更适合**超大规模网络**的层次化结构探索。

## 9. 不引入的算法（Not Recommended / No Plans）

> 本节从 `docs/clust.md` 迁移而来，记录为何不将下列经典算法作为 `necom clust` 核心功能。这些算法在大规模生物数据场景下存在限制，故不计划引入。

### 9.1 K-Means

- **原因**：速度快，但假设簇为球形且方差相等；质心通常不是真实样本，缺乏生物学可解释性（如不能直接作为代表序列）。
- **替代**：`K-Medoids`（已实现），medoid 必须是真实样本，且支持任意距离矩阵，更适合生物序列分析。

### 9.2 Bisecting K-Means

- **原理**：自顶向下的分裂聚类。初始所有点为一簇；选择 SSE 最大的簇用 K-Means 二分，直到达到 K 个簇。
- **原因**：虽能产生树结构（二叉树），但继承 K-Means 的限制（需欧氏距离；质心非真实样本）。生物建树通常偏好自底向上的凝聚方法，如 UPGMA/NJ。

### 9.3 Affinity Propagation (AP)

- **原理**：消息传递机制，所有点竞争成为 exemplar。无需指定簇数，但计算复杂度高。
- **原因**：时间和空间复杂度高（$O(N^2)$），难以处理大规模生物序列数据（如 >10k 序列）。
- **替代**：小数据集找代表点用 `K-Medoids`；自动确定簇数用 `DBSCAN` 或 `MCL`。

### 9.4 Spectral Clustering

- **原理**：利用拉普拉斯矩阵的特征向量降维，在低维空间做 K-Means。本质是图的最小规范化割（Normalized Cut）。
- **原因**：构造拉普拉斯矩阵和特征分解计算昂贵（$O(N^3)$）。
- **替代**：`MCL` 在生物网络聚类上通常给出相似或更好的结果，且扩展性更好。

### 9.5 Mean Shift

- **原理**：基于密度的爬坡算法。点反复向邻域密度中心（mean shift）移动，直至收敛到局部密度峰值（modes）。
- **原因**：计算复杂度高，带宽参数难以自适应选择。
- **替代**：`DBSCAN` 或 `GMM` 通常覆盖其密度估计需求。

### 9.6 OPTICS

- **原理**：按可达性距离对点排序生成可达性图，单次运行捕获所有可能的密度层次。解决 DBSCAN 对全局 `eps` 的敏感性。
- **原因**：其核心思想（层次密度聚类）已被 **HDBSCAN** 更好地继承和自动化；OPTICS 结果（可达性图）需复杂后处理才能得到清晰簇。
- **替代**：使用更现代、参数更少、更自动化的 `HDBSCAN`。

### 9.7 Biclustering

- **原因**：同时聚类行和列（如 Spectral Co-Clustering），主要用于基因表达谱等特定矩阵子块挖掘场景。与 necom 当前聚焦的"样本分组"有本质差异。
- **替代**：若需聚类特征（列），可转置矩阵后用标准聚类；若需共表达模块，使用专门的表达谱工具（如 WGCNA）。

### 9.8 BIRCH

- **原理**：基于 Clustering Feature Tree（CF Tree）的增量聚类。单次扫描构建高度压缩的树；树节点存储簇统计量（和、平方和），适合超大数据集。
- **原因**：强依赖欧氏空间统计特性（计算质心和半径），不适合生物序列的复杂距离度量（如编辑距离）；且限制簇形状。
- **替代**：大规模向量用 MiniBatch K-Means 更通用；大规模序列用 `MCL`（图聚类）或 `CD-HIT/MMseqs2`（贪心聚类）。
