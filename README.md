# tmux-easy-menu

**tmux-easy-menu** provides a small `tmux-menu` binary that lets you build rich
interactive menus and popups inside tmux using simple YAML files.  Commands run
in the current pane by default, but you can also spawn new sessions or run tasks
in the background without touching tmux scripting.

![demo](https://github.com/Ja-sonYun/tmux-easy-menu/blob/main/examples/example.gif?raw=true)

## Features
- Declarative YAML menus with nested sub‑menus
- Run commands, background jobs or new tmux sessions
- Dynamic labels with `$(command)` substitution
- Prompt for user input and inject values into commands
- Per‑item environment variables, borders and positions
- Generate menus at runtime
- Standalone popup helper for one‑off prompts

## Installation

```bash
cargo install tmux-menu
```

## Quick start
1. Bind a key in `~/.tmux.conf`:
   ```tmux
   bind-key k run-shell "tmux-menu show --menu $HOME/.config/tmux-menu/menu.yaml --working_dir #{pane_current_path}"
   ```
2. Create `~/.config/tmux-menu/menu.yaml`:
   ```yaml
title: " menu "
items:
  - Menu:
      name: "git tools"
      shortcut: g
      next_menu: "./git.yaml"
  - Menu:
      name: "terminal"
      shortcut: t
      command: "bash"
      session: true
  - NoDim:
      name: "User: $(whoami)"
  - Seperate: {}
   ```
3. Reload tmux and press `<prefix>+k`.

## Menu files
A menu file is a YAML document:

```yaml
title: " title "
border: rounded
position:
  x: 0
  y: 0
items:
  - Menu:
      name: "run command"
      shortcut: r
      command: "echo hi"
      background: false
      close_after_command: true
      session: false
      session_name: "name"
      session_on_dir: false
      run_on_git_root: false
      inputs: [KEY]
      environment:
        FOO: bar
      position:
        x: 0
        y: 0
        w: 80
        h: 20
      border: single
  - NoDim:
      name: "label"
  - Seperate: {}
```

### Item types
- **Menu** – selectable entry that runs a command or shows `next_menu`
- **NoDim** – non‑selectable text line
- **Seperate** – horizontal separator

### Dynamic labels
Values in `name` may include `$(cmd)` which executes when the menu loads.

### Inputs and variables
Use `inputs` to prompt for values.  The entered text replaces `%%KEY%%` tokens in
the command.  The `environment` map exports variables for commands, sessions and
background tasks.

### Borders and position
`border` accepts `single`, `rounded`, `double`, `heavy`, `simple`, `padded` or
`none`.  `position` sets popup location and size (`x`, `y`, `w`, `h`).

## Dynamic menus
Menus can be generated on the fly.  The script below lists running `brew`
services and creates a menu to restart them:

```bash
#!/bin/bash
TEMP_MENU="/tmp/temp_menu.yaml"
cat > "$TEMP_MENU" <<'EOM'
title: " brew services "
items:
  - Seperate: {}
  - NoDim: {name: " Running services "}
  - NoDim: {name: " (select to restart) "}
  - Seperate: {}
EOM

brew services list | awk '$2=="started"{print $1}' | while read svc; do
  cat >> "$TEMP_MENU" <<EOM
  - Menu:
      name: "$svc"
      command: "brew services restart $svc"
      background: true
EOM
done

tmux-menu show --menu "$TEMP_MENU"
rm -f "$TEMP_MENU"
```

Add an item to your main menu to call this script and show the generated menu.

## Popup helper
Display a popup without writing a menu file:

```bash
tmux-menu popup \
  --cmd "echo hi && read" \
  --x 10 --y 10 --w 40 --h 10 \
  --border rounded \
  --key NAME \
  --working_dir .
```

The command prompts for `NAME`, substitutes it into the command as `%%NAME%%`
and shows the result in a tmux popup.

## Examples
More demos and sample configurations live in the [`examples`](examples) folder.

## License
MIT

