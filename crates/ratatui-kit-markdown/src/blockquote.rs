use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::{
    layout::Rect,
    style::{Color, Style},
};

use crate::theme::{BlockquoteTheme, apply_color_override, resolve_style};

/// 引用块容器组件。
///
/// 在内容左侧渲染实心竖线（空格反色），支持嵌套深度。
/// 类似 HTML 的 `<blockquote>` 或 Markdown 的引用块。
///
/// ## 用法示例
/// ```rust,no_run
/// use ratatui_kit::prelude::*;
/// use ratatui_kit_markdown::Blockquote;
///
/// let _quote = element!(Blockquote(depth: Some(1)) {
///     Text(text: "这是一段被引用的内容")
/// });
/// ```
#[with_layout_style]
#[derive(Props)]
pub struct BlockquoteProps<'a> {
    /// 引用深度（嵌套层级）。默认 1。
    pub depth: Option<u32>,
    /// 竖线颜色。兼容旧 API；设置后会覆盖主题里的竖线背景色。
    pub prefix_color: Option<Color>,
    /// 背景颜色。兼容旧 API；设置后会覆盖主题里的内容背景色。
    pub bg_color: Option<Color>,
    /// 竖线样式覆盖。`None` 用主题，`Some(style)` patch 主题，`Some(Style::reset())` 清空。
    pub bar_style: Option<Style>,
    /// 内容区样式覆盖。`None` 用主题，`Some(style)` patch 主题，`Some(Style::reset())` 清空。
    pub style: Option<Style>,
    /// 子元素列表。
    pub children: Vec<AnyElement<'a>>,
}

impl Default for BlockquoteProps<'_> {
    fn default() -> Self {
        Self {
            depth: Some(1),
            prefix_color: None,
            bg_color: None,
            bar_style: None,
            style: None,
            children: Vec::new(),
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

/// 引用块组件。
pub struct Blockquote {
    depth: u32,
    /// 竖线样式：空格 + 背景色 = 实心竖线
    bar_style: Style,
    /// 内容区背景样式：极淡的背景色
    bg_style: Style,
}

impl Blockquote {
    fn from_props(props: &BlockquoteProps<'_>, theme: BlockquoteTheme) -> Self {
        let depth = props.depth.unwrap_or(1).max(1);
        let bar_style = apply_color_override(
            resolve_style(theme.bar_style, props.bar_style),
            None,
            props.prefix_color,
        );
        let bg_style = apply_color_override(
            resolve_style(theme.style, props.style),
            None,
            props.bg_color,
        );
        Self {
            depth,
            bar_style,
            bg_style,
        }
    }

    /// 每层 1 列竖线 + 末尾 1 列间距。
    fn prefix_width(&self) -> u16 {
        self.depth as u16 + 1
    }
}

impl Component for Blockquote {
    type Props<'a> = BlockquoteProps<'a>;

    fn new(props: &Self::Props<'_>) -> Self {
        Self::from_props(props, BlockquoteTheme::default())
    }

    fn update(
        &mut self,
        props: &mut Self::Props<'_>,
        _hooks: Hooks,
        updater: &mut ComponentUpdater,
    ) {
        let theme = updater.use_component_theme::<BlockquoteTheme>();
        *self = Self::from_props(props, theme);
        updater.set_layout_style(props.layout_style());
        updater.update_children(&mut props.children, None);
    }

    fn draw(&mut self, drawer: &mut ComponentDrawer<'_, '_>) {
        let prefix_w = self.prefix_width();
        let area = drawer.area;

        {
            let buf = drawer.buffer_mut();
            for row in area.top()..area.bottom() {
                // 背景色 + 竖线一次遍历完成
                let mut c = area.left();
                // 先填背景色
                for col in c..area.right() {
                    buf[(col, row)].set_style(self.bg_style);
                }
                // 再画竖线
                for _d in 0..self.depth {
                    if c < area.right() {
                        buf.set_string(c, row, " ", self.bar_style);
                    }
                    c += 1;
                }
            }
        }

        drawer.area = Rect {
            x: area.x + prefix_w,
            y: area.y,
            width: area.width.saturating_sub(prefix_w),
            height: area.height,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_kit::ratatui::style::Color;

    #[test]
    fn resolves_theme_and_legacy_color_overrides() {
        let theme = BlockquoteTheme {
            bar_style: Style::new().bg(Color::Blue),
            style: Style::new().fg(Color::White).bg(Color::Black),
        };
        let props = BlockquoteProps {
            prefix_color: Some(Color::Red),
            bg_color: Some(Color::Green),
            ..BlockquoteProps::default()
        };

        let quote = Blockquote::from_props(&props, theme);

        assert_eq!(quote.bar_style.bg, Some(Color::Red));
        assert_eq!(quote.bg_style.fg, Some(Color::White));
        assert_eq!(quote.bg_style.bg, Some(Color::Green));
    }

    #[test]
    fn style_reset_clears_blockquote_theme() {
        let theme = BlockquoteTheme {
            bar_style: Style::new().bg(Color::Blue),
            style: Style::new().fg(Color::White).bg(Color::Black),
        };
        let props = BlockquoteProps {
            style: Some(Style::reset()),
            ..BlockquoteProps::default()
        };

        let quote = Blockquote::from_props(&props, theme);

        assert_eq!(quote.bg_style, Style::reset());
    }

    /// When both the legacy `Option<Color>` prop and its sibling new
    /// `Option<Style>` prop are set at once, the legacy color prop wins on the
    /// channel it controls, while the rest of the new style patch survives.
    #[test]
    fn legacy_color_prop_wins_over_sibling_style_prop_on_the_same_channel() {
        let theme = BlockquoteTheme {
            bar_style: Style::new().bg(Color::Blue),
            style: Style::new().fg(Color::White).bg(Color::Black),
        };
        let props = BlockquoteProps {
            bar_style: Some(Style::new().bg(Color::Cyan)),
            prefix_color: Some(Color::Red),
            style: Some(Style::new().fg(Color::Yellow).bg(Color::Magenta)),
            bg_color: Some(Color::Green),
            ..BlockquoteProps::default()
        };

        let quote = Blockquote::from_props(&props, theme);

        assert_eq!(quote.bar_style.bg, Some(Color::Red));
        assert_eq!(quote.bg_style.fg, Some(Color::Yellow));
        assert_eq!(quote.bg_style.bg, Some(Color::Green));
    }
}
