# necom eval 设计稿

> **实现状态注记**：本文档为设计提案。当前 `necom eval compare` 已实现（由 `nwk compare` 迁移而来）；`necom clust eval` 仍待迁移为 `necom eval partition`；`necom eval tree` 尚未实现。本文已合并原 `nwk-eval.md` 中关于树评估的设计内容。本文提议将评估相关功能统一提升为顶级命令 `necom eval`，以容纳未来显著增长的聚类评估、树评估及支持值计算功能。
>
> **代码现状订正（2026-07）**：本文档已根据 `src/` 实际代码核对。关键订正：`compute_avg_clade_distances` 实际位于 `libs/phylo/tree/balance.rs`（`stat.rs` 仅 re-export）；代码库**不存在**簇内最大两两距离（`max_clade`）的 O(N) 算法，`stat.rs::diameter` 为全树双 BFS；`get_distance`/`node_distance` 位于 `query.rs`，`distance.rs` 是 CLI 输出辅助。详见 §5。

## 1. 背景与动机

随着 `necom` 从单纯的聚类/构树工具向评估与分析工具扩展，评估类功能将显著增长：

- 分区统计有效性评估（已存在于 `clust eval`）
- 树结构与参考树/性状/分类一致性评估（原 `nwk eval` 规划）
- 四分体采样一致性 / 分支支持值（未来）
- Bootstrap / Jackknife 支持值（未来）
- 树与树之间的拓扑距离（当前 `eval compare`）

当前这些功能分散在 `clust` 与 `nwk` 两个命名空间下，用户难以一眼发现所有评估能力。与此同时，`necom cut` 已经作为独立的跨域命令存在，说明项目认可"桥接/后处理类工具值得顶级命名空间"。评估类功能同样具有跨域属性（既评估聚类分区，也评估演化树），因此适合统一为独立的 `necom eval` 命令。

## 2. 设计目标

- **统一入口**：所有"评估已有结果质量、支持度或一致性"的功能都通过 `necom eval` 访问。
- **清晰边界**：`eval` 只负责评估，不生成聚类、不推断树、不转换格式。
- **底层复用**：共享 `libs/clust/eval/`、`libs/phylo/cmp.rs`、`libs/phylo/tree/{query,balance}.rs` 等基础设施，避免指标重复实现。
- **可扩展性**：新增评估类型时，以子命令形式加入，不破坏现有接口。
- **不预留命名空间**：未列入 Phase 1 的子命令（quartet、bootstrap 等）仅在 §3.4/§3.5 记录为"未来候选方向"，待出现真实需求时再设计，避免推测性设计违反项目准则。

## 3. 命令结构

```text
necom eval <subcommand>
```

### 3.1 `necom eval partition`

**定位**：评估聚类分区的统计有效性。

**来源**：由现有 `necom clust eval` 迁移而来。

**CLI surface 约定**：迁移阶段**保留现有 `clust eval` 的全部参数命名**，仅移动命令位置。这把破坏性变更限制在"子命令路径变化"本身，避免同时改动参数语义。现状参数（见 [args.rs:408-437](../../src/cmd_necom/args.rs#L408-L437)）：

- 位置参数 `p1`（必填）：待评估的分区文件
- `--other <FILE>`（alias `--truth`）：参考分区（Ground Truth），用于外部指标
- `--matrix <FILE>`：距离矩阵（PHYLIP 或 Pair TSV）
- `--tree <FILE>`：系统发育树（Newick），通过 `TreeDistance` 适配为 patistic 距离
- `--coords <FILE>`：坐标矩阵
- `--input-format <cluster|pair|long>`：分区文件格式（默认 `pair`）
- `--no-singletons`：从参考分区中移除单例
- `-o, --outfile <FILE>`：输出文件（默认 stdout）

> **注意**：早期草案曾提议引入 `-p`/`--partition`/`--ref` 等新命名。经核对，现状已用位置参数 `p1` + `--other` 表达同等语义，重新命名只会增加迁移成本，故不采纳。

**输出**：TSV，每行一个分区或每个簇的指标。

**指标**：外部指标（ARI、AMI、NMI、V-Measure、Jaccard、FMI 等）、内部指标（Silhouette、Dunn、Davies-Bouldin、Calinski-Harabasz、Hubert Gamma 等），与现有 `clust eval` 保持一致。底层实现已在 `libs/clust/eval/` 中良好分层（`EvalTarget`/`DistanceMatrix`/`TreeDistance`/`run_single`/`run_batch`），迁移为纯文件搬运。

### 3.2 `necom eval tree`

**定位**：评估单棵系统发育树的质量、拓扑拟合度及与外部生物学语境的一致性。

**来源**：由规划中的 `necom nwk eval` 迁移至此。

**目标**：建立一个**多维度的树评估框架**。除了基于树拓扑的几何指标外，重点引入**生物学语境**，评估基因树（Gene Tree）与物种树（Species Tree）或分类单元（Taxonomy）的一致性。

**旨在回答的四个核心问题**：
1. **几何紧密性**：从图论角度，分组是否紧密且分离？（Silhouette, Diameter）
2. **分类一致性**：分组是否对应自然的生物分类单元（如科、属、种）？（Taxonomic Purity）
3. **演化一致性**：基因树的分组与公认的物种演化历史是否冲突？（Discordance）
4. **地理/性状一致性**：分组是否对应特定的地理区域或表型特征？（Trait Purity/Entropy）

详细指标定义见 [§4](eval.md#4-eval-tree-详细设计)。

### 3.3 `necom eval compare`

**定位**：计算两棵或多棵树之间的拓扑距离。

**来源**：由原 `necom nwk compare` 迁移而来。

**决策**：**迁移 `nwk compare` → `eval compare`。** 早期草案曾基于「pairwise 比较与单树评估语义不同」主张保留原位，但经评估，统一入口带来的可发现性收益大于迁移成本，且底层 `compute_tree_metrics`（[cmp.rs:290](../../src/libs/phylo/cmp.rs#L290)）本就是无状态 `pub` 函数，迁移仅为 CLI 位置移动。

**CLI surface 约定**：迁移阶段**保留现有 `nwk compare` 的全部参数命名**，仅移动命令位置。现状参数：

- 位置参数 `infile`（必填）：第一个输入文件
- 位置参数 `compare_file`（可选）：第二个输入文件
- `--include-trivial`：在 WRF/KF 中包含平凡分裂（单叶分支）
- `-o, --outfile <FILE>`：输出文件（默认 stdout）

**输出**：TSV，列 `Tree1\tTree2\tRF_Dist\tWRF_Dist\tKF_Dist`。

**行为**：

- 单文件模式：文件内所有树两两比较（跳过自比较和重复对）。
- 双文件模式：文件1每棵树 vs 文件2每棵树，全交叉比较。
- 单文件模式仅 1 棵树时发出警告，输出仅含表头。

**指标**：RF（拓扑差异）、WRF（枝长差异，默认排除平凡分裂）、KF（Branch Score，默认排除平凡分裂）。底层复用 [cmp.rs:290](../../src/libs/phylo/cmp.rs#L290) `compute_tree_metrics`。

### 3.4 `necom eval quartet`（未来候选，不预留命名空间）

**定位**：基于四分体采样计算树中各分支的四分体一致性或支持值。

**状态**：**仅作为未来方向记录。** 在出现具体需求前，不设计 CLI、不注册子命令、不预留 `quartet` 名称。若需求出现，应先单独设计文档再实施。

**预期典型输入**（仅作记录，非承诺）：
- `-t, --tree <FILE>`：待评估的树
- `--align <FILE>`：序列比对文件（或预计算的四分体集合）
- `--samples <N>`：采样数量

**边界**：若功能是"用四分体法推断一棵新树"，则属于 `nwk`；若是"评估给定树的分支支持度"，则属于 `eval`。

### 3.5 `necom eval bootstrap`（未来候选，不预留命名空间）

**定位**：通过 Bootstrap / Jackknife 重采样评估树或聚类的稳定性。

**状态**：**仅作为未来方向记录。** 同 §3.4，不预留命名空间。

**边界**：若涉及"生成大量重采样树"，可能需要一个独立命令（如 `nwk bootstrap`）先完成重采样，`eval bootstrap` 仅负责汇总支持值。具体分工待需求出现时设计。

## 4. `eval tree` 详细设计

### 4.1 设计目标与范围

- **多层次评估**：支持从纯数学指标到引入外部生物学知识（分类表、参考树、地理分布）的深度评估。
- **基因树 vs 物种树**：承认并量化由 ILS、HGT 或基因重复/丢失引起的不一致性。
- **性状映射**：支持将叶子节点映射到分类层级或地理区域，计算分组的"纯度"和"熵"。
- **距离定义**：优先采用分支长度（Patristic distance）；当长度缺失时，以边数作为距离替代（见 [query.rs:124](../../src/libs/phylo/tree/query.rs#L124) `node_distance` 的现有约定）。

### 4.2 输入与输出

#### 输入

- **Target Tree**（位置参数，必填）: 待评估的树（通常是基因树）。Newick 格式。
- **Partition (`--part <FILE>`)**: 待评估的分组（可选；若不提供，则评估整棵树或根据 `--cut` 动态生成）。支持 `cluster` (列表) 和 `pair` (对) 格式。统一使用 `--part` 长选项（与 §4.4 示例一致），不提供短选项以避免与 partition 评估的位置参数混淆。
- **External Context (可选)**:
  - **Trait Map (`--traits <FILE>`)**: TSV 文件，用于分类或地理信息。格式 `LeafName <tab> Trait1,Trait2...`。
  - **Reference Tree (`--ref <FILE>`)**: 参考树（通常是物种树），用于计算拓扑差异。需注意叶子集匹配问题（见 §4.3.3）。
  - **Original Matrix (`--dist <FILE>`)**: 原始 PHYLIP 距离矩阵，用于计算 Cophenetic 相关性。

> **命名说明**：`eval tree` 使用 `--ref`（参考树）和 `--part`（分组），与 `eval partition` 的 `--other`（参考分区）命名不同。这是因为两者的语义不同：partition 评估比较两个对等的分区，tree 评估比较一棵目标树与一棵权威参考树。`--ref` 在 tree 语境下是新增参数，不与 partition 评估冲突。

#### 输出

- **TSV 格式**，包含多组列：
  - `Basic`: Size, Diameter, AvgDist.
  - `Geom`: Silhouette, Separation.
  - `Trait`: Purity, Entropy, DominantTrait. (复用分类学指标逻辑)
  - `Phylo`: RF-Distance (to Ref), ConflictScore.
  - `Fit`: CopheneticCorrelation.

### 4.3 指标详细定义

#### 4.3.1 几何/拓扑指标
*无需外部信息，仅基于输入树的边长和拓扑。*

设树上任意两个叶子的距离为 `d(x, y)`，由 [query.rs:79](../../src/libs/phylo/tree/query.rs#L79) `get_distance` 或 [query.rs:124](../../src/libs/phylo/tree/query.rs#L124) `node_distance` 计算。

**关键复杂度约束 — clade vs 任意 partition**：

几何指标的计算复杂度**取决于分组是否对应树上的 clade（子树）**：

- **Clade 分组**（如 `necom cut` 的输出）：每个簇的成员恰好构成一个子树。此时可用 O(N) 自底向上聚合（见 §5.1）。
- **任意 partition**（如 dbscan/mcl 输出、外部 ground truth）：簇成员可能跨多个子树。此时 avg/diameter/Silhouette 的 `b(x)` 本质是 O(N²)（每对叶子可能跨子树，需逐对计算 LCA 距离）。

**实施策略**：对每个簇先用 `Tree::is_clade`（[query.rs:182](../../src/libs/phylo/tree/query.rs#L182)）检测；clade 走 O(N) 路径，非 clade 走 O(N²) 路径。对大树提供 `--samples <N>` 对 Silhouette 采样。

**簇内紧密性（Cohesion）**
- **簇内平均两两距离**: `mean(d_intra) = 平均{ d(x, y) | x, y ∈ C, x≠y }`
  - Clade 路径：复用 [balance.rs:96](../../src/libs/phylo/tree/balance.rs#L96) `compute_avg_clade_distances` 的 O(N) 聚合。
  - 任意路径：O(|C|²) 次两两 `node_distance` 调用。
- **簇直径**: `diameter(C) = max{ d(x, y) | x, y ∈ C }`
  - **注意**：代码库当前**不存在**簇内直径的 O(N) 算法。`stat.rs::diameter`（[stat.rs:153](../../src/libs/phylo/tree/stat.rs#L153)）是全树双 BFS（Dijkstra 风格 BinaryHeap），不是簇内直径。Phase 1 需新增 `max_pairwise_distance` 函数到 `libs/phylo/tree/`。对 clade 可用自底向上聚合（维护子树内最长两条根-叶路径），对任意 partition 退化为 O(|C|²)。

**簇间分离度（Separation）**
- **最近簇间距离**: `d_min_inter(Ci) = min_{j≠i} min_{x∈Ci, y∈Cj} d(x, y)`
  - 本质 O(N²)（跨簇两两比较）。对大树可采样或基于 LCA 深度剪枝。

**Silhouette（基于树距离）**
对样本 `x`：
- `a(x)`：与同簇其他成员的平均距离
- `b(x)`：对所有其他簇，取"与该簇所有成员的平均距离"的最小值
- `s(x) = (b(x) - a(x)) / max(a(x), b(x))`
- **聚合**: 计算全局平均值和每簇的均值/中位数。
- **单例处理**：对大小为 1 的簇，标准做法是 `s(x) = 0`（scikit-learn 约定），而非由 `b(x)` 单独决定。早期草案"单例 s(x) 仅由 b(x) 决定"的说法会导致 `s(x)=1`，不正确。
- **复杂度**：O(N²)（任意 partition）。`b(x)` 需对每个样本遍历所有其他簇的成员。对大树用 `--samples` 采样。

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
- **Entropy**: 标签分布的香农熵。`H(C) = - sum(p_i * log2(p_i))`。衡量簇内标签的混乱程度。
- **LCA Rank Consistency** (仅限分类): 如果提供层级信息，评估 LCA 是否对应特定层级。

#### 4.3.3 系统发育指标
*需提供 `--ref`。评估基因树局部结构与物种树的冲突。*

**叶子集匹配前置处理**：

[cmp.rs:90](../../src/libs/phylo/cmp.rs#L90) `check_leaves_and_build_map` 要求两棵树叶子集**完全相等**，否则直接 `bail!`。基因树 vs 物种树的真实场景中，基因树常只采样部分 taxa。因此在计算 RF 前必须：

1. 计算两棵树叶集的交集 L = L_gene ∩ L_species。
2. 若 |L| < min(|L_gene|, |L_species|)，发出警告并 prune 两棵树到 L（复用 [algo.rs](../../src/libs/phylo/tree/algo.rs) 或 [ops.rs](../../src/libs/phylo/tree/ops.rs) 的 prune 操作）。
3. 若 |L| 过小（如 < 4），RF 距离无统计意义，应报错退出。

此项 prune 逻辑是 `eval tree --ref` 的必要前置步骤，非可选优化。

**指标**：

- **Local RF Distance**: 簇内子树与参考树对应子集的 Robinson-Foulds 距离。复用 [cmp.rs:183](../../src/libs/phylo/cmp.rs#L183) `robinson_foulds`。
- **Monophyly Check**: 基因树上的簇成员，在物种树上是否也聚集成单系群？
  - 复用 [query.rs:140](../../src/libs/phylo/tree/query.rs#L140) `is_monophyletic`（经 `Tree::is_clade` 包装，已在 [nwk/label.rs:112](../../src/cmd_necom/nwk/label.rs#L112)、[nwk/subtree.rs:70](../../src/cmd_necom/nwk/subtree.rs#L70) 验证）。
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
- [ ] **CLI 搭建**: 创建 `src/cmd_necom/eval/{mod,tree}.rs`，注册 `necom eval` 顶级命令。支持位置参数 tree、`--part`、`--dist`。
- [ ] **clade 检测**: 对每个簇调用 `Tree::is_clade` 分流。
- [ ] **核心指标**: Size, Diameter, AvgDist, MinInterDist。
  - Clade 路径：复用 `compute_avg_clade_distances`（AvgDist）。
  - 任意路径：基于 `node_distance` 的 O(|C|²) 计算。
  - **新增**：`max_pairwise_distance`（簇直径）到 `libs/phylo/tree/`，clade 路径用自底向上聚合，任意路径 O(|C|²)。
- [ ] **Silhouette**: 实现任意 partition 的 O(N²) 版本，单例 `s(x)=0`。提供 `--samples`。
- [ ] **Cophenetic**: 实现 O(N²) Pearson 相关系数。

#### Phase 2：分类学扩展（待 Phase 1 稳定后启动）
- [ ] 解析 `--traits` 文件（KV 映射）。
- [ ] 实现 `Purity` 和 `Entropy` 指标。

#### Phase 3：参考树对比（待 Phase 2 稳定后启动）
- [ ] 实现 `--ref` 叶子集交集 prune 前置处理。
- [ ] 接入 RF 距离（复用 [cmp.rs:183](../../src/libs/phylo/cmp.rs#L183)）与单系性检查（复用 [query.rs:140](../../src/libs/phylo/tree/query.rs#L140)）到 eval 输出。

#### Phase 4：未来方向（不预设时间表）
- NCBI Taxonomy Dump 支持、Tanglegram、Baker Gamma 等。仅在出现具体需求时启动单独设计文档。

## 5. 共享基础设施

为避免重复实现，通用评估逻辑沉淀到 `libs/`。以下路径均已核对代码现状：

- **`libs/clust/eval/`**：分区级指标（ARI、AMI、Silhouette、Davies-Bouldin 等）。`eval partition` 使用。已良好分层（`EvalTarget`/`DistanceMatrix`/`TreeDistance`/`run_single`/`run_batch`，见 [mod.rs:33](../../src/libs/clust/eval/mod.rs#L33)）。
- **`libs/phylo/cmp.rs`**：树拓扑比较（RF、WRF、KF）。`eval tree` 复用；`eval compare` 已迁移为顶级命令。
- **`libs/phylo/tree/balance.rs`**：`compute_avg_clade_distances`（[balance.rs:96](../../src/libs/phylo/tree/balance.rs#L96)）的 O(N) 自底向上聚合，`stat.rs` 仅 re-export。当前唯一消费者是 [tree_cut/clade.rs:93](../../src/libs/clust/tree_cut/clade.rs#L93)（即 `necom cut` 的底层）。
- **`libs/phylo/tree/query.rs`**：叶子间距离（`get_distance`/`node_distance`，[query.rs:79](../../src/libs/phylo/tree/query.rs#L79)）、LCA、`is_monophyletic`/`is_clade`（[query.rs:140](../../src/libs/phylo/tree/query.rs#L140)）。
- **`libs/phylo/tree/stat.rs`**：树统计（`diameter` 全树双 BFS、`compute_node_heights`、`TreeSummary` 等）。**注意**：`stat.rs::diameter` 不是簇内直径。
- **`libs/phylo/tree/distance.rs`**：CLI 输出辅助（`dist_root`/`dist_pairwise`/`dist_phylip` 等），**不是**叶子间距离 API。早期草案将其与 `Tree::get_distance` 混淆，已订正。
- **`libs/clust/feature.rs`**：`FeatureVector` 基础设施，供基于坐标的指标复用。

### 5.1 实现备注（`eval tree`）

- **距离计算**：
  - 基础：复用 `Tree::node_distance`（[query.rs:124](../../src/libs/phylo/tree/query.rs#L124)，自动在枝长和与边数间切换）。
  - **clade 路径优化**：对 AvgDist，复用 [balance.rs:96](../../src/libs/phylo/tree/balance.rs#L96) `compute_avg_clade_distances` 的 O(N) 聚合（返回 `HashMap<NodeId, f64>`，键为子树根）。该算法已在 [tree_cut/clade.rs:93](../../src/libs/clust/tree_cut/clade.rs#L93) 中使用。
  - **任意 partition 路径**：无 O(N) 折中，必须 O(|C|²) 调用 `node_distance`。
- **簇直径（max_clade）**：
  - **当前代码库不存在此算法**。`stat.rs::diameter` 是全树双 BFS，不适用。
  - Phase 1 需新增：对 clade 用自底向上聚合（每个节点维护子树内最长/次长根-叶路径，直径 = max(子直径, 最长+次长)）；对任意 partition 退化为 O(|C|²)。
- **拓扑比较**：
  - RF 距离：核心逻辑在 [cmp.rs:183](../../src/libs/phylo/cmp.rs#L183)（`TreeComparison` trait 提供 `robinson_foulds`/`weighted_robinson_foulds`/`kuhner_felsenstein`），优化入口 `compute_tree_metrics` 在 [cmp.rs:290](../../src/libs/phylo/cmp.rs#L290)。
  - CLI 入口为 [eval/compare.rs](../../src/cmd_necom/eval/compare.rs)，由 `nwk/compare.rs` 迁移而来（见 §3.3 决策）。
- **单系性检查**：
  - 复用 `Tree::is_monophyletic`（[query.rs:140](../../src/libs/phylo/tree/query.rs#L140)）或更严格的 `Tree::is_clade`（[query.rs:182](../../src/libs/phylo/tree/query.rs#L182)）。已在 [nwk/label.rs:112](../../src/cmd_necom/nwk/label.rs#L112) 和 [nwk/subtree.rs:70](../../src/cmd_necom/nwk/subtree.rs#L70) 中使用。
- **性能策略**：
  - clade 检测后分流，clade 走 O(N)，任意 partition 走 O(N²)。
  - 对于 Silhouette 和 Cophenetic，**明确承认 O(N²) 是本质开销**，不试图用遍历聚合规避；对大树通过 `--samples` 采样。
  - **避免**构建全距离矩阵的场景仅限 clade 分组。
- **数值格式**：统一到六位小数，移除尾随零（复用 [cmp.rs:350](../../src/libs/phylo/cmp.rs#L350) `format_float`，与 `eval compare` 保持一致）。

## 6. 输入输出约定

### 6.1 通用输入格式

- **分区文件**：沿用 `clust eval` 的格式约定（cluster 列表、pair 列表、long 批量格式）。
- **Newick 树**：标准 Newick 格式。
- **距离矩阵**：PHYLIP 方阵或 Pair TSV。
- **坐标矩阵**：TSV，每行一个样本，列为坐标。
- **性状表**：TSV `LeafName <tab> Trait1,Trait2...`，用于分类、地理或表型标签映射。
- **参考分区/树**：与待评估对象格式相同。

### 6.2 通用输出格式

- 统一为 **TSV**，首行为列名。
- 数值精度：默认六位小数，移除尾随零（复用 [cmp.rs:350](../../src/libs/phylo/cmp.rs#L350) `format_float`）。
- 当输出包含多类指标时，可用 `--metrics` 控制输出列，避免冗余。

## 7. 迁移计划

### 阶段 1：搭建 `necom eval` 框架

1. 创建 `src/cmd_necom/eval/` 目录（`mod.rs` + 后续子命令文件）。
2. 实现 `necom eval --help` 与顶层命令注册（[necom.rs:18-22](../../src/necom.rs#L18-L22) 新增 `.subcommand(cmd_necom::eval::make_subcommand())`，并在 [necom.rs:27](../../src/necom.rs#L27) 的 `after_help` 中新增 `* Evaluation: eval - ...` 分组）。
3. 先实现 `necom eval tree` 的 Phase 1（几何核心），因为它没有已存在的 CLI 需要兼容。

### 阶段 2：迁移 `clust eval`

1. 将 [src/cmd_necom/clust/eval.rs](../../src/cmd_necom/clust/eval.rs) 的实现迁移到 `src/cmd_necom/eval/partition.rs`。**保留全部现有参数命名**（`p1`、`--other`、`--matrix`、`--tree`、`--coords`、`--input-format`、`--no-singletons`），仅移动命令位置。
2. 底层指标代码 `libs/clust/eval/` **不动**（已良好分层，无需抽取）。
3. 更新 [src/cmd_necom/clust/mod.rs](../../src/cmd_necom/clust/mod.rs)，移除 `eval` 子命令注册。
4. 更新 [src/necom.rs:27](../../src/necom.rs#L27) 的 `after_help`：把 `clust - Algorithms: cc, dbscan, eval, hier, ...` 中的 `eval` 去掉。
5. 更新集成测试：`tests/cli_clust_eval.rs` → `tests/cli_eval_partition.rs`，`tests/cli_clust_eval_batch.rs` → `tests/cli_eval_partition_batch.rs`，`tests/cli_clust_eval_tree.rs` → `tests/cli_eval_partition_tree.rs`。测试数据目录 `tests/clust/` 下与 eval 相关的文件视情况迁移到 `tests/eval/`。
6. 更新文档：
   - `docs/help/clust/eval.md` → `docs/help/eval/partition.md`。
   - `docs/clust-eval.md` 决定去留：内容整合进 `docs/eval.md` 或保留为 `docs/eval-partition.md`（见 §9.5 决策）。
   - 在 `docs/clust.md` 中移除 `eval` 子命令的描述，添加指向 `docs/eval.md` 的说明。
7. 这是一个 breaking change，按项目规则不添加反向兼容 shim。

### 阶段 3：迁移 `nwk compare` → `eval compare`

按 §3.3 决策，将 `nwk compare` 迁移为 `eval compare`：

1. 创建 `src/cmd_necom/eval/{mod,compare}.rs`，从 `src/cmd_necom/nwk/compare.rs` 整体迁移实现，保留全部参数命名。
2. 在 `src/cmd_necom/mod.rs` 注册 `eval` 模块；在 `src/necom.rs` 注册 `eval` 顶级命令，更新 `after_help`（新增 `* Evaluation:` 分组，从 `nwk` 行移除 `compare`）。
3. 从 `src/cmd_necom/nwk/mod.rs` 移除 `compare` 注册；删除 `src/cmd_necom/nwk/compare.rs`。
4. 迁移帮助文档 `docs/help/nwk/compare.md` → `docs/help/eval/compare.md`（示例命令路径同步更新）。
5. 迁移测试 `tests/cli_nwk_compare.rs` → `tests/cli_eval_compare.rs`（命令路径 `nwk compare` → `eval compare`）。
6. 更新 `docs/nwk.md`：移除 `compare` 子命令描述，调整 Branch Length Handling 与 Planned Subcommands 措辞。
7. 新建 `docs/eval.md`：顶层命令介绍，风格与 `docs/nwk.md`/`docs/mat.md` 一致。
8. 底层 `libs/phylo/cmp.rs` **不动**。
9. 这是 breaking change，按项目规则不添加反向兼容 shim。

### 阶段 4：扩展 `eval tree`

按 §4.5 实施计划完成 Phase 2~3。Phase 4（NCBI Taxonomy、Tanglegram 等）不预设时间表。

### 阶段 5：未来扩展（按需启动）

出现具体需求时，单独设计文档后再实施：
1. `eval quartet`（见 §3.4）
2. `eval bootstrap`（见 §3.5）
3. 其他评估子命令（如 `eval stability`、`eval purity` 等）。

## 8. 与现有文档的关系

- 树评估相关内容（原 `nwk-eval.md`，已删除）已合并到本文档 §4。
- **[clust-impl.md](clust-impl.md)**：其中提到的 `libs/clust/feature.rs`、Phase 7 真实分布验证等内容，将继续为 `eval partition` 提供底层支持。
- **[clust-eval.md](../../docs/clust-eval.md)**：迁移后应整合进 `docs/eval.md` 或保留为 `docs/eval-partition.md`（见 §9.5）。

## 9. 待决策问题

1. **`nwk compare` 是否迁移？** — **已决策：迁移**（见 §3.3）。统一入口的可发现性收益优先于迁移成本；底层 `compute_tree_metrics` 为无状态 `pub` 函数，迁移仅移动 CLI 位置。

2. **`eval tree` 是否支持 `--tree` 多树输入？**
   - 建议否。`eval tree` 只处理单树；多树比较由 `eval compare` 承担。两者边界清晰。

3. **`eval bootstrap` 的边界**：
   - 待需求出现时设计。当前倾向：`eval bootstrap` 只负责汇总支持值（输入为重采样结果），执行重采样由独立命令（如 `nwk bootstrap`）完成。

4. **输出列控制**：
   - 倾向提供 `--metrics` 全局选项，让用户选择输出哪些指标列。具体设计在 Phase 1 实施时确定。

5. **文档组织**：
   - 倾向合并为一份 `docs/eval.md` 顶层文档，每个子命令一节。避免 `docs/eval-*.md` 文件 proliferation。具体在阶段 2 迁移时定。

## 10. 结论

将评估功能统一为独立的 `necom eval` 顶级命令，能够：

- 提高评估功能的可发现性；
- 与 `necom cut` 的独立地位保持一致；
- 为未来 quartet、bootstrap 等扩展预留清晰的位置（但不预占命名空间）；
- 通过 `libs/` 层共享实现，避免指标重复开发。

主要成本是迁移已存在的 `clust eval` 与 `nwk compare`。考虑到未来评估功能的增长速度，这一迁移越早完成，成本越低。

**实施优先级**：阶段 1（eval tree Phase 1）、阶段 2（clust eval 迁移）与阶段 3（nwk compare 迁移）为近期目标；阶段 4 按需推进；阶段 5 待需求出现