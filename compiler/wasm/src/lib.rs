#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]

// See Cargo.toml for explanation.
use getrandom as _;

use gloo_utils::format::JsValueSerdeExt;
use log::Level;
use noirc_driver::{GIT_COMMIT, GIT_DIRTY, NOIRC_VERSION};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

mod circuit;
mod compile;
mod compile_new;
mod errors;

pub use circuit::{acir_read_bytes, acir_write_bytes};
pub use compile::compile;

// Expose the new Context-Centric API
pub use compile_new::{compile_, CompilerContext, CrateIDWrapper};

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

    let log_level = Level::from_str(&level).unwrap_or(Level::Error);
    static SET_HOOK: Once = Once::new();
    SET_HOOK.call_once(|| {
        wasm_logger::init(wasm_logger::Config::new(log_level));
    });
}

const BUILD_INFO: BuildInfo =
    BuildInfo { git_hash: GIT_COMMIT, version: NOIRC_VERSION, dirty: GIT_DIRTY };

#[wasm_bindgen]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}
