Output crates/ratatui-kit-markdown/assets/markdown-two-phase.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example markdown_two_phase --features markdown-highlight"
Enter

# 关键：启动后不发送任何按键 / 滚动，纯等待。
# 代码块若显示语法高亮，即证明静态文档的第二帧被 use_future 自动触发
# （而非依赖某个无关事件才补高亮）。
Wait+Screen@40s /两阶段渲染演示/
Sleep 2500ms

Screenshot target/vhs-markdown-two-phase.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
