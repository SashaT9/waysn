use std::collections::HashMap;
use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, Proxy, QueueHandle,
};

struct OutputInfo {
    output: wl_output::WlOutput,
    width: i32,
    height: i32,
    refresh: i32,
}
struct AppData {
    outputs: HashMap<u32, OutputInfo>,
}
impl AppData {
    fn new() -> Self {
        let app_data = Self {
            outputs: HashMap::new(),
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
                        width: 0,
                        height: 0,
                        refresh: 0,
                    },
                );
            }
        }
    }
}
impl Dispatch<wl_output::WlOutput, u32> for AppData {
    fn event(
        state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        idx: &u32,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_output::Event::Geometry { make, model, .. } => {
                println!("make={} model={}", make, model);
            }
            wl_output::Event::Mode {
                width,
                height,
                refresh,
                ..
            } => {
                let Some(output) = state.outputs.get_mut(idx) else {
                    todo!();
                };
                output.width = width;
                output.height = height;
                output.refresh = refresh / 1000;
            }
            _ => {}
        }
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
    event_queue.roundtrip(&mut state).unwrap();
    for (_idx, output) in state.outputs.into_iter() {
        println!("{}x{} @{}Hz", output.width, output.height, output.refresh);
    }
}
