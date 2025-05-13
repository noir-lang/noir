use js_sys::{Error, JsString};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::prelude::*;
use tracing_web::MakeWebConsoleWriter;
use wasm_bindgen::prelude::*;

fn set_log_level(filter: EnvFilter) -> bool {
    use std::sync::Once;

    static SET_HOOK: Once = Once::new();

    if SET_HOOK.is_completed() {
        return false;
    }

    SET_HOOK.call_once(|| {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .without_time()
            .with_writer(MakeWebConsoleWriter::new());

        tracing_subscriber::registry().with(fmt_layer.with_filter(filter)).init();
    });

    true
}

/// Sets the package's logging level.
///
/// @param {LogLevel} level - The maximum level of logging to be emitted.
#[wasm_bindgen(js_name = initLogLevel, skip_jsdoc)]
pub fn init_log_level(filter: String) -> Result<(), JsLogInitError> {
    // Set the static variable from Rust

    let filter: EnvFilter = filter.parse().map_err(|err| {
        JsLogInitError::constructor(
            format!("Could not parse log filter while initializing logger: {err}").into(),
        )
    })?;

    if set_log_level(filter) {
        Ok(())
    } else {
        Err(JsLogInitError::constructor("log filter level already set".to_string().into()))
    }
}

/// `LogInitError` is a raw js error.
/// It'd be ideal that `LogInitError` was a subclass of Error, but for that we'd need to use JS snippets or a js module.
/// Currently JS snippets don't work with a nodejs target. And a module would be too much for just a custom error type.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Error, js_name = "LogInitError", typescript_type = "LogInitError")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsLogInitError;

    #[wasm_bindgen(constructor, js_class = "Error")]
    fn constructor(message: JsString) -> JsLogInitError;
}
