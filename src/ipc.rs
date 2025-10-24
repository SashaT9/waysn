use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum IpcCommand {
    SetTemperature { kelvin: u32 },
    GetTemperature {},
    Kill {},
}

#[derive(Encode, Decode, Debug)]
pub enum IpcResponse {
    Temperature { kelvin: u32 },
}
