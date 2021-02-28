use std::path::PathBuf;

// This file runs the examples. Currently, we will not know if there is a failure however since
// all errors are written to stderr and std::process::exit is called
//
// This requires a refactor of nargo, however it should not affect this file, so
// the scaffolding below is being committed.

fn main() {
    let mut examples_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    examples_dir.push("examples/");

    let paths = std::fs::read_dir(examples_dir).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        nargo::cli::build(path);
    }
}
