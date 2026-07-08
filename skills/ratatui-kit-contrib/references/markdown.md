# ratatui-kit-markdown — Components Reference

All props below are `Option<T>` unless noted, following ratatui-kit's usual
convention: omit a prop in `element!` and it falls back to `Default`. Every
prop table also lists which `*Theme` slot supplies the default when the prop
is `None`.

## Markdown

Parses and renders a full Markdown document: headings, inline styles (bold,
italic, strikethrough, inline code, links), ordered/unordered/nested lists,
GFM tables (with column alignment), fenced code blocks (highlighted if
`highlight` is on), blockquotes, and horizontal rules.

```rust
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::Markdown;

element!(Markdown(content: "# Title\n\nSome **bold** text and `code`.".to_string()));
```

| Prop | Type | Notes |
| --- | --- | --- |
| `content` | `String` (required) | The full Markdown source. Re-parsed only when it changes (`use_memo`) — cheap to re-render every frame even for long docs. |
| layout props (`width`, `height`, `margin`, `gap`, `flex_direction`, `justify_content`) | present but **ignored** | See the *Markdown's layout props are ignored* pitfall in `SKILL.md`. Wrap in a `View`/`ScrollView`/`Border` to size/position it. |

Feature: `markdown` (default on). Needs `highlight` too (or `markdown-highlight`)
for fenced code blocks to be syntax-highlighted — without it they render as
plain unhighlighted text, not an error.

**Scrolling a long document.** `Markdown` computes an exact total content
height (including code blocks and tables, counted as unwrapped logical
lines), so putting it straight inside a `ScrollView` scrolls precisely with
no manual height estimation:

```rust
use ratatui_kit::prelude::*;
use ratatui_kit::ratatui::layout::Direction; // not in prelude::* — import separately
use ratatui_kit_markdown::Markdown;

element!(
    ScrollView(flex_direction: Direction::Vertical) {
        Markdown(content: long_doc)
    }
)
```

Theme: `MarkdownTheme`, derived from the ambient `Palette`:

| Slot | Style | Default derivation |
| --- | --- | --- |
| `heading_marker_style` | the `#`/`##`/... prefix | `palette.fg_dim` |
| `heading_style` | heading text | `palette.warning`, bold |
| `list_marker_style` | bullet / number prefix | `palette.fg_dim` |
| `inline_code_style` | `` `code` `` spans | `palette.info` on `palette.surface` |
| `link_style` | link text | `palette.info`, underlined |
| `link_url_style` | the appended `(url)` suffix | `palette.fg_dim` |
| `table_border_style` | GFM table borders | `palette.border` |
| `rule_style` | `---` horizontal rules | `palette.border` |

There's no per-`Markdown`-call override for individual theme slots (no
`heading_style: Option<Style>` prop) — recolor via `Palette`/`PaletteProvider`,
or `ThemeOverride::<MarkdownTheme>(theme: ...)` for a scoped override (see the
core `ratatui-kit` skill for `ThemeOverride`'s turbofish syntax).

---

## CodeBlock

A standalone code block — line numbers, a language label, optional syntax
highlighting. Use this directly (not through `Markdown`) when you have one
snippet to show, e.g. a diff preview pane, a "generated command" box, or a
config file viewer.

```rust
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::CodeBlock;

element!(CodeBlock(
    lines: vec!["fn main() {".to_string(), "    println!(\"hi\");".to_string(), "}".to_string()],
    lang: "rust".to_string(),
    show_line_numbers: true,
));
```

| Prop | Type | Default | Notes |
| --- | --- | --- | --- |
| `lines` | `Vec<String>` | `[]` | One entry per source line (no trailing `\n`). |
| `lang` | `Option<String>` | `None` | Language id for highlighting (`"rust"`, `"python"`, ...) and the border's language-label title. `None` → no label, no highlighting even with `highlight` on. |
| `show_line_numbers` | `bool` | `true` | |
| `highlight_theme` | `String` | `"base16-ocean.dark"` | A `syntect` theme **name**, not a ratatui `Style`. Built-ins: `base16-ocean.dark`/`.light`, `base16-eighties.dark`, `base16-mocha.dark`, `Solarized (dark)`/`(light)`, `InspiredGitHub`. Custom `.tmTheme` files load via `syntect::ThemeSet::add_from_folder`. Only matters with `highlight` feature on. |
| `line_number_style` | `Option<Style>` | theme | |
| `code_style` | `Option<Style>` | theme | Fallback color for unhighlighted code (no `highlight` feature, or `lang: None`). Ignored when syntax highlighting actually applies — token colors come from the syntect theme instead. |
| `border_style` | `Option<Style>` | theme | |
| `language_label_style` | `Option<Style>` | theme | The `lang` name shown in the border title. |
| `show_border` | `bool` | `true` | |

Always available (no feature required for the component itself; `highlight`
only gates syntax-highlighting behavior).

Theme: `CodeBlockTheme` — `line_number_style` (`palette.fg_dim`), `code_style`
(`palette.fg`), `border_style` (`palette.border`), `language_label_style`
(`palette.info`).

---

## Diff

Two-version text diff with line- and word-level highlighting (word-level
diff shown within changed lines, powered by `similar`).

```rust
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::Diff;

element!(Diff(
    old: "line1\nline2\n".to_string(),
    new: "line1\nmodified\n".to_string(),
    show_line_numbers: true,
));
```

| Prop | Type | Default | Notes |
| --- | --- | --- | --- |
| `old` / `new` | `String` (required) | | Full text of each version. |
| `show_line_numbers` | `bool` | `false` | Shows old/new line-number gutter. |
| `add_style` / `remove_style` / `unchanged_style` / `line_number_style` | `Option<Style>` | theme | Preferred way to recolor. |
| `add_fg` / `add_bg` / `remove_fg` / `remove_bg` / `line_num_color` | `Option<Color>` | theme | Legacy per-channel overrides, kept for backward compatibility. If both a legacy color prop and its sibling `*_style` prop are set, **the legacy color prop wins on that channel**. Don't mix them for the same call — pick one style. |

Feature: `diff` (adds `similar`).

Theme: `DiffTheme` — `add_style` (`palette.success` on `palette.surface`),
`remove_style` (`palette.error` on `palette.surface`), `unchanged_style`
(`palette.fg`), `line_number_style` (`palette.fg_dim`).

---

## Blockquote

A quoted container: a solid left bar plus a subtly-tinted content area,
supporting nested depth. It's a real `Component` (owns its own layout node),
so its children are ordinary child elements, not a `content: String` prop.

```rust
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::Blockquote;

element!(
    Blockquote(depth: 2) {
        Text(text: "Quoted, nesting depth 2")
    }
)
```

| Prop | Type | Default | Notes |
| --- | --- | --- | --- |
| `depth` | `u32` | `1` | Nesting level; widens the left gutter by one column per level. Clamped to a minimum of 1. |
| `bar_style` / `style` | `Option<Style>` | theme | `bar_style` is the left bar, `style` is the content area (fg + background tint). Preferred over the legacy color props below. |
| `prefix_color` / `bg_color` | `Option<Color>` | theme | Legacy: sets just the bar's/content area's background color. Wins over `bar_style`/`style` on the channel it touches if both are set — same precedence rule as `Diff`. |
| `children` | element children | | Arbitrary child elements, not a string. |

Always available, no feature required.

Theme: `BlockquoteTheme` — `bar_style` (background = `palette.border_active`),
`style` (fg `palette.fg` on bg `palette.surface`).

---

## Divider

A horizontal rule — one line of a repeated character.

```rust
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::Divider;

element!(Divider(char: '━'));
```

| Prop | Type | Default | Notes |
| --- | --- | --- | --- |
| `char` | `char` | `'─'` | |
| `style_cfg` | `Option<Style>` | theme (`palette.border`) | Note the field is `style_cfg`, not `style` — `style` is taken on other components but this one predates the convention. |

Always available, no feature required. This is a `#[component]` fn (transparent
layout) but, unlike `Markdown`, it *does* forward its layout props to the
element it returns — `Divider(height: Constraint::Length(1))` works as
expected.

Theme: `DividerTheme` — single `style` slot from `palette.border`.

---

## Theme override semantics (applies to all five components)

Every style-typed prop above follows the same three-state contract, matching
core ratatui-kit components:

- `None` (the default when you don't mention the prop) → use the theme slot as-is.
- `Some(style)` → `theme_style.patch(style)` — your style's set fields win,
  unset fields fall through to the theme.
- `Some(Style::reset())` → clears the theme slot entirely (`Style::reset()`
  has every field set to "cleared", so `patch` wipes it).

This means passing `Style::new().fg(Color::Red)` only overrides the
foreground — background/modifiers from the theme still apply. To fully
replace a slot's look, either set every field you care about explicitly, or
reset first mentally (you rarely need `Style::reset()` — it's for "make this
plain", not "start a fresh custom style").
