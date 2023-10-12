use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use fm::FileManager;
use noirc_errors::FileDiagnostic;

#[wasm_bindgen(typescript_custom_section)]
const DIAGNOSTICS: &'static str = r#"
export type Diagnostic = {
    message: string;
    file_path: string;
    secondaries: ReadonlyArray<{
        message: string;
        start: number;
        end: number;
    }>;
}

interface CompileError {
    diagnostics: ReadonlyArray<Diagnostic>;
}
"#;

#[derive(Serialize, Deserialize)]
struct JsDiagnosticLabel {
    message: String,
    start: u32,
    end: u32,
}

#[derive(Serialize, Deserialize)]
struct JsDiagnostic {
    message: String,
    file_path: String,
    secondaries: Vec<JsDiagnosticLabel>,
}

impl JsDiagnostic {
    fn new(file_diagnostic: &FileDiagnostic, file_path: String) -> JsDiagnostic {
        let diagnostic = &file_diagnostic.diagnostic;
        let message = diagnostic.message.clone();

        let secondaries = diagnostic
            .secondaries
            .iter()
            .map(|label| JsDiagnosticLabel {
                message: label.message.clone(),
                start: label.span.start(),
                end: label.span.end(),
            })
            .collect();

        JsDiagnostic { message, file_path, secondaries }
    }
}

#[wasm_bindgen(getter_with_clone, js_name = "CompileError")]
pub struct JsCompileError {
    pub message: js_sys::JsString,
    pub diagnostics: JsValue,
}

impl JsCompileError {
    pub fn new(
        message: &str,
        file_diagnostics: Vec<FileDiagnostic>,
        file_manager: &FileManager,
    ) -> JsCompileError {
        let diagnostics: Vec<_> = file_diagnostics
            .iter()
            .map(|err| {
                JsDiagnostic::new(err, file_manager.path(err.file_id).to_str().unwrap().to_string())
            })
            .collect();

        JsCompileError {
            message: js_sys::JsString::from(message.to_string()),
            diagnostics: <JsValue as JsValueSerdeExt>::from_serde(&diagnostics).unwrap(),
        }
    }
}
