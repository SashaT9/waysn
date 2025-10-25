use clap::{Parser, Subcommand, value_parser};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
    /// Output in JSON format
    #[arg(short, long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Action {
    /// Set the temperature in Kelvin
    #[command(override_usage = "[KELVIN] [OPTIONS]")]
    Set {
        // The temperature in Kelvin
        #[arg(default_value_t = 6600, value_parser = value_parser!(u32).range(1000..=10000))]
        kelvin: u32,
        // The gamma correction value
        #[arg(short, long, default_value_t = 1.0, value_parser = |s: &str| {
                    let val: f32 = s.parse().map_err(|_| String::from("must be a number"))?;
                    if (0.5..=3.0).contains(&val) {
                        Ok(val)
                    } else {
                        Err(String::from("must be between 0.5 and 3.0"))
                    }
                })]
        gamma: f32,
        /// The names of the outputs (e.g eDP-1)
        #[arg(short, long, num_args(1..))]
        outputs: Vec<String>,
    },
    /// Get the temperature in Kelvin
    Get {
        /// The names of the outputs (e.g eDP-1)
        outputs: Vec<String>,
    },
    /// Kills the daemon
    Kill {},
}

impl Action {
    pub fn get_kelvin(&self) -> u32 {
        match self {
            Self::Set {
                kelvin: provided,
                gamma: _,
                outputs: _,
            } => *provided,
            _ => 0,
        }
    }
    pub fn get_gamma(&self) -> f32 {
        match self {
            Self::Set {
                kelvin: _,
                gamma: provided,
                outputs: _,
            } => *provided,
            _ => 1_f32,
        }
    }
}
