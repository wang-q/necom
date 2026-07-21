# necom nwk 未来工作计划

> **实现状态注记**：本文档记录 `necom nwk` 命令的**未实现工作计划**与已决策不实现的功能。截至
> 2026-07-21，`necom nwk` 主体命令体系已实现（详见 [`phylo.md`](phylo.md)），Optimal Leaf Ordering
> 已作为 `necom nwk order --olo` 落地。本文档仅保留随机树生成（§1）与已决策不实现的功能（§2）。

## 1. 随机树生成

### 1.1 背景

模拟研究常需要生成随机系统发育树。常见的生成模型包括：

- **Yule 模型**：纯生过程，每个现存谱系以相同速率分裂。
- **Coalescent 模型**：溯祖过程，从当代样本倒推至共同祖先。

### 1.2 necom 规划

`necom nwk` 未来可能提供 `generate_random_tree` 命令（或类似子命令），支持 Yule/Coalescent
模型生成指定叶子数的 Newick 树。当前优先级较低，主要用于模拟研究。

## 2. 已决策不实现的功能

以下功能经过评估后决定不实现：

- **`nwk condense`**：由 `necom nwk subtree --condense` 提供，不计划独立子命令。
- **`nwk match` / `ed` / `gen` / `duration`**：来自 `newick_utils` 的映射，当前没有明确需求。
- **`colless_yule` / `colless_pda` / `sackin_yule` / `sackin_pda`**：标准化统计指标，当前未实现。
- **`inorder` 遍历**：仅适用于二叉树，`necom` 支持多叉树，故不实现。

