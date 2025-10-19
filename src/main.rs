mod args;
mod wayland;
use args::Args;
use clap::Parser;
use std::error::Error;
use wayland_client::Connection;

fn main() -> Result<(), Box<dyn Error>> {
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
}
