# tmux-easy-menu v0.1.16

### Easy configurable tmux display-menu

![Alt Text](https://github.com/Ja-sonYun/tmux-easy-menu/blob/main/examples/example.gif?raw=true)

## Setup
```
cargo install tmux-menu
```


## Configuration
To see more actual config files, checkout `./examples` folder.
```yaml
# On tmux.conf, add below line.
#
# bind-key k run-shell "tmux-menu show --menu $HOME/tmux-menu/examples/menu.yaml --working_dir #{pane_current_path}"
#                      
# =============================
#
title: "..."
border: "rounded"                   # Optional, possible options are: single, rounded, double, heavy, simple, padded, none
position:
  x: ...
  y: ...

items:
  - Seperate: {}                    # Draw seperate line

  - NoDim:                          # Add row, but unselectable
      name: "..."

  - Menu:                           # Add selectable row
      name: "..."
      shortcut: "..."               # Optional
#     ------------------
      # You can define next_menu or command
      next_menu: "..."              # Show this menu when this row selected

#       ... OR

      command: "command %%KEY%% --cwd %%PWD"    # Run command, %%PWD will replaced with cwd
      inputs:
        - KEY                       # This input will be replaced with '%%KEY%%' on command
#     ------------------
      background: false             # Run command in background, popup will closed immediately
      close_after_command: true     # Close popup after command exited. if false, you should type <C-c> to close popup.
      border: none                  # Select popup border type, optional, possible options are: single, rounded, double, heavy, simple, padded, none
      session: false                # Run commmand in new session. Useful for long running command. To hide popup while command running, use <C-d> to detach and close.
      session_name: name            # Session name, which will be used if session is true. This must be unique.
      session_on_dir: false         # Include directory path in session name
      run_on_git_root: false        # Run command from git repository root instead of current directory
      environment:                  # Set environment variables for command execution
        MY_VAR: "value"
        ANOTHER_VAR: "another value"
      position:
        x: ...
        y: ...
        w: ...
        h: ...
```

#### Dynamic menu
![Alt Text](https://github.com/Ja-sonYun/tmux-easy-menu/blob/main/examples/dynamic2.gif?raw=true)
![Alt Text](https://github.com/Ja-sonYun/tmux-easy-menu/blob/main/examples/dynamic.gif?raw=true)
Below example will show running brew services on display-menu, and restart it if clicked.
```bash
#!/bin/bash
# generate_brew_services_restart_menu.sh

TEMP_MENU_FILE="/tmp/temp_menu.yaml"
rm -f $TEMP_MENU_FILE
cat > $TEMP_MENU_FILE << EOM
title: " brew services "
items:
  - Seperate: {}
  - NoDim:
      name: " Running services "
  - NoDim:
      name: " (select to restart) "
  - Seperate: {}
EOM

brew services list | while read line
do
    program=$(echo $line | awk '{print $1}')
    status=$(echo $line | awk '{print $2}')

    if [ "$status" == "started" ]; then
        cat >> $TEMP_MENU_FILE <<- EOM
  - Menu:
      name: "$program"
      shortcut: ".."
      command: "brew services restart $program"
      background: true
EOM
    fi
done

tmux-menu show --menu $TEMP_MENU_FILE
rm -f $TEMP_MENU_FILE
```
and add menu item as below
```yaml
  - Menu:
      name: "restart brew services"
      shortcut: b
      command: "$PATH_TO_SCRIPT/generate_brew_services_restart_menu.sh"
      background: true
```

#### Environment Variables
You can set environment variables that will be available when your commands execute:

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
- Regular commands: Variables are exported before command execution
- Session commands: Variables are set in the new tmux session
- Background commands: Variables are available to the background process
