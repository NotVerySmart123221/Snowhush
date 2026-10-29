use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::f32::consts::{PI, TAU};
use std::hash::{BuildHasher, Hasher};
use std::ops::Range;

use crate::geometry::{Rect, V2, smoothstep};
use crate::layer::SnowLayer;

pub type WindowId = isize;

pub const COLUMN_WIDTH: f32 = 2.0;
pub const SIDE_MARGIN: f32 = 400.0;
const GROUND_DEPTH: f32 = 40.0;
const LEDGE_DEPTH: f32 = 14.0;
const THAW_DEPTH: f32 = 8.0;
const LEDGE_THAW_DEPTH: f32 = 4.0;
const THAW_RATE: f32 = 0.02;
const THAW_TIME: f32 = 30.0;
const THAW_BALANCE: f32 = 184.0;
const THAW_EXPONENT: f32 = 0.74;
const DEPOSITION: f32 = 1.1;
const REPOSE: f32 = 0.8;
const ROUNDING: f32 = 0.15;
const CURSOR_RADIUS: f32 = 10.0;
const CURSOR_OFFSET: V2 = V2::new(4.0, -8.0);
const GRAVITY: f32 = 2500.0;
const GRAIN_DRAG: f32 = 0.8;
const MAX_GRAINS: usize = 12000;
const PHASE_WRAP: f32 = 20.0 * PI;
const LEDGE_INSET: f32 = 0.0;
const GRIP: f32 = 3.0;
const SLIP: f32 = 2.0;
const BREAK_LIFT: f32 = 10.0;
/// Snow sticks to a window: it only lets go when pulled away harder than this, on top of its weight.
const ADHESION: f32 = 3500.0;
const SHAKING: f32 = 4.0;
const JUMP_SPEED: f32 = 6000.0;
const TRACK_TIME: f32 = 0.06;
const MAX_MOTES: usize = 1500;
const GUST_SWEEP: f32 = 0.6;
const GUST_LENGTH: f32 = 2.2;
const GUST_WIND: f32 = 1000.0;
/// The snowfall is caught this long after the drifts, so it trails behind their powder.
const GUST_LAG: f32 = 0.3;
/// Snow a wall scrapes up packs down to this share of its fresh bulk.
const PACKING: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Snowfall {
    Off,
    Light,
    Steady,
    Heavy,
    Variable,
}

impl Snowfall {
    pub const ALL: [Snowfall; 5] = [Snowfall::Off, Snowfall::Light, Snowfall::Steady, Snowfall::Heavy, Snowfall::Variable];

    pub fn title(self) -> &'static str {
        match self {
            Snowfall::Off => "Off",
            Snowfall::Light => "Light",
            Snowfall::Steady => "Steady",
            Snowfall::Heavy => "Heavy",
            Snowfall::Variable => "Variable",
        }
    }

    fn flakes_per_megapoint(self) -> f32 {
        match self {
            Snowfall::Off => 0.0,
            Snowfall::Light => 150.0,
            Snowfall::Steady => 380.0,
            Snowfall::Heavy => 800.0,
            Snowfall::Variable => 380.0,
        }
    }
}

/// How briskly the scene changes, which sets how often it is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    /// Only snowfall and slow drifts: a low frame rate is indistinguishable.
    Calm,
    /// Loose snow in flight.
    Lively,
    /// The cursor or a window is pushing snow around: drawn as often as the display allows.
    Interactive,
}

#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct Sprite {
    pub position: V2,
    /// Smears fast snow into a short streak, as a camera would.
    pub velocity: V2,
    pub size: f32,
    pub alpha: f32,
    pub soft: f32,
    /// How much of the tint shows instead of snow white.
    pub tint_amount: f32,
    pub tint: [f32; 3],
    /// Gives a lump of snow its ragged outline and tumble; zero for a round flake.
    pub lump: f32,
}

/// A bit of something that crumbles into snow, such as a window: it keeps its colour for a moment while it falls.
#[derive(Clone, Copy, Debug)]
pub struct ScatterPiece {
    pub position: V2,
    pub velocity: V2,
    pub color: [f32; 3],
}

pub struct PileLayer<'a> {
    pub offset: usize,
    pub count: usize,
    pub origin: V2,
    pub top: f32,
    pub bottom: f32,
    pub occluders: &'a [[f32; 4]],
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowSurface {
    pub id: WindowId,
    pub frame: Rect,
    /// Frames of windows in front of this one.
    pub blockers: Vec<Rect>,
    pub corner_radius: f32,
    pub corner_exponent: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WindowWake {
    pub frame: Rect,
    pub velocity: V2,
}

#[derive(Clone, Copy)]
struct Flake {
    position: V2,
    velocity: V2,
    fall_speed: f32,
    size: f32,
    depth: f32,
    alpha: f32,
    soft: f32,
    phase: f32,
    sway_rate: f32,
    sway_amount: f32,
    settles: bool,
    skipped_layer: usize,
    /// Scales the snow it leaves where it lands.
    mass: f32,
    tint: [f32; 3],
    tint_amount: f32,
    /// The direction of the gust that caught the flake and carries it off the screen; zero until then.
    blown: f32,
    /// True only for ambient snowfall that spawned from the sky and has not landed on any surface or come from an effect.
    can_wrap: bool,
}

#[derive(Clone, Copy)]
struct Grain {
    position: V2,
    velocity: V2,
    area: f32,
    size: f32,
    /// A lump holds together in flight and bursts into powder where it lands; zero for loose snow.
    lump: f32,
    /// Powder carried off by a gust: it rides the wind off the screen instead of settling.
    blown: bool,
}

/// Powder thrown up by a lump bursting: it billows, slows in the air and fades without settling.
#[derive(Clone, Copy)]
struct Mote {
    position: V2,
    velocity: V2,
    size: f32,
    alpha: f32,
    age: f32,
    life: f32,
}

struct Random(u64);

impl Random {
    fn new() -> Random {
        Random(RandomState::new().build_hasher().finish().max(1))
    }

    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u32 << 24) as f32
    }

    fn range(&mut self, lower: f32, upper: f32) -> f32 {
        lower + (upper - lower) * self.next()
    }
}

/// A gust clearing the snow: its front sweeps across from the windward side, lifting the drifts behind it.
#[derive(Clone, Copy)]
struct Gust {
    start: f64,
    direction: f32,
    /// Melts the snow in place instead, for Reduce Motion.
    calm: bool,
}

/// Everything but the layers, so a layer can be borrowed alongside it.
struct Air {
    flakes: Vec<Flake>,
    grains: Vec<Grain>,
    motes: Vec<Mote>,
    random: Random,
    clock: f64,
    strength: f32,
    static_wind: f32,
    window_leak: f32,
    pub motion_shed: f32,
    pub cursor_force: f32,
    pub speed_factor: f32,
    pub sway_factor: f32,
    cursor: Option<V2>,
    cursor_velocity: V2,
    held_layers: Vec<usize>,
    heights_changed: bool,
    windows_changed: bool,
    last_window_motion: f64,
    last_cursor_push: f64,
    /// Snow released while the air is full, laid straight onto the ground at the end of the step.
    overflow: Vec<(f32, f32)>,
    gust: Option<Gust>,
    pub workspace_wind: f32,
    pub workspace_wind_dir: f32,
    pub workspace_wind_start: f64,
    pub workspace_wind_until: f64,
}

pub struct SnowSimulation {
    width: f32,
    height: f32,
    pace: Pace,
    snowfall: Snowfall,
    density: f32,
    pub thaws: bool,
    pub thaw_rate_multiplier: f32,
    pub wind_accel: bool,
    pub wind_factor: f32,
    pub static_wind: f32,
    pub window_leak: f32,
    pub motion_shed: f32,
    pub cursor_force: f32,
    pub speed_factor: f32,
    pub sway_factor: f32,
    pub accumulates: bool,
    pub windows_solid: bool,
    /// Side edges, left and right, with no display past them: a window slid past one leaves its snow heaped against the edge.
    pub walls: (bool, bool),
    /// Keeps the snowfall from starting until a scattered window's snow first reaches the bottom of the screen.
    snowfall_held: bool,
    /// Set when the held snowfall starts by itself, so the other displays can start theirs.
    snowfall_released: bool,
    scattered_until: f64,
    /// The ground first, then one ledge per window, with `ids` alongside.
    layers: Vec<SnowLayer>,
    ids: Vec<WindowId>,
    /// Snow on windows that left the screen for another desktop, kept until the window shows up again.
    dormant: HashMap<WindowId, SnowLayer>,
    /// Windows that went off screen without a word yet on why; a desktop switch keeps their snow, anything else drops it.
    parking: Vec<(WindowId, SnowLayer, f64)>,
    window_frames: HashMap<WindowId, Rect>,
    pub active_wakes: Vec<WindowWake>,
    pub workspace_transition_until: f64,
    space_changed_at: f64,
    target_flakes: usize,
    strength_target: f32,
    next_shift: f64,
    air: Air,
}

impl SnowSimulation {
    pub fn new(width: f32, height: f32, snowfall: Snowfall) -> SnowSimulation {
        let mut seed = Random::new();
        let ground = SnowLayer::new(V2::ZERO, column_count(width), COLUMN_WIDTH, GROUND_DEPTH, true, (seed.range(0.0, TAU), seed.range(0.0, TAU)));
        let mut simulation = SnowSimulation {
            width,
            height,
            pace: Pace::Calm,
            snowfall,
            density: 1.0,
            thaws: false,
            thaw_rate_multiplier: 1.0,
            wind_accel: true,
            wind_factor: 1.0,
            static_wind: 20.0,
            window_leak: 0.35,
            motion_shed: 1.0,
            cursor_force: 1.0,
            speed_factor: 1.0,
            sway_factor: 1.0,
            accumulates: true,
            windows_solid: true,
            walls: (false, false),
            snowfall_held: false,
            snowfall_released: false,
            scattered_until: -10.0,
            layers: vec![ground],
            ids: vec![0],
            dormant: HashMap::new(),
            parking: Vec::new(),
            window_frames: HashMap::new(),
            active_wakes: Vec::new(),
            workspace_transition_until: -10.0,
            space_changed_at: -10.0,
            target_flakes: 0,
            strength_target: 1.0,
            next_shift: 0.0,
            air: Air {
                flakes: Vec::new(),
                grains: Vec::new(),
                motes: Vec::new(),
                random: Random::new(),
                clock: 0.0,
                strength: 1.0,
                static_wind: 20.0,
                window_leak: 0.35,
                motion_shed: 1.0,
            cursor_force: 1.0,
            speed_factor: 1.0,
            sway_factor: 1.0,
                cursor: None,
                cursor_velocity: V2::ZERO,
                held_layers: Vec::new(),
                heights_changed: false,
                windows_changed: false,
                last_window_motion: -10.0,
                last_cursor_push: -10.0,
                overflow: Vec::new(),
                gust: None,
                workspace_wind: 0.0,
                workspace_wind_dir: 0.0,
                workspace_wind_start: -10.0,
                workspace_wind_until: -10.0,
            },
        };
        simulation.update_target();
        simulation
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    #[cfg(test)]
    pub fn layers_mut(&mut self) -> &mut [SnowLayer] {
        &mut self.layers
    }

    #[cfg(test)]
    pub fn spawn_flake_for_test(&mut self, pos: V2, vel: V2, can_wrap: bool) {
        let mut flake = self.air.make_flake(self.width, pos.y);
        flake.position = pos;
        flake.velocity = vel;
        flake.can_wrap = can_wrap;
        self.air.flakes.push(flake);
    }

    pub fn pace(&self) -> Pace {
        self.pace
    }

    pub fn set_snowfall(&mut self, snowfall: Snowfall) {
        if snowfall == Snowfall::Variable && self.snowfall != Snowfall::Variable {
            self.next_shift = self.air.clock;
        }
        if snowfall != Snowfall::Variable {
            self.air.strength = 1.0;
            self.strength_target = 1.0;
        }
        self.snowfall = snowfall;
        self.update_target();
    }

    pub fn set_snowfall_held(&mut self, held: bool) {
        self.snowfall_held = held;
        self.update_target();
    }

    /// Whether the held snowfall started by itself since the last call.
    pub fn take_snowfall_released(&mut self) -> bool {
        std::mem::take(&mut self.snowfall_released)
    }

    /// Scales the snowfall; flakes come and go through the usual cycle, so a change shows gradually.
    pub fn set_density(&mut self, density: f32) {
        self.density = density;
        self.update_target();
    }

    pub fn has_snow(&self) -> bool {
        self.layers.iter().any(|layer| layer.top > 0.5)
    }

    pub fn sprite_count(&self) -> usize {
        self.air.flakes.len() + self.air.grains.len() + self.air.motes.len()
    }

    /// Floats the renderer needs per frame: each layer's surface and bed.
    pub fn column_count(&self) -> usize {
        self.layers.iter().map(|layer| 2 * layer.count()).sum()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if width == self.width && height == self.height {
            return;
        }
        self.layers[0].resize(column_count(width));
        self.width = width;
        self.height = height;
        self.update_target();
    }

    /// Returns whether anything visible changed.
    pub fn step(&mut self, dt: f32, cursor: V2) -> bool {
        self.air.clock += dt as f64;
        self.air.static_wind = self.static_wind;
        self.air.window_leak = self.window_leak;
        self.air.motion_shed = self.motion_shed;
        self.air.cursor_force = self.cursor_force;
        self.air.speed_factor = self.speed_factor;
        self.air.sway_factor = self.sway_factor;
        let was_moving = self.sprite_count() > 0;
        self.air.heights_changed = false;

        self.vary(dt);
        self.expire_parking();
        for ledge in &mut self.layers[1..] {
            self.air.carry(ledge, dt);
        }
        self.track_cursor(cursor, dt);
        for (li, ledge) in self.layers.iter_mut().enumerate().skip(1) {
            self.air.confine(ledge, li, self.width, self.walls);
        }
        self.blow(dt);
        self.erode_snowdrifts_by_wind(dt);
        self.advance_flakes(dt);
        self.advance_grains(dt);
        self.air.advance_motes(dt);
        self.settle(dt);
        self.spawn_flakes();
        for (area, x) in self.air.overflow.drain(..) {
            self.layers[0].deposit(area, x, true);
        }

        for layer in &mut self.layers {
            layer.refresh_top();
            layer.previous_base_y = layer.base().y;
        }
        let air = &self.air;
        // Only a cursor that actually displaces snow needs the full rate; one passing near it or resting in it changes nothing.
        let pushed = air.clock - air.last_window_motion < 0.6
            || air.clock - air.last_cursor_push < 0.3
            || air.gust.is_some()
            || air.clock < self.workspace_transition_until;
        // The steady trickle off window corners is a few tiny grains at a time; they read fine at the calm rate.
        let flying = air.grains.len() > 24 || air.clock < self.scattered_until;
        self.pace = if pushed {
            Pace::Interactive
        } else if flying {
            Pace::Lively
        } else {
            Pace::Calm
        };
        let changed = was_moving || self.sprite_count() > 0 || self.air.heights_changed || self.air.windows_changed;
        self.air.windows_changed = false;
        self.active_wakes.clear();
        changed
    }

    pub fn write_sprites(&self, buffer: &mut [Sprite]) -> usize {
        let flakes = self.air.flakes.iter().map(|flake| Sprite {
            position: flake.position,
            velocity: flake.velocity,
            size: flake.size,
            alpha: flake.alpha,
            soft: flake.soft,
            tint_amount: flake.tint_amount,
            tint: flake.tint,
            ..Sprite::default()
        });
        let grains = self.air.grains.iter().map(|grain| Sprite {
            position: grain.position,
            velocity: grain.velocity,
            size: grain.size,
            alpha: if grain.lump > 0.0 {
                0.97
            } else if grain.blown {
                0.8
            } else {
                0.95
            },
            soft: if grain.lump > 0.0 { 0.2 } else { 0.3 },
            tint: [1.0; 3],
            lump: grain.lump,
            ..Sprite::default()
        });
        let motes = self.air.motes.iter().map(|mote| {
            let t = mote.age / mote.life;
            let fade = (mote.age / 0.06).min(1.0) * (1.0 - t).powf(1.5);
            Sprite { position: mote.position, size: mote.size * (1.0 + t), alpha: mote.alpha * fade, soft: 1.0, tint: [1.0; 3], ..Sprite::default() }
        });
        let mut count = 0;
        for (slot, sprite) in buffer.iter_mut().zip(flakes.chain(grains).chain(motes)) {
            *slot = sprite;
            count += 1;
        }
        count
    }

    pub fn write_layers(&self, buffer: &mut [f32]) -> Vec<PileLayer<'_>> {
        let mut result = Vec::new();
        let mut offset = 0;
        for layer in &self.layers {
            let span = 2 * layer.count();
            if layer.top <= 0.05 || offset + span > buffer.len() {
                continue;
            }
            layer.write_smoothed(&mut buffer[offset..offset + span]);
            result.push(PileLayer {
                offset,
                count: layer.count(),
                origin: layer.base(),
                top: layer.top,
                bottom: -layer.bed_depth,
                occluders: &layer.occluders,
            });
            offset += span;
        }
        result
    }

    /// Turns the pieces into snowflakes that drift down and fade to white; together they leave a small drift.
    pub fn scatter(&mut self, pieces: &[ScatterPiece]) {
        self.scattered_until = self.air.clock + 8.0;
        for piece in pieces {
            let random = &mut self.air.random;
            let depth = random.next().powf(0.7);
            self.air.flakes.push(Flake {
                position: piece.position,
                velocity: piece.velocity,
                fall_speed: 70.0 + depth * 170.0 + random.range(0.0, 40.0),
                size: 1.8 + depth * 2.6,
                depth,
                alpha: 0.5 + depth * 0.45,
                soft: 0.7 - depth * 0.4,
                phase: random.range(0.0, PHASE_WRAP),
                sway_rate: random.range(0.6, 1.8),
                sway_amount: random.range(8.0, 24.0),
                settles: true,
                skipped_layer: 0,
                mass: 0.06,
                tint: piece.color,
                tint_amount: 1.0,
                blown: 0.0,
                can_wrap: false,
            });
        }
    }

    // Windows

    pub fn space_changed(&mut self) {
        self.space_changed_at = self.air.clock;
        for (id, ledge, _) in self.parking.drain(..) {
            self.dormant.insert(id, ledge);
        }
    }

    pub fn update_windows(
        &mut self,
        surfaces: &[WindowSurface],
        interval: f32,
        parked: &HashSet<WindowId>,
        elsewhere: &HashSet<WindowId>,
        sliding: bool,
    ) {
        for li in self.air.held_layers.drain(..) {
            if let Some(layer) = self.layers.get_mut(li) {
                layer.release_support();
            }
        }
        let known: HashMap<WindowId, usize> = self.ids.iter().enumerate().skip(1).map(|(li, &id)| (id, li)).collect();
        let mut seen = HashSet::new();
        let mut active_movers = Vec::new();

        for surface in surfaces {
            let left = surface.frame.min_x() + LEDGE_INSET;
            let top = surface.frame.max_y();
            let columns = ((surface.frame.width - 2.0 * LEDGE_INSET) / COLUMN_WIDTH) as i64;
            if columns < 8 {
                continue;
            }
            let columns = columns as usize;
            seen.insert(surface.id);

            if let Some(&old_frame) = self.window_frames.get(&surface.id) {
                if old_frame != surface.frame {
                    let dt = interval.max(1.0 / 120.0);
                    let velocity = V2::new(surface.frame.x - old_frame.x, surface.frame.y - old_frame.y) / dt;
                    let speed = velocity.length();
                    let min_x_diff = (surface.frame.min_x() - old_frame.min_x()).abs() / dt;
                    let max_x_diff = (surface.frame.max_x() - old_frame.max_x()).abs() / dt;
                    let min_y_diff = (surface.frame.min_y() - old_frame.min_y()).abs() / dt;
                    let max_y_diff = (surface.frame.max_y() - old_frame.max_y()).abs() / dt;
                    if speed > 15.0 || min_x_diff > 15.0 || max_x_diff > 15.0 || min_y_diff > 15.0 || max_y_diff > 15.0 {
                        active_movers.push((surface.id, old_frame, surface.frame, surface.corner_radius, velocity));
                    }
                }
            }
            self.window_frames.insert(surface.id, surface.frame);

            let li = if let Some(&li) = known.get(&surface.id) {
                let ledge = &mut self.layers[li];
                if sliding && columns == ledge.count() {
                    self.air.pan(ledge, left, top);
                } else {
                    self.air.move_ledge(ledge, left, top, columns, interval);
                }
                li
            } else {
                let ledge = match self.dormant.remove(&surface.id).or_else(|| self.unpark(surface.id)) {
                    Some(mut returning) => {
                        restore(&mut returning, left, top, columns);
                        returning
                    }
                    None => {
                        let phases = (self.air.random.range(0.0, TAU), self.air.random.range(0.0, TAU));
                        SnowLayer::new(V2::new(left, top), columns, COLUMN_WIDTH, LEDGE_DEPTH, true, phases)
                    }
                };
                self.layers.push(ledge);
                self.ids.push(surface.id);
                self.layers.len() - 1
            };
            let ledge = &mut self.layers[li];
            ledge.shape_corners(surface.corner_radius, surface.corner_exponent);
            let blockers: Vec<[f32; 4]> = surface.blockers.iter().map(|b| [b.min_x(), b.min_y(), b.max_x(), b.max_y()]).collect();
            ledge.occluders = blockers.iter().take(16).copied().collect();
            ledge.update_visibility(self.width, &blockers);
        }

        let layers = std::mem::take(&mut self.layers);
        let ids = std::mem::take(&mut self.ids);
        for (li, (mut ledge, id)) in layers.into_iter().zip(ids).enumerate() {
            if li == 0 || seen.contains(&id) {
                self.layers.push(ledge);
                self.ids.push(id);
            } else if elsewhere.contains(&id) {
                self.dormant.insert(id, ledge);
            } else if !parked.contains(&id) {
                let velocity = ledge.track_velocity + ledge.offset_velocity;
                self.air.shed(&mut ledge, 1.0, velocity);
            } else if self.air.clock - self.space_changed_at < 0.6 {
                self.dormant.insert(id, ledge);
            } else {
                self.parking.push((id, ledge, self.air.clock));
            }
        }
        if self.dormant.len() > 200
            && let Some(&stale) = self.dormant.keys().next()
        {
            self.dormant.remove(&stale);
        }
        self.window_frames.retain(|id, _| seen.contains(id));

        self.active_wakes.clear();
        for (_, _, new_frame, _, velocity) in &active_movers {
            let speed = velocity.length();
            if speed > 20.0 {
                self.active_wakes.push(WindowWake {
                    frame: *new_frame,
                    velocity: *velocity,
                });
            }
        }

        let known_now: HashMap<WindowId, usize> = self.ids.iter().enumerate().skip(1).map(|(li, &id)| (id, li)).collect();
        if !active_movers.is_empty() {
            self.air.last_window_motion = self.air.clock;
        }
        for (mover_id, old_frame, new_frame, corner_radius, velocity) in active_movers {
            let mover_li = known_now.get(&mover_id).copied();
            self.plow_with_window(mover_li, old_frame, new_frame, corner_radius, velocity);
        }
    }

    fn plow_with_window(
        &mut self,
        mover_li: Option<usize>,
        old_frame: Rect,
        new_frame: Rect,
        corner_radius: f32,
        velocity: V2,
    ) {
        if !self.windows_solid {
            return;
        }
        let speed = velocity.length().min(3000.0);
        let spray_share = (smoothstep(15.0, 300.0, speed) * 0.75).max(0.35);

        let swept_min_x = old_frame.min_x().min(new_frame.min_x());
        let swept_max_x = old_frame.max_x().max(new_frame.max_x());
        let swept_min_y = old_frame.min_y().min(new_frame.min_y());
        let swept_max_y = old_frame.max_y().max(new_frame.max_y());

        for li in 0..self.layers.len() {
            if Some(li) == mover_li {
                continue;
            }
            let layer = &mut self.layers[li];
            let base = layer.base();
            let layer_top = base.y + layer.top;
            let layer_bed = base.y - layer.bed_depth;
            let layer_min_x = base.x;
            let layer_max_x = base.x + layer.width();

            if swept_max_x < layer_min_x || swept_min_x > layer_max_x {
                continue;
            }
            if swept_min_y > layer_top || swept_max_y < layer_bed {
                continue;
            }

            let count = layer.count();
            if count == 0 {
                continue;
            }

            let col_start = (((swept_min_x - base.x) / COLUMN_WIDTH).floor() as i64).clamp(0, count as i64 - 1) as usize;
            let col_end = (((swept_max_x - base.x) / COLUMN_WIDTH).ceil() as i64).clamp(0, count as i64 - 1) as usize;
            if col_start > col_end {
                continue;
            }

            let mut pushed_left = 0.0;
            let mut pushed_right = 0.0;
            let mut any_removed = false;

            for col in col_start..=col_end {
                if !layer.visible[col] {
                    continue;
                }
                let h = layer.heights[col];
                if h <= 0.05 {
                    continue;
                }
                let x = layer.center_x(col);
                let bed_y = layer.floor(col);
                let surface_y = bed_y + h;

                let progress = if swept_max_x > swept_min_x {
                    ((x - swept_min_x) / (swept_max_x - swept_min_x)).clamp(0.0, 1.0)
                } else {
                    0.5
                };
                let mut win_bottom = old_frame.min_y() + (new_frame.min_y() - old_frame.min_y()) * progress;

                if corner_radius > 0.0 && x >= new_frame.min_x() && x <= new_frame.max_x() {
                    if x < new_frame.min_x() + corner_radius {
                        let dx = (new_frame.min_x() + corner_radius) - x;
                        let lift = corner_radius - (corner_radius * corner_radius - dx * dx).max(0.0).sqrt();
                        win_bottom += lift;
                    } else if x > new_frame.max_x() - corner_radius {
                        let dx = x - (new_frame.max_x() - corner_radius);
                        let lift = corner_radius - (corner_radius * corner_radius - dx * dx).max(0.0).sqrt();
                        win_bottom += lift;
                    }
                }

                if win_bottom < surface_y && swept_max_y >= bed_y {
                    let allowed_h = (win_bottom - bed_y).max(0.0);
                    if allowed_h < h {
                        let removed = h - allowed_h;
                        layer.set(col, allowed_h);
                        any_removed = true;

                        let area = removed * COLUMN_WIDTH;
                        let sprayed = if spray_share > 0.0 {
                            self.air.emit_window_plow(
                                area * spray_share,
                                V2::new(x, win_bottom),
                                velocity + V2::new(0.0, 30.0),
                            )
                        } else {
                            0.0
                        };

                        let leftover = area - sprayed;
                        if velocity.x > 15.0 {
                            pushed_right += leftover;
                        } else if velocity.x < -15.0 {
                            pushed_left += leftover;
                        } else {
                            let center_x = (new_frame.min_x() + new_frame.max_x()) * 0.5;
                            if x < center_x {
                                pushed_left += leftover;
                            } else {
                                pushed_right += leftover;
                            }
                        }
                    }
                }
            }

            if any_removed {
                self.air.heights_changed = true;
                self.air.last_window_motion = self.air.clock;
                layer.agitation = (layer.agitation + 12.0).min(50.0);

                if pushed_left > 0.0 {
                    let deposit_x = new_frame.min_x() - COLUMN_WIDTH * 2.0;
                    let missed = layer.deposit(pushed_left, deposit_x, true);
                    if missed > 0.0 {
                        self.air.drop_past(layer, missed, deposit_x, -speed.min(100.0));
                    }
                }
                if pushed_right > 0.0 {
                    let deposit_x = new_frame.max_x() + COLUMN_WIDTH * 2.0;
                    let missed = layer.deposit(pushed_right, deposit_x, true);
                    if missed > 0.0 {
                        self.air.drop_past(layer, missed, deposit_x, speed.min(100.0));
                    }
                }
            }
        }
    }

    fn unpark(&mut self, id: WindowId) -> Option<SnowLayer> {
        let index = self.parking.iter().position(|entry| entry.0 == id)?;
        Some(self.parking.remove(index).1)
    }

    /// Off-screen windows that no desktop switch accounts for were minimized or hidden: their snow falls.
    fn expire_parking(&mut self) {
        while let Some(first) = self.parking.first()
            && self.air.clock - first.2 > 0.25
        {
            let (_, mut ledge, _) = self.parking.remove(0);
            self.air.shed(&mut ledge, 1.0, V2::ZERO);
        }
    }

    // Cursor

    fn track_cursor(&mut self, point: V2, dt: f32) {
        for li in self.air.held_layers.drain(..) {
            self.layers[li].release_support();
        }

        let center = point + CURSOR_OFFSET;
        let Some(previous) = self.air.cursor.replace(center) else { return };

        let offscreen = previous.x < -500.0 || previous.y < -500.0 || center.x < -500.0 || center.y < -500.0;
        let delta = if offscreen { V2::ZERO } else { center - previous };
        let mut velocity = if offscreen { V2::ZERO } else { delta / dt.max(1.0 / 240.0) };
        let speed = velocity.length();
        if speed > 3000.0 {
            velocity *= 3000.0 / speed;
        }
        self.air.cursor_velocity = if offscreen { V2::ZERO } else { self.air.cursor_velocity + (velocity - self.air.cursor_velocity) * 0.5 };

        if self.cursor_force <= 0.01 || offscreen {
            return;
        }

        let radius = CURSOR_RADIUS * (0.6 + 0.4 * self.cursor_force.sqrt());
        let low = center.y.min(previous.y) - radius;
        let high = center.y.max(previous.y) + radius;
        let left = center.x.min(previous.x) - radius;
        let right = center.x.max(previous.x) + radius;
        let touched: Vec<usize> = (0..self.layers.len())
            .filter(|&li| {
                let layer = &self.layers[li];
                let base = layer.base();
                low <= base.y + layer.top + 1.0
                    && high >= base.y - layer.bed_depth
                    && right >= base.x
                    && left <= base.x + layer.width()
                    && (li == 0 || layer.top > 0.01)
            })
            .collect();
        if touched.is_empty() {
            return;
        }

        let distance = delta.length();
        if distance > 400.0 {
            for &li in &touched {
                self.air.carve(&mut self.layers[li], li, center, V2::ZERO, 0.0, true);
            }
            return;
        }
        let steps = (distance / (radius * 0.5)) as usize + 1;
        for step in 1..=steps {
            let position = previous + delta * (step as f32 / steps as f32);
            for &li in &touched {
                self.air.carve(&mut self.layers[li], li, position, velocity, distance / steps as f32, step == steps);
            }
        }
    }

    // Particles

    fn advance_flakes(&mut self, dt: f32) {
        let wind = self.air.breeze();
        let follow = (dt * 1.2).min(1.0);
        let settle = (dt * 1.6).min(1.0);
        let base_wake_radius: f32 = 80.0;
        let wake_radius = base_wake_radius * (0.6 + 0.4 * self.cursor_force.sqrt());
        let mut wake_velocity = self.air.cursor_velocity * self.cursor_force;
        let max_speed = (500.0 * self.cursor_force.max(0.2)).clamp(100.0, 3000.0);
        let wake_speed = wake_velocity.length();
        if wake_speed > max_speed {
            wake_velocity *= max_speed / wake_speed;
        }
        let wake_center = self.air.cursor.unwrap_or(V2::new(-10_000.0, -10_000.0));
        let wake_active = self.cursor_force > 0.01 && wake_speed > 15.0;
        let speed_factor = smoothstep(15.0, 150.0, wake_speed);

        let mut i = self.air.flakes.len();
        while i > 0 {
            i -= 1;
            let mut flake = self.air.flakes[i];
            if flake.blown != 0.0 {
                // The nearer the flake, the faster the gust seems to carry it.
                flake.velocity.x += (flake.blown * GUST_WIND * (1.3 + 1.3 * flake.depth) - flake.velocity.x) * (dt * 2.5).min(1.0);
            } else {
                flake.velocity.x += (wind * (0.35 + 0.65 * flake.depth) - flake.velocity.x) * follow;
            }
            flake.velocity.y += (-flake.fall_speed * self.air.speed_factor - flake.velocity.y) * settle;

            if wake_active {
                let diff = flake.position - wake_center;
                let distance_squared = diff.length_squared();
                if distance_squared < wake_radius * wake_radius && distance_squared > 0.01 {
                    let distance = distance_squared.sqrt();
                    let falloff = 1.0 - distance / wake_radius;
                    let drag = (wake_velocity - flake.velocity) * (falloff * falloff * 5.0 * self.cursor_force * dt).min(1.0);
                    let push = (diff / distance) * (falloff * 240.0 * speed_factor * self.cursor_force * dt);
                    flake.velocity += drag + push;
                }
            }

            for wake in &self.active_wakes {
                let speed = wake.velocity.length();
                let wake_radius = 80.0 * (0.6 + 0.4 * self.cursor_force.sqrt());

                if flake.position.x < wake.frame.min_x() - wake_radius
                    || flake.position.x > wake.frame.max_x() + wake_radius
                    || flake.position.y < wake.frame.min_y() - wake_radius
                    || flake.position.y > wake.frame.max_y() + wake_radius
                {
                    continue;
                }

                let clamped_x = flake.position.x.clamp(wake.frame.min_x(), wake.frame.max_x());
                let clamped_y = flake.position.y.clamp(wake.frame.min_y(), wake.frame.max_y());
                let diff = flake.position - V2::new(clamped_x, clamped_y);
                let dist_sq = diff.length_squared();

                if dist_sq < wake_radius * wake_radius {
                    let dist = dist_sq.sqrt();
                    let falloff = 1.0 - dist / wake_radius;
                    let drag = (wake.velocity - flake.velocity) * (falloff * falloff * 5.0 * self.cursor_force * dt).min(1.0);
                    let push_dir = if dist > 0.01 {
                        diff / dist
                    } else if speed > 1.0 {
                        wake.velocity / speed
                    } else {
                        V2::new(0.0, 1.0)
                    };
                    let push = push_dir * (falloff * 240.0 * self.cursor_force * dt);
                    flake.velocity += drag + push;
                }
            }

            // Harmonic ratios 2.3 and 1.3 stay continuous across the 20π wrap.
            flake.phase += flake.sway_rate * dt;
            if flake.phase > PHASE_WRAP {
                flake.phase -= PHASE_WRAP;
            }
            let sway = (flake.phase.cos() + 0.3 * (flake.phase * 2.3).cos()) * flake.sway_amount * self.air.sway_factor;
            let flutter = 1.0 + 0.12 * (flake.phase * 1.3).sin();
            if flake.tint_amount > 0.0 {
                flake.tint_amount = (flake.tint_amount - dt * 1.3).max(0.0);
            }
            let previous_y = flake.position.y;
            flake.position.x += (flake.velocity.x + sway) * dt;
            flake.position.y += flake.velocity.y * flutter * dt;

            if flake.can_wrap && flake.blown == 0.0 {
                if flake.position.x < -30.0 {
                    flake.position.x += self.width + 60.0;
                } else if flake.position.x > self.width + 30.0 {
                    flake.position.x -= self.width + 60.0;
                }
            } else if flake.position.x < -30.0 || flake.position.x > self.width + 30.0 {
                self.recycle(i);
                continue;
            }

            if flake.settles && self.accumulates && flake.blown == 0.0 {
                let hit = landing(&self.layers, flake.position.x, previous_y, flake.position.y, flake.skipped_layer, self.windows_solid);
                if let Some((li, column)) = hit {
                    if li > 0 && self.window_leak > 0.0 && self.air.random.next() < self.window_leak {
                        // Flake slips past this ledge with probability window_leak and continues falling!
                        flake.skipped_layer = li;
                        flake.can_wrap = false;
                        flake.position.y = (flake.position.y - flake.size * 0.5).min(self.layers[li].surface(column) - 0.5);
                    } else {
                        if self.snowfall_held && flake.mass < 1.0 && li == 0 {
                            self.snowfall_held = false;
                            self.snowfall_released = true;
                            self.update_target();
                        }
                        let layer = &mut self.layers[li];
                        let amount = flake.size * flake.size * DEPOSITION * flake.mass * layer.fill(column);
                        layer.deposit(amount, flake.position.x, true);
                        self.air.heights_changed = true;
                        self.recycle(i);
                        continue;
                    }
                }
            } else if flake.position.y < -flake.size * 2.0 {
                self.recycle(i);
                continue;
            }
            self.air.flakes[i] = flake;
        }
    }

    fn advance_grains(&mut self, dt: f32) {
        if self.air.grains.is_empty() {
            return;
        }
        let damping = (-GRAIN_DRAG * dt).exp();
        let mut i = self.air.grains.len();
        while i > 0 {
            i -= 1;
            let mut grain = self.air.grains[i];
            let previous_y = grain.position.y;
            if grain.blown {
                // Light powder: the wind keeps it moving toward the edge it was blown to, and it barely sinks.
                let speed = if grain.velocity.x < 0.0 { -GUST_WIND * 1.6 } else { GUST_WIND * 1.6 };
                grain.velocity.x += (speed - grain.velocity.x) * (dt * 2.0).min(1.0);
                grain.velocity.y -= GRAVITY * 0.08 * dt;
            } else {
                grain.velocity.y -= GRAVITY * dt;
                grain.velocity *= damping;
            }
            grain.position += grain.velocity * dt;

            for wake in &self.active_wakes {
                let wake_radius = 60.0;
                if grain.position.x < wake.frame.min_x() - wake_radius
                    || grain.position.x > wake.frame.max_x() + wake_radius
                    || grain.position.y < wake.frame.min_y() - wake_radius
                    || grain.position.y > wake.frame.max_y() + wake_radius
                {
                    continue;
                }

                let clamped_x = grain.position.x.clamp(wake.frame.min_x(), wake.frame.max_x());
                let clamped_y = grain.position.y.clamp(wake.frame.min_y(), wake.frame.max_y());
                let diff = grain.position - V2::new(clamped_x, clamped_y);
                let dist_sq = diff.length_squared();

                if dist_sq < wake_radius * wake_radius {
                    let dist = dist_sq.sqrt();
                    let falloff = 1.0 - dist / wake_radius;
                    let drag = (wake.velocity - grain.velocity) * (falloff * falloff * 5.0 * dt).min(1.0);
                    let push_dir = if dist > 0.01 { diff / dist } else { V2::new(0.0, 1.0) };
                    grain.velocity += drag + push_dir * (falloff * 200.0 * dt);
                }
            }

            if grain.position.x < -30.0 || grain.position.x > self.width + 30.0 {
                self.air.grains.swap_remove(i);
                continue;
            }
            if grain.position.y < -10.0 {
                self.air.grains.swap_remove(i);
                continue;
            } else if !grain.blown
                && grain.velocity.y <= 0.0
                && let Some((li, _)) = landing(&self.layers, grain.position.x, previous_y, grain.position.y, 0, self.windows_solid)
            {
                let layer = &mut self.layers[li];
                if grain.lump > 0.0 {
                    self.air.burst(&grain, layer);
                } else {
                    let missed = layer.deposit(grain.area, grain.position.x, true);
                    self.air.drop_past(layer, missed, grain.position.x, grain.velocity.x * 0.5);
                }
                self.air.heights_changed = true;
                self.air.grains.swap_remove(i);
            } else {
                self.air.grains[i] = grain;
            }
        }
    }

    fn recycle(&mut self, i: usize) {
        if self.air.flakes.len() <= self.target_flakes && !self.air.blowing() {
            let y = self.height + self.air.random.range(4.0, 40.0);
            self.air.flakes[i] = self.air.make_flake(self.width, y);
        } else {
            self.air.flakes.swap_remove(i);
        }
    }

    /// No new snow falls until the gust has carried off the last flake, then the snowfall starts again from the top.
    fn spawn_flakes(&mut self) {
        if self.air.blowing() {
            return;
        }
        while self.air.flakes.len() < self.target_flakes {
            let y = self.height + self.air.random.range(0.0, self.height * 0.8);
            let flake = self.air.make_flake(self.width, y);
            self.air.flakes.push(flake);
        }
    }

    /// Wanders between light lulls, steady snow and heavy flurries.
    fn vary(&mut self, dt: f32) {
        // In case none of the scattered snow makes it to the bottom, say because it all came to rest on windows.
        if self.snowfall_held && self.scattered_until > 0.0 && self.air.clock > self.scattered_until {
            self.snowfall_held = false;
            self.snowfall_released = true;
            self.update_target();
        }
        if self.snowfall != Snowfall::Variable {
            self.strength_target = 1.0;
        } else if self.air.clock >= self.next_shift {
            let random = &mut self.air.random;
            let roll = random.next();
            self.strength_target = if roll < 0.25 {
                random.range(0.25, 0.5)
            } else if roll < 0.75 {
                random.range(0.7, 1.3)
            } else {
                random.range(1.5, 2.1)
            };
            self.next_shift = self.air.clock + random.range(60.0, 240.0) as f64;
        }
        if (self.strength_target - self.air.strength).abs() <= 0.001 {
            return;
        }
        self.air.strength += (self.strength_target - self.air.strength) * (1.0 - (-dt / 25.0).exp());
        self.update_target();
    }

    // Gust

    /// The way the breeze blows now: 1 to the right, -1 to the left.
    pub fn breeze_direction(&self) -> f32 {
        if self.air.breeze() < 0.0 { -1.0 } else { 1.0 }
    }

    /// Clears all the snow: a gust in `direction` lifts it off the ground and the windows and blows it off the screen,
    /// or, `calm`, it melts away where it lies.
    pub fn blow_away(&mut self, direction: f32, calm: bool) {
        let direction = if direction < 0.0 { -1.0 } else { 1.0 };
        self.air.gust = Some(Gust { start: self.air.clock, direction, calm });
        // Snow kept for windows on other desktops goes too, out of sight.
        self.dormant.clear();
        for (_, ledge, _) in &mut self.parking {
            for i in 0..ledge.count() {
                ledge.set(i, 0.0);
            }
        }
    }

    /// Sweeps snow off window ledges in the direction of workspace transition,
    /// bursting it into powder and clouds, and accelerating airborne flakes.
    /// Does not sweep surfaces in `static_ids` (e.g. Quickshell layers, pinned windows).
    /// On window ledges: 100% of snow flies away into particles and clouds (drifts do not remain).
    /// On the ground: snowdrifts are partially swept (~12-28%), shedding loose powder.
    pub fn sweep_window_ledges(&mut self, direction: f32, distance: f32, static_ids: &HashSet<WindowId>) {
        let dir = if direction < 0.0 { -1.0 } else { 1.0 };
        let duration = (0.8 + 0.4 * distance.max(1.0)).min(2.5) as f64;
        self.workspace_transition_until = self.air.clock + duration;
        self.air.workspace_wind_start = self.air.clock;
        self.air.workspace_wind_until = self.air.clock + duration;
        self.air.workspace_wind_dir = dir;
        let sweep_speed = (550.0 + 150.0 * distance.clamp(1.0, 3.0)) * self.wind_factor.max(0.5);
        self.air.workspace_wind = sweep_speed;

        // On window ledges: 100% of snow flies away ("весь снег улетает сугробы не остаются"),
        // swept into flying grains and cloud motes just like a cursor snowplow!
        for li in 1..self.layers.len() {
            let id = self.ids[li];
            if static_ids.contains(&id) {
                continue;
            }
            let layer = &mut self.layers[li];
            let mut dust = 0.0;
            for i in 0..layer.count() {
                let h = layer.heights[i];
                if h <= 0.05 {
                    continue;
                }
                let x = layer.center_x(i);
                let surface = layer.surface(i);
                let area = h * layer.column_width;
                dust += area;
                while dust >= 3.0 {
                    self.air.plow_sweep_snow(3.0, V2::new(x, surface), dir, sweep_speed);
                    dust -= 3.0;
                }
                layer.set(i, 0.0);
            }
            if dust > 0.5 {
                let x = layer.base().x + layer.width() * 0.5;
                let surface = layer.base().y;
                self.air.plow_sweep_snow(dust, V2::new(x, surface), dir, sweep_speed);
            }
        }

        // On the ground: snowdrifts are partially swept ("сугробы могут немного сметаться")
        // shedding loose top snow (12-28%) into flying grains and cloud motes
        let ground_sweep_fraction = (0.18 + 0.05 * (distance - 1.0).max(0.0)).clamp(0.12, 0.28) * self.wind_factor.clamp(0.2, 2.0);
        let ground = &mut self.layers[0];
        let mut ground_dust = 0.0;
        for i in 0..ground.count() {
            let h = ground.heights[i];
            if h <= 0.05 {
                continue;
            }
            let loss = (h * ground_sweep_fraction).min(h);
            let x = ground.center_x(i);
            let surface = ground.surface(i);
            let area = loss * ground.column_width;
            ground_dust += area;
            while ground_dust >= 3.5 {
                self.air.plow_sweep_snow(3.5, V2::new(x, surface), dir, sweep_speed);
                ground_dust -= 3.5;
            }
            ground.set(i, h - loss);
        }

        if self.wind_accel {
            let dist_multiplier = (0.2 + 0.4 * distance.max(1.0)).min(4.5);
            let power = dist_multiplier * self.wind_factor.max(0.0);
            for flake in &mut self.air.flakes {
                let random = &mut self.air.random;
                flake.velocity.x += dir * random.range(300.0, 700.0) * power;
                flake.velocity.y += random.range(20.0, 80.0) * power;
            }
        }
        self.air.heights_changed = true;
    }

    /// Sweeps snow off window ledges AND the ground.
    pub fn sweep_all(&mut self, direction: f32, distance: f32, static_ids: &HashSet<WindowId>) {
        let dir = if direction < 0.0 { -1.0 } else { 1.0 };
        self.sweep_window_ledges(dir, distance, static_ids);
        let sweep_speed = (550.0 + 150.0 * distance.clamp(1.0, 3.0)) * self.wind_factor.max(0.5);
        // sweep_all applies an additional sweep on ground for more aggressive clearing if requested
        let sweep_fraction = (0.35 + 0.10 * (distance - 1.0).max(0.0)).clamp(0.25, 0.60) * self.wind_factor.clamp(0.2, 2.0);
        let ground = &mut self.layers[0];
        let mut dust = 0.0;
        for i in 0..ground.count() {
            let h = ground.heights[i];
            if h <= 0.05 {
                continue;
            }
            let loss = (h * sweep_fraction).min(h);
            let x = ground.center_x(i);
            let surface = ground.surface(i);
            let area = loss * ground.column_width;
            dust += area;
            while dust >= 3.5 {
                self.air.plow_sweep_snow(3.5, V2::new(x, surface), dir, sweep_speed);
                dust -= 3.5;
            }
            ground.set(i, h - loss);
        }
        self.air.heights_changed = true;
    }

    /// Continuous aerodynamic wind erosion on snowdrifts.
    /// When wind (including static_wind and workspace wind) exceeds the erosion threshold (45 px/s),
    /// surface snow is gradually eroded and swept off into airborne powder (lift grains and motes).
    fn erode_snowdrifts_by_wind(&mut self, dt: f32) {
        let wind = self.air.wind_speed();
        let wind_abs = wind.abs();
        let threshold = 45.0;
        if wind_abs <= threshold {
            return;
        }
        let excess = wind_abs - threshold;
        let wind_dir = if wind >= 0.0 { 1.0 } else { -1.0 };
        let erosion_rate = (0.00025 * excess.min(600.0) * self.wind_factor.clamp(0.2, 2.0)) * dt;
        if erosion_rate <= 0.000001 {
            return;
        }

        let is_ws = self.air.clock < self.workspace_transition_until;

        for (li, layer) in self.layers.iter_mut().enumerate() {
            let layer_factor = if li == 0 { 0.6 } else { 1.0 };
            let rate = (erosion_rate * layer_factor).min(0.25);
            let mut dust = 0.0;
            let count = layer.count();
            for i in 0..count {
                let h = layer.heights[i];
                if h <= 0.5 {
                    continue;
                }
                let loss = (h * rate).min(h - 0.2);
                if loss > 0.001 {
                    layer.set(i, h - loss);
                    self.air.heights_changed = true;
                    if layer.visible[i] {
                        dust += loss * layer.column_width;
                        let threshold_dust = if is_ws { 2.5 } else { 4.0 };
                        if dust >= threshold_dust {
                            let x = layer.center_x(i);
                            let surface = layer.surface(i);
                            self.air.lift(V2::new(x, surface), wind_dir);
                            dust = 0.0;
                        }
                    }
                }
            }
        }
    }

    /// Bursts the snow on a window in all directions into clouds and powder (e.g. when going fullscreen).
    pub fn burst_window_omnidirectional(&mut self, id: WindowId) {
        let (found, width, base_x) = if let Some(li) = self.ids.iter().position(|&lid| lid == id) {
            let l = &self.layers[li];
            (Some(li), l.width(), l.base().x)
        } else {
            (None, 0.0, 0.0)
        };

        let center_x = base_x + width * 0.5;
        let mut had_snow = false;

        if let Some(li) = found {
            let layer = &mut self.layers[li];
            let mut dust = 0.0;
            for i in 0..layer.count() {
                let h = layer.heights[i];
                if h <= 0.05 {
                    continue;
                }
                had_snow = true;
                let x = layer.center_x(i);
                let surface = layer.surface(i);
                dust += h * layer.column_width;
                while dust >= 3.0 {
                    let dir = if self.air.random.next() > 0.5 { 1.0 } else { -1.0 };
                    self.air.lift(V2::new(x, surface), dir);
                    if self.air.motes.len() < MAX_MOTES {
                        let random = &mut self.air.random;
                        let angle = random.range(0.2, std::f32::consts::PI - 0.2);
                        let speed = random.range(60.0, 200.0);
                        self.air.motes.push(Mote {
                            position: V2::new(x, surface),
                            velocity: V2::new(angle.cos() * speed, angle.sin() * speed),
                            size: random.range(8.0, 18.0),
                            alpha: random.range(0.12, 0.28),
                            age: 0.0,
                            life: random.range(0.5, 1.2),
                        });
                    }
                    dust -= 3.0;
                }
                layer.set(i, 0.0);
            }
        } else if let Some(layer) = self.dormant.get_mut(&id) {
            let mut dust = 0.0;
            for i in 0..layer.count() {
                let h = layer.heights[i];
                if h <= 0.05 {
                    continue;
                }
                had_snow = true;
                let x = layer.center_x(i);
                let surface = layer.surface(i);
                dust += h * layer.column_width;
                while dust >= 3.0 {
                    let dir = if self.air.random.next() > 0.5 { 1.0 } else { -1.0 };
                    self.air.lift(V2::new(x, surface), dir);
                    if self.air.motes.len() < MAX_MOTES {
                        let random = &mut self.air.random;
                        let angle = random.range(0.2, std::f32::consts::PI - 0.2);
                        let speed = random.range(60.0, 200.0);
                        self.air.motes.push(Mote {
                            position: V2::new(x, surface),
                            velocity: V2::new(angle.cos() * speed, angle.sin() * speed),
                            size: random.range(8.0, 18.0),
                            alpha: random.range(0.12, 0.28),
                            age: 0.0,
                            life: random.range(0.5, 1.2),
                        });
                    }
                    dust -= 3.0;
                }
                layer.set(i, 0.0);
            }
        }

        if had_snow {
            for flake in &mut self.air.flakes {
                let dx = flake.position.x - center_x;
                if dx.abs() < width * 0.6 {
                    let sign = if dx >= 0.0 { 1.0 } else { -1.0 };
                    flake.velocity.x += sign * self.air.random.range(150.0, 350.0);
                    flake.velocity.y += self.air.random.range(40.0, 120.0);
                }
            }
            self.air.heights_changed = true;
        }
    }

    fn blow(&mut self, dt: f32) {
        let Some(gust) = self.air.gust else { return };
        let age = (self.air.clock - gust.start) as f32;
        if age >= GUST_LENGTH && !self.air.flakes.iter().any(|flake| flake.blown != 0.0) {
            self.air.gust = None;
            return;
        }
        let margin = SIDE_MARGIN;
        let front = gust_front(self.width, age, gust.direction, margin);
        // Whatever the front has long passed is gone by now, however deep it was.
        let finish = age > GUST_SWEEP + if gust.calm { 0.6 } else { 0.8 };
        if !gust.calm {
            let trailing = gust_front(self.width, age - GUST_LAG, gust.direction, margin);
            for flake in self.air.flakes.iter_mut().filter(|flake| flake.blown == 0.0) {
                // Each flake is caught a little further behind, so the snowfall doesn't go as one straight line.
                let stagger = flake.phase / PHASE_WRAP * 240.0;
                if finish || (trailing - flake.position.x) * gust.direction > stagger {
                    flake.blown = gust.direction;
                }
            }
        }
        for layer in &mut self.layers {
            let mut dust = 0.0;
            for i in 0..layer.count() {
                let height = layer.heights[i];
                if height <= 0.0 {
                    continue;
                }
                let x = layer.center_x(i);
                if !gust.calm && (front - x) * gust.direction <= 0.0 {
                    continue;
                }
                let surface = layer.surface(i);
                let rate = if gust.calm { height * 2.5 + 6.0 } else { height * 8.0 + 50.0 };
                let loss = if finish { height } else { height.min(rate * dt) };
                layer.set(i, height - loss);
                self.air.heights_changed = true;
                if gust.calm || !layer.visible[i] {
                    continue;
                }
                dust += loss * layer.column_width;
                while dust >= 3.5 {
                    self.air.lift(V2::new(x, surface), gust.direction);
                    dust -= 3.5;
                }
            }
        }
    }

    // Piles

    fn settle(&mut self, dt: f32) {
        // Melting nearly keeps pace with the snowfall, so heavier snowfall builds drifts only a little higher.
        let falling = self.snowfall.flakes_per_megapoint() * self.air.strength * self.density;
        let melt_pace = if falling > 0.0 { (falling / THAW_BALANCE).powf(THAW_EXPONENT) } else { 1.0 };
        for (li, layer) in self.layers.iter_mut().enumerate() {
            if !self.accumulates || self.thaws {
                let floor = if !self.accumulates {
                    0.0
                } else if li == 0 {
                    THAW_DEPTH
                } else {
                    LEDGE_THAW_DEPTH
                };
                if layer.thaw(dt * melt_pace * self.thaw_rate_multiplier, floor, THAW_RATE, THAW_TIME) {
                    self.air.heights_changed = true;
                }
            }
            // Shaken harder than gravity, a granular pile fluidizes and slumps to a flatter slope.
            let repose = if layer.agitation > SHAKING { REPOSE * 0.35 } else { REPOSE };
            if layer.relax(repose, ROUNDING) {
                self.air.heights_changed = true;
            }
            if layer.spill_left > 1.0 {
                let velocity = if li == 0 {
                    V2::new(-20.0, 5.0)
                } else {
                    layer.track_velocity + V2::new(-6.0, -15.0)
                };
                self.air.release(layer.spill_left, edge_point(layer, true), velocity, V2::new(0.25, 1.0), 0.0);
                layer.spill_left = 0.0;
            }
            if layer.spill_right > 1.0 {
                let velocity = if li == 0 {
                    V2::new(20.0, 5.0)
                } else {
                    layer.track_velocity + V2::new(6.0, -15.0)
                };
                self.air.release(layer.spill_right, edge_point(layer, false), velocity, V2::new(0.25, 1.0), 0.0);
                layer.spill_right = 0.0;
            }
        }
    }

    fn update_target(&mut self) {
        let per_megapoint = if self.snowfall_held { 0.0 } else { self.snowfall.flakes_per_megapoint() };
        let sim_width = self.width + 2.0 * SIDE_MARGIN;
        self.target_flakes = (per_megapoint * self.air.strength * self.density * sim_width * self.height / 1_000_000.0) as usize;
    }
}

impl Air {
    /// Carries snow along with a window that the whole desktop is sliding past: nothing pushes the snow, so it doesn't react.
    fn pan(&mut self, ledge: &mut SnowLayer, left: f32, top: f32) {
        let shift = V2::new(left, top) - ledge.origin;
        if shift == V2::ZERO {
            return;
        }
        ledge.origin += shift;
        ledge.track += shift;
        ledge.previous_base_y = ledge.base().y;
        if ledge.top > 0.05 {
            self.last_window_motion = self.clock;
            self.windows_changed = true;
        }
    }

    fn move_ledge(&mut self, ledge: &mut SnowLayer, left: f32, top: f32, columns: usize, interval: f32) {
        let old_left = ledge.origin.x;
        let new_right = left + columns as f32 * COLUMN_WIDTH;
        let old_right = old_left + ledge.width();
        // A resize keeps the edge that didn't move fixed; a plain move shifts both edges alike.
        let shift = if (left - old_left).abs() <= (new_right - old_right).abs() { left - old_left } else { new_right - old_right };
        let displacement = V2::new(shift, top - ledge.origin.y);
        let resized = columns != ledge.count() || (left - old_left - shift).abs() > 0.5;
        if displacement == V2::ZERO && !resized {
            return;
        }

        let elapsed = interval.max(1.0 / 120.0);
        // Only a motion that began unobserved, between slow polls, has to be picked up mid-flight.
        let was_resting = self.clock - ledge.moved_at > 0.3 && interval > 0.05;
        ledge.moved_at = self.clock;
        if ledge.top > 0.05 {
            self.last_window_motion = self.clock;
            self.windows_changed = true;
        }

        let speed = displacement.length() / elapsed;
        let jumped = speed > JUMP_SPEED;
        if jumped {
            // The window vanished from under the snow and reappeared elsewhere: nothing carried the snow along.
            self.shed(ledge, 1.0, V2::ZERO);
            ledge.offset = V2::ZERO;
            ledge.offset_velocity = V2::ZERO;
            ledge.track_velocity = V2::ZERO;
        } else if self.motion_shed > 0.01 {
            let shed_threshold = (400.0 / self.motion_shed).clamp(50.0, 2500.0);
            if speed > shed_threshold {
                let excess = (speed - shed_threshold) / (shed_threshold * 0.6);
                let share = (excess * 1.2 * self.motion_shed).clamp(0.25, 1.0);
                let toss = displacement / elapsed * 0.45 + V2::new(self.random.range(-60.0, 60.0), self.random.range(40.0, 140.0));
                self.shed(ledge, share, toss);
            }
        }

        if resized {
            let mut fresh = vec![0.0; columns];
            for i in 0..ledge.count() {
                let h = ledge.heights[i];
                if h <= 0.0 {
                    continue;
                }
                let j = ((ledge.center_x(i) + shift - left - ledge.offset.x) / COLUMN_WIDTH).floor();
                if j >= 0.0 && (j as usize) < columns {
                    fresh[j as usize] += h;
                } else {
                    let position = V2::new(ledge.center_x(i), ledge.floor(i));
                    self.release(h * COLUMN_WIDTH, position, ledge.track_velocity, V2::new(COLUMN_WIDTH / 2.0, h), 0.0);
                }
            }
            ledge.replace(fresh);
        }
        ledge.origin = V2::new(left, top);
        if jumped {
            ledge.track = ledge.origin;
        } else if was_resting {
            // Join the motion at its current pace, with the lag a steady follow would have, instead of as a jolt.
            ledge.track_velocity = displacement / elapsed;
            ledge.track = ledge.origin - ledge.track_velocity * TRACK_TIME;
        }
    }

    /// Moves a ledge's snow as a body riding on its window: it feels gravity plus the inertia of the
    /// window's acceleration, sticks by friction, slides when that's exceeded, and flies free when the
    /// window drops away faster than gravity or stops short after rising.
    fn carry(&mut self, ledge: &mut SnowLayer, dt: f32) {
        let previous_velocity = ledge.track_velocity;
        smooth_damp(&mut ledge.track, &mut ledge.track_velocity, ledge.origin, TRACK_TIME, dt);
        let acceleration = (ledge.track_velocity - previous_velocity) / dt;
        // Agitation is vibration: two consecutive back-and-forth swings, not a single start or stop.
        ledge.agitation *= (-dt / 0.3).exp();
        let magnitude = acceleration.length();
        if magnitude > 1.0 && acceleration.dot(ledge.swing_direction) < 0.0 {
            ledge.agitation = ledge.agitation.max(ledge.previous_swing_peak.min(ledge.swing_peak) / GRAVITY);
            ledge.previous_swing_peak = ledge.swing_peak;
            ledge.swing_peak = magnitude;
            ledge.swing_direction = acceleration / magnitude;
        } else {
            ledge.swing_peak = ledge.swing_peak.max(magnitude);
            if magnitude > 1.0 && ledge.swing_direction == V2::ZERO {
                ledge.swing_direction = acceleration / magnitude;
            }
        }

        if ledge.top <= 0.05 {
            ledge.offset = V2::ZERO;
            ledge.offset_velocity = V2::ZERO;
            return;
        }

        let mut lift = ledge.offset.y;
        let mut lift_velocity = ledge.offset_velocity.y;
        let fall = -GRAVITY - acceleration.y;
        let effective_adhesion = if self.motion_shed > 0.01 {
            (ADHESION / (self.motion_shed * 2.0).max(0.5)).clamp(100.0, ADHESION)
        } else {
            ADHESION
        };
        let effective_grip = if self.motion_shed > 0.01 {
            (GRIP / (self.motion_shed * 2.0).max(0.5)).clamp(0.1, GRIP)
        } else {
            GRIP
        };
        if lift > 0.0 || fall > effective_adhesion {
            lift_velocity += fall * dt;
            lift += lift_velocity * dt;
            if lift <= 0.0 {
                lift = 0.0;
                lift_velocity = 0.0;
            }
        }

        let pressure = if lift > 0.0 { 0.0 } else { (GRAVITY + effective_adhesion + acceleration.y).max(0.0) };
        let drive = -acceleration.x;
        let mut slide_velocity = ledge.offset_velocity.x;
        if !(slide_velocity == 0.0 && drive.abs() <= effective_grip * pressure) {
            let direction: f32 = if slide_velocity != 0.0 {
                slide_velocity.signum()
            } else if drive > 0.0 {
                1.0
            } else {
                -1.0
            };
            let next = slide_velocity + (drive - direction * SLIP * pressure) * dt;
            slide_velocity = if pressure > 0.0 && next * direction <= 0.0 { 0.0 } else { next };
        }

        let mut slide = ledge.offset.x + slide_velocity * dt;
        let carried = ledge.track_velocity + V2::new(slide_velocity, lift_velocity);
        while slide.abs() >= COLUMN_WIDTH {
            let direction = if slide > 0.0 { 1 } else { -1 };
            let (height, x, y) = ledge.slide_column(direction);
            slide -= direction as f32 * COLUMN_WIDTH;
            if height > 0.01 {
                let x = x + direction as f32 * COLUMN_WIDTH;
                self.release(height * COLUMN_WIDTH, V2::new(x, y), carried, V2::new(COLUMN_WIDTH / 2.0, height), 0.0);
            }
        }

        let moved = lift != ledge.offset.y || slide != ledge.offset.x;
        ledge.offset = V2::new(slide, lift);
        ledge.offset_velocity = V2::new(slide_velocity, lift_velocity);
        if moved {
            self.heights_changed = true;
        }

        let shake_threshold = if self.motion_shed > 0.01 {
            (0.5 / self.motion_shed).clamp(0.08, SHAKING)
        } else {
            SHAKING
        };
        if lift > BREAK_LIFT {
            self.shed(ledge, 1.0, carried);
            ledge.offset = V2::ZERO;
            ledge.offset_velocity = V2::ZERO;
        } else if ledge.agitation > shake_threshold {
            // A granular bed shaken well past gravity throws grains off its surface.
            let rate = (0.25 * (ledge.agitation - shake_threshold) * self.motion_shed.max(0.5)).min(0.95);
            let picks = (ledge.count() as f32 * rate * dt * 4.0 + self.random.next()) as usize;
            for _ in 0..picks {
                let column = ((self.random.next() * ledge.count() as f32) as usize).min(ledge.count() - 1);
                let h = ledge.heights[column];
                if h <= 0.1 {
                    continue;
                }
                let shed_amount = (h * 0.6 * self.motion_shed.max(0.5)).min(h);
                ledge.set(column, h - shed_amount);
                let velocity = carried + V2::new(self.random.range(-100.0, 100.0), self.random.range(60.0, 220.0));
                let position = V2::new(ledge.center_x(column), ledge.floor(column) + h - shed_amount * 0.5);
                self.release(shed_amount * COLUMN_WIDTH, position, velocity, V2::new(COLUMN_WIDTH / 2.0, shed_amount), 0.25);
            }
            if picks > 0 {
                self.heights_changed = true;
            }
        }
    }

    /// The part of a window slid past a wall carries no snow: the wall scrapes it off, and it heaps up against the
    /// wall, which holds up the heap's face as the cursor holds up the snow it touches.
    fn confine(&mut self, ledge: &mut SnowLayer, li: usize, width: f32, walls: (bool, bool)) {
        let count = ledge.count() as i64;
        let base = ledge.base().x;
        let first = if walls.0 { ((-base / COLUMN_WIDTH - 0.5).ceil() as i64).clamp(0, count) } else { 0 };
        let last = if walls.1 { (((width - base) / COLUMN_WIDTH - 0.5).floor() as i64).clamp(-1, count - 1) } else { count - 1 };
        if first == 0 && last == count - 1 {
            return;
        }
        if first > last {
            // Nothing of the window is left this side of the wall: its snow falls down the wall.
            let x = if first > 0 { 1.0 } else { width - 1.0 };
            for i in 0..ledge.count() {
                let h = ledge.heights[i];
                if h <= 0.05 {
                    continue;
                }
                ledge.set(i, 0.0);
                self.release(h * COLUMN_WIDTH, V2::new(x, ledge.floor(i)), V2::ZERO, V2::new(1.0, h), 0.0);
            }
            self.heights_changed = true;
            return;
        }
        if !self.held_layers.contains(&li) {
            self.held_layers.push(li);
        }
        self.scrape(ledge, 0..first as usize, first as usize, 1);
        self.scrape(ledge, (last + 1) as usize..ledge.count(), last as usize, -1);
    }

    /// Heaps the snow on `columns`, past a wall, onto column `edge` just inside it, packing down what wasn't packed
    /// already. Scraped off fast, some of it crumbles off the heap and tumbles back over the window.
    fn scrape(&mut self, ledge: &mut SnowLayer, columns: Range<usize>, edge: usize, inward: i32) {
        let mut area = 0.0;
        for i in columns {
            let packed = ledge.packed[i].min(ledge.heights[i]);
            area += (packed + (ledge.heights[i] - packed) * PACKING) * COLUMN_WIDTH;
            ledge.set(i, 0.0);
            ledge.hold(i);
        }
        if area <= 0.01 {
            return;
        }
        self.heights_changed = true;
        let direction = inward as f32;
        let speed = (-direction * (ledge.track_velocity.x + ledge.offset_velocity.x)).max(0.0);
        let crumbled = area * 0.4 * smoothstep(200.0, 1200.0, speed);
        if crumbled > 0.5 {
            let position = V2::new(ledge.center_x(edge), ledge.surface(edge));
            let velocity = V2::new(direction * speed * 0.25, 60.0 + speed * 0.12);
            self.release(crumbled, position, velocity, V2::new(COLUMN_WIDTH, 2.0), 0.6);
        }
        ledge.heap(area - crumbled, edge, inward, REPOSE * COLUMN_WIDTH);
    }

    /// Turns part of a ledge's snow into falling snow where it lies: a few columns at a time hold together
    /// as a lump, the rest pours off as powder.
    fn shed(&mut self, ledge: &mut SnowLayer, share: f32, velocity: V2) {
        let mut start = 0;
        while start < ledge.count() {
            let end = ledge.count().min(start + self.random.range(2.0, 7.0) as usize);
            let mut area = 0.0;
            let mut lowest = f32::MAX;
            let mut highest = -f32::MAX;
            for i in start..end {
                let h = ledge.heights[i];
                if h <= 0.05 {
                    continue;
                }
                area += h * share * COLUMN_WIDTH;
                lowest = lowest.min(ledge.floor(i) + h * (1.0 - share));
                highest = highest.max(ledge.floor(i) + h);
            }
            let cohesion = if area > 14.0 { self.random.range(0.4, 0.7) } else { 0.0 };
            for i in start..end {
                let h = ledge.heights[i];
                if h <= 0.05 {
                    continue;
                }
                let amount = h * share;
                ledge.set(i, h - amount);
                let position = V2::new(ledge.center_x(i), ledge.floor(i) + h - amount);
                self.release(amount * COLUMN_WIDTH * (1.0 - cohesion), position, velocity, V2::new(COLUMN_WIDTH / 2.0, amount), 0.25);
            }
            if cohesion > 0.0 {
                let left = ledge.center_x(start);
                let right = ledge.center_x(end - 1);
                self.throw_lump(area * cohesion, V2::new((left + right) / 2.0, (lowest + highest) / 2.0), velocity);
            }
            if self.motes.len() < MAX_MOTES && self.random.next() < 0.5 {
                let left = ledge.center_x(start);
                let right = ledge.center_x(end - 1);
                let center = V2::new((left + right) * 0.5, (lowest + highest) * 0.5);
                let speed = velocity.length();
                self.motes.push(Mote {
                    position: center + V2::new(self.random.range(-5.0, 5.0), self.random.range(-2.0, 4.0)),
                    velocity: velocity * 0.5 + V2::new(self.random.range(-30.0, 30.0), self.random.range(20.0, 90.0) + speed * 0.05),
                    size: self.random.range(8.0, 18.0),
                    alpha: self.random.range(0.08, 0.2),
                    age: 0.0,
                    life: self.random.range(0.5, 1.2),
                });
            }
            start = end;
        }
        self.heights_changed = true;
    }

    fn throw_lump(&mut self, area: f32, position: V2, velocity: V2) {
        let pieces = (area / 90.0) as usize + 1;
        let share = area / pieces as f32;
        for _ in 0..pieces {
            if self.grains.len() >= MAX_GRAINS {
                break;
            }
            let random = &mut self.random;
            self.grains.push(Grain {
                position: position + V2::new(random.range(-2.0, 2.0), random.range(-1.0, 1.0)),
                // Lumps break away a moment before the powder around them, so a falling sheet comes apart.
                velocity: velocity * (1.0 + random.range(-0.2, 0.2)) + V2::new(random.range(-25.0, 25.0), random.range(-80.0, 0.0)),
                area: share,
                size: (share.sqrt() * 1.05).min(11.0),
                lump: random.range(0.05, 1.0),
                blown: false,
            });
        }
    }

    /// A lump hitting the snow: most of it packs in where it lands, a few crumbs hop off and a puff of powder rises.
    fn burst(&mut self, lump: &Grain, layer: &mut SnowLayer) {
        let impact = (-lump.velocity.y).max(0.0);
        let force = (impact / 900.0).clamp(0.35, 1.3);
        let crumbs = lump.area * 0.18;
        for spot in [-0.3, 0.0, 0.3] {
            let x = lump.position.x + spot * lump.size;
            let missed = layer.deposit((lump.area - crumbs) / 3.0, x, true);
            self.drop_past(layer, missed, x, lump.velocity.x * 0.3);
        }
        let surface = layer.index(lump.position.x).map_or(lump.position.y, |i| layer.surface(i));
        let velocity = V2::new(lump.velocity.x * 0.3, impact * self.random.range(0.08, 0.16));
        self.release(crumbs, V2::new(lump.position.x, surface + 1.0), velocity, V2::new(lump.size * 0.4, 1.0), 0.6);

        let count = MAX_MOTES.saturating_sub(self.motes.len()).min((2.0 + lump.size * 0.6 * force) as usize);
        for _ in 0..count {
            let random = &mut self.random;
            let angle = random.range(0.15, PI - 0.15);
            let speed = random.range(25.0, 110.0) * force;
            self.motes.push(Mote {
                position: V2::new(lump.position.x + random.range(-0.5, 0.5) * lump.size, surface + random.range(0.0, lump.size * 0.4)),
                velocity: V2::new(angle.cos() * speed * 1.4 + lump.velocity.x * 0.2, angle.sin() * speed * 0.6),
                size: random.range(0.3, 0.6) * lump.size + 1.0,
                alpha: random.range(0.1, 0.22),
                age: 0.0,
                life: random.range(0.45, 0.9),
            });
        }
    }

    /// `scatter` varies each clump's speed by that fraction, so a thrown sheet of snow breaks up instead of flying as one band.
    fn release(&mut self, area: f32, position: V2, velocity: V2, spread: V2, scatter: f32) {
        if area <= 0.02 {
            return;
        }
        let room = MAX_GRAINS.saturating_sub(self.grains.len());
        if room == 0 {
            self.overflow.push((area, position.x));
            return;
        }
        let count = room.min(((area / 3.0) as usize).clamp(1, 6));
        let share = area / count as f32;
        for _ in 0..count {
            let random = &mut self.random;
            let jitter = V2::new(random.range(-12.0, 12.0) - 40.0 * scatter * random.range(-1.0, 1.0), random.range(-10.0, 8.0));
            self.grains.push(Grain {
                position: position + V2::new(random.range(-spread.x, spread.x), random.range(0.0, spread.y)),
                velocity: velocity * (1.0 + random.range(-scatter, scatter)) + jitter,
                area: share,
                size: (share.sqrt() * 1.05).clamp(1.1, 5.0),
                lump: 0.0,
                blown: false,
            });
        }
    }

    /// Snow that lands past the end of a ledge drops off that end instead.
    fn drop_past(&mut self, layer: &SnowLayer, area: f32, x: f32, velocity: f32) {
        if !layer.opens_at_edges || area <= 0.05 {
            return;
        }
        let left_side = x < layer.base().x + layer.width() / 2.0;
        self.release(area, edge_point(layer, left_side), V2::new(velocity, -10.0), V2::new(0.25, 1.0), 0.0);
    }

    fn carve(&mut self, layer: &mut SnowLayer, li: usize, center: V2, velocity: V2, travel: f32, last: bool) {
        let radius = CURSOR_RADIUS;
        let base = layer.base();
        let lower = (((center.x - radius - base.x) / COLUMN_WIDTH).floor() as i64).max(0);
        let upper = (((center.x + radius - base.x) / COLUMN_WIDTH).floor() as i64).min(layer.count() as i64 - 1);
        if lower > upper {
            return;
        }

        let local = center.y - base.y;
        let spray_share = smoothstep(60.0, 420.0, velocity.length());
        let mut pushed_left = 0.0;
        let mut pushed_right = 0.0;

        for i in lower as usize..=upper as usize {
            if !layer.visible[i] {
                continue;
            }
            let x = layer.center_x(i);
            let dx = x - center.x;
            let reach = radius * radius - dx * dx;
            if reach <= 0.0 {
                continue;
            }
            let half = reach.sqrt();
            // Heights here are measured up from the bed under this column.
            let bed_level = layer.bed[i];
            let bottom = (local - half - bed_level).max(0.0);
            let top = local + half - bed_level;
            if top <= bottom {
                continue;
            }
            let h = layer.heights[i];
            let buried = h > top;
            if last && !buried && h >= bottom - 1.0 {
                if !self.held_layers.contains(&li) {
                    self.held_layers.push(li);
                }
                layer.hold(i);
            }
            if h <= bottom + 0.01 {
                continue;
            }

            // A buried cursor only displaces the snow it sweeps through; the rest rests on top of it.
            let removed = if buried { (top - bottom) * (travel / (2.0 * radius)).min(1.0) } else { h - bottom };
            if removed <= 0.001 {
                continue;
            }
            layer.set(i, h - removed);
            self.last_cursor_push = self.clock;
            self.heights_changed = true;

            let area = removed * COLUMN_WIDTH;
            let surface = layer.floor(i) + if buried { h - removed } else { h };
            let sprayed = if spray_share > 0.0 { self.emit(area * spray_share, V2::new(x, surface), center, velocity) } else { 0.0 };
            let side = if velocity.x.abs() > 40.0 { velocity.x } else { dx };
            if side < 0.0 {
                pushed_left += area - sprayed;
            } else {
                pushed_right += area - sprayed;
            }
        }

        for (area, x) in [(pushed_left, center.x - radius - COLUMN_WIDTH * 3.0), (pushed_right, center.x + radius + COLUMN_WIDTH * 3.0)] {
            if area <= 0.0 {
                continue;
            }
            let missed = layer.deposit(area, x, true);
            self.drop_past(layer, missed, x, velocity.x * 0.5);
        }
    }

    fn emit(&mut self, area: f32, origin: V2, center: V2, velocity: V2) -> f32 {
        if area <= 0.6 || self.grains.len() >= MAX_GRAINS {
            return 0.0;
        }
        let count = ((area / 1.6) as usize + 1).min(4);
        let share = area / count as f32;
        let speed = velocity.length();
        let offset = origin - center;
        let length = offset.length();
        let outward = if length > 0.001 { offset / length } else { V2::new(0.0, 1.0) };

        for _ in 0..count {
            let random = &mut self.random;
            let mut v = velocity * random.range(0.55, 1.25) + outward * (speed * random.range(0.05, 0.25));
            v.y += speed * random.range(0.02, 0.2);
            v += V2::new(random.range(-30.0, 30.0), random.range(-20.0, 40.0));
            self.grains.push(Grain {
                position: origin + V2::new(random.range(-1.0, 1.0), random.range(0.0, 2.0)),
                velocity: v,
                area: share,
                size: (share.sqrt() * 1.05).clamp(1.1, 4.2),
                lump: 0.0,
                blown: false,
            });
        }

        // Add cloud motes (эффект облачков) for the cursor
        let random = &mut self.random;
        if (random.next() < 0.45 || speed > 150.0) && self.motes.len() < MAX_MOTES {
            let mote_v = velocity * random.range(0.25, 0.65)
                + outward * (speed * random.range(0.08, 0.25))
                + V2::new(random.range(-35.0, 35.0), random.range(20.0, 80.0));
            self.motes.push(Mote {
                position: origin + V2::new(random.range(-3.0, 3.0), random.range(-1.0, 4.0)),
                velocity: mote_v,
                size: random.range(7.0, 16.0),
                alpha: random.range(0.08, 0.18),
                age: 0.0,
                life: random.range(0.5, 1.2),
            });
        }

        area
    }

    /// Converts a swept volume of snow into airborne spray (grains + billowing cloud motes),
    /// just like a cursor snowplow, but blown in the direction of the workspace transition.
    fn plow_sweep_snow(&mut self, area: f32, origin: V2, dir: f32, speed: f32) {
        if area <= 0.05 {
            return;
        }
        let grains_to_spawn = ((area / 2.5) as usize).clamp(1, 4);
        let share = area / grains_to_spawn as f32;

        for _ in 0..grains_to_spawn {
            if self.grains.len() >= MAX_GRAINS {
                break;
            }
            let r = &mut self.random;
            let vx = dir * r.range(speed * 0.7, speed * 1.5) + r.range(-30.0, 30.0);
            let vy = r.range(40.0, 280.0) + speed * 0.1 * r.range(0.0, 1.0);
            self.grains.push(Grain {
                position: origin + V2::new(r.range(-2.0, 2.0), r.range(-2.0, 3.0)),
                velocity: V2::new(vx, vy),
                area: share,
                size: (share.sqrt() * 1.1).clamp(1.2, 3.8),
                lump: 0.0,
                blown: true,
            });
        }

        // Billowing cloud motes (эффект облачков)
        let r = &mut self.random;
        let mote_chance = (area / 4.0).clamp(0.25, 0.9);
        if r.next() < mote_chance && self.motes.len() < MAX_MOTES {
            let mote_vx = dir * r.range(speed * 0.35, speed * 0.85) + r.range(-20.0, 20.0);
            let mote_vy = r.range(30.0, 140.0);
            self.motes.push(Mote {
                position: origin + V2::new(r.range(-4.0, 4.0), r.range(0.0, 6.0)),
                velocity: V2::new(mote_vx, mote_vy),
                size: r.range(8.0, 18.0),
                alpha: r.range(0.08, 0.20),
                age: 0.0,
                life: r.range(0.7, 1.5),
            });
        }
    }

    /// Converts snow plowed by a moving window into physical grains AND cloud motes.
    fn emit_window_plow(&mut self, area: f32, origin: V2, velocity: V2) -> f32 {
        if area <= 0.05 {
            return 0.0;
        }
        let speed = velocity.length();
        let count = ((area / 1.6) as usize + 1).clamp(1, 5);
        let share = area / count as f32;

        for _ in 0..count {
            if self.grains.len() >= MAX_GRAINS {
                break;
            }
            let r = &mut self.random;
            let vx = velocity.x * r.range(0.7, 1.4) + r.range(-25.0, 25.0);
            let vy = r.range(35.0, 160.0) + speed * 0.12 * r.range(0.2, 1.0);
            self.grains.push(Grain {
                position: origin + V2::new(r.range(-2.0, 2.0), r.range(-1.0, 3.0)),
                velocity: V2::new(vx, vy),
                area: share,
                size: (share.sqrt() * 1.1).clamp(1.2, 3.8),
                lump: 0.0,
                blown: false,
            });
        }

        let r = &mut self.random;
        if self.motes.len() < MAX_MOTES {
            let mote_vx = velocity.x * r.range(0.3, 0.7) + r.range(-20.0, 20.0);
            let mote_vy = r.range(20.0, 80.0) + speed * 0.08;
            self.motes.push(Mote {
                position: origin + V2::new(r.range(-3.0, 3.0), r.range(0.0, 5.0)),
                velocity: V2::new(mote_vx, mote_vy),
                size: r.range(8.0, 18.0),
                alpha: r.range(0.08, 0.18),
                age: 0.0,
                life: r.range(0.5, 1.2),
            });
        }
        area
    }

    fn advance_motes(&mut self, dt: f32) {
        if self.motes.is_empty() {
            return;
        }
        let damping = (-3.2 * dt).exp();
        let drift = self.wind_speed() * 0.4;
        let mut i = self.motes.len();
        while i > 0 {
            i -= 1;
            let mote = &mut self.motes[i];
            mote.age += dt;
            if mote.age >= mote.life {
                self.motes.swap_remove(i);
            } else {
                mote.velocity.y -= 35.0 * dt;
                mote.velocity *= damping;
                mote.position += (mote.velocity + V2::new(drift, 0.0)) * dt;
            }
        }
    }

    fn make_flake(&mut self, width: f32, y: f32) -> Flake {
        let wind = self.wind_speed();
        let random = &mut self.random;
        let near = random.next() < 0.02;
        let depth = if near { 1.0 } else { random.next().powf(1.6) };
        let base_fall_speed = if near { random.range(75.0, 100.0) } else { 16.0 + depth * 50.0 + random.range(0.0, 10.0) };
        let fall_speed = base_fall_speed * self.speed_factor;
        Flake {
            position: V2::new(random.range(-SIDE_MARGIN, width + SIDE_MARGIN), y),
            velocity: V2::new(wind * (0.35 + 0.65 * depth), -fall_speed),
            fall_speed,
            size: if near { random.range(8.0, 13.0) } else { 1.4 + depth * 4.4 + random.range(0.0, 1.0) },
            depth,
            alpha: if near { 0.28 } else { 0.4 + depth * 0.5 },
            soft: if near { 0.95 } else { 0.75 - depth * 0.45 },
            phase: random.range(0.0, PHASE_WRAP),
            sway_rate: random.range(0.5, 1.4) * (0.5 + 0.5 * self.sway_factor),
            sway_amount: (if near { random.range(10.0, 20.0) } else { random.range(4.0, 9.0) + depth * 10.0 }) * self.sway_factor,
            settles: !near,
            skipped_layer: 0,
            mass: 1.0,
            tint: [1.0; 3],
            tint_amount: 0.0,
            blown: 0.0,
            can_wrap: true,
        }
    }

    fn workspace_wind_speed(&self) -> f32 {
        if self.clock >= self.workspace_wind_until {
            return 0.0;
        }
        let total_time = (self.workspace_wind_until - self.workspace_wind_start) as f32;
        if total_time <= 0.001 {
            return 0.0;
        }
        let elapsed = (self.clock - self.workspace_wind_start) as f32;
        let progress = (elapsed / total_time).clamp(0.0, 1.0);
        let envelope = if progress < 0.15 {
            progress / 0.15
        } else {
            (1.0 - progress) / 0.85
        };
        self.workspace_wind_dir * self.workspace_wind * envelope
    }

    fn wind_speed(&self) -> f32 {
        self.breeze() + self.gust_speed() + self.workspace_wind_speed()
    }

    fn breeze(&self) -> f32 {
        let t = self.clock;
        let natural = (10.0 * (t * 0.07).sin() + 6.0 * (t * 0.19 + 1.7).sin() + 3.0 * (t * 0.43 + 4.1).sin()) as f32;
        let turbulence = if self.static_wind.abs() > 0.01 {
            0.35 * self.static_wind * ((t * 0.23).sin() + 0.4 * (t * 0.53 + 1.2).sin()) as f32
        } else {
            0.0
        };
        (natural + self.static_wind + turbulence * self.sway_factor) * (0.6 + 0.4 * self.strength)
    }

    fn gust_speed(&self) -> f32 {
        let Some(gust) = self.gust.filter(|gust| !gust.calm) else { return 0.0 };
        let age = (self.clock - gust.start) as f32;
        gust.direction * GUST_WIND * smoothstep(0.0, 0.12, age) * (1.0 - smoothstep(0.9, GUST_LENGTH, age))
    }

    fn blowing(&self) -> bool {
        self.gust.is_some_and(|gust| !gust.calm)
    }

    /// A pinch of snow torn off by the gust: a grain of powder, now and then with a faint puff around it.
    fn lift(&mut self, position: V2, direction: f32) {
        if self.grains.len() >= MAX_GRAINS {
            return;
        }
        let random = &mut self.random;
        self.grains.push(Grain {
            position: position + V2::new(random.range(-1.0, 1.0), random.range(-3.0, 0.0)),
            velocity: V2::new(direction * random.range(800.0, 1800.0), random.range(60.0, 420.0)),
            area: 0.0,
            size: random.range(1.1, 2.6),
            lump: 0.0,
            blown: true,
        });
        if random.next() >= 0.35 || self.motes.len() >= MAX_MOTES {
            return;
        }
        self.motes.push(Mote {
            position,
            velocity: V2::new(direction * random.range(200.0, 600.0), random.range(20.0, 120.0)),
            size: random.range(6.0, 14.0),
            alpha: random.range(0.06, 0.14),
            age: 0.0,
            life: random.range(0.6, 1.3),
        });
    }
}

/// Where the gust's front is `age` seconds in, coming in from `margin` past the windward edge of a display `width` wide.
/// It hits at full speed and eases off as it crosses.
fn gust_front(width: f32, age: f32, direction: f32, margin: f32) -> f32 {
    let t = (age / GUST_SWEEP).clamp(0.0, 1.0);
    let travel = (width + 2.0 * margin) * (1.0 - (1.0 - t) * (1.0 - t));
    if direction > 0.0 { -margin + travel } else { width + margin - travel }
}

/// Puts snow that waited off screen back on its window, wherever the window is now, without treating the jump as motion.
fn restore(ledge: &mut SnowLayer, left: f32, top: f32, columns: usize) {
    ledge.resize(columns);
    ledge.origin = V2::new(left, top);
    ledge.track = ledge.origin;
    ledge.track_velocity = V2::ZERO;
    ledge.offset = V2::ZERO;
    ledge.offset_velocity = V2::ZERO;
    ledge.previous_base_y = top;
    ledge.moved_at = -1.0;
}

/// The highest visible snow surface crossed while falling from `from` to `to` at `x`, allowing for the surface moving meanwhile.
fn landing(layers: &[SnowLayer], x: f32, from: f32, to: f32, skipped_layer: usize, windows_solid: bool) -> Option<(usize, usize)> {
    let mut hit = None;
    let mut highest = -f32::MAX;
    if windows_solid {
        for (li, ledge) in layers.iter().enumerate().skip(1) {
            if li == skipped_layer {
                continue;
            }
            let Some(i) = ledge.index(x) else { continue };
            if !ledge.visible[i] {
                continue;
            }
            let surface = ledge.surface(i);
            // Entering the snow from above the window's edge counts, even if the surface rose this frame.
            if from >= ledge.previous_base_y + ledge.bed[i] && to <= surface && surface > highest {
                hit = Some((li, i));
                highest = surface;
            }
        }
        if hit.is_some() {
            return hit;
        }
    }
    let ground = &layers[0];
    let Some(i) = ground.index(x).or_else(|| {
        if x >= -ground.column_width * 0.5 && x < 0.0 {
            Some(0)
        } else if x >= ground.base().x + ground.heights.len() as f32 * ground.column_width
            && x < ground.base().x + ground.heights.len() as f32 * ground.column_width + ground.column_width * 0.5
        {
            Some(ground.heights.len() - 1)
        } else {
            None
        }
    }) else {
        return None;
    };
    (to <= ground.surface(i)).then_some((0, i))
}

/// Where snow leaves an open layer: just past its end, at the height the snow reaches on the end column,
/// which on a rounded corner lies well down the curve.
fn edge_point(layer: &SnowLayer, left: bool) -> V2 {
    let end = if left { 0 } else { layer.count() - 1 };
    let x = if left { layer.base().x - 0.75 } else { layer.base().x + layer.width() + 0.75 };
    V2::new(x, layer.surface(end))
}

/// Critically damped follow of `target` (the SmoothDamp approximation), stable at any time step.
fn smooth_damp(current: &mut V2, velocity: &mut V2, target: V2, time: f32, dt: f32) {
    let omega = 2.0 / time;
    let x = omega * dt;
    let decay = 1.0 / (1.0 + x + 0.48 * x * x + 0.235 * x * x * x);
    let change = *current - target;
    let temp = (*velocity + change * omega) * dt;
    *velocity = (*velocity - temp * omega) * decay;
    *current = target + (change + temp) * decay;
}

fn column_count(width: f32) -> usize {
    (width / COLUMN_WIDTH).ceil() as usize + 1
}
