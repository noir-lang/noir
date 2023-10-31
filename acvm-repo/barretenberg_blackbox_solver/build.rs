use std::path::PathBuf;

const BARRETENBERG_BIN_DIR: &str = "BARRETENBERG_BIN_DIR";

fn main() -> Result<(), String> {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let dest_path = PathBuf::from(out_dir.clone()).join("acvm_backend.wasm");

    println!("cargo:rustc-env={BARRETENBERG_BIN_DIR}={out_dir}");
    std::fs::copy("./src/acvm_backend.wasm", dest_path).unwrap();

    Ok(())
}
