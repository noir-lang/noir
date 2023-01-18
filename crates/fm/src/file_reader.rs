use std::io::Error;
use std::path::Path;
// Based on the environment, we either read files using the rust standard library or we
// read files using the javascript host function

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        wit_bindgen_guest_rust::generate!({
            path: "./wit/fm.wit",
        });

        pub fn read_file_to_string(path_to_file: &Path) -> Result<String, Error> {
            use std::io::ErrorKind;
            let path_str = path_to_file.as_os_str().to_str().unwrap();
            match fs::read_file(path_str) {
                Ok(buffer) => Ok(buffer),
                Err(msg) => Err(Error::new(ErrorKind::Other, msg)),
            }
        }
    } else {
        pub fn read_file_to_string(path_to_file: &Path) -> Result<String, Error> {
            std::fs::read_to_string(path_to_file)
        }
    }
}
