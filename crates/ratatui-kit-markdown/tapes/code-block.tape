Output crates/ratatui-kit-markdown/assets/code-block.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example code_block --features highlight"
Enter
Wait+Screen@20s /code_block component/
Sleep 700ms

# Exercise theme switching and line number toggling.
Right
Wait+Screen@5s /base16-ocean.light/
Sleep 900ms
Right
Wait+Screen@5s /base16-eighties.dark/
Sleep 900ms
Type "n"
Wait+Screen@5s /numbers: off/
Sleep 900ms

Screenshot target/vhs-code-block.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
