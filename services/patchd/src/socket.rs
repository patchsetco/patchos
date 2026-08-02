use crate::status::SystemStatus;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::time::Duration;

const MAX_REQUEST_BYTES: u64 = 128;
const PROTOCOL_VERSION: u8 = 1;

pub fn request_status(socket_path: &Path) -> io::Result<()> {
    let mut stream = UnixStream::connect(socket_path)?;

    stream.write_all(b"status\n")?;
    stream.shutdown(Shutdown::Write)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    print!("{response}");

    Ok(())
}

pub fn serve_status_socket(socket_path: &Path, device_id: &str) -> io::Result<()> {
    let listener = bind_status_socket(socket_path)?;

    println!("socket={}", socket_path.display());

    // ponytail: serve clients serially; add workers when concurrent control traffic exists.
    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                if let Err(error) = handle_status_client(stream, device_id) {
                    eprintln!("patchd: client error: {error}");
                }
            }
            Err(error) => return Err(error),
        }
    }

    Ok(())
}

fn bind_status_socket(socket_path: &Path) -> io::Result<UnixListener> {
    match UnixListener::bind(socket_path) {
        Ok(listener) => Ok(listener),
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
            if UnixStream::connect(socket_path).is_ok() {
                return Err(io::Error::new(
                    io::ErrorKind::AddrInUse,
                    "another patchd instance is already running",
                ));
            }

            fs::remove_file(socket_path)?;
            UnixListener::bind(socket_path)
        }
        Err(error) => Err(error),
    }
}

fn handle_status_client(stream: UnixStream, device_id: &str) -> io::Result<()> {
    handle_status_client_with(stream, || SystemStatus::collect(device_id.to_string()))
}

fn handle_status_client_with(
    mut stream: UnixStream,
    collect_status: impl FnOnce() -> io::Result<SystemStatus>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;

    let mut request = String::new();
    let reader = BufReader::new(stream.try_clone()?);
    let bytes_read = reader.take(MAX_REQUEST_BYTES + 1).read_line(&mut request)?;

    if bytes_read == 0 {
        return Ok(());
    }

    if bytes_read as u64 > MAX_REQUEST_BYTES {
        writeln!(stream, "error=request_too_large")?;
        return stream.flush();
    }

    let command = request.trim_end_matches(['\r', '\n']);

    match command {
        "status" => write_status_response(&mut stream, &collect_status()?),
        _ => {
            writeln!(stream, "error=unknown_command")?;
            stream.flush()
        }
    }
}

fn write_status_response(writer: &mut impl Write, status: &SystemStatus) -> io::Result<()> {
    writeln!(writer, "protocol_version={PROTOCOL_VERSION}")?;
    writeln!(writer, "patchd_version={}", status.patchd_version)?;
    writeln!(
        writer,
        "device_id={}",
        sanitize_wire_value(&status.device_id)
    )?;
    writeln!(
        writer,
        "os_version={}",
        sanitize_wire_value(&status.os_version)
    )?;
    writeln!(writer, "hostname={}", sanitize_wire_value(&status.hostname))?;

    match status.uptime_seconds {
        Some(value) => writeln!(writer, "uptime_seconds={value}")?,
        None => writeln!(writer, "uptime_seconds=unknown")?,
    }

    match status.memory_total_bytes {
        Some(value) => writeln!(writer, "memory_total_bytes={value}")?,
        None => writeln!(writer, "memory_total_bytes=unknown")?,
    }

    writeln!(writer)?;
    writer.flush()
}

fn sanitize_wire_value(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    const EXISTING_ID: &str = "3e0cd5c8-aed4-44d5-bfd2-bc45745d794a";

    fn test_status() -> SystemStatus {
        SystemStatus {
            patchd_version: "0.1.0",
            device_id: EXISTING_ID.to_string(),
            os_version: "0.0.1".to_string(),
            hostname: "patchos".to_string(),
            uptime_seconds: Some(14),
            memory_total_bytes: Some(8_338_726_912),
        }
    }

    #[test]
    fn serves_status_over_socket() {
        let (server, mut client) = UnixStream::pair().unwrap();

        let server_thread = thread::spawn(move || {
            handle_status_client_with(server, || Ok(test_status())).unwrap();
        });

        client.write_all(b"status\n").unwrap();
        client.shutdown(Shutdown::Write).unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server_thread.join().unwrap();

        assert!(response.contains("protocol_version=1\n"));
        assert!(response.contains("patchd_version=0.1.0\n"));
        assert!(response.contains(&format!("device_id={EXISTING_ID}\n")));
        assert!(response.contains("os_version="));
        assert!(response.contains("hostname="));
        assert!(response.contains("uptime_seconds="));
        assert!(response.contains("memory_total_bytes="));
        assert!(response.ends_with("\n\n"));
    }

    #[test]
    fn rejects_unknown_socket_commands() {
        let (server, mut client) = UnixStream::pair().unwrap();

        let server_thread = thread::spawn(move || {
            handle_status_client(server, EXISTING_ID).unwrap();
        });

        client.write_all(b"delete-everything\n").unwrap();
        client.shutdown(Shutdown::Write).unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server_thread.join().unwrap();

        assert_eq!(response, "error=unknown_command\n");
    }

    #[test]
    fn rejects_oversized_socket_commands() {
        let (server, mut client) = UnixStream::pair().unwrap();

        let server_thread = thread::spawn(move || {
            handle_status_client(server, EXISTING_ID).unwrap();
        });

        client.write_all(&[b'x'; 129]).unwrap();
        client.shutdown(Shutdown::Write).unwrap();

        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        server_thread.join().unwrap();

        assert_eq!(response, "error=request_too_large\n");
    }

    #[test]
    fn sanitizes_wire_values() {
        assert_eq!(sanitize_wire_value("one\ntwo\rthree"), "one two three");
    }
}
