use acvm::brillig_vm::brillig::{ForeignCallOutput, ForeignCallResult, Value};
use wasm_bindgen::JsValue;

use crate::js_witness_map::js_value_to_field_element;

fn decode_foreign_call_output(output: JsValue) -> Result<ForeignCallOutput, String> {
    if output.is_string() {
        let value = Value::from(js_value_to_field_element(output)?);
        Ok(ForeignCallOutput::Single(value))
    } else if output.is_array() {
        let output = js_sys::Array::from(&output);

        let mut values: Vec<Value> = Vec::with_capacity(output.length() as usize);
        for elem in output.iter() {
            values.push(Value::from(js_value_to_field_element(elem)?))
        }
        Ok(ForeignCallOutput::Array(values))
    } else {
        return Err("Non-string-or-array element in foreign_call_handler return".into());
    }
}

pub(super) fn decode_foreign_call_result(
    js_array: js_sys::Array,
) -> Result<ForeignCallResult, String> {
    let mut values: Vec<ForeignCallOutput> = Vec::with_capacity(js_array.length() as usize);
    for elem in js_array.iter() {
        values.push(decode_foreign_call_output(elem)?);
    }
    Ok(ForeignCallResult { values })
}
