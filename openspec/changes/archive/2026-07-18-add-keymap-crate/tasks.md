# Tasks: add-keymap-crate

## 1. crate 脚手架

- [x] 1.1 新建 `crates/ratatui-kit-keymap`（workspace members 接入，继承 workspace.package；`crokey`、`toml` 进 workspace.dependencies 钉版，`toml` 为 crate 的 feature 门控依赖）
- [x] 1.2 CI 接入现有 workflow（workspace 全量命令自动覆盖新成员）；本地 `cargo tree` 确认 crossterm 全树统一 0.29.0（crokey ^0.29 与框架一致）

## 2. keymap-core

- [x] 2.1 定义 `Keymap<A>`、`KeymapBuilder<A>`（bind/desc/build，默认表解析失败与自身冲突在 build panic）与 `KeymapWarning` 枚举
- [x] 2.2 实现合并：`KeymapOverrides<A>`（serde 反序列化）、按 action 整条替换、三类校验回退与告警收集（tests：部分覆盖/多键/ParseError/Conflict/UnknownAction/serde 变体名契约）
- [x] 2.3 实现反查 `KeyCombination → Option<A>` 与键名描述（crokey `KeyCombinationFormat`）；re-export `crokey`
- [x] 2.4 `toml` feature：`merge_toml_table` 便捷层与 `to_toml_example()`（含 desc 注释；test：导出可被无告警回读）

## 3. keymap-hook

- [x] 3.1 扩展 trait `UseKeymapHandler`：组合 `use_event_handler`，Press 事件转 `KeyCombination` 查表分发，未命中 `Ignored`（仅用 Extension API 稳定面）
- [x] 3.2 package-local example：action 枚举 + 默认表 + TOML 覆盖（含一条非法配置告警）+ hook 驱动界面 + 帮助区显示实际绑定

## 4. 文档与发布

- [x] 4.1 README（定位、快速上手、「serde 变体名是配置契约」、crossterm 版本约束、与 TRNovel#49 的消费关系）+ rustdoc（`RUSTDOCFLAGS="-D warnings"` 过）
- [x] 4.2 全套检查（test/clippy/fmt/doc）绿后发布 `ratatui-kit-keymap 0.1.0`，通知 TRNovel 侧 change `configurable-keybindings` 可开工
