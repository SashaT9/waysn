use bincode;
use bincode::config::standard;
use std::error::Error;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use wayland_client::Connection;
use waysn::ipc::IpcCommand;
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
    let listener = UnixListener::bind(socket_path)?;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.unwrap();
            let tx = tx.clone();
            tokio::spawn(async move {
                let length = stream.read_u32().await;
                if let Ok(length) = length {
                    println!("{}", length);
                    let mut buf = vec![0u8; length as usize];
                    if stream.read_exact(&mut buf).await.is_ok() {
                        if let Ok(cmd) =
                            bincode::decode_from_slice::<IpcCommand, _>(&buf, standard())
                        {
                            println!("{:?}", cmd);
                            if let Err(e) = tx.send(cmd) {
                                eprintln!("{}", e);
                            }
                        }
                    }
                }
            });
        }
    });

    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut state = wayland::AppData::new();
    event_queue.roundtrip(&mut state)?;
    state.assign_gamma_control_all(&qh);
    event_queue.roundtrip(&mut state)?;

    loop {
        tokio::select! {
            Some((cmd, _)) = rx.recv() => {
                if let IpcCommand::SetTemperature { kelvin } = cmd {
                    state.apply_gamma_control_all(kelvin)?;
                }
                conn.flush()?;
                event_queue.roundtrip(&mut state)?;
            }
        }
    }
}
