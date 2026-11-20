# Snowhush for Wayland

Port of [Snowhush](https://snowhush.pages.dev) for Wayland compositors supporting `wlr-layer-shell`.

## Supported Compositors
Auto-detected at startup via compositor IPC environment variables:
- **Hyprland** (`HYPRLAND_INSTANCE_SIGNATURE`): native Unix socket IPC (`j/clients`, `j/monitors`, `j/layers`, `j/cursorpos`) with per-frame window tracking, `pinned > floating > tiled` occlusion, cursor repulsion, fullscreen burst, and workspace sweep.
- **Sway** (`SWAYSOCK`): native `i3-ipc` (`GET_TREE`) over Unix socket with zero subprocess overhead, output-local multi-monitor coordinates, `sticky > floating > tiled` occlusion, fullscreen detection, and workspace sweep.
- **Niri** (`NIRI_SOCKET`): native JSON-line IPC (`Workspaces`, `Windows`) over Unix socket with active workspace filtering, `floating > tiled` occlusion, fullscreen detection, and workspace sweep.
- **River / Wayfire / Labwc / generic `wlr-layer-shell`**: click-through overlay snowfall, lockscreen detection (`hyprlock`, `swaylock`, `waylock`, `gtklock`), and optional custom panel ledges via `/dev/shm/snowhush_ledges.json`.

## Quick Install

```bash
./wayland/install.sh
```

Builds the release binary and installs `snowhush-wayland`, `snowhush-toggle`, `snowhush-settings`, the Quickshell UI (`~/.config/quickshell/snowhush/shell.qml`), and a `.desktop` entry.

Options:
- `./wayland/install.sh --no-build` — install using an already built release binary
- `./wayland/install.sh --prefix /usr/local` — custom install prefix (default `~/.local`)
- `./wayland/install.sh --uninstall` — remove installed files (add `--purge` to also delete `~/.config/snowhush`)

## Compositor Keybinds & Rules

### Hyprland
```conf
layerrule = above_lock 1, match:namespace ^(snowhush)$
windowrule = float on, center on, size 780 620, match:class .*snowhush-settings.*
bindl = ALT, S, exec, snowhush-toggle
bindl = CTRL ALT, S, exec, snowhush-settings
```

### Sway
```conf
for_window [app_id="snowhush-settings"] floating enable, resize set 780 620, move position center
bindsym Mod1+s exec snowhush-toggle
bindsym Control+Mod1+s exec snowhush-settings
```

### Niri
```kdl
window-rule {
    match app-id="snowhush-settings"
    open-floating true
    default-column-width { fixed 780; }
    default-window-height { fixed 620; }
}
binds {
    Alt+S { spawn "snowhush-toggle"; }
    Ctrl+Alt+S { spawn "snowhush-settings"; }
}
```

### River
```sh
riverctl rule-add -app-id 'snowhush-settings' float
riverctl map normal Alt S spawn snowhush-toggle
riverctl map normal Control+Alt S spawn snowhush-settings
```
