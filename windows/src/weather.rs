use std::str::FromStr;
use crate::simulation::Random;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherKind {
    Calm,
    Breeze,
    Blizzard,
    Squall,
    Thaw,
}

impl FromStr for WeatherKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "calm" => Ok(WeatherKind::Calm),
            "breeze" => Ok(WeatherKind::Breeze),
            "blizzard" => Ok(WeatherKind::Blizzard),
            "squall" => Ok(WeatherKind::Squall),
            "thaw" => Ok(WeatherKind::Thaw),
            _ => Err(()),
        }
    }
}

impl WeatherKind {
    pub fn name(&self) -> &'static str {
        match self {
            WeatherKind::Calm => "Calm Flurries",
            WeatherKind::Breeze => "Rising Breeze",
            WeatherKind::Blizzard => "Blizzard / Snowstorm",
            WeatherKind::Squall => "Icy Squall",
            WeatherKind::Thaw => "Gentle Thaw",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    pub fn targets(&self, random: &mut Random) -> WeatherParams {
        match self {
            WeatherKind::Calm => WeatherParams {
                wind: random.range(-8.0, 8.0),
                speed: random.range(0.65, 0.85),
                density: random.range(0.7, 1.0),
                sway: random.range(0.4, 0.7),
            },
            WeatherKind::Breeze => {
                let dir = if random.next() > 0.5 { 1.0 } else { -1.0 };
                WeatherParams {
                    wind: dir * random.range(35.0, 65.0),
                    speed: random.range(1.0, 1.3),
                    density: random.range(1.1, 1.4),
                    sway: random.range(0.9, 1.4),
                }
            }
            WeatherKind::Blizzard => {
                let dir = if random.next() > 0.5 { 1.0 } else { -1.0 };
                WeatherParams {
                    wind: dir * random.range(85.0, 145.0),
                    speed: random.range(1.4, 1.9),
                    density: random.range(1.7, 2.5),
                    sway: random.range(1.8, 2.6),
                }
            }
            WeatherKind::Squall => {
                let dir = if random.next() > 0.5 { 1.0 } else { -1.0 };
                WeatherParams {
                    wind: dir * random.range(50.0, 110.0),
                    speed: random.range(1.5, 2.1),
                    density: random.range(1.3, 1.9),
                    sway: random.range(1.3, 1.8),
                }
            }
            WeatherKind::Thaw => WeatherParams {
                wind: random.range(-15.0, 15.0),
                speed: random.range(0.5, 0.7),
                density: random.range(0.3, 0.6),
                sway: random.range(0.6, 0.9),
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WeatherParams {
    pub wind: f32,
    pub speed: f32,
    pub density: f32,
    pub sway: f32,
}

#[derive(Clone, Debug)]
pub struct WeatherSystem {
    pub enabled: bool,
    pub mode: String,
    pub current_kind: WeatherKind,
    pub target_kind: WeatherKind,
    pub cycle_speed: f32,
    pub phase_timer: f64,
    pub phase_duration: f64,
    pub current_params: WeatherParams,
    pub target_params: WeatherParams,
    pub output_wind: f32,
    pub output_speed: f32,
    pub output_density: f32,
    pub output_sway: f32,
}

impl Default for WeatherSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl WeatherSystem {
    pub fn new() -> Self {
        let initial_kind = WeatherKind::Calm;
        let mut r = Random::new();
        let initial_params = initial_kind.targets(&mut r);
        Self {
            enabled: false,
            mode: "auto".to_string(),
            current_kind: initial_kind,
            target_kind: initial_kind,
            cycle_speed: 1.0,
            phase_timer: 0.0,
            phase_duration: 45.0,
            current_params: initial_params,
            target_params: initial_params,
            output_wind: initial_params.wind,
            output_speed: initial_params.speed,
            output_density: initial_params.density,
            output_sway: initial_params.sway,
        }
    }

    pub fn set_mode(&mut self, mode: &str, random: &mut Random) {
        self.mode = mode.to_string();
        if let Some(kind) = WeatherKind::from_str(mode) {
            self.target_kind = kind;
            self.target_params = kind.targets(random);
            self.phase_timer = 0.0;
        }
    }

    pub fn step(&mut self, dt: f32, clock: f64, random: &mut Random) {
        if !self.enabled {
            return;
        }

        self.phase_timer += dt as f64;

        if self.mode == "auto" && self.phase_timer >= self.phase_duration {
            self.phase_timer = 0.0;
            let duration_base = random.range(30.0, 75.0) as f64;
            self.phase_duration = (duration_base / self.cycle_speed.max(0.1) as f64).clamp(10.0, 300.0);

            self.current_kind = self.target_kind;
            self.target_kind = match self.current_kind {
                WeatherKind::Calm => if random.next() > 0.4 { WeatherKind::Breeze } else { WeatherKind::Thaw },
                WeatherKind::Breeze => if random.next() > 0.4 { WeatherKind::Blizzard } else { WeatherKind::Squall },
                WeatherKind::Blizzard => if random.next() > 0.5 { WeatherKind::Squall } else { WeatherKind::Calm },
                WeatherKind::Squall => if random.next() > 0.5 { WeatherKind::Calm } else { WeatherKind::Breeze },
                WeatherKind::Thaw => WeatherKind::Calm,
            };
            self.target_params = self.target_kind.targets(random);
        }

        // Smooth transition rate
        let rate = (dt / 8.0).min(0.25);
        self.current_params.wind += (self.target_params.wind - self.current_params.wind) * rate;
        self.current_params.speed += (self.target_params.speed - self.current_params.speed) * rate;
        self.current_params.density += (self.target_params.density - self.current_params.density) * rate;
        self.current_params.sway += (self.target_params.sway - self.current_params.sway) * rate;

        // Organic micro-gusting turbulence on top of base wind
        let gust_wave = ((clock * 0.35).sin() * 12.0 + (clock * 0.82).cos() * 6.0) as f32;
        self.output_wind = self.current_params.wind + gust_wave;
        self.output_speed = self.current_params.speed;
        self.output_density = self.current_params.density;
        self.output_sway = self.current_params.sway;
    }
}
