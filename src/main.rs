use std::collections::HashMap;
use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols_wlr::gamma_control::v1::client::zwlr_gamma_control_manager_v1;
use wayland_protocols_wlr::gamma_control::v1::client::zwlr_gamma_control_v1;

struct OutputInfo {
    output: wl_output::WlOutput,
    gamma: Option<zwlr_gamma_control_v1::ZwlrGammaControlV1>,
}
struct AppData {
    outputs: HashMap<u32, OutputInfo>,
    manager: Option<zwlr_gamma_control_manager_v1::ZwlrGammaControlManagerV1>,
}
impl AppData {
    fn new() -> Self {
        let app_data = Self {
            outputs: HashMap::new(),
            manager: None,
        };
        app_data
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
impl Dispatch<zwlr_gamma_control_v1::ZwlrGammaControlV1, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &zwlr_gamma_control_v1::ZwlrGammaControlV1,
        _: zwlr_gamma_control_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    let _registry = display.get_registry(&qh, ());
    let mut state = AppData::new();
    event_queue.roundtrip(&mut state).unwrap();
    if let Some(manager) = &state.manager {
        for (_, output) in state.outputs.iter_mut() {
            let gamma = manager.get_gamma_control(&output.output, &qh, ());
            output.gamma = Some(gamma);
        }
    }
}
