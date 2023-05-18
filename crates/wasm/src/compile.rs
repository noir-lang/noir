use gloo_utils::format::JsValueSerdeExt;
use log::debug;
use noirc_driver::{CompileOptions, Driver};
use noirc_frontend::graph::{CrateName, CrateType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use wasm_bindgen::prelude::*;

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

fn add_noir_lib(driver: &mut Driver, crate_name: &str) {
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
    let mut driver = noirc_driver::Driver::new(
        &language,
        #[allow(deprecated)]
        Box::new(acvm::default_is_opcode_supported(language.clone())),
    );

    let path = PathBuf::from(&options.entry_point);
    driver.create_local_crate(path, CrateType::Binary);

    for dependency in options.optional_dependencies_set {
        add_noir_lib(&mut driver, dependency.as_str());
    }

    // We are always adding std lib implicitly. It comes bundled with binary.
    add_noir_lib(&mut driver, "std");

    driver.check_crate(&options.compile_options).unwrap_or_else(|_| panic!("Crate check failed"));

    if options.contracts {
        let compiled_contracts = driver
            .compile_contracts(&options.compile_options)
            .unwrap_or_else(|_| panic!("Contract compilation failed"));

        <JsValue as JsValueSerdeExt>::from_serde(&compiled_contracts).unwrap()
    } else {
        let main =
            driver.main_function().unwrap_or_else(|_| panic!("Could not find main function!"));
        let compiled_program = driver
            .compile_no_check(&options.compile_options, main)
            .unwrap_or_else(|_| panic!("Compilation failed"));

        <JsValue as JsValueSerdeExt>::from_serde(&compiled_program).unwrap()
    }
}
