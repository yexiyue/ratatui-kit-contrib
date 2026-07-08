//! # ratatui-kit-markdown
//!
//! A first-party [`ratatui-kit`](https://docs.rs/ratatui-kit) extension crate that
//! renders Markdown (and the building blocks it is made of) in the terminal:
//!
//! - [`Markdown`] — parse and render a Markdown document (headings, inline styles,
//!   lists, tables, code fences, blockquotes, rules). *(feature `markdown`, default)*
//! - [`CodeBlock`] — a standalone code block with optional line numbers, a language
//!   label and (feature `highlight`) syntect-powered syntax highlighting.
//! - [`Diff`] — a two-version text diff with line/word level highlighting.
//!   *(feature `diff`)*
//! - [`Blockquote`] — a quoted container with a solid left bar and nesting depth.
//! - [`Divider`] — a horizontal rule.
//!
//! Everything follows the framework's authoring contract (see
//! [`COMPONENT_GUIDE.md`](https://github.com/yexiyue/ratatui-kit/blob/main/COMPONENT_GUIDE.md)):
//! it depends only on the [Extension API](https://github.com/yexiyue/ratatui-kit/blob/main/EXTENSION_API.md)
//! surface, reaches `ratatui` types through `ratatui_kit::ratatui`, and keeps runtime
//! messages in English.
//!
//! ## Features
//!
//! | Feature | Default | Enables |
//! | --- | --- | --- |
//! | `markdown` | ✅ | the [`Markdown`] component (pulldown-cmark, unicode-width) |
//! | `highlight` | | syntax highlighting for [`CodeBlock`] (syntect) |
//! | `diff` | | the [`Diff`] component (similar) |
//! | `markdown-highlight` | | `markdown` + `highlight` |
//!
//! [`CodeBlock`], [`Blockquote`] and [`Divider`] are always available (no heavy
//! dependencies).
//!
//! ## Example
//!
//! ```no_run
//! use ratatui_kit::prelude::*;
//! use ratatui_kit_markdown::Markdown;
//!
//! #[tokio::main]
//! async fn main() {
//!     element!(Markdown(content: "# Hello\n\nSome **bold** text.".to_string()))
//!         .fullscreen()
//!         .await
//!         .expect("failed to run the application");
//! }
//! ```

/// Theme slots derived from `ratatui-kit` palettes.
pub mod theme;
pub use theme::*;

/// Quoted container component with a solid left bar and nesting depth.
pub mod blockquote;
pub use blockquote::*;

/// Standalone code block component with optional syntax highlighting.
pub mod code_block;
pub use code_block::*;

/// Horizontal rule / separator component.
pub mod divider;
pub use divider::*;

/// Two-version text diff component (feature `diff`).
#[cfg(feature = "diff")]
pub mod diff;
#[cfg(feature = "diff")]
pub use diff::*;

/// Markdown document component (feature `markdown`).
#[cfg(feature = "markdown")]
pub mod markdown;
#[cfg(feature = "markdown")]
pub use markdown::*;
