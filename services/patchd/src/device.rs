use std::fs;
use std::io;
use std::path::Path;

const UUID_SOURCE: &str = "/proc/sys/kernel/random/uuid";

pub fn load_or_create_device_id(state_directory: &Path) -> io::Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::process;
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
