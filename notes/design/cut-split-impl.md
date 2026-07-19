# `necom cut` 命令拆分实施文档

## 背景

当前 `necom cut` 单命令承载了 14 个互斥切割方法、动态切割、混合切割、参数扫描、矩阵输入、支持值过滤等大量参数，帮助文档冗长，用户容易误用。本方案将 `cut` 拆为 5 个子命令，使命令职责清晰、参数聚焦。

## 目标

- 用户层：`necom cut <subcommand>`，与 `eval` 的 `compare/partition/replicate` 模式一致。
- 算法层：`src/libs/tree_cut/` 保持现有 API 不变，命令层只做参数转换。
- 不保留旧 `cut` 语法（用户已确认）。

## 拆分后的命令结构

```text
necom cut simple        <infile> --method <METHOD> --threshold <T> [options]
necom cut dynamic       <infile> --min-size <N> [options]
necom cut hybrid        <infile> --matrix <M> --min-size <N> [options]
necom cut scan-simple   <infile> --method <METHOD> --range <start,end,step> [options]
necom cut scan-dynamic  <infile> --range <start,end,step> [options]
```

### `cut simple`

覆盖原所有静态阈值方法：

- `--method` 必填。clap 层允许用户输入连字符形式（如 `root-dist`、`max-clade`）；命令层将连字符替换为下划线后，在 `tree_cut::METHOD_NAMES` 中查找并返回 `&'static str`，再传给算法层。取值：`k`, `height`, `root-dist`, `max-clade`, `avg-clade`, `med-clade`, `sum-branch`, `leaf-dist-max`, `leaf-dist-min`, `leaf-dist-avg`, `max-edge`, `inconsistent`。
- `--threshold` 必填，`f64`；对 `k` 方法运行期校验必须为正整数。
- `--deep` 可选，默认 2，仅 `inconsistent` 使用。
- 公共输出选项：`--format`（cluster/pair，默认 cluster）、`-o/--outfile`、`--rep`（root/first/medoid，默认 root）。
- 公共过滤选项：`--support`（启用支持值过滤）。

### `cut dynamic`

覆盖原 `--dynamic-tree`：

- `--min-size` 必填，最小簇大小。
- `--deep-split` flag。
- `--max-tree-height` 可选 f64。
- 公共输出选项与 `--support` 同上。

### `cut hybrid`

覆盖原 `--dynamic-hybrid`：

- `--matrix` 必填，距离矩阵文件。
- `--min-size` 必填。
- `--max-pam-dist` 可选 f64。
- `--no-pam-dendro` flag。
- `--deep-split` flag。
- `--max-tree-height` 可选 f64。
- 公共输出选项与 `--support` 同上。

### `cut scan-simple`

覆盖原 standard 方法的 `--scan`：

- `--method` 必填。
- `--range <start,end,step>` 必填，严格逗号分隔，不允许空格；step 与 method 阈值同类型（simple 用 f64，dynamic 用 usize）。
- `--deep` 可选，默认 2（method=inconsistent 时有效）。
- `--stats-out` 可选。
- `--support` 可选（flag）。
- 输出固定为 long 格式（`Group\tClusterID\tSampleID`），由 `tree_cut::scan::run_scan` 直接产生，命令层不做 `--format`/`--rep` 转换。

### `cut scan-dynamic`

覆盖原 `--dynamic-tree` 的 `--scan`：

- `--range <start,end,step>` 必填，三个值都必须是整数（因 dynamic-tree 按最小簇大小扫描）。建议在 `scan_dynamic::execute` 解析 range 时一次性完成整数校验并给出清晰错误，而不是依赖 scan 循环中的逐 step 校验。
- `--deep-split` flag。
- `--max-tree-height` 可选 f64。
- `--stats-out` 可选。
- `--support` 可选（flag）。
- 输出固定为 long 格式，命令层不做 `--format`/`--rep` 转换。

> 注意：两个 scan 命令都不包含 `--matrix`、`--max-pam-dist`、`--no-pam-dendro`，因为 hybrid 不支持扫描。

### 直接运行 `necom cut`

不跟子命令时应显示总览帮助（列出 5 个子命令及一句话说明）。`cut/mod.rs` 的 `make_subcommand` 需设置 `.subcommand_required(true).arg_required_else_help(true)`，禁止默认进入某个子命令；顶层 `necom.rs` 的 `arg_required_else_help(true)` 仅控制根命令。

## 源码目录变更

```text
src/cmd_necom/
  cut.rs                 # 删除
  cut/
    mod.rs               # 子命令路由 + 公共辅助函数
    simple.rs            # cut simple
    dynamic.rs           # cut dynamic
    hybrid.rs            # cut hybrid
    scan_simple.rs       # cut scan-simple（模块名 scan_simple；clap 命令名 scan-simple）
    scan_dynamic.rs      # cut scan-dynamic（模块名 scan_dynamic；clap 命令名 scan-dynamic）
```

`src/cmd_necom/mod.rs` 保持 `pub mod cut;` 不变；目录模块自动解析。

## 公共辅助函数（`cut/mod.rs`）

为避免 `simple/dynamic/hybrid` 重复实现：

1. `load_tree(args: &ArgMatches) -> Result<Tree>`
   - 读取 `infile`。
   - 检查输入仅含单棵树。
   - 若 `--support` 存在，调用 `tree_cut::apply_support_filter`。
   - 返回 `Tree`。
2. `OutputOptions { outfile, format, rep_mode }`
   - 用小型 struct 取代元组，避免调用处把 `outfile`/`format`/`rep_mode` 顺序写反。
   - `outfile` 从 `args::get_outfile(args)` 取得。
   - `format` 从 `clust_format` id 取得，默认 `"cluster"`。
   - `rep_mode` 从 `--rep` 解析为 `RepMode`。

格式化输出直接调用 `tree_cut::partition_to_clusters` + `tree_cut::format_clusters`，由各 `execute` 写入文件。

## 各子命令实现映射

### `simple::execute`

1. `load_tree(args)` 加载树。
2. 读取 `--method`、`--threshold`、`--deep`。
3. 将 `--method` 归一化：连字符形式（`root-dist`）替换为下划线（`root_dist`），校验是否属于 `tree_cut::METHOD_NAMES`。
4. 调用 `tree_cut::build_dispatch(tree, Some(name), val, deep, None, None, None, false, false, None, None)`。
5. 调用 `tree_cut::dispatch_cut`。
6. 格式化并输出。

### `dynamic::execute`

1. `load_tree(args)`。
2. 读取 `--min-size`、`--deep-split`、`--max-tree-height`。
3. 调用 `tree_cut::build_dispatch(tree, None, 0.0, 2, Some(min_size), None, max_tree_height, deep_split, false, None, None)`。
4. 调用 `tree_cut::dispatch_cut`。
5. 格式化并输出。

### `hybrid::execute`

1. `load_tree(args)`。
2. 读取 `--matrix` 并用 `NamedMatrix::from_relaxed_phylip` 加载，加 `.with_context(|| format!("Failed to load matrix from {}", matrix_file))` 提供清晰错误。
3. 读取 `--min-size`、`--max-pam-dist`、`--no-pam-dendro`、`--deep-split`、`--max-tree-height`。
4. 调用 `tree_cut::build_dispatch(tree, None, 0.0, 2, None, Some(min_size), max_tree_height, deep_split, no_pam_dendro, max_pam_dist, Some(matrix))`。
5. 调用 `tree_cut::dispatch_cut`。
6. 格式化并输出。

### `scan_simple::execute`

1. `load_tree(args)`。
2. 解析 `--method`（归一化为下划线形式）和 `--range`。
3. 构造 `ScanParams { method_name: Some(name), dynamic_tree: false, ... }`。
4. 初始化 stats_writer（若 `--stats-out`）。
5. 调用 `tree_cut::scan::run_scan`，dynamic 相关参数（`deep_split`, `max_tree_height`, `no_pam_dendro`, `max_pam_dist`）传默认值。

### `scan_dynamic::execute`

1. `load_tree(args)`。
2. 解析 `--range`，并一次性校验三个值均为整数。
3. 构造 `ScanParams { method_name: None, dynamic_tree: true, ... }`。
4. 初始化 stats_writer（若 `--stats-out`）。
5. 调用 `tree_cut::scan::run_scan`，传入 `--deep-split`、`--max-tree-height`，其余参数传默认值。

## 参数帮助函数清理

`src/cmd_necom/args.rs` 中以下函数拆分后不再使用，应删除：

- `dynamic_tree_arg()`
- `dynamic_hybrid_arg()`
- `scan_arg()`（新 `scan-simple` / `scan-dynamic` 使用 `--range`）

以下函数继续保留并在新子命令中使用：

- `rep_arg()`、`support_arg()`、`deep_arg()`：simple / dynamic / hybrid 共用。
- `stats_out_arg()`：scan-simple / scan-dynamic 使用。
- `deep_split_arg()`、`max_tree_height_arg()`：dynamic / hybrid / scan-dynamic 共用。
- `max_pam_dist_arg()`、`no_pam_dendro_arg()`：仅 hybrid 使用。
- `matrix_arg()`：hybrid 使用（也被 `eval partition` 使用，必须保留）。

若实施时发现某些函数最终未被引用，则一并删除。

## 文档更新

1. 创建目录 `docs/help/cut/`。
2. 重写 `docs/help/cut.md` 为总览页，列出 5 个子命令及一句话说明与跳转示例。
3. 新增：
   - `docs/help/cut/simple.md`
   - `docs/help/cut/dynamic.md`
   - `docs/help/cut/hybrid.md`
   - `docs/help/cut/scan-simple.md`
   - `docs/help/cut/scan-dynamic.md`
   - 遵循现有 `docs/help/*.md` 风格（`Section Name:`、`* ` 列表、缩进代码示例）。
4. 同步更新所有仍引用旧 `necom cut` 语法的文档：
   - `docs/cut.md`（用户教程，含 `--scan`、`--max-clade`、`--inconsistent` 等大量示例）
   - `docs/clust.md`（`hier → cut --scan → eval partition` 工作流）
   - `docs/eval-partition.md`
   - `docs/formats.md`
   - `docs/README.md`
   - `docs/help/eval/partition.md`
5. 更新 `src/necom.rs` 的 `after_help`，将 `cut` 一行补充为类似：
   ```text
   * cut   - Cut a Newick tree into flat partitions (simple/dynamic/hybrid/scan-simple/scan-dynamic)
   ```

## 测试迁移

重写以下测试文件，保持断言不变，仅调整命令行参数：

- `tests/cli_cut.rs`
  - `cut ... --k 5` → `cut simple ... --method k --threshold 5`
  - 其余方法类推，建议 `--method` 使用连字符形式（如 `root-dist`）。
  - 负数阈值测试保持 `--threshold=-1.0` 形式，避免 clap 把负号解析为短选项。
- `tests/cli_cut_dynamic.rs`
  - `--dynamic-tree 20` → `cut dynamic ... --min-size 20`
- `tests/cli_cut_hybrid.rs`
  - `--dynamic-hybrid 20` → `cut hybrid ... --min-size 20 --matrix ...`
- `tests/cli_eval_partition_batch.rs`
  - 原 `cut --height 0.5 --scan 0.0,0.6,0.2` 中 `--height 0.5` 是为满足旧 `ArgGroup` 的 dummy 参数；拆分后删除，改为 `cut scan-simple tree.nwk --method height --range 0.0,0.6,0.2`。
- `tests/cli_clust_pipeline.rs`
  - `cut --k 3 --format pair` → `cut simple tree.nwk --method k --threshold 3 --format pair`。

新增：

- `tests/cli_cut_scan_simple.rs` 和 `tests/cli_cut_scan_dynamic.rs`，分别覆盖原 `--scan` 在 standard 方法和 dynamic-tree 下的用法（截至 2026-07-20 尚未创建，将在实施本方案时一并添加）。

补充负面测试：

- 直接运行 `necom cut` 不跟子命令，验证显示总览帮助。
- `cut simple` 缺 `--method` 或 `--threshold`。
- `--method` 取值不在 `METHOD_NAMES` 中。
- `--method k` 但 `--threshold` 不是正整数。
- `cut scan-dynamic --range` 含浮点数（命令层整数校验应拒绝）。
- `--range` 格式非法（非三个值、含空格、含非数字）。
- `--min-size` 为 0 或负数（运行期校验）。
- `cut hybrid` 缺 `--matrix`（从现有 `tests/cli_cut.rs::test_cut_missing_matrix_for_dynamic_hybrid` 迁移）。

## 实施顺序

1. 删除 `src/cmd_necom/cut.rs`。
2. 创建 `src/cmd_necom/cut/mod.rs` 及路由。
   - `make_subcommand` 需设置 `.subcommand_required(true).arg_required_else_help(true)`，使直接运行 `necom cut` 时显示 5 个子命令的总览帮助。
3. 创建 `simple.rs`、`dynamic.rs`、`hybrid.rs`、`scan_simple.rs`、`scan_dynamic.rs` 骨架，`execute` 可先 `unimplemented!()`。
   - 注意：`unimplemented!()` 仅用于本地开发占位，必须在同一 PR 内全部替换为真实实现，不得进入主分支。
4. `cargo check` 确认模块解析正确。
5. 依次实现 `simple`、`dynamic`、`hybrid`、`scan-simple`、`scan-dynamic`。
6. 从 `args.rs` 删除不再使用的帮助函数。
7. 更新文档。
8. 迁移测试。
9. 验证：
   - `cargo fmt`
   - `cargo clippy -- -D warnings`
   - `cargo test`
   - `cargo build --release`
   - 手动检查 `--help` 及各子命令示例输出。

## 风险

1. **模块解析冲突**：必须删除 `cut.rs` 后再创建 `cut/mod.rs`，否则 Rust 优先使用 `cut.rs`。
2. **方法名归一化**：`--method root-dist`（推荐用户输入）与 `--method root_dist`（兼容）都要支持；clap `value_parser` 可只列连字符形式，命令层用 `replace('-', '_')` 统一为下划线后查 `tree_cut::METHOD_NAMES`，返回 `&'static str` 再传给算法层。
3. **`scan` 的 dynamic-tree 整数校验**：`--range` 值在 `scan-dynamic` 模式下必须为整数。建议在 `scan_dynamic::execute` 解析 range 时一次性完成整数校验并给出清晰错误，而不是依赖 scan 循环中 `build_dynamic_tree_dispatch` 的逐 step 校验。
4. **测试数据文件**：只改命令参数，不改 `tests/newick/`、`tests/cut/` 等数据文件。
5. **`build_dispatch` 长参数签名脆弱**：`simple/dynamic/hybrid` 都使用 10 个位置参数的 `build_dispatch`，大量 `None`/`false`/`0.0` 魔术值容易写错顺序。建议在 `cut/mod.rs` 内封装一个命令层专用的 builder 或 wrapper（不改动 `tree_cut` API），将位置参数转换为命名参数，降低误用风险。
6. **旧语法残留引用**：虽然不保留旧 `cut` 语法，但仓库中可能存在脚本、CI、README 或其他文档仍在使用旧参数。实施前应全局搜索 `necom cut` 相关调用，确保全部迁移。
