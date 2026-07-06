mod parser;

use std::sync::Arc;

use pulldown_cmark::{Alignment, HeadingLevel};
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::{
    layout::{Constraint, Direction},
    style::{Color, Style},
    text::{Line, Span},
};
use unicode_width::UnicodeWidthStr;

use crate::{CodeBlock, Divider};

// Re-export the parser types: `render_blocks` (public) takes `&[ParsedBlock]`, so
// these must be part of the public surface too.
pub use parser::{ListItemData, ParseResult, ParsedBlock, parse_markdown};

#[with_layout_style]
#[derive(Props, Default)]
pub struct MarkdownProps {
    pub content: String,
    pub children: Vec<AnyElement<'static>>,
}

/// 渲染结果：元素列表 + 总高度
pub struct RenderedMarkdown {
    pub elements: Vec<AnyElement<'static>>,
    /// 内容总行数（含所有间距），用于 ScrollView 精确定位。
    ///
    /// 注意：代码块 / 表格的预留高度按「未换行的逻辑行数」计算。终端宽度在计算
    /// 高度时未知（布局阶段才确定），因此超宽行的换行高度无法在此精确预留。
    pub total_height: u16,
}

/// Markdown 文档组件。解析并渲染标题、行内样式、列表、表格、代码块、引用块与分割线。
///
/// ```no_run
/// use ratatui_kit::prelude::*;
/// use ratatui_kit_markdown::Markdown;
///
/// let _md = element!(Markdown(content: "# Title\n\nsome **bold** text".to_string()));
/// ```
#[component]
pub fn Markdown(mut hooks: Hooks, props: &MarkdownProps) -> impl Into<AnyElement<'static>> {
    // 用 use_memo 缓存解析结果，只有 content 变化时才重新解析。
    // render_blocks 每帧调用（开销很小，只遍历 blocks + clone Span）。
    let parsed = hooks.use_memo(|| parse_markdown(&props.content), props.content.clone());
    let rendered = render_blocks(&parsed.blocks);
    element! {
        View(
            flex_direction: Direction::Vertical,
            height: Constraint::Length(rendered.total_height),
        ) {
            { rendered.elements.into_iter() }
        }
    }
}

// ── 行级布局中间表示 ─────────────────────────────────────────────

/// 一个渲染行（或占据自身高度的复合块）。把块级 IR 摊平成「行」使得渲染高度、
/// 段落间距、列表前缀都能被单元测试直接断言。
pub(crate) enum RenderRow {
    /// 单行文本（空行 = 空 `Line`）。高度 1。
    Line(Line<'static>),
    /// 分割线，渲染为一条 `Divider`。高度 1。
    Rule,
    /// 代码块。保留高度 = 逻辑行数。
    Code {
        lang: Option<String>,
        lines: Vec<String>,
    },
    /// 表格。高度已按行数算好。
    Table {
        columns: Vec<TableColumn>,
        rows: Vec<Vec<Vec<Span<'static>>>>,
        height: u16,
    },
}

impl RenderRow {
    fn height(&self) -> u16 {
        match self {
            RenderRow::Line(_) | RenderRow::Rule => 1,
            RenderRow::Code { lines, .. } => lines.len() as u16,
            RenderRow::Table { height, .. } => *height,
        }
    }
}

fn heading_level_num(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn heading_line(level_num: usize, line: &Line<'static>) -> Line<'static> {
    let prefix = "#".repeat(level_num);
    let mut spans = vec![
        Span::styled(prefix, Style::new().fg(Color::DarkGray)),
        Span::raw(" "),
    ];
    spans.extend(line.spans.clone());
    Line::from(spans)
}

fn list_item_line(item: &ListItemData) -> Line<'static> {
    let indent = "  ".repeat(item.depth as usize);
    let prefix = if item.ordered {
        format!("{}{}. ", indent, item.number.unwrap_or(1))
    } else {
        format!("{indent}• ")
    };
    let mut spans = vec![Span::styled(prefix, Style::new().fg(Color::DarkGray))];
    spans.extend(item.spans.clone());
    Line::from(spans)
}

/// 计算 span 列表的显示宽度。
fn span_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|s| s.content.width()).sum()
}

/// 把表格块转成一个 `RenderRow::Table`；列数为 0 时返回 `None`（renderer 用空行占位）。
fn table_row(
    headers: &[Vec<Span<'static>>],
    rows: &[Vec<Vec<Span<'static>>>],
    alignments: &[Alignment],
) -> Option<RenderRow> {
    let col_count = headers
        .len()
        .max(rows.first().map(|r| r.len()).unwrap_or(0));
    if col_count == 0 {
        return None;
    }

    let mut col_widths = vec![0usize; col_count];
    for (i, cell) in headers.iter().enumerate() {
        col_widths[i] = col_widths[i].max(span_width(cell));
    }
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < col_count {
                col_widths[i] = col_widths[i].max(span_width(cell));
            }
        }
    }
    for w in &mut col_widths {
        *w = (*w).max(3);
    }

    let columns: Vec<TableColumn> = (0..col_count)
        .map(|i| {
            let header = headers
                .get(i)
                .map(|spans| Line::from(spans.clone()))
                .unwrap_or_default();
            let alignment = match alignments.get(i) {
                Some(Alignment::Center) => TableCellAlignment::Center,
                Some(Alignment::Right) => TableCellAlignment::Right,
                _ => TableCellAlignment::Left,
            };
            let width = col_widths[i] as u16;
            TableColumn::new(header, Constraint::Length(width)).alignment(alignment)
        })
        .collect();

    // 表格高度: header(1) + rows + header_sep(1) + row_seps + grid_borders(2)
    let n = rows.len() as u16;
    let table_height = 1 + n + 1 + n.saturating_sub(1) + 2;

    Some(RenderRow::Table {
        columns,
        rows: rows.to_vec(),
        height: table_height,
    })
}

/// 把解析块摊平成渲染行列表（含所有空行间距）。
pub(crate) fn render_rows(blocks: &[ParsedBlock]) -> Vec<RenderRow> {
    let mut rows: Vec<RenderRow> = Vec::new();
    let mut prev_added_trailing = false;
    let mut prev_was_major = false;
    let mut prev_was_real_para = false;

    for block in blocks {
        let is_major = matches!(
            block,
            ParsedBlock::Heading(..)
                | ParsedBlock::CodeBlock(..)
                | ParsedBlock::Table(..)
                | ParsedBlock::Rule
        );

        // 相邻 major 块之间补空行（上一个 major 没有自带 trailing 空行时）。
        if !prev_added_trailing && prev_was_major && is_major {
            rows.push(RenderRow::Line(Line::default()));
        }
        prev_added_trailing = false;

        // 相邻的两个真实段落之间插入空行（bug #1：段落不再拼进同一行）。
        let is_real_para = matches!(block, ParsedBlock::Paragraph(lines) if !lines.is_empty());
        if is_real_para && prev_was_real_para {
            rows.push(RenderRow::Line(Line::default()));
        }

        match block {
            ParsedBlock::Heading(level, line) => {
                rows.push(RenderRow::Line(heading_line(
                    heading_level_num(*level),
                    line,
                )));
                rows.push(RenderRow::Line(Line::default()));
                prev_added_trailing = true;
            }
            ParsedBlock::Paragraph(lines) => {
                if lines.is_empty() {
                    // 空段落是显式的空行占位（如列表首尾）。
                    rows.push(RenderRow::Line(Line::default()));
                } else {
                    // 每一行独立成行（硬换行也是真实换行）。
                    for line in lines {
                        rows.push(RenderRow::Line(line.clone()));
                    }
                }
            }
            ParsedBlock::CodeBlock(lang, code_lines) => {
                rows.push(RenderRow::Line(Line::default()));
                let lang_opt = if lang.is_empty() {
                    None
                } else {
                    Some(lang.clone())
                };
                rows.push(RenderRow::Code {
                    lang: lang_opt,
                    lines: code_lines.clone(),
                });
                rows.push(RenderRow::Line(Line::default()));
                prev_added_trailing = true;
            }
            ParsedBlock::ListItem(item) => {
                rows.push(RenderRow::Line(list_item_line(item)));
            }
            ParsedBlock::Table(headers, table_rows, alignments) => {
                match table_row(headers, table_rows, alignments) {
                    Some(row) => rows.push(row),
                    None => rows.push(RenderRow::Line(Line::default())),
                }
            }
            ParsedBlock::Rule => {
                rows.push(RenderRow::Rule);
            }
        }

        prev_was_major = is_major;
        prev_was_real_para = is_real_para;
    }

    rows
}

/// 把一个渲染行构造成 `AnyElement`。
fn build_row(row: RenderRow) -> AnyElement<'static> {
    match row {
        RenderRow::Line(line) => element! {
            View(height: Constraint::Length(1)) {
                Text(text: line)
            }
        }
        .into_any(),
        RenderRow::Rule => element! {
            View(height: Constraint::Length(1)) {
                Divider(char: '─', style_cfg: Style::new().fg(Color::DarkGray))
            }
        }
        .into_any(),
        RenderRow::Code { lang, lines } => {
            let line_count = lines.len() as u16;
            element! {
                CodeBlock(
                    lines: lines,
                    lang: lang,
                    show_border: false,
                    show_line_numbers: false,
                    height: Constraint::Length(line_count),
                )
            }
            .into_any()
        }
        RenderRow::Table {
            columns,
            rows,
            height,
        } => {
            type RowType = Vec<Vec<Span<'static>>>;
            let render_row: RenderTableRow<RowType> = Arc::new(|row, _selected| {
                row.iter()
                    .map(|cell| TableCell::new(Line::from(cell.clone())))
                    .collect()
            });
            element! {
                Table<RowType>(
                    columns: columns,
                    rows: rows,
                    render_row: Some(render_row),
                    active: false,
                    border_mode: TableBorderMode::Grid,
                    border_style: Style::new().fg(Color::DarkGray),
                    row_separator: true,
                    height: Constraint::Length(height),
                )
            }
            .into_any()
        }
    }
}

/// 将解析块渲染为 `RenderedMarkdown`（元素列表 + 总高度）。
///
/// `total_height` 即可作为 ScrollView 的内容高度，与渲染输出精确一致
/// （代码块 / 表格按未换行的逻辑行数计，见 [`RenderedMarkdown::total_height`]）。
pub fn render_blocks(blocks: &[ParsedBlock]) -> RenderedMarkdown {
    let rows = render_rows(blocks);
    let mut total_height: u16 = 0;
    let mut elements = Vec::with_capacity(rows.len());
    for row in rows {
        total_height = total_height.saturating_add(row.height());
        elements.push(build_row(row));
    }
    RenderedMarkdown {
        elements,
        total_height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 把渲染行摊平成可断言的文本行（复合块用占位符表示）。
    fn row_texts(blocks: &[ParsedBlock]) -> Vec<String> {
        render_rows(blocks)
            .iter()
            .map(|row| match row {
                RenderRow::Line(line) => line.spans.iter().map(|s| s.content.as_ref()).collect(),
                RenderRow::Rule => "<rule>".to_string(),
                RenderRow::Code { .. } => "<code>".to_string(),
                RenderRow::Table { .. } => "<table>".to_string(),
            })
            .collect()
    }

    /// 回归 bug #1：`a\n\nb` 产出三行 `a` / 空 / `b`（相邻段落不再被拼成一行）。
    #[test]
    fn adjacent_paragraphs_render_as_three_rows() {
        let parsed = parse_markdown("a\n\nb");
        assert_eq!(
            row_texts(&parsed.blocks),
            vec!["a".to_string(), String::new(), "b".to_string()],
        );
        assert_eq!(render_blocks(&parsed.blocks).total_height, 3);
    }

    /// 回归 bug #2：`- a\n  - b\n- c` 产出 `• a` / `  • b` / `• c`
    /// （父项保留 bullet、无裸文本、无多余空 bullet）。
    #[test]
    fn nested_list_keeps_parent_bullet_without_empty_bullet() {
        let parsed = parse_markdown("- a\n  - b\n- c");
        let bullets: Vec<String> = row_texts(&parsed.blocks)
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect();
        assert_eq!(
            bullets,
            vec!["• a".to_string(), "  • b".to_string(), "• c".to_string()],
        );
    }

    /// 顺带覆盖有序嵌套列表的编号：父项 `1.`，子项 `1.`，同级下一项 `2.`。
    #[test]
    fn nested_ordered_list_numbers_are_correct() {
        let parsed = parse_markdown("1. a\n   1. b\n2. c");
        let bullets: Vec<String> = row_texts(&parsed.blocks)
            .into_iter()
            .filter(|s| !s.trim().is_empty())
            .collect();
        assert_eq!(
            bullets,
            vec!["1. a".to_string(), "  1. b".to_string(), "2. c".to_string()],
        );
    }
}
