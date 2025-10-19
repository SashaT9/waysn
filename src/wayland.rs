use std::collections::HashMap;
use wayland_client::delegate_noop;
use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    protocol::{wl_output, wl_registry},
};
use wayland_protocols_wlr::gamma_control::v1::client::{
    zwlr_gamma_control_manager_v1, zwlr_gamma_control_v1,
};
pub struct OutputInfo {
    pub output: wl_output::WlOutput,
    pub gamma_control: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
    pub ramp_size: u32,
}
pub struct AppData {
    pub outputs: HashMap<u32, OutputInfo>,
    pub manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
}

impl AppData {
    pub fn new() -> Self {
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
