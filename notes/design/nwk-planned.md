# necom nwk 已决策不实现的功能

> **实现状态注记**：截至 2026-07-21，`necom nwk` 主体命令体系已实现（详见
> [`phylo.md`](phylo.md)），Optimal Leaf Ordering 已作为 `necom nwk order --olo` 落地。本文档仅
> 记录已决策不实现的功能。

## 1. 已决策不实现的功能

以下功能经过评估后决定不实现：

- **`nwk condense`**：由 `necom nwk subtree --condense` 提供，不计划独立子命令。
- **`nwk match` / `ed` / `duration`**：来自 `newick_utils` 的映射，当前没有明确需求。
- **`colless_yule` / `colless_pda` / `sackin_yule` / `sackin_pda`**：标准化统计指标，当前未实现。
- **`inorder` 遍历**：仅适用于二叉树，`necom` 支持多叉树，故不实现。
- **`generate_random_tree`（随机树生成）**：Yule/Coalescent 模型生成随机系统发育树，主要用于模拟
  研究。优先级较低，不计划实现。
