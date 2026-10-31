# Snowhush for Wayland

Port of [Snowhush](https://snowhush.pages.dev) for Wayland compositors (Hyprland, Niri, and generic `wlr-layer-shell` compositors).

## Features
- Interactive snowfall physics and drifts settling on window tops.
- Real-time window tracking and cursor repulsion over Hyprland Unix socket (zero process spawn overhead).
- Fallback support for Niri and standard layer-shell compositors.
- 100% click-through transparent overlay on Wayland overlay layer.
- WGPU (Vulkan) hardware-accelerated rendering with sub-pixel sprite and pile shaders.

## Build and Run

```bash
cargo build --release --manifest-path wayland/Cargo.toml
./wayland/target/release/snowhush-wayland --snowfall steady
```

### Options
- `-s, --snowfall <RATE>`: Snowfall intensity: `off`, `light`, `steady`, `heavy`, `variable` (default: `steady`).

### Settings GUI & Controls
Snowhush includes a Quickshell-based GUI interface in `wayland/quickshell/shell.qml` and control scripts in `wayland/scripts/`:
- `wayland/scripts/snowhush-toggle`: Toggles the daemon on/off with rapid keypress debounce protection.
- `wayland/scripts/snowhush-settings`: Launches the floating settings control panel.
