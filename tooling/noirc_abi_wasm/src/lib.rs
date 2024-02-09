#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

// See Cargo.toml for explanation.
use getrandom as _;

use acvm::acir::native_types::WitnessMap;
use iter_extended::try_btree_map;
use noirc_abi::{
    errors::InputParserError,
    input_parser::{json::JsonTypes, InputValue},
    Abi, MAIN_RETURN_NAME,
};
use serde::Serialize;
use std::collections::BTreeMap;

use gloo_utils::format::JsValueSerdeExt;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod errors;
mod js_witness_map;

use errors::JsAbiError;
use js_witness_map::JsWitnessMap;

#[wasm_bindgen(typescript_custom_section)]
const INPUT_MAP: &'static str = r#"
import { Field, InputValue, InputMap, Visibility, Sign, AbiType, AbiParameter, Abi, WitnessMap } from "@noir-lang/types";
export { Field, InputValue, InputMap, Visibility, Sign, AbiType, AbiParameter, Abi, WitnessMap } from "@noir-lang/types";
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Object, js_name = "InputMap", typescript_type = "InputMap")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsInputMap;

    #[wasm_bindgen(extends = js_sys::Object, js_name = "InputValue", typescript_type = "InputValue")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsInputValue;

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
    let return_value: Option<InputValue> = return_value
        .map(|return_value| {
            let toml_return_value = JsValueSerdeExt::into_serde(&JsValue::from(return_value))
                .expect("could not decode return value");
            InputValue::try_from_json(
                toml_return_value,
                &abi.return_type.as_ref().unwrap().abi_type,
                MAIN_RETURN_NAME,
            )
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
    <wasm_bindgen::JsValue as JsValueSerdeExt>::from_serde(&return_struct)
        .map_err(|err| err.to_string().into())
}
