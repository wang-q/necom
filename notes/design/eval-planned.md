# necom eval 设计稿

> **实现状态注记**：本文档为 `necom eval` 命令的**未实现工作计划**。`eval partition` 与 `eval compare` 已实现，用户文档见 [`docs/eval.md`](../../docs/eval.md) 与 [`docs/help/eval.md`](../../docs/help/eval.md)。本文档聚焦尚未实现的 `eval tree`（§3.2、§4）以及未来候选方向 `eval quartet`（§3.4）。`eval replicate`（§3.5）已实现，迁移自 `nwk support`。
>
> **代码现状订正（2026-07）**：本文档已根据 `src/` 实际代码核对。关键订正：`compute_avg_clade_distances` 实际位于 `libs/phylo/tree/balance.rs`（`stat.rs` 仅 re-export）；代码库**不存在**簇内最大两两距离（`max_clade`）的 O(N) 算法，`stat.rs::diameter` 为全树双 BFS；`get_distance`/`node_distance` 位于 `query.rs`，`distance.rs` 是 CLI 输出辅助。详见 §5。

## 1. 背景与动机

随着 `necom` 从单纯的聚类/构树工具向评估与分析工具扩展，评估类功能将显著增长：

- 树结构与参考树/性状/分类一致性评估（`eval tree`，尚未实现 — 见 §3.2、§4）
- 四分体采样一致性 / 分支支持值（`eval quartet`，未来候选 — 见 §3.4）
- Bootstrap / Jackknife 支持值（`eval replicate`，已实现 — 见 §3.5）

当前这些功能分散在 `clust` 与 `nwk` 两个命名空间下，用户难以一眼发现所有评估能力。与此同时，`necom cut` 已经作为独立的跨域命令存在，说明项目认可"桥接/后处理类工具值得顶级命名空间"。评估类功能同样具有跨域属性（既评估聚类分区，也评估演化树），因此适合统一为独立的 `necom eval` 命令。

## 2. 设计目标

- **统一入口**：所有"评估已有结果质量、支持度或一致性"的功能都通过 `necom eval` 访问。
- **清晰边界**：`eval` 只负责评估，不生成聚类、不推断树、不转换格式。
- **底层复用**：共享 `libs/eval/`、`libs/phylo/cmp.rs`、`libs/phylo/tree/{query,balance}.rs` 等基础设施，避免指标重复实现。
- **可扩展性**：新增评估类型时，以子命令形式加入，不破坏现有接口。
- **不预留命名空间**：未列入 Phase 1 的子命令（如 quartet）仅在 §3.4 记录为"未来候选方向"，待出现真实需求时再设计，避免推测性设计违反项目准则。

## 3. 命令结构

```text
necom eval <subcommand>
```

> §3.1（`eval compare`）与 §3.3（`eval partition`）已实现，用户文档见 [docs/eval.md](../../docs/eval.md) 与 [docs/help/eval.md](../../docs/help/eval.md)，故本节略过，仅保留尚未实现的子命令设计。

### 3.2 `necom eval tree`

**定位**：评估单棵系统发育树的质量、拓扑拟合度及与外部生物学语境的一致性。

**来源**：由规划中的 `necom nwk eval` 迁移至此。

**目标**：建立一个**多维度的树评估框架**。除了基于树拓扑的几何指标外，重点引入**生物学语境**，评估基因树（Gene Tree）与物种树（Species Tree）或分类单元（Taxonomy）的一致性。

**旨在回答的四个核心问题**：
1. **几何紧密性**：从图论角度，分组是否紧密且分离？（Silhouette, Diameter）
2. **分类一致性**：分组是否对应自然的生物分类单元（如科、属、种）？（Taxonomic Purity）
3. **演化一致性**：基因树的分组与公认的物种演化历史是否冲突？（Discordance）
4. **地理/性状一致性**：分组是否对应特定的地理区域或表型特征？（Trait Purity/Entropy）

详细指标定义见 [§4](eval-planned.md#4-eval-tree-详细设计)。

**与 `eval partition --tree` 的边界**：`eval partition` 子命令已提供 `--tree` 选项（[args.rs](../../src/cmd_necom/args.rs)），通过 `TreeDistance` 包装树距离做内部 Silhouette 评估。`eval tree` 与之差异在于：(1) 单棵树作为**主输入**（位置参数），而非距离来源选项；(2) 对 clade 分组与任意 partition **分流优化**（前者 O(N)、后者 O(N²)）；(3) 引入 Cophenetic 拟合度、性状纯度、参考树对比等多维度评估，这些在 partition 评估中不存在。两者关系互补：partition 评估以分组为中心、树为辅助；tree 评估以树为中心、分组为辅助。

### 3.4 `necom eval quartet`（未来候选，不预留命名空间）

**定位**：基于四分体采样计算树中各分支的四分体一致性或支持值。

**状态**：**仅作为未来方向记录。** 在出现具体需求前，不设计 CLI、不注册子命令、不预留 `quartet` 名称。若需求出现，应先单独设计文档再实施。

**预期典型输入**（仅作记录，非承诺）：
- `-t, --tree <FILE>`：待评估的树
- `--align <FILE>`：序列比对文件（或预计算的四分体集合）
- `--samples <N>`：采样数量

**边界**：若功能是"用四分体法推断一棵新树"，则属于 `nwk`；若是"评估给定树的分支支持度"，则属于 `eval`。

### 3.5 `necom eval replicate`（已实现，迁移自 `nwk support`）

**定位**：通过 replicate trees（bootstrap/jackknife 重采样结果）为目标树内部节点赋支持值。

**状态**：**已实现**（2026-07 从 `nwk support` 迁移）。用户文档见 [`docs/eval.md`](../../docs/eval.md) 与 [`docs/help/eval/replicate.md`](../../docs/help/eval/replicate.md)。

**命名说明**：选择 `replicate` 而非 `bootstrap`，因命令不预设重采样来源（bootstrap 与 jackknife 均适用）；未来 quartet-based 支持（§3.4）将作为独立子命令，两者同属"分支支持"类别但方法不同。

**边界**：接受任意 replicate trees；不生成重采样，重采样由外部工具完成。

## 4. `eval tree` 详细设计

### 4.1 设计目标与范围

- **多层次评估**：支持从纯数学指标到引入外部生物学知识（分类表、参考树、地理分布）的深度评估。
- **基因树 vs 物种树**：承认并量化由 ILS、HGT 或基因重复/丢失引起的不一致性。
- **性状映射**：支持将叶子节点映射到分类层级或地理区域，计算分组的"纯度"和"熵"。
- **距离定义**：优先采用分支长度（Patristic distance）；当长度缺失时，以边数作为距离替代（见 [query.rs](../../src/libs/phylo/tree/query.rs) `node_distance` 的现有约定）。

### 4.2 输入与输出

#### 输入

- **Target Tree**（位置参数，必填）: 待评估的树（通常是基因树）。Newick 格式。
- **Partition (`--part <FILE>`)**: 待评估的分组（可选；若不提供，则评估整棵树或根据 `--cut` 动态生成）。支持 `cluster` (列表) 和 `pair` (对) 格式。统一使用 `--part` 长选项（与 §4.4 示例一致），不提供短选项以避免与 partition 评估的位置参数混淆。
- **External Context (可选)**:
  - **Trait Map (`--traits <FILE>`)**: TSV 文件，用于分类或地理信息。格式 `LeafName <tab> Trait1,Trait2...`。
  - **Reference Tree (`--ref <FILE>`)**: 参考树（通常是物种树），用于计算拓扑差异。需注意叶子集匹配问题（见 §4.3.3）。
  - **Original Matrix (`--dist <FILE>`)**: 原始 PHYLIP 距离矩阵，用于计算 Cophenetic 相关性。

> **命名说明**：`eval tree` 使用 `--ref`（参考树）和 `--part`（分组），与 `eval partition` 的 `--other`（参考分区）命名不同。这是因为两者的语义不同：partition 评估比较两个对等的分区，tree 评估比较一棵目标树与一棵权威参考树。`--ref` 在 tree 语境下是新增参数，不与 partition 评估冲突。
>
> **`--part` vs `--tree` 角色**：`eval tree` 的 `--part`（待评估的分组）与 `eval partition` 的 `--tree`（patristic 距离来源，[args.rs](../../src/cmd_necom/args.rs)）角色相反。两者命名差异反映语义差异：partition 评估以分组为主、树为辅；tree 评估以树为主、分组为辅。用户不应混淆。

#### 输出

- **TSV 格式**，包含多组列：
  - `Basic`: Size, Diameter, AvgDist.
  - `Geom`: Silhouette, Separation.
  - `Trait`: Purity, Entropy, DominantTrait. (复用分类学指标逻辑)
  - `Phylo`: RF-Distance (to Ref), ConflictScore.
  - `Fit`: CopheneticCorrelation.
- **`--metrics` 全局选项**：控制输出哪些列组（如 `--metrics basic,geom` 或 `--metrics cophenet` 单选），避免为不需要的指标付出计算开销（特别是 Cophenetic 的强制 O(N²)）。与 §4.4 场景 D 的 `--metrics cophenet` 用法一致；具体取值设计在 Phase 1 实施时确定（见 §9 问题 3）。

### 4.3 指标详细定义

#### 4.3.1 几何/拓扑指标
*无需外部信息，仅基于输入树的边长和拓扑。*

设树上任意两个叶子的距离为 `d(x, y)`，由 [query.rs](../../src/libs/phylo/tree/query.rs) `get_distance` 或 [query.rs](../../src/libs/phylo/tree/query.rs) `node_distance` 计算。

**关键复杂度约束 — clade vs 任意 partition**：

几何指标的计算复杂度**取决于分组是否对应树上的 clade（子树）**：

- **Clade 分组**（如 `necom cut` 的输出）：每个簇的成员恰好构成一个子树。此时可用 O(N) 自底向上聚合（见 §5.1）。
- **任意 partition**（如 dbscan/mcl 输出、外部 ground truth）：簇成员可能跨多个子树。此时 avg/diameter/Silhouette 的 `b(x)` 本质是 O(N²)（每对叶子可能跨子树，需逐对计算 LCA 距离）。

**实施策略**：对每个簇先用 `Tree::is_clade`（[query.rs](../../src/libs/phylo/tree/query.rs)）检测；clade 走 O(N) 路径，非 clade 走 O(N²) 路径。对大树提供 `--samples <N>` 对 Silhouette 采样。

**簇内紧密性（Cohesion）**
- **簇内平均两两距离**: `mean(d_intra) = 平均{ d(x, y) | x, y ∈ C, x≠y }`
  - Clade 路径：复用 [balance.rs](../../src/libs/phylo/tree/balance.rs) `compute_avg_clade_distances` 的 O(N) 聚合。
  - 任意路径：O(|C|²) 次两两 `node_distance` 调用。
- **簇直径**: `diameter(C) = max{ d(x, y) | x, y ∈ C }`
  - **注意**：代码库当前**不存在**簇内直径的 O(N) 算法。`stat.rs::diameter`（[stat.rs](../../src/libs/phylo/tree/stat.rs)）是全树双 BFS（Dijkstra 风格 BinaryHeap），不是簇内直径。Phase 1 需新增 `max_pairwise_distance` 函数到 `libs/phylo/tree/`。对 clade 可用自底向上聚合（维护子树内最长两条根-叶路径），对任意 partition 退化为 O(|C|²)。

**簇间分离度（Separation）**
- **最近簇间距离**: `d_min_inter(Ci) = min_{j≠i} min_{x∈Ci, y∈Cj} d(x, y)`
  - 本质 O(N²)（跨簇两两比较）。对大树可采样或基于 LCA 深度剪枝。

**Silhouette（基于树距离）**
对样本 `x`：
- `a(x)`：与同簇其他成员的平均距离
- `b(x)`：对所有其他簇，取"与该簇所有成员的平均距离"的最小值
- `s(x) = (b(x) - a(x)) / max(a(x), b(x))`
- **聚合**: 计算全局平均值和每簇的均值/中位数。
- **复用现有实现**：`libs/eval/distance.rs` 已提供 `silhouette_score(partition: &LabelMap, dist_mat: &dyn DistanceMatrix)`。`TreeDistance`（[distance.rs](../../src/libs/eval/distance.rs)）已实现 `DistanceMatrix` trait，包装 `Tree::node_distance`。复用路径：`TreeDistance::new(tree)` → `silhouette_score(&partition, &td)`，无需重写。
- **单例处理**：现有 `silhouette_score` 已遵循 scikit-learn 约定（大小为 1 的簇 `s(x)=0`）。早期草案"单例 s(x) 仅由 b(x) 决定"的说法会导致 `s(x)=1`，不正确。
- **复杂度**：O(N²)（任意 partition）。`b(x)` 需对每个样本遍历所有其他簇的成员。对大树用 `--samples` 采样（**新增工作**：现有 `silhouette_score` 无采样参数，Phase 1 需扩展或在外层包装采样逻辑）。

**Cophenetic 相关系数（树拟合度）**
衡量树结构对原始距离矩阵的保真度。需提供 `--dist`。
- **定义**: 树上任意两叶子节点的 Cophenetic 距离，定义为它们最近共同祖先（LCA）的高度（即从叶子到根方向的距离）。注意：有些定义使用两倍高度（即路径长度），但在计算相关系数时，常数倍数不影响结果。
- **计算**: 计算原始距离矩阵 D 与 Cophenetic 距离矩阵 C 之间的 Pearson 相关系数 r。
- **复杂度**：**强制 O(N²)**。需为所有 N(N-1)/2 对叶子计算 Cophenetic 距离并存储，无法用遍历聚合避免。这是该指标的本质开销，非算法缺陷。
- **SciPy 参考**: `scipy.cluster.hierarchy.cophenet`。
- **应用**: 评估不同聚类方法（如 UPGMA vs NJ vs Ward）对数据的拟合优度。r 越接近 1，表示树结构越能真实反映原始数据。通常，r > 0.8 认为拟合良好。

#### 4.3.2 性状/分类指标
*需提供 `--traits` (或兼容 `--tax`)。评估分组与外部标签（分类、地理、表型）的一致性。*

- **Purity**: 簇内最优势标签的占比。
  - 分类示例：9 个 E. coli，1 个 S. enterica -> Purity = 0.9。
  - 地理示例：9 个 Asia，1 个 Europe -> Purity = 0.9。
- **Entropy**: 标签分布的香农熵。`H(C) = - sum(p_i * log(p_i))`。衡量簇内标签的混乱程度。If reusing the existing `libs/eval/pairwise.rs::entropy` helper, the implementation uses natural logarithms; any standalone entropy implementation must document and consistently use its chosen base.
- **LCA Rank Consistency** (仅限分类): 如果提供层级信息，评估 LCA 是否对应特定层级。

#### 4.3.3 系统发育指标
*需提供 `--ref`。评估基因树局部结构与物种树的冲突。*

**叶子集匹配前置处理**：

[cmp.rs](../../src/libs/phylo/cmp.rs) `check_leaves_and_build_map` 要求两棵树叶子集**完全相等**，否则直接 `bail!`。基因树 vs 物种树的真实场景中，基因树常只采样部分 taxa。因此在计算 RF 前必须：

1. 计算两棵树叶集的交集 L = L_gene ∩ L_species。
2. 若 |L| < min(|L_gene|, |L_species|)，发出警告并 prune 两棵树到 L。现有工具需组合：`algo::compute_keep_set`（[algo.rs](../../src/libs/phylo/tree/algo.rs)，计算保留集 = 交集叶子 ∪ 祖先）+ `algo::prune_nodes`（[algo.rs](../../src/libs/phylo/tree/algo.rs)，彻底 prune + `compact()`）。**注意**：`algo::prune_nodes` 调用后 NodeId 失效（因 `compact()` 物理回收），后续操作需基于 prune 后的新树重建映射。Phase 3 需新增 `intersect_leaves(t1, t2) -> (Tree, Tree)` 封装此流程。
3. 若 |L| 过小（如 < 4），RF 距离无统计意义，应报错退出。

此项 prune 逻辑是 `eval tree --ref` 的必要前置步骤，非可选优化。

**指标**：

- **Local RF Distance**: 簇内子树与参考树对应子集的 Robinson-Foulds 距离。复用 [cmp.rs](../../src/libs/phylo/cmp.rs) `robinson_foulds`。
- **Monophyly Check**: 基因树上的簇成员，在物种树上是否也聚集成单系群？
  - 复用 `Tree::is_clade`（[query.rs](../../src/libs/phylo/tree/query.rs)，要求 ≥2 节点）或 `Tree::is_monophyletic`（[query.rs](../../src/libs/phylo/tree/query.rs)，单节点也视为单系）。`is_clade` 是 `is_monophyletic` 的严格包装。CLI 层（[nwk/label.rs](../../src/cmd_necom/nwk/label.rs)、[nwk/subtree.rs](../../src/cmd_necom/nwk/subtree.rs)）当前统一使用 `Tree::is_clade`，未直接调用 `is_monophyletic`。
  - 若基因树聚类但物种树分散 -> 可能暗示 HGT 或 LBA（长枝吸引）。

#### 4.3.4 高级树比较（未来候选，不纳入 Phase 1）
*参考 R dendextend 包的功能。仅作为未来方向记录，不预留子命令或参数命名。*

- **Tanglegram**: 可视化两棵树的对应关系。`necom` 是 CLI 工具，可输出用于绘图的匹配表（link file）。
- **Baker Gamma Index**: 类似于 Cophenetic 相关系数，但基于秩（Rank），对非线性关系更鲁棒。
- **Tree Distance**:
  - **Robinson-Foulds (RF)**: 拓扑差异（已在 §4.3.3 提及）。
  - **Weighted RF**: 考虑枝长的 RF 距离。
  - **Branch Score Distance (Kuhner-Felsenstein)**: 基于枝长的距离。

这些指标在出现具体需求前不设计 CLI；底层 RF/WRF/KF 实现已存在于 [cmp.rs](../../src/libs/phylo/cmp.rs)，需要时直接复用。

### 4.4 典型用法

```bash
# 场景 A: 纯几何评估 (无外部信息)
necom eval tree tree.nwk --part clusters.tsv > geom_eval.tsv
# 输出: ClusterID, Size, Silhouette, Diameter

# 场景 B: 性状/地理一致性验证
# traits.tsv: LeafName <tab> Region
necom eval tree tree.nwk --part clusters.tsv --traits location.tsv > geo_eval.tsv
# 输出: ..., Purity, DominantTrait, Entropy

# 场景 C: 基因树质量控制 (Reference Tree)
necom eval tree gene_tree.nwk --ref species_tree.nwk > phylo_eval.tsv
# 输出: Global_RF_Dist, Cluster_Conflict_Score

# 场景 D: 原始距离拟合度 (Cophenetic)
necom eval tree tree.nwk --dist matrix.phy --metrics cophenet > fit.tsv
```

### 4.5 实施计划

#### Phase 1：几何核心（含 clade/任意 partition 分支）
- [ ] **CLI 搭建**: 在 `src/cmd_necom/eval/` 下新建 `tree.rs`，并在 `mod.rs` 中注册 `eval tree` 子命令（顶级 `necom eval` 命令已注册，见 §7 阶段 1）。支持位置参数 tree、`--part`、`--dist`。
- [ ] **clade 检测**: 对每个簇调用 `Tree::is_clade` 分流。
- [ ] **核心指标**: Size, Diameter, AvgDist, MinInterDist。
  - Clade 路径：复用 `compute_avg_clade_distances`（AvgDist）。
  - 任意路径：基于 `node_distance` 的 O(|C|²) 计算。
  - **新增**：`max_pairwise_distance`（簇直径）到 `libs/phylo/tree/`，clade 路径用自底向上聚合，任意路径 O(|C|²)。
- [ ] **Silhouette**: 复用 `libs/eval/distance.rs::silhouette_score`，通过 `TreeDistance::new(tree)` 适配树距离。单例 `s(x)=0` 已在现有实现中遵循。**新增工作**：为大树支持 `--samples` 采样参数（现有 `silhouette_score` 无采样，需扩展或外层包装）。
- [ ] **Cophenetic**: 实现 O(N²) Pearson 相关系数。

#### Phase 2：分类学扩展（待 Phase 1 稳定后启动）
- [ ] 解析 `--traits` 文件（KV 映射）。
- [ ] 实现 `Purity` 和 `Entropy` 指标。

#### Phase 3：参考树对比（待 Phase 2 稳定后启动）
- [ ] 实现 `--ref` 叶子集交集 prune 前置处理。新增 `intersect_leaves(t1, t2) -> (Tree, Tree)` 工具函数（封装 `algo::compute_keep_set` + `algo::prune_nodes`），返回 prune 后的新树。注意 `algo::prune_nodes` 后 NodeId 失效，需重建映射。
- [ ] 接入 RF 距离（复用 [cmp.rs](../../src/libs/phylo/cmp.rs)）与单系性检查（复用 [query.rs](../../src/libs/phylo/tree/query.rs) `is_clade`）到 eval 输出。

#### Phase 4：未来方向（不预设时间表）
- NCBI Taxonomy Dump 支持、Tanglegram、Baker Gamma 等。仅在出现具体需求时启动单独设计文档。

## 5. 共享基础设施

为避免重复实现，通用评估逻辑沉淀到 `libs/`。以下路径均已核对代码现状：

- **`libs/eval/`**：分区级指标，三类共 23 个：
  - **外部指标**（[pairwise.rs](../../src/libs/eval/pairwise.rs)，12 个）：ARI、AMI、Homogeneity、Completeness、V-Measure、FMI、NMI、MI、RI、Jaccard、Precision、Recall。
  - **距离矩阵指标**（[distance.rs](../../src/libs/eval/distance.rs)，5 个）：`silhouette_score`（[distance.rs](../../src/libs/eval/distance.rs)）、`dunn_score`、`c_index_score`、`gamma_score`、`tau_score`。
  - **坐标指标**（[coordinates.rs](../../src/libs/eval/coordinates.rs)，6 个）：`davies_bouldin_score`、`calinski_harabasz_score`、`pbm_score`、`ball_hall_score`、`xie_beni_score`、`wemmert_gancarski_score`。
  - **关键**：`TreeDistance`（[distance.rs](../../src/libs/eval/distance.rs)）已实现 `DistanceMatrix` trait，包装 `Tree::node_distance`。**所有 5 个距离矩阵指标可直接用于树评估**，无需重写。
  - 已良好分层（`EvalTarget`/`DistanceMatrix`/`TreeDistance`/`run_single`/`run_batch`，见 [mod.rs](../../src/libs/eval/mod.rs)）。`eval partition` 使用；`eval tree` 复用 `silhouette_score` + `TreeDistance`。
  - **缺失（需新增）**：Purity、Entropy（Phase 2）、Cophenetic 相关系数（Phase 1）。
- **`libs/phylo/cmp.rs`**：树拓扑比较（RF、WRF、KF）。`eval tree` 复用；`eval compare` 已迁移为顶级命令。
- **`libs/phylo/tree/balance.rs`**：`compute_avg_clade_distances`（[balance.rs](../../src/libs/phylo/tree/balance.rs)）的 O(N) 自底向上聚合，`stat.rs` 仅 re-export。当前唯一消费者是 [tree_cut/clade.rs](../../src/libs/tree_cut/clade.rs)（即 `necom cut` 的底层）。
- **`libs/phylo/tree/query.rs`**：叶子间距离（`get_distance`/`node_distance`，[query.rs](../../src/libs/phylo/tree/query.rs)）、LCA、`is_monophyletic`/`is_clade`（[query.rs](../../src/libs/phylo/tree/query.rs)）。
- **`libs/phylo/tree/stat.rs`**：树统计（`diameter` 全树双 BFS、`compute_node_heights`、`TreeSummary` 等）。**注意**：`stat.rs::diameter` 不是簇内直径。
- **`libs/phylo/tree/distance.rs`**：CLI 输出辅助（`dist_root`/`dist_pairwise`/`dist_phylip` 等），**不是**叶子间距离 API。早期草案将其与 `Tree::get_distance` 混淆，已订正。
- **`libs/feature.rs`**：`FeatureVector` 基础设施，供基于坐标的指标复用。

### 5.1 实现备注（`eval tree`）

- **距离计算**：
  - 基础：复用 `Tree::node_distance`（[query.rs](../../src/libs/phylo/tree/query.rs)，自动在枝长和与边数间切换）。
  - **clade 路径优化**：对 AvgDist，复用 [balance.rs](../../src/libs/phylo/tree/balance.rs) `compute_avg_clade_distances` 的 O(N) 聚合（返回 `HashMap<NodeId, f64>`，键为子树根）。该算法已在 [tree_cut/clade.rs](../../src/libs/tree_cut/clade.rs) 中使用。
  - **任意 partition 路径**：无 O(N) 折中，必须 O(|C|²) 调用 `node_distance`。
- **簇直径（max_clade）**：
  - **当前代码库不存在此算法**。`stat.rs::diameter` 是全树双 BFS，不适用。
  - Phase 1 需新增：对 clade 用自底向上聚合（每个节点维护子树内最长/次长根-叶路径，直径 = max(子直径, 最长+次长)）；对任意 partition 退化为 O(|C|²)。
- **拓扑比较**：
  - RF 距离：核心逻辑在 [cmp.rs](../../src/libs/phylo/cmp.rs)（`TreeComparison` trait 提供 `robinson_foulds`/`weighted_robinson_foulds`/`kuhner_felsenstein`），优化入口 `compute_tree_metrics` 在 [cmp.rs](../../src/libs/phylo/cmp.rs)。
  - CLI 入口为 [eval/compare.rs](../../src/cmd_necom/eval/compare.rs)，由 `nwk/compare.rs` 迁移而来。
- **单系性检查**：
  - 复用 `Tree::is_clade`（[query.rs](../../src/libs/phylo/tree/query.rs)，要求 ≥2 节点）或 `Tree::is_monophyletic`（[query.rs](../../src/libs/phylo/tree/query.rs)，单节点也视为单系）。CLI 层（[nwk/label.rs](../../src/cmd_necom/nwk/label.rs) 和 [nwk/subtree.rs](../../src/cmd_necom/nwk/subtree.rs)）当前统一使用 `Tree::is_clade`。
- **Silhouette 复用**：直接复用 `libs/eval/distance.rs::silhouette_score` + `TreeDistance`（[distance.rs](../../src/libs/eval/distance.rs)），无需重写。`TreeDistance::new(tree)` 包装 `Tree::node_distance` 并实现 `DistanceMatrix` trait。现有实现已遵循 sklearn 单例 `s(x)=0` 约定。`--samples` 采样为 Phase 1 新增工作。
- **`tree_medoid`**：[query.rs](../../src/libs/phylo/tree/query.rs) 提供 `tree_medoid(tree, ids) -> Option<usize>`，O(N²) 调用 `get_distance`。当前注释标记 "Currently unused by the CLI"。`eval tree` 可考虑复用于 representative 选择或 Medoid-based 聚类评估。
- **性能策略**：
  - clade 检测后分流，clade 走 O(N)，任意 partition 走 O(N²)。
  - 对于 Silhouette 和 Cophenetic，**明确承认 O(N²) 是本质开销**，不试图用遍历聚合规避；对大树通过 `--samples` 采样。
  - **避免**构建全距离矩阵的场景仅限 clade 分组。
- **数值格式**：统一到六位小数，移除尾随零（复用 [cmp.rs](../../src/libs/phylo/cmp.rs) `format_float`，与 `eval compare` 保持一致）。

## 6. 输入输出约定

### 6.1 通用输入格式

- **分区文件**：沿用 `eval partition` 的格式约定（cluster 列表、pair 列表、long 批量格式）。
- **Newick 树**：标准 Newick 格式。
- **距离矩阵**：PHYLIP 方阵或 Pair TSV。
- **坐标矩阵**：TSV，每行一个样本，列为坐标。
- **性状表**：TSV `LeafName <tab> Trait1,Trait2...`，用于分类、地理或表型标签映射。
- **参考分区/树**：与待评估对象格式相同。

### 6.2 通用输出格式

- 统一为 **TSV**，首行为列名。
- 数值精度：默认六位小数，移除尾随零（复用 [cmp.rs](../../src/libs/phylo/cmp.rs) `format_float`）。
- 当输出包含多类指标时，可用 `--metrics` 控制输出列，避免冗余。

## 7. 迁移计划

> 阶段 2（迁移 `clust eval` → `eval partition`）与阶段 3（迁移 `nwk compare` → `eval compare`）均已完成（2026-07），故下文略过，仅保留未完成阶段的记录。

### 阶段 1：搭建 `necom eval` 框架

> **状态（2026-07）**：步骤 1-2 已完成；步骤 3（`eval tree` Phase 1）仍待办。

1. ✅ 创建 `src/cmd_necom/eval/` 目录：已含 `mod.rs` + `compare.rs` + `partition.rs`（`tree.rs` 尚未创建）。
2. ✅ 顶层命令注册：[necom.rs](../../src/necom.rs) `.subcommand(cmd_necom::eval::make_subcommand())`、[necom.rs](../../src/necom.rs) dispatch、[necom.rs](../../src/necom.rs) after_help 文本 `* eval - Metrics: compare, partition`。
3. ☐ 实现 `necom eval tree` 的 Phase 1（几何核心），它没有已存在的 CLI 需要兼容。

### 阶段 4：扩展 `eval tree`

按 §4.5 实施计划完成 Phase 2~3。Phase 4（NCBI Taxonomy、Tanglegram 等）不预设时间表。

### 阶段 5：未来扩展（按需启动）

出现具体需求时，单独设计文档后再实施：
1. `eval quartet`（见 §3.4）
2. 其他评估子命令（如 `eval stability`、`eval purity` 等）。

## 8. 与现有文档的关系

- 树评估相关内容（原 `nwk-eval.md`，已删除）已合并到本文档 §4。
- **[clust-impl.md](clust-impl.md)**：其中提到的 `libs/feature.rs`、Phase 7 真实分布验证等内容，将继续为 `eval partition` 提供底层支持。
- **[eval-partition.md](../../docs/eval-partition.md)**：`partition` 子命令的详细指标参考（原 `docs/clust-eval.md`）。

## 9. 待决策问题

1. **`eval tree` 是否支持 `--tree` 多树输入？**
   - 建议否。`eval tree` 只处理单树；多树比较由 `eval compare` 承担。两者边界清晰。

2. **`eval replicate` 的边界（已决策）**：
   - `eval replicate` 只负责汇总支持值（输入为重采样结果），执行重采样由外部工具完成。已实现（2026-07）。

3. **输出列控制**：
   - 倾向提供 `--metrics` 全局选项，让用户选择输出哪些指标列组（如 `--metrics basic,geom` 或 `--metrics cophenet` 单选）。设计模式与 `eval partition` 的 `--input-format`（分模式控制）一致。具体取值在 Phase 1 实施时确定，§4.2 已预留该选项的语义说明。

## 10. 结论

将评估功能统一为独立的 `necom eval` 顶级命令，能够：

- 提高评估功能的可发现性；
- 与 `necom cut` 的独立地位保持一致；
- 为未来 quartet 等扩展预留清晰的位置（但不预占命名空间）；
- 通过 `libs/` 层共享实现，避免指标重复开发。

主要成本是迁移已存在的 `clust eval` 与 `nwk compare`，**两者均已完成**。本文档剩余内容聚焦 `eval tree` 及后续候选方向（quartet），按需推进。`eval replicate` 已实现（见 §3.5）。