use std::path::PathBuf;

const BLNS_JSON_PATH: &str = "BLNS_JSON_PATH";

fn main() -> Result<(), String> {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let dest_path = PathBuf::from(out_dir.clone()).join("blns.base64.json");
    let dest_path_str = dest_path.to_str().unwrap();

    println!("cargo:rustc-env={BLNS_JSON_PATH}={dest_path_str}");
    std::fs::copy("./src/blns/blns.base64.json", dest_path).unwrap();

    Ok(())
}
