//! Runnable demo: semantic actions + default keys + user TOML overrides.
//!
//! The embedded "user config" rebinds `down`, contains one invalid key string
//! and one unknown action — watch the warnings section: the app keeps working
//! on defaults for the broken entries, and the help line always shows the
//! *effective* bindings (`down` shows `Ctrl-n/n`, not `j/Down`).
//!
//! Run with: `cargo run -p ratatui-kit-keymap --example keymap_demo`

use std::sync::Arc;

use ratatui_kit::{EventPriority, EventResult, EventScope, prelude::*};
use ratatui_kit_keymap::{Keymap, KeymapWarning, UseKeymapHandler};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DemoAction {
    Up,
    Down,
    Top,
    Bottom,
    Quit,
}

const ITEM_COUNT: usize = 8;

/// Pretend this came from `~/.config/<app>/keybindings.toml`.
const USER_TOML: &str = r#"
down = ["ctrl-n", "n"]   # rebind: replaces j/Down entirely
top = "ctrl-"            # invalid key string -> warning, falls back to defaults
typo_action = "x"        # unknown action -> warning, ignored
"#;

/// 启动期一次性构建的所有不变量:keymap 本体 + 预渲染好的帮助/告警文本
/// (它们在 merge 后不再变化,没必要每帧重新拼字符串)。
struct DemoSetup {
    keymap: Arc<Keymap<DemoAction>>,
    help: String,
    warnings_text: String,
}

fn build_setup() -> DemoSetup {
    let mut keymap = Keymap::builder()
        .bind(DemoAction::Up, ["k", "up"])
        .desc(DemoAction::Up, "move up")
        .bind(DemoAction::Down, ["j", "down"])
        .desc(DemoAction::Down, "move down")
        .bind(DemoAction::Top, ["home", "g"])
        .desc(DemoAction::Top, "jump to top")
        .bind(DemoAction::Bottom, ["end", "shift-g"])
        .desc(DemoAction::Bottom, "jump to bottom")
        .bind(DemoAction::Quit, ["q", "esc"])
        .desc(DemoAction::Quit, "quit")
        .build();
    let warnings: Vec<KeymapWarning> = keymap
        .merge_toml_str(USER_TOML)
        .expect("demo TOML is well-formed");

    // 帮助行从 keymap 动态取 —— 覆盖后显示的就是新键。
    let help = keymap
        .entries()
        .map(|e| {
            let keys = e
                .keys
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("/");
            format!("{} [{keys}]", e.desc.unwrap_or(e.name))
        })
        .collect::<Vec<_>>()
        .join("  ");

    let warnings_text = if warnings.is_empty() {
        "no warnings".to_string()
    } else {
        warnings
            .iter()
            .map(|w| format!("⚠ {w}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    DemoSetup {
        // 挂进 Arc:hook 每帧重注册,Arc 克隆只是引用计数,不做深拷贝。
        keymap: Arc::new(keymap),
        help,
        warnings_text,
    }
}

#[tokio::main]
async fn main() {
    element!(Demo)
        .fullscreen()
        .await
        .expect("Failed to run the application");
}

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let setup = hooks.use_state(build_setup);
    let mut selected = hooks.use_state(|| 0usize);
    let mut exit = hooks.use_exit();

    // 先收集释放读锁,再交给 hook/渲染(避免读 guard 跨回调存活)。
    let (keymap, help, warnings_text) = {
        let guard = setup.read();
        (
            guard.keymap.clone(),
            guard.help.clone(),
            guard.warnings_text.clone(),
        )
    };

    hooks.use_keymap_handler(
        EventScope::Current,
        EventPriority::Normal,
        keymap,
        move |action, _key| {
            match action {
                DemoAction::Up => selected.set(selected.get().saturating_sub(1)),
                DemoAction::Down => selected.set((selected.get() + 1).min(ITEM_COUNT - 1)),
                DemoAction::Top => selected.set(0),
                DemoAction::Bottom => selected.set(ITEM_COUNT - 1),
                DemoAction::Quit => exit(),
            }
            EventResult::Consumed
        },
    );

    let list = (0..ITEM_COUNT)
        .map(|i| {
            let marker = if selected.get() == i { "▶" } else { " " };
            format!("{marker} item {i}")
        })
        .collect::<Vec<_>>()
        .join("\n");

    element!(Border() {
        Text(text: list)
        Text(text: help)
        Text(text: warnings_text)
    })
}
