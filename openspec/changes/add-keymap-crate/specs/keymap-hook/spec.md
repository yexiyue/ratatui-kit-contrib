# keymap-hook

## ADDED Requirements

### Requirement: use_keymap_handler 按 action 分发按键事件
crate SHALL 提供 `Hooks` 扩展 trait `UseKeymapHandler`：`use_keymap_handler(scope, priority, keymap, callback)` 内部组合 core 的 `use_event_handler`，把 `KeyEventKind::Press` 的按键事件转换为 `KeyCombination` 后向 keymap 反查；命中时以 action（及原始按键事件）调用回调并返回其 `EventResult`，未命中或非按键事件 MUST 返回 `Ignored`（事件继续按既有优先级传递）。实现 MUST 仅使用框架 Extension API 稳定面，core 零改动。

#### Scenario: 命中绑定
- **WHEN** 用户按下 keymap 中绑定了某 action 的键
- **THEN** 回调以该 action 被调用，事件按回调返回的 `EventResult` 处理

#### Scenario: 未绑定键不拦截
- **WHEN** 用户按下 keymap 中无绑定的键
- **THEN** hook 返回 `Ignored`，同层其他 handler（如 app shell 键）仍能收到该事件

#### Scenario: 非 Press 事件不处理
- **WHEN** 收到按键 Release/Repeat 或非按键事件
- **THEN** hook 返回 `Ignored`，回调不被调用

### Requirement: 提供可运行示例与文档
crate SHALL 附带 package-local 可运行 example：演示定义 action 枚举、builder 默认表、从 TOML 字符串合并用户覆盖（含一条非法配置的告警展示）、`use_keymap_handler` 驱动的界面响应，以及帮助文案显示当前实际绑定。README MUST 说明「serde 变体名是配置契约」与 crossterm 版本统一约束。

#### Scenario: 示例运行
- **WHEN** 运行 example 并按默认键与覆盖后的键
- **THEN** 界面按 action 语义响应，帮助区域显示的键名与实际绑定一致
