#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

// See Cargo.toml for explanation.
use getrandom as _;
use rust_embed as _;

use gloo_utils::format::JsValueSerdeExt;

use noirc_driver::{GIT_COMMIT, GIT_DIRTY, NOIRC_VERSION};
use serde::{Deserialize, Serialize};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;
use tracing_web::MakeWebConsoleWriter;

mod compile;
mod compile_new;
mod errors;

pub use compile::{compile_contract, compile_program};

// Expose the new Context-Centric API
pub use compile_new::{compile_contract_, compile_program_, CompilerContext, CrateIDWrapper};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    git_hash: &'static str,
    version: &'static str,
    dirty: &'static str,
}

#[wasm_bindgen]
pub fn init_log_level(level: String) {
    // Set the static variable from Rust
    use std::sync::Once;

    let level_filter: EnvFilter =
        level.parse().expect("Could not parse log filter while initializing logger");

    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .without_time()
            .with_writer(MakeWebConsoleWriter::new());

        tracing_subscriber::registry().with(fmt_layer.with_filter(level_filter)).init();
    });
}

const BUILD_INFO: BuildInfo =
    BuildInfo { git_hash: GIT_COMMIT, version: NOIRC_VERSION, dirty: GIT_DIRTY };

#[wasm_bindgen]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}
