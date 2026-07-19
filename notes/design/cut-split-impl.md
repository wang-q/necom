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

- `--method` 必填，取值：`k`, `height`, `root-dist`, `max-clade`, `avg-clade`, `med-clade`, `sum-branch`, `leaf-dist-max`, `leaf-dist-min`, `leaf-dist-avg`, `max-edge`, `inconsistent`。
- `--threshold` 必填，`f64`；对 `k` 方法运行期校验必须为正整数。
- `--deep` 可选，默认 2，仅 `inconsistent` 使用。
- 公共输出选项：`--format`（cluster/pair，默认 cluster）、`-o/--outfile`、`--rep`（root/first/medoid，默认 root）、`--support`。

### `cut dynamic`

覆盖原 `--dynamic-tree`：

- `--min-size` 必填，最小簇大小。
- `--deep-split` flag。
- `--max-tree-height` 可选 f64。
- 公共输出选项同上。

### `cut hybrid`

覆盖原 `--dynamic-hybrid`：

- `--matrix` 必填，距离矩阵文件。
- `--min-size` 必填。
- `--max-pam-dist` 可选 f64。
- `--no-pam-dendro` flag。
- `--deep-split` flag。
- `--max-tree-height` 可选 f64。
- 公共输出选项同上。

### `cut scan-simple`

覆盖原 standard 方法的 `--scan`：

- `--method` 必填。
- `--range <start,end,step>` 必填。
- `--deep` 可选，默认 2（method=inconsistent 时有效）。
- `--stats-out` 可选。
- `--support` 可选。
- 输出固定为 long 格式（`Group\tClusterID\tSampleID`），不提供 `--format`/`--rep`。

### `cut scan-dynamic`

覆盖原 `--dynamic-tree` 的 `--scan`：

- `--range <start,end,step>` 必填。
- `--deep-split` flag。
- `--max-tree-height` 可选 f64。
- `--stats-out` 可选。
- `--support` 可选。
- 输出固定为 long 格式，不提供 `--format`/`--rep`。

> 注意：两个 scan 命令都不包含 `--matrix`、`--max-pam-dist`、`--no-pam-dendro`，因为 hybrid 不支持扫描。

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
2. `output_options(args: &ArgMatches) -> (&str, &str, RepMode)`
   - 返回 `(outfile, format, rep_mode)`。
   - 注意 `args::format_arg()` 的 clap id 是 `clust_format`，提取时用这个 id。

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
2. 读取 `--matrix` 并用 `NamedMatrix::from_relaxed_phylip` 加载。
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
2. 解析 `--range`。
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

若实施时发现某些函数最终未被引用，则一并删除。

## 文档更新

1. 创建目录 `docs/help/cut/`。
2. 重写 `docs/help/cut.md` 为总览页，列出 5 个子命令及跳转示例。
3. 新增：
   - `docs/help/cut/simple.md`
   - `docs/help/cut/dynamic.md`
   - `docs/help/cut/hybrid.md`
   - `docs/help/cut/scan-simple.md`
   - `docs/help/cut/scan-dynamic.md`
   - 遵循现有 `docs/help/*.md` 风格（`Section Name:`、`* ` 列表、缩进代码示例）。
4. 更新 `src/necom.rs` 的 `after_help`，将 `cut` 一行补充为类似：
   ```text
   * cut   - Cut a Newick tree into flat partitions (simple/dynamic/hybrid/scan-simple/scan-dynamic)
   ```

## 测试迁移

重写以下测试文件，保持断言不变，仅调整命令行参数：

- `tests/cli_cut.rs`
  - `cut ... --k 5` → `cut simple ... --method k --threshold 5`
  - 其余方法类推，建议 `--method` 使用连字符形式（如 `root-dist`）。
- `tests/cli_cut_dynamic.rs`
  - `--dynamic-tree 20` → `cut dynamic ... --min-size 20`
- `tests/cli_cut_hybrid.rs`
  - `--dynamic-hybrid 20` → `cut hybrid ... --min-size 20 --matrix ...`
- 新增 `tests/cli_cut_scan_simple.rs` 和 `tests/cli_cut_scan_dynamic.rs`，分别覆盖原 `--scan` 在 standard 方法和 dynamic-tree 下的用法（截至 2026-07-20 尚未创建，将在实施本方案时一并添加）。

## 实施顺序

1. 删除 `src/cmd_necom/cut.rs`。
2. 创建 `src/cmd_necom/cut/mod.rs` 及路由。
3. 创建 `simple.rs`、`dynamic.rs`、`hybrid.rs`、`scan_simple.rs`、`scan_dynamic.rs` 骨架，`execute` 可先 `unimplemented!()`。
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
2. **方法名归一化**：`--method root-dist` 与 `--method root_dist` 都要支持，内部统一为 `root_dist` 后传给 `tree_cut::METHOD_NAMES`。
3. **`scan` 的 dynamic-tree 整数校验**：`--range` 值在 dynamic-tree 模式下必须为整数，可复用 `tree_cut::scan::build_dynamic_tree_dispatch` 的校验逻辑。
4. **测试数据文件**：只改命令参数，不改 `tests/newick/`、`tests/cut/` 等数据文件。
