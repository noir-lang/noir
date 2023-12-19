use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_web::MakeWebConsoleWriter;
use wasm_bindgen::prelude::*;

/// Sets the package's logging level.
///
/// @param {LogLevel} level - The maximum level of logging to be emitted.
#[wasm_bindgen(js_name = initLogLevel, skip_jsdoc)]
pub fn init_log_level(filter: String) {
    // Set the static variable from Rust
    use std::sync::Once;

    let filter: EnvFilter =
        filter.parse().expect("Could not parse log filter while initializing logger");

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .without_time()
            .with_writer(MakeWebConsoleWriter::new());

        tracing_subscriber::registry().with(fmt_layer.with_filter(filter)).init();
    });
}
