use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    /// Set the temperature in Kelvin (default: 6500)
    #[command(override_usage = "[KELVIN] [OPTIONS]")]
    Set {
        #[arg(default_value_t = 6500)]
        kelvin: u32,
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
                outputs: _,
            } => *provided,
            _ => 0,
        }
    }
}
