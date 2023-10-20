use fm::FileManager;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::Object;
use nargo::artifacts::{
    contract::{PreprocessedContract, PreprocessedContractFunction},
    program::PreprocessedProgram,
};
use noirc_driver::{
    add_dep, compile_contract, compile_main, prepare_crate, prepare_dependency, CompileOptions,
    CompiledContract, CompiledProgram,
};
use noirc_frontend::{
    graph::{CrateGraph, CrateId, CrateName},
    hir::Context,
};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use wasm_bindgen::prelude::*;

use crate::errors::{CompileError, JsCompileError};

const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

#[wasm_bindgen(typescript_custom_section)]
const DEPENDENCY_GRAPH: &'static str = r#"
export type DependencyGraph = {
    root_dependencies: readonly string[];
    library_dependencies: Readonly<Record<string, readonly string[]>>;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, js_name = "DependencyGraph", typescript_type = "DependencyGraph")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsDependencyGraph;
}

#[derive(Deserialize)]
struct DependencyGraph {
    root_dependencies: Vec<CrateName>,
    library_dependencies: HashMap<CrateName, Vec<CrateName>>,
}

#[wasm_bindgen]
pub fn compile(
    entry_point: String,
    contracts: Option<bool>,
    js_dependency_graph: Option<JsDependencyGraph>,
) -> Result<JsValue, JsCompileError> {
    console_error_panic_hook::set_once();

    let dependency_graph: DependencyGraph = if let Some(js_dependency_graph) = js_dependency_graph {
        <JsValue as JsValueSerdeExt>::into_serde(&JsValue::from(js_dependency_graph))
            .map_err(|err| err.to_string())?
    } else {
        DependencyGraph { root_dependencies: vec![], library_dependencies: HashMap::new() }
    };

    let root = Path::new("/");
    let fm = FileManager::new(root, Box::new(get_non_stdlib_asset));
    let graph = CrateGraph::default();
    let mut context = Context::new(fm, graph);

    let path = Path::new(&entry_point);
    let crate_id = prepare_crate(&mut context, path);

    process_dependency_graph(&mut context, dependency_graph);

    let compile_options = CompileOptions::default();

    // For now we default to plonk width = 3, though we can add it as a parameter
    let np_language = acvm::Language::PLONKCSat { width: 3 };
    #[allow(deprecated)]
    let is_opcode_supported = acvm::pwg::default_is_opcode_supported(np_language);

    if contracts.unwrap_or_default() {
        let compiled_contract = compile_contract(&mut context, crate_id, &compile_options)
            .map_err(|errs| {
                CompileError::with_file_diagnostics(
                    "Failed to compile contract",
                    errs,
                    &context.file_manager,
                )
            })?
            .0;

        let optimized_contract =
            nargo::ops::optimize_contract(compiled_contract, np_language, &is_opcode_supported)
                .expect("Contract optimization failed");

        let preprocessed_contract = preprocess_contract(optimized_contract);

        Ok(<JsValue as JsValueSerdeExt>::from_serde(&preprocessed_contract).unwrap())
    } else {
        let compiled_program = compile_main(&mut context, crate_id, &compile_options, None, true)
            .map_err(|errs| {
                CompileError::with_file_diagnostics(
                    "Failed to compile program",
                    errs,
                    &context.file_manager,
                )
            })?
            .0;

        let optimized_program =
            nargo::ops::optimize_program(compiled_program, np_language, &is_opcode_supported)
                .expect("Program optimization failed");

        let preprocessed_program = preprocess_program(optimized_program);

        Ok(<JsValue as JsValueSerdeExt>::from_serde(&preprocessed_program).unwrap())
    }
}

fn process_dependency_graph(context: &mut Context, dependency_graph: DependencyGraph) {
    let mut crate_names: HashMap<&CrateName, CrateId> = HashMap::new();

    for lib in &dependency_graph.root_dependencies {
        let crate_id = add_noir_lib(context, lib);
        crate_names.insert(lib, crate_id);

        add_dep(context, *context.root_crate_id(), crate_id, lib.clone());
    }

    for (lib_name, dependencies) in &dependency_graph.library_dependencies {
        // first create the library crate if needed
        // this crate might not have been registered yet because of the order of the HashMap
        // e.g. {root: [lib1], libs: { lib2 -> [lib3], lib1 -> [lib2] }}
        let crate_id = crate_names
            .entry(&lib_name)
            .or_insert_with(|| add_noir_lib(context, &lib_name))
            .clone();

        for dependency_name in dependencies {
            let dep_crate_id: &CrateId = crate_names
                .entry(&dependency_name)
                .or_insert_with(|| add_noir_lib(context, &dependency_name));

            add_dep(context, crate_id, *dep_crate_id, dependency_name.clone());
        }
    }
}

fn add_noir_lib(context: &mut Context, library_name: &CrateName) -> CrateId {
    let path_to_lib = Path::new(&library_name.to_string()).join("lib.nr");
    let library_crate_id = prepare_dependency(context, &path_to_lib);

    library_crate_id
}

fn preprocess_program(program: CompiledProgram) -> PreprocessedProgram {
    PreprocessedProgram {
        hash: program.hash,
        backend: String::from(BACKEND_IDENTIFIER),
        abi: program.abi,
        bytecode: program.circuit,
    }
}

fn preprocess_contract(contract: CompiledContract) -> PreprocessedContract {
    let preprocessed_functions = contract
        .functions
        .into_iter()
        .map(|func| PreprocessedContractFunction {
            name: func.name,
            function_type: func.function_type,
            is_internal: func.is_internal,
            abi: func.abi,
            bytecode: func.bytecode,
        })
        .collect();

    PreprocessedContract {
        name: contract.name,
        backend: String::from(BACKEND_IDENTIFIER),
        functions: preprocessed_functions,
        events: contract.events,
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "wasi")] {
        fn get_non_stdlib_asset(path_to_file: &Path) -> std::io::Result<String> {
            std::fs::read_to_string(path_to_file)
        }
    } else {
        use std::io::{Error, ErrorKind};

        #[wasm_bindgen(module = "@noir-lang/source-resolver")]
        extern "C" {
            #[wasm_bindgen(catch)]
            fn read_file(path: &str) -> Result<String, JsValue>;
        }

        fn get_non_stdlib_asset(path_to_file: &Path) -> std::io::Result<String> {
            let path_str = path_to_file.to_str().unwrap();
            match read_file(path_str) {
                Ok(buffer) => Ok(buffer),
                Err(_) => Err(Error::new(ErrorKind::Other, "could not read file using wasm")),
            }
        }
    }
}
