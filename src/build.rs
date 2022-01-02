#[cfg(target_os = "linux")]
extern crate winres;

#[cfg(target_os = "linux")]
use std::{env, io};

#[cfg(target_os = "linux")]
fn main() -> io::Result<()> {
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap();
    if target_family == "windows" {
        winres::WindowsResource::new()
            .set_toolkit_path("/usr/bin")
            .set_windres_path("windres")
            .set_ar_path("ar")
            .set_icon("resources/icons/eris.ico")
            .compile()?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn main() {}

#[cfg(target_os = "macos")]
fn main() {}
