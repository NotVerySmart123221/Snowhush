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

Requires Rust (`cargo`) to build; `quickshell` (`qs`) is optional and only needed for the settings GUI.

```bash
./wayland/install.sh --apply-binds
```

Installs `snowhush-wayland`, `snowhush-toggle`, `snowhush-settings`, `~/.config/quickshell/snowhush/shell.qml`, and a `.desktop` entry.

| Flag | Description |
| --- | --- |
| `--apply-binds` | Append marked keybinds & window rules to detected WM config (`hyprland.conf`, `sway/config`, `niri/config.kdl`, `river/init`) |
| `--no-build` | Install using an already built release binary |
| `--prefix <DIR>` | Custom install prefix (default `~/.local`) |
| `--uninstall` | Stop daemon and remove installed files + applied WM binds |
| `--purge` | With `--uninstall`, also delete `~/.config/snowhush` |

Manual compositor config snippets: [wayland/README.md](wayland/README.md).

## Keybinds & Controls

### Default WM Keybinds
- `Alt+S` — toggle snowfall daemon (`snowhush-toggle`)
- `Ctrl+Alt+S` — open settings GUI (`snowhush-settings`)

### Daemon Script (`snowhush-toggle`)
```bash
snowhush-toggle [toggle|start|stop|restart]
```

### Settings GUI (`snowhush-settings`)
- `Ctrl+F` — search across all settings
- `Ctrl+1` .. `Ctrl+5` — switch sections
- `Esc` — clear search or close window
- **Double-click / Right-click** any slider or value badge — reset that parameter to default
- **Mouse wheel** on any slider — step by one increment
- **Reset** button — two-click confirmation to restore all defaults

## Config & CLI

Live hot-reloaded config: `~/.config/snowhush/config.json` (changes apply immediately without restarting the daemon).

Common `snowhush-wayland` flags (run `snowhush-wayland --help` for the full list):
- `-s, --snowfall <off|light|steady|heavy|variable>` — base snowfall preset
- `--static-wind <PX/S>` — ambient horizontal wind (`-300..+300`)
- `--wind-factor <FLOAT>` — dynamic breeze gusts (`0` = steady)
- `--wind-threshold <PX/S>` / `--wind-erosion-rate <FLOAT>` — peak scouring threshold & speed
- `--motion-shed <FLOAT>` / `--shed-threshold <PX/S>` — own-ledge shedding force & speed threshold
- `--window-wake <FLOAT>` — airborne snow scatter around moving/opening windows
- `--melt-rate <FLOAT>` / `--no-melt` — natural snow thawing
- `--workspace-sweep <FLOAT>` / `--sweep-ground` — workspace transition sweep
- `--top-margin <PX>` / `--no-layers` / `--no-fullscreen-snow` — compositor surface rules
- `--dynamic-weather` / `--weather-mode <auto|calm|breeze|blizzard|squall|thaw>`
- `--max-snow <INT>` / `--max-effects <INT>` — particle caps

Custom bars/docks on any Wayland compositor can also export ledges to `/dev/shm/snowhush_ledges.json` (`[{"id":"...","x":0,"y":0,"width":300,"height":40,"radius":12,"visible":true}]`).

## License

[MIT](LICENSE)
