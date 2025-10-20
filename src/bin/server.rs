use args::Args;
use bincode;
use bincode::config::standard;
use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use wayland_client::Connection;
use waysn::args;
use waysn::ipc::IpcCommand;
use waysn::wayland;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let xdg_runtime_path = std::env::var("XDG_RUNTIME_DIR")?;
    let wayland_display = std::env::var("WAYLAND_DISPLAY")?;
    let mut socket_path = PathBuf::from(xdg_runtime_path);
    socket_path.push(format!("{}-waysn.sock", wayland_display));
    if std::fs::metadata(&socket_path).is_ok() {
        std::fs::remove_file(&socket_path)?;
    }
    let listener = UnixListener::bind(socket_path)?;
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = Vec::new();
            if stream.read_to_end(&mut buf).await.is_ok() {
                if let Ok(cmd) = bincode::decode_from_slice::<IpcCommand, _>(&buf, standard()) {
                    println!("{:?}", cmd);
                }
            }
        });
    }

    /*
    let Args { action } = Args::parse();
    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut state = wayland::AppData::new();
    event_queue.roundtrip(&mut state)?;
    state.assign_gamma_control_all(&qh);
    event_queue.roundtrip(&mut state)?;
    state.apply_gamma_control_all(action.get_kelvin())?;
    conn.flush()?;
    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
    */
}
