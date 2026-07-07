//! Ensures every publishable crate under `acvm-repo/` is tracked by
//! release-please, so its version is bumped in lockstep with the rest of the
//! workspace on each release.
//!
//! release-please only rewrites the version in a file that is explicitly listed
//! in the `extra-files` array of `release-please-config.json`. The
//! `x-release-please-*` annotation comments in a `Cargo.toml` do nothing on
//! their own — without the file being registered, the version is never touched.
//! A publishable acvm crate that is missing from that list silently stops
//! tracking the release version (it stays behind while everything else is
//! bumped) and is then published to crates.io at a stale version by
//! `.github/workflows/publish-acvm.yml`.
//!
//! The check is directory-based, so new acvm crates are picked up automatically
//! with no list to maintain here.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[test]
fn all_publishable_acvm_crates_are_tracked_by_release_please() {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("failed to invoke `cargo metadata`");
    assert!(
        output.status.success(),
        "`cargo metadata` failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let metadata: Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON from `cargo metadata`");
    let workspace_root = PathBuf::from(
        metadata["workspace_root"].as_str().expect("workspace_root missing from cargo metadata"),
    );

    // Publishable crates whose manifest lives under `acvm-repo/`, as
    // `acvm-repo/<dir>/Cargo.toml` paths relative to the workspace root — the
    // exact form release-please expects in `extra-files`.
    let mut expected: Vec<String> = Vec::new();
    for pkg in metadata["packages"].as_array().expect("packages missing from cargo metadata") {
        let manifest_path = Path::new(pkg["manifest_path"].as_str().expect("manifest_path missing"));
        let Ok(relative) = manifest_path.strip_prefix(&workspace_root) else {
            continue;
        };
        if !relative.starts_with("acvm-repo") {
            continue;
        }
        // `publish = false` serialises as an empty array; anything else
        // (including `null`) means the crate can be published.
        if matches!(pkg["publish"].as_array(), Some(registries) if registries.is_empty()) {
            continue;
        }
        expected.push(relative.to_string_lossy().replace('\\', "/"));
    }
    expected.sort();
    assert!(!expected.is_empty(), "expected to find publishable crates under `acvm-repo/`");

    // Plain-string `extra-files` entries registered for the root package.
    // Object entries (e.g. the JSON `package.json` updaters) are not relevant
    // to acvm `Cargo.toml` tracking and are skipped.
    let config_path = workspace_root.join("release-please-config.json");
    let config_bytes = std::fs::read(&config_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", config_path.display()));
    let config: Value =
        serde_json::from_slice(&config_bytes).expect("invalid JSON in release-please-config.json");
    let extra_files: Vec<&str> = config["packages"]["."]["extra-files"]
        .as_array()
        .expect("packages.\".\".extra-files missing from release-please-config.json")
        .iter()
        .filter_map(Value::as_str)
        .collect();

    let missing: Vec<&String> =
        expected.iter().filter(|path| !extra_files.contains(&path.as_str())).collect();
    assert!(
        missing.is_empty(),
        "The following publishable `acvm-repo` crates are not listed in the `extra-files` array \
         of `release-please-config.json`, so release-please will not bump their version on \
         release:\n{}\n\nAdd each path to `extra-files` (next to the other acvm-repo crates) so \
         its version stays in sync with the workspace.",
        missing.iter().map(|p| format!("  {p}")).collect::<Vec<_>>().join("\n"),
    );

    // Symmetric guard: an acvm `Cargo.toml` listed in `extra-files` that no
    // longer maps to a publishable crate (a rename, removal, or typo) would
    // make release-please fail at release time. Flag it here instead.
    let stale: Vec<&str> = extra_files
        .iter()
        .copied()
        .filter(|path| {
            path.starts_with("acvm-repo/")
                && path.ends_with("/Cargo.toml")
                && !expected.iter().any(|e| e == path)
        })
        .collect();
    assert!(
        stale.is_empty(),
        "The following `acvm-repo` entries in `extra-files` do not correspond to a publishable \
         crate (renamed, removed, or misspelled path):\n{}",
        stale.iter().map(|p| format!("  {p}")).collect::<Vec<_>>().join("\n"),
    );
}
