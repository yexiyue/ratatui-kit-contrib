//! Integration tests proving markdown components actually resolve their theme
//! through a real `PaletteProvider` tree (not just `ComponentTheme::from_palette`
//! called directly), and that runtime palette switching recolors the next frame.
//! Uses the core crate's `test-util` offscreen-render harness.

use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::style::Color;
use ratatui_kit::test_util::render_frame;
use ratatui_kit_markdown::{Blockquote, Divider, Markdown};

fn code_cell_fg(buf: &ratatui_kit::ratatui::buffer::Buffer) -> Color {
    buf[(0, 0)].style().fg.unwrap_or(Color::Reset)
}

#[test]
fn markdown_inline_code_follows_palette_provider() {
    let mut red = Palette::default();
    red.info = Color::Red;
    let mut blue = Palette::default();
    blue.info = Color::Blue;

    let red_buf = render_frame(
        element!(PaletteProvider(palette: red) {
            Markdown(content: "`code`".to_string())
        }),
        10,
        1,
    );
    let blue_buf = render_frame(
        element!(PaletteProvider(palette: blue) {
            Markdown(content: "`code`".to_string())
        }),
        10,
        1,
    );

    assert_eq!(
        code_cell_fg(&red_buf),
        Color::Red,
        "inline code should follow palette.info through PaletteProvider"
    );
    assert_eq!(
        code_cell_fg(&blue_buf),
        Color::Blue,
        "switching the injected Palette should change the rendered color"
    );
}

#[test]
fn divider_style_follows_palette_provider() {
    let mut palette = Palette::default();
    palette.border = Color::Magenta;

    let buf = render_frame(
        element!(PaletteProvider(palette: palette) {
            Divider(char: '-')
        }),
        4,
        1,
    );

    assert_eq!(
        buf[(0, 0)].style().fg,
        Some(Color::Magenta),
        "Divider should derive its rule style from palette.border via use_component_theme"
    );
}

#[test]
fn blockquote_bar_follows_palette_provider() {
    let mut palette = Palette::default();
    palette.border_active = Color::Green;

    let buf = render_frame(
        element!(PaletteProvider(palette: palette) {
            Blockquote { Text(text: "x") }
        }),
        6,
        1,
    );

    assert_eq!(
        buf[(0, 0)].style().bg,
        Some(Color::Green),
        "Blockquote's bar should derive from palette.border_active via use_component_theme"
    );
}
