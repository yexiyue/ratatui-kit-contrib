Output crates/ratatui-kit-markdown/assets/diff-viewer.gif

Set Width 1000
Set Height 600
Set FontSize 18
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet --example diff_viewer --features diff"
Enter
Wait+Screen@20s /diff component/
Sleep 700ms

# Move across representative diff samples and toggle line numbers.
Right
Wait+Screen@5s /add line/
Sleep 900ms
Right
Wait+Screen@5s /remove line/
Sleep 900ms
Type "n"
Wait+Screen@5s /numbers: on/
Sleep 900ms
Right
Wait+Screen@5s /CJK mixed/
Sleep 900ms
Right
Wait+Screen@5s /empty/
Sleep 900ms

Screenshot target/vhs-diff-viewer.png
Sleep 1500ms

Ctrl+C
Sleep 500ms
