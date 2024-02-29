use gloo_utils::format::JsValueSerdeExt;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const LOG_LEVEL: &'static str = r#"
/**
* @typedef {Object} BuildInfo - Information about how the installed package was built
* @property {string} gitHash - The hash of the git commit from which the package was built. 
* @property {string} version - The version of the package at the built git commit.
* @property {boolean} dirty - Whether the package contained uncommitted changes when built.
 */
export type BuildInfo = {
  gitHash: string;
  version: string;
  dirty: string;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "BuildInfo")]
    pub type JsBuildInfo;
}

#[derive(Serialize, Deserialize)]
struct BuildInfo {
    #[serde(rename = "gitHash")]
    git_hash: &'static str,
    version: &'static str,
    dirty: bool,
}

const BUILD_INFO: BuildInfo = BuildInfo {
    git_hash: env!("GIT_COMMIT"),
    version: env!("CARGO_PKG_VERSION"),
    dirty: const_str::equal!(env!("GIT_DIRTY"), "true"),
};

/// Returns the `BuildInfo` object containing information about how the installed package was built.
/// @returns {BuildInfo} - Information on how the installed package was built.
#[wasm_bindgen(js_name = buildInfo, skip_jsdoc)]
pub fn build_info() -> JsBuildInfo {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap().into()
}
