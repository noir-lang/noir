use fm::FileManager;
use gloo_utils::format::JsValueSerdeExt;
use log::debug;
use noirc_driver::{
    add_dep, compile_contract, compile_main, prepare_crate, prepare_dependency, CompileOptions,
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

fn add_noir_lib(context: &mut Context, library_name: &str) {
    let path_to_lib = Path::new(&library_name).join("lib.nr");
    let library_crate_id = prepare_dependency(context, &path_to_lib);

    add_dep(context, *context.root_crate_id(), library_crate_id, library_name.parse().unwrap());

    // TODO: Remove this code that attaches every crate to every other crate as a dependency
    let root_crate_id = context.root_crate_id();
    let stdlib_crate_id = context.stdlib_crate_id();
    let other_crate_ids: Vec<_> = context
        .crate_graph
        .iter_keys()
        .filter(|crate_id| {
            // We don't want to attach this crate to itself or stdlib, nor re-attach it to the root crate
            crate_id != &library_crate_id
                && crate_id != root_crate_id
                && crate_id != stdlib_crate_id
        })
        .collect();

    for crate_id in other_crate_ids {
        context
            .crate_graph
            .add_dep(crate_id, library_name.parse().unwrap(), library_crate_id)
            .unwrap_or_else(|_| panic!("ICE: Cyclic error triggered by {} library", library_name));
    }
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

    // For now we default to plonk width = 3, though we can add it as a parameter
    let np_language = acvm::Language::PLONKCSat { width: 3 };
    #[allow(deprecated)]
    let is_opcode_supported = acvm::pwg::default_is_opcode_supported(np_language);

    if options.contracts {
        let compiled_contract = compile_contract(&mut context, crate_id, &options.compile_options)
            .expect("Contract compilation failed")
            .0;

        let optimized_contract =
            nargo::ops::optimize_contract(compiled_contract, np_language, &is_opcode_supported)
                .expect("Contract optimization failed");

        <JsValue as JsValueSerdeExt>::from_serde(&optimized_contract).unwrap()
    } else {
        let compiled_program =
            compile_main(&mut context, crate_id, &options.compile_options, None, true)
                .expect("Compilation failed")
                .0;

        let optimized_program =
            nargo::ops::optimize_program(compiled_program, np_language, &is_opcode_supported)
                .expect("Program optimization failed");

        <JsValue as JsValueSerdeExt>::from_serde(&optimized_program).unwrap()
    }
}
