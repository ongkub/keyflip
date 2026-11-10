#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]
// Pure modules are also compiled by `cargo test` on macOS.
#![cfg_attr(not(windows), allow(dead_code))]
mod exclude;
#[cfg(windows)]
mod hook;
#[cfg(windows)]
mod http;
#[cfg(windows)]
mod inject;
mod input;
mod keys;
mod pairs;
mod privacy;
mod report;
mod shift_tap;
mod store;
#[cfg(windows)]
mod tray;
#[cfg(windows)]
mod uia;

fn main() {
    #[cfg(windows)]
    if let Err(error) = tray::run() {
        tray::error(&error);
    }
}
