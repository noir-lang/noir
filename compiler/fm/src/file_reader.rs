use rust_embed::RustEmbed;
use std::io::{Error, ErrorKind};
use std::path::Path;

// Based on the environment, we either read files using the rust standard library or we
// read files using the javascript host function

#[derive(RustEmbed)]
#[folder = "../../noir_stdlib/src"]
#[cfg_attr(not(target_os = "windows"), prefix = "std/")]
#[cfg_attr(target_os = "windows", prefix = r"std\")] // Note reversed slash direction
struct StdLibAssets;

#[cfg(target_os = "windows")]
pub(super) fn is_stdlib_asset(path: &Path) -> bool {
    path.starts_with("std\\")
}

#[cfg(not(target_os = "windows"))]
pub(super) fn is_stdlib_asset(path: &Path) -> bool {
    path.starts_with("std/")
}

fn get_stdlib_asset(path: &Path) -> std::io::Result<String> {
    if !is_stdlib_asset(path) {
        return Err(Error::new(ErrorKind::InvalidInput, "requested a non-stdlib asset"));
    }

    match StdLibAssets::get(path.to_str().unwrap()) {
        Some(std_lib_asset) => {
            Ok(std::str::from_utf8(std_lib_asset.data.as_ref()).unwrap().to_string())
        }

        None => Err(Error::new(ErrorKind::NotFound, "invalid stdlib path")),
    }
}

pub(crate) fn read_file_to_string(path_to_file: &Path) -> std::io::Result<String> {
    if is_stdlib_asset(path_to_file) {
        get_stdlib_asset(path_to_file)
    } else {
        get_non_stdlib_asset(path_to_file)
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "wasm32", not(target_os = "wasi")))] {
        use wasm_bindgen::{prelude::*, JsValue};

        #[wasm_bindgen(module = "@noir-lang/source-resolver")]
        extern "C" {

            #[wasm_bindgen(catch)]
            fn read_file(path: &str) -> Result<String, JsValue>;

        }

        fn get_non_stdlib_asset(path_to_file: &Path) -> std::io::Result<String> {
            match read_file(path_str) {
                Ok(buffer) => Ok(buffer),
                Err(_) => Err(Error::new(ErrorKind::Other, "could not read file using wasm")),
            }
        }
    } else {

        fn get_non_stdlib_asset(path_to_file: &Path) -> std::io::Result<String> {
            std::fs::read_to_string(path_to_file)
        }
    }
}
