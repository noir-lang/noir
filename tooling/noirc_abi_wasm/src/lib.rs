#![warn(unused_crate_dependencies, unused_extern_crates)]

// See Cargo.toml for explanation.
use getrandom as _;
use getrandom_v2 as _;
use getrandom_v4 as _; // cSpell:disable-line

use acvm::{
    AcirField, FieldElement,
    acir::native_types::{WitnessMap, WitnessStack},
    pwg::RawAssertionPayload,
};
use iter_extended::try_btree_map;
use noirc_abi::{
    Abi, AbiErrorType, MAIN_RETURN_NAME, decode_value, display_abi_error,
    errors::InputParserError,
    input_parser::{InputValue, json::JsonTypes},
};
use serde::Serialize;
use std::collections::BTreeMap;

use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

mod errors;
mod js_witness_map;

use errors::JsAbiError;
use js_witness_map::JsWitnessMap;

#[wasm_bindgen(typescript_custom_section)]
const INPUT_MAP: &'static str = r#"
import { Field, InputValue, InputMap, Visibility, Sign, AbiType, AbiParameter, Abi, WitnessMap, RawAssertionPayload } from "@noir-lang/types";
export { Field, InputValue, InputMap, Visibility, Sign, AbiType, AbiParameter, Abi, WitnessMap, RawAssertionPayload } from "@noir-lang/types";
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object, js_name = "InputMap", typescript_type = "InputMap")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsInputMap;

    #[wasm_bindgen(extends = js_sys::Object, js_name = "InputValue", typescript_type = "InputValue")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsInputValue;

    #[wasm_bindgen(extends = js_sys::Object, js_name = "RawAssertionPayload", typescript_type = "RawAssertionPayload")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsRawAssertionPayload;

    #[wasm_bindgen(extends = js_sys::Object, js_name = "Abi", typescript_type = "Abi")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsAbi;
}

#[wasm_bindgen(js_name = abiEncode)]
pub fn abi_encode(
    abi: JsAbi,
    inputs: JsInputMap,
    return_value: Option<JsInputValue>,
) -> Result<JsWitnessMap, JsAbiError> {
    console_error_panic_hook::set_once();
    let abi: Abi =
        JsValueSerdeExt::into_serde(&JsValue::from(abi)).map_err(|err| err.to_string())?;
    let inputs: BTreeMap<String, JsonTypes> =
        JsValueSerdeExt::into_serde(&JsValue::from(inputs)).map_err(|err| err.to_string())?;
    for value in inputs.values() {
        reject_unsafe_integers(value)?;
    }
    let return_value: Option<InputValue> = return_value
        .map(|return_value| {
            let toml_return_value: JsonTypes =
                JsValueSerdeExt::into_serde(&JsValue::from(return_value))
                    .expect("could not decode return value");
            reject_unsafe_integers(&toml_return_value)?;
            InputValue::try_from_json(
                toml_return_value,
                &abi.return_type.as_ref().unwrap().abi_type,
                MAIN_RETURN_NAME,
            )
            .map_err(JsAbiError::from)
        })
        .transpose()?;

    let abi_map = abi.to_btree_map();
    let parsed_inputs: BTreeMap<String, InputValue> =
        try_btree_map(abi_map, |(arg_name, abi_type)| {
            // Check that toml contains a value for each argument in the ABI.
            let value = inputs
                .get(&arg_name)
                .ok_or_else(|| InputParserError::MissingArgument(arg_name.clone()))?;
            InputValue::try_from_json(value.clone(), &abi_type, &arg_name)
                .map(|input_value| (arg_name, input_value))
        })?;

    let witness_map = abi.encode(&parsed_inputs, return_value)?;

    Ok(witness_map.into())
}

/// The largest integer JavaScript can represent exactly, i.e. `Number.MAX_SAFE_INTEGER` (`2^53 - 1`).
const MAX_SAFE_INTEGER: i64 = (1 << 53) - 1;

/// Rejects any numeric input whose magnitude exceeds [`MAX_SAFE_INTEGER`].
///
/// JavaScript represents `number` as a 64-bit float, so an integer above `2^53 - 1` is silently
/// rounded before it ever reaches wasm. Accepting such a value would encode a witness for a
/// different field element than the one written at the call site. Callers must pass large values as
/// strings, which preserve their exact decimal representation.
fn reject_unsafe_integers(value: &JsonTypes) -> Result<(), JsAbiError> {
    match value {
        JsonTypes::Integer(integer) => {
            if integer.unsigned_abs() > MAX_SAFE_INTEGER as u64 {
                return Err(JsAbiError::new(format!(
                    "Numeric input `{integer}` is not a safe integer (its magnitude exceeds \
                     2^53 - 1, so JavaScript may have already rounded it); pass it as a string \
                     instead"
                )));
            }
        }
        JsonTypes::Array(items) => {
            for item in items {
                reject_unsafe_integers(item)?;
            }
        }
        JsonTypes::Table(map) => {
            for item in map.values() {
                reject_unsafe_integers(item)?;
            }
        }
        JsonTypes::String(_) | JsonTypes::Bool(_) => {}
    }

    Ok(())
}

#[wasm_bindgen(js_name = abiDecode)]
pub fn abi_decode(abi: JsAbi, witness_map: JsWitnessMap) -> Result<JsValue, JsAbiError> {
    console_error_panic_hook::set_once();
    let abi: Abi =
        JsValueSerdeExt::into_serde(&JsValue::from(abi)).map_err(|err| err.to_string())?;

    let witness_map = WitnessMap::from(witness_map);

    let (inputs, return_value) = abi.decode(&witness_map)?;

    let abi_types = abi.to_btree_map();
    let inputs_map: BTreeMap<String, JsonTypes> = try_btree_map(inputs, |(key, value)| {
        JsonTypes::try_from_input_value(&value, &abi_types[&key]).map(|value| (key, value))
    })?;

    let return_value = return_value
        .map(|value| JsonTypes::try_from_input_value(&value, &abi.return_type.unwrap().abi_type))
        .transpose()?;

    #[derive(Serialize)]
    struct InputsAndReturn {
        inputs: BTreeMap<String, JsonTypes>,
        return_value: Option<JsonTypes>,
    }

    let return_struct = InputsAndReturn { inputs: inputs_map, return_value };
    <JsValue as JsValueSerdeExt>::from_serde(&return_struct).map_err(|err| err.to_string().into())
}

#[wasm_bindgen(js_name = serializeWitness)]
pub fn serialize_witness(witness_map: JsWitnessMap) -> Result<Vec<u8>, JsAbiError> {
    console_error_panic_hook::set_once();
    let converted_witness: WitnessMap<FieldElement> = witness_map.into();
    let witness_stack: WitnessStack<FieldElement> = converted_witness.into();
    let output = witness_stack.serialize();
    output.map_err(|_| JsAbiError::new("Failed to serialize witness stack".to_string()))
}

#[wasm_bindgen(js_name = abiDecodeError)]
pub fn abi_decode_error(
    abi: JsAbi,
    raw_error: JsRawAssertionPayload,
) -> Result<JsValue, JsAbiError> {
    console_error_panic_hook::set_once();
    let mut abi: Abi =
        JsValueSerdeExt::into_serde(&JsValue::from(abi)).map_err(|err| err.to_string())?;

    let raw_error: RawAssertionPayload<String> =
        JsValueSerdeExt::into_serde(&JsValue::from(raw_error)).map_err(|err| err.to_string())?;
    // `FieldElement` is represented as a string in JS so we must convert these into `FieldElements` manually.
    let error_data = raw_error
        .data
        .iter()
        .map(|field| FieldElement::from_hex(field))
        .collect::<Option<Vec<_>>>()
        .unwrap();

    let error_type = abi.error_types.remove(&raw_error.selector).expect("Missing error type");
    match error_type {
        AbiErrorType::FmtString { .. } => {
            let string = display_abi_error(&error_data, error_type).to_string();
            Ok(JsValue::from_str(&string))
        }
        AbiErrorType::Custom(typ) => {
            let input_value = decode_value(&mut error_data.into_iter(), &typ, "error")?;
            let json_types = JsonTypes::try_from_input_value(&input_value, &typ)?;
            <JsValue as JsValueSerdeExt>::from_serde(&json_types)
                .map_err(|err| err.to_string().into())
        }
        AbiErrorType::String { string } => Ok(JsValue::from_str(&string)),
    }
}
