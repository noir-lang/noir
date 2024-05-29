use acvm::{brillig_vm::brillig::ForeignCallParam, FieldElement};

use crate::js_witness_map::field_element_to_js_string;

pub(super) fn encode_foreign_call_inputs(
    foreign_call_inputs: &[ForeignCallParam<FieldElement>],
) -> js_sys::Array {
    let inputs = js_sys::Array::default();
    for input in foreign_call_inputs {
        let input_array = js_sys::Array::default();
        for value in input.fields() {
            let hex_js_string = field_element_to_js_string(&value);
            input_array.push(&hex_js_string);
        }
        inputs.push(&input_array);
    }

    inputs
}
