<p align="center">
  <img src="site/assets/icon-512.png" width="128" height="128" alt="Snowhush icon">
</p>

<h1 align="center">Snowhush</h1>

<p align="center">It's snowing on your desktop.</p>

<p align="center">
  <a href="#linux--wayland"><b>Install for Linux (Wayland)</b></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/mbezruchko/Snowhush/releases/latest/download/Snowhush.dmg"><b>macOS</b></a>
  &nbsp;·&nbsp;
  <a href="https://github.com/mbezruchko/Snowhush/releases/latest/download/Snowhush-Setup.exe"><b>Windows</b></a>
  &nbsp;·&nbsp;
  <a href="https://snowhush.pages.dev">snowhush.pages.dev</a>
  <br>
  <sub>Linux (Wayland) · macOS 14+ · Windows 10/11 · free and open source</sub>
</p>

https://github.com/user-attachments/assets/291c3cd5-3563-4b86-8b66-dfd924f980fc

A lightweight desktop snowfall overlay for **Linux (Wayland)**, **macOS**, and **Windows**. Soft snow drifts across your screen, settles along the tops of windows and panels, levels out in light wind, scours in strong gusts, and reacts to your cursor and window motion.

## What the snow does

- **Falls.** Every flake keeps its own pace and sways in the breeze, with optional dynamic weather cycles (`Calm`, `Breeze`, `Blizzard`, `Squall`, `Thaw`, or `Auto`).
- **Settles.** Snow piles up with realistic area-matched deposition along the bottom of the screen, window tops, and bars/docks, respecting window stacking order (`pinned > floating > tiled`).
- **Reacts to wind.** Light wind gently levels bumps into even drifts; strong wind above the erosion threshold peels peaks and carries snow downwind as real grains.
- **Reacts to windows & cursor.** Independently tuned controls for shedding snow off a moving window's own ledge, plowing settled drifts on the ground/other windows, and scattering airborne snowflakes when windows move or open.
- **Drops & sweeps.** Closing a window drops its snow in bursting lumps; switching workspaces sweeps drifts across the screen; fullscreen apps can pause accumulation.
- **Melts.** Adjustable thaw rate keeps snowdrifts balanced over time (set to `0` to disable).

## Linux / Wayland

Hardware-accelerated (`wgpu` / Vulkan) click-through `wlr-layer-shell` overlay with native IPC trackers:

- **Hyprland** (`HYPRLAND_INSTANCE_SIGNATURE`): per-frame Unix socket tracking (`j/clients`, `j/monitors`, `j/layers`, `j/cursorpos`), z-order occlusion, cursor repulsion, fullscreen burst, workspace sweep.
- **Sway** (`SWAYSOCK`): native `i3-ipc` (`GET_TREE`) over Unix socket with multi-monitor output offset, `sticky > floating > tiled` occlusion, fullscreen detection, and workspace sweep.
- **Niri** (`NIRI_SOCKET`): native JSON-line IPC (`Workspaces`, `Windows`) with active workspace filtering, `floating > tiled` occlusion, and workspace sweep.
- **River / Wayfire / Labwc / generic `wlr-layer-shell`**: overlay snowfall, lockscreen detection (`hyprlock`, `swaylock`, `waylock`, `gtklock`), and optional custom ledges via `/dev/shm/snowhush_ledges.json`.

### Quick Install

```bash
./wayland/install.sh
```

Builds the release binary and installs `snowhush-wayland`, `snowhush-toggle`, `snowhush-settings`, the **Quickshell** settings UI (`~/.config/quickshell/snowhush/shell.qml`), and a `.desktop` entry.

- `./wayland/install.sh --no-build` — install using an already built release binary
- `./wayland/install.sh --prefix /usr/local` — custom install prefix (default `~/.local`)
- `./wayland/install.sh --uninstall` — remove installed files (add `--purge` to also remove `~/.config/snowhush`)

See [wayland/README.md](wayland/README.md) for Hyprland, Sway, Niri, and River keybind & window-rule snippets.

## Building (macOS & Windows)

```sh
./build.sh            # build/Snowhush.app, signed ad hoc
open build/Snowhush.app
windows/build.sh      # dist/Snowhush-Setup.exe, cross-compiled from macOS
```

## License

[MIT](LICENSE)
