Output crates/ratatui-kit-markdown/assets/markdown-streaming.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example markdown_streaming --features markdown"
Enter
Wait+Screen@20s /流式 Markdown 演示/
Sleep 700ms

# Restart the stream, let automatic mode render progressively, then reveal the
# completed document so both transient and final Markdown states are covered.
Type "s"
Sleep 400ms
Type "a"
Sleep 6500ms
Type "f"
Wait+Screen@10s /列表/
Sleep 700ms
PageDown
Sleep 700ms
Wait+Screen@10s /表格/
Sleep 900ms

Screenshot target/vhs-markdown-streaming.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
