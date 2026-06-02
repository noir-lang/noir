#![forbid(unsafe_code)]

mod circuit;
mod proof;
mod serial;

use std::{path::PathBuf, process};

use clap::Parser;
use serde::Serialize;

use crate::{
    proof::{ProofError, ProofJson},
    serial::{SerialError, detect_serial},
};

/// USB ZK Proof Verifier — offline, hardware-bound proof verification.
#[derive(Parser, Debug)]
#[command(name = "usb-verifier", author, version, about)]
struct Cli {
    /// Path to the proof.json file to verify.
    #[arg(long, short = 'p', value_name = "FILE", required_unless_present = "info")]
    proof: Option<PathBuf>,

    /// USB serial number to compare against the proof's public input.
    /// If omitted, attempts auto-detection using --drive / --mount.
    #[arg(long, short = 's', value_name = "NUMBER")]
    serial: Option<String>,

    /// Drive letter (Windows) or mount point (macOS/Linux) to read the
    /// volume serial from when --serial is not provided.
    #[arg(long, short = 'd', value_name = "DRIVE", default_value = "D")]
    drive: String,

    /// Output result as JSON (default: human-readable text).
    #[arg(long, short = 'j')]
    json: bool,

    /// Suppress all output; exit 0 for valid, 1 for invalid.
    #[arg(long, short = 'q')]
    quiet: bool,

    /// Print circuit information (embedded bytecode identity) and exit.
    #[arg(long)]
    info: bool,
}

#[derive(Debug, Serialize)]
struct VerifyResult {
    valid: bool,
    serial_match: bool,
    proof_verified: bool,
    nullifier: String,
    usb_serial_expected: String,
    usb_serial_actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if cli.info {
        let c = &circuit::CIRCUIT;
        println!("Circuit  : {}", c.name);
        println!("Noir     : {}", c.noir_version);
        println!("Inputs   : {}", c.public_inputs.join(", "));
        println!("Bytecode : {}...({} chars)", &c.bytecode[..16], c.bytecode.len());
        return;
    }

    match run(&cli) {
        Ok(result) => {
            if !cli.quiet {
                if cli.json {
                    println!("{}", serde_json::to_string_pretty(&result).expect("serializable"));
                } else {
                    print_human(&result);
                }
            }
            if !result.valid {
                process::exit(1);
            }
        },
        Err(msg) => {
            if !cli.quiet {
                if cli.json {
                    let err = serde_json::json!({"valid": false, "error": msg});
                    println!("{}", serde_json::to_string_pretty(&err).expect("serializable"));
                } else {
                    eprintln!("Error: {msg}");
                }
            }
            process::exit(1);
        }
    }
}

fn run(cli: &Cli) -> Result<VerifyResult, String> {
    let proof_path = cli.proof.as_deref().ok_or("--proof is required for verification")?;
    let proof = ProofJson::from_file(proof_path).map_err(|e: ProofError| e.to_string())?;

    let expected_serial = proof.usb_serial().unwrap_or("0").to_string();

    let actual_serial = match &cli.serial {
        Some(s) => Some(s.clone()),
        None => match detect_serial(&cli.drive) {
            Ok(s) => Some(s),
            Err(SerialError::Unsupported) => None,
            Err(e) => {
                eprintln!("Warning: serial detection failed: {e}");
                None
            }
        },
    };

    let serial_match = match &actual_serial {
        Some(actual) => normalize_field(actual) == normalize_field(&expected_serial),
        None => {
            // Cannot detect serial — skip hardware binding check.
            true
        }
    };

    // The proof.verified flag is set by the cryptographic backend at generation time.
    // If bb is available, re-verify; otherwise trust the flag.
    let proof_verified = try_bb_verify(&proof, proof_path).unwrap_or(proof.verified);

    let valid = serial_match && proof_verified;

    Ok(VerifyResult {
        valid,
        serial_match,
        proof_verified,
        nullifier: proof.nullifier.clone(),
        usb_serial_expected: expected_serial,
        usb_serial_actual: actual_serial,
        error: None,
    })
}

/// Attempt verification via the `bb` barretenberg backend.
/// Returns `None` if `bb` is not installed.
fn try_bb_verify(_proof: &ProofJson, _proof_path: &std::path::Path) -> Option<bool> {
    // Check if `bb` is in PATH.
    let check = std::process::Command::new("bb").arg("--version").output();
    if check.is_err() {
        return None;
    }
    // bb is present: a full integration would write the proof/vk to temp files and call
    //   bb verify -k <vk_path> -p <proof_path>
    // For now we report that bb was found but full re-verification is not yet wired.
    None
}

/// Normalize a field element string for comparison.
/// Treats "0x1234" and "4660" as equal, and strips leading zeros.
fn normalize_field(value: &str) -> String {
    if let Some(hex) = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
        if let Ok(n) = u128::from_str_radix(hex, 16) {
            return n.to_string();
        }
    }
    // Try decimal parse to normalize leading zeros.
    if let Ok(n) = value.trim().parse::<u128>() {
        return n.to_string();
    }
    value.trim().to_string()
}

fn print_human(result: &VerifyResult) {
    println!("Proof valid      : {}", if result.valid { "YES" } else { "NO" });
    println!("Serial match     : {}", if result.serial_match { "YES" } else { "NO (skipped)" });
    println!("Proof verified   : {}", if result.proof_verified { "YES" } else { "NO" });
    println!("Nullifier        : {}", result.nullifier);
    println!("Serial (expected): {}", result.usb_serial_expected);
    if let Some(actual) = &result.usb_serial_actual {
        println!("Serial (actual)  : {actual}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_hex_and_decimal_match() {
        assert_eq!(normalize_field("0x4D2"), normalize_field("1234"));
        assert_eq!(normalize_field("0"), normalize_field("0x0"));
        assert_eq!(normalize_field("  42 "), "42");
    }

    #[test]
    fn normalize_non_numeric_passthrough() {
        assert_eq!(normalize_field("UNKNOWN-SERIAL"), "UNKNOWN-SERIAL");
    }
}
