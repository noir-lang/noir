use acvm::acir::circuit::Circuit;
use fm::FileManager;
use gloo_utils::format::JsValueSerdeExt;
use log::debug;
use noirc_driver::{
    check_crate, compile_contracts, compile_main, prepare_crate, propagate_dep, CompileOptions,
    CompiledProgram,
};
use noirc_frontend::{graph::CrateGraph, hir::Context};
use serde::{Deserialize, Serialize};
use std::path::Path;
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

fn add_noir_lib(context: &mut Context, crate_name: &str) {
    let path_to_lib = Path::new(&crate_name).join("lib.nr");
    let library_crate = prepare_crate(context, &path_to_lib);

    propagate_dep(context, library_crate, &crate_name.parse().unwrap());
}

#[wasm_bindgen]
pub fn compile(args: JsValue) -> JsValue {
    console_error_panic_hook::set_once();

    let options: WASMCompileOptions = if args.is_undefined() || args.is_null() {
        debug!("Initializing compiler with default values.");
        WASMCompileOptions::default()
    } else {
        JsValueSerdeExt::into_serde(&args).expect("Could not deserialize compile arguments")
    };

    debug!("Compiler configuration {:?}", &options);

    let root = Path::new("/");
    let fm = FileManager::new(root);
    let graph = CrateGraph::default();
    let mut context = Context::new(fm, graph);

    let path = Path::new(&options.entry_point);
    let crate_id = prepare_crate(&mut context, path);

    for dependency in options.optional_dependencies_set {
        add_noir_lib(&mut context, dependency.as_str());
    }

    check_crate(&mut context, crate_id, false).expect("Crate check failed");

    if options.contracts {
        let compiled_program = compile_contracts(&mut context, crate_id, &options.compile_options)
            .expect("Contract compilation failed")
            .0;

        let optimized_program = optimize_program(compiled_program);

        <JsValue as JsValueSerdeExt>::from_serde(&optimized_program).unwrap()
    } else {
        let compiled_program = compile_main(&mut context, crate_id, &options.compile_options)
            .expect("Compilation failed")
            .0;

        let optimized_program = optimize_program(compiled_program);

        <JsValue as JsValueSerdeExt>::from_serde(&optimized_program).unwrap()
    }
}

fn optimize_program(contract: CompiledProgram) -> CompiledProgram {
    CompiledProgram {
        name: contract.name,
        functions: contract
            .functions
            .into_iter()
            .map(|mut func| {
                func.bytecode = optimize_circuit(func.bytecode);
                func
            })
            .collect(),
    }
}

fn optimize_circuit(circuit: Circuit) -> Circuit {
    // For now we default to plonk width = 3, though we can add it as a parameter
    let language = acvm::Language::PLONKCSat { width: 3 };
    #[allow(deprecated)]
    let opcode_supported = acvm::pwg::default_is_opcode_supported(language);
    acvm::compiler::compile(circuit, language, opcode_supported)
        .expect("Circuit optimization failed")
        .0
}
