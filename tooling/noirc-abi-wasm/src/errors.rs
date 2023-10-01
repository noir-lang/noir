use js_sys::{Error, JsString};
use noirc_abi::errors::{AbiError, InputParserError};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(typescript_custom_section)]
const ABI_ERROR: &'static str = r#"
export type ABIError = Error;
"#;

/// JsAbiError is a raw js error.
/// It'd be ideal that ABI error was a subclass of Error, but for that we'd need to use JS snippets or a js module.
/// Currently JS snippets don't work with a nodejs target. And a module would be too much for just a custom error type.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Error, js_name = "AbiError", typescript_type = "AbiError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsAbiError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn constructor(message: JsString) -> JsAbiError;
}

impl JsAbiError {
    /// Creates a new execution error with the given call stack.
    /// Call stacks won't be optional in the future, after removing ErrorLocation in ACVM.
    pub fn new(message: String) -> Self {
        JsAbiError::constructor(JsString::from(message))
    }
}

impl From<String> for JsAbiError {
    fn from(value: String) -> Self {
        JsAbiError::new(value)
    }
}

impl From<AbiError> for JsAbiError {
    fn from(value: AbiError) -> Self {
        JsAbiError::new(value.to_string())
    }
}

impl From<InputParserError> for JsAbiError {
    fn from(value: InputParserError) -> Self {
        JsAbiError::new(value.to_string())
    }
}
