//! Adapters from [`ratatui-themes`](https://docs.rs/ratatui-themes) to
//! [`ratatui-kit`](https://docs.rs/ratatui-kit)'s core [`Palette`].
//!
//! This crate intentionally exposes only conversion helpers. Applications still
//! use `ratatui-kit`'s [`PaletteProvider`] and `ComponentTheme` pipeline.
//!
//! ```no_run
//! use ratatui_kit::prelude::*;
//! use ratatui_kit_themes::{IntoKitPalette, ThemeName};
//!
//! let palette = ThemeName::TokyoNight.into_kit_palette();
//! let _app = element!(PaletteProvider(palette: palette) {
//!     Text(text: "Hello from Tokyo Night")
//! });
//! ```
//!
//! [`Palette`]: ratatui_kit::Palette
//! [`PaletteProvider`]: ratatui_kit::PaletteProvider

use ratatui_kit::Palette;
use ratatui_kit::ratatui::style::Color;

pub use ratatui_themes::{Theme, ThemeName, ThemePalette};

/// Extension trait for converting supported upstream theme types into a kit
/// [`Palette`].
///
/// This trait is local to this crate, so it can be implemented for
/// `ratatui-themes` types without violating Rust's orphan rules.
pub trait IntoKitPalette {
    /// Convert into a `ratatui-kit` palette.
    fn into_kit_palette(self) -> Palette;
}

impl IntoKitPalette for ThemeName {
    fn into_kit_palette(self) -> Palette {
        palette_from_name(self)
    }
}

impl IntoKitPalette for Theme {
    fn into_kit_palette(self) -> Palette {
        palette_from_theme_palette(self.palette())
    }
}

impl IntoKitPalette for ThemePalette {
    fn into_kit_palette(self) -> Palette {
        palette_from_theme_palette(self)
    }
}

/// Convert a [`ThemeName`] into a `ratatui-kit` [`Palette`].
#[must_use]
pub fn palette_from_name(name: ThemeName) -> Palette {
    palette_from_theme_palette(name.palette())
}

/// Convert a [`ThemePalette`] into a `ratatui-kit` [`Palette`].
///
/// The mapping is deterministic and keeps `ThemePalette::bg` as the default
/// background strategy. Use [`terminal_background`] when you want the app
/// background to follow the terminal instead.
#[must_use]
pub fn palette_from_theme_palette(source: ThemePalette) -> Palette {
    let mut palette = Palette::default();
    palette.fg = source.fg;
    palette.fg_dim = source.muted;
    palette.bg = source.bg;
    palette.surface = source.bg;
    palette.overlay = source.bg;
    palette.accent = source.accent;
    // on_accent is composited over both `accent` (e.g. Input/SearchInput cursor:
    // bg(accent).fg(on_accent)) and `selection` (e.g. Select/Table highlight:
    // fg(on_accent).bg(selection)) in core ratatui-kit, so it must stay readable
    // against whichever of the two is the worse background, not just one of them.
    palette.on_accent = readable_foreground_for(&[source.accent, source.selection]);
    palette.selection = source.selection;
    palette.border = source.muted;
    palette.border_active = source.accent;
    palette.success = source.success;
    palette.warning = source.warning;
    palette.error = source.error;
    palette.info = source.info;
    palette.placeholder = source.muted;
    palette
}

/// Reset palette background layers so the terminal theme remains visible.
///
/// This preserves all semantic foreground/accent colors from the converted
/// palette and only clears `bg`, `surface`, and `overlay`.
#[must_use]
pub fn terminal_background(mut palette: Palette) -> Palette {
    palette.bg = Color::Reset;
    palette.surface = Color::Reset;
    palette.overlay = Color::Reset;
    palette
}

/// Pick Black or White, whichever gives the better worst-case WCAG contrast
/// ratio across every background it will actually be composited over.
fn readable_foreground_for(backgrounds: &[Color]) -> Color {
    let black_worst = backgrounds
        .iter()
        .map(|&bg| contrast_ratio(Color::Black, bg))
        .fold(f32::INFINITY, f32::min);
    let white_worst = backgrounds
        .iter()
        .map(|&bg| contrast_ratio(Color::White, bg))
        .fold(f32::INFINITY, f32::min);
    if black_worst >= white_worst {
        Color::Black
    } else {
        Color::White
    }
}

/// WCAG relative contrast ratio between two colors (1.0 = no contrast, 21.0 = max).
fn contrast_ratio(fg: Color, bg: Color) -> f32 {
    let l1 = relative_luminance(fg).unwrap_or(1.0);
    let l2 = relative_luminance(bg).unwrap_or(0.0);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

fn relative_luminance(color: Color) -> Option<f32> {
    let (r, g, b) = color_to_rgb(color)?;
    let [r, g, b] = [r, g, b].map(|channel| {
        let channel = f32::from(channel) / 255.0;
        if channel <= 0.03928 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    });
    Some(0.2126 * r + 0.7152 * g + 0.0722 * b)
}

fn color_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Reset => None,
        Color::Black => Some((0, 0, 0)),
        Color::Red => Some((128, 0, 0)),
        Color::Green => Some((0, 128, 0)),
        Color::Yellow => Some((128, 128, 0)),
        Color::Blue => Some((0, 0, 128)),
        Color::Magenta => Some((128, 0, 128)),
        Color::Cyan => Some((0, 128, 128)),
        Color::Gray => Some((192, 192, 192)),
        Color::DarkGray => Some((128, 128, 128)),
        Color::LightRed => Some((255, 0, 0)),
        Color::LightGreen => Some((0, 255, 0)),
        Color::LightYellow => Some((255, 255, 0)),
        Color::LightBlue => Some((0, 0, 255)),
        Color::LightMagenta => Some((255, 0, 255)),
        Color::LightCyan => Some((0, 255, 255)),
        Color::White => Some((255, 255, 255)),
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(index) => indexed_to_rgb(index),
    }
}

fn indexed_to_rgb(index: u8) -> Option<(u8, u8, u8)> {
    const ANSI_16: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    match index {
        0..=15 => Some(ANSI_16[index as usize]),
        16..=231 => {
            let index = index - 16;
            let r = index / 36;
            let g = (index % 36) / 6;
            let b = index % 6;
            Some((cube_channel(r), cube_channel(g), cube_channel(b)))
        }
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            Some((gray, gray, gray))
        }
    }
}

fn cube_channel(value: u8) -> u8 {
    if value == 0 { 0 } else { 55 + value * 40 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_every_upstream_theme_name() {
        for &name in ThemeName::all() {
            let source = name.palette();
            let palette = palette_from_name(name);

            assert_eq!(palette.fg, source.fg, "{}", name.slug());
            assert_eq!(palette.bg, source.bg, "{}", name.slug());
            assert_eq!(palette.accent, source.accent, "{}", name.slug());
            assert_eq!(palette.selection, source.selection, "{}", name.slug());
        }
    }

    #[test]
    fn maps_theme_palette_fields_deterministically() {
        let source = ThemePalette {
            accent: Color::Rgb(1, 2, 3),
            secondary: Color::Rgb(4, 5, 6),
            bg: Color::Rgb(7, 8, 9),
            fg: Color::Rgb(10, 11, 12),
            muted: Color::Rgb(13, 14, 15),
            selection: Color::Rgb(240, 240, 240),
            error: Color::Rgb(16, 17, 18),
            warning: Color::Rgb(19, 20, 21),
            success: Color::Rgb(22, 23, 24),
            info: Color::Rgb(25, 26, 27),
        };

        let palette = palette_from_theme_palette(source);

        assert_eq!(palette.fg, source.fg);
        assert_eq!(palette.fg_dim, source.muted);
        assert_eq!(palette.bg, source.bg);
        assert_eq!(palette.surface, source.bg);
        assert_eq!(palette.overlay, source.bg);
        assert_eq!(palette.accent, source.accent);
        // Worst-case contrast pick: accent (near-black, luminance ~0.00056) wants
        // Black text (contrast ~1.01) while selection (near-white, ~0.87) wants
        // White text (contrast ~1.14) -- White has the better worst case here.
        assert_eq!(palette.on_accent, Color::White);
        assert_eq!(palette.selection, source.selection);
        assert_eq!(palette.border, source.muted);
        assert_eq!(palette.border_active, source.accent);
        assert_eq!(palette.success, source.success);
        assert_eq!(palette.warning, source.warning);
        assert_eq!(palette.error, source.error);
        assert_eq!(palette.info, source.info);
        assert_eq!(palette.placeholder, source.muted);
    }

    fn sample_theme_palette() -> ThemePalette {
        ThemePalette {
            accent: Color::Rgb(100, 100, 100),
            secondary: Color::Rgb(100, 100, 100),
            bg: Color::Rgb(20, 20, 20),
            fg: Color::Rgb(220, 220, 220),
            muted: Color::Rgb(120, 120, 120),
            selection: Color::Rgb(100, 100, 100),
            error: Color::Rgb(200, 50, 50),
            warning: Color::Rgb(200, 150, 50),
            success: Color::Rgb(50, 200, 50),
            info: Color::Rgb(50, 150, 200),
        }
    }

    #[test]
    fn infers_on_accent_when_accent_and_selection_agree() {
        let light = ThemePalette {
            accent: Color::Rgb(245, 245, 245),
            selection: Color::Rgb(245, 245, 245),
            ..sample_theme_palette()
        };
        assert_eq!(palette_from_theme_palette(light).on_accent, Color::Black);

        let dark = ThemePalette {
            accent: Color::Rgb(10, 10, 10),
            selection: Color::Rgb(10, 10, 10),
            ..sample_theme_palette()
        };
        assert_eq!(palette_from_theme_palette(dark).on_accent, Color::White);
    }

    /// Regression test: a theme (shaped like GruvboxDark) where `accent` is a
    /// bright warm color that wants Black text while `selection` is a dark,
    /// desaturated color that wants White text. on_accent must not ignore
    /// `accent` just because `selection` disagrees -- Input/SearchInput's
    /// cursor renders `bg(accent).fg(on_accent)`, so picking White here would
    /// produce near-illegible white-on-bright-yellow text.
    #[test]
    fn on_accent_considers_accent_not_just_selection() {
        let source = ThemePalette {
            accent: Color::Rgb(250, 189, 47),
            selection: Color::Rgb(80, 73, 69),
            ..sample_theme_palette()
        };
        assert_eq!(palette_from_theme_palette(source).on_accent, Color::Black);
    }

    /// Not a full WCAG AA bar (4.5:1) -- some bundled themes have `accent` and
    /// `selection` close enough in luminance that no single Black/White choice
    /// hits that against both. 1.2 is calibrated to the worst real case among
    /// `ThemeName::all()` today (`cyberpunk`, ~1.31:1) with a small margin, so
    /// this still catches a regression back toward the pre-fix ~1.0-1.1 range.
    #[test]
    fn on_accent_is_reasonably_readable_against_both_accent_and_selection() {
        for &name in ThemeName::all() {
            let palette = palette_from_name(name);
            let accent_ratio = contrast_ratio(palette.on_accent, palette.accent);
            let selection_ratio = contrast_ratio(palette.on_accent, palette.selection);
            assert!(
                accent_ratio >= 1.2,
                "{}: on_accent {:?} has poor contrast ({:.2}:1) against accent {:?}",
                name.slug(),
                palette.on_accent,
                accent_ratio,
                palette.accent
            );
            assert!(
                selection_ratio >= 1.2,
                "{}: on_accent {:?} has poor contrast ({:.2}:1) against selection {:?}",
                name.slug(),
                palette.on_accent,
                selection_ratio,
                palette.selection
            );
        }
    }

    #[test]
    fn terminal_background_resets_only_background_layers() {
        let palette = palette_from_name(ThemeName::Nord);
        let reset = terminal_background(palette);

        assert_eq!(reset.bg, Color::Reset);
        assert_eq!(reset.surface, Color::Reset);
        assert_eq!(reset.overlay, Color::Reset);
        assert_eq!(reset.fg, palette.fg);
        assert_eq!(reset.accent, palette.accent);
        assert_eq!(reset.selection, palette.selection);
    }

    #[test]
    fn extension_trait_covers_supported_theme_types() {
        let name_palette = ThemeName::TokyoNight.into_kit_palette();
        let theme_palette = Theme::new(ThemeName::TokyoNight).into_kit_palette();
        let raw_palette = ThemeName::TokyoNight.palette().into_kit_palette();

        assert_eq!(name_palette, theme_palette);
        assert_eq!(name_palette, raw_palette);
    }

    #[test]
    fn does_not_reference_the_builder_theme_crate() {
        let manifest = include_str!("../Cargo.toml");
        let lib = include_str!("lib.rs");
        let crate_name = concat!("ratatui-", "themekit");
        let rust_path = concat!("ratatui_", "themekit");

        assert!(!manifest.contains(crate_name));
        assert!(!lib.contains(rust_path));
    }
}
