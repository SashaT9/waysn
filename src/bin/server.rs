use bincode;
use bincode::config::standard;
use std::error::Error;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use wayland_client::Connection;
use waysn::ipc::{IpcCommand, IpcResponse};
use waysn::wayland;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let xdg_runtime_path = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;
    let mut socket_path = PathBuf::from(xdg_runtime_path);
    socket_path.push(format!("{}-waysn.sock", wayland_display));
    if std::fs::metadata(&socket_path).is_ok() {
        if UnixStream::connect(&socket_path).await.is_ok() {
            return Err("server is already running".into());
        }
        std::fs::remove_file(&socket_path)?;
    }
    let socket_path_clone = socket_path.clone();
    let listener = UnixListener::bind(socket_path)?;
    let (wayland_tx, mut wayland_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let wayland_tx = wayland_tx.clone();
                    tokio::spawn(async move {
                        let length = stream.read_u32().await;
                        if let Ok(length) = length {
                            println!("{}", length);
                            let mut buf = vec![0u8; length as usize];
                            if stream.read_exact(&mut buf).await.is_ok() {
                                if let Ok((cmd, _)) =
                                    bincode::decode_from_slice::<IpcCommand, _>(&buf, standard())
                                {
                                    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
                                    println!("{:?}", cmd);
                                    if let Err(e) = wayland_tx.send((cmd, resp_tx)) {
                                        eprintln!("{}", e);
                                    }
                                    match resp_rx.await {
                                        Ok(response) => {
                                            if let Ok(data) =
                                                bincode::encode_to_vec(response, standard())
                                            {
                                                let len = data.len();
                                                stream.write_u32(len as u32).await.unwrap();
                                                stream.write_all(&data).await.unwrap();
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    });
                }
                Err(_) => {}
            }
        }
    });

    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut state = wayland::AppData::new();
    event_queue.roundtrip(&mut state)?;
    event_queue.roundtrip(&mut state)?;

    loop {
        tokio::select! {
            Some((cmd, resp)) = wayland_rx.recv() => {
                match cmd {
                    IpcCommand::SetTemperature { kelvin } => {
                        state.apply_gamma_control_all(kelvin)?;
                    },
                    IpcCommand::GetTemperature { outputs } => {
                        let response = IpcResponse::Temperature { temperatures: state.get_temperatures(outputs) };
                        let _ = resp.send(response);
                    },
                    IpcCommand::Kill {} => {
                        break;
                    }
                }
                event_queue.roundtrip(&mut state)?;
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }
    std::fs::remove_file(&socket_path_clone)?;
    Ok(())
}
