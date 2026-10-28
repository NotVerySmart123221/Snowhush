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
