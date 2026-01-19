use acvm::acir::native_types::{WitnessMap, WitnessStack};
use js_sys::JsString;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::{JsWitnessMap, JsWitnessStack};

/// Compresses a `WitnessMap` into the binary format outputted by Nargo.
///
/// @param {WitnessMap} witness_map - A witness map.
/// @returns {Uint8Array} A compressed witness map
#[wasm_bindgen(js_name = compressWitness, skip_jsdoc)]
pub fn compress_witness(witness_map: JsWitnessMap) -> Result<Vec<u8>, JsString> {
    console_error_panic_hook::set_once();

    let witness_map = WitnessMap::from(witness_map);
    let witness_stack = WitnessStack::from(witness_map);
    let compressed_witness_stack = witness_stack.serialize().map_err(|err| err.to_string())?;

    Ok(compressed_witness_stack)
}

/// Decompresses a compressed witness as outputted by Nargo into a `WitnessMap`.
/// This should be used to only fetch the witness map for the main function.
///
/// @param {Uint8Array} compressed_witness - A compressed witness.
/// @returns {WitnessMap} The decompressed witness map.
#[wasm_bindgen(js_name = decompressWitness, skip_jsdoc)]
pub fn decompress_witness(compressed_witness: Vec<u8>) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();

    let mut witness_stack =
        WitnessStack::deserialize(compressed_witness.as_slice()).map_err(|err| err.to_string())?;

    let witness =
        witness_stack.pop().expect("Should have at least one witness on the stack").witness;
    Ok(witness.into())
}

/// Compresses a `WitnessStack` into the binary format outputted by Nargo.
///
/// @param {WitnessStack} witness_stack - A witness stack.
/// @returns {Uint8Array} A compressed witness stack
#[wasm_bindgen(js_name = compressWitnessStack, skip_jsdoc)]
pub fn compress_witness_stack(witness_stack: JsWitnessStack) -> Result<Vec<u8>, JsString> {
    console_error_panic_hook::set_once();

    let witness_stack = WitnessStack::from(witness_stack);
    let compressed_witness_stack = witness_stack.serialize().map_err(|err| err.to_string())?;

    Ok(compressed_witness_stack)
}

/// Decompresses a compressed witness stack as outputted by Nargo into a `WitnessStack`.
///
/// @param {Uint8Array} compressed_witness - A compressed witness.
/// @returns {WitnessStack} The decompressed witness stack.
#[wasm_bindgen(js_name = decompressWitnessStack, skip_jsdoc)]
pub fn decompress_witness_stack(compressed_witness: Vec<u8>) -> Result<JsWitnessStack, JsString> {
    console_error_panic_hook::set_once();

    let witness_stack =
        WitnessStack::deserialize(compressed_witness.as_slice()).map_err(|err| err.to_string())?;

    Ok(witness_stack.into())
}
