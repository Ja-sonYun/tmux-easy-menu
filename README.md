# tmux-easy-menu

Easy configurable menus and popups for tmux.

![example](https://github.com/Ja-sonYun/tmux-easy-menu/blob/main/examples/example.gif?raw=true)

## Installation

```bash
cargo install tmux-menu
```

## Quick start

Bind a key in your `~/.tmux.conf` to show a menu:

```tmux
bind-key k run-shell "tmux-menu show --menu $HOME/.config/tmux-menu/menu.yaml --working_dir #{pane_current_path}"
```

Create `~/.config/tmux-menu/menu.yaml`:

```yaml
title: " menu "
items:
  - Menu:
      name: "git"
      shortcut: g
      next_menu: "./git.yaml"
  - Menu:
      name: "terminal"
      shortcut: t
      command: "bash"
      session: true
      session_on_dir: true
      run_on_git_root: true
  - NoDim:
      name: "User: $(whoami)"
  - Seperate: {}
```

Reload tmux and press <prefix>+k to see the menu.  Additional menu files can be
placed in the same directory and referenced with `next_menu`.

## Menu file format

Each menu file is a YAML document with the following structure:

```yaml
title: " menu "           # Title shown at the top
border: "single"          # Default border style (single, rounded, double, heavy, simple, padded, none)
position:                 # Default menu position
  x: 0
  y: 0
items:                     # List of rows in the menu
  - Menu:
      name: "label"       # Text shown in the menu. $(cmd) is evaluated when the menu loads
      shortcut: "k"       # Optional key binding
      command: "echo hi"  # Command to execute
      next_menu: "path"    # Alternatively show another menu file
      background: false    # Run command in background
      close_after_command: true
      session: false       # Run command in a new tmux session
      session_name: "name"
      session_on_dir: false  # Include directory path in session name
      run_on_git_root: false # Execute command from git repo root
      inputs: [KEY]       # Prompt for KEY and replace %%KEY%% in command
      environment:        # Extra environment variables
        MY_VAR: "value"
      position:           # Override popup position
        x: 0
        y: 0
        w: 80
        h: 20
      border: "rounded"  # Override border style
  - NoDim:                 # Non‑selectable line of text
      name: "..."
  - Seperate: {}          # Horizontal separator
```

### Dynamic labels
Names can include `$(command)` expressions which are executed when the menu
loads and replaced with the command output.

### Inputs and placeholders
Use the `inputs` array to ask the user for values before running a command.  The
values are substituted into the command where `%%KEY%%` tokens appear.

### Environment variables
Environment variables defined in an item are available to commands, background
processes and new sessions:

```yaml
  - Menu:
      name: "run with env vars"
      shortcut: e
      command: "echo $MY_VAR && echo $PATH_VAR"
      environment:
        MY_VAR: "Hello World"
        PATH_VAR: "/custom/path"
```

Environment variables work with all execution modes:

- Regular commands: variables are exported before command execution
- Session commands: variables are set in the new tmux session
- Background commands: variables are available to the background process

## Dynamic menus

Menus can be generated on the fly.  The script below lists running `brew`
services and creates a menu to restart them:

```bash
#!/bin/bash
TEMP_MENU_FILE="/tmp/temp_menu.yaml"
rm -f "$TEMP_MENU_FILE"
cat > "$TEMP_MENU_FILE" <<'EOM'
title: " brew services "
items:
  - Seperate: {}
  - NoDim:
      name: " Running services "
  - NoDim:
      name: " (select to restart) "
  - Seperate: {}
EOM

brew services list | while read -r line; do
    program=$(echo "$line" | awk '{print $1}')
    status=$(echo "$line" | awk '{print $2}')
    if [ "$status" = "started" ]; then
        cat >> "$TEMP_MENU_FILE" <<-EOM
  - Menu:
      name: "$program"
      shortcut: ".."
      command: "brew services restart $program"
      background: true
EOM
    fi
done

tmux-menu show --menu "$TEMP_MENU_FILE"
rm -f "$TEMP_MENU_FILE"
```

Add an item in your main menu to call this script:

```yaml
  - Menu:
      name: "restart brew services"
      shortcut: b
      command: "$PATH_TO_SCRIPT/generate_brew_services_restart_menu.sh"
      background: true
```

## Popup command

`tmux-menu` also provides a small helper to show popups and gather input without
a menu definition:

```bash
tmux-menu popup \
  --cmd "echo hi && read" \
  --x 10 --y 10 --w 40 --h 10 \
  --border rounded \
  --key NAME \
  --working_dir .
```

The command prompts for `NAME`, substitutes it into the command as `%%NAME%%` and
displays the result in a tmux popup.

## Examples

See the [`examples`](examples) directory for more configuration samples and
animated demos.
