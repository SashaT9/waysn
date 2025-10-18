mod args;
use args::Args;
use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::io::{Seek, Write};
use std::os::fd::AsFd;
use tempergb::rgb_from_temperature;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::{wl_output, wl_registry},
};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1,
};

struct OutputInfo {
    output: wl_output::WlOutput,
    gamma: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
    ramp_size: u32,
}
struct AppData {
    outputs: HashMap<u32, OutputInfo>,
    manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
}
impl AppData {
    fn new() -> Self {
        Self {
            outputs: HashMap::new(),
            manager: None,
        }
    }
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            // println!("[{}] {} (v{})", name, interface, version);
            if interface == wl_output::WlOutput::interface().name {
                let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, name);
                state.outputs.insert(
                    name,
                    OutputInfo {
                        output: output,
                        gamma: None,
                        ramp_size: 0,
                    },
                );
            }
            if interface
                == zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1::interface().name
            {
                let manager = registry
                    .bind::<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, _, _>(
                        name,
                        version,
                        qh,
                        (),
                    );
                state.manager = Some(manager);
            }
        }
    }
}
impl Dispatch<wl_output::WlOutput, u32> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        _event: wl_output::Event,
        _idx: &u32,
        _conn: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

// actually not used. Written so that bind does not complain
impl Dispatch<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1,
        _: zwlr_gamma_control_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, u32> for AppData {
    fn event(
        state: &mut Self,
        _: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        event: zwlr_gamma_control_v1::Event,
        idx: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let zwlr_gamma_control_v1::Event::GammaSize { size } = event {
            if let Some(output_info) = state.outputs.get_mut(idx) {
                output_info.ramp_size = size;
                println!("{}", size);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let Args { action } = Args::parse();
    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut state = AppData::new();
    event_queue.roundtrip(&mut state)?;
    if let Some(manager) = &state.manager {
        for (idx, output) in state.outputs.iter_mut() {
            let gamma = manager.get_gamma_control(&output.output, &qh, *idx);
            output.gamma = Some(gamma);
        }
    }
    event_queue.roundtrip(&mut state)?;
    let mut temp_files = Vec::new();
    for (_idx, output) in state.outputs.iter_mut() {
        let size = output.ramp_size as usize;
        let mut table = vec![0u16; size * 3];
        fill_gamma_table(
            &mut table,
            output.ramp_size,
            rgb_from_temperature(action.get_kelvin()),
        );
        let mut f = tempfile::tempfile()?;
        let byte_slice: &[u8] = bytemuck::cast_slice(&table);
        f.write_all(byte_slice)?;
        f.rewind()?;
        let fd = f.as_fd();
        if let Some(gamma) = &output.gamma {
            gamma.set_gamma(fd);
        }
        temp_files.push(f);
    }
    conn.flush()?;
    loop {
        event_queue.blocking_dispatch(&mut state)?;
    }
}

fn fill_gamma_table(table: &mut [u16], ramp_size: u32, rgb: tempergb::Color) {
    let r_16bit = rgb.r() as u16 * 257;
    let g_16bit = rgb.g() as u16 * 257;
    let b_16bit = rgb.b() as u16 * 257;
    let size = ramp_size as usize;
    for i in 0..size {
        let fraction = i as f32 / (ramp_size - 1) as f32;
        table[i] = (r_16bit as f32 * fraction) as u16;
        table[i + size] = (g_16bit as f32 * fraction) as u16;
        table[i + 2 * size] = (b_16bit as f32 * fraction) as u16;
    }
}
