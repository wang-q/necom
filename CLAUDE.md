# CLAUDE.md

此文件是我（AI 助手）在本仓库工作时的行为准则。所有规则都是硬性要求，除非用户明确覆盖。

## 语言规则

- **与用户交流**: 中文
- 本文件 (`CLAUDE.md`) : 使用中文编写
- **代码注释 (doc comments `///` `//!` 和行内 `//`)**: 英文
- **Git 提交信息**: 英文
- **文档正文** (如 `docs/*.md` 中的说明文字): 英文
- **Notes** (如 `notes/*.md` 中的供我自己看的说明文字): 中文

## 代码风格

**权衡：** 这些准则偏向谨慎而非速度。对于简单任务，自行判断。

### 编码前先思考

**不要假设。不要隐藏困惑。呈现权衡。**

在实现之前：
- 明确陈述你的假设。如果不确定，就问。
- 如果存在多种理解，把它们都列出来 — 不要默默地选一个。
- 如果有更简单的方法，说出来。必要时提出反对意见。
- 如果有不明白的地方，停下来。指出困惑之处。问。

### 简洁优先

**用最少的代码解决问题。不做任何推测性设计。**

- 不添加未被要求的功能。
- 不添加未被要求的"灵活性"或"可配置性"。
- 不为不可能发生的场景写错误处理。
- 如果你写了 200 行但其实 50 行就够了，重写它。

问自己："资深工程师会觉得这过于复杂吗？" 如果是，简化。

### 分层原则

**复杂逻辑放 `libs/`，`cmd_necom/` 保持薄壳。**

- `src/libs/` 是复杂逻辑、算法、格式 I/O、共享工具的归宿。
- `src/cmd_necom/` 仅负责：CLI 参数解析、参数转换、调用 `libs`、输出格式化。
- 单命令专用的复杂逻辑也放 `libs/`，即使当前只有一个消费者。
- 命令文件中内联的算法/业务逻辑应回迁 `libs/`。

判断标准：涉及算法、数据结构、复杂流程控制的代码属 `libs/`；只是 `clap` 参数 → 调用 → 打印的代码属 `cmd_necom/`。

反例：在 `cmd_necom/foo.rs` 里实现距离计算函数 → 应迁到 `libs/`。
正例：`cmd_necom/foo.rs` 只做 `let args = parse(matches); let result = libs::foo::run(args); println!("{result}")`。

> 注："三次相似代码"原则（同一模式出现三次后再抽象）针对的是重复代码的抽象提取，与本节的代码分层无关。

### 精准修改

**只改必须改的。只清理自己造成的混乱。**

编辑现有代码时：
- 不要"改进"相邻的代码、注释或格式。
- 不要重构没有坏的东西。
- 匹配现有风格，即使你不会这样写。
- 如果你注意到无关的死代码，提出来 — 不要删除它。

当你的修改产生了孤立代码时：
- 删除因你的修改而变得未使用的 import/变量/函数。
- 不要删除之前就存在的死代码，除非被要求。

检验标准：每一行改动都应该能追溯到用户的请求。

### 目标驱动执行

**定义成功标准。循环直到验证通过。**

将任务转化为可验证的目标：
- "添加验证" → "为无效输入写测试，然后让它们通过"
- "修复 bug" → "写一个能复现它的测试，然后让它通过"
- "重构 X" → "确保重构前后测试都通过"

对于多步骤任务，陈述简要计划：
```
1. [步骤] → 验证: [检查]
2. [步骤] → 验证: [检查]
3. [步骤] → 验证: [检查]
```

强有力的成功标准让你可以独立循环。薄弱的标准（"让它能用"）需要不断澄清。

### 必须遵守

- 每个 PR / commit 跑 `cargo fmt` 和 `cargo clippy -- -D warnings`，clean 之后再提交
- 公共 API (pub fn / pub struct / pub trait) 必须写英文 doc comment；一行即可，trait 定义或复杂不变量可例外
- 不写冗余注释 — 如果函数名和类型签名已经说明了行为，不要画蛇添足
- 用 `anyhow::Result<T>` 做函数返回值，`anyhow::bail!` / `anyhow::anyhow!` 构造错误

### 禁止

- 不要引入新依赖，除非用户明确要求
- 不要为了"可能"的未来需求写抽象 — 三次相似代码出现之后再考虑提取
- 不要写半成品实现 — stub / TODO 必须有明确的后续任务链接
- 不要用 `unsafe`，除非有充分理由且用户同意
- 不要写反向兼容的 shim（rename `_vars`、re-export 旧类型等）

## 项目概览

**当前状态**: 活跃开发中 | **主要语言**: Rust

**目录约定**: 任何被 `.gitignore` 完全忽略的目录，均仅作为参考资料，**不是本项目的一部分**。

这个 `necom` clone 已被精简为聚焦聚类、距离矩阵与系统发育树的工具集。它仅保留以下命令：

- `necom clust` — 聚类算法与评估
- `necom mat` — 距离矩阵处理
- `necom nwk` — Newick 树操作与可视化
- `necom pl condense` — 基于分类学的树压缩流程

其他命令（`fa`、`fas`、`fq`、`gff`、`axt`、`chain`、`lav`、`maf`、`ms`、`net`、`paf`、`plot`、`psl`、`twobit`、`dist` 以及 `pl` 的其余子命令）均已被移除。

## 构建命令

### 构建

```bash
# 开发构建
cargo build

# 发布构建 (高性能)
cargo build --release
```

### 测试

```bash
# 运行所有测试
cargo test
```

## 架构

### 源代码组织

- **`src/necom.rs`** - 主程序入口，负责命令行解析和分发。
    - 使用 `clap` 进行参数解析。
    - 在 `main` 函数中注册所有子命令模块。
- **`src/lib.rs`** - 库入口，导出模块。
- **`src/cmd_necom/`** - 命令实现模块。当前仅保留：
    - `clust` (Clustering)
    - `mat` (Matrix)
    - `nwk` (Phylogeny/Newick)
    - `pl` (Pipelines，仅 `condense` 子命令)
- **`src/libs/`** - 共享工具库和核心逻辑。当前仅保留：
  - **`clust/`** - 聚类算法实现。
    - **`hier.rs`**: 层次聚类 (NN-chain 算法)。
    - **`dbscan.rs`, `mcl.rs`, `k_medoids.rs`**: 其他聚类算法。
    - **`nj.rs`, `upgma.rs`**: 建树算法 (Neighbor-Joining, UPGMA)。
    - **`tree_cut/`**: 树切分算法。
    - **`eval/`**: 聚类评估指标。
  - **`phylo/`** - 系统发育分析核心库。
    - **`node.rs`/`parser.rs`/`error.rs`**: 树节点定义、Newick 解析、错误类型（位于 `phylo/` 根级）。
    - **`cmp.rs`**: 树拓扑比较 (`TreeComparison` trait，Robinson-Foulds 等)。
    - **`tree/`**: 树算法与操作。
      - `ops.rs`/`algo.rs`: 节点操作 (add/remove/reroot/prune 等) 与算法。
      - `traversal.rs`/`query.rs`: 遍历 (pre/post/level-order) 与查询 (LCA/路径/距离/单系性)。
      - `stat.rs`/`balance.rs`/`distance.rs`/`support.rs`: 统计、平衡性指标、距离、支持值。
      - `io/`: 格式 I/O — Newick/DOT/SVG/Forest。
  - **`pairmat/`** - 配对距离矩阵。
  - **`linalg.rs`** - 线性代数/距离计算辅助。
  - **`par.rs`** - 并行计算辅助。
  - **`io.rs`** - I/O 辅助函数。
  - **`pl/`** - `pl condense` 的共享流程上下文。

## 关键设计文档

- **`docs/`**: 用户面向命令文档（英文）。当前保留的文档包括 `clust.md`、`mat.md`、`nwk.md`，以及 `docs/formats/` 下的 `README.md` 和 `distance.md`。
- **`notes/`**: 开发者面向笔记（中文）：`notes/design/` 下保留与 `clust` / `nwk` / `phylo` 相关的设计稿，其余笔记已随命令移除而删除。

## 命令结构 (Command Structure)

每个命令在 `src/cmd_necom/` 下作为一个独立的模块实现，通常包含两个公开函数：

1.  **`make_subcommand`**: 定义命令行接口。
    -   返回 `clap::Command`。
    -   使用 `.about(...)` 设置简短描述 (第三人称单数)。
    - 推荐使用 `.after_help(include_str!("../../docs/help/<category>/<cmd>.md"))` 引入详细文档。
2.  **`execute`**: 命令执行逻辑。
    -   接收 `&clap::ArgMatches`。
    -   返回 `anyhow::Result<()>`。

### 关键依赖

- **`clap`**: 命令行参数解析。
- **`anyhow`**: 错误处理。
- **`rayon`**: 并行计算。
- **`nom`**: 文本解析 (Newick 等)。
- **`regex`**: 正则表达式。
- **`intspan`**: 区间集合数据结构。
- **`petgraph`**: 图结构。
- **`indexmap`**: 保序 HashMap (名称→id 映射统一模式)。
- **`rand` + `num-traits`**: 随机化与数值计算（聚类算法）。

## 开发工作流

### 添加新命令

1.  在 `src/cmd_necom/` 下相应的类别目录中创建新文件 (或新建目录)。
2.  在 `src/cmd_necom/mod.rs` (或子目录的 `mod.rs`) 中声明该模块。
3.  在 `src/necom.rs` 中注册该子命令。
4.  实现 `make_subcommand` 和 `execute`。
5.  添加测试文件 `tests/cli_<command>.rs`。

### 测试约定

- 集成测试位于 `tests/` 目录下，文件命名为 `cli_<command>.rs`。
- 测试数据通常放在 `tests/<command>/` 目录下。
- **推荐使用 `NecomCmd` 辅助结构体**（定义在 `tests/common/mod.rs`）来编写集成测试，以简化子进程调用和断言。
- 测试函数**不需要**返回 `anyhow::Result<()>`，也不需要以 `Ok(())` 结尾。直接在函数体中执行断言即可。
- 必须使用 `assert_cmd` 来定位二进制文件，以兼容自定义构建目录。
- **稳定性原则 (Zero Panic)**: 任何用户输入（包括畸形数据、二进制文件）都不应导致程序 Panic。必须捕获所有错误并返回友好的错误信息。
- **基准测试**: 性能敏感的变更必须伴随 `benches/` 下的基准测试结果（使用 `criterion`）。

- **文档示例**: 使用 `ignore` 属性标记文档中的代码示例，仅用于展示 API 用法，不作为测试执行
  **原因**: doctest 编译和执行速度较慢，会显著增加 `cargo test` 的运行时间。将示例标记为 `ignore` 可以保持文档的完整性，同时确保测试快速执行。

## 帮助文本规范 (Help Text Style Guide)

### Rust 实现规范 (Implementation)

* **`about`**: 使用第三人称单数动词，简要描述操作。
* **`after_help`**: 使用 `include_str!("../../docs/help/<category>/<cmd>.md")` 引入外部文档。
* **Arguments**:
    * **Input**: 命名为 `infile` (单文件) 或 `infiles` (多文件)。
    * **Output**: 命名为 `outfile` (`-o`, `--outfile`)。

### 文档内容规范 (Markdown Content)

所有子命令的帮助文档 (`docs/help/<category>/<cmd>.md`) 必须遵循以下统一风格：

1. **简述 (Description)**:
    * 标题后紧跟简洁的功能描述，可以比 .about(...) 更详细，但不能超过两行。

2. **分节结构**（按顺序）:
    * **Behavior** (可选): 命令的核心行为说明。
    * **Input**: 输入源说明（使用标准格式）。
    * **Output** (如适用): 输出说明。
    - **Notes**: Bullet points starting with `*`.
    * **Examples**: 使用示例。

3. **节标题格式**:
    * 使用 `Section Name:`（首字母大写，后跟冒号）。
    * **不要**使用 Markdown 的 `##`。
    * **不要**包含 `Usage:` 或 `Options:` 小节（由 `clap` 自动生成）。

4. **内容格式**:
    * **列表**: 使用 `* `（星号 + 1 个空格）引导无序列表。
        * 子项使用 `    * `（4空格缩进 + 星号 + 1 个空格）。
    * **代码示例**: 使用缩进（4空格）而非 ` ``` `。
    * 例外：多行命令示例（如包含 `\` 换行的命令）可使用 ` ``` ` 代码块。
    * **参数引用**: 使用反引号包裹，如 `` `--header` / `-H` ``。

5. **示例 (Examples)**:
    * 使用 `Examples:` 作为标题。
    * 采用编号列表 (`1. `, `2. `)，末尾不要句号或冒号。
    * 下一行命令示例（3 空格缩进, 用反引号包裹单行命令）。
