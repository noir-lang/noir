use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    // For the wasm environment to have access to the stdlib
    // We need to manually add it as a dependency.
    // we can't do this at runtime, so we need a copy of the stdlib at compile time
    // to add into the CrateManager.
    //
    // Using the noir-packer python script, we pack the stdlib into
    // a single file, and then include it in this crate ensuring that
    // it is always available to the wasm package
    let link_to_python_file = concat!(env!("CARGO_MANIFEST_DIR"), "/noirpacker.py");

    let link_to_stdlib_dir =
        Path::new(&env!("CARGO_MANIFEST_DIR")).join("..").join("std_lib").join("src");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let link_to_output_file = out_path.join("packed_stdlib.nr");

    let mut cmd = std::process::Command::new(link_to_python_file);

    cmd.arg(link_to_stdlib_dir);
    cmd.arg(link_to_output_file);
    cmd.status().expect("could not run python script to compress the stdlib into one file");
}
