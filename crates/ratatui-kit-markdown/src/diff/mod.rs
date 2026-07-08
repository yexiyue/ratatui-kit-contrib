mod compute;
mod render;

use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Paragraph, Widget, Wrap},
};

use crate::DiffTheme;
use crate::theme::{apply_color_override, resolve_style};

pub use compute::DiffTag;
use compute::compute_diff;
use render::render_diff;

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
    /// 新增行样式覆盖。`None` 用主题，`Some(style)` patch 主题，`Some(Style::reset())` 清空。
    pub add_style: Option<Style>,
    /// 删除行样式覆盖。
    pub remove_style: Option<Style>,
    /// 未修改行样式覆盖。
    pub unchanged_style: Option<Style>,
    /// 行号样式覆盖。
    pub line_number_style: Option<Style>,
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
            add_style: None,
            remove_style: None,
            unchanged_style: None,
            line_number_style: None,
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

/// Resolve the effective [`DiffTheme`] from the base (theme-derived) slots and
/// this call's props: new `Option<Style>` overrides patch first via
/// [`resolve_style`], then the legacy per-channel `Option<Color>` props win on
/// top via [`apply_color_override`] (kept for backward API compatibility).
fn resolve_theme(props: &DiffProps, base: DiffTheme) -> DiffTheme {
    DiffTheme {
        add_style: apply_color_override(
            resolve_style(base.add_style, props.add_style),
            props.add_fg,
            props.add_bg,
        ),
        remove_style: apply_color_override(
            resolve_style(base.remove_style, props.remove_style),
            props.remove_fg,
            props.remove_bg,
        ),
        unchanged_style: resolve_style(base.unchanged_style, props.unchanged_style),
        line_number_style: apply_color_override(
            resolve_style(base.line_number_style, props.line_number_style),
            props.line_num_color,
            None,
        ),
    }
}

#[component]
pub fn Diff(props: &DiffProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let show_line_numbers = props.show_line_numbers.unwrap_or(false);

    let base = hooks.use_component_theme::<DiffTheme>();
    let theme = resolve_theme(props, base);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_styles_from_theme() {
        let theme = DiffTheme {
            add_style: Style::new().fg(Color::Green),
            remove_style: Style::new().fg(Color::Red),
            unchanged_style: Style::new().fg(Color::White),
            line_number_style: Style::new().fg(Color::DarkGray),
        };

        let theme = resolve_theme(&DiffProps::default(), theme);

        assert_eq!(theme.add_style.fg, Some(Color::Green));
        assert_eq!(theme.remove_style.fg, Some(Color::Red));
        assert_eq!(theme.unchanged_style.fg, Some(Color::White));
        assert_eq!(theme.line_number_style.fg, Some(Color::DarkGray));
    }

    #[test]
    fn style_override_patches_and_reset_clears_theme() {
        let theme = DiffTheme {
            add_style: Style::new().fg(Color::Green).bg(Color::Black),
            remove_style: Style::new().fg(Color::Red),
            unchanged_style: Style::new().fg(Color::White),
            line_number_style: Style::new().fg(Color::DarkGray),
        };
        let props = DiffProps {
            add_style: Some(Style::new().fg(Color::LightGreen)),
            remove_style: Some(Style::reset()),
            ..DiffProps::default()
        };

        let theme = resolve_theme(&props, theme);

        assert_eq!(theme.add_style.fg, Some(Color::LightGreen));
        assert_eq!(theme.add_style.bg, Some(Color::Black));
        assert_eq!(theme.remove_style, Style::reset());
    }

    /// When both a legacy `Option<Color>` prop and its sibling new
    /// `Option<Style>` prop are set at once, the legacy color prop wins on the
    /// channel it controls, while the rest of the new style patch survives.
    #[test]
    fn legacy_color_props_win_over_sibling_style_props_on_the_same_channel() {
        let theme = DiffTheme {
            add_style: Style::new().fg(Color::Green).bg(Color::Black),
            remove_style: Style::new().fg(Color::Red).bg(Color::Black),
            unchanged_style: Style::new().fg(Color::White),
            line_number_style: Style::new().fg(Color::DarkGray),
        };
        let props = DiffProps {
            add_style: Some(Style::new().fg(Color::LightGreen).bg(Color::Blue)),
            add_fg: Some(Color::Magenta),
            remove_style: Some(Style::new().bg(Color::Yellow)),
            remove_bg: Some(Color::Cyan),
            line_number_style: Some(Style::new().fg(Color::Gray)),
            line_num_color: Some(Color::LightRed),
            ..DiffProps::default()
        };

        let theme = resolve_theme(&props, theme);

        assert_eq!(theme.add_style.fg, Some(Color::Magenta));
        assert_eq!(theme.add_style.bg, Some(Color::Blue));
        assert_eq!(theme.remove_style.fg, Some(Color::Red));
        assert_eq!(theme.remove_style.bg, Some(Color::Cyan));
        assert_eq!(theme.line_number_style.fg, Some(Color::LightRed));
    }
}
