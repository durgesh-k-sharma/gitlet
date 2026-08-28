use chrono::{DateTime, Local, TimeZone, Utc};
use sha1::{Digest, Sha1};
use std::fmt::Write;
use std::fs;
use std::path::Path;

pub fn sha1_bytes(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hex = String::with_capacity(40);
    for byte in result {
        let _ = write!(hex, "{:02x}", byte);
    }
    hex
}

pub fn sha1_str(data: &str) -> String {
    sha1_bytes(data.as_bytes())
}

pub fn initial_commit_timestamp() -> String {
    let epoch = Utc.timestamp_opt(0, 0).unwrap();
    epoch.format("%a %b %-d %H:%M:%S %Y %z").to_string()
}

pub fn current_timestamp() -> String {
    let now: DateTime<Local> = Local::now();
    now.format("%a %b %-d %H:%M:%S %Y %z").to_string()
}

pub fn read_file_bytes(path: &Path) -> std::io::Result<Vec<u8>> {
    fs::read(path)
}

pub fn write_file_bytes(path: &Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, data)
}
