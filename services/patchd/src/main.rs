use std::env;
use std::fs;
use std::thread;

const VERSION: &str = "0.1.0";

fn main() {
    if env::args().nth(1).as_deref() == Some("--version") {
        println!("patchd {VERSION}");
        return;
    }

    println!("PatchOS controller starting");
    println!("version={VERSION}");
    println!("os_version={}", read_os_version());
    println!("hostname={}", read_hostname());
    println!("uptime_seconds={}", read_uptime_seconds());
    println!("memory_total_bytes={}", read_memory_total_bytes());

    thread::park();
}

fn read_os_version() -> String {
    let contents = fs::read_to_string("/etc/os-release").unwrap_or_default();

    for key in ["VERSION_ID", "VERSION"] {
        if let Some(value) = read_os_release_value(&contents, key) {
            return value;
        }
    }

    "unknown".to_string()
}

fn read_os_release_value(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        if name != key {
            return None;
        }

        Some(value.trim_matches(['"', '\'']).to_string())
    })
}

fn read_hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .map(|value| value.trim().to_string())
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn read_uptime_seconds() -> String {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|contents| contents.split_whitespace().next()?.parse::<f64>().ok())
        .map(|seconds| (seconds as u64).to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn read_memory_total_bytes() -> String {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| {
            contents.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name != "MemTotal" {
                    return None;
                }

                value
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()?
                    .checked_mul(1024)
            })
        })
        .map(|bytes| bytes.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
