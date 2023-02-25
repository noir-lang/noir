#![forbid(unsafe_code)]
use acvm::acir::circuit::Circuit;
use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    git_hash: &'static str,
    version: &'static str,
    dirty: &'static str,
}

const BUILD_INFO: BuildInfo = BuildInfo {
    git_hash: env!("GIT_COMMIT"),
    version: env!("CARGO_PKG_VERSION"),
    dirty: env!("GIT_DIRTY"),
};

// Returns a compiled program which is the ACIR circuit along with the ABI
#[wasm_bindgen]
pub fn compile(src: String) -> JsValue {
    console_error_panic_hook::set_once();
    // For now we default to plonk width = 3, though we can add it as a parameter
    let language = acvm::Language::PLONKCSat { width: 3 };
    let path = PathBuf::from(src);
    let compiled_program = noirc_driver::Driver::compile_file(path, language);
    <JsValue as JsValueSerdeExt>::from_serde(&compiled_program).unwrap()
}

// Deserializes bytes into ACIR structure
#[deprecated(
    note = "we have moved away from this serialization strategy. Call `acir_read_bytes` instead"
)]
#[allow(deprecated)]
#[wasm_bindgen]
pub fn acir_from_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::from_bytes(&bytes);
    <JsValue as JsValueSerdeExt>::from_serde(&circuit).unwrap()
}

#[deprecated(
    note = "we have moved away from this serialization strategy. Call `acir_write_bytes` instead"
)]
#[allow(deprecated)]
#[wasm_bindgen]
pub fn acir_to_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = JsValueSerdeExt::into_serde(&acir).unwrap();
    circuit.to_bytes()
}

// Deserializes bytes into ACIR structure
#[wasm_bindgen]
pub fn acir_read_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::read(&*bytes).unwrap();
    <JsValue as JsValueSerdeExt>::from_serde(&circuit).unwrap()
}

#[wasm_bindgen]
pub fn acir_write_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = JsValueSerdeExt::into_serde(&acir).unwrap();
    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();
    bytes
}

#[wasm_bindgen]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}
