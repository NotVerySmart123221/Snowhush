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

#[derive(Clone, Copy, Debug)]
pub struct AppConfig {
    pub snowfall: Snowfall,
    pub top_margin: f32,
    pub track_layers: bool,
    pub sweep_ground: bool,
    pub melt: bool,
    pub melt_rate: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            snowfall: Snowfall::Steady,
            top_margin: 48.0,
            track_layers: true,
            sweep_ground: false,
            melt: true,
            melt_rate: 1.0,
        }
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
        simulation.thaws = config.melt;
        simulation.thaw_rate_multiplier = config.melt_rate;

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

        let w = self.width as f32;
        let h = self.height as f32;

        let tracked = self.tracker.poll(w, h);

        if let Some(dir) = tracked.workspace_changed {
            if self.config.sweep_ground {
                self.simulation.sweep_all(dir);
            } else {
                self.simulation.sweep_window_ledges(dir);
            }
        }

        for &id in &tracked.fullscreen_windows {
            self.simulation.burst_window_omnidirectional(id);
        }

        let parked = HashSet::new();
        let elsewhere = HashSet::new();

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
