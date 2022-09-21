use acvm::acir::circuit::Circuit;
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
// Deserialises bytes into ACIR structure
#[wasm_bindgen]
pub fn acir_from_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::from_bytes(&bytes);
    JsValue::from_serde(&circuit).unwrap()
}

#[wasm_bindgen]
pub fn acir_to_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = acir.into_serde().unwrap();
    circuit.to_bytes()
}
