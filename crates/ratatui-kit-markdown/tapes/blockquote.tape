Output crates/ratatui-kit-markdown/assets/blockquote.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example blockquote"
Enter
Wait+Screen@15s /blockquote component/
Sleep 700ms

# Cycle the controlled depth indicator while showing all quote depths.
Right
Wait+Screen@5s /current depth: 2/
Sleep 900ms
Right
Wait+Screen@5s /current depth: 3/
Sleep 900ms

Screenshot target/vhs-blockquote.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
