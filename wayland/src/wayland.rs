use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, Region},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
};
use snowhush::geometry::V2;
use snowhush::simulation::{SnowSimulation, Snowfall};
use std::collections::HashSet;
use std::ptr::NonNull;
use std::time::{Duration, Instant};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_seat, wl_surface},
    Connection, Proxy, QueueHandle,
};

use crate::render::{Ambient, Renderer};
use crate::tracker::{create_tracker, CompositorTracker, TrackerConfig};

#[derive(Clone, Debug, PartialEq)]
pub struct AppConfig {
    pub snowfall: Snowfall,
    pub density: f32,
    pub speed_factor: f32,
    pub sway_factor: f32,
    pub top_margin: f32,
    pub track_layers: bool,
    pub sweep_ground: bool,
    pub melt: bool,
    pub melt_rate: f32,
    pub wind_accel: bool,
    pub wind_factor: f32,
    pub static_wind: f32,
    pub window_leak: f32,
    pub motion_shed: f32,
    pub cursor_force: f32,
    pub cloud_density: f32,
    pub cloud_amount: f32,
    pub cloud_size: f32,
    pub spray_density: f32,
    pub spray_amount: f32,
    pub spray_size: f32,
    pub spray_force: f32,
    pub window_wake: f32,
    pub motion_blur: f32,
    pub no_fullscreen_snow: bool,
    pub grain_speed: f32,
    pub grain_gravity: f32,
    pub dynamic_weather: bool,
    pub weather_mode: String,
    pub weather_cycle_speed: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            snowfall: Snowfall::Steady,
            density: 1.0,
            speed_factor: 1.0,
            sway_factor: 1.0,
            top_margin: 48.0,
            track_layers: true,
            sweep_ground: false,
            melt: true,
            melt_rate: 1.0,
            wind_accel: true,
            wind_factor: 1.0,
            static_wind: 20.0,
            window_leak: 0.35,
            motion_shed: 1.0,
            cursor_force: 1.0,
            cloud_density: 1.0,
            cloud_amount: 1.0,
            cloud_size: 1.0,
            spray_density: 1.0,
            spray_amount: 1.0,
            spray_size: 1.0,
            spray_force: 1.0,
            window_wake: 1.0,
            motion_blur: 1.0,
            no_fullscreen_snow: false,
            grain_speed: 1.0,
            grain_gravity: 1.0,
            dynamic_weather: false,
            weather_mode: "auto".to_string(),
            weather_cycle_speed: 1.0,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct JsonConfig {
    pub snowfall: String,
    #[serde(default = "default_one")]
    pub density: f32,
    #[serde(default = "default_one")]
    pub speed_factor: f32,
    #[serde(default = "default_one")]
    pub sway_factor: f32,
    pub top_margin: f32,
    pub track_layers: bool,
    pub sweep_ground: bool,
    pub melt: bool,
    pub melt_rate: f32,
    #[serde(default = "default_true")]
    pub wind_accel: bool,
    #[serde(default = "default_one")]
    pub wind_factor: f32,
    #[serde(default = "default_static_wind")]
    pub static_wind: f32,
    #[serde(default = "default_window_leak")]
    pub window_leak: f32,
    #[serde(default = "default_one")]
    pub motion_shed: f32,
    #[serde(default = "default_one")]
    pub cursor_force: f32,
    #[serde(default = "default_one")]
    pub cloud_density: f32,
    #[serde(default = "default_one")]
    pub cloud_amount: f32,
    #[serde(default = "default_one")]
    pub cloud_size: f32,
    #[serde(default = "default_one")]
    pub spray_density: f32,
    #[serde(default = "default_one")]
    pub spray_amount: f32,
    #[serde(default = "default_one")]
    pub spray_size: f32,
    #[serde(default = "default_one")]
    pub spray_force: f32,
    #[serde(default = "default_one")]
    pub window_wake: f32,
    #[serde(default = "default_one")]
    pub motion_blur: f32,
    #[serde(default = "default_false")]
    pub no_fullscreen_snow: bool,
    #[serde(default = "default_one")]
    pub grain_speed: f32,
    #[serde(default = "default_one")]
    pub grain_gravity: f32,
    #[serde(default = "default_false")]
    pub dynamic_weather: bool,
    #[serde(default = "default_weather_mode")]
    pub weather_mode: String,
    #[serde(default = "default_one")]
    pub weather_cycle_speed: f32,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_one() -> f32 {
    1.0
}

fn default_weather_mode() -> String {
    "auto".to_string()
}

fn default_static_wind() -> f32 {
    20.0
}

fn default_window_leak() -> f32 {
    0.35
}

impl JsonConfig {
    pub fn from_app_config(cfg: &AppConfig) -> Self {
        Self {
            snowfall: match cfg.snowfall {
                Snowfall::Off => "off",
                Snowfall::Light => "light",
                Snowfall::Steady => "steady",
                Snowfall::Heavy => "heavy",
                Snowfall::Variable => "variable",
            }.into(),
            density: cfg.density,
            speed_factor: cfg.speed_factor,
            sway_factor: cfg.sway_factor,
            top_margin: cfg.top_margin,
            track_layers: cfg.track_layers,
            sweep_ground: cfg.sweep_ground,
            melt: cfg.melt,
            melt_rate: cfg.melt_rate,
            wind_accel: cfg.wind_accel,
            wind_factor: cfg.wind_factor,
            static_wind: cfg.static_wind,
            window_leak: cfg.window_leak,
            motion_shed: cfg.motion_shed,
            cursor_force: cfg.cursor_force,
            cloud_density: cfg.cloud_density,
            cloud_amount: cfg.cloud_amount,
            cloud_size: cfg.cloud_size,
            spray_density: cfg.spray_density,
            spray_amount: cfg.spray_amount,
            spray_size: cfg.spray_size,
            spray_force: cfg.spray_force,
            window_wake: cfg.window_wake,
            motion_blur: cfg.motion_blur,
            no_fullscreen_snow: cfg.no_fullscreen_snow,
            grain_speed: cfg.grain_speed,
            grain_gravity: cfg.grain_gravity,
            dynamic_weather: cfg.dynamic_weather,
            weather_mode: cfg.weather_mode.clone(),
            weather_cycle_speed: cfg.weather_cycle_speed,
        }
    }

    pub fn to_app_config(&self) -> AppConfig {
        let snowfall = match self.snowfall.to_lowercase().as_str() {
            "off" => Snowfall::Off,
            "light" => Snowfall::Light,
            "steady" => Snowfall::Steady,
            "heavy" => Snowfall::Heavy,
            "variable" => Snowfall::Variable,
            _ => Snowfall::Steady,
        };
        AppConfig {
            snowfall,
            density: self.density.clamp(0.0, 5.0),
            speed_factor: self.speed_factor.clamp(0.1, 5.0),
            sway_factor: self.sway_factor.clamp(0.0, 5.0),
            top_margin: self.top_margin,
            track_layers: self.track_layers,
            sweep_ground: self.sweep_ground,
            melt: self.melt,
            melt_rate: self.melt_rate,
            wind_accel: self.wind_accel,
            wind_factor: self.wind_factor,
            static_wind: self.static_wind,
            window_leak: self.window_leak,
            motion_shed: self.motion_shed,
            cursor_force: self.cursor_force,
            cloud_density: self.cloud_density.clamp(0.0, 5.0),
            cloud_amount: self.cloud_amount.clamp(0.0, 5.0),
            cloud_size: self.cloud_size.clamp(0.1, 5.0),
            spray_density: self.spray_density.clamp(0.0, 5.0),
            spray_amount: self.spray_amount.clamp(0.1, 5.0),
            spray_size: self.spray_size.clamp(0.1, 5.0),
            spray_force: self.spray_force.clamp(0.1, 5.0),
            window_wake: self.window_wake.clamp(0.0, 5.0),
            motion_blur: self.motion_blur.clamp(0.0, 5.0),
            no_fullscreen_snow: self.no_fullscreen_snow,
            grain_speed: self.grain_speed.clamp(0.1, 5.0),
            grain_gravity: self.grain_gravity.clamp(0.05, 5.0),
            dynamic_weather: self.dynamic_weather,
            weather_mode: self.weather_mode.clone(),
            weather_cycle_speed: self.weather_cycle_speed.clamp(0.1, 5.0),
        }
    }
}

pub fn config_path() -> std::path::PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home).join(".config/snowhush/config.json")
    } else {
        std::path::PathBuf::from("/tmp/snowhush_config.json")
    }
}

pub fn load_config_file() -> Option<(AppConfig, std::time::SystemTime)> {
    let p = config_path();
    if let Ok(meta) = std::fs::metadata(&p) {
        if let Ok(mtime) = meta.modified() {
            if let Ok(content) = std::fs::read_to_string(&p) {
                if let Ok(json_cfg) = serde_json::from_str::<JsonConfig>(&content) {
                    return Some((json_cfg.to_app_config(), mtime));
                }
            }
        }
    }
    None
}

pub fn save_config_file(cfg: &AppConfig) {
    let p = config_path();
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json_cfg = JsonConfig::from_app_config(cfg);
    if let Ok(s) = serde_json::to_string_pretty(&json_cfg) {
        let _ = std::fs::write(&p, s);
    }
}

pub struct SnowhushApp {
    pub registry_state: RegistryState,
    pub seat_state: SeatState,
    pub output_state: OutputState,
    #[allow(dead_code)]
    pub layer: LayerSurface,
    #[allow(dead_code)]
    pub input_region: Option<Region>,

    pub width: u32,
    pub height: u32,
    pub first_configure: bool,
    pub exit: bool,

    pub tracker: Box<dyn CompositorTracker>,
    pub simulation: SnowSimulation,
    pub renderer: Option<Renderer>,
    pub start_time: Instant,
    pub last_frame: Instant,
    pub frame_count: u64,
    pub config: AppConfig,
    pub last_config_mtime: Option<std::time::SystemTime>,
}

impl SnowhushApp {
    pub fn run(config: AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            let uid = unsafe { libc::getuid() };
            for i in 0..10 {
                let sock = format!("/run/user/{}/wayland-{}", uid, i);
                if std::path::Path::new(&sock).exists() {
                    unsafe {
                        std::env::set_var("WAYLAND_DISPLAY", format!("wayland-{}", i));
                    }
                    break;
                }
            }
        }

        let conn = Connection::connect_to_env()?;
        let (globals, mut event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let compositor_state =
            CompositorState::bind(&globals, &qh).map_err(|e| format!("wl_compositor failed: {:?}", e))?;
        let layer_shell =
            LayerShell::bind(&globals, &qh).map_err(|e| format!("layer_shell failed: {:?}", e))?;

        let surface = compositor_state.create_surface(&qh);
        let layer = layer_shell.create_layer_surface(
            &qh,
            surface,
            Layer::Overlay,
            Some("snowhush"),
            None,
        );

        // Fill entire monitor
        layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer.set_exclusive_zone(-1);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(0, 0);

        // Make completely click-through with empty input region
        let input_region = Region::new(&compositor_state).ok();
        if let Some(ref reg) = input_region {
            layer.set_input_region(Some(reg.wl_region()));
        }

        layer.commit();

        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(conn.backend().display_ptr() as *mut _).unwrap(),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(layer.wl_surface().id().as_ptr() as *mut _).unwrap(),
        ));

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface_wgpu = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle,
                raw_window_handle,
            })?
        };

        let renderer = pollster::block_on(Renderer::new(&instance, surface_wgpu, 1920, 1080))?;

        let tracker_config = TrackerConfig {
            top_margin: config.top_margin,
            track_layers: config.track_layers,
        };
        let tracker = create_tracker(tracker_config);
        let mut simulation = SnowSimulation::new(1920.0, 1080.0, config.snowfall);
        simulation.set_density(config.density);
        simulation.speed_factor = config.speed_factor;
        simulation.sway_factor = config.sway_factor;
        simulation.thaws = config.melt;
        simulation.thaw_rate_multiplier = config.melt_rate;
        simulation.wind_accel = config.wind_accel;
        simulation.wind_factor = config.wind_factor;
        simulation.static_wind = config.static_wind;
        simulation.window_leak = config.window_leak;
        simulation.motion_shed = config.motion_shed;
        simulation.cursor_force = config.cursor_force;
        simulation.cloud_density = config.cloud_density;
        simulation.cloud_amount = config.cloud_amount;
        simulation.cloud_size = config.cloud_size;
        simulation.spray_density = config.spray_density;
        simulation.spray_amount = config.spray_amount;
        simulation.spray_size = config.spray_size;
        simulation.spray_force = config.spray_force;
        simulation.window_wake = config.window_wake;
        simulation.motion_blur = config.motion_blur;
        simulation.no_fullscreen_snow = config.no_fullscreen_snow;
        simulation.grain_speed = config.grain_speed;
        simulation.grain_gravity = config.grain_gravity;
        simulation.configure_weather(config.dynamic_weather, &config.weather_mode, config.weather_cycle_speed);

        if load_config_file().is_none() {
            save_config_file(&config);
        }
        let last_config_mtime = std::fs::metadata(config_path()).ok().and_then(|m| m.modified().ok());

        let mut app = Self {
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),
            layer,
            input_region,
            width: 1920,
            height: 1080,
            first_configure: true,
            exit: false,
            tracker,
            simulation,
            renderer: Some(renderer),
            start_time: Instant::now(),
            last_frame: Instant::now(),
            frame_count: 0,
            config,
            last_config_mtime,
        };

        log::info!("Snowhush Wayland loop started");

        let target_frame_time = Duration::from_micros(16_666);

        while !app.exit {
            if app.first_configure {
                event_queue.blocking_dispatch(&mut app)?;
            } else {
                let frame_start = Instant::now();
                let _ = event_queue.dispatch_pending(&mut app);
                let _ = conn.flush();

                app.render_frame();

                let elapsed = frame_start.elapsed();
                if elapsed < target_frame_time {
                    std::thread::sleep(target_frame_time - elapsed);
                }
            }
        }

        Ok(())
    }

    pub fn render_frame(&mut self) {
        self.frame_count += 1;
        if self.frame_count == 1 || self.frame_count.is_multiple_of(60) {
            log::info!(
                "Snowhush frame {}: {}x{}, sprites: {}",
                self.frame_count,
                self.width,
                self.height,
                self.simulation.sprite_count()
            );
        }

        let now = Instant::now();
        let dt = (now - self.last_frame).as_secs_f32().min(0.05);
        self.last_frame = now;

        if self.frame_count.is_multiple_of(30) {
            if let Some((new_cfg, mtime)) = load_config_file() {
                if Some(mtime) != self.last_config_mtime {
                    self.last_config_mtime = Some(mtime);
                    if new_cfg.snowfall != self.config.snowfall {
                        self.simulation.set_snowfall(new_cfg.snowfall);
                    }
                    if (new_cfg.density - self.config.density).abs() > 0.001 {
                        self.simulation.set_density(new_cfg.density);
                    }
                    self.simulation.speed_factor = new_cfg.speed_factor;
                    self.simulation.sway_factor = new_cfg.sway_factor;
                    self.simulation.thaws = new_cfg.melt;
                    self.simulation.thaw_rate_multiplier = new_cfg.melt_rate;
                    self.simulation.wind_accel = new_cfg.wind_accel;
                    self.simulation.wind_factor = new_cfg.wind_factor;
                    self.simulation.static_wind = new_cfg.static_wind;
                    self.simulation.window_leak = new_cfg.window_leak;
                    self.simulation.motion_shed = new_cfg.motion_shed;
                    self.simulation.cursor_force = new_cfg.cursor_force;
                    self.simulation.cloud_density = new_cfg.cloud_density;
                    self.simulation.cloud_amount = new_cfg.cloud_amount;
                    self.simulation.cloud_size = new_cfg.cloud_size;
                    self.simulation.spray_density = new_cfg.spray_density;
                    self.simulation.spray_amount = new_cfg.spray_amount;
                    self.simulation.spray_size = new_cfg.spray_size;
                    self.simulation.spray_force = new_cfg.spray_force;
                    self.simulation.window_wake = new_cfg.window_wake;
                    self.simulation.motion_blur = new_cfg.motion_blur;
                    self.simulation.grain_speed = new_cfg.grain_speed;
                    self.simulation.grain_gravity = new_cfg.grain_gravity;
                    self.simulation.no_fullscreen_snow = new_cfg.no_fullscreen_snow;
                    self.simulation.configure_weather(new_cfg.dynamic_weather, &new_cfg.weather_mode, new_cfg.weather_cycle_speed);
                    self.tracker.update_config(TrackerConfig {
                        top_margin: new_cfg.top_margin,
                        track_layers: new_cfg.track_layers,
                    });
                    self.config = new_cfg;
                }
            }
        }

        let w = self.width as f32;
        let h = self.height as f32;

        let tracked = self.tracker.poll(w, h);

        if let Some(delta) = tracked.workspace_changed {
            let dir = if delta > 0.0 { 1.0 } else { -1.0 };
            let distance = delta.abs();
            if self.config.sweep_ground {
                self.simulation.sweep_all(dir, distance, &tracked.static_surface_ids);
            } else {
                self.simulation.sweep_window_ledges(dir, distance, &tracked.static_surface_ids);
            }
        }

        for &id in &tracked.fullscreen_windows {
            self.simulation.burst_window_omnidirectional(id);
        }

        if self.config.no_fullscreen_snow && tracked.has_fullscreen {
            self.simulation.accumulates = false;
        } else {
            self.simulation.accumulates = true;
        }

        let parked = HashSet::new();
        let mut elsewhere = HashSet::new();

        if tracked.is_locked {
            // While screen is locked, stash desktop windows into elsewhere (dormant),
            // so their snow does not hang in mid-air on the lock screen.
            for &id in &tracked.desktop_window_ids {
                elsewhere.insert(id);
            }
        }

        self.simulation.windows_solid = true;
        self.simulation.update_windows(&tracked.surfaces, dt, &parked, &elsewhere, false);

        let cursor = tracked.cursor.unwrap_or(V2::new(-1000.0, -1000.0));
        self.simulation.step(dt, cursor);

        if let Some(ref mut renderer) = self.renderer {
            let elapsed = self.start_time.elapsed().as_secs_f32();
            if let Err(e) = renderer.draw(&self.simulation, Ambient::default(), elapsed) {
                log::warn!("Render error: {e}");
            }
        }
    }
}

impl CompositorHandler for SnowhushApp {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for SnowhushApp {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl LayerShellHandler for SnowhushApp {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;
        let nw = if w == 0 { self.width } else { w };
        let nh = if h == 0 { self.height } else { h };

        let resized = nw != self.width || nh != self.height;
        self.width = nw;
        self.height = nh;
        if resized {
            self.simulation.resize(nw as f32, nh as f32);
            if let Some(ref mut renderer) = self.renderer {
                renderer.resize(nw, nh);
            }
        }

        if self.first_configure {
            self.first_configure = false;
        }
    }
}

impl SeatHandler for SnowhushApp {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _seat: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

delegate_registry!(SnowhushApp);

impl ProvidesRegistryState for SnowhushApp {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

smithay_client_toolkit::delegate_dispatch2!(SnowhushApp);
