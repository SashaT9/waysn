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
    Set {
        #[arg(default_value_t = 6500)]
        kelvin: u32,
    },
}

impl Action {
    pub fn get_kelvin(&self) -> u32 {
        match self {
            Self::Set { kelvin: provided } => *provided,
        }
    }
}
