use crate::compile::{
    file_manager_with_source_map, preprocess_contract, preprocess_program, JsCompileResult,
    PathToFileSourceMap,
};
use crate::errors::{CompileError, JsCompileError};
use noirc_driver::{
    add_dep, compile_contract, compile_main, prepare_crate, prepare_dependency, CompileOptions,
};
use noirc_frontend::{
    graph::{CrateGraph, CrateId, CrateName},
    hir::Context,
};
use std::path::Path;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = "Context")]
pub struct ContextWrapper {
    context: Context,
}

#[wasm_bindgen(js_name = "CrateId")]
#[derive(Debug, Copy, Clone)]
pub struct CrateIDWrapper(CrateId);

#[wasm_bindgen]
impl ContextWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(source_map: PathToFileSourceMap) -> ContextWrapper {
        console_error_panic_hook::set_once();

        let fm = file_manager_with_source_map(source_map);
        let graph = CrateGraph::default();
        ContextWrapper { context: Context::new(fm, graph) }
    }

    #[cfg(test)]
    pub(crate) fn crate_graph(&self) -> &CrateGraph {
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
    pub fn add_dependency_edge(
        &mut self,
        crate_name: String,
        from: CrateIDWrapper,
        to: CrateIDWrapper,
    ) {
        let parsed_crate_name: CrateName = crate_name
            .parse()
            .unwrap_or_else(|_| panic!("Failed to parse crate name {}", crate_name));
        add_dep(&mut self.context, from.0, to.0, parsed_crate_name);
    }

    pub fn compile_program(mut self) -> Result<JsCompileResult, JsCompileError> {
        let compile_options = CompileOptions::default();
        // For now we default to plonk width = 3, though we can add it as a parameter
        let np_language = acvm::Language::PLONKCSat { width: 3 };
        #[allow(deprecated)]
        let is_opcode_supported = acvm::pwg::default_is_opcode_supported(np_language);

        let root_crate_id = *self.context.root_crate_id();

        let compiled_program =
            compile_main(&mut self.context, root_crate_id, &compile_options, None, true)
                .map_err(|errs| {
                    CompileError::with_file_diagnostics(
                        "Failed to compile program",
                        errs,
                        &self.context.file_manager,
                    )
                })?
                .0;

        let optimized_program =
            nargo::ops::optimize_program(compiled_program, np_language, &is_opcode_supported)
                .expect("Program optimization failed");

        let compile_output = preprocess_program(optimized_program);
        Ok(JsCompileResult::new(compile_output))
    }

    pub fn compile_contract(mut self) -> Result<JsCompileResult, JsCompileError> {
        let compile_options = CompileOptions::default();
        // For now we default to plonk width = 3, though we can add it as a parameter
        let np_language = acvm::Language::PLONKCSat { width: 3 };
        #[allow(deprecated)]
        let is_opcode_supported = acvm::pwg::default_is_opcode_supported(np_language);

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
            nargo::ops::optimize_contract(compiled_contract, np_language, &is_opcode_supported)
                .expect("Contract optimization failed");

        let compile_output = preprocess_contract(optimized_contract);
        Ok(JsCompileResult::new(compile_output))
    }
}

#[cfg(test)]
mod test {
    use noirc_driver::prepare_crate;
    use noirc_frontend::{graph::CrateGraph, hir::Context};

    use crate::compile::{file_manager_with_source_map, PathToFileSourceMap};

    use std::path::Path;

    use super::ContextWrapper;

    fn setup_test_context(source_map: PathToFileSourceMap) -> ContextWrapper {
        let mut fm = file_manager_with_source_map(source_map);
        // Add this due to us calling prepare_crate on "/main.nr" below
        fm.add_file_with_source(Path::new("/main.nr"), "fn foo() {}".to_string());

        let graph = CrateGraph::default();
        let mut context = Context::new(fm, graph);
        prepare_crate(&mut context, Path::new("/main.nr"));

        ContextWrapper { context }
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

        context.add_dependency_edge("lib1".to_string(), root_crate_id, lib1_crate_id);
        context.add_dependency_edge("lib1".to_string(), root_crate_id, lib1_crate_id);

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

        context.add_dependency_edge("lib1".to_string(), root_crate_id, lib1_crate_id);
        context.add_dependency_edge("lib2".to_string(), lib1_crate_id, lib2_crate_id);
        context.add_dependency_edge("lib3".to_string(), lib2_crate_id, lib3_crate_id);

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

        context.add_dependency_edge("lib1".to_string(), root_crate_id, lib1_crate_id);
        context.add_dependency_edge("lib3".to_string(), lib2_crate_id, lib3_crate_id);

        assert_eq!(context.crate_graph().number_of_crates(), 5);
    }
}
