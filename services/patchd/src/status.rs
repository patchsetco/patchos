use std::fs;
use std::io;

pub const PATCHD_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct SystemStatus {
    pub patchd_version: &'static str,
    pub device_id: String,
    pub os_version: String,
    pub hostname: String,
    pub uptime_seconds: Option<u64>,
    pub memory_total_bytes: Option<u64>,
}

impl SystemStatus {
    pub fn collect(device_id: String) -> io::Result<Self> {
        Ok(Self {
            patchd_version: PATCHD_VERSION,
            device_id,
            os_version: read_os_version()?,
            hostname: read_hostname()?,
            uptime_seconds: read_uptime_seconds(),
            memory_total_bytes: read_memory_total_bytes(),
        })
    }
}

fn read_os_version() -> io::Result<String> {
    let contents = fs::read_to_string("/etc/os-release")?;
    parse_os_version(&contents).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "/etc/os-release has no valid version",
        )
    })
}

fn read_hostname() -> io::Result<String> {
    let contents = fs::read_to_string("/etc/hostname")?;
    let hostname = contents.trim();

    if hostname.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "/etc/hostname is empty",
        ))
    } else {
        Ok(hostname.to_string())
    }
}

pub fn read_uptime_seconds() -> Option<u64> {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|contents| parse_uptime(&contents))
}

pub fn read_memory_total_bytes() -> Option<u64> {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|contents| parse_memory_total(&contents))
}

fn parse_uptime(contents: &str) -> Option<u64> {
    let value = contents.split_whitespace().next()?;
    let (seconds, fraction) = value.split_once('.')?;

    if seconds.is_empty()
        || fraction.is_empty()
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }

    seconds.parse().ok()
}

fn parse_memory_total(contents: &str) -> Option<u64> {
    contents.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name != "MemTotal" {
            return None;
        }

        let mut fields = value.split_whitespace();
        let kibibytes = fields.next()?.parse::<u64>().ok()?;

        if fields.next()? != "kB" || fields.next().is_some() {
            return None;
        }

        kibibytes.checked_mul(1024)
    })
}

fn parse_os_version(contents: &str) -> Option<String> {
    ["VERSION_ID", "VERSION"].into_iter().find_map(|key| {
        contents.lines().find_map(|line| {
            let (name, value) = line.split_once('=')?;
            if name != key {
                return None;
            }

            let value = value.trim().trim_matches(['"', '\'']);
            (!value.is_empty()).then(|| value.to_string())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_uptime_seconds() {
        assert_eq!(parse_uptime("14.73 8.21\n"), Some(14));
        assert_eq!(parse_uptime("unknown\n"), None);
        assert_eq!(parse_uptime("14.bad 8.21\n"), None);
    }

    #[test]
    fn parses_total_memory_bytes() {
        assert_eq!(
            parse_memory_total("MemFree: 1 kB\nMemTotal: 8143288 kB\n"),
            Some(8_338_726_912)
        );
        assert_eq!(parse_memory_total("MemTotal: unknown kB\n"), None);
        assert_eq!(parse_memory_total("MemTotal: 8143288 MB\n"), None);
    }

    #[test]
    fn parses_os_version() {
        assert_eq!(
            parse_os_version("NAME=PatchOS\nVERSION_ID=\"0.0.1\"\n"),
            Some("0.0.1".to_string())
        );
        assert_eq!(
            parse_os_version("VERSION='PatchOS bootstrap'\n"),
            Some("PatchOS bootstrap".to_string())
        );
        assert_eq!(parse_os_version("NAME=PatchOS\n"), None);
    }
}
