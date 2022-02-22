use std::path::PathBuf;

// This file runs the examples. Currently, we will not know if there is a failure however since
// all errors are written to stderr and std::process::exit is called
//
// This requires a refactor of nargo, however it should not affect this file, so
// the scaffolding below is being committed.
#[test]
fn run_examples() {
    let mut examples_dir = PathBuf::from(format!("{}/examples", env!("CARGO_MANIFEST_DIR")));

    for _ in 0..5 {
        eprintln!("trying {}", examples_dir.display());
        let paths = match std::fs::read_dir(&examples_dir) {
            Ok(dir) => dir,
            Err(error) => {
                eprintln!(
                    "Could not read from directory '{}', io error: {}",
                    examples_dir.display(),
                    error
                );
                examples_dir = examples_dir
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("examples/");
                continue;
            }
        };

        let mut success = false;
        for path in paths {
            let path = path.unwrap().path();
            if !path.is_dir() {
                continue;
            }

            eprintln!("  Building from path {}", path.display());
            match nargo::cli::build_from_path(path) {
                Ok(_) => success = true,
                Err(error) => {
                    eprintln!("error: {:?}", error);
                }
            }
        }

        if success {
            break;
        }
    }

    panic!("error");
}
