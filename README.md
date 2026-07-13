<div align="center">

# Ratatui Kit Contrib

**The official monorepo for first-party [`ratatui-kit`](https://github.com/yexiyue/ratatui-kit) extension crates.**

[![CI](https://github.com/yexiyue/ratatui-kit-contrib/actions/workflows/ci.yml/badge.svg)](https://github.com/yexiyue/ratatui-kit-contrib/actions/workflows/ci.yml)
[![ratatui-kit](https://img.shields.io/crates/v/ratatui-kit?logo=rust&color=E43717&label=ratatui-kit)](https://crates.io/crates/ratatui-kit)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](./LICENSE)

**[Main framework](https://github.com/yexiyue/ratatui-kit)** ·
**[Author contract](https://github.com/yexiyue/ratatui-kit/blob/main/COMPONENT_GUIDE.md)** ·
**[Extension API](https://github.com/yexiyue/ratatui-kit/blob/main/EXTENSION_API.md)** ·
**[awesome-ratatui-kit](https://github.com/yexiyue/awesome-ratatui-kit)**

</div>

---

![ratatui-kit-markdown demo](./crates/ratatui-kit-markdown/assets/markdown.gif)

The demo above is generated from the real
[`ratatui-kit-markdown`](./crates/ratatui-kit-markdown) example with its package-local
[`tapes/markdown.tape`](./crates/ratatui-kit-markdown/tapes/markdown.tape). The recording
asset lives next to the crate at
[`assets/markdown.gif`](./crates/ratatui-kit-markdown/assets/markdown.gif).

---

## Overview

This is the home for **official** `ratatui-kit-<name>` component crates — extensions
maintained alongside the framework but shipped as their own packages. If you want to
build UIs with the framework itself, start at the [main repository](https://github.com/yexiyue/ratatui-kit).

Each crate here:

- is **independently versioned** and **independently published** to crates.io as
  `ratatui-kit-<name>` (e.g. `ratatui-kit-markdown`);
- depends on the framework through the public [Extension API](https://github.com/yexiyue/ratatui-kit/blob/main/EXTENSION_API.md)
  from the `ratatui-kit = ">=0.10"` baseline, reaching `ratatui` /
  `crossterm` types via `ratatui_kit::ratatui` / `ratatui_kit::crossterm` rather than a
  direct dependency;
- lives in its own directory under [`crates/`](./crates/) and is released by pushing a
  per-crate tag `ratatui-kit-<name>-v<version>`.

Community (non-official) crates do **not** need to live here — publish them anywhere and
list them in [awesome-ratatui-kit](https://github.com/yexiyue/awesome-ratatui-kit).
This monorepo exists so the crates the core team maintains stay in lockstep with the
framework's release cadence and quality bar.

---

## Crates

| Crate | Description | Version | Original author | Feature-gated deps |
| --- | --- | --- | --- | --- |
| [`ratatui-kit-markdown`](./crates/ratatui-kit-markdown) | Markdown, code block, diff, blockquote and divider components | `0.3.0` | [KonghaYao](https://github.com/KonghaYao) via [ratatui-kit#12](https://github.com/yexiyue/ratatui-kit/pull/12) | `pulldown-cmark` (`markdown`), `syntect` (`highlight`), `similar` (`diff`) |
| [`ratatui-kit-themes`](./crates/ratatui-kit-themes) | `ratatui-themes` catalog adapters for the core `Palette` / `PaletteProvider` pipeline | `0.1.0` | yexiyue | — (`ratatui-themes` is a core dep, not feature-gated) |

> As crates land, add a row here and a member entry in the workspace
> [`Cargo.toml`](./Cargo.toml).

---

## AI-assisted development

This repo ships an **AI agent skill** for developers *using* these crates in their
own `ratatui-kit` app — it teaches your AI coding assistant the real component
props, feature flags, and theming API for `ratatui-kit-markdown` and
`ratatui-kit-themes`, verified against the published source rather than guessed,
so a request like *"render this README with syntax highlighting, themed like
Dracula"* reaches for `Markdown` + `ratatui-kit-themes` and compiles on the first
or second try instead of reinventing a Markdown parser.

```bash
npx skills add yexiyue/ratatui-kit-contrib --skill ratatui-kit-contrib
```

The skill lives in [`skills/ratatui-kit-contrib/`](skills/ratatui-kit-contrib/).
Pair it with the [main framework's `ratatui-kit` skill](https://github.com/yexiyue/ratatui-kit#ai-assisted-development)
— this one only covers what these extension crates add on top.

---

## Repository layout

```text
ratatui-kit-contrib/
├── Cargo.toml           # [workspace] resolver = "3"; members added per crate
├── cliff.toml           # per-crate tag-prefix aware CHANGELOG config
├── rustfmt.toml         # tab_spaces = 4
├── .github/workflows/   # fmt · clippy -D warnings · test · doc, workspace-wide
└── crates/
    └── ratatui-kit-<name>/   # one directory per official extension crate
        ├── assets/           # generated example GIFs for this crate
        ├── tapes/            # reproducible VHS recordings for real examples
        ├── examples/
        └── src/
```

---

## Contributing a crate

Official extension crates are held to the same quality bar as the framework: the
[**author contract**](https://github.com/yexiyue/ratatui-kit/blob/main/COMPONENT_GUIDE.md).
In short:

- **Depend only on the [Extension API surface](https://github.com/yexiyue/ratatui-kit/blob/main/EXTENSION_API.md)**;
  reach `ratatui` / `crossterm` through `ratatui_kit::*`, never a direct dependency.
- **Feature-gate heavy dependencies** (`optional = true` + a feature); keep default
  features minimal so `cargo add ratatui-kit-<name>` stays light.
- **English** panic / expect / error messages shown to library users.
- **Layout props go on the returned root element** for `#[component]` functions
  (transparent-layout wrappers do not own a layout node).
- **Compile baseline**: all examples and doctests must compile — it is the CI gate.
- Publish as `ratatui-kit-<name>` with `keywords = ["ratatui-kit", "tui"]` and use
  the workspace `ratatui-kit >=0.10` baseline.

Run the same validation matrix CI uses before opening a PR:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo test --all-features --workspace --lib --tests --examples
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items --all-features --workspace --examples
```

### Recording examples

Example GIFs live inside the crate they demonstrate. Each package-local tape
writes to that crate's `assets/` directory:

```bash
for tape in crates/*/tapes/*.tape; do
    vhs "$tape"
done
```

Tapes may also emit verification screenshots under `target/vhs-*.png`; `target/`
is ignored, so these PNGs are for local visual checks only. Commit the tape and
generated GIF when an example output changes.

### Releasing

Bump the crate's version → commit → tag `ratatui-kit-<name>-v<version>` → push the tag.
CI runs `cargo publish` for that crate and generates its CHANGELOG with
[git-cliff](https://git-cliff.org/) using the per-crate tag prefix (see [`cliff.toml`](./cliff.toml)).

---

## License

Released under the [MIT License](./LICENSE).
