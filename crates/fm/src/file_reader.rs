use std::io::Error;
use std::path::Path;
// Based on the environment, we either read files using the rust standard library or we
// read files using the javascript host function
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen::JsValue;

#[cfg(feature = "wasm")]
#[wasm_bindgen(module = "/file_reader.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn read_file(path: &str) -> Result<String, JsValue>;
}

#[cfg(feature = "wasm")]
pub fn read_file_to_string(path_to_file: &Path) -> Result<String, Error> {
    use std::io::ErrorKind;
    let path_str = path_to_file.as_os_str().to_str().unwrap();
    match read_file(path_str) {
        Ok(buffer) => Ok(buffer),
        Err(_) => Err(Error::new(ErrorKind::Other, "could not read file using wasm")),
    }
}

#[cfg(not(feature = "wasm"))]
pub fn read_file_to_string(path_to_file: &Path) -> Result<String, Error> {
    std::fs::read_to_string(&path_to_file)
}
