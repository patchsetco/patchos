mod device;
mod status;

use std::env;
use std::io;
use std::path::Path;
use std::process;
use std::thread;

use status::{SystemStatus, PATCHD_VERSION};

const STATE_DIRECTORY: &str = "/var/lib/patchd";

fn main() {
    if env::args().nth(1).as_deref() == Some("--version") {
        println!("patchd {PATCHD_VERSION}");
        return;
    }

    if let Err(error) = run() {
        eprintln!("patchd: {error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let device_id = device::load_or_create_device_id(Path::new(STATE_DIRECTORY))?;
    let status = SystemStatus::collect(device_id)?;

    println!("PatchOS controller starting");
    println!("version={}", status.patchd_version);
    println!("device_id={}", status.device_id);
    println!("os_version={}", status.os_version);
    println!("hostname={}", status.hostname);

    match status.uptime_seconds {
        Some(value) => println!("uptime_seconds={value}"),
        None => println!("uptime_seconds=unknown"),
    }

    match status.memory_total_bytes {
        Some(value) => println!("memory_total_bytes={value}"),
        None => println!("memory_total_bytes=unknown"),
    }

    loop {
        thread::park();
    }
}
