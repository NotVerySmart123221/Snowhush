mod render;
mod tracker;
mod wayland;

use snowhush::simulation::Snowfall;
use wayland::SnowhushApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let mut snowfall = Snowfall::Steady;
    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--snowfall" | "-s" => {
                if i + 1 < args.len() {
                    snowfall = match args[i + 1].to_lowercase().as_str() {
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
            "--help" | "-h" => {
                println!("Snowhush for Wayland (Hyprland, Niri & generic)");
                println!("Usage: snowhush-wayland [OPTIONS]");
                println!();
                println!("Options:");
                println!("  -s, --snowfall <RATE>   Snowfall intensity: off, light, steady, heavy, variable [default: steady]");
                println!("  -h, --help              Print help");
                return Ok(());
            }
            _ => {}
        }
        i += 1;
    }

    println!("Starting Snowhush Wayland (snowfall: {:?})...", snowfall);
    SnowhushApp::run(snowfall)?;

    Ok(())
}
