use serde::Deserialize;
use snowhush::geometry::{Rect, V2};
use snowhush::simulation::WindowSurface;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub struct TrackerConfig {
    pub top_margin: f32,
    pub track_layers: bool,
}

impl Default for TrackerConfig {
    fn default() -> Self {
        Self {
            top_margin: 48.0,
            track_layers: true,
        }
    }
}

#[derive(Default)]
pub struct TrackedState {
    pub surfaces: Vec<WindowSurface>,
    pub cursor: Option<V2>,
    pub workspace_changed: Option<f32>,
    pub fullscreen_windows: Vec<isize>,
    #[allow(dead_code)]
    pub is_idle: bool,
}

pub trait CompositorTracker {
    fn poll(&mut self, screen_width: f32, screen_height: f32) -> TrackedState;
}

pub struct HyprlandTracker {
    instance_sig: Option<String>,
    socket_path: Option<PathBuf>,
    cached_surfaces: Vec<WindowSurface>,
    last_surfaces_poll: Instant,
    config: TrackerConfig,
    previous_workspace: Option<i64>,
    previous_fullscreen: HashSet<isize>,
}

#[derive(Deserialize)]
struct HyprMonitor {
    #[allow(dead_code)]
    id: i64,
    #[serde(rename = "activeWorkspace")]
    active_workspace: HyprWorkspace,
    #[serde(rename = "specialWorkspace")]
    special_workspace: Option<HyprWorkspace>,
    #[serde(default)]
    focused: bool,
}

#[derive(Deserialize)]
struct HyprWorkspace {
    id: i64,
}

#[derive(Deserialize)]
struct HyprClient {
    address: String,
    mapped: bool,
    hidden: bool,
    at: [i32; 2],
    size: [i32; 2],
    workspace: HyprWorkspace,
    #[allow(dead_code)]
    floating: bool,
    #[serde(rename = "focusHistoryID", default)]
    focus_history_id: i64,
    #[serde(default)]
    fullscreen: i32,
}

#[derive(Deserialize)]
struct HyprCursor {
    x: f32,
    y: f32,
}

#[derive(Deserialize)]
struct HyprLayerItem {
    address: String,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    #[serde(default)]
    namespace: Option<String>,
}

#[derive(Deserialize)]
struct HyprMonitorLayers {
    levels: HashMap<String, Vec<HyprLayerItem>>,
}

impl HyprlandTracker {
    pub fn new(config: TrackerConfig) -> Self {
        let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok().or_else(|| {
            let uid = nix_uid();
            let hypr_dir = format!("/run/user/{}/hypr", uid);
            if let Ok(entries) = fs::read_dir(&hypr_dir) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let name = file_name.to_string_lossy();
                    if name.contains('_') && entry.path().is_dir() {
                        return Some(name.to_string());
                    }
                }
            }
            None
        });

        let socket_path = sig.as_ref().and_then(|s| {
            let uid = nix_uid();
            let p = PathBuf::from(format!("/run/user/{}/hypr/{}/.socket.sock", uid, s));
            if p.exists() {
                Some(p)
            } else {
                None
            }
        });

        Self {
            instance_sig: sig,
            socket_path,
            cached_surfaces: Vec::new(),
            last_surfaces_poll: Instant::now() - Duration::from_secs(10),
            config,
            previous_workspace: None,
            previous_fullscreen: HashSet::new(),
        }
    }

    fn query_socket(&self, query: &str) -> Option<String> {
        if let Some(path) = &self.socket_path {
            use std::io::{Read, Write};
            use std::os::unix::net::UnixStream;
            if let Ok(mut stream) = UnixStream::connect(path) {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(50)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(50)));
                if stream.write_all(query.as_bytes()).is_ok() {
                    let mut buf = Vec::new();
                    if stream.read_to_end(&mut buf).is_ok() {
                        return String::from_utf8(buf).ok();
                    }
                }
            }
        }
        None
    }

    fn query(&self, socket_cmd: &str, cli_args: &[&str]) -> Option<String> {
        if let Some(res) = self.query_socket(socket_cmd) {
            return Some(res);
        }
        let mut cmd = Command::new("hyprctl");
        if let Some(sig) = &self.instance_sig {
            cmd.env("HYPRLAND_INSTANCE_SIGNATURE", sig);
        }
        cmd.args(cli_args);
        let output = cmd.output().ok()?;
        if output.status.success() {
            String::from_utf8(output.stdout).ok()
        } else {
            None
        }
    }
}

fn nix_uid() -> u32 {
    unsafe { libc::getuid() }
}

impl CompositorTracker for HyprlandTracker {
    fn poll(&mut self, screen_width: f32, screen_height: f32) -> TrackedState {
        let mut state = TrackedState::default();

        // Cursor position - fast socket query
        if let Some(cursor) = self
            .query("j/cursorpos", &["cursorpos", "-j"])
            .and_then(|s| serde_json::from_str::<HyprCursor>(&s).ok())
        {
            state.cursor = Some(V2::new(cursor.x, screen_height - cursor.y));
        }

        let now = Instant::now();
        if now.duration_since(self.last_surfaces_poll) >= Duration::from_millis(50) || self.cached_surfaces.is_empty() {
            self.last_surfaces_poll = now;
            let mut surfaces = Vec::new();
            let mut frames_on_top: Vec<Rect> = Vec::new();

            // Active workspace & workspace transition detection
            let mut active_workspaces = Vec::new();
            if let Some(monitors) = self
                .query("j/monitors", &["monitors", "-j"])
                .and_then(|s| serde_json::from_str::<Vec<HyprMonitor>>(&s).ok())
            {
                if let Some(focused_mon) = monitors.iter().find(|m| m.focused).or_else(|| monitors.first()) {
                    let active_ws = focused_mon.active_workspace.id;
                    if let Some(prev) = self.previous_workspace
                        && prev != active_ws
                    {
                        let dir = if active_ws > prev { 1.0 } else { -1.0 };
                        state.workspace_changed = Some(dir);
                    }
                    self.previous_workspace = Some(active_ws);
                }

                for m in monitors {
                    if let Some(sw) = m.special_workspace
                        && sw.id > 0
                    {
                        active_workspaces.push(sw.id);
                    }
                    active_workspaces.push(m.active_workspace.id);
                }
            }

            // Layer surfaces (Quickshell, panels, docks)
            if self.config.track_layers
                && let Some(layers_map) = self
                    .query("j/layers", &["layers", "-j"])
                    .and_then(|s| serde_json::from_str::<HashMap<String, HyprMonitorLayers>>(&s).ok())
            {
                for (_mon, mon_layers) in layers_map {
                    // Check overlay ("3"), then top ("2"), then bottom ("1")
                    for level_key in &["3", "2", "1"] {
                        if let Some(items) = mon_layers.levels.get(*level_key) {
                            for item in items {
                                if let Some(ref ns) = item.namespace
                                    && ns == "snowhush"
                                {
                                    continue;
                                }
                                if item.w <= 10 || item.h <= 10 {
                                    continue;
                                }

                                let win_x = item.x as f32;
                                let win_y = item.y as f32;
                                let win_w = item.w as f32;
                                let win_h = item.h as f32;

                                let frame = Rect {
                                    x: win_x,
                                    y: screen_height - (win_y + win_h),
                                    width: win_w,
                                    height: win_h,
                                };

                                let room_above = win_y >= self.config.top_margin && frame.max_y() < screen_height;
                                let on_screen = frame.max_x() > 0.0 && frame.min_x() < screen_width && frame.max_y() > 0.0;
                                let fills_screen = frame.width * frame.height >= screen_width * screen_height * 0.9;

                                let mut hasher = DefaultHasher::new();
                                item.address.hash(&mut hasher);
                                let id = hasher.finish() as isize;

                                if room_above && on_screen && !fills_screen && frame.width >= 40.0 && frame.height >= 20.0 {
                                    let radius = 8.0f32.min(frame.width / 2.0).min(frame.height / 2.0);
                                    let region = Rect {
                                        x: frame.x,
                                        y: frame.max_y() - radius,
                                        width: frame.width,
                                        height: 30.0 + radius,
                                    };
                                    let blockers = frames_on_top
                                        .iter()
                                        .filter(|b| intersects(b, &region))
                                        .copied()
                                        .collect();

                                    surfaces.push(WindowSurface {
                                        id,
                                        frame,
                                        blockers,
                                        corner_radius: radius,
                                        corner_exponent: 2.0,
                                    });
                                }

                                if !fills_screen {
                                    frames_on_top.push(frame);
                                }
                            }
                        }
                    }
                }
            }

            // Window list
            if let Some(mut clients) = self
                .query("j/clients", &["clients", "-j"])
                .and_then(|s| serde_json::from_str::<Vec<HyprClient>>(&s).ok())
            {
                // Fullscreen detection
                let mut current_fullscreen = HashSet::new();
                for client in &clients {
                    let mut hasher = DefaultHasher::new();
                    client.address.hash(&mut hasher);
                    let id = hasher.finish() as isize;

                    if client.fullscreen > 0 {
                        current_fullscreen.insert(id);
                        if !self.previous_fullscreen.contains(&id) {
                            state.fullscreen_windows.push(id);
                        }
                    }
                }
                self.previous_fullscreen = current_fullscreen;

                // Sort by focus_history_id: lower id means focused more recently (on top)
                clients.sort_by_key(|c| c.focus_history_id);

                for client in clients {
                    if !client.mapped || client.hidden {
                        continue;
                    }
                    if !active_workspaces.is_empty() && !active_workspaces.contains(&client.workspace.id) {
                        continue;
                    }
                    if client.size[0] <= 10 || client.size[1] <= 10 {
                        continue;
                    }

                    // Convert to bottom-up coordinates
                    let win_x = client.at[0] as f32;
                    let win_y = client.at[1] as f32;
                    let win_w = client.size[0] as f32;
                    let win_h = client.size[1] as f32;

                    let frame = Rect {
                        x: win_x,
                        y: screen_height - (win_y + win_h),
                        width: win_w,
                        height: win_h,
                    };

                    let room_above = win_y >= self.config.top_margin && frame.max_y() < screen_height;
                    let on_screen = frame.max_x() > 0.0 && frame.min_x() < screen_width && frame.max_y() > 0.0;
                    let fills_screen = frame.width * frame.height >= screen_width * screen_height * 0.9;

                    let mut hasher = DefaultHasher::new();
                    client.address.hash(&mut hasher);
                    let id = hasher.finish() as isize;

                    if room_above && on_screen && !fills_screen && frame.width >= 80.0 && frame.height >= 40.0 {
                        let radius = 12.0f32.min(frame.width / 2.0).min(frame.height / 2.0);
                        let region = Rect {
                            x: frame.x,
                            y: frame.max_y() - radius,
                            width: frame.width,
                            height: 40.0 + radius,
                        };
                        let blockers = frames_on_top
                            .iter()
                            .filter(|b| intersects(b, &region))
                            .copied()
                            .collect();

                        surfaces.push(WindowSurface {
                            id,
                            frame,
                            blockers,
                            corner_radius: radius,
                            corner_exponent: 2.0,
                        });
                    }

                    if !fills_screen {
                        frames_on_top.push(frame);
                    }
                }
            }
            self.cached_surfaces = surfaces;
        }

        state.surfaces = self.cached_surfaces.clone();
        state
    }
}

fn intersects(a: &Rect, b: &Rect) -> bool {
    a.min_x() < b.max_x() && b.min_x() < a.max_x() && a.min_y() < b.max_y() && b.min_y() < a.max_y()
}

pub struct NiriTracker {
    config: TrackerConfig,
}

impl NiriTracker {
    pub fn new(config: TrackerConfig) -> Self {
        Self { config }
    }
}

impl CompositorTracker for NiriTracker {
    fn poll(&mut self, _screen_width: f32, screen_height: f32) -> TrackedState {
        // Niri window query via `niri msg -j windows`
        let mut state = TrackedState::default();
        if let Some(arr) = Command::new("niri")
            .args(["msg", "-j", "windows"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| serde_json::from_slice::<serde_json::Value>(&o.stdout).ok())
            .and_then(|val| val.as_array().cloned())
        {
            for item in arr {
                // If layout info available, parse rect
                if let (Some(x), Some(y), Some(w), Some(h)) = (
                    item.get("x").and_then(|v| v.as_f64()),
                    item.get("y").and_then(|v| v.as_f64()),
                    item.get("width").and_then(|v| v.as_f64()),
                    item.get("height").and_then(|v| v.as_f64()),
                ) {
                    let win_y = y as f32;
                    if win_y >= self.config.top_margin {
                        let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
                        state.surfaces.push(WindowSurface {
                            id,
                            frame: Rect {
                                x: x as f32,
                                y: screen_height - (win_y + h as f32),
                                width: w as f32,
                                height: h as f32,
                            },
                            blockers: Vec::new(),
                            corner_radius: 12.0,
                            corner_exponent: 2.0,
                        });
                    }
                }
            }
        }
        state
    }
}

pub struct FallbackTracker;

impl CompositorTracker for FallbackTracker {
    fn poll(&mut self, _screen_width: f32, _screen_height: f32) -> TrackedState {
        TrackedState::default()
    }
}

pub fn create_tracker(config: TrackerConfig) -> Box<dyn CompositorTracker> {
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() || HyprlandTracker::new(config).instance_sig.is_some() {
        Box::new(HyprlandTracker::new(config))
    } else if std::env::var("NIRI_SOCKET").is_ok() {
        Box::new(NiriTracker::new(config))
    } else {
        Box::new(FallbackTracker)
    }
}
