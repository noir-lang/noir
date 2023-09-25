use acvm::acir::{
    circuit::Circuit,
    native_types::{Witness, WitnessMap},
};
use js_sys::JsString;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::JsWitnessMap;

fn extract_indices(witness_map: &WitnessMap, indices: Vec<Witness>) -> Result<WitnessMap, String> {
    let mut extracted_witness_map = WitnessMap::new();
    for witness in indices {
        let witness_value = witness_map.get(&witness).ok_or(format!(
            "Failed to extract witness {} from witness map. Witness not found.",
            witness.0
        ))?;
        extracted_witness_map.insert(witness, *witness_value);
    }
    Ok(extracted_witness_map)
}

/// Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's return values.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
/// @returns {WitnessMap} A witness map containing the circuit's return values.
#[wasm_bindgen(js_name = getReturnWitness)]
pub fn get_return_witness(
    circuit: Vec<u8>,
    witness_map: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let witness_map = WitnessMap::from(witness_map);

    let return_witness =
        extract_indices(&witness_map, circuit.return_values.0.into_iter().collect())?;

    Ok(JsWitnessMap::from(return_witness))
}

/// Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's public parameters.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
/// @returns {WitnessMap} A witness map containing the circuit's public parameters.
#[wasm_bindgen(js_name = getPublicParametersWitness)]
pub fn get_public_parameters_witness(
    circuit: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let witness_map = WitnessMap::from(solved_witness);

    let public_params_witness =
        extract_indices(&witness_map, circuit.public_parameters.0.into_iter().collect())?;

    Ok(JsWitnessMap::from(public_params_witness))
}

/// Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's public inputs.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
/// @returns {WitnessMap} A witness map containing the circuit's public inputs.
#[wasm_bindgen(js_name = getPublicWitness)]
pub fn get_public_witness(
    circuit: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = Circuit::read(&*circuit).expect("Failed to deserialize circuit");
    let witness_map = WitnessMap::from(solved_witness);

    let public_witness =
        extract_indices(&witness_map, circuit.public_inputs().0.into_iter().collect())?;

    Ok(JsWitnessMap::from(public_witness))
}
