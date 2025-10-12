use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub action: Action,
}

#[derive(Subcommand)]
pub enum Action {
    Set { kelvin: u32 },
}

impl Action {
    pub fn get_kelvin(&self) -> u32 {
        match self {
            Self::Set { kelvin: provided } => *provided,
        }
    }
}
