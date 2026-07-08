use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Paragraph, Widget},
};

use crate::theme::{DividerTheme, resolve_style};

/// 水平分割线（分隔符）组件。
///
/// 渲染为一行由指定字符重复构成的水平线，类似 HTML 的 `<hr>`。
///
/// ## 用法示例
/// ```rust,no_run
/// use ratatui_kit::prelude::*;
/// use ratatui_kit::ratatui::style::{Color, Style};
/// use ratatui_kit_markdown::Divider;
///
/// let _hr = element!(Divider(char: Some('─'), style_cfg: Some(Style::new().fg(Color::Cyan))));
/// ```
#[with_layout_style]
#[derive(Props)]
pub struct DividerProps {
    /// 分割线使用的字符。默认 `'─'`。
    pub char: Option<char>,
    /// 分割线样式。`None` 用主题，`Some(style)` patch 主题，`Some(Style::reset())` 清空。
    pub style_cfg: Option<Style>,
}

impl Default for DividerProps {
    fn default() -> Self {
        Self {
            char: Some('─'),
            style_cfg: None,
            margin: Default::default(),
            offset: Default::default(),
            width: Default::default(),
            height: Default::default(),
            gap: Default::default(),
            flex_direction: Default::default(),
            justify_content: Default::default(),
        }
    }
}

/// 分割线渲染 adapter。
///
/// 使用足够长的重复字符行填充，Widget 渲染时由 ratatui 在分配区域内自动截断。
#[derive(Clone)]
struct DividerLine {
    paragraph: Paragraph<'static>,
}

impl Widget for DividerLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.render(area, buf);
    }
}

impl Widget for &DividerLine {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.clone().render(area, buf);
    }
}

#[component]
pub fn Divider(props: &DividerProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_component_theme::<DividerTheme>();
    let ch = props.char.unwrap_or('─');
    let style = resolve_style(theme.style, props.style_cfg);

    let line = ch.to_string().repeat(256);
    let paragraph = Paragraph::new(line).style(style);

    // `#[component]` 是透明布局包装器：布局属性必须落到返回的根元素上，否则
    // `Divider(width: ...)`（等）会被忽略。因此把 props 的布局字段转发到 `View`。
    element! {
        View(
            margin: props.margin,
            offset: props.offset,
            width: props.width,
            height: props.height,
            gap: props.gap,
            flex_direction: props.flex_direction,
            justify_content: props.justify_content,
        ) {
            widget(DividerLine { paragraph })
        }
    }
}
