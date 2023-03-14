use rustc_version::{version, Version};
use std::path::Path;

/// Expects that the given directory is an existing path
fn rerun_if_stdlib_changes(directory: &Path) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();

        if path.is_dir() {
            rerun_if_stdlib_changes(&path);
        } else {
            // Tell Cargo that if the given file changes, to rerun this build script.
            println!("cargo:rerun-if-changed={}", path.to_string_lossy());
        }
    }
}

fn check_rustc_version() {
    assert!(
        version().unwrap() >= Version::parse("1.66.0").unwrap(),
        "The minimal supported rustc version is 1.66.0."
    );
}

fn main() {
    check_rustc_version();

    if let Ok(git_commit) = std::env::var(GIT_COMMIT) {
        println!("Using environment defined $GIT_COMMIT={git_commit}")
    } else {
        println!("Collecting Git Data from system...");
        build_data::set_GIT_COMMIT();
        build_data::set_GIT_DIRTY();
        build_data::no_debug_rebuilds();
    }

    let stdlib_src_dir = Path::new("../../noir_stdlib/");
    rerun_if_stdlib_changes(stdlib_src_dir);
}
