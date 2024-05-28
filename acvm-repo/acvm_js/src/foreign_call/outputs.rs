use acvm::{
    brillig_vm::brillig::{ForeignCallParam, ForeignCallResult},
    FieldElement,
};
use wasm_bindgen::JsValue;

use crate::js_witness_map::js_value_to_field_element;

fn decode_foreign_call_output(output: JsValue) -> Result<ForeignCallParam<FieldElement>, String> {
    if output.is_string() {
        let value = js_value_to_field_element(output)?;
        Ok(ForeignCallParam::Single(value))
    } else if output.is_array() {
        let output = js_sys::Array::from(&output);

        let mut values: Vec<_> = Vec::with_capacity(output.length() as usize);
        for elem in output.iter() {
            values.push(js_value_to_field_element(elem)?);
        }
        Ok(ForeignCallParam::Array(values))
    } else {
        return Err("Non-string-or-array element in foreign_call_handler return".into());
    }
}

pub(super) fn decode_foreign_call_result(
    js_array: js_sys::Array,
) -> Result<ForeignCallResult<FieldElement>, String> {
    let mut values: Vec<ForeignCallParam<FieldElement>> =
        Vec::with_capacity(js_array.length() as usize);
    for elem in js_array.iter() {
        values.push(decode_foreign_call_output(elem)?);
    }
    Ok(ForeignCallResult { values })
}
