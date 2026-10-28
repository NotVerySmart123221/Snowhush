mod render;
mod tracker;
mod wayland;

use snowhush::simulation::Snowfall;
use wayland::{AppConfig, SnowhushApp};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut config = AppConfig::default();
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
                println!("  -h, --help                Print help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    println!(
        "Starting Snowhush Wayland (snowfall: {:?}, top_margin: {}, layers: {}, melt: {} (x{}))...",
        config.snowfall, config.top_margin, config.track_layers, config.melt, config.melt_rate
    );
    SnowhushApp::run(config)?;

    Ok(())
}
