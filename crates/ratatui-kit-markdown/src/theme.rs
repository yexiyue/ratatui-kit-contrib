use ratatui_kit::ratatui::style::{Color, Modifier, Style};
use ratatui_kit::{ComponentTheme, Palette};

/// Markdown document theme slots.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkdownTheme {
    /// Prefix markers for headings (`#`, `##`, ...).
    pub heading_marker_style: Style,
    /// Heading text style.
    pub heading_style: Style,
    /// Bullet / ordered-list marker style.
    pub list_marker_style: Style,
    /// Inline code style.
    pub inline_code_style: Style,
    /// Link text style.
    pub link_style: Style,
    /// Appended link destination style.
    pub link_url_style: Style,
    /// Table border/grid style.
    pub table_border_style: Style,
    /// Horizontal rule style.
    pub rule_style: Style,
}

impl ComponentTheme for MarkdownTheme {
    fn from_palette(palette: &Palette) -> Self {
        Self {
            heading_marker_style: Style::new().fg(palette.fg_dim),
            heading_style: Style::new()
                .fg(palette.warning)
                .add_modifier(Modifier::BOLD),
            list_marker_style: Style::new().fg(palette.fg_dim),
            inline_code_style: Style::new().fg(palette.info).bg(palette.surface),
            link_style: Style::new()
                .fg(palette.info)
                .add_modifier(Modifier::UNDERLINED),
            link_url_style: Style::new().fg(palette.fg_dim),
            table_border_style: Style::new().fg(palette.border),
            rule_style: Style::new().fg(palette.border),
        }
    }
}

impl Default for MarkdownTheme {
    fn default() -> Self {
        Self::from_palette(&Palette::default())
    }
}

/// Standalone code block theme slots.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CodeBlockTheme {
    /// Line number style.
    pub line_number_style: Style,
    /// Fallback code text style when syntax highlighting is unavailable.
    pub code_style: Style,
    /// Outer border style.
    pub border_style: Style,
    /// Language label style.
    pub language_label_style: Style,
}

impl ComponentTheme for CodeBlockTheme {
    fn from_palette(palette: &Palette) -> Self {
        Self {
            line_number_style: Style::new().fg(palette.fg_dim),
            code_style: Style::new().fg(palette.fg),
            border_style: Style::new().fg(palette.border),
            language_label_style: Style::new().fg(palette.info),
        }
    }
}

impl Default for CodeBlockTheme {
    fn default() -> Self {
        Self::from_palette(&Palette::default())
    }
}

/// Blockquote theme slots.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockquoteTheme {
    /// Solid left bar style.
    pub bar_style: Style,
    /// Content area style.
    pub style: Style,
}

impl ComponentTheme for BlockquoteTheme {
    fn from_palette(palette: &Palette) -> Self {
        Self {
            bar_style: Style::new().bg(palette.border_active),
            style: Style::new().fg(palette.fg).bg(palette.surface),
        }
    }
}

impl Default for BlockquoteTheme {
    fn default() -> Self {
        Self::from_palette(&Palette::default())
    }
}

/// Divider theme slots.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DividerTheme {
    /// Rule line style.
    pub style: Style,
}

impl ComponentTheme for DividerTheme {
    fn from_palette(palette: &Palette) -> Self {
        Self {
            style: Style::new().fg(palette.border),
        }
    }
}

impl Default for DividerTheme {
    fn default() -> Self {
        Self::from_palette(&Palette::default())
    }
}

/// Diff renderer theme slots.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffTheme {
    /// Added line / word style.
    pub add_style: Style,
    /// Removed line / word style.
    pub remove_style: Style,
    /// Unchanged content style.
    pub unchanged_style: Style,
    /// Line number gutter style.
    pub line_number_style: Style,
}

impl ComponentTheme for DiffTheme {
    fn from_palette(palette: &Palette) -> Self {
        Self {
            add_style: Style::new().fg(palette.success).bg(palette.surface),
            remove_style: Style::new().fg(palette.error).bg(palette.surface),
            unchanged_style: Style::new().fg(palette.fg),
            line_number_style: Style::new().fg(palette.fg_dim),
        }
    }
}

impl Default for DiffTheme {
    fn default() -> Self {
        Self::from_palette(&Palette::default())
    }
}

pub(crate) fn resolve_style(theme: Style, override_style: Option<Style>) -> Style {
    theme.patch(override_style.unwrap_or_default())
}

/// Apply legacy per-channel `Option<Color>` overrides on top of an
/// already-resolved [`Style`] (theme patched with any new `Option<Style>`
/// override). Used to keep the old `prefix_color`/`add_fg`-style props
/// working while `resolve_style` handles the newer `Option<Style>` props.
pub(crate) fn apply_color_override(style: Style, fg: Option<Color>, bg: Option<Color>) -> Style {
    let style = fg.map_or(style, |color| style.fg(color));
    bg.map_or(style, |color| style.bg(color))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui_kit::ratatui::style::Color;

    #[test]
    fn themes_derive_from_palette() {
        let mut palette = Palette::default();
        palette.fg = Color::White;
        palette.fg_dim = Color::DarkGray;
        palette.surface = Color::Rgb(10, 11, 12);
        palette.border = Color::Blue;
        palette.border_active = Color::Cyan;
        palette.info = Color::Magenta;
        palette.success = Color::Green;
        palette.error = Color::Red;
        palette.warning = Color::Yellow;

        let markdown = MarkdownTheme::from_palette(&palette);
        let code = CodeBlockTheme::from_palette(&palette);
        let quote = BlockquoteTheme::from_palette(&palette);
        let divider = DividerTheme::from_palette(&palette);
        let diff = DiffTheme::from_palette(&palette);

        assert_eq!(markdown.inline_code_style.fg, Some(Color::Magenta));
        assert_eq!(markdown.inline_code_style.bg, Some(Color::Rgb(10, 11, 12)));
        assert_eq!(markdown.table_border_style.fg, Some(Color::Blue));
        assert_eq!(code.border_style.fg, Some(Color::Blue));
        assert_eq!(quote.bar_style.bg, Some(Color::Cyan));
        assert_eq!(divider.style.fg, Some(Color::Blue));
        assert_eq!(diff.add_style.fg, Some(Color::Green));
        assert_eq!(diff.remove_style.fg, Some(Color::Red));
    }

    #[test]
    fn style_overrides_patch_or_reset_theme() {
        let theme = Style::new().fg(Color::Blue).bg(Color::Black);

        assert_eq!(resolve_style(theme, None), theme);
        assert_eq!(
            resolve_style(theme, Some(Style::new().fg(Color::Red))),
            Style::new().fg(Color::Red).bg(Color::Black)
        );
        assert_eq!(resolve_style(theme, Some(Style::reset())), Style::reset());
    }
}
