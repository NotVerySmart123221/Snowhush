use serde::Deserialize;
use snowhush::geometry::{Rect, V2};
use snowhush::simulation::WindowSurface;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct TrackedState {
    pub surfaces: Vec<WindowSurface>,
    pub cursor: Option<V2>,
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
}

#[derive(Deserialize)]
struct HyprMonitor {
    #[allow(dead_code)]
    id: i64,
    #[serde(rename = "activeWorkspace")]
    active_workspace: HyprWorkspace,
    #[serde(rename = "specialWorkspace")]
    special_workspace: Option<HyprWorkspace>,
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
}

#[derive(Deserialize)]
struct HyprCursor {
    x: f32,
    y: f32,
}

impl HyprlandTracker {
    pub fn new() -> Self {
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
    fn poll(&mut self, _screen_width: f32, screen_height: f32) -> TrackedState {
        let mut state = TrackedState::default();

        // Cursor position - fast socket query
        if let Some(cursor) = self
            .query("j/cursorpos", &["cursorpos", "-j"])
            .and_then(|s| serde_json::from_str::<HyprCursor>(&s).ok())
        {
            state.cursor = Some(V2::new(cursor.x, screen_height - cursor.y));
        }

        let now = Instant::now();
        if now.duration_since(self.last_surfaces_poll) >= Duration::from_millis(100) || self.cached_surfaces.is_empty() {
            self.last_surfaces_poll = now;
            let mut surfaces = Vec::new();

            // Active workspace
            let mut active_workspaces = Vec::new();
            if let Some(monitors) = self
                .query("j/monitors", &["monitors", "-j"])
                .and_then(|s| serde_json::from_str::<Vec<HyprMonitor>>(&s).ok())
            {
                for m in monitors {
                    if let Some(sw) = m.special_workspace
                        && sw.id > 0
                    {
                        active_workspaces.push(sw.id);
                    }
                    active_workspaces.push(m.active_workspace.id);
                }
            }

            // Window list
            if let Some(mut clients) = self
                .query("j/clients", &["clients", "-j"])
                .and_then(|s| serde_json::from_str::<Vec<HyprClient>>(&s).ok())
            {
                // Sort by focus_history_id: lower id means focused more recently (on top)
                clients.sort_by_key(|c| c.focus_history_id);

                    let mut frames_on_top: Vec<Rect> = Vec::new();

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

                        let room_above = win_y >= 6.0 && frame.max_y() < screen_height;
                        let on_screen = frame.max_x() > 0.0 && frame.min_x() < _screen_width && frame.max_y() > 0.0;
                        let fills_screen = frame.width * frame.height >= _screen_width * screen_height * 0.9;

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

pub struct NiriTracker;

impl NiriTracker {
    pub fn new() -> Self {
        Self
    }
}

impl CompositorTracker for NiriTracker {
    fn poll(&mut self, _screen_width: f32, _screen_height: f32) -> TrackedState {
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
                    let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0) as isize;
                    state.surfaces.push(WindowSurface {
                        id,
                        frame: Rect {
                            x: x as f32,
                            y: _screen_height - (y as f32 + h as f32),
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
        state
    }
}

pub struct FallbackTracker;

impl CompositorTracker for FallbackTracker {
    fn poll(&mut self, _screen_width: f32, _screen_height: f32) -> TrackedState {
        TrackedState::default()
    }
}

pub fn create_tracker() -> Box<dyn CompositorTracker> {
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_ok() || HyprlandTracker::new().instance_sig.is_some() {
        Box::new(HyprlandTracker::new())
    } else if std::env::var("NIRI_SOCKET").is_ok() {
        Box::new(NiriTracker::new())
    } else {
        Box::new(FallbackTracker)
    }
}
