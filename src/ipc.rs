use bincode::{Decode, Encode};

#[derive(Encode, Decode, Debug)]
pub enum IpcCommand {
    SetTemperature { kelvin: u32 },
    Kill {},
}
