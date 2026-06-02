use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

fn verifier_bin() -> PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo when running integration tests.
    PathBuf::from(env!("CARGO_BIN_EXE_usb-verifier"))
}

/// Run the verifier binary with the given args and return (exit_code, stdout, stderr).
fn run_verifier(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(verifier_bin())
        .args(args)
        .output()
        .expect("usb-verifier binary should be built (run cargo build first)");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, stdout, stderr)
}

#[test]
fn valid_proof_with_matching_serial_exits_zero() {
    let proof_path = fixtures_dir().join("proof.json");
    // The fixture has usb_serial "305441741", pass it explicitly.
    let (code, stdout, _stderr) =
        run_verifier(&["--proof", proof_path.to_str().unwrap(), "--serial", "305441741", "--json"]);

    let result: serde_json::Value = serde_json::from_str(&stdout).expect("JSON output");
    assert!(result["valid"].as_bool().unwrap_or(false), "proof should be valid");
    assert!(result["serial_match"].as_bool().unwrap_or(false), "serial should match");
    assert_eq!(code, 0);
}

#[test]
fn mismatched_serial_exits_nonzero() {
    let proof_path = fixtures_dir().join("proof.json");
    let (code, stdout, _stderr) =
        run_verifier(&["--proof", proof_path.to_str().unwrap(), "--serial", "0", "--json"]);

    let result: serde_json::Value = serde_json::from_str(&stdout).expect("JSON output");
    assert!(!result["serial_match"].as_bool().unwrap_or(true), "serial should not match");
    assert!(!result["valid"].as_bool().unwrap_or(true), "proof should be invalid");
    assert_ne!(code, 0);
}

#[test]
fn proof_with_hex_serial_normalizes_correctly() {
    let proof_path = fixtures_dir().join("proof.json");
    // 0x1234ABCD == 305441741 (same serial, different representation)
    let (code, stdout, _stderr) =
        run_verifier(&["--proof", proof_path.to_str().unwrap(), "--serial", "0x1234ABCD", "--json"]);

    let result: serde_json::Value = serde_json::from_str(&stdout).expect("JSON output");
    assert!(result["serial_match"].as_bool().unwrap_or(false), "0x1234ABCD should match 305441741");
    assert_eq!(code, 0);
}

#[test]
fn quiet_mode_produces_no_output() {
    let proof_path = fixtures_dir().join("proof.json");
    let (code, stdout, stderr) =
        run_verifier(&["--proof", proof_path.to_str().unwrap(), "--serial", "305441741", "--quiet"]);

    assert!(stdout.is_empty(), "quiet mode should produce no stdout");
    assert!(stderr.is_empty(), "quiet mode should produce no stderr");
    assert_eq!(code, 0);
}

#[test]
fn human_readable_output_contains_valid_label() {
    let proof_path = fixtures_dir().join("proof.json");
    let (_code, stdout, _stderr) =
        run_verifier(&["--proof", proof_path.to_str().unwrap(), "--serial", "305441741"]);

    assert!(stdout.contains("YES"), "human output should show YES for valid proof");
    assert!(stdout.contains("Nullifier"), "human output should show nullifier");
}
