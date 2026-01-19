use gloo_utils::format::JsValueSerdeExt;
use js_sys::JsString;
use serde::Serialize;
use wasm_bindgen::prelude::*;

use fm::FileManager;
use noirc_errors::CustomDiagnostic;

#[wasm_bindgen(typescript_custom_section)]
const DIAGNOSTICS: &'static str = r#"
export type Diagnostic = {
    message: string;
    file: string;
    secondaries: ReadonlyArray<{
        message: string;
        start: number;
        end: number;
    }>;
}

export interface CompileError extends Error {
    message: string;
    diagnostics: ReadonlyArray<Diagnostic>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = js_sys::Error, js_name = "CompileError", typescript_type = "CompileError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsCompileError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn constructor(message: JsString) -> JsCompileError;
}

impl JsCompileError {
    const DIAGNOSTICS_PROP: &'static str = "diagnostics";
    const NAME_PROP: &'static str = "name";
    const ERROR_NAME: &'static str = "CompileError";

    pub fn new(message: String, diagnostics: Vec<Diagnostic>) -> Self {
        let err = JsCompileError::constructor(JsString::from(message));

        js_sys::Reflect::set(
            &err,
            &JsString::from(JsCompileError::NAME_PROP),
            &JsString::from(JsCompileError::ERROR_NAME),
        )
        .unwrap();

        js_sys::Reflect::set(
            &err,
            &JsString::from(JsCompileError::DIAGNOSTICS_PROP),
            &<JsValue as JsValueSerdeExt>::from_serde(&diagnostics).unwrap(),
        )
        .unwrap();

        err
    }
}

impl From<String> for JsCompileError {
    fn from(value: String) -> Self {
        JsCompileError::new(value, vec![])
    }
}

impl From<CompileError> for JsCompileError {
    fn from(value: CompileError) -> Self {
        JsCompileError::new(value.message, value.diagnostics)
    }
}

#[derive(Serialize)]
struct DiagnosticLabel {
    message: String,
    start: u32,
    end: u32,
}

#[derive(Serialize)]
pub struct Diagnostic {
    message: String,
    file: String,
    secondaries: Vec<DiagnosticLabel>,
}

impl Diagnostic {
    fn new(diagnostic: &CustomDiagnostic, file: String) -> Diagnostic {
        let message = diagnostic.message.clone();

        let secondaries = diagnostic
            .secondaries
            .iter()
            .map(|label| DiagnosticLabel {
                message: label.message.clone(),
                start: label.location.span.start(),
                end: label.location.span.end(),
            })
            .collect();

        Diagnostic { message, file, secondaries }
    }
}

#[derive(Serialize)]
pub struct CompileError {
    pub message: String,
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileError {
    pub fn new(message: &str) -> CompileError {
        CompileError { message: message.to_string(), diagnostics: vec![] }
    }

    pub fn with_custom_diagnostics(
        message: &str,
        custom_diagnostics: Vec<CustomDiagnostic>,
        file_manager: &FileManager,
    ) -> CompileError {
        let diagnostics: Vec<_> = custom_diagnostics
            .iter()
            .map(|err| {
                let file_path = file_manager
                    .path(err.file)
                    .expect("File must exist to have caused diagnostics");
                Diagnostic::new(err, file_path.to_str().unwrap().to_string())
            })
            .collect();

        CompileError { message: message.to_string(), diagnostics }
    }
}
