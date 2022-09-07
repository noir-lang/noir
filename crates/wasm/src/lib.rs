use std::path::PathBuf;
use wasm_bindgen::prelude::*;

// Returns a compiled program which is the ACIR circuit along with the ABI
#[wasm_bindgen]
pub fn compile(src: String) -> JsValue {
    console_error_panic_hook::set_once();
    // For now we default to plonk width = 3, though we can add it as a parameter
    let language = acvm::Language::PLONKCSat { width: 3 };
    let path = PathBuf::from(src);
    let compiled_program = noirc_driver::Driver::compile_file(path, language);
    JsValue::from_serde(&compiled_program).unwrap()
}
