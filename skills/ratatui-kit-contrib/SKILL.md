---
name: ratatui-kit-contrib
description: >-
  Use the official ratatui-kit-contrib extension crates in a ratatui-kit
  terminal UI: ratatui-kit-markdown (Markdown, CodeBlock, Diff, Blockquote,
  Divider components) and ratatui-kit-themes (drop-in Catppuccin / Nord /
  Tokyo Night / Dracula / Gruvbox / Solarized / ... theme presets via
  ratatui-themes). Use this skill whenever a ratatui-kit app needs to render
  Markdown or docs, show a syntax-highlighted code block, display a text
  diff, render a quoted/indented block, draw a horizontal rule, or apply a
  named color theme / let the user switch themes — even if the user just
  says "render this markdown", "show a diff view", "add a code block",
  "theme this TUI like Dracula/Nord/Catppuccin", or "let users pick a color
  scheme" without naming the crates. Reach for this before hand-rolling
  Markdown parsing, a diff algorithm, or a color palette — these crates
  already solve it and integrate with ratatui-kit's Palette/PaletteProvider
  theme protocol.
license: MIT
metadata:
  author: yexiyue
  framework: ratatui-kit-contrib
  version: "1.0.0"
---

# ratatui-kit-contrib — Markdown & Theme Extensions for ratatui-kit

[`ratatui-kit-contrib`](https://github.com/yexiyue/ratatui-kit-contrib) is the
official extension monorepo for [ratatui-kit](https://ratatui-kit's core
framework). It ships two published crates:

| Crate | Gives you | crates.io |
| --- | --- | --- |
| `ratatui-kit-markdown` | `Markdown`, `CodeBlock`, `Diff`, `Blockquote`, `Divider` components | [`ratatui-kit-markdown`](https://crates.io/crates/ratatui-kit-markdown) |
| `ratatui-kit-themes` | `ratatui-themes` catalog (Catppuccin, Nord, Tokyo Night, Dracula, Gruvbox, Solarized, ...) → `ratatui_kit::Palette` adapter | [`ratatui-kit-themes`](https://crates.io/crates/ratatui-kit-themes) |

**Requires the `ratatui-kit` skill too.** This skill only covers what these
two crates add — component props, feature flags, and the theme adapter API.
It assumes you already know `element!`, `#[component]`, hooks, and
`PaletteProvider` from the core framework. Install/consult `ratatui-kit`
alongside this skill; don't reinvent core concepts here.

**Small crates, easy to get subtly wrong.** Props are feature-gated, some
components silently ignore certain props (see *Pitfalls*), and the version
history matters (`ratatui-kit-markdown` jumped from hardcoded colors to a
theme system in `0.2.0`). Verify against this skill and a real `cargo check`
rather than recalling the API from general Markdown/diff-library experience.

---

## Which crate — quick decision guide

| Need | Use |
| --- | --- |
| Render a `.md` file / doc / README / changelog in the TUI | `ratatui-kit-markdown`'s `Markdown` |
| A code snippet with line numbers / syntax highlighting, **not** inside a larger Markdown doc | `ratatui-kit-markdown`'s `CodeBlock` |
| Side-by-side or unified view of two text versions (before/after, git-style) | `ratatui-kit-markdown`'s `Diff` |
| A callout / admonition / quoted block with a left bar | `ratatui-kit-markdown`'s `Blockquote` |
| A plain horizontal rule | `ratatui-kit-markdown`'s `Divider` |
| Ship a named color scheme (Dracula, Nord, Catppuccin, ...) instead of hand-picking colors | `ratatui-kit-themes` |
| Let the user cycle/pick a theme at runtime | `ratatui-kit-themes` + `use_state`/`Atom<Palette>` driving `PaletteProvider` |

For full props tables and theming details:

| Working on… | Read |
| --- | --- |
| Markdown/CodeBlock/Diff/Blockquote/Divider props, theming, feature flags | `references/markdown.md` |
| ratatui-kit-themes API, palette mapping, background strategies, gallery pattern | `references/themes.md` |

---

## Project setup

Neither crate is bundled by `ratatui-kit`'s own `full` feature — add them
explicitly. `ratatui-kit-markdown` keeps its own defaults light (only the
`Markdown` component ships by default); `ratatui-kit-themes` re-exports
`ratatui-themes`' types, so you do **not** need `ratatui-themes` as a
separate dependency.

```toml
[dependencies]
ratatui-kit = { version = "0.10", features = ["full"] }
ratatui-kit-markdown = { version = "0.2", features = ["markdown-highlight", "diff"] }
ratatui-kit-themes = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
```

| `ratatui-kit-markdown` feature | Default | Unlocks |
| --- | --- | --- |
| `markdown` | ✅ | the `Markdown` component (`pulldown-cmark`) |
| `highlight` | | syntax highlighting for `CodeBlock` (`syntect`) — without it, code renders unhighlighted plain text, it does **not** fail to compile |
| `diff` | | the `Diff` component (`similar`) |
| `markdown-highlight` | | shorthand for `markdown` + `highlight` |

`CodeBlock`, `Blockquote`, and `Divider` are **always available**, no feature
needed. `ratatui-kit-themes` has no features to pick — it's one thing.

---

## Minimal example (Markdown + a real theme together)

This is the shape most requests take: render some Markdown, themed with a
named palette instead of hand-picked colors.

```rust
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::Markdown;
use ratatui_kit_themes::{IntoKitPalette, ThemeName};

#[tokio::main]
async fn main() {
    let palette = ThemeName::TokyoNight.into_kit_palette();
    let content = "# Hello\n\nSome **bold** text, `inline code`, and a [link](https://example.com).";

    element!(PaletteProvider(palette: palette) {
        Markdown(content: content.to_string())
    })
    .fullscreen()
    .await
    .expect("Failed to run the application");
}
```

`Markdown`, `CodeBlock`, `Blockquote`, `Divider`, and `Diff` all derive their
default colors from whatever `Palette` is in scope (via `PaletteProvider`) —
you get a coherent look automatically, no per-component styling needed.
Override one spot with the component's own style prop (`None` = use theme,
`Some(style)` = patch over the theme, `Some(Style::reset())` = clear it —
same semantics as core ratatui-kit components). Full theme-slot tables are in
`references/markdown.md`.

---

## Common pitfalls (verified against the source, not guessed)

- **`Markdown`'s layout props compile but do nothing.** `MarkdownProps` has
  `width`/`height`/`margin`/`gap`/`flex_direction`/`justify_content` fields
  (every `ratatui-kit-markdown` props type does, via `#[with_layout_style]`),
  so `Markdown(width: Constraint::Length(40))` **compiles fine** — but
  `Markdown`'s implementation never reads them. Its height is always
  auto-computed to fit the parsed content, and nothing else is forwarded.
  This is different from `CodeBlock`/`Blockquote`/`Divider`/`Diff`, which
  *do* honor their layout props. To size or position a `Markdown` block,
  wrap it: `View(width: ..., height: ...) { Markdown(content: ...) }` or put
  it in a `ScrollView`/`Border`/`Center`. `Markdown`'s computed height is
  exact, so it composes correctly inside a `ScrollView` for scrolling long
  documents — don't manually estimate a height for that case.

- **No `PaletteProvider` → colors still look sane, but they're the
  *default* palette, not your app's.** These components never panic or
  render unstyled without a provider (every `ComponentTheme` falls back to
  `Palette::default()`) — but if you expect a specific look, make sure the
  component is actually inside your `PaletteProvider` subtree, not a sibling
  of it.

- **`highlight_theme` on `CodeBlock` is a `syntect` theme *name* string**
  (e.g. `"base16-ocean.dark"`, `"InspiredGitHub"`), not a `ratatui_kit`
  `Style`/`Palette` — it only takes effect when the `highlight` feature is
  on and only affects syntax-highlighted token colors, not the border/line
  number/language-label styles (those come from `CodeBlockTheme`, i.e. the
  ambient `Palette`).

- **`Diff`'s legacy color props (`add_fg`, `add_bg`, `remove_fg`,
  `remove_bg`, `line_num_color`) and its newer style props (`add_style`,
  `remove_style`, `unchanged_style`, `line_number_style`) can both be set at
  once** — the legacy color prop wins on the channel it controls. Prefer the
  newer `*_style` props in new code; the color props exist for backward
  compatibility.

- **Don't add `ratatui-themes` as a direct dependency.** `ratatui-kit-themes`
  re-exports `ThemeName`, `Theme`, and `ThemePalette` — importing them from
  `ratatui_kit_themes` is enough. Adding a second, separate `ratatui-themes`
  dependency just risks a version mismatch for no benefit.

- **`ThemeName::X.into_kit_palette()` keeps the theme's own background by
  default.** If your app should keep the *terminal's* background (e.g. a
  translucent-terminal setup, or matching other CLI tools), wrap the result:
  `terminal_background(ThemeName::X.into_kit_palette())`. See
  `references/themes.md` for exactly which `Palette` fields that resets.

For the general ratatui-kit pitfalls (hook order, transparent layout on
`#[component]` in general, missing `mut` on state handles, feature gating a
component you forgot to enable), see the `ratatui-kit` skill — they apply
here too since these are ordinary ratatui-kit components.

---

## Verifying

Same discipline as core ratatui-kit: **a clean compile is the definition of
done**. In your own app, the features you need are pinned directly on the
dependency lines in `Cargo.toml` (as in *Project setup* above), so a plain
`cargo check` picks them up — you don't pass `--features` on the CLI unless
your own crate defines its own feature flags that forward to these crates:

```bash
cargo check
cargo clippy --all-targets -- -D warnings
cargo run
```

(The `cargo check --features <name>` form only applies if you're working
*inside* `ratatui-kit-markdown`/`ratatui-kit-themes` themselves, not a
downstream app — don't copy that form into your own project's checks.)

If a prop you passed seems to have no effect, check *Pitfalls* above first —
`Markdown`'s layout props are the most common surprise, followed by using a
color-only `Diff` prop when you meant the newer `*_style` prop.
