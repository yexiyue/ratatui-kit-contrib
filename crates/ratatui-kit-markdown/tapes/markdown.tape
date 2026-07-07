Output crates/ratatui-kit-markdown/assets/markdown.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example markdown --features markdown-highlight"
Enter
Wait+Screen@20s /Markdown 渲染示例/
Sleep 1200ms

# Move through the rendered Markdown document so the recording shows headings,
# code fences, lists, GFM table rendering, and scrollbars from the real example.
PageDown
Sleep 1200ms
PageDown
Sleep 1200ms
PageUp
Wait+Screen@5s /列表/
Sleep 900ms

Screenshot target/vhs-markdown.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
