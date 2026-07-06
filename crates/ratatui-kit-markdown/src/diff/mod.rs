mod compute;
mod render;

use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{Paragraph, Widget, Wrap},
};

pub use compute::DiffTag;
use compute::compute_diff;
use render::{DiffTheme, render_diff};

/// Diff 对比组件，用于展示两个文本版本的差异。
///
/// ## 用法示例
/// ```rust,no_run
/// use ratatui_kit::prelude::*;
/// use ratatui_kit_markdown::Diff;
///
/// let _diff = element!(Diff(
///     old: "line1\nline2\n".to_string(),
///     new: "line1\nmodified\n".to_string(),
///     show_line_numbers: Some(true),
/// ));
/// ```
#[with_layout_style]
#[derive(Props)]
pub struct DiffProps {
    /// 旧版本文本内容
    pub old: String,
    /// 新版本文本内容
    pub new: String,
    /// 是否显示行号。默认 false。
    pub show_line_numbers: Option<bool>,
    /// 新增行文字颜色
    pub add_fg: Option<Color>,
    /// 新增行背景色
    pub add_bg: Option<Color>,
    /// 删除行文字颜色
    pub remove_fg: Option<Color>,
    /// 删除行背景色
    pub remove_bg: Option<Color>,
    /// 行号颜色
    pub line_num_color: Option<Color>,
}

impl Default for DiffProps {
    fn default() -> Self {
        Self {
            old: String::new(),
            new: String::new(),
            show_line_numbers: Some(false),
            add_fg: None,
            add_bg: None,
            remove_fg: None,
            remove_bg: None,
            line_num_color: None,
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

/// Diff 渲染 adapter。
#[derive(Clone)]
struct DiffParagraph {
    paragraph: Paragraph<'static>,
}

impl Widget for DiffParagraph {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.render(area, buf);
    }
}

impl Widget for &DiffParagraph {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.paragraph.clone().render(area, buf);
    }
}

#[component]
pub fn Diff(props: &DiffProps) -> impl Into<AnyElement<'static>> {
    let show_line_numbers = props.show_line_numbers.unwrap_or(false);

    let default = DiffTheme::default();
    let theme = DiffTheme {
        add_fg: props.add_fg.unwrap_or(default.add_fg),
        add_bg: props.add_bg.unwrap_or(default.add_bg),
        remove_fg: props.remove_fg.unwrap_or(default.remove_fg),
        remove_bg: props.remove_bg.unwrap_or(default.remove_bg),
        line_num: props.line_num_color.unwrap_or(default.line_num),
        unchanged: default.unchanged,
    };

    let diff_lines = compute_diff(&props.old, &props.new);
    let text = render_diff(&diff_lines, show_line_numbers, &theme);

    let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });

    // `#[component]` 是透明布局包装器：布局属性要落到返回的根元素上，故转发到 `View`。
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
            widget(DiffParagraph { paragraph })
        }
    }
}
