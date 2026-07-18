# Design: add-keymap-crate

## Context

contrib 仓库是官方扩展 crate 的家：独立发版 `ratatui-kit-<name>`，经框架公开 Extension API（semver 保证面含 `Hooks::use_hook`、`UseEventHandler`、`EventResult/EventPriority/EventScope`、`Context`、feature 门控的 `UseAtom`）集成，crossterm 类型经 `ratatui_kit::crossterm` 取用、不直连 ratatui。既有成员 `ratatui-kit-markdown`、`ratatui-kit-themes` 树立了模式：core 提供机制协议，contrib 提供策略与电池。

需求源头是 TRNovel#49（用户自定义按键），其 TRNovel 侧方案（change `configurable-keybindings`）已完成完整调研与设计：主流 TUI（yazi/helix/gitui）共性为语义 action + scope 分层 + 代码内默认 + 用户文件部分覆盖 + 帮助动态生成；键位解析选型 `crokey`（broot 作者维护、v1.4、约 410 万下载，提供 `KeyCombination` 解析/格式化/serde）。本 change 把其中与宿主无关的通用层上移到 contrib。

## Goals / Non-Goals

**Goals:**

- 任何 ratatui-kit 应用引入本 crate 后，只需定义 action 枚举 + 默认表 + 配置文件路径，即获得完整的按键自定义能力（解析、合并、校验、反查、帮助键名、示例导出）。
- API 对 action 枚举泛型、对配置来源中立（core 逻辑 serde 驱动；toml 便捷层 feature 门控）。
- 仅依赖 Extension API 稳定面，框架 core 零改动。
- 校验产出结构化告警而非 panic/Err 中断——按键配置坏了应用必须照常可用。

**Non-Goals:**

- 不做 proc-macro（derive 默认键声明留 0.2，待 TRNovel 实战验证 API）。
- 不做多键序列（vim 式 `g g`）与 crokey `Combiner`（Kitty 协议多普通键组合）。
- 不做配置文件的读写/落盘（宿主 app 决定路径、读取与告警呈现）。
- 不做通用帮助浮层组件（后续可另起 crate 或版本迭代）。

## Decisions

### D1：一个 `Keymap<A>` 即一个 scope，多 scope 由 app 组合

泛型参数 `A` 为宿主定义的 action 枚举（约束 `Copy + Eq + Hash + Serialize + DeserializeOwned`，unit variants）。scope 即类型边界，天然避免跨 scope 的 action 混用与巨型全局枚举；宿主把多个 `Keymap<A1>`、`Keymap<A2>` 组进自己的配置结构（TOML 中每 scope 一个表名）。备选「crate 内建 scope 容器」被否：容器要抹平类型（trait object / 字符串 action），失去编译期保证，而组合多实例对 app 是零成本。

### D2：action 的配置键名取自 serde 变体名

TOML 中的 action 键名（如 `page_down`）通过序列化 unit variant 得到，宿主用 `#[serde(rename_all = "snake_case")]` 控制风格。不引入 strum、不要求 builder 重复写字符串名——单一事实来源是枚举定义本身。未知键名（拼写错误或宿主已删 action 的旧配置）在合并时产出 `UnknownAction` 告警并忽略该条，保证向后兼容。

### D3：builder 构建默认表，build 时断言无冲突

`Keymap::builder().bind(A::PageDown, ["pagedown"]).desc(A::PageDown, "向下翻页")...build()`。默认表键位字符串用 crokey 编译期 `key!` 宏或运行时解析均可（builder 接受 `IntoIterator<Item = &str>`，解析失败在 build 时 panic——默认表是开发者代码，错误应在开发期暴露；用户配置的解析失败才走告警）。同表两 action 绑同键同样 build panic。`desc` 供帮助渲染与示例导出的注释。

### D4：合并语义 = 按 action 整条替换，校验产出结构化告警

`merge(overrides) -> Vec<KeymapWarning>`：用户写了某 action 即以其键列表整体替换默认；未写的保持默认。`KeymapWarning` 枚举：`ParseError { action, input }`（该 action 回退默认）、`Conflict { key, actions }`（冲突的用户覆盖整条回退默认）、`UnknownAction { name }`（忽略该条）。overrides 类型为 `KeymapOverrides<A>`（serde 反序列化得到，`HashMap<String, Vec<String>>` 语义），核心 API 与格式无关；`toml` feature 提供 `merge_toml_table(&toml::Table)` 与 `to_toml_example()` 便捷层。

### D5：hook 以参数注入 keymap，不做 Provider

`hooks.use_keymap_handler(scope, priority, keymap, |action, key_event| -> EventResult)` 作为 `Hooks` 的扩展 trait（`UseKeymapHandler`），内部组合 `use_event_handler`：`KeyEvent → KeyCombination::from → keymap 反查 → 命中回调 / 未命中 Ignored`。keymap 如何全局分发（Context、Atom、props）是宿主的架构选择（TRNovel 用 Atom），crate 不代替决定；`KeymapProvider` 组件留待多消费者出现真实需求再加。仅处理 `KeyEventKind::Press`，与各 app 现行约定一致。

### D6：crokey 直接暴露在公开 API（`KeyCombination` 类型）

不包一层自有键位类型。crokey 的 crossterm ^0.29 与框架 0.29.0 同 major、cargo 统一，`KeyCombination::from(crossterm::event::KeyEvent)` 类型互通；re-export `crokey`（含 `KeyCombinationFormat`）供宿主做帮助渲染。代价是 crokey 升 major 时本 crate 也 breaking——可接受，crokey API 已 1.x 稳定多年。

## Risks / Trade-offs

- [crokey 与框架的 crossterm 各自升版后错开 major] → workspace 钉版 + CI `cargo tree -d` 检查重复 crossterm；升级时两侧一起动（同仓库 workspace 统一管理）。
- [serde 变体名做配置键名，宿主改枚举名即破坏用户配置] → README 明示「变体名是配置契约」，建议宿主用 `#[serde(rename_all)]` 固定风格并把改名视为 breaking；`UnknownAction` 告警兜底旧配置。
- [无宏导致默认表声明啰嗦] → builder 链式 API 尽量紧凑；这是刻意取舍，0.2 的 derive 宏在 API 验证后补糖。
- [首版 API 面向 TRNovel 单一消费者设计，可能带偏通用性] → 设计已刻意剥离宿主决策（文件 IO、告警 UI、全局分发均不进 crate）；0.1 标注实验性，语义化版本兜底。

## Migration Plan

1. 本 crate 在 contrib 落地并发布 0.1.0（workspace 接入、README、example、CI 绿）。
2. TRNovel（change `configurable-keybindings`）钉版消费，作为首个真实消费者回灌 API 反馈。
3. 回滚策略：crate 独立发版，出问题宿主锁旧版即可；框架 core 零改动，无回滚面。

## Open Questions

- `to_toml_example()` 注释的语言（中/英）——倾向英文默认 + desc 原样输出（desc 由宿主用自己的语言写）。
