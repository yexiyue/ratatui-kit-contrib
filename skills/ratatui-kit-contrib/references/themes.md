# ratatui-kit-themes — Theme Adapter Reference

`ratatui-kit-themes` converts [`ratatui-themes`](https://crates.io/crates/ratatui-themes)'
theme catalog into `ratatui_kit::Palette`. It does **not** add a second theme
system — the output is a plain `Palette` you feed into the core
`PaletteProvider`, exactly like a hand-built one.

## API

```rust
pub use ratatui_themes::{Theme, ThemeName, ThemePalette};  // re-exported, don't add ratatui-themes yourself

pub trait IntoKitPalette {
    fn into_kit_palette(self) -> Palette;
}
// implemented for ThemeName, Theme, and ThemePalette

pub fn palette_from_name(name: ThemeName) -> Palette;
pub fn palette_from_theme_palette(source: ThemePalette) -> Palette;
pub fn terminal_background(palette: Palette) -> Palette;
```

Three equivalent ways to get a `Palette` from a `ThemeName` — pick whichever
reads best at the call site:

```rust
use ratatui_kit_themes::{IntoKitPalette, ThemeName, palette_from_name};

let a = ThemeName::Dracula.into_kit_palette();   // extension-trait style
let b = palette_from_name(ThemeName::Dracula);   // plain function
```

## Available themes (`ThemeName`, all 15)

`Dracula`, `OneDarkPro`, `Nord`, `CatppuccinMocha`, `CatppuccinLatte`,
`GruvboxDark`, `GruvboxLight`, `TokyoNight`, `SolarizedDark`,
`SolarizedLight`, `MonokaiPro`, `RosePine`, `Kanagawa`, `Everforest`,
`Cyberpunk`.

`ThemeName::all()` returns a `&'static [ThemeName]` slice — iterate it to
build a theme picker. `.next()` / `.prev()` cycle to the adjacent theme
(wrapping), which is the standard way to wire a "press `t` for next theme"
key handler:

```rust
let mut theme_name = hooks.use_state(|| ThemeName::Dracula);
// in a key handler:
theme_name.set(theme_name.get().next());
```

`.display_name()` gives a human-readable label (`"Tokyo Night"`), `.slug()`
gives a kebab-case id (`"tokyo-night"`) — useful for a status line or a
`--theme` CLI flag that maps a string to a `ThemeName`.

## Palette field mapping

`palette_from_theme_palette` maps every `ThemePalette` field deterministically:

| `Palette` field | Comes from |
| --- | --- |
| `fg` | `fg` |
| `fg_dim` | `muted` |
| `bg`, `surface`, `overlay` | `bg` (all three — see *background strategy* below) |
| `accent` | `accent` |
| `on_accent` | derived — picks Black or White, whichever has better worst-case WCAG contrast against **both** `accent` and `selection` (on_accent is composited over both, depending on which core component reads it) |
| `selection` | `selection` |
| `border` | `muted` |
| `border_active` | `accent` |
| `success` / `warning` / `error` / `info` | same-named field |
| `placeholder` | `muted` |

`ThemePalette` doesn't have a distinct "border" or "placeholder" concept of
its own — those ratatui-kit-specific slots are approximated from `muted`.
This is a lossy, best-effort mapping (documented as such); if a specific
theme's contrast looks off for your app, override just that one `Palette`
field after conversion — `Palette` is a plain struct, not opaque:

```rust
let mut palette = ThemeName::Cyberpunk.into_kit_palette();
palette.border = Color::Rgb(120, 120, 140); // tweak just this one slot
```

## Background strategy: theme background vs. terminal background

By default, the converted `Palette`'s `bg`/`surface`/`overlay` all take the
upstream theme's own background color — your app looks fully "themed",
independent of the user's terminal background:

```rust
let palette = ThemeName::CatppuccinMocha.into_kit_palette();
```

If your app should keep the *terminal's* background instead (common for
CLI tools meant to blend into whatever terminal theme/transparency the user
already has), reset just the background layers with `terminal_background` —
every other color (accent, selection, status colors, fg) still comes from
the theme:

```rust
use ratatui_kit_themes::{IntoKitPalette, ThemeName, terminal_background};

let palette = terminal_background(ThemeName::CatppuccinMocha.into_kit_palette());
```

A common pattern is to let the user toggle between the two at runtime (see
the gallery example below, key `b`) rather than picking one at compile time.

## Runtime theme switching

Same pattern as any other reactive `Palette` in ratatui-kit: hold the
`Palette` (or the `ThemeName` you derive it from) in `use_state` or an
`Atom<Palette>`, and feed it to `PaletteProvider` — a write triggers a
re-render on the next frame automatically, no manual redraw:

```rust
#[component]
fn App(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut theme_name = hooks.use_state(|| ThemeName::Dracula);

    hooks.use_event_handler(EventScope::Current, EventPriority::Normal, move |event| {
        let Event::Key(key) = event else { return EventResult::Ignored };
        if key.kind != KeyEventKind::Press { return EventResult::Ignored }
        if let KeyCode::Char('t') = key.code {
            theme_name.set(theme_name.get().next());
            return EventResult::Consumed;
        }
        EventResult::Ignored
    });

    let palette = theme_name.get().into_kit_palette();
    element!(PaletteProvider(palette: palette) { /* ... your app ... */ })
}
```

## Reference implementation: the gallery example

`ratatui-kit-themes`' own example (`cargo run -p ratatui-kit-themes --example
gallery`) is the most complete reference for combining this crate with
`ratatui-kit-markdown` and core components in one screen — palette swatches,
`Select`/`Table` highlight colors, `SearchInput`, `Markdown`, `CodeBlock`,
`Blockquote`, `Divider`, and `Diff` all under one `PaletteProvider`, with `t`
to cycle themes and `b` to toggle the background strategy. Read
`examples/gallery.rs` in the `ratatui-kit-themes` crate source when you need
a worked example of composing several of these components together, not just
one in isolation.
