use acvm::{
    acir::{
        circuit::Program,
        native_types::{Witness, WitnessMap},
    },
    FieldElement,
};
use js_sys::JsString;
use wasm_bindgen::prelude::wasm_bindgen;

use crate::JsWitnessMap;

pub(crate) fn extract_indices(
    witness_map: &WitnessMap<FieldElement>,
    indices: Vec<Witness>,
) -> Result<WitnessMap<FieldElement>, String> {
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
    // TODO(https://github.com/noir-lang/noir/issues/4428): These need to be updated to match the same interfaces
    // as the native ACVM executor. Right now native execution still only handles one circuit so I do not feel the need
    // to break the JS interface just yet.
    program: Vec<u8>,
    witness_map: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let program: Program<FieldElement> =
        Program::deserialize_program(&program).expect("Failed to deserialize circuit");
    let circuit = match program.functions.len() {
        0 => return Ok(JsWitnessMap::from(WitnessMap::new())),
        1 => &program.functions[0],
        _ => return Err(JsString::from("Program contains multiple circuits however ACVM currently only supports programs containing a single circuit"))
    };

    let witness_map = WitnessMap::from(witness_map);

    let return_witness =
        extract_indices(&witness_map, circuit.return_values.0.iter().copied().collect())?;

    Ok(JsWitnessMap::from(return_witness))
}

/// Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's public parameters.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
/// @returns {WitnessMap} A witness map containing the circuit's public parameters.
#[wasm_bindgen(js_name = getPublicParametersWitness)]
pub fn get_public_parameters_witness(
    program: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let program: Program<FieldElement> =
        Program::deserialize_program(&program).expect("Failed to deserialize circuit");
    let circuit = match program.functions.len() {
        0 => return Ok(JsWitnessMap::from(WitnessMap::new())),
        1 => &program.functions[0],
        _ => return Err(JsString::from("Program contains multiple circuits however ACVM currently only supports programs containing a single circuit"))
    };

    let witness_map = WitnessMap::from(solved_witness);

    let public_params_witness =
        extract_indices(&witness_map, circuit.public_parameters.0.iter().copied().collect())?;

    Ok(JsWitnessMap::from(public_params_witness))
}

/// Extracts a `WitnessMap` containing the witness indices corresponding to the circuit's public inputs.
///
/// @param {Uint8Array} circuit - A serialized representation of an ACIR circuit
/// @param {WitnessMap} witness_map - The completed witness map after executing the circuit.
/// @returns {WitnessMap} A witness map containing the circuit's public inputs.
#[wasm_bindgen(js_name = getPublicWitness)]
pub fn get_public_witness(
    program: Vec<u8>,
    solved_witness: JsWitnessMap,
) -> Result<JsWitnessMap, JsString> {
    console_error_panic_hook::set_once();
    let program: Program<FieldElement> =
        Program::deserialize_program(&program).expect("Failed to deserialize circuit");
    let circuit = match program.functions.len() {
        0 => return Ok(JsWitnessMap::from(WitnessMap::new())),
        1 => &program.functions[0],
        _ => return Err(JsString::from("Program contains multiple circuits however ACVM currently only supports programs containing a single circuit"))
    };

    let witness_map = WitnessMap::from(solved_witness);

    let public_witness =
        extract_indices(&witness_map, circuit.public_inputs().0.clone().into_iter().collect())?;

    Ok(JsWitnessMap::from(public_witness))
}
