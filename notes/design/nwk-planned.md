# necom nwk 未来工作计划

> **实现状态注记**：本文档记录 `necom nwk` 命令的**未实现工作计划**与已决策不实现的功能。截至 2026-07-21，`necom nwk` 主体命令体系已实现（详见 [`phylo.md`](phylo.md)）。本文档聚焦尚未落地的树排序增强（§1）与随机树生成（§2）。

## 1. Optimal Leaf Ordering (OLO)

### 1.1 背景

标准层次聚类或构树算法生成的树，其内部节点的左右子树顺序通常是任意的。这导致在绘制热图（Heatmap）或线性展示叶子时，相似的叶子可能不相邻，视觉效果杂乱。

### 1.2 SciPy 方案

`scipy.cluster.hierarchy.optimal_leaf_ordering` 实现了 Bar-Joseph et al. (2001) 的动态规划算法。该算法在不改变树拓扑结构的前提下，通过旋转内部节点，最小化相邻叶子之间的距离之和。

### 1.3 necom 规划

计划在 `necom nwk order` 中提供 OLO 支持，可能通过 `--olo` 或 `--optimal` 选项触发。实现后将作为聚类后的标准优化步骤，显著提升下游可视化（外部工具）的效果。

**相关代码位置**：
- 当前排序实现：`src/libs/phylo/tree/algo.rs`（`sort_by_name`、`ladderize`、`deladderize`、`sort_by_list`）
- CLI 入口：`src/cmd_necom/nwk/order.rs`

### 1.4 输入前提

OLO 需要叶子之间的距离信息。`necom nwk order` 当前仅对树结构进行操作，因此实现时可能需要：
- 接受一个 PHYLIP 距离矩阵作为输入；或
- 接受一个 Pair TSV 作为输入；或
- 复用树的分支长度信息（如果可用）作为近似距离。

具体接口需在实现阶段根据使用场景确定。

## 2. 随机树生成

### 2.1 背景

模拟研究常需要生成随机系统发育树。常见的生成模型包括：
- **Yule 模型**：纯生过程，每个现存谱系以相同速率分裂。
- **Coalescent 模型**：溯祖过程，从当代样本倒推至共同祖先。

### 2.2 necom 规划

`necom nwk` 未来可能提供 `generate_random_tree` 命令（或类似子命令），支持 Yule/Coalescent 模型生成指定叶子数的 Newick 树。当前优先级较低，主要用于模拟研究。

## 3. 已决策不实现的功能

以下功能经过评估后决定不实现：

- **`nwk condense`**：由 `necom nwk subtree --condense` 提供，不计划独立子命令。
- **`nwk match` / `ed` / `gen` / `duration`**：来自 `newick_utils` 的映射，当前没有明确需求。
- **`colless_yule` / `colless_pda` / `sackin_yule` / `sackin_pda`**：标准化统计指标，当前未实现。
- **`inorder` 遍历**：仅适用于二叉树，`necom` 支持多叉树，故不实现。
