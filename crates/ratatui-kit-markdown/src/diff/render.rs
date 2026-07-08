use ratatui_kit::ratatui::{
    style::{Color, Style},
    text::{Line, Span, Text},
};

use crate::DiffTheme;

use super::compute::{DiffLine, DiffTag};

/// 将 diff 行列表渲染为 ratatui Text。
pub fn render_diff(
    lines: &[DiffLine],
    show_line_numbers: bool,
    theme: &DiffTheme,
) -> Text<'static> {
    lines
        .iter()
        .map(|line| {
            let mut spans = Vec::new();

            let (prefix, line_style) = match line.tag {
                DiffTag::Insert => ("+", theme.add_style),
                DiffTag::Delete => ("-", theme.remove_style),
                DiffTag::Unchanged => (" ", theme.unchanged_style),
            };
            let gutter_style = with_bg(theme.line_number_style, line_style.bg);
            let unchanged_in_line_style = with_bg(theme.unchanged_style, line_style.bg);

            if show_line_numbers {
                let old_str = line
                    .old_line_num
                    .map(|n| format!("{n:>3} "))
                    .unwrap_or_else(|| "    ".to_string());
                let new_str = line
                    .new_line_num
                    .map(|n| format!("{n:>3} "))
                    .unwrap_or_else(|| "    ".to_string());
                spans.push(Span::styled(
                    format!("{prefix}{old_str}{new_str}"),
                    gutter_style,
                ));
            } else {
                spans.push(Span::styled(format!("{prefix} "), line_style));
            }

            let content = line.content.trim_end_matches('\n');
            if let Some(ref word_diffs) = line.word_diffs {
                // 单词级 diff：Delete 行只显示 Unchanged + Delete，Insert 行只显示 Unchanged + Insert
                let skip_tag = match line.tag {
                    DiffTag::Delete => Some(DiffTag::Insert),
                    DiffTag::Insert => Some(DiffTag::Delete),
                    DiffTag::Unchanged => None,
                };
                for wd in word_diffs {
                    if Some(wd.tag) == skip_tag {
                        continue;
                    }
                    let word_style = match wd.tag {
                        DiffTag::Insert => theme.add_style,
                        DiffTag::Delete => theme.remove_style,
                        DiffTag::Unchanged => unchanged_in_line_style,
                    };
                    spans.push(Span::styled(wd.text.clone(), word_style));
                }
            } else {
                spans.push(Span::styled(content.to_string(), line_style));
            }

            Line::from(spans)
        })
        .collect::<Vec<_>>()
        .into()
}

/// Fill in the row's highlight background for the gutter/unchanged-in-line
/// styles, but only when that style doesn't already carry its own `bg` --
/// otherwise an explicit `line_number_style`/`unchanged_style` override (which
/// already patched its `bg` via `resolve_style`) would get silently clobbered
/// back to the row's default bg.
fn with_bg(mut style: Style, bg: Option<Color>) -> Style {
    if style.bg.is_none() {
        style.bg = bg;
    }
    style
}
