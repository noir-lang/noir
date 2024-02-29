use acvm::acir::circuit::OpcodeLocation;
use js_sys::{Array, Error, JsString, Reflect};
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

#[wasm_bindgen(typescript_custom_section)]
const EXECUTION_ERROR: &'static str = r#"
export type ExecutionError = Error & {
    callStack?: string[];
};
"#;

/// JsExecutionError is a raw js error.
/// It'd be ideal that execution error was a subclass of Error, but for that we'd need to use JS snippets or a js module.
/// Currently JS snippets don't work with a nodejs target. And a module would be too much for just a custom error type.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Error, js_name = "ExecutionError", typescript_type = "ExecutionError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsExecutionError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn constructor(message: JsString) -> JsExecutionError;
}

impl JsExecutionError {
    /// Creates a new execution error with the given call stack.
    /// Call stacks won't be optional in the future, after removing ErrorLocation in ACVM.
    pub fn new(message: String, call_stack: Option<Vec<OpcodeLocation>>) -> Self {
        let mut error = JsExecutionError::constructor(JsString::from(message));
        let js_call_stack = match call_stack {
            Some(call_stack) => {
                let js_array = Array::new();
                for loc in call_stack {
                    js_array.push(&JsValue::from(format!("{}", loc)));
                }
                js_array.into()
            }
            None => JsValue::UNDEFINED,
        };

        error.set_property("callStack", js_call_stack);

        error
    }

    fn set_property(&mut self, property: &str, value: JsValue) {
        assert!(
            Reflect::set(self, &JsValue::from(property), &value).expect("Errors should be objects"),
            "Errors should be writable"
        );
    }
}
