//! Configurable keybindings for [`ratatui-kit`](https://docs.rs/ratatui-kit)
//! applications.
//!
//! Define semantic actions as a plain enum, declare their default keys in
//! code, let users override them from a config file, and dispatch key events
//! by action — with validation that never breaks the app and help output that
//! always shows the real bindings.
//!
//! One [`Keymap<A>`] is one scope (one action enum per scope); apps with
//! several scopes compose several keymaps. Key syntax and formatting come
//! from [`crokey`] (`"j"`, `"ctrl-d"`, `"shift-g"`, `"pagedown"`, `"f1"`...,
//! note the `-` separator), which is re-exported for convenience.
//!
//! ```
//! use ratatui_kit_keymap::{Keymap, crokey::key};
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
//! #[serde(rename_all = "snake_case")]
//! enum ReaderAction {
//!     ScrollDown,
//!     PageDown,
//! }
//!
//! // Defaults live in code; variant names are the config-file contract.
//! let mut keymap = Keymap::builder()
//!     .bind(ReaderAction::ScrollDown, ["j", "down"])
//!     .bind(ReaderAction::PageDown, ["pagedown"])
//!     .desc(ReaderAction::PageDown, "scroll one page down")
//!     .build();
//!
//! // User overrides replace an action's keys entirely; problems become
//! // warnings, never errors — the app stays usable.
//! # #[cfg(feature = "toml")]
//! let warnings = keymap.merge_toml_str("page_down = [\"ctrl-d\"]").unwrap();
//! # #[cfg(feature = "toml")]
//! assert!(warnings.is_empty());
//! # #[cfg(feature = "toml")]
//! assert_eq!(keymap.action_for(key!(ctrl-d)), Some(ReaderAction::PageDown));
//! ```
//!
//! In a component, dispatch with the [`UseKeymapHandler`] hook:
//!
//! ```ignore
//! hooks.use_keymap_handler(
//!     EventScope::Current,
//!     EventPriority::Normal,
//!     keymap.clone(),
//!     move |action, _key| match action {
//!         ReaderAction::ScrollDown => { /* ... */ EventResult::Consumed }
//!         ReaderAction::PageDown => { /* ... */ EventResult::Consumed }
//!     },
//! );
//! ```

mod keymap;
mod use_keymap_handler;

pub use crokey;
pub use keymap::{Keymap, KeymapBuilder, KeymapEntry, KeymapOverrides, KeymapWarning};
#[cfg(feature = "toml")]
pub use toml;
pub use use_keymap_handler::UseKeymapHandler;
