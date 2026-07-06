# ratatui-kit-markdown

[![crates.io](https://img.shields.io/crates/v/ratatui-kit-markdown?logo=rust&color=E43717)](https://crates.io/crates/ratatui-kit-markdown)
[![docs.rs](https://img.shields.io/docsrs/ratatui-kit-markdown?logo=docsdotrs)](https://docs.rs/ratatui-kit-markdown)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

Markdown rendering (and the components it is built from) for
[`ratatui-kit`](https://github.com/yexiyue/ratatui-kit). The first official
extension crate in [`ratatui-kit-contrib`](https://github.com/yexiyue/ratatui-kit-contrib).

## Components

| Component | Feature | Description |
| --- | --- | --- |
| `Markdown` | `markdown` (default) | Parse and render a Markdown document — headings, inline styles, ordered/unordered (and nested) lists, GFM tables, code fences, blockquotes and rules. |
| `CodeBlock` | always on | A code block with optional line numbers and a language label. With `highlight`, code is syntax-highlighted via [syntect](https://crates.io/crates/syntect). |
| `Diff` | `diff` | A two-version text diff with line- and word-level highlighting via [similar](https://crates.io/crates/similar). |
| `Blockquote` | always on | A quoted container with a solid left bar and configurable nesting depth. |
| `Divider` | always on | A horizontal rule. |

## Features

Heavy dependencies are optional and feature-gated so `cargo add ratatui-kit-markdown`
stays light:

| Feature | Default | Pulls in |
| --- | --- | --- |
| `markdown` | ✅ | `pulldown-cmark`, `unicode-width` — the `Markdown` component |
| `highlight` | | `syntect` — syntax highlighting for `CodeBlock` / code fences |
| `diff` | | `similar` — the `Diff` component |
| `markdown-highlight` | | `markdown` + `highlight` |

```toml
[dependencies]
# Markdown with highlighted code fences:
ratatui-kit-markdown = { version = "0.1", features = ["markdown-highlight"] }
```

## Usage

```rust,no_run
use ratatui_kit::prelude::*;
use ratatui_kit_markdown::Markdown;

#[tokio::main]
async fn main() {
    let content = "# Hello\n\nSome **bold** and `code`.\n\n- a\n  - b\n- c";
    element!(Markdown(content: content.to_string()))
        .fullscreen()
        .await
        .expect("failed to run the application");
}
```

The renderer computes an exact content height for each document, so a `Markdown`
placed inside a `ScrollView` scrolls precisely. See [`examples/`](./examples) for
runnable demos:

```bash
cargo run --example markdown --features markdown-highlight
cargo run --example markdown_streaming --features markdown
cargo run --example code_block --features highlight
cargo run --example diff_viewer --features diff
cargo run --example blockquote
cargo run --example divider
```

## Authoring contract

This crate follows the framework's
[`COMPONENT_GUIDE.md`](https://github.com/yexiyue/ratatui-kit/blob/main/COMPONENT_GUIDE.md)
and depends only on the public
[Extension API surface](https://github.com/yexiyue/ratatui-kit/blob/main/EXTENSION_API.md):

- `ratatui` / `crossterm` types are reached through `ratatui_kit::ratatui` /
  `ratatui_kit::crossterm` — never a direct `ratatui` dependency;
- heavy dependencies are `optional` + feature-gated, default features minimal;
- runtime panic / error messages are English;
- `#[component]` functions forward their layout props onto the returned root
  element (transparent-layout wrappers do not own a layout node);
- all examples and doctests compile — the CI regression baseline.

Run the same checks CI runs:

```bash
cargo test --all-features --lib --tests --examples
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items --all-features --examples
```

## License

Released under the [MIT License](./LICENSE).
