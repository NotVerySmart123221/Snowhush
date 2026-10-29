mod render;
mod tracker;
mod wayland;

use snowhush::simulation::Snowfall;
use wayland::{AppConfig, SnowhushApp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut config = if let Some((file_cfg, _)) = wayland::load_config_file() {
        file_cfg
    } else {
        AppConfig::default()
    };
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--snowfall" | "-s" => {
                if i + 1 < args.len() {
                    config.snowfall = match args[i + 1].to_lowercase().as_str() {
                        "off" => Snowfall::Off,
                        "light" => Snowfall::Light,
                        "steady" => Snowfall::Steady,
                        "heavy" => Snowfall::Heavy,
                        "variable" => Snowfall::Variable,
                        other => {
                            eprintln!("Unknown snowfall rate '{}', defaulting to steady", other);
                            Snowfall::Steady
                        }
                    };
                    i += 1;
                }
            }
            "--top-margin" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.top_margin = val;
                    }
                    i += 1;
                }
            }
            "--no-layers" => {
                config.track_layers = false;
            }
            "--sweep-ground" => {
                config.sweep_ground = true;
            }
            "--no-melt" => {
                config.melt = false;
            }
            "--melt-rate" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.melt_rate = val;
                    }
                    i += 1;
                }
            }
            "--no-wind-accel" => {
                config.wind_accel = false;
            }
            "--wind-factor" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.wind_factor = val;
                    }
                    i += 1;
                }
            }
            "--static-wind" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.static_wind = val;
                    }
                    i += 1;
                }
            }
            "--window-leak" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.window_leak = val.clamp(0.0, 1.0);
                    }
                    i += 1;
                }
            }
            "--motion-shed" | "--motion-sensitivity" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.motion_shed = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--cursor-force" | "--cursor-factor" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.cursor_force = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--density" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.density = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--speed" | "--speed-factor" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.speed_factor = val.clamp(0.1, 5.0);
                    }
                    i += 1;
                }
            }
            "--sway" | "--sway-factor" | "--turbulence" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.sway_factor = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--cloud-density" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.cloud_density = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--cloud-amount" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.cloud_amount = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--cloud-size" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.cloud_size = val.max(0.1);
                    }
                    i += 1;
                }
            }
            "--spray-density" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.spray_density = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--spray-amount" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.spray_amount = val.max(0.1);
                    }
                    i += 1;
                }
            }
            "--spray-size" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.spray_size = val.max(0.1);
                    }
                    i += 1;
                }
            }
            "--spray-force" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.spray_force = val.max(0.1);
                    }
                    i += 1;
                }
            }
            "--window-wake" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.window_wake = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--motion-blur" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.motion_blur = val.max(0.0);
                    }
                    i += 1;
                }
            }
            "--no-fullscreen-snow" => {
                config.no_fullscreen_snow = true;
            }
            "--grain-speed" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.grain_speed = val.clamp(0.1, 5.0);
                    }
                    i += 1;
                }
            }
            "--grain-gravity" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.grain_gravity = val.clamp(0.05, 5.0);
                    }
                    i += 1;
                }
            }
            "--dynamic-weather" => {
                config.dynamic_weather = true;
            }
            "--weather-mode" => {
                if i + 1 < args.len() {
                    config.weather_mode = args[i + 1].clone();
                    i += 1;
                }
            }
            "--weather-speed" => {
                if i + 1 < args.len() {
                    if let Ok(val) = args[i + 1].parse::<f32>() {
                        config.weather_cycle_speed = val.clamp(0.1, 5.0);
                    }
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("Snowhush for Wayland (Hyprland, Niri & generic)");
                println!("Usage: snowhush-wayland [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -s, --snowfall <RATE>     Snowfall intensity: off, light, steady, heavy, variable [default: steady]");
                println!("      --top-margin <PX>     Minimum distance from screen top for ledges to catch snow [default: 48]");
                println!("      --no-layers           Disable tracking layer-shell surfaces (e.g. Quickshell, docks)");
                println!("      --sweep-ground        Also sweep ground snow during workspace switch transitions");
                println!("      --no-melt             Disable snow thawing and melting over time");
                println!("      --melt-rate <FLOAT>   Snow thaw rate multiplier [default: 1.0]");
                println!("      --no-wind-accel       Disable airborne snowflake acceleration on workspace switch");
                println!("      --wind-factor <FLOAT> Wind gust strength multiplier [default: 1.0]");
                println!("      --static-wind <FLOAT> Ambient horizontal wind speed in px/s [default: 20.0]");
                println!("      --window-leak <FLOAT> Fraction of snowflakes that pass windows to ground [0.0-1.0, default: 0.35]");
                println!("      --motion-shed <FLOAT> Window motion snow shedding sensitivity [default: 1.0]");
                println!("      --cursor-force <FLOAT> Cursor displacement and wind force on snowflakes [default: 1.0]");
                println!("      --window-wake <FLOAT> Moving window aerodynamic wake on airborne snow [default: 1.0]");
                println!("      --motion-blur <FLOAT> Snowflake motion streak blur multiplier [default: 1.0]");
                println!("      --no-fullscreen-snow  Disable snow accumulation while any window is fullscreen");
                println!("      --grain-speed <FLOAT> Physical snow grain velocity multiplier [default: 1.0]");
                println!("      --grain-gravity <FLOAT> Physical snow grain gravity multiplier [default: 1.0]");
                println!("      --dynamic-weather     Enable dynamic weather cycles (wind, speed, density)");
                println!("      --weather-mode <MODE> Weather mode: auto, calm, breeze, blizzard, squall, thaw [default: auto]");
                println!("      --weather-speed <FLOAT> Dynamic weather cycle speed multiplier [default: 1.0]");
                println!("  -h, --help                Print help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    println!(
        "Starting Snowhush Wayland (snowfall: {:?}, top_margin: {}, layers: {}, melt: {} (x{}), wind: {} (x{}), static_wind: {}, leak: {}, motion_shed: {}, cursor: {})...",
        config.snowfall, config.top_margin, config.track_layers, config.melt, config.melt_rate, config.wind_accel, config.wind_factor, config.static_wind, config.window_leak, config.motion_shed, config.cursor_force
    );
    SnowhushApp::run(config)?;

    Ok(())
}
