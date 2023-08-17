# tmux-easy-menu v0.1.7

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
#                       ^~~ Or add binary to your path
# =============================
#
# title: "..."
# border: "rounded"                   # Optional
# position:
#   x: ...
#   y: ...
#
# items:
#   - Seperate: {}                    # Draw seperate line
#
#   - NoDim:                          # Add row, but unselectable
#       name: "..."
#
#   - Menu:                           # Add selectable row
#       name: "..."
#       shortcut: "..."               # Optional
#       ------------------
#
#       next_menu: "..."              # Show this menu if it selected
#
#        ... OR
#
#       command: "command %%KEY%% --cwd %%PWD"    # Or run command, %%PWD will replaced with cwd
#       background: false
#       close_after_command: true
#       border: none                  # Select popup border type, optional
#       session: false                # Run commmand in new session. Usefull for long running command.
#       inputs:
#         - KEY                       # This input will be replaced with '%%KEY%%' on command
#       position:
#         x: ...
#         y: ...
#         w: ...
#         h: ...
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
