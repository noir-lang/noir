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

fn main() {
    let stdlib_src_dir = Path::new("../../noir_stdlib/");
    rerun_if_stdlib_changes(stdlib_src_dir);
}
