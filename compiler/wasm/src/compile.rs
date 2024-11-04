use acvm::acir::circuit::ExpressionWidth;
use fm::FileManager;
use gloo_utils::format::JsValueSerdeExt;
use js_sys::{JsString, Object};
use nargo::parse_all;
use noirc_artifacts::{contract::ContractArtifact, program::ProgramArtifact};
use noirc_driver::{
    add_dep, file_manager_with_stdlib, prepare_crate, prepare_dependency, CompileOptions,
};
use noirc_evaluator::errors::SsaReport;
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::Context,
};
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use wasm_bindgen::prelude::*;

use crate::errors::{CompileError, JsCompileError};

#[wasm_bindgen(typescript_custom_section)]
const DEPENDENCY_GRAPH: &'static str = r#"
export type DependencyGraph = {
    root_dependencies: readonly string[];
    library_dependencies: Readonly<Record<string, readonly string[]>>;
}

export type ContractOutputsArtifact = {
    structs: Record<string, Array<any>>;
    globals: Record<string, Array<any>>;
}

export type ContractArtifact = {
    noir_version: string;
    name: string;
    functions: Array<any>;
    outputs: ContractOutputsArtifact;
    file_map: Record<number, any>;
};

export type ProgramArtifact = {
    noir_version: string;
    hash: number;
    abi: any;
    bytecode: string;
    debug_symbols: any;
    file_map: Record<number, any>;
}

type WarningsCompileResult = { warnings: Array<any>; };

export type ContractCompileResult = { contract: CompiledContract; } & WarningsCompileResult;

export type ProgramCompileResult = { program: CompiledProgram; } & WarningsCompileResult;

export type CompileResult = ContractCompileResult | ProgramCompileResult;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Object, js_name = "DependencyGraph", typescript_type = "DependencyGraph")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsDependencyGraph;

    #[wasm_bindgen(extends = Object, js_name = "ProgramCompileResult", typescript_type = "ProgramCompileResult")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsCompileProgramResult;

    #[wasm_bindgen(constructor, js_class = "Object")]
    fn constructor() -> JsCompileProgramResult;

    #[wasm_bindgen(extends = Object, js_name = "ContractCompileResult", typescript_type = "ContractCompileResult")]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub type JsCompileContractResult;

    #[wasm_bindgen(constructor, js_class = "Object")]
    fn constructor() -> JsCompileContractResult;
}

impl JsCompileProgramResult {
    const PROGRAM_PROP: &'static str = "program";
    const WARNINGS_PROP: &'static str = "warnings";

    pub fn new(program: ProgramArtifact, warnings: Vec<SsaReport>) -> JsCompileProgramResult {
        let obj = JsCompileProgramResult::constructor();

        js_sys::Reflect::set(
            &obj,
            &JsString::from(JsCompileProgramResult::PROGRAM_PROP),
            &<JsValue as JsValueSerdeExt>::from_serde(&program).unwrap(),
        )
        .unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsString::from(JsCompileProgramResult::WARNINGS_PROP),
            &<JsValue as JsValueSerdeExt>::from_serde(&warnings).unwrap(),
        )
        .unwrap();

        obj
    }
}

impl JsCompileContractResult {
    const CONTRACT_PROP: &'static str = "contract";
    const WARNINGS_PROP: &'static str = "warnings";

    pub fn new(contract: ContractArtifact, warnings: Vec<SsaReport>) -> JsCompileContractResult {
        let obj = JsCompileContractResult::constructor();

        js_sys::Reflect::set(
            &obj,
            &JsString::from(JsCompileContractResult::CONTRACT_PROP),
            &<JsValue as JsValueSerdeExt>::from_serde(&contract).unwrap(),
        )
        .unwrap();
        js_sys::Reflect::set(
            &obj,
            &JsString::from(JsCompileContractResult::WARNINGS_PROP),
            &<JsValue as JsValueSerdeExt>::from_serde(&warnings).unwrap(),
        )
        .unwrap();

        obj
    }
}

#[derive(Deserialize, Default)]
pub(crate) struct DependencyGraph {
    pub(crate) root_dependencies: Vec<CrateName>,
    pub(crate) library_dependencies: HashMap<CrateName, Vec<CrateName>>,
}
#[wasm_bindgen]
// This is a map containing the paths of all of the files in the entry-point crate and
// the transitive dependencies of the entry-point crate.
//
// This is for all intents and purposes the file system that the compiler will use to resolve/compile
// files in the crate being compiled and its dependencies.
#[derive(Deserialize, Default)]
pub struct PathToFileSourceMap(pub(crate) HashMap<std::path::PathBuf, String>);

#[wasm_bindgen]
impl PathToFileSourceMap {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PathToFileSourceMap {
        PathToFileSourceMap::default()
    }
    // Inserts a path and its source code into the map.
    //
    // Returns true, if there was already source code in the map for the given path
    pub fn add_source_code(&mut self, path: String, source_code: String) -> bool {
        let path_buf = Path::new(&path).to_path_buf();
        let old_value = self.0.insert(path_buf, source_code);
        old_value.is_some()
    }
}

#[wasm_bindgen]
pub fn compile_program(
    entry_point: String,
    dependency_graph: Option<JsDependencyGraph>,
    file_source_map: PathToFileSourceMap,
) -> Result<JsCompileProgramResult, JsCompileError> {
    console_error_panic_hook::set_once();
    let (crate_id, mut context) = prepare_context(entry_point, dependency_graph, file_source_map)?;

    let expression_width = ExpressionWidth::Bounded { width: 4 };
    let compile_options =
        CompileOptions { expression_width: Some(expression_width), ..CompileOptions::default() };

    let compiled_program =
        noirc_driver::compile_main(&mut context, crate_id, &compile_options, None)
            .map_err(|errs| {
                CompileError::with_file_diagnostics(
                    "Failed to compile program",
                    errs,
                    &context.file_manager,
                )
            })?
            .0;

    let optimized_program = nargo::ops::transform_program(compiled_program, expression_width);
    nargo::ops::check_program(&optimized_program).map_err(|errs| {
        CompileError::with_file_diagnostics(
            "Compiled program is not solvable",
            errs,
            &context.file_manager,
        )
    })?;
    let warnings = optimized_program.warnings.clone();

    Ok(JsCompileProgramResult::new(optimized_program.into(), warnings))
}

#[wasm_bindgen]
pub fn compile_contract(
    entry_point: String,
    dependency_graph: Option<JsDependencyGraph>,
    file_source_map: PathToFileSourceMap,
) -> Result<JsCompileContractResult, JsCompileError> {
    console_error_panic_hook::set_once();
    let (crate_id, mut context) = prepare_context(entry_point, dependency_graph, file_source_map)?;

    let expression_width = ExpressionWidth::Bounded { width: 4 };
    let compile_options =
        CompileOptions { expression_width: Some(expression_width), ..CompileOptions::default() };

    let compiled_contract =
        noirc_driver::compile_contract(&mut context, crate_id, &compile_options)
            .map_err(|errs| {
                CompileError::with_file_diagnostics(
                    "Failed to compile contract",
                    errs,
                    &context.file_manager,
                )
            })?
            .0;

    let optimized_contract = nargo::ops::transform_contract(compiled_contract, expression_width);
    let warnings = optimized_contract.warnings.clone();

    Ok(JsCompileContractResult::new(optimized_contract.into(), warnings))
}

fn prepare_context(
    entry_point: String,
    dependency_graph: Option<JsDependencyGraph>,
    file_source_map: PathToFileSourceMap,
) -> Result<(CrateId, Context<'static, 'static>), JsCompileError> {
    let dependency_graph: DependencyGraph = if let Some(dependency_graph) = dependency_graph {
        <JsValue as JsValueSerdeExt>::into_serde(&JsValue::from(dependency_graph))
            .map_err(|err| err.to_string())?
    } else {
        DependencyGraph { root_dependencies: vec![], library_dependencies: HashMap::new() }
    };

    let fm = file_manager_with_source_map(file_source_map);
    let parsed_files = parse_all(&fm);
    let mut context = Context::new(fm, parsed_files);

    let path = Path::new(&entry_point);
    let crate_id = prepare_crate(&mut context, path);

    process_dependency_graph(&mut context, dependency_graph);

    Ok((crate_id, context))
}

// Create a new FileManager with the given source map
//
// Note: Use this method whenever initializing a new FileManager
// to ensure that the file manager contains all of the files
// that one intends the compiler to use.
//
// For all intents and purposes, the file manager being returned
// should be considered as immutable.
pub(crate) fn file_manager_with_source_map(source_map: PathToFileSourceMap) -> FileManager {
    let root = Path::new("");
    let mut fm = file_manager_with_stdlib(root);

    for (path, source) in source_map.0 {
        fm.add_file_with_source(path.as_path(), source);
    }

    fm
}

// Root dependencies are dependencies which the entry-point package relies upon.
// These will be in the Nargo.toml of the package being compiled.
//
// Library dependencies are transitive dependencies; for example, if the entry-point relies
// upon some library `lib1`. Then the packages that `lib1` depend upon will be placed in the
// `library_dependencies` list and the `lib1` will be placed in the `root_dependencies` list.
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
        let crate_id =
            *crate_names.entry(lib_name).or_insert_with(|| add_noir_lib(context, lib_name));

        for dependency_name in dependencies {
            let dep_crate_id: &CrateId = crate_names
                .entry(dependency_name)
                .or_insert_with(|| add_noir_lib(context, dependency_name));

            add_dep(context, crate_id, *dep_crate_id, dependency_name.clone());
        }
    }
}

fn add_noir_lib(context: &mut Context, library_name: &CrateName) -> CrateId {
    let path_to_lib = Path::new(&library_name.to_string()).join("lib.nr");
    prepare_dependency(context, &path_to_lib)
}

#[cfg(test)]
mod test {
    use nargo::parse_all;
    use noirc_driver::prepare_crate;
    use noirc_frontend::{graph::CrateName, hir::Context};

    use crate::compile::PathToFileSourceMap;

    use super::{file_manager_with_source_map, process_dependency_graph, DependencyGraph};
    use std::{collections::HashMap, path::Path};

    fn setup_test_context(source_map: PathToFileSourceMap) -> Context<'static, 'static> {
        let mut fm = file_manager_with_source_map(source_map);
        // Add this due to us calling prepare_crate on "/main.nr" below
        fm.add_file_with_source(Path::new("/main.nr"), "fn foo() {}".to_string());
        let parsed_files = parse_all(&fm);

        let mut context = Context::new(fm, parsed_files);
        prepare_crate(&mut context, Path::new("/main.nr"));

        context
    }

    fn crate_name(name: &str) -> CrateName {
        name.parse().unwrap()
    }

    #[test]
    fn test_works_with_empty_dependency_graph() {
        let dependency_graph =
            DependencyGraph { root_dependencies: vec![], library_dependencies: HashMap::new() };

        let source_map = PathToFileSourceMap::default();
        let mut context = setup_test_context(source_map);

        process_dependency_graph(&mut context, dependency_graph);

        // one stdlib + one root crate
        assert_eq!(context.crate_graph.number_of_crates(), 2);
    }

    #[test]
    fn test_works_with_root_dependencies() {
        let dependency_graph = DependencyGraph {
            root_dependencies: vec![crate_name("lib1")],
            library_dependencies: HashMap::new(),
        };

        let source_map = PathToFileSourceMap(
            vec![(Path::new("lib1/lib.nr").to_path_buf(), "fn foo() {}".to_string())]
                .into_iter()
                .collect(),
        );

        let mut context = setup_test_context(source_map);

        process_dependency_graph(&mut context, dependency_graph);

        assert_eq!(context.crate_graph.number_of_crates(), 3);
    }

    #[test]
    fn test_works_with_duplicate_root_dependencies() {
        let dependency_graph = DependencyGraph {
            root_dependencies: vec![crate_name("lib1"), crate_name("lib1")],
            library_dependencies: HashMap::new(),
        };

        let source_map = PathToFileSourceMap(
            vec![(Path::new("lib1/lib.nr").to_path_buf(), "fn foo() {}".to_string())]
                .into_iter()
                .collect(),
        );
        let mut context = setup_test_context(source_map);

        process_dependency_graph(&mut context, dependency_graph);

        assert_eq!(context.crate_graph.number_of_crates(), 3);
    }

    #[test]
    fn test_works_with_transitive_dependencies() {
        let dependency_graph = DependencyGraph {
            root_dependencies: vec![crate_name("lib1")],
            library_dependencies: HashMap::from([
                (crate_name("lib1"), vec![crate_name("lib2")]),
                (crate_name("lib2"), vec![crate_name("lib3")]),
            ]),
        };

        let source_map = PathToFileSourceMap(
            vec![
                (Path::new("lib1/lib.nr").to_path_buf(), "fn foo() {}".to_string()),
                (Path::new("lib2/lib.nr").to_path_buf(), "fn foo() {}".to_string()),
                (Path::new("lib3/lib.nr").to_path_buf(), "fn foo() {}".to_string()),
            ]
            .into_iter()
            .collect(),
        );

        let mut context = setup_test_context(source_map);
        process_dependency_graph(&mut context, dependency_graph);

        assert_eq!(context.crate_graph.number_of_crates(), 5);
    }

    #[test]
    fn test_works_with_missing_dependencies() {
        let dependency_graph = DependencyGraph {
            root_dependencies: vec![crate_name("lib1")],
            library_dependencies: HashMap::from([(crate_name("lib2"), vec![crate_name("lib3")])]),
        };

        let source_map = PathToFileSourceMap(
            vec![
                (Path::new("lib1/lib.nr").to_path_buf(), "fn foo() {}".to_string()),
                (Path::new("lib2/lib.nr").to_path_buf(), "fn foo() {}".to_string()),
                (Path::new("lib3/lib.nr").to_path_buf(), "fn foo() {}".to_string()),
            ]
            .into_iter()
            .collect(),
        );

        let mut context = setup_test_context(source_map);
        process_dependency_graph(&mut context, dependency_graph);

        assert_eq!(context.crate_graph.number_of_crates(), 5);
    }
}
