use std::env;

// Useful for printing debugging messages during the build
// macro_rules! p {
//     ($($tokens: tt)*) => {
//         println!("cargo:warning={}", format!($($tokens)*))
//     }
// }

fn main() -> Result<(), String> {
    let native_backend = env::var("CARGO_FEATURE_NATIVE").is_ok();

    if native_backend {
        Ok(())
    } else {
        match env::var("BARRETENBERG_BIN_DIR") {
            Ok(bindir) => {
                println!("cargo:rustc-env=BARRETENBERG_BIN_DIR={bindir}");
                Ok(())
            }
            Err(_) => {
                if let Ok(bindir) = pkg_config::get_variable("barretenberg", "bindir") {
                    println!("cargo:rustc-env=BARRETENBERG_BIN_DIR={bindir}");
                    Ok(())
                } else {
                    Err("Unable to locate barretenberg.wasm - Please set the BARRETENBERG_BIN_DIR env var to the directory where it exists".into())
                }
            }
        }
    }
}
