use wayland_client::{
    protocol::{wl_output, wl_registry},
    Connection, Dispatch, QueueHandle,
};
struct AppData;

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        _state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<AppData>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            // println!("[{}] {} (v{})", name, interface, version);
            if interface == "wl_output" {
                registry.bind::<wl_output::WlOutput, _, _>(name, version, qh, ());
            }
        }
    }
}
impl Dispatch<wl_output::WlOutput, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &wl_output::WlOutput,
        event: wl_output::Event,
        _: &(),
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
                println!("{}x{} @{}Hz", width, height, refresh / 1000);
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
    event_queue.roundtrip(&mut AppData).unwrap();
    event_queue.roundtrip(&mut AppData).unwrap();
}
