use std::sync::Arc;

use ratatui_kit::{
    crossterm::event::{Event, KeyCode, KeyEventKind},
    prelude::*,
    ratatui::{
        layout::{Constraint, Direction, Flex},
        style::{Color, Style, Stylize},
        text::Line,
    },
};
use ratatui_kit_markdown::{Blockquote, CodeBlock, Diff, Divider, Markdown};
use ratatui_kit_themes::{IntoKitPalette, ThemeName, terminal_background};

#[derive(Clone)]
struct ComponentRow {
    component: &'static str,
    state: &'static str,
    note: &'static str,
}

const COMPONENT_ROWS: [ComponentRow; 4] = [
    ComponentRow {
        component: "Table",
        state: "selected",
        note: "selection + on_accent",
    },
    ComponentRow {
        component: "SearchInput",
        state: "success",
        note: "semantic border",
    },
    ComponentRow {
        component: "CodeBlock",
        state: "fallback",
        note: "plain code style",
    },
    ComponentRow {
        component: "Diff",
        state: "changed",
        note: "success / error",
    },
];

const MARKDOWN_PREVIEW: &str = r#"# Theme preview

Markdown uses `inline code`, [links](https://example.com), tables, rules and lists.

- list marker
- selected theme colors

| slot | source |
| --- | --- |
| link | info |
| rule | border |
"#;

#[tokio::main]
async fn main() {
    element!(Gallery)
        .fullscreen()
        .await
        .expect("Failed to run the application");
}

#[component]
fn Gallery(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut theme_name = hooks.use_state(|| ThemeName::Dracula);
    let mut terminal_bg = hooks.use_state(|| false);
    let mut query = hooks.use_state(|| "palette".to_string());
    let mut submitted = hooks.use_state(|| "ready".to_string());
    let mut exit = hooks.use_exit();

    hooks.use_event_handler(EventScope::Current, EventPriority::High, move |event| {
        let Event::Key(key) = event else {
            return EventResult::Ignored;
        };
        if key.kind != KeyEventKind::Press {
            return EventResult::Ignored;
        }
        match key.code {
            KeyCode::Char('t') | KeyCode::Char('T') => {
                theme_name.set(theme_name.get().next());
                EventResult::Consumed
            }
            KeyCode::Char('b') | KeyCode::Char('B') => {
                terminal_bg.set(!terminal_bg.get());
                EventResult::Consumed
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                exit();
                EventResult::Consumed
            }
            _ => EventResult::Ignored,
        }
    });

    let name = theme_name.get();
    let raw_palette = name.into_kit_palette();
    let palette = if terminal_bg.get() {
        terminal_background(raw_palette)
    } else {
        raw_palette
    };
    let background_mode = if terminal_bg.get() {
        "terminal background"
    } else {
        "theme background"
    };
    let query_view = query.read().to_string();
    let submitted_view = submitted.read().to_string();
    let swatches = [
        ("accent", palette.accent, palette.on_accent),
        ("selection", palette.selection, palette.on_accent),
        ("success", palette.success, Color::Reset),
        ("warning", palette.warning, Color::Reset),
        ("error", palette.error, Color::Reset),
        ("info", palette.info, Color::Reset),
    ];

    let select_items = vec![
        "accent border".to_string(),
        "selection row".to_string(),
        "markdown link".to_string(),
        "diff semantic".to_string(),
    ];
    let columns = vec![
        TableColumn::new(Line::from("Component"), Constraint::Length(14)),
        TableColumn::new(Line::from("State"), Constraint::Length(10)),
        TableColumn::new(Line::from("Theme slot"), Constraint::Length(22)),
    ];
    let rows = COMPONENT_ROWS.to_vec();
    let render_row: RenderTableRow<ComponentRow> = Arc::new(|row, _selected| {
        vec![
            TableCell::new(Line::from(row.component)),
            TableCell::new(Line::from(row.state)),
            TableCell::new(Line::from(row.note)),
        ]
    });

    element!(PaletteProvider(palette: palette) {
        Center(
            width: Constraint::Length(118),
            height: Constraint::Length(36),
        ) {
            Border(
                flex_direction: Direction::Vertical,
                gap: 1,
                style: Style::new().fg(palette.fg).bg(palette.bg),
                top_title: Line::from(format!(" ratatui-kit-themes gallery · {} ({}) ", name.display_name(), name.slug())).centered(),
                bottom_title: Line::from(" t next theme · b background mode · s edit search · q quit ").centered(),
            ) {
                View(height: Constraint::Length(2), flex_direction: Direction::Vertical) {
                    Text(text: Line::from(format!("mode: {background_mode}")))
                    Text(text: Line::from(format!("search: {query_view} · submit: {submitted_view}")).dark_gray())
                }

                View(
                    flex_direction: Direction::Horizontal,
                    gap: 2,
                    height: Constraint::Fill(1),
                ) {
                    Border(
                        width: Constraint::Length(38),
                        flex_direction: Direction::Vertical,
                        gap: 1,
                        top_title: Line::from(" palette + core ").centered(),
                    ) {
                        Border(
                            height: Constraint::Length(8),
                            flex_direction: Direction::Vertical,
                            top_title: Line::from(" swatches ").centered(),
                        ) {
                            for (index, (label, bg, fg)) in swatches.into_iter().enumerate() {
                                View(height: Constraint::Length(1), key: index) {
                                    Text(text: Line::styled(format!(" {label:<10} "), Style::new().fg(fg).bg(bg)))
                                }
                            }
                        }

                        Select<String>(
                            height: Constraint::Length(8),
                            items: select_items,
                            default_index: Some(1),
                            highlight_symbol: "> ",
                            top_title: Line::from(" Select ").centered(),
                        )

                        SearchInput(
                            width: Constraint::Fill(1),
                            value: query.read().to_string(),
                            placeholder: "Press s then type".to_string(),
                            on_change: move |next: String| query.set(next),
                            on_clear: move |_: ()| submitted.set("cleared".to_string()),
                            on_submit: move |value: String| {
                                submitted.set(if value.is_empty() {
                                    "empty submit".to_string()
                                } else {
                                    format!("submitted {value}")
                                });
                                true
                            },
                            validate: move |value: String| {
                                if value.len() > 16 {
                                    (false, "too long".to_string())
                                } else {
                                    (true, "valid".to_string())
                                }
                            },
                        )

                        Border(
                            height: Constraint::Fill(1),
                            flex_direction: Direction::Vertical,
                            justify_content: Flex::Center,
                            top_title: Line::from(" focused border ").centered(),
                            border_style: Style::new().fg(palette.border_active),
                        ) {
                            Text(text: Line::from("Border focus slot").centered())
                        }
                    }

                    Border(
                        width: Constraint::Fill(1),
                        flex_direction: Direction::Vertical,
                        gap: 1,
                        top_title: Line::from(" markdown + table ").centered(),
                    ) {
                        Table<ComponentRow>(
                            height: Constraint::Length(8),
                            columns: columns,
                            rows: rows,
                            render_row: Some(render_row),
                            active: true,
                            default_index: Some(0),
                            border_mode: TableBorderMode::Grid,
                            row_separator: true,
                        )

                        Border(
                            height: Constraint::Length(9),
                            flex_direction: Direction::Vertical,
                            top_title: Line::from(" Markdown ").centered(),
                        ) {
                            Markdown(content: MARKDOWN_PREVIEW.to_string())
                        }

                        View(flex_direction: Direction::Horizontal, gap: 2, height: Constraint::Length(8)) {
                            Border(
                                width: Constraint::Percentage(50),
                                flex_direction: Direction::Vertical,
                                top_title: Line::from(" CodeBlock + quote ").centered(),
                            ) {
                                CodeBlock(
                                    lines: vec![
                                        "let palette = name.into_kit_palette();".to_string(),
                                        "PaletteProvider(palette)".to_string(),
                                    ],
                                    lang: Some("rust".to_string()),
                                    show_line_numbers: Some(true),
                                    show_border: Some(false),
                                    height: Constraint::Length(3),
                                )
                                Divider(char: '─')
                                Blockquote(height: Constraint::Length(2)) {
                                    Text(text: "blockquote uses border_active + surface")
                                }
                            }
                            Border(
                                width: Constraint::Fill(1),
                                flex_direction: Direction::Vertical,
                                top_title: Line::from(" Diff ").centered(),
                            ) {
                                Diff(
                                    old: "theme = dracula\nbackground = theme\n".to_string(),
                                    new: "theme = latte\nbackground = terminal\n".to_string(),
                                    show_line_numbers: Some(true),
                                )
                            }
                        }
                    }
                }
            }
        }
    })
}
