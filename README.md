# tmux-easy-menu

### Tmux easy configuration

![Alt Text](https://github.com/Ja-sonYun/tmux-easy-menu/blob/main/examples/example.gif?raw=true)

## Setup
```
cargo build
```
And run with
```
./target/debug/tmux-menu show --menu {path-to-your-config}
```


## Configuration
To see more actual config files, checkout `./examples` folder.
```yaml
# On tmux.conf, add below line.
#
# bind-key k run-shell "$HOME/tmux-menu/target/debug/tmux-menu show --menu $HOME/tmux-menu/examples/menu.yaml --working_dir #{pane_current_path}"
#                       ^~~ Or add binary to your path
# =============================
#
# title: "..."
#
# position:  <- Optional
#   x: ...
#   y: ...
#
# items:
#   - Seperate: {}
#
#   - NoDim:
#       name: "..."
#
#   - Menu:
#       name: "..."
#       shortcut: "..."
#       ------------------
#
#       next_menu: "..."
#
#        ... OR
#
#       command: "command %%KEY%%"
#       close_after_command: true  <- Optional
#       inputs:
#         - KEY  <- This replace %%KEY%% on command
#       position:  <- Optional
#         x: ...
#         y: ...
#         w: ...
#         h: ...
```
