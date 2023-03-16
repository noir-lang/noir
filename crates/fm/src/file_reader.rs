use rust_embed::RustEmbed;
use std::io::Error;
use std::path::Path;

// Based on the environment, we either read files using the rust standard library or we
// read files using the javascript host function

#[derive(RustEmbed)]
#[folder = "../../noir_stdlib/src"]
#[prefix = "std/"]
struct StdLibAssets;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use wasm_bindgen::{prelude::*, JsValue};

        #[wasm_bindgen(module = "@noir-lang/noir-source-resolver")]
        extern "C" {

            #[wasm_bindgen(catch)]
            fn read_file(path: &str) -> Result<String, JsValue>;

        }

        pub(crate) fn read_file_to_string(path_to_file: &Path) -> Result<String, Error> {
            use std::io::ErrorKind;

            match StdLibAssets::get(path_to_file.to_str().unwrap()) {

                Some(std_lib_asset) => {
                    Ok(std::str::from_utf8(std_lib_asset.data.as_ref()).unwrap().to_string())
                },

                None => match read_file(path_str) {
                    Ok(buffer) => Ok(buffer),
                    Err(_) => Err(Error::new(ErrorKind::Other, "could not read file using wasm")),
                }

            }
        }
    } else {
        pub(crate) fn read_file_to_string(path_to_file: &Path) -> Result<String, Error> {

            match StdLibAssets::get(path_to_file.to_str().unwrap()) {

                Some(std_lib_asset) => {
                    Ok(std::str::from_utf8(std_lib_asset.data.as_ref()).unwrap().to_string())
                },

                None => std::fs::read_to_string(path_to_file)

            }
        }
    }
}
