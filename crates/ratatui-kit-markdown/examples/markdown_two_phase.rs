//! Markdown 两阶段渲染示例。
//!
//! 演示 `Markdown` 组件的「两阶段渲染」：首帧跳过 syntect 语法高亮
//! （<1ms 立即显示纯文本），随后通过 `use_future` 自动触发第二帧补上完整高亮。
//!
//! 关键在于**全程无需任何按键或滚动**：ratatui-kit 的渲染循环是按需驱动的，
//! 若不主动唤醒，静态文档会永久停在首帧纯文本。本示例启动后不做任何交互，
//! 代码块依然会自动高亮 —— 这正是第二帧被主动触发的证据。
//!
//! 语法高亮需启用 `highlight` feature
//! （`cargo run --example markdown_two_phase --features markdown-highlight`）。

use ratatui_kit::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    ratatui::layout::{Constraint, Direction},
};
use ratatui_kit_markdown::Markdown;

const CONTENT: &str = r#"# 两阶段渲染演示

`Markdown` 组件采用**两阶段渲染**：首帧跳过语法高亮（<1ms 立即显示），
随后自动补上完整的 syntect 高亮 —— 全程无需任何按键或滚动。

下面的代码块在这个**静态文档**中也会自动高亮：

## Rust

```rust
fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

## Python

```python
def quicksort(xs):
    if len(xs) <= 1:
        return xs
    pivot = xs[len(xs) // 2]
    left = [x for x in xs if x < pivot]
    mid = [x for x in xs if x == pivot]
    right = [x for x in xs if x > pivot]
    return quicksort(left) + mid + quicksort(right)
```

## TypeScript

```typescript
const greet = (name: string): string => `Hello, ${name}!`;
console.log(greet("ratatui-kit"));
```
"#;

#[tokio::main]
async fn main() {
    element!(App)
        .fullscreen()
        .await
        .expect("Failed to run the application");
}

#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut exit = hooks.use_exit();

    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        if let Event::Key(key) = event
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('q')
        {
            exit();
            return EventResult::Consumed;
        }
        EventResult::Ignored
    });

    element!(
        ScrollView(
            flex_direction: Direction::Vertical,
            scrollbars: Scrollbars {
                vertical_scrollbar_visibility: ScrollbarVisibility::Automatic,
                ..Default::default()
            },
        ) {
            Markdown(content: CONTENT)
            View(height: Constraint::Length(1)) {
                Text(text: " q 退出 ".to_string())
            }
        }
    )
}
