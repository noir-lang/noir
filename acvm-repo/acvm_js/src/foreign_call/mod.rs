use acvm::{brillig_vm::brillig::ForeignCallResult, pwg::ForeignCallWaitInfo, FieldElement};

use js_sys::{Error, JsString};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

mod inputs;
mod outputs;

#[wasm_bindgen(typescript_custom_section)]
const FOREIGN_CALL_HANDLER: &'static str = r#"
export type ForeignCallInput = string[]
export type ForeignCallOutput = string | string[]

/**
* A callback which performs an foreign call and returns the response.
* @callback ForeignCallHandler
* @param {string} name - The identifier for the type of foreign call being performed.
* @param {string[][]} inputs - An array of hex encoded inputs to the foreign call.
* @returns {Promise<string[]>} outputs - An array of hex encoded outputs containing the results of the foreign call.
*/
export type ForeignCallHandler = (name: string, inputs: ForeignCallInput[]) => Promise<ForeignCallOutput[]>;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "ForeignCallHandler")]
    pub type ForeignCallHandler;
}

pub(super) async fn resolve_brillig(
    foreign_call_callback: &ForeignCallHandler,
    foreign_call_wait_info: &ForeignCallWaitInfo<FieldElement>,
) -> Result<ForeignCallResult<FieldElement>, Error> {
    // Prepare to call
    let name = JsString::from(foreign_call_wait_info.function.clone());
    let inputs = inputs::encode_foreign_call_inputs(&foreign_call_wait_info.inputs);

    // Perform foreign call
    let outputs = perform_foreign_call(foreign_call_callback, name, inputs).await?;

    // The Brillig VM checks that the number of return values from
    // the foreign call is valid so we don't need to do it here.
    outputs::decode_foreign_call_result(outputs).map_err(|message| Error::new(&message))
}

async fn perform_foreign_call(
    foreign_call_handler: &ForeignCallHandler,
    name: JsString,
    inputs: js_sys::Array,
) -> Result<js_sys::Array, Error> {
    // Call and await
    let this = JsValue::null();
    let ret_js_val = foreign_call_handler
        .call2(&this, &name, &inputs)
        .map_err(|err| wrap_js_error("Error calling `foreign_call_callback`", &err))?;
    let ret_js_prom: js_sys::Promise = ret_js_val.into();
    let ret_future: wasm_bindgen_futures::JsFuture = ret_js_prom.into();
    let js_resolution = ret_future
        .await
        .map_err(|err| wrap_js_error("Error awaiting `foreign_call_handler`", &err))?;

    // Check that result conforms to expected shape.
    if !js_resolution.is_array() {
        return Err(Error::new("Expected `foreign_call_handler` to return an array"));
    }

    Ok(js_sys::Array::from(&js_resolution))
}

fn wrap_js_error(message: &str, err: &JsValue) -> Error {
    let new_error = Error::new(message);
    new_error.set_cause(err);
    new_error
}
