//! Cross-platform USB volume serial number detection.
//!
//! Reads the numeric volume serial of a drive/mount point and normalizes it to a
//! decimal string so it can be compared against the `usb_serial` public input in
//! the Noir proof.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerialError {
    #[allow(dead_code)]
    #[error("Cannot detect serial on this platform without specifying --serial")]
    Unsupported,
    #[error("Failed to run '{cmd}': {reason}")]
    Command { cmd: &'static str, reason: String },
    #[error("Could not parse serial from output: '{output}'")]
    Parse { output: String },
}

/// Detect the volume serial for the given mount path and return it as a decimal
/// string matching the `usb_serial` Noir field value.
pub fn detect_serial(mount: &str) -> Result<String, SerialError> {
    detect_serial_impl(mount)
}

/// Parse a Windows-style "XXXX-XXXX" volume serial to a decimal field value.
///
/// Windows `vol` reports volume serial as "XXXX-XXXX" hex; we strip the dash
/// and parse as a 32-bit hex integer so it fits in a BN254 field element.
pub fn parse_windows_serial(raw: &str) -> Result<String, SerialError> {
    let compact = raw.replace('-', "");
    let value = u32::from_str_radix(compact.trim(), 16).map_err(|_| SerialError::Parse {
        output: raw.to_string(),
    })?;
    Ok(value.to_string())
}

#[cfg(target_os = "windows")]
fn detect_serial_impl(mount: &str) -> Result<String, SerialError> {
    use std::process::Command;

    let drive = if mount.ends_with(':') { mount.to_string() } else { format!("{mount}:") };
    let output = Command::new("cmd")
        .args(["/C", &format!("vol {drive}")])
        .output()
        .map_err(|e| SerialError::Command { cmd: "vol", reason: e.to_string() })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_vol_output(&stdout)
}

#[cfg(target_os = "macos")]
fn detect_serial_impl(mount: &str) -> Result<String, SerialError> {
    use std::process::Command;

    let output = Command::new("diskutil")
        .args(["info", mount])
        .output()
        .map_err(|e| SerialError::Command { cmd: "diskutil", reason: e.to_string() })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_diskutil_output(&stdout)
}

#[cfg(target_os = "linux")]
fn detect_serial_impl(mount: &str) -> Result<String, SerialError> {
    use std::process::Command;

    let output = Command::new("blkid")
        .args(["-o", "value", "-s", "SERIAL", mount])
        .output()
        .map_err(|e| SerialError::Command { cmd: "blkid", reason: e.to_string() })?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Err(SerialError::Parse { output: String::from("<empty blkid output>") });
    }
    // blkid serial is usually hex; try to parse as u64 if it looks like hex, else treat as
    // a decimal string directly.
    if stdout.starts_with("0x") || stdout.starts_with("0X") {
        let value = u64::from_str_radix(&stdout[2..], 16)
            .map_err(|_| SerialError::Parse { output: stdout.clone() })?;
        Ok(value.to_string())
    } else {
        Ok(stdout)
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn detect_serial_impl(_mount: &str) -> Result<String, SerialError> {
    Err(SerialError::Unsupported)
}

/// Parse the serial number from Windows `vol D:` output.
/// Output looks like:
///   Volume in drive D is MY_USB
///   Volume Serial Number is 1234-ABCD
pub(crate) fn parse_vol_output(output: &str) -> Result<String, SerialError> {
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("serial number is") {
            let raw = line
                .split_once("is")
                .map(|(_, suffix)| suffix.trim())
                .ok_or_else(|| SerialError::Parse { output: line.to_string() })?;
            return parse_windows_serial(raw);
        }
    }
    Err(SerialError::Parse { output: output.to_string() })
}

/// Parse the serial from `diskutil info` output on macOS.
/// Looks for a line like "Volume UUID: 1234-ABCD" or "Disk / Partition UUID".
#[allow(dead_code)]
pub(crate) fn parse_diskutil_output(output: &str) -> Result<String, SerialError> {
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("volume serial") || lower.contains("disk serial") {
            if let Some((_, value)) = line.split_once(':') {
                let raw = value.trim();
                if !raw.is_empty() {
                    return parse_windows_serial(raw);
                }
            }
        }
    }
    Err(SerialError::Parse { output: output.to_string() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_windows_vol_output() {
        let output = " Volume in drive D is MY_USB\r\n Volume Serial Number is 1234-ABCD\r\n";
        // 0x1234ABCD = 305441741
        assert_eq!(parse_vol_output(output).unwrap(), "305441741");
    }

    #[test]
    fn parse_windows_serial_strips_dash() {
        assert_eq!(parse_windows_serial("1234-ABCD").unwrap(), "305441741");
        assert_eq!(parse_windows_serial("0000-0001").unwrap(), "1");
        assert_eq!(parse_windows_serial("FFFF-FFFF").unwrap(), u32::MAX.to_string());
    }

    #[test]
    fn parse_windows_serial_rejects_invalid() {
        assert!(parse_windows_serial("not-hex!").is_err());
        assert!(parse_windows_serial("ZZZZ-ZZZZ").is_err());
    }

    #[test]
    fn parse_vol_output_missing_serial_line() {
        let output = " Volume in drive D is MY_USB\r\n";
        assert!(parse_vol_output(output).is_err());
    }
}
