# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this
project adheres to [Semantic Versioning](https://semver.org/).

This file can be generated from Conventional Commit history with
[git-cliff](https://git-cliff.org):

```bash
git cliff --tag-pattern '^ratatui-kit-markdown-v[0-9].*' \
          --include-path 'crates/ratatui-kit-markdown/**' \
          -o crates/ratatui-kit-markdown/CHANGELOG.md
```

## [unreleased]

### 🚀 Features

- Initial release: `Markdown`, `CodeBlock`, `Diff`, `Blockquote` and `Divider`
  components migrated out of the core `ratatui-kit` crate.
