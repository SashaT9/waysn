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
    output_name: String,
    gamma_control: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
    ramp_size: u32,
    current_temperature: u32,
    current_gamma: f32,
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
    pub fn assign_gamma_control_one(&mut self, qh: &QueueHandle<Self>, name: u32) {
        if let Some(manager) = &self.manager
            && let Some(output_info) = self.outputs.get_mut(&name)
        {
            if output_info.gamma_control.is_some() {
                return;
            }
            let gamma_control = manager.get_gamma_control(&output_info.output, qh, name);
            output_info.gamma_control = Some(gamma_control);
        }
    }
    pub fn assign_gamma_control_all(&mut self, qh: &QueueHandle<Self>) {
        if let Some(manager) = &self.manager {
            for (name, output_info) in self.outputs.iter_mut() {
                if output_info.gamma_control.is_some() {
                    continue;
                }
                let gamma_control = manager.get_gamma_control(&output_info.output, qh, *name);
                output_info.gamma_control = Some(gamma_control);
            }
        }
    }
    pub fn apply_gamma_control(
        &mut self,
        names: Vec<String>,
        kelvin: u32,
        gamma: f32,
    ) -> Result<(), Box<dyn Error>> {
        for (_, output_info) in self.outputs.iter_mut() {
            if !names.is_empty() && !names.contains(&output_info.output_name) {
                continue;
            }
            let size = output_info.ramp_size as usize;
            let mut table = vec![0u16; size * 3];
            fill_gamma_table(
                &mut table,
                output_info.ramp_size,
                rgb_from_temperature(kelvin),
                gamma,
            );
            let mut f = tempfile::tempfile()?;
            let byte_slice: &[u8] = bytemuck::cast_slice(&table);
            f.write_all(byte_slice)?;
            f.rewind()?;
            let fd = f.as_fd();
            if let Some(gamma_control) = &output_info.gamma_control {
                gamma_control.set_gamma(fd);
                output_info.current_temperature = kelvin;
                output_info.current_gamma = gamma;
            }
        }
        Ok(())
    }
    pub fn get_temperatures(&mut self, names: Vec<String>) -> HashMap<String, (u32, f32)> {
        let mut result = HashMap::new();
        for (_, output_info) in self.outputs.iter() {
            if names.is_empty() || names.contains(&output_info.output_name) {
                result.insert(
                    output_info.output_name.clone(),
                    (output_info.current_temperature, output_info.current_gamma),
                );
            }
        }
        result
    }
}

impl Default for AppData {
    fn default() -> Self {
        Self::new()
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
        match event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => {
                if interface == wl_output::WlOutput::interface().name {
                    let output =
                        registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, name);
                    state.outputs.insert(
                        name,
                        OutputInfo {
                            output,
                            output_name: String::new(),
                            gamma_control: None,
                            ramp_size: 0,
                            current_temperature: 6600,
                            current_gamma: 1.0,
                        },
                    );
                    state.assign_gamma_control_one(qh, name);
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
                    state.assign_gamma_control_all(qh);
                }
            }
            wl_registry::Event::GlobalRemove { name } => {
                if let Some(output_info) = state.outputs.remove(&name)
                    && let Some(gamma_control) = output_info.gamma_control
                {
                    gamma_control.destroy();
                }
            }
            _ => {}
        }
    }
}
impl Dispatch<wl_output::WlOutput, u32> for AppData {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        idx: &u32,
        _conn: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_output::Event::Name { name } = event
            && let Some(output_info) = state.outputs.get_mut(idx)
        {
            output_info.output_name = name;
        }
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
fn fill_gamma_table(table: &mut [u16], ramp_size: u32, rgb: tempergb::Color, gamma: f32) {
    // println!("r={}, g={}, b={}", rgb.r(), rgb.g(), rgb.b());
    let r = (rgb.r() as u16) | ((rgb.r() as u16) << 8);
    let g = (rgb.g() as u16) | ((rgb.g() as u16) << 8);
    let b = (rgb.b() as u16) | ((rgb.b() as u16) << 8);
    let size = ramp_size as usize;
    for i in 0..size {
        let mut fraction = i as f32 / (ramp_size - 1) as f32;
        fraction = fraction.powf(gamma);
        table[i] = (r as f32 * fraction) as u16;
        table[i + size] = (g as f32 * fraction) as u16;
        table[i + 2 * size] = (b as f32 * fraction) as u16;
    }
}
