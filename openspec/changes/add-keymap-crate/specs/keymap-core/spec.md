# keymap-core

## ADDED Requirements

### Requirement: builder 构建默认表并在开发期暴露错误
crate SHALL 提供对 action 枚举泛型的 `Keymap<A>` builder：`bind(action, keys)` 声明默认键位（crokey 语法字符串）、`desc(action, text)` 声明帮助文案。默认表内键位字符串解析失败或两个 action 绑定同一键时，`build()` MUST panic（默认表是开发者代码，错误应在开发期暴露而非运行期吞掉）。

#### Scenario: 正常构建
- **WHEN** builder 以合法键位字符串为若干 action 声明默认键并 build
- **THEN** 得到可用的 `Keymap<A>`，每个 action 的键列表与声明一致

#### Scenario: 默认表自身冲突
- **WHEN** builder 把两个不同 action 绑到同一键并 build
- **THEN** build panic 并指出冲突的键与 action

### Requirement: 用户覆盖按 action 整条替换合并
`Keymap<A>` SHALL 提供合并接口：接受 serde 反序列化得到的用户覆盖（action 名 → 键位字符串列表），用户写了的 action 以其键列表整体替换默认，未写的 action MUST 保持默认。action 的配置键名 MUST 取自 serde 变体名（宿主经 `#[serde(rename_all)]` 控制风格）。

#### Scenario: 部分覆盖
- **WHEN** 用户覆盖仅包含 `page_down = ["ctrl+d"]`
- **THEN** 合并后 `page_down` 仅绑定 ctrl+d（默认键不再生效），其余 action 保持默认键位

#### Scenario: 一个 action 多个键
- **WHEN** 用户覆盖写 `page_down = ["ctrl+d", "space"]`
- **THEN** 合并后两个键都反查命中该 action

### Requirement: 合并校验产出结构化告警且不中断
合并 SHALL 校验用户覆盖并返回结构化告警列表（而非 Err/panic）：键位字符串解析失败 → 该 action 整条回退默认（`ParseError`）；合并后同表两 action 同键 → 冲突的用户覆盖整条回退默认（`Conflict`）；action 名不存在 → 忽略该条（`UnknownAction`）。任何非法输入 MUST NOT 使合并失败或破坏其余合法条目。

#### Scenario: 非法键位字符串
- **WHEN** 用户覆盖中某 action 的键位字符串无法解析
- **THEN** 该 action 回退默认键位，返回含 `ParseError` 的告警，其余条目正常合并

#### Scenario: 覆盖引入冲突
- **WHEN** 用户把两个 action 覆盖到同一键
- **THEN** 两个 action 均回退默认键位，返回含 `Conflict` 的告警

#### Scenario: 未知 action 名
- **WHEN** 用户覆盖包含枚举中不存在的 action 名（拼写错误或已删除的旧配置）
- **THEN** 该条被忽略，返回含 `UnknownAction` 的告警

### Requirement: 反查与键名描述
`Keymap<A>` SHALL 提供 `KeyCombination → Option<A>` 反查（供事件分发）与 action → 可读键名字符串列表的描述接口（基于 crokey 格式化，供帮助界面渲染当前实际绑定）。crate SHALL re-export `crokey`（含 `KeyCombination`、`KeyCombinationFormat`）。

#### Scenario: 反查命中与未命中
- **WHEN** 以某键的 `KeyCombination` 反查
- **THEN** 该键有绑定时返回对应 action，无绑定时返回 None

#### Scenario: 描述反映用户覆盖
- **WHEN** 用户把某 action 覆盖为 ctrl+d 后请求其键名描述
- **THEN** 返回的字符串列表为 ctrl+d 的格式化结果，而非默认键

### Requirement: 导出带注释的示例配置
`toml` feature 下 `Keymap<A>` SHALL 提供 `to_toml_example()`：从默认表生成完整示例配置文本，包含全部 action 的键名、默认键位与 desc 注释，用户可直接以其为模板修改。

#### Scenario: 示例导出完整性
- **WHEN** 对含 N 个 action 的默认表调用 `to_toml_example()`
- **THEN** 输出包含全部 N 个 action 条目、各自默认键位字符串及 desc 注释，且该输出可被合并接口无告警地解析
