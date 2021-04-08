use std::fs;
use std::path::{Path, PathBuf};

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

pub fn copy<U: AsRef<Path>, V: AsRef<Path>>(from: U, to: V) -> Result<(), std::io::Error> {
    let mut stack = vec![PathBuf::from(from.as_ref())];

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).is_err() {
            println!(" mkdir: {:?}", dest);
            fs::create_dir_all(&dest)?;
        }

        for entry in fs::read_dir(working_path)? {
            let entry = entry?;
            let path = entry.path();

            if is_rs_file(&path) {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else {
                match path.file_name() {
                    Some(filename) => {
                        let dest_path = dest.join(filename);
                        println!("  copy: {:?} -> {:?}", &path, &dest_path);
                        fs::copy(&path, &dest_path)?;
                    }
                    None => {
                        println!("failed: {:?}", path);
                    }
                }
            }
        }
    }

    Ok(())
}

// We use lib.rs to lean on Rusts build system, but we do not want it or any other rust files to be copied
fn is_rs_file(src: &Path) -> bool {
    // assert_eq!("rs", path.extension().unwrap());
    match src.extension() {
        Some(ext) => ext == "rs",
        None => false,
    }
}

fn main() {
    let stdlib_src_dir = Path::new("src/");
    rerun_if_stdlib_changes(stdlib_src_dir);
    let target = dirs::config_dir().unwrap().join("noir-lang").join("std");
    copy(stdlib_src_dir, &target).unwrap();
}
