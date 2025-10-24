use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum IpcCommand {
    SetTemperature { kelvin: u32 },
    GetTemperature { outputs: Vec<String> },
    Kill {},
}

#[derive(Encode, Decode, Debug)]
pub enum IpcResponse {
    Temperature { temperatures: Vec<(String, u32)> },
}
