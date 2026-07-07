Output crates/ratatui-kit-markdown/assets/divider.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example divider"
Enter
Wait+Screen@15s /divider component/
Sleep 700ms

# Cycle through divider glyph styles.
Right
Wait+Screen@5s /style: thick/
Sleep 800ms
Right
Wait+Screen@5s /style: double/
Sleep 800ms
Right
Wait+Screen@5s /style: dot/
Sleep 800ms
Left
Wait+Screen@5s /style: double/
Sleep 800ms

Screenshot target/vhs-divider.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
