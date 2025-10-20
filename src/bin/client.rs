use std::{error::Error, path::PathBuf};

use bincode::config::standard;
use tokio::{io::AsyncWriteExt, net::UnixStream};
use waysn::ipc::IpcCommand;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let xdg_runtime_path = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;
    let mut socket_path = PathBuf::from(xdg_runtime_path);
    socket_path.push(format!("{}-waysn.sock", wayland_display));

    let mut stream = UnixStream::connect(socket_path).await?;
    let cmd = IpcCommand::SetTemperature { kelvin: 4000 };
    let data = bincode::encode_to_vec(&cmd, standard())?;
    stream.write_all(&data).await?;
    Ok(())
}
