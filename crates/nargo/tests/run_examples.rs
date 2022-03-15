// This file runs the examples. Currently, we will not know if there is a failure however since
// all errors are written to stderr and std::process::exit is called
//
// This requires a refactor of nargo, however it should not affect this file, so
// the scaffolding below is being committed.
#[test]
fn run_examples() {
    let examples_dir = format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"));
    let paths = std::fs::read_dir(&examples_dir)
        .expect(&format!("Could not read from directory {}", examples_dir));

    for path in paths {
        let path = path.unwrap().path();
        if !path.is_dir() {
            continue;
        }

        nargo::cli::build_from_path(path).unwrap();
    }
}
