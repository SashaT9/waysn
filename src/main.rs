mod args;
mod wayland;
use args::Args;
use clap::Parser;
use std::error::Error;
use std::io::{Seek, Write};
use std::os::fd::AsFd;
use tempergb::rgb_from_temperature;
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
    if let Some(manager) = &state.manager {
        for (idx, output) in state.outputs.iter_mut() {
            let gamma_control = manager.get_gamma_control(&output.output, &qh, *idx);
            output.gamma_control = Some(gamma_control);
        }
    }
    event_queue.roundtrip(&mut state)?;
    let mut temp_files = Vec::new();
    for (_idx, output) in state.outputs.iter_mut() {
        let size = output.ramp_size as usize;
        let mut table = vec![0u16; size * 3];
        wayland::fill_gamma_table(
            &mut table,
            output.ramp_size,
            rgb_from_temperature(action.get_kelvin()),
        );
        let mut f = tempfile::tempfile()?;
        let byte_slice: &[u8] = bytemuck::cast_slice(&table);
        f.write_all(byte_slice)?;
        f.rewind()?;
        let fd = f.as_fd();
        if let Some(gamma_control) = &output.gamma_control {
            gamma_control.set_gamma(fd);
        }
        temp_files.push(f);
    }
    conn.flush()?;
    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}
