use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum IpcCommand {
    SetTemperature { kelvin: u32, outputs: Vec<String> },
    GetTemperature { outputs: Vec<String> },
    Kill {},
}

#[derive(Encode, Decode, Debug)]
pub enum IpcResponse {
    Temperature { temperatures: Vec<(String, u32)> },
    Ok,
    Err { message: String },
}
