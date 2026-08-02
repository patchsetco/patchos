use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process;
use std::thread;

const VERSION: &str = "0.1.0";
const STATE_DIRECTORY: &str = "/var/lib/patchd";
const UUID_SOURCE: &str = "/proc/sys/kernel/random/uuid";

fn main() {
    if env::args().nth(1).as_deref() == Some("--version") {
        println!("patchd {VERSION}");
        return;
    }

    if let Err(error) = run() {
        eprintln!("patchd: {error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let device_id = load_or_create_device_id(Path::new(STATE_DIRECTORY))?;

    println!("PatchOS controller starting");
    println!("version={VERSION}");
    println!("device_id={device_id}");
    println!("os_version={}", read_os_version());
    println!("hostname={}", read_hostname());
    println!("uptime_seconds={}", read_uptime_seconds());
    println!("memory_total_bytes={}", read_memory_total_bytes());

    loop {
        thread::park();
    }
}

fn load_or_create_device_id(state_directory: &Path) -> io::Result<String> {
    let path = state_directory.join("device-id");

    match fs::read_to_string(&path) {
        Ok(contents) => validate_device_id(&contents),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let device_id = validate_device_id(&fs::read_to_string(UUID_SOURCE)?)?;
            let temporary_path = state_directory.join("device-id.tmp");

            fs::write(&temporary_path, format!("{device_id}\n"))?;
            fs::rename(&temporary_path, &path)?;

            Ok(device_id)
        }
        Err(error) => Err(error),
    }
}

fn validate_device_id(contents: &str) -> io::Result<String> {
    let device_id = contents.trim();
    let valid = device_id.len() == 36
        && device_id.bytes().enumerate().all(|(index, byte)| {
            if matches!(index, 8 | 13 | 18 | 23) {
                byte == b'-'
            } else {
                byte.is_ascii_hexdigit()
            }
        });

    if valid {
        Ok(device_id.to_string())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "device identity is empty or malformed",
        ))
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    const EXISTING_ID: &str = "3e0cd5c8-aed4-44d5-bfd2-bc45745d794a";

    struct TestState {
        path: std::path::PathBuf,
    }

    impl TestState {
        fn new() -> Self {
            let unique = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = env::temp_dir().join(format!(
                "patchd-test-{}-{timestamp}-{unique}",
                process::id()
            ));

            fs::create_dir(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TestState {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.path).unwrap();
        }
    }

    #[test]
    fn creates_an_id_when_none_exists() {
        let state = TestState::new();
        let device_id = load_or_create_device_id(&state.path).unwrap();

        assert_eq!(validate_device_id(&device_id).unwrap(), device_id);
        assert_eq!(
            fs::read_to_string(state.path.join("device-id"))
                .unwrap()
                .trim(),
            device_id
        );
        assert!(!state.path.join("device-id.tmp").exists());
    }

    #[test]
    fn returns_the_existing_id_on_the_second_call() {
        let state = TestState::new();
        let first = load_or_create_device_id(&state.path).unwrap();
        let second = load_or_create_device_id(&state.path).unwrap();

        assert_eq!(second, first);
    }

    #[test]
    fn rejects_an_empty_identity_file() {
        let state = TestState::new();
        fs::write(state.path.join("device-id"), "\n").unwrap();

        let error = load_or_create_device_id(&state.path).unwrap_err();

        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(
            fs::read_to_string(state.path.join("device-id")).unwrap(),
            "\n"
        );
    }

    #[test]
    fn does_not_replace_a_valid_existing_identity() {
        let state = TestState::new();
        let path = state.path.join("device-id");
        let original_contents = format!("  {EXISTING_ID}  \n");
        fs::write(&path, &original_contents).unwrap();

        let device_id = load_or_create_device_id(&state.path).unwrap();

        assert_eq!(device_id, EXISTING_ID);
        assert_eq!(fs::read_to_string(path).unwrap(), original_contents);
    }
}
