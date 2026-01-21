use std::path::PathBuf;

use anyhow::{Context, Result};

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
async fn main() -> Result<()> {
    let Args { action, json } = Args::parse();
    let xdg_runtime_path = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;
    let mut socket_path = PathBuf::from(xdg_runtime_path);
    socket_path.push(format!("{}-waysn.sock", wayland_display));

    let mut stream = UnixStream::connect(socket_path)
        .await
        .with_context(|| "Couldn't connect to the daemon, check if waysn-daemon is running")?;
    match action {
        waysn::args::Action::Set {
            kelvin,
            gamma,
            outputs,
        } => {
            send_message(
                IpcCommand::SetTemperature {
                    kelvin,
                    gamma,
                    outputs,
                },
                &mut stream,
            )
            .await?;
        }
        waysn::args::Action::Get { outputs } => {
            send_message(IpcCommand::GetTemperature { outputs }, &mut stream).await?;
        }
        waysn::args::Action::Kill {} => {
            send_message(IpcCommand::Kill {}, &mut stream).await?;
        }
    }
    let length = stream.read_u32().await?;
    let mut buf = vec![0u8; length as usize];
    stream.read_exact(&mut buf).await?;
    let (response, _) = bincode::decode_from_slice::<IpcResponse, _>(&buf, standard())?;
    if json {
        if let Ok(json_response) = match response {
            IpcResponse::Temperature { temperatures } => serde_json::to_string(&temperatures),
            _ => serde_json::to_string(&response),
        } {
            println!("{}", json_response);
        }
    } else {
        print!("{}", response);
    }
    Ok(())
}

async fn send_message(msg: IpcCommand, stream: &mut UnixStream) -> Result<()> {
    let data = bincode::encode_to_vec(&msg, standard())?;
    let length = data.len();
    stream.write_u32(length as u32).await?;
    stream.write_all(&data).await?;
    Ok(())
}
