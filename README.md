# Snowhush (Wayland Fork)

Interactive desktop snowfall overlay for Linux Wayland compositors (`wgpu` / Vulkan + `wlr-layer-shell`) with a Quickshell settings GUI.

Fork of [mbezruchko/Snowhush](https://github.com/mbezruchko/Snowhush) ([snowhush.pages.dev](https://snowhush.pages.dev) · [macOS](https://github.com/mbezruchko/Snowhush/releases/latest/download/Snowhush.dmg) · [Windows](https://github.com/mbezruchko/Snowhush/releases/latest/download/Snowhush-Setup.exe)).

## Supported Compositors

- **Hyprland** (`HYPRLAND_INSTANCE_SIGNATURE`): native Unix socket IPC (`j/clients`, `j/monitors`, `j/layers`, `j/cursorpos`), `pinned > floating > tiled` occlusion, cursor repulsion, fullscreen burst, workspace sweep.
- **Sway** (`SWAYSOCK`): native `i3-ipc` (`GET_TREE`), multi-monitor offset, `sticky > floating > tiled` occlusion, fullscreen detection, workspace sweep.
- **Niri** (`NIRI_SOCKET`): native JSON-line IPC (`Workspaces`, `Windows`), active workspace filtering, `floating > tiled` occlusion, workspace sweep.
- **River / Wayfire / Labwc / generic `wlr-layer-shell`**: click-through overlay snowfall, lockscreen detection (`hyprlock`, `swaylock`, `waylock`, `gtklock`), custom panel ledges via `/dev/shm/snowhush_ledges.json`.

## Features

- Area-matched snow deposition on screen bottom, window tops, and panels/docks.
- Two-regime wind physics: light wind levels snowdrifts; strong wind above threshold scours peaks and carries grains downwind.
- Independent window-motion controls: ledge shedding (`motion_shed` + `shed_threshold`), settled drift plowing (`window_plow`), and airborne snowflake scatter (`window_wake`) on move and window open.
- Optional dynamic weather cycles (`Auto`, `Calm`, `Breeze`, `Blizzard`, `Squall`, `Thaw`), natural thaw, and Quickshell settings UI (`snowhush-settings`).

## Install

```bash
./wayland/install.sh
```

Installs `snowhush-wayland`, `snowhush-toggle`, `snowhush-settings`, `~/.config/quickshell/snowhush/shell.qml`, and a `.desktop` entry.

- `./wayland/install.sh --no-build` — install using existing release binary
- `./wayland/install.sh --prefix /usr/local` — custom prefix (default `~/.local`)
- `./wayland/install.sh --uninstall` — remove installed files (`--purge` also removes `~/.config/snowhush`)

Compositor keybind & window-rule examples: [wayland/README.md](wayland/README.md).

## License

[MIT](LICENSE)
