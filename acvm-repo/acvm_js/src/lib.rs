#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]

// TODO: Absence of per package targets
// https://doc.rust-lang.org/cargo/reference/unstable.html#per-package-target
// otherwise could be reorganized to make this file more pretty.

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod build_info;
        mod compression;
        mod execute;
        mod foreign_call;
        mod js_witness_map;
        mod logging;
        mod public_witness;
        mod js_execution_error;

        pub use build_info::build_info;
        pub use compression::{compress_witness, decompress_witness};
        pub use execute::{execute_circuit, execute_circuit_with_black_box_solver, create_black_box_solver};
        pub use js_witness_map::JsWitnessMap;
        pub use logging::{init_log_level, LogLevel};
        pub use public_witness::{get_public_parameters_witness, get_public_witness, get_return_witness};
        pub use js_execution_error::JsExecutionError;
    }
}
