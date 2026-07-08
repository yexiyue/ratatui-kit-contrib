Output crates/ratatui-kit-themes/assets/gallery.gif

Set Width 1100
Set Height 720
Set FontSize 16
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet -p ratatui-kit-themes --example gallery"
Enter
Wait+Screen@20s /ratatui-kit-themes gallery/
Sleep 1200ms
Screenshot target/vhs-themes-gallery-dark.png
Sleep 500ms

# Cycle to Catppuccin Latte so verification covers a light upstream theme.
Type "t"
Sleep 800ms
Type "t"
Sleep 800ms
Type "t"
Sleep 800ms
Type "t"
Wait+Screen@5s /Catppuccin Latte/
Sleep 1200ms
Screenshot target/vhs-themes-gallery-light.png
Sleep 500ms

# Show terminal-background mode as a separate state.
Type "b"
Sleep 1200ms

Type "q"
Sleep 500ms
