use std::{error::Error, path::PathBuf};

use bincode::config::standard;
use clap::Parser;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use waysn::{
    args::Args,
    ipc::{IpcCommand, IpcResponse},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Args { action } = Args::parse();
    let xdg_runtime_path = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;
    let mut socket_path = PathBuf::from(xdg_runtime_path);
    socket_path.push(format!("{}-waysn.sock", wayland_display));

    let mut stream = UnixStream::connect(socket_path).await?;
    match action {
        waysn::args::Action::Set { kelvin } => {
            let _ =
                send_message(IpcCommand::SetTemperature { kelvin: kelvin }, &mut stream).await?;
        }
        waysn::args::Action::Get { outputs } => {
            let _ =
                send_message(IpcCommand::GetTemperature { outputs: outputs }, &mut stream).await?;
            let length = stream.read_u32().await;
            if let Ok(length) = length {
                let mut buf = vec![0u8; length as usize];
                if stream.read_exact(&mut buf).await.is_ok() {
                    if let Ok(response) =
                        bincode::decode_from_slice::<IpcResponse, _>(&buf, standard())
                    {
                        println!("{:?}", response);
                    }
                }
            }
        }
        waysn::args::Action::Kill {} => {
            let _ = send_message(IpcCommand::Kill {}, &mut stream).await?;
        }
    }
    Ok(())
}

async fn send_message(msg: IpcCommand, stream: &mut UnixStream) -> Result<(), Box<dyn Error>> {
    let data = bincode::encode_to_vec(&msg, standard())?;
    let length = data.len();
    stream.write_u32(length as u32).await?;
    stream.write_all(&data).await?;
    Ok(())
}
