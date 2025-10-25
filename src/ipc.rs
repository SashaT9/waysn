use std::{collections::HashMap, fmt};

use bincode::{Decode, Encode};
use serde::Serialize;

#[derive(Encode, Decode, Debug)]
pub enum IpcCommand {
    SetTemperature { kelvin: u32, outputs: Vec<String> },
    GetTemperature { outputs: Vec<String> },
    Kill {},
}

#[derive(Encode, Decode, Debug, Serialize)]
pub enum IpcResponse {
    Temperature { temperatures: HashMap<String, u32> },
    Ok,
    Err { message: String },
}

impl fmt::Display for IpcResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpcResponse::Temperature { temperatures } => {
                for (name, kelvin) in temperatures {
                    writeln!(f, "{}: {}K", name, kelvin)?;
                }
                Ok(())
            }
            IpcResponse::Ok => writeln!(f, "Ok"),
            IpcResponse::Err { message } => writeln!(f, "Error: {}", message),
        }
    }
}
