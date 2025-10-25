# waysn
Waysn is a command-line tool for adjusting screen gamma on Wayland compositors that support the `wlr-gamma-control-unstable-v1` protocol. It allows you to set the color temperature of your screen.

## Dependencies
* A Wayland compositor that supports the `wlr-gamma-control-unstable-v1` protocol (e.g., Sway, niri, Hyprland).
* Rust and Cargo for building from source.

## Building from Source
Clone this repository and run
```
cargo build --release
```
Then, put both binaries `target/release/waysn` and `target/release/waysn-daemon` in your path.

## Features
- Manipulate the temperature for specific outputs
- Get the temperatures of specific outputs

## Why
-  Many similar tools change the temperature automatically depanding on to your timezone. I prefer not to have this feature and to change it manually.

## Usage
Start by initializing the daemon:
```
waysn-daemon
```
Then, in a different terminal, simply pass the temperature you want to apply:
```
waysn set 4000
// You can also specify the outputs:
waysn set 4000 -o eDP-1 HDMI-A-1
```
If you want to get the temperatures of the outputs run:
```
waysn get
// You can also specify the outputs:
waysn get eDP-1 HDMI-A-1
// You can specify the JSON format
waysn --json get eDP-1 HDMI-A-1
```
Finally, to stop the daemon, kill it:
```
waysn kill
```
Run `waysn --help` or `waysn <subcommand> --help` for more info.

## Similar tools
- [wlsunset](https://github.com/kennylevinsen/wlsunset)
- [gammastep](https://gitlab.com/chinstrap/gammastep)
