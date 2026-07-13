# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this
project adheres to [Semantic Versioning](https://semver.org/).

## [0.3.0] - 2026-07-13

### 🚀 Features

- *(markdown)* Two-phase rendering - first frame light, subsequent frames full syntect (#1)

### 🐛 Bug Fixes

- *(markdown)* 两阶段渲染改用 use_future 主动唤醒第二帧,并恢复 render API 兼容

### 📚 Documentation

- *(markdown)* 加 markdown_two_phase 示例演示两阶段渲染
- *(markdown)* README 增补 markdown_two_phase 示例与两阶段渲染说明
## [0.2.0] - 2026-07-08

### 🚀 Features

- 新增 ratatui-kit-themes,为 ratatui-kit-markdown 接入主题协议

### 📚 Documentation

- *(markdown)* Credit original contributor

### 🧪 Testing

- *(markdown)* 补 PaletteProvider 端到端集成测试
## [0.1.0] - 2026-07-07

### 🚀 Features

- 新增 ratatui-kit-markdown(从 ratatui-kit #12 迁移,修 review 3 个 bug,基于 0.8.0)

### 📚 Documentation

- *(markdown)* Add example recordings

### ⚙️ Miscellaneous Tasks

- *(release)* 迁移 markdown 发布流程
