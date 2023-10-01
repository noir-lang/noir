use js_sys::JsString;
use log::Level;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const LOG_LEVEL: &'static str = r#"
export type LogLevel = "OFF" | "ERROR" | "WARN" | "INFO" | "DEBUG" | "TRACE";
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = JsString, typescript_type = "LogLevel")]
    pub type LogLevel;
}

/// Sets the package's logging level.
///
/// @param {LogLevel} level - The maximum level of logging to be emitted.
#[wasm_bindgen(js_name = initLogLevel, skip_jsdoc)]
pub fn init_log_level(level: LogLevel) {
    // Set the static variable from Rust
    use std::sync::Once;

    let log_level = level.as_string().unwrap();
    let log_level = Level::from_str(&log_level).unwrap_or(Level::Error);
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        wasm_logger::init(wasm_logger::Config::new(log_level));
    });
}
