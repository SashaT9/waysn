use bincode::config::standard;
use std::error::Error;
use std::os::fd::AsFd;
use std::path::PathBuf;
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use wayland_client::Connection;
use wayland_client::backend::WaylandError;
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
            if let Ok((stream, _)) = listener.accept().await {
                let wayland_tx = wayland_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, wayland_tx).await {
                        eprintln!("{}", e);
                    }
                });
            }
        }
    });

    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let wayland_fd = AsyncFd::new(conn.as_fd())?;
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
                    IpcCommand::SetTemperature { kelvin, outputs } => {
                        let _ = match state.apply_gamma_control(outputs, kelvin) {
                            Ok(_) => resp.send(IpcResponse::Ok),
                            Err(e)  => resp.send(IpcResponse::Err { message: e.to_string() }),
                        };
                    },
                    IpcCommand::GetTemperature { outputs } => {
                        let _ = resp.send(IpcResponse::Temperature { temperatures: state.get_temperatures(outputs) });
                    },
                    IpcCommand::Kill {} => {
                        let _ = resp.send(IpcResponse::Ok);
                        break;
                    }
                }
                event_queue.flush()?;
            },
            Ok(mut guard) = wayland_fd.readable() => {
                if let Some(read_guard) = event_queue.prepare_read() {
                    match read_guard.read() {
                        Ok(_) => {
                            event_queue.dispatch_pending(&mut state)?;
                        }
                        Err(WaylandError::Io(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // not fatal. https://doc.rust-lang.org/std/io/enum.ErrorKind.html#variant.WouldBlock
                        }
                        Err(e) => return Err(e.into())
                    }
                } else {
                    event_queue.dispatch_pending(&mut state)?;
                }
                guard.clear_ready();
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }
    std::fs::remove_file(&socket_path_clone)?;
    Ok(())
}

async fn handle_connection(
    mut stream: UnixStream,
    wayland_tx: tokio::sync::mpsc::UnboundedSender<(
        IpcCommand,
        tokio::sync::oneshot::Sender<IpcResponse>,
    )>,
) -> Result<(), Box<dyn Error>> {
    let length = stream.read_u32().await?;
    println!("{}", length);
    let mut buf = vec![0u8; length as usize];
    stream.read_exact(&mut buf).await?;
    let (cmd, _) = bincode::decode_from_slice::<IpcCommand, _>(&buf, standard())?;
    let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
    println!("{:?}", cmd);
    wayland_tx.send((cmd, resp_tx))?;
    let response = resp_rx.await?;
    let data = bincode::encode_to_vec(response, standard())?;
    let len = data.len();
    stream.write_u32(len as u32).await?;
    stream.write_all(&data).await?;
    Ok(())
}
