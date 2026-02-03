const GIT_COMMIT: &&str = &"GIT_COMMIT";
use std::path::Path;

fn main() -> Result<(), String> {
    // Only use build_data if the environment variable isn't set.
    if std::env::var(GIT_COMMIT).is_err() {
        build_data::set_GIT_COMMIT()?;
        build_data::set_GIT_DIRTY()?;
        build_data::no_debug_rebuilds()?;
    }

    let stdlib_src_dir = Path::new("../../noir_stdlib/");
    rerun_if_stdlib_changes(stdlib_src_dir);
    Ok(())
}

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
