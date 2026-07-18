use crokey::key;
use serde::{Deserialize, Serialize};

use super::{Keymap, KeymapOverrides, KeymapWarning};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Action {
    ScrollUp,
    ScrollDown,
    PageDown,
    Quit,
}

fn keymap() -> Keymap<Action> {
    Keymap::builder()
        .bind(Action::ScrollUp, ["k", "up"])
        .desc(Action::ScrollUp, "向上滚动")
        .bind(Action::ScrollDown, ["j", "down"])
        .bind(Action::PageDown, ["pagedown"])
        .desc(Action::PageDown, "向下翻页")
        .bind(Action::Quit, ["q"])
        .build()
}

fn overrides(pairs: &[(&str, &[&str])]) -> KeymapOverrides {
    pairs
        .iter()
        .map(|(name, keys)| {
            (
                *name,
                keys.iter().map(|k| k.to_string()).collect::<Vec<_>>(),
            )
        })
        .collect()
}

#[test]
fn defaults_lookup_and_describe() {
    let km = keymap();
    assert_eq!(km.action_for(key!(k)), Some(Action::ScrollUp));
    assert_eq!(km.action_for(key!(up)), Some(Action::ScrollUp));
    assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
    assert_eq!(km.action_for(key!(x)), None);
    assert_eq!(km.describe(Action::ScrollUp), vec!["k", "Up"]);
    assert_eq!(km.desc(Action::PageDown), Some("向下翻页"));
    // 声明顺序 + serde 变体名契约。
    let names: Vec<_> = km.entries().map(|e| e.name.to_string()).collect();
    assert_eq!(names, ["scroll_up", "scroll_down", "page_down", "quit"]);
}

#[test]
#[should_panic(expected = "bound twice")]
fn duplicate_bind_panics() {
    let _ = Keymap::builder()
        .bind(Action::Quit, ["q"])
        .bind(Action::Quit, ["x"]);
}

#[test]
#[should_panic(expected = "invalid default key")]
fn bad_default_key_panics() {
    let _ = Keymap::builder().bind(Action::Quit, ["ctrl-"]);
}

#[test]
#[should_panic(expected = "bound to both")]
fn conflicting_defaults_panic() {
    let _ = Keymap::builder()
        .bind(Action::ScrollUp, ["k"])
        .bind(Action::ScrollDown, ["k"])
        .build();
}

#[test]
fn partial_override_replaces_whole_action() {
    let mut km = keymap();
    let warnings = km.merge(overrides(&[("page_down", &["ctrl-d", "space"])]));
    assert!(warnings.is_empty());
    // 覆盖后新键生效、默认键整条失效。
    assert_eq!(km.action_for(key!(ctrl - d)), Some(Action::PageDown));
    assert_eq!(km.action_for(key!(space)), Some(Action::PageDown));
    assert_eq!(km.action_for(key!(pagedown)), None);
    // 未覆盖的 action 不受影响。
    assert_eq!(km.action_for(key!(j)), Some(Action::ScrollDown));
    // 帮助描述反映覆盖。
    assert_eq!(km.describe(Action::PageDown), vec!["Ctrl-d", "Space"]);
}

#[test]
fn parse_error_falls_back_to_default() {
    let mut km = keymap();
    let warnings = km.merge(overrides(&[("page_down", &["ctrl-"])]));
    assert_eq!(warnings.len(), 1);
    assert!(matches!(
        &warnings[0],
        KeymapWarning::ParseError { action, input, .. }
            if action == "page_down" && input == "ctrl-"
    ));
    assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
}

#[test]
fn unknown_action_is_ignored() {
    let mut km = keymap();
    let warnings = km.merge(overrides(&[("page_dwon", &["ctrl-d"])]));
    assert_eq!(
        warnings,
        vec![KeymapWarning::UnknownAction {
            name: "page_dwon".into()
        }]
    );
    assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
}

#[test]
fn conflicting_overrides_revert_to_defaults() {
    let mut km = keymap();
    // 两个 action 都覆盖到同一键 → 双双回退默认。
    let warnings = km.merge(overrides(&[
        ("page_down", &["ctrl-d"]),
        ("quit", &["ctrl-d"]),
    ]));
    assert!(
        matches!(&warnings[..], [KeymapWarning::Conflict { actions, .. }]
        if actions.contains(&"page_down".to_string()) && actions.contains(&"quit".to_string()))
    );
    assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
    assert_eq!(km.action_for(key!(q)), Some(Action::Quit));
    assert_eq!(km.action_for(key!(ctrl - d)), None);
}

#[test]
fn override_conflicting_with_default_reverts_override_only() {
    let mut km = keymap();
    // quit 覆盖到 j,与 scroll_down 的默认键相撞 → quit 回退,scroll_down 不动。
    let warnings = km.merge(overrides(&[("quit", &["j"])]));
    assert_eq!(warnings.len(), 1);
    assert_eq!(km.action_for(key!(j)), Some(Action::ScrollDown));
    assert_eq!(km.action_for(key!(q)), Some(Action::Quit));
}

#[test]
fn chained_revert_converges() {
    let mut km = keymap();
    // quit 覆盖到 j(撞 scroll_down 默认) → quit 回退到 q;
    // 而 page_down 覆盖到 q(撞回退后的 quit 默认) → page_down 也回退。
    let warnings = km.merge(overrides(&[("quit", &["j"]), ("page_down", &["q"])]));
    assert_eq!(warnings.len(), 2);
    assert_eq!(km.action_for(key!(j)), Some(Action::ScrollDown));
    assert_eq!(km.action_for(key!(q)), Some(Action::Quit));
    assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
}

#[test]
fn empty_override_unbinds_action() {
    let mut km = keymap();
    // 显式空列表 = 有意解绑(文档化行为),不产生告警。
    let warnings = km.merge(overrides(&[("quit", &[])]));
    assert!(warnings.is_empty());
    assert_eq!(km.action_for(key!(q)), None);
    assert!(km.keys(Action::Quit).is_empty());
}

#[test]
fn duplicate_keys_in_override_are_deduplicated() {
    let mut km = keymap();
    // 同一 action 内重复键去重,不得误报为跨 action 冲突并回退。
    let warnings = km.merge(overrides(&[("page_down", &["ctrl-d", "ctrl-d"])]));
    assert!(warnings.is_empty(), "got: {warnings:?}");
    assert_eq!(km.action_for(key!(ctrl - d)), Some(Action::PageDown));
    assert_eq!(km.keys(Action::PageDown).len(), 1);
}

#[test]
fn parse_error_reverts_to_defaults_across_layered_merges() {
    let mut km = keymap();
    assert!(km.merge(overrides(&[("quit", &["x"])])).is_empty());
    // 第二层的非法覆盖必须回到「默认」而非停留在上一层的 x
    // —— 告警文案承诺的是 using default keys。
    let warnings = km.merge(overrides(&[("quit", &["ctrl-"])]));
    assert_eq!(warnings.len(), 1);
    assert_eq!(km.action_for(key!(q)), Some(Action::Quit));
    assert_eq!(km.action_for(key!(x)), None);
}

#[test]
fn uppercase_letter_implies_shift() {
    let mut km = keymap();
    // 大写字母习惯写法按 shift 意图处理(crokey parse 本身会整体转小写)。
    let warnings = km.merge(overrides(&[("page_down", &["G"])]));
    assert!(warnings.is_empty());
    use crokey::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let shift_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
    assert_eq!(km.action_for(shift_g), Some(Action::PageDown));
    assert_eq!(km.action_for(key!(g)), None);
}

#[test]
fn chord_override_is_rejected_with_warning() {
    let mut km = keymap();
    // 事件侧永远只产生单码组合,多键 chord 是永不命中的死绑定,必须拒绝。
    let warnings = km.merge(overrides(&[("page_down", &["g-g"])]));
    assert!(
        matches!(&warnings[..], [KeymapWarning::ParseError { message, .. }]
        if message.contains("multi-key"))
    );
    assert_eq!(km.action_for(key!(pagedown)), Some(Action::PageDown));
}

#[test]
#[should_panic(expected = "multi-key")]
fn chord_default_panics() {
    let _ = Keymap::builder().bind(Action::Quit, ["g-g"]);
}

#[test]
#[should_panic(expected = "no keys")]
fn bind_empty_keys_panics() {
    let _ = Keymap::builder().bind(Action::Quit, Vec::<&str>::new());
}

#[test]
fn shifted_punctuation_matches_windows_events() {
    let mut km = keymap();
    let warnings = km.merge(overrides(&[("quit", &["?"])]));
    assert!(warnings.is_empty());
    use crokey::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    // Windows 后端给 shift+标点附带 SHIFT;解析侧 "?" 不带,需回退匹配。
    let win_question = KeyEvent::new(KeyCode::Char('?'), KeyModifiers::SHIFT);
    assert_eq!(km.action_for(win_question), Some(Action::Quit));
    // 无 SHIFT 的常规形态照常命中。
    assert_eq!(km.action_for(key!('?')), Some(Action::Quit));
}

#[test]
fn shift_letters_are_normalized() {
    let mut km = keymap();
    let warnings = km.merge(overrides(&[("page_down", &["shift-g"])]));
    assert!(warnings.is_empty());
    // 终端里按 Shift+G 送来的是「大写 G + SHIFT」,须命中。
    use crokey::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    let event = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
    assert_eq!(km.action_for(event), Some(Action::PageDown));
}
