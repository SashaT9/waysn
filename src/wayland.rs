use std::collections::HashMap;
use std::error::Error;
use std::io::{Seek, Write};
use std::os::fd::AsFd;
use tempergb::rgb_from_temperature;
use wayland_client::delegate_noop;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::{wl_output, wl_registry},
};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1,
};
pub struct OutputInfo {
    output: wl_output::WlOutput,
    gamma_control: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
    ramp_size: u32,
}
pub struct AppData {
    outputs: HashMap<u32, OutputInfo>,
    manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
            manager: None,
        }
    }
    pub fn assign_gamma_control_all(&mut self, qh: &QueueHandle<Self>) {
        if let Some(manager) = &self.manager {
            for (idx, output_info) in self.outputs.iter_mut() {
                let gamma_control = manager.get_gamma_control(&output_info.output, &qh, *idx);
                output_info.gamma_control = Some(gamma_control);
            }
        }
    }
    pub fn apply_gamma_control_all(&mut self, kelvin: u32) -> Result<(), Box<dyn Error>> {
        for (_, output) in self.outputs.iter_mut() {
            let size = output.ramp_size as usize;
            let mut table = vec![0u16; size * 3];
            fill_gamma_table(&mut table, output.ramp_size, rgb_from_temperature(kelvin));
            let mut f = tempfile::tempfile()?;
            let byte_slice: &[u8] = bytemuck::cast_slice(&table);
            f.write_all(byte_slice)?;
            f.rewind()?;
            let fd = f.as_fd();
            if let Some(gamma_control) = &output.gamma_control {
                gamma_control.set_gamma(fd);
            }
        }
        Ok(())
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
            if interface == wl_output::WlOutput::interface().name {
                let output = registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, name);
                state.outputs.insert(
                    name,
                    OutputInfo {
                        output: output,
                        gamma_control: None,
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

delegate_noop!(AppData: ignore zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1);

impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, u32> for AppData {
    fn event(
        state: &mut Self,
        gamma_control: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        event: zwlr_gamma_control_v1::Event,
        idx: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_gamma_control_v1::Event::GammaSize { size } => {
                if let Some(output_info) = state.outputs.get_mut(idx) {
                    output_info.ramp_size = size;
                }
            }
            zwlr_gamma_control_v1::Event::Failed => {
                eprintln!("gamma control is no longer valid");
                gamma_control.destroy();
            }
            _ => {}
        }
    }
}
pub fn fill_gamma_table(table: &mut [u16], ramp_size: u32, rgb: tempergb::Color) {
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
