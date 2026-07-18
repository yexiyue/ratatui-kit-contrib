# Proposal: add-keymap-crate

## Why

ratatui-kit 应用的快捷键普遍硬编码在各组件的 `use_event_handler` 里（TRNovel 有约 22 个文件如此，用户已提出自定义按键诉求 TRNovel#49）。框架 core 已提供事件**机制**（分发/优先级/输入层），但缺一层「物理键 → 语义 action」的可配置映射**策略**。该能力对所有 ratatui-kit 用户通用，适合作为官方 contrib 扩展 crate 落地，而非塞进 core（core 不应绑定 crokey 依赖与配置格式意见）。

## What Changes

- 新增 crate `crates/ratatui-kit-keymap`（独立发版 `ratatui-kit-keymap`，初版 0.1.0）：
  - `Keymap<A>`：对 action 枚举泛型的键位表。一个 `Keymap<A>` 即一个 scope（每 scope 一个 action 枚举），多 scope 由宿主 app 组合多个实例。
  - builder 声明代码内默认键（含 `desc` 帮助文案），build 时断言默认表自身无冲突；键位语法与解析/格式化基于 `crokey`。
  - 用户覆盖合并：按 action 整条替换；校验非法键位字符串、未知 action 名、同表键冲突，返回结构化告警列表（呈现方式交给宿主 app）。
  - 反查（`KeyCombination → Option<A>`）与键名描述（action → 可读键名列表，供帮助界面渲染）。
  - `to_toml_example()`：从默认表生成带注释的完整示例配置（全部 action、默认键、desc）。
  - hook `use_keymap_handler`：组合 core 的 `use_event_handler`，按键事件查表命中后回调 action，未命中自动 `Ignored`；仅使用 Extension API 稳定面。
- **不含 proc-macro**（derive 声明默认键的糖留待 API 经首个消费者 TRNovel 实战验证后的 0.2）。
- 新增依赖：`crokey`（其 crossterm ^0.29 与框架 crossterm 0.29 同 major，cargo 统一版本，类型互通）、`toml`（feature 门控，用于合并输入与示例导出）。

## Capabilities

### New Capabilities

- `keymap-core`: 泛型键位表的定义、默认表构建、用户覆盖合并与校验、反查与键名描述、示例配置导出。
- `keymap-hook`: 与 ratatui-kit 事件系统集成的 `use_keymap_handler` hook，经 Extension API 稳定面实现。

### Modified Capabilities

（无 —— 新仓库首个 openspec change，无既存 capability。）

## Impact

- **新增代码**：`crates/ratatui-kit-keymap/`（含 README、package-local example）。
- **workspace**：`Cargo.toml` members 增加该 crate；`workspace.dependencies` 增加 `crokey`、`toml` 钉版。
- **依赖约束**：crossterm 类型经 `ratatui_kit::crossterm` 取用（workspace 既有护栏）；crokey 的 crossterm 为传递依赖，与框架同 major 由 cargo 统一。
- **框架 core**：零改动（Extension API 稳定面已覆盖所需：`Hooks::use_hook`、`UseEventHandler`、`EventResult/EventPriority/EventScope`）。
- **首个消费者**：TRNovel（change `configurable-keybindings`）依赖本 crate 实现阅读页按键自定义，跨仓库排序为本 crate 0.1.0 先行。
