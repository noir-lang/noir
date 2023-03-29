#![forbid(unsafe_code)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(unreachable_pub)]
use acvm::acir::circuit::Circuit;
use gloo_utils::format::JsValueSerdeExt;
use log::{debug, Level};
use noirc_driver::{CompileOptions, Driver};
use noirc_frontend::graph::{CrateName, CrateType};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct BuildInfo {
    git_hash: &'static str,
    version: &'static str,
    dirty: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WASMCompileOptions {
    #[serde(default = "default_entry_point")]
    entry_point: String,

    #[serde(default = "default_circuit_name")]
    circuit_name: String,

    // Compile each contract function used within the program
    #[serde(default = "bool::default")]
    contracts: bool,

    #[serde(default)]
    compile_options: CompileOptions,

    #[serde(default)]
    optional_dependencies_set: Vec<String>,

    #[serde(default = "default_log_level")]
    log_level: String,
}

fn default_log_level() -> String {
    String::from("info")
}

fn default_circuit_name() -> String {
    String::from("contract")
}

fn default_entry_point() -> String {
    String::from("main.nr")
}

impl Default for WASMCompileOptions {
    fn default() -> Self {
        Self {
            entry_point: default_entry_point(),
            circuit_name: default_circuit_name(),
            log_level: default_log_level(),
            contracts: false,
            compile_options: CompileOptions::default(),
            optional_dependencies_set: vec![],
        }
    }
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
const BUILD_INFO: BuildInfo = BuildInfo {
    git_hash: env!("GIT_COMMIT"),
    version: env!("CARGO_PKG_VERSION"),
    dirty: env!("GIT_DIRTY"),
};
pub fn add_noir_lib(driver: &mut Driver, crate_name: &str) {
    let path_to_lib = PathBuf::from(&crate_name).join("lib.nr");
    let library_crate = driver.create_non_local_crate(path_to_lib, CrateType::Library);

    driver.propagate_dep(library_crate, &CrateName::new(crate_name).unwrap());
}

#[wasm_bindgen]
pub fn compile(args: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: WASMCompileOptions = if args.is_undefined() || args.is_null() {
        debug!("Initializing compiler with default values.");
        WASMCompileOptions::default()
    } else {
        JsValueSerdeExt::into_serde(&args)
            .unwrap_or_else(|_| panic!("Could not deserialize compile arguments"))
    };

    debug!("Compiler configuration {:?}", &options);

    // For now we default to plonk width = 3, though we can add it as a parameter
    let language = acvm::Language::PLONKCSat { width: 3 };
    let mut driver = noirc_driver::Driver::new(&language);

    let path = PathBuf::from(&options.entry_point);
    driver.create_local_crate(path, CrateType::Binary);

    // We are always adding std lib implicitly. It comes bundled with binary.
    add_noir_lib(&mut driver, "std");

    for dependency in options.optional_dependencies_set {
        add_noir_lib(&mut driver, dependency.as_str());
    }

    driver.check_crate(&options.compile_options).unwrap_or_else(|_| panic!("Crate check failed"));

    if options.contracts {
        let compiled_contracts = driver
            .compile_contracts(&options.compile_options)
            .unwrap_or_else(|_| panic!("Contract compilation failed"));

        // Flatten each contract into a list of its functions, each being assigned a unique name.
        let collected_compiled_programs: Vec<_> = compiled_contracts
            .into_iter()
            .flat_map(|contract| {
                let contract_id = format!("{}-{}", options.circuit_name, &contract.name);
                contract.functions.into_iter().map(move |function| {
                    let program_name = format!("{}-{}", contract_id, &function.name);
                    (program_name, function.bytecode)
                })
            })
            .collect();

        <JsValue as JsValueSerdeExt>::from_serde(&collected_compiled_programs).unwrap()
    } else {
        let main =
            driver.main_function().unwrap_or_else(|_| panic!("Could not find main function!"));
        let compiled_program = driver
            .compile_no_check(&options.compile_options, main)
            .unwrap_or_else(|_| panic!("Compilation failed"));

        <JsValue as JsValueSerdeExt>::from_serde(&compiled_program).unwrap()
    }
}

// Deserializes bytes into ACIR structure
#[deprecated(
    note = "we have moved away from this serialization strategy. Call `acir_read_bytes` instead"
)]
#[allow(deprecated)]
#[wasm_bindgen]
pub fn acir_from_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::from_bytes(&bytes);
    <JsValue as JsValueSerdeExt>::from_serde(&circuit).unwrap()
}

#[deprecated(
    note = "we have moved away from this serialization strategy. Call `acir_write_bytes` instead"
)]
#[allow(deprecated)]
#[wasm_bindgen]
pub fn acir_to_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = JsValueSerdeExt::into_serde(&acir).unwrap();
    circuit.to_bytes()
}

// Deserializes bytes into ACIR structure
#[wasm_bindgen]
pub fn acir_read_bytes(bytes: Vec<u8>) -> JsValue {
    console_error_panic_hook::set_once();
    let circuit = Circuit::read(&*bytes).unwrap();
    <JsValue as JsValueSerdeExt>::from_serde(&circuit).unwrap()
}

#[wasm_bindgen]
pub fn acir_write_bytes(acir: JsValue) -> Vec<u8> {
    console_error_panic_hook::set_once();
    let circuit: Circuit = JsValueSerdeExt::into_serde(&acir).unwrap();
    let mut bytes = Vec::new();
    circuit.write(&mut bytes).unwrap();
    bytes
}

#[wasm_bindgen]
pub fn build_info() -> JsValue {
    console_error_panic_hook::set_once();
    <JsValue as JsValueSerdeExt>::from_serde(&BUILD_INFO).unwrap()
}
