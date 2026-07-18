Output crates/ratatui-kit-keymap/assets/keymap-demo.gif

Set Width 1300
Set Height 700
Set FontSize 16
Set Theme "Catppuccin Mocha"
Set TypingSpeed 30ms

Require cargo

Env TERM "xterm-256color"
Env COLORTERM "truecolor"
Env NO_COLOR ""

Type "cargo run --quiet -p ratatui-kit-keymap --example keymap_demo"
Enter
Wait+Screen@30s /item 7/
Sleep 1500ms

# 初始态:item 0 选中;帮助行显示 down 的覆盖键 Ctrl-n/n;告警区两条 ⚠。
# 截图是异步落盘的 —— 每张截图后留足间隔再发下一个键,避免帧偏移。
Screenshot target/vhs-keymap-initial.png
Sleep 1500ms

# j 是被覆盖前的默认键 —— 整条替换后应当无效,选中不动。
Type "j"
Sleep 1200ms
Screenshot target/vhs-keymap-after-j.png
Sleep 1500ms

# 覆盖后的新键 n / Ctrl-n 生效,向下移动两行。
Type "n"
Sleep 600ms
Ctrl+N
Sleep 1200ms
Screenshot target/vhs-keymap-after-n.png
Sleep 1500ms

# shift-g 跳到底部(默认键未被覆盖,照常工作)。
Type "G"
Sleep 1200ms
Screenshot target/vhs-keymap-bottom.png
Sleep 1500ms

# g 回到顶部。
Type "g"
Sleep 1200ms
Screenshot target/vhs-keymap-top.png
Sleep 1500ms

Type "q"
Sleep 800ms
