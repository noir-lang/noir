use crate::compile::{
    file_manager_with_source_map, JsCompileContractResult, JsCompileProgramResult,
    PathToFileSourceMap,
};
use crate::errors::{CompileError, JsCompileError};
use acvm::acir::circuit::ExpressionWidth;
use nargo::parse_all;
use noirc_driver::{
    add_dep, compile_contract, compile_main, prepare_crate, prepare_dependency, CompileOptions,
};
use noirc_frontend::{
    graph::{CrateId, CrateName},
    hir::Context,
};
use std::path::Path;
use wasm_bindgen::prelude::wasm_bindgen;

/// This is a wrapper class that is wasm-bindgen compatible
/// We do not use js_name and rename it like CrateId because
/// then the impl block is not picked up in javascript.
#[wasm_bindgen]
pub struct CompilerContext {
    // `wasm_bindgen` currently doesn't allow lifetime parameters on structs so we must use a `'static` lifetime.
    // `Context` must then own the `FileManager` to satisfy this lifetime.
    context: Context<'static, 'static>,
}

#[wasm_bindgen(js_name = "CrateId")]
#[derive(Debug, Copy, Clone)]
pub struct CrateIDWrapper(CrateId);

#[wasm_bindgen]
impl CompilerContext {
    #[wasm_bindgen(constructor)]
    pub fn new(source_map: PathToFileSourceMap) -> CompilerContext {
        console_error_panic_hook::set_once();

        let fm = file_manager_with_source_map(source_map);
        let parsed_files = parse_all(&fm);

        CompilerContext { context: Context::new(fm, parsed_files) }
    }

    #[cfg(test)]
    pub(crate) fn crate_graph(&self) -> &noirc_frontend::graph::CrateGraph {
        &self.context.crate_graph
    }
    #[cfg(test)]
    pub(crate) fn root_crate_id(&self) -> CrateIDWrapper {
        CrateIDWrapper(*self.context.root_crate_id())
    }

    // Processes the root crate by adding it to the package graph and automatically
    // importing the stdlib as a dependency for it.
    //
    // Its ID in the package graph is returned
    pub fn process_root_crate(&mut self, path_to_crate: String) -> CrateIDWrapper {
        let path_to_crate = Path::new(&path_to_crate);

        // Adds the root crate to the crate graph and returns its crate id
        CrateIDWrapper(prepare_crate(&mut self.context, path_to_crate))
    }

    pub fn process_dependency_crate(&mut self, path_to_crate: String) -> CrateIDWrapper {
        let path_to_crate = Path::new(&path_to_crate);

        // Adds the root crate to the crate graph and returns its crate id
        CrateIDWrapper(prepare_dependency(&mut self.context, path_to_crate))
    }

    // Adds a named edge from one crate to the other.
    //
    // For example, lets say we have two crates CrateId1 and CrateId2
    // This function will add an edge from CrateId1 to CrateId2 and the edge will be named `crate_name`
    //
    // This essentially says that CrateId1 depends on CrateId2 and the dependency is named `crate_name`
    //
    // We pass references to &CrateIdWrapper even though it is a copy because Rust's move semantics are
    // not respected once we use javascript. ie it will actually allocated a new object in javascript
    // then deallocate that object if we do not pass as a reference.
    pub fn add_dependency_edge(
        &mut self,
        crate_name: String,
        from: &CrateIDWrapper,
        to: &CrateIDWrapper,
    ) -> Result<(), JsCompileError> {
        let parsed_crate_name: CrateName =
            crate_name.parse().map_err(|err_string| JsCompileError::new(err_string, Vec::new()))?;

        add_dep(&mut self.context, from.0, to.0, parsed_crate_name);
        Ok(())
    }

    pub fn compile_program(
        mut self,
        program_width: usize,
    ) -> Result<JsCompileProgramResult, JsCompileError> {
        let expression_width = if program_width == 0 {
            ExpressionWidth::Unbounded
        } else {
            ExpressionWidth::Bounded { width: 4 }
        };
        let compile_options = CompileOptions {
            expression_width: Some(expression_width),
            ..CompileOptions::default()
        };

        let root_crate_id = *self.context.root_crate_id();
        let compiled_program =
            compile_main(&mut self.context, root_crate_id, &compile_options, None)
                .map_err(|errs| {
                    CompileError::with_file_diagnostics(
                        "Failed to compile program",
                        errs,
                        &self.context.file_manager,
                    )
                })?
                .0;

        let optimized_program = nargo::ops::transform_program(compiled_program, expression_width);
        nargo::ops::check_program(&optimized_program).map_err(|errs| {
            CompileError::with_file_diagnostics(
                "Compiled program is not solvable",
                errs,
                &self.context.file_manager,
            )
        })?;
        let warnings = optimized_program.warnings.clone();

        Ok(JsCompileProgramResult::new(optimized_program.into(), warnings))
    }

    pub fn compile_contract(
        mut self,
        program_width: usize,
    ) -> Result<JsCompileContractResult, JsCompileError> {
        let expression_width = if program_width == 0 {
            ExpressionWidth::Unbounded
        } else {
            ExpressionWidth::Bounded { width: 4 }
        };
        let compile_options = CompileOptions {
            expression_width: Some(expression_width),
            ..CompileOptions::default()
        };

        let root_crate_id = *self.context.root_crate_id();
        let compiled_contract =
            compile_contract(&mut self.context, root_crate_id, &compile_options)
                .map_err(|errs| {
                    CompileError::with_file_diagnostics(
                        "Failed to compile contract",
                        errs,
                        &self.context.file_manager,
                    )
                })?
                .0;

        let optimized_contract =
            nargo::ops::transform_contract(compiled_contract, expression_width);
        let warnings = optimized_contract.warnings.clone();

        Ok(JsCompileContractResult::new(optimized_contract.into(), warnings))
    }
}

/// This is a method that exposes the same API as `compile`
/// But uses the Context based APi internally
#[wasm_bindgen]
pub fn compile_program_(
    entry_point: String,
    dependency_graph: Option<crate::compile::JsDependencyGraph>,
    file_source_map: PathToFileSourceMap,
) -> Result<JsCompileProgramResult, JsCompileError> {
    console_error_panic_hook::set_once();

    let compiler_context =
        prepare_compiler_context(entry_point, dependency_graph, file_source_map)?;
    let program_width = 4;

    compiler_context.compile_program(program_width)
}

/// This is a method that exposes the same API as `compile`
/// But uses the Context based APi internally
#[wasm_bindgen]
pub fn compile_contract_(
    entry_point: String,
    dependency_graph: Option<crate::compile::JsDependencyGraph>,
    file_source_map: PathToFileSourceMap,
) -> Result<JsCompileContractResult, JsCompileError> {
    console_error_panic_hook::set_once();

    let compiler_context =
        prepare_compiler_context(entry_point, dependency_graph, file_source_map)?;
    let program_width = 4;

    compiler_context.compile_contract(program_width)
}

/// This is a method that exposes the same API as `prepare_context`
/// But uses the Context based API internally
fn prepare_compiler_context(
    entry_point: String,
    dependency_graph: Option<crate::compile::JsDependencyGraph>,
    file_source_map: PathToFileSourceMap,
) -> Result<CompilerContext, JsCompileError> {
    use std::collections::HashMap;

    let dependency_graph: crate::compile::DependencyGraph =
        if let Some(dependency_graph) = dependency_graph {
            <wasm_bindgen::JsValue as gloo_utils::format::JsValueSerdeExt>::into_serde(
                &wasm_bindgen::JsValue::from(dependency_graph),
            )
            .map_err(|err| err.to_string())?
        } else {
            crate::compile::DependencyGraph::default()
        };

    let mut compiler_context = CompilerContext::new(file_source_map);

    // Set the root crate
    let root_id = compiler_context.process_root_crate(entry_point.clone());

    let add_noir_lib = |context: &mut CompilerContext, lib_name: &CrateName| -> CrateIDWrapper {
        let lib_name_string = lib_name.to_string();
        let path_to_lib = Path::new(&lib_name_string)
            .join("lib.nr")
            .to_str()
            .expect("paths are expected to be valid utf-8")
            .to_string();
        context.process_dependency_crate(path_to_lib)
    };

    // Add the dependency graph
    let mut crate_names: HashMap<CrateName, CrateIDWrapper> = HashMap::new();
    //
    // Process the direct dependencies of the root
    for lib_name in dependency_graph.root_dependencies {
        let lib_name_string = lib_name.to_string();

        let crate_id = add_noir_lib(&mut compiler_context, &lib_name);

        crate_names.insert(lib_name.clone(), crate_id);

        // Add the dependency edges
        compiler_context.add_dependency_edge(lib_name_string, &root_id, &crate_id)?;
    }

    // Process the transitive dependencies of the root
    for (lib_name, dependencies) in &dependency_graph.library_dependencies {
        // first create the library crate if needed
        // this crate might not have been registered yet because of the order of the HashMap
        // e.g. {root: [lib1], libs: { lib2 -> [lib3], lib1 -> [lib2] }}
        let crate_id = *crate_names
            .entry(lib_name.clone())
            .or_insert_with(|| add_noir_lib(&mut compiler_context, lib_name));

        for dependency_name in dependencies {
            let dependency_name_string = dependency_name.to_string();

            let dep_crate_id = crate_names
                .entry(dependency_name.clone())
                .or_insert_with(|| add_noir_lib(&mut compiler_context, dependency_name));

            compiler_context.add_dependency_edge(
                dependency_name_string,
                &crate_id,
                dep_crate_id,
            )?;
        }
    }

    Ok(compiler_context)
}

#[cfg(test)]
mod test {
    use nargo::parse_all;
    use noirc_driver::prepare_crate;
    use noirc_frontend::hir::Context;

    use crate::compile::{file_manager_with_source_map, PathToFileSourceMap};

    use std::path::Path;

    use super::CompilerContext;

    fn setup_test_context(source_map: PathToFileSourceMap) -> CompilerContext {
        let mut fm = file_manager_with_source_map(source_map);
        // Add this due to us calling prepare_crate on "/main.nr" below
        fm.add_file_with_source(Path::new("/main.nr"), "fn foo() {}".to_string());
        let parsed_files = parse_all(&fm);

        let mut context = Context::new(fm, parsed_files);
        prepare_crate(&mut context, Path::new("/main.nr"));

        CompilerContext { context }
    }

    #[test]
    fn test_works_with_empty_dependency_graph() {
        let source_map = PathToFileSourceMap::default();
        let context = setup_test_context(source_map);

        // one stdlib + one root crate
        assert_eq!(context.crate_graph().number_of_crates(), 2);
    }

    #[test]
    fn test_works_with_root_dependencies() {
        let source_map = PathToFileSourceMap(
            vec![(Path::new("lib1/lib.nr").to_path_buf(), "fn foo() {}".to_string())]
                .into_iter()
                .collect(),
        );

        let mut context = setup_test_context(source_map);
        context.process_dependency_crate("lib1/lib.nr".to_string());

        assert_eq!(context.crate_graph().number_of_crates(), 3);
    }

    #[test]
    fn test_works_with_duplicate_root_dependencies() {
        let source_map = PathToFileSourceMap(
            vec![(Path::new("lib1/lib.nr").to_path_buf(), "fn foo() {}".to_string())]
                .into_iter()
                .collect(),
        );
        let mut context = setup_test_context(source_map);

        let lib1_crate_id = context.process_dependency_crate("lib1/lib.nr".to_string());
        let root_crate_id = context.root_crate_id();

        context.add_dependency_edge("lib1".to_string(), &root_crate_id, &lib1_crate_id).unwrap();
        context.add_dependency_edge("lib1".to_string(), &root_crate_id, &lib1_crate_id).unwrap();

        assert_eq!(context.crate_graph().number_of_crates(), 3);
    }

    #[test]
    fn test_works_with_transitive_dependencies() {
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

        let lib1_crate_id = context.process_dependency_crate("lib1/lib.nr".to_string());
        let lib2_crate_id = context.process_dependency_crate("lib2/lib.nr".to_string());
        let lib3_crate_id = context.process_dependency_crate("lib3/lib.nr".to_string());
        let root_crate_id = context.root_crate_id();

        context.add_dependency_edge("lib1".to_string(), &root_crate_id, &lib1_crate_id).unwrap();
        context.add_dependency_edge("lib2".to_string(), &lib1_crate_id, &lib2_crate_id).unwrap();
        context.add_dependency_edge("lib3".to_string(), &lib2_crate_id, &lib3_crate_id).unwrap();

        assert_eq!(context.crate_graph().number_of_crates(), 5);
    }

    #[test]
    fn test_works_with_missing_dependencies() {
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

        let lib1_crate_id = context.process_dependency_crate("lib1/lib.nr".to_string());
        let lib2_crate_id = context.process_dependency_crate("lib2/lib.nr".to_string());
        let lib3_crate_id = context.process_dependency_crate("lib3/lib.nr".to_string());
        let root_crate_id = context.root_crate_id();

        context.add_dependency_edge("lib1".to_string(), &root_crate_id, &lib1_crate_id).unwrap();
        context.add_dependency_edge("lib3".to_string(), &lib2_crate_id, &lib3_crate_id).unwrap();

        assert_eq!(context.crate_graph().number_of_crates(), 5);
    }
}
