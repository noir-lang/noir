use js_sys::JsString;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const COMPILE_ERROR: &'static str = r#"
export type CompileError = Error;
"#;

/// `CompileError` is a raw js error.
/// It'd be ideal that `CompileError` was a subclass of `Error`, but for that we'd need to use JS snippets or a js module.
/// Currently JS snippets don't work with a nodejs target. And a module would be too much for just a custom error type.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Error, js_name = "CompileError", typescript_type = "CompileError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsCompileError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn constructor(message: JsString) -> JsCompileError;
}

impl JsCompileError {
    /// Creates a new execution error with the given call stack.
    /// Call stacks won't be optional in the future, after removing ErrorLocation in ACVM.
    pub fn new(message: String) -> Self {
        JsCompileError::constructor(JsString::from(message))
    }
}
