mod device;
mod socket;
mod status;

use std::env;
use std::io;
use std::path::Path;
use std::process;

use status::{SystemStatus, PATCHD_VERSION};

const STATE_DIRECTORY: &str = "/var/lib/patchd";
const SOCKET_PATH: &str = "/run/patchd/patchd.sock";

fn main() {
    let result = match env::args().nth(1).as_deref() {
        None => run(),
        Some("--version") => {
            println!("patchd {PATCHD_VERSION}");
            Ok(())
        }
        Some("status") => socket::request_status(Path::new(SOCKET_PATH)),
        Some(argument) => {
            eprintln!("patchd: unknown command: {argument}");
            eprintln!("usage: patchd [--version|status]");
            process::exit(2);
        }
    };

    if let Err(error) = result {
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

    socket::serve_status_socket(Path::new(SOCKET_PATH), &status.device_id)
}
