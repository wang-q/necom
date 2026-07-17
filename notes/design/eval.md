# necom eval 设计稿

> **实现状态注记**：本文档为设计提案。当前 `necom eval` 尚未实现；`necom clust eval` 已实现。本文已合并原 `nwk-eval.md` 中关于树评估的设计内容。本文提议将评估相关功能统一提升为顶级命令 `necom eval`，以容纳未来显著增长的聚类评估、树评估及支持值计算功能。

## 1. 背景与动机

随着 `necom` 从单纯的聚类/构树工具向评估与分析工具扩展，评估类功能将显著增长：

- 分区统计有效性评估（已存在于 `clust eval`）
- 树结构与参考树/性状/分类一致性评估（原 `nwk eval` 规划）
- 四分体采样一致性 / 分支支持值（未来）
- Bootstrap / Jackknife 支持值（未来）
- 树与树之间的拓扑距离（当前 `nwk compare`）

当前这些功能分散在 `clust` 与 `nwk` 两个命名空间下，用户难以一眼发现所有评估能力。与此同时，`necom cut` 已经作为独立的跨域命令存在，说明项目认可"桥接/后处理类工具值得顶级命名空间"。评估类功能同样具有跨域属性（既评估聚类分区，也评估演化树），因此适合统一为独立的 `necom eval` 命令。

## 2. 设计目标

- **统一入口**：所有"评估已有结果质量、支持度或一致性"的功能都通过 `necom eval` 访问。
- **清晰边界**：`eval` 只负责评估，不生成聚类、不推断树、不转换格式。
- **底层复用**：共享 `libs/clust/eval/`、`libs/phylo/cmp.rs`、`libs/phylo/tree/stat.rs` 等基础设施，避免指标重复实现。
- **可扩展性**：新增评估类型时，以子命令形式加入，不破坏现有接口。

## 3. 命令结构

```text
necom eval <subcommand>
```

### 3.1 `necom eval partition`

**定位**：评估聚类分区的统计有效性。

**来源**：由现有 `necom clust eval` 迁移而来。

**典型输入**：
- `-p, --partition <FILE>`：分区文件（cluster 或 pair 格式）
- `--matrix <FILE>`：距离矩阵（PHYLIP 或 Pair TSV）
- `--coords <FILE>`：坐标矩阵
- `--tree <FILE>`：系统发育树（Newick），用于基于树距离的指标
- `--ref <FILE>`：参考分区（Ground Truth）

**输出**：TSV，每行一个分区或每个簇的指标。

**指标**：外部指标（ARI、AMI、NMI、V-Measure、Jaccard、FMI 等）、内部指标（Silhouette、Dunn、Davies-Bouldin、Calinski-Harabasz、Hubert's Gamma 等），与现有 `clust eval` 保持一致。

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

**来源**：建议由现有 `necom nwk compare` 迁移至此，以强化 `eval` 作为统一评估入口的一致性。

**典型输入**：
- 一个或两个 Newick 文件
- `--include-trivial`：是否在 WRF/KF 中包含平凡分支

**输出**：TSV `Tree1	Tree2	RF_Dist	WRF_Dist	KF_Dist`，与现有 `nwk compare` 保持一致。

**底层复用**：`libs/phylo/cmp.rs` 中的 `compute_tree_metrics`。

**保留 `nwk compare` 的备选**：如果认为 pairwise 比较与单树评估差异过大，也可将 `eval compare` 留空，继续保留 `nwk compare`。但统一后用户学习成本更低。

### 3.4 `necom eval quartet`（未来）

**定位**：基于四分体采样计算树中各分支的四分体一致性或支持值。

**典型输入**：
- `-t, --tree <FILE>`：待评估的树
- `--align <FILE>`：序列比对文件（或预计算的四分体集合）
- `--samples <N>`：采样数量

**输出**：TSV，每行一个内部分支及其 quartet concordance / support 值。

**边界**：若功能是"用四分体法推断一棵新树"，则属于 `nwk`；若是"评估给定树的分支支持度"，则属于 `eval`。

### 3.5 `necom eval bootstrap`（未来）

**定位**：通过 Bootstrap / Jackknife 重采样评估树或聚类的稳定性。

**典型输入**：
- 原始数据矩阵或比对文件
- 重采样次数
- 可选：已有的重采样结果集合

**输出**：各分支或各簇的 Bootstrap 支持值 / 聚类稳定性分数。

**边界**：若涉及"生成大量重采样树"，可能需要一个独立命令（如 `nwk bootstrap`）先完成重采样，`eval bootstrap` 仅负责汇总支持值。

## 4. `eval tree` 详细设计

### 4.1 设计目标与范围

- **多层次评估**：支持从纯数学指标到引入外部生物学知识（分类表、参考树、地理分布）的深度评估。
- **基因树 vs 物种树**：承认并量化由 ILS、HGT 或基因重复/丢失引起的不一致性。
- **性状映射**：支持将叶子节点映射到分类层级或地理区域，计算分组的"纯度"和"熵"。
- **距离定义**：优先采用分支长度（Patristic distance）；当长度缺失时，以边数作为距离替代。

### 4.2 输入与输出

#### 输入

- **Target Tree (`-t`)**: 待评估的树（通常是基因树）。Newick 格式。
- **Partition (`-p`)**: 待评估的分组（可选；若不提供，则评估整棵树或根据 `--cut` 动态生成）。支持 `cluster` (列表) 和 `pair` (对) 格式。
- **External Context (可选)**:
  - **Trait Map (`--traits`)**: TSV 文件，用于分类或地理信息。格式 `LeafName <tab> Trait1,Trait2...`。
  - **Reference Tree (`--ref`)**: 参考树（通常是物种树），用于计算拓扑差异。
  - **Original Matrix (`--dist`)**: 原始 PHYLIP 距离矩阵，用于计算 Cophenetic 相关性。

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

设树上任意两个叶子的距离为 `d(x, y)`。

**簇内紧密性（Cohesion）**
- **簇内平均两两距离**: `mean(d_intra) = 平均{ d(x, y) | x, y ∈ C, x≠y }`
- **簇直径**: `diameter(C) = max{ d(x, y) | x, y ∈ C }`

**簇间分离度（Separation）**
- **最近簇间距离**: `d_min_inter(Ci) = min_{j≠i} min_{x∈Ci, y∈Cj} d(x, y)`

**Silhouette（基于树距离）**
对样本 `x`：
- `a(x)`：与同簇其他成员的平均距离
- `b(x)`：对所有其他簇，取"与该簇所有成员的平均距离"的最小值
- `s(x) = (b(x) - a(x)) / max(a(x), b(x))`
- **聚合**: 计算全局平均值和每簇的均值/中位数。单例的 `s(x)` 仅由 `b(x)` 决定（`a(x)=0`）。

**Cophenetic 相关系数（树拟合度）**
衡量树结构对原始距离矩阵的保真度。需提供 `--dist`。
- **定义**: 树上任意两叶子节点的 Cophenetic 距离，定义为它们最近共同祖先（LCA）的高度（即从叶子到根方向的距离）。注意：有些定义使用两倍高度（即路径长度），但在计算相关系数时，常数倍数不影响结果。
- **计算**: 计算原始距离矩阵 $D$ 与 Cophenetic 距离矩阵 $C$ 之间的 Pearson 相关系数 $r$。
- **SciPy 参考**: `scipy.cluster.hierarchy.cophenet`。
- **应用**: 评估不同聚类方法（如 UPGMA vs NJ vs Ward）对数据的拟合优度。$r$ 越接近 1，表示树结构越能真实反映原始数据。通常，$r > 0.8$ 认为拟合良好。

#### 4.3.2 性状/分类指标
*需提供 `--traits` (或兼容 `--tax`)。评估分组与外部标签（分类、地理、表型）的一致性。*

- **Purity**: 簇内最优势标签的占比。
  - 分类示例：9 个 *E. coli*，1 个 *S. enterica* -> Purity = 0.9。
  - 地理示例：9 个 *Asia*，1 个 *Europe* -> Purity = 0.9。
- **Entropy**: 标签分布的香农熵。`H(C) = - sum(p_i * log2(p_i))`。衡量簇内标签的混乱程度。
- **LCA Rank Consistency** (仅限分类): 如果提供层级信息，评估 LCA 是否对应特定层级。

#### 4.3.3 系统发育指标
*需提供 `--ref`。评估基因树局部结构与物种树的冲突。*

- **Local RF Distance**: 簇内子树与参考树对应子集的 Robinson-Foulds 距离。
- **Monophyly Check**: 基因树上的簇成员，在物种树上是否也聚集成单系群？
  - 若基因树聚类但物种树分散 -> 可能暗示 HGT 或 LBA（长枝吸引）。

#### 4.3.4 高级树比较 [计划中]
*参考 R `dendextend` 包的功能，支持更深入的树结构比较。*

- **Tanglegram**: 可视化两棵树的对应关系（通常用于比较基因树与物种树，或不同聚类方法的树）。虽然 `necom` 是 CLI 工具，但可以输出用于绘图的匹配表（link file）。
- **Baker's Gamma Index**: 类似于 Cophenetic 相关系数，但基于秩（Rank），对非线性关系更鲁棒。
- **Tree Distance**:
  - **Robinson-Foulds (RF)**: 拓扑差异（已在 §4.3.3 提及）。
  - **Weighted RF**: 考虑枝长的 RF 距离。
  - **Branch Score Distance (Kuhner-Felsenstein)**: 基于枝长的距离。

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

#### Phase 1：几何核心
- [ ] **CLI 搭建**: 支持 `-t`, `-p`, `--dist`。
- [ ] **核心指标**: Size, Diameter, AvgDist, MinInterDist。
- [ ] **Silhouette**: 实现树上距离计算逻辑。
- [ ] **Cophenetic**: 实现 Pearson 相关系数。

#### Phase 2：分类学扩展
- [ ] 解析 `--tax` 文件（KV 映射）。
- [ ] 实现 `Purity` 和 `Entropy` 指标。

#### Phase 3：参考树对比
- [ ] 在 `eval tree` 中复用已有的 `phylo::cmp` 模块（已实现，见 §5）。
- [ ] 接入 RF 距离（`necom eval compare` 或 `necom nwk compare`）与单系性检查（`is_monophyletic` 已实现）到 eval 输出。

#### Phase 4：文档与高级功能
- [ ] 完善文档，添加 Benchmark。
- [ ] 支持 NCBI Taxonomy Dump。

## 5. 共享基础设施

为避免重复实现，建议将通用评估逻辑沉淀到 `libs/`：

- **`libs/clust/eval/`**：分区级指标（ARI、AMI、Silhouette、Davies-Bouldin 等）。`eval partition` 使用。
- **`libs/phylo/cmp.rs`**：树拓扑比较（RF、WRF、KF）。`eval tree` 与 `eval compare` 共同复用。
- **`libs/phylo/tree/stat.rs`**：树内距离聚合（Diameter、AvgDist、LCA 高度）。`eval tree` 的几何指标复用。
- **`libs/phylo/tree/distance.rs`** / `Tree::get_distance`：叶子间距离计算。
- **`libs/clust/feature.rs`**：`FeatureVector` 基础设施，供基于坐标的指标复用。

### 5.1 实现备注（`eval tree`）

- **距离计算**：
  - 基础：复用 `Tree::get_distance`（来自 `libs/phylo/tree/mod.rs`，CLI 入口为 `nwk/distance.rs`）。
  - 优化：对于 `avg_clade` (AvgDist) 和 `max_clade` (Diameter)，直接复用 `libs/phylo/tree/stat.rs` 中的 $O(N)$ 自底向上聚合算法（已在 `necom cut` 中验证）。
- **拓扑比较**：
  - RF 距离：核心逻辑在 `libs/phylo/cmp.rs`（`TreeComparison` trait 提供 `robinson_foulds`/`weighted_robinson_foulds`/`kuhner_felsenstein`），CLI 入口为 `nwk/compare.rs`。
- **单系性检查**：
  - 复用 `Tree::is_monophyletic`（已在 `nwk/label.rs` 和 `nwk/subtree.rs` 中使用）。
- **性能策略**：
  - 优先使用基于遍历的聚合算法，避免构建 $O(N^2)$ 全距离矩阵。
  - 对于 Silhouette，针对大树考虑采样。
- **数值格式**：统一到六位小数，移除尾随零。

## 6. 输入输出约定

### 6.1 通用输入格式

- **分区文件**：沿用 `clust eval` 的格式约定（cluster 列表或 pair 列表）。
- **Newick 树**：标准 Newick 格式。
- **距离矩阵**：PHYLIP 方阵或 Pair TSV。
- **坐标矩阵**：TSV，每行一个样本，列为坐标。
- **性状表**：TSV `LeafName <tab> Trait1,Trait2...`，用于分类、地理或表型标签映射。
- **参考分区/树**：与待评估对象格式相同。

### 6.2 通用输出格式

- 统一为 **TSV**，首行为列名。
- 数值精度：默认六位小数，移除尾随零（与 `nwk compare` 保持一致）。
- 当输出包含多类指标时，可用 `--metrics` 控制输出列，避免冗余。

## 7. 迁移计划

### 阶段 1：搭建 `necom eval` 框架

1. 创建 `src/cmd_necom/eval/` 目录。
2. 实现 `necom eval --help` 与顶层命令注册（`src/necom.rs`）。
3. 先实现 `necom eval tree` 的 Phase 1（几何核心），因为它没有已存在的 CLI 需要兼容。

### 阶段 2：迁移 `clust eval`

1. 将 `src/cmd_necom/clust/eval.rs` 的实现逻辑迁移到 `src/cmd_necom/eval/partition.rs`。
2. 将底层指标代码抽到 `libs/clust/eval/`（如果尚未完全抽取）。
3. 更新 `src/cmd_necom/clust/mod.rs`，移除 `eval` 子命令。
4. 更新集成测试：`tests/cli_clust_eval.rs` → `tests/cli_eval_partition.rs`。
5. 更新文档：
   - `docs/clust-eval.md` 改为 `docs/eval-partition.md` 或合并进新的 `docs/eval.md`
   - 在 `docs/clust.md` 中添加指向 `docs/eval.md` 的说明
6. 这是一个 breaking change，按项目规则不添加反向兼容 shim。

### 阶段 3：迁移 `nwk compare`（可选）

1. 将 `src/cmd_necom/nwk/compare.rs` 的实现迁移到 `src/cmd_necom/eval/compare.rs`。
2. 保留 `nwk compare` 还是直接移除，取决于最终决策。如果统一，则直接移除；如果保留，则在 `eval compare` 中作为 thin wrapper 或 alias 不推荐。

### 阶段 4：扩展 `eval tree`

按 §4.5 实施计划完成 Phase 2~4。

### 阶段 5：实现未来扩展

1. `eval quartet`
2. `eval bootstrap`
3. 根据需求增加其他评估子命令（如 `eval stability`、`eval purity` 等）。

## 8. 与现有文档的关系

- **原 `nwk-eval.md`**：其关于树评估的内容已合并到本文档 §4，原文件已删除。
- **[clust-impl.md](clust-impl.md)**：其中提到的 `libs/clust/feature.rs`、Phase 7 真实分布验证等内容，将继续为 `eval partition` 提供底层支持。
- **[clust-eval.md](../docs/clust-eval.md)**：未来应迁移为 `docs/eval-partition.md` 或整合进 `docs/eval.md`。

## 9. 待决策问题

1. **`nwk compare` 是否迁移？**
   - 迁移：统一入口，但改变现有命令位置。
   - 保留：保持 pairwise 比较与单树评估的语义分离。

2. **`eval tree` 是否支持 `--tree` 多树输入？**
   - 如果支持，可能与 `eval compare` 的边界模糊。建议 `eval tree` 只处理单树，`eval compare` 处理多树。

3. **`eval bootstrap` 的边界**：
   - 是只负责汇总支持值（输入为重采样结果），还是也负责执行重采样（输入为原始数据）？

4. **输出列控制**：
   - 是否提供 `--metrics` 全局选项，让用户选择输出哪些指标列？

5. **文档组织**：
   - 是保留 `docs/eval-*.md` 每个子命令一份文档，还是合并为一份 `docs/eval.md` 顶层文档？

## 10. 结论

将评估功能统一为独立的 `necom eval` 顶级命令，能够：

- 提高评估功能的可发现性；
- 与 `necom cut` 的独立地位保持一致；
- 为未来 quartet、bootstrap 等扩展预留清晰的位置；
- 通过 `libs/` 层共享实现，避免指标重复开发。

主要成本是迁移已存在的 `clust eval`（以及可选的 `nwk compare`）。考虑到未来评估功能的增长速度，这一迁移越早完成，成本越低。
