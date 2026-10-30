use serde::Deserialize;
use snowhush::geometry::{Rect, V2};
use snowhush::simulation::WindowSurface;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
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
    pub has_fullscreen: bool,
    pub static_surface_ids: HashSet<isize>,
    pub desktop_window_ids: HashSet<isize>,
    pub is_locked: bool,
    #[allow(dead_code)]
    pub is_idle: bool,
}

pub trait CompositorTracker {
    fn poll(&mut self, screen_width: f32, screen_height: f32) -> TrackedState;
    fn update_config(&mut self, _config: TrackerConfig) {}
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ShmLedge {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    #[serde(default = "default_shm_radius")]
    pub radius: f32,
    #[serde(default = "default_shm_visible")]
    pub visible: bool,
}

fn default_shm_radius() -> f32 {
    12.0
}

fn default_shm_visible() -> bool {
    true
}

#[derive(Clone, Debug)]
struct AnimatedSurface {
    id: isize,
    current_frame: Rect,
    target_frame: Rect,
    velocity: [f32; 4],
    corner_radius: f32,
    corner_exponent: f32,
    is_static: bool,
    is_shm: bool,
    focus_order: i64,
    last_seen: Instant,
    wants_removal: bool,
}

fn smooth_damp_val(current: &mut f32, velocity: &mut f32, target: f32, time: f32, dt: f32) {
    let omega = 2.0 / time.max(0.0001);
    let x = omega * dt;
    let exp = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = *current - target;
    let temp = (*velocity + change * omega) * dt;
    *velocity = (*velocity - temp * omega) * exp;
    *current = target + (change + temp) * exp;
}

fn smooth_damp_rect(current: &mut Rect, velocity: &mut [f32; 4], target: Rect, time: f32, dt: f32) {
    if (current.x - target.x).abs() > 1200.0 || (current.y - target.y).abs() > 1000.0 {
        *current = target;
        *velocity = [0.0; 4];
        return;
    }

    smooth_damp_val(&mut current.x, &mut velocity[0], target.x, time, dt);
    smooth_damp_val(&mut current.y, &mut velocity[1], target.y, time, dt);
    smooth_damp_val(&mut current.width, &mut velocity[2], target.width, time, dt);
    smooth_damp_val(&mut current.height, &mut velocity[3], target.height, time, dt);

    if (current.x - target.x).abs() < 0.25 && velocity[0].abs() < 1.0 {
        current.x = target.x;
        velocity[0] = 0.0;
    }
    if (current.y - target.y).abs() < 0.25 && velocity[1].abs() < 1.0 {
        current.y = target.y;
        velocity[1] = 0.0;
    }
    if (current.width - target.width).abs() < 0.25 && velocity[2].abs() < 1.0 {
        current.width = target.width;
        velocity[2] = 0.0;
    }
    if (current.height - target.height).abs() < 0.25 && velocity[3].abs() < 1.0 {
        current.height = target.height;
        velocity[3] = 0.0;
    }
}

pub struct HyprlandTracker {
    instance_sig: Option<String>,
    socket_path: Option<PathBuf>,
    animated_surfaces: HashMap<isize, AnimatedSurface>,
    hypr_targets: HashMap<isize, (Rect, f32, bool, bool, i64)>,
    desktop_window_ids: HashSet<isize>,
    last_surfaces_poll: Instant,
    last_frame_poll: Instant,
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
    #[serde(default)]
    pinned: bool,
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
            animated_surfaces: HashMap::new(),
            hypr_targets: HashMap::new(),
            desktop_window_ids: HashSet::new(),
            last_surfaces_poll: Instant::now() - Duration::from_secs(10),
            last_frame_poll: Instant::now(),
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

pub fn check_is_locked() -> bool {
    // 1. Check Serpantinum lock file in runtime dir or /tmp
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let lock_path = std::path::Path::new(&runtime_dir).join("serpantinum/lock/locked");
        if lock_path.exists() {
            return true;
        }
    } else {
        let uid = nix_uid();
        let lock_path = PathBuf::from(format!("/run/user/{}/serpantinum/lock/locked", uid));
        if lock_path.exists() {
            return true;
        }
    }
    if std::path::Path::new("/tmp/serpantinum/lock/locked").exists() {
        return true;
    }

    // 2. Throttle process scan to ~3 times per second (300ms)
    static LAST_CHECK_MS: AtomicU64 = AtomicU64::new(0);
    static IS_LOCKED_PROC: AtomicBool = AtomicBool::new(false);

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let last = LAST_CHECK_MS.load(Ordering::Relaxed);
    if now_ms.saturating_sub(last) > 300 {
        LAST_CHECK_MS.store(now_ms, Ordering::Relaxed);
        let my_pid = std::process::id();
        let mut proc_locked = false;
        if let Ok(entries) = fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                if let Some(s) = file_name.to_str()
                    && let Ok(pid) = s.parse::<u32>()
                {
                    if pid == my_pid {
                        continue;
                    }
                    let cmdline_path = entry.path().join("cmdline");
                    if let Ok(cmd) = fs::read(cmdline_path) {
                        let cmd_str = String::from_utf8_lossy(&cmd);
                        if cmd_str.contains("Lock.qml")
                            || cmd_str.contains("hyprlock")
                            || cmd_str.contains("swaylock")
                            || cmd_str.contains("waylock")
                            || cmd_str.contains("gtklock")
                        {
                            proc_locked = true;
                            break;
                        }
                    }
                }
            }
        }
        IS_LOCKED_PROC.store(proc_locked, Ordering::Relaxed);
        proc_locked
    } else {
        IS_LOCKED_PROC.load(Ordering::Relaxed)
    }
}

impl CompositorTracker for HyprlandTracker {
    fn poll(&mut self, screen_width: f32, screen_height: f32) -> TrackedState {
        let mut state = TrackedState::default();

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_poll).as_secs_f32().clamp(0.001, 0.05);
        self.last_frame_poll = now;

        // Cursor position - fast socket query
        if let Some(cursor) = self
            .query("j/cursorpos", &["cursorpos", "-j"])
            .and_then(|s| serde_json::from_str::<HyprCursor>(&s).ok())
        {
            state.cursor = Some(V2::new(cursor.x, screen_height - cursor.y));
        }

        let is_locked = check_is_locked();
        state.is_locked = is_locked;

        let should_poll_hypr = now.duration_since(self.last_surfaces_poll) >= Duration::from_millis(16) || self.hypr_targets.is_empty();
        if should_poll_hypr {
            self.last_surfaces_poll = now;

            // Target surfaces collection: id -> (target_frame, radius, is_static, is_shm, focus_order)
            let mut hypr: HashMap<isize, (Rect, f32, bool, bool, i64)> = HashMap::new();

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
                        let delta = (active_ws - prev) as f32;
                        state.workspace_changed = Some(delta);
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

            let mut desktop_window_ids = HashSet::new();

            // Layer surfaces fallback / additional (only when not locked)
            if !is_locked && self.config.track_layers
                && let Some(layers_map) = self
                    .query("j/layers", &["layers", "-j"])
                    .and_then(|s| serde_json::from_str::<HashMap<String, HyprMonitorLayers>>(&s).ok())
            {
                for (_mon, mon_layers) in layers_map {
                    for level_key in &["3", "2", "1"] {
                        if let Some(items) = mon_layers.levels.get(*level_key) {
                            for item in items {
                                let ns = item.namespace.as_deref().unwrap_or("");
                                if ns == "snowhush" || ns == "wallpaper-bg" {
                                    continue;
                                }
                                if item.w <= 10 || item.h <= 10 {
                                    continue;
                                }

                                let win_x = item.x as f32;
                                let win_y = item.y as f32;
                                let win_w = item.w as f32;
                                let win_h = item.h as f32;

                                if win_w >= screen_width * 0.98 && win_h >= screen_height * 0.98 {
                                    continue;
                                }

                                let eff_top = if self.config.top_margin > 0.0 {
                                    win_y.max(self.config.top_margin)
                                } else {
                                    win_y
                                };
                                let eff_bottom = win_y + win_h;

                                if eff_bottom <= eff_top + 4.0 {
                                    continue;
                                }

                                let eff_h = eff_bottom - eff_top;
                                let frame = Rect {
                                    x: win_x,
                                    y: screen_height - eff_bottom,
                                    width: win_w,
                                    height: eff_h,
                                };

                                let mut hasher = DefaultHasher::new();
                                item.address.hash(&mut hasher);
                                let id = hasher.finish() as isize;
                                hypr.insert(id, (frame, 12.0, true, false, -800));
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
                    desktop_window_ids.insert(id);

                    let is_visible_ws = client.pinned || active_workspaces.is_empty() || active_workspaces.contains(&client.workspace.id);
                    if client.fullscreen > 0 && is_visible_ws {
                        current_fullscreen.insert(id);
                        if !self.previous_fullscreen.contains(&id) {
                            state.fullscreen_windows.push(id);
                        }
                    }
                }
                self.previous_fullscreen = current_fullscreen;
                state.has_fullscreen = !self.previous_fullscreen.is_empty();

                if !is_locked {
                    clients.sort_by_key(|c| c.focus_history_id);

                    for client in clients {
                        if !client.mapped || client.hidden {
                            continue;
                        }
                        if !client.pinned && !active_workspaces.is_empty() && !active_workspaces.contains(&client.workspace.id) {
                            continue;
                        }
                        if client.fullscreen > 0 {
                            continue;
                        }
                        if client.size[0] <= 10 || client.size[1] <= 10 {
                            continue;
                        }

                        let win_x = client.at[0] as f32;
                        let win_y = client.at[1] as f32;
                        let win_w = client.size[0] as f32;
                        let win_h = client.size[1] as f32;

                        let eff_top = if self.config.top_margin > 0.0 {
                            win_y.max(self.config.top_margin)
                        } else {
                            win_y
                        };
                        let eff_bottom = win_y + win_h;

                        if eff_bottom <= eff_top + 4.0 {
                            continue;
                        }

                        let frame = Rect {
                            x: win_x,
                            y: screen_height - eff_bottom,
                            width: win_w,
                            height: eff_bottom - eff_top,
                        };

                        let mut hasher = DefaultHasher::new();
                        client.address.hash(&mut hasher);
                        let id = hasher.finish() as isize;

                        let radius = 12.0f32.min(win_w / 2.0).min(win_h / 2.0);
                        hypr.insert(id, (frame, radius, client.pinned, false, client.focus_history_id));
                    }
                }
            }

            self.hypr_targets = hypr;
            self.desktop_window_ids = desktop_window_ids;
        }

        state.desktop_window_ids = self.desktop_window_ids.clone();

        // Target surfaces collection: id -> (target_frame, radius, is_static, is_shm, focus_order)
        let mut targets: HashMap<isize, (Rect, f32, bool, bool, i64)> = self.hypr_targets.clone();

        // Check Quickshell explicit ledges via /dev/shm/snowhush_ledges.json EVERY FRAME (fastest response)
        if (self.config.track_layers || is_locked)
            && let Ok(content) = std::fs::read_to_string("/dev/shm/snowhush_ledges.json")
            && let Ok(shm_list) = serde_json::from_str::<Vec<ShmLedge>>(&content)
        {
            for ledge in shm_list {
                if ledge.id == "qs-lockscreen" {
                    if is_locked && ledge.visible && ledge.width > 20.0 && ledge.height > 20.0 {
                        let mut hasher = DefaultHasher::new();
                        "qs-shm-".hash(&mut hasher);
                        ledge.id.hash(&mut hasher);
                        let id = hasher.finish() as isize;

                        let frame = Rect {
                            x: ledge.x,
                            y: screen_height - (ledge.y + ledge.height),
                            width: ledge.width,
                            height: ledge.height,
                        };
                        targets.insert(id, (frame, ledge.radius, true, true, -1000));
                    }
                    continue;
                }

                // If locked, skip regular desktop layers
                if is_locked {
                    continue;
                }

                if ledge.width <= 10.0 || ledge.height <= 10.0 {
                    continue;
                }

                // Trim or exclude ledges within top_margin
                let eff_top = if self.config.top_margin > 0.0 {
                    ledge.y.max(self.config.top_margin)
                } else {
                    ledge.y
                };
                let eff_bottom = ledge.y + ledge.height;

                if eff_bottom <= eff_top + 4.0 {
                    continue;
                }

                let mut hasher = DefaultHasher::new();
                "qs-shm-".hash(&mut hasher);
                ledge.id.hash(&mut hasher);
                let id = hasher.finish() as isize;

                let eff_h = eff_bottom - eff_top;
                let target_y = if ledge.visible {
                    screen_height - eff_bottom
                } else {
                    // Slide off-screen when hidden
                    if ledge.y > screen_height / 2.0 {
                        -eff_h - 40.0
                    } else {
                        screen_height + 40.0
                    }
                };

                let frame = Rect {
                    x: ledge.x,
                    y: target_y,
                    width: ledge.width,
                    height: eff_h,
                };

                targets.insert(id, (frame, ledge.radius, true, true, -1000));
            }
        }

        // Sync targets into animated_surfaces map EVERY FRAME
        for (id, (target_frame, radius, is_static, is_shm, focus_order)) in &targets {
            if let Some(anim) = self.animated_surfaces.get_mut(id) {
                anim.target_frame = *target_frame;
                anim.corner_radius = *radius;
                anim.is_static = *is_static;
                anim.is_shm = *is_shm;
                anim.focus_order = *focus_order;
                anim.last_seen = now;
                anim.wants_removal = false;
            } else {
                self.animated_surfaces.insert(*id, AnimatedSurface {
                    id: *id,
                    current_frame: *target_frame,
                    target_frame: *target_frame,
                    velocity: [0.0; 4],
                    corner_radius: *radius,
                    corner_exponent: 2.0,
                    is_static: *is_static,
                    is_shm: *is_shm,
                    focus_order: *focus_order,
                    last_seen: now,
                    wants_removal: false,
                });
            }
        }

        self.animated_surfaces.retain(|id, _| targets.contains_key(id));

        // EVERY FRAME: Smooth animation interpolation
        let mut sorted_surfaces: Vec<&mut AnimatedSurface> = self.animated_surfaces.values_mut().collect();
        sorted_surfaces.sort_by_key(|s| s.focus_order);

        let mut frames_on_top: Vec<Rect> = Vec::new();
        let mut surfaces_out = Vec::new();
        let mut static_out = HashSet::new();

        for anim in sorted_surfaces {
            let time_constant = if anim.is_shm { 0.012 } else if anim.is_static { 0.05 } else { 0.04 };
            smooth_damp_rect(&mut anim.current_frame, &mut anim.velocity, anim.target_frame, time_constant, dt);

            let frame = anim.current_frame;
            let on_screen = frame.max_x() > 0.0 && frame.min_x() < screen_width && frame.max_y() > 0.0 && frame.min_y() < screen_height;

            if on_screen && frame.width >= 30.0 && frame.height >= 8.0 {
                let radius = anim.corner_radius.min(frame.width / 2.0).min(frame.height / 2.0);
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

                surfaces_out.push(WindowSurface {
                    id: anim.id,
                    frame,
                    blockers,
                    corner_radius: radius,
                    corner_exponent: anim.corner_exponent,
                });

                if anim.is_static {
                    static_out.insert(anim.id);
                }

                frames_on_top.push(frame);
            }
        }

        // Cleanup surfaces that are dead and finished animating
        self.animated_surfaces.retain(|_, anim| {
            if anim.wants_removal {
                let off_screen = anim.current_frame.max_y() <= 0.0 || anim.current_frame.min_y() >= screen_height;
                let old = now.duration_since(anim.last_seen) > Duration::from_secs(1);
                !off_screen && !old
            } else {
                true
            }
        });

        state.surfaces = surfaces_out;
        state.static_surface_ids = static_out;
        state.is_locked = check_is_locked();
        state
    }

    fn update_config(&mut self, config: TrackerConfig) {
        self.config = config;
        self.last_surfaces_poll = Instant::now() - Duration::from_secs(1);
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
                    let win_h = h as f32;
                    let eff_top = if self.config.top_margin > 0.0 {
                        win_y.max(self.config.top_margin)
                    } else {
                        win_y
                    };
                    let eff_bottom = win_y + win_h;
                    if eff_bottom > eff_top + 4.0 {
                        let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
                        state.surfaces.push(WindowSurface {
                            id,
                            frame: Rect {
                                x: x as f32,
                                y: screen_height - eff_bottom,
                                width: w as f32,
                                height: eff_bottom - eff_top,
                            },
                            blockers: Vec::new(),
                            corner_radius: 12.0,
                            corner_exponent: 2.0,
                        });
                    }
                }
            }
        }
        state.is_locked = check_is_locked();
        state
    }

    fn update_config(&mut self, config: TrackerConfig) {
        self.config = config;
    }
}

pub struct FallbackTracker;

impl CompositorTracker for FallbackTracker {
    fn poll(&mut self, _screen_width: f32, _screen_height: f32) -> TrackedState {
        TrackedState {
            is_locked: check_is_locked(),
            ..Default::default()
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smooth_damp_rect_movement() {
        let mut current = Rect { x: 100.0, y: 100.0, width: 400.0, height: 300.0 };
        let mut velocity = [0.0; 4];
        let target = Rect { x: 500.0, y: 200.0, width: 600.0, height: 400.0 };

        // Step by 16ms
        for _ in 0..10 {
            smooth_damp_rect(&mut current, &mut velocity, target, 0.12, 0.016);
            assert!(current.x > 100.0 && current.x < 500.0);
            assert!(velocity[0] > 0.0);
        }

        // After 40 more steps (~640ms total), should be at target
        for _ in 0..40 {
            smooth_damp_rect(&mut current, &mut velocity, target, 0.12, 0.016);
        }
        assert!((current.x - target.x).abs() < 0.5);
        assert!((current.y - target.y).abs() < 0.5);
        assert!((current.width - target.width).abs() < 0.5);
        assert!((current.height - target.height).abs() < 0.5);
    }

    #[test]
    fn test_shm_ledge_deserialization() {
        let json = r#"[
            {"id":"qs-topbar","x":4,"y":4,"width":1592,"height":40,"radius":12,"visible":true},
            {"id":"qs-dock","x":644,"y":920,"width":312,"height":68,"radius":16,"visible":false}
        ]"#;
        let ledges: Vec<ShmLedge> = serde_json::from_str(json).unwrap();
        assert_eq!(ledges.len(), 2);
        assert_eq!(ledges[0].id, "qs-topbar");
        assert!(ledges[0].visible);
        assert_eq!(ledges[1].id, "qs-dock");
        assert!(!ledges[1].visible);
    }

    #[test]
    fn test_top_margin_filtering_logic() {
        let top_margin = 48.0;

        // Top bar at y=4, height=40 (ends at 44 <= 48) -> should be excluded
        let bar_y = 4.0f32;
        let bar_h = 40.0f32;
        let eff_top = bar_y.max(top_margin);
        let eff_bottom = bar_y + bar_h;
        assert!(eff_bottom <= eff_top + 4.0);

        // Window at y=30, height=200 -> should be trimmed to start at 48
        let win_y = 30.0f32;
        let win_h = 200.0f32;
        let eff_top = win_y.max(top_margin);
        let eff_bottom = win_y + win_h;
        assert!(eff_bottom > eff_top + 4.0);
        assert_eq!(eff_top, 48.0);
        assert_eq!(eff_bottom - eff_top, 182.0);

        // Dock at bottom y=824, height=68 -> completely untouched
        let dock_y = 824.0f32;
        let dock_h = 68.0f32;
        let eff_top = dock_y.max(top_margin);
        let eff_bottom = dock_y + dock_h;
        assert!(eff_bottom > eff_top + 4.0);
        assert_eq!(eff_top, 824.0);
        assert_eq!(eff_bottom - eff_top, 68.0);
    }
}


