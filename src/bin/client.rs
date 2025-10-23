use std::{error::Error, path::PathBuf};

use bincode::config::standard;
use clap::Parser;
use tokio::{io::AsyncWriteExt, net::UnixStream};
use waysn::{args::Args, ipc::IpcCommand};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Args { action } = Args::parse();
    let xdg_runtime_path = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;
    let mut socket_path = PathBuf::from(xdg_runtime_path);
    socket_path.push(format!("{}-waysn.sock", wayland_display));

    let stream = UnixStream::connect(socket_path).await?;
    match action {
        waysn::args::Action::Set { kelvin } => {
            let _ = send_message(IpcCommand::SetTemperature { kelvin: kelvin }, stream).await?;
        }
        waysn::args::Action::Get {  } => {
            let _ = send_message(IpcCommand::GetTemperature {}, stream).await?;
        }
        waysn::args::Action::Kill {} => {
            let _ = send_message(IpcCommand::Kill {}, stream).await?;
        }
    }
    Ok(())
}

async fn send_message(msg: IpcCommand, mut stream: UnixStream) -> Result<(), Box<dyn Error>> {
    let data = bincode::encode_to_vec(&msg, standard())?;
    let length = data.len();
    stream.write_u32(length as u32).await?;
    stream.write_all(&data).await?;
    Ok(())
}
