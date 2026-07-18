# ratatui-kit-keymap

Configurable keybindings for [`ratatui-kit`](https://github.com/yexiyue/ratatui-kit)
applications: define semantic actions as a plain enum, declare default keys in
code, let users override them from a config file, and dispatch key events by
action.

![keymap demo](assets/keymap-demo.gif)

The recording above is generated from the real
[`examples/keymap_demo.rs`](examples/keymap_demo.rs) with its package-local
[`tapes/keymap_demo.tape`](tapes/keymap_demo.tape): the embedded user config
rebinds `down` to `Ctrl-n`/`n` (watch the help line), while an invalid key
string and an unknown action fall back to defaults with visible warnings.

- **Generic keymap** — one `Keymap<A>` per scope (one action enum per scope);
  compose several for multi-scope apps.
- **Defaults in code, overrides in config** — an action present in the user
  config has its key list replaced entirely; absent actions keep defaults.
- **Validation that never breaks the app** — invalid key strings, conflicts and
  unknown actions come back as structured `KeymapWarning`s (surface them
  however you like); the affected entries fall back to their defaults.
- **Help that never lies** — `describe`/`entries` return the *effective*
  bindings, so help screens reflect user overrides automatically.
- **Config template export** — `to_toml_example()` renders a commented,
  round-trippable template from the default table.
- **`use_keymap_handler` hook** — composes the framework's `use_event_handler`;
  unbound keys stay `Ignored` and keep flowing to other handlers.

Key syntax and formatting come from [`crokey`](https://docs.rs/crokey)
(re-exported): `"j"`, `"ctrl-d"`, `"pagedown"`, `"f1"`... — note the **`-`
separator** (`ctrl-d`, not `ctrl+d`). On top of crokey, a single uppercase
letter implies shift (`"G"` ≡ `"shift-g"`), and multi-key chords (`"g-g"`) are
rejected — terminal key events can never match them.

## Quick start

```toml
[dependencies]
ratatui-kit = "0.10"
ratatui-kit-keymap = "0.1"     # `toml` feature is on by default
serde = { version = "1", features = ["derive"] }
```

```rust
use ratatui_kit_keymap::{Keymap, UseKeymapHandler};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReaderAction {
    ScrollDown,
    PageDown,
    Quit,
}

// At startup: defaults + user overrides (path/IO is your choice).
let mut keymap = Keymap::builder()
    .bind(ReaderAction::ScrollDown, ["j", "down"])
    .desc(ReaderAction::ScrollDown, "scroll down")
    .bind(ReaderAction::PageDown, ["pagedown"])
    .desc(ReaderAction::PageDown, "scroll one page down")
    .bind(ReaderAction::Quit, ["q"])
    .build();
let warnings = keymap.merge_toml_str(&user_config_string)?;
// show `warnings` to the user however fits your app; it stays usable.
```

In a component:

```rust
// Store the merged keymap in an Arc — handlers re-register every frame, and
// an Arc clone is a refcount bump, not a deep copy.
let keymap = std::sync::Arc::new(keymap);

hooks.use_keymap_handler(
    EventScope::Current,
    EventPriority::Normal,
    keymap.clone(),
    move |action, _key| match action {
        ReaderAction::ScrollDown => { /* ... */ EventResult::Consumed }
        ReaderAction::PageDown => { /* ... */ EventResult::Consumed }
        ReaderAction::Quit => { exit(); EventResult::Consumed }
    },
);
```

See [`examples/keymap_demo.rs`](examples/keymap_demo.rs) for a runnable app
demonstrating overrides, warnings and dynamic help:

```sh
cargo run -p ratatui-kit-keymap --example keymap_demo
```

## The config contract

**Serde variant names are your config-file API.** The action's config key name
is its serde variant name (`#[serde(rename_all = "snake_case")]` above turns
`PageDown` into `page_down`). Renaming a variant breaks user configs — treat it
as a breaking change. Stale names in old configs surface as harmless
`UnknownAction` warnings.

A user config is a TOML table of action name → key string(s); trimming it to
only the entries the user wants to change is the intended usage:

```toml
page_down = ["ctrl-d", "space"]  # replaces the default binding entirely
quit = "esc"                     # single string works too
help = []                        # explicit empty list unbinds the action
```

Warning semantics on merge (only real TOML *syntax* errors fail the parse —
everything entry-level degrades gracefully):

| Problem | Effect |
| --- | --- |
| Invalid key string or multi-key chord | that action falls back to its defaults |
| Wrong value type (e.g. `quit = 3`) | that action falls back to its defaults |
| One key bound to several actions | the *overridden* ones fall back to defaults |
| Unknown action name | entry ignored |

An explicit `[]` unbinds an action *without* a warning — it's a feature, but
hosts guarding critical actions (quit!) can check `keymap.keys(action)` after
merging.

The default table itself is developer code: conflicts or invalid strings there
panic in `build()` so they surface during development, not at users' machines.

## Feature flags

| Feature | Default | Unlocks |
| --- | --- | --- |
| `toml` | ✅ | `merge_toml_str` / `merge_toml_table` / TOML parsing of overrides |

`to_toml_example()` and the serde-based `KeymapOverrides` work without any
feature — bring your own format if you don't use TOML.

## Version alignment

`crokey`'s `KeyCombination` is built on `crossterm` types. This crate relies on
cargo unifying `crokey`'s `crossterm ^0.29` with the one re-exported by
`ratatui-kit` (`ratatui_kit::crossterm`, currently 0.29) — if either side ever
moves to a different crossterm major, both must move together.

## First consumer

Designed for and validated by [TRNovel](https://github.com/yexiyue/TRNovel)
(issue #49): user-configurable reader keybindings from
`~/.novel/keybindings.toml`.

## License

MIT
