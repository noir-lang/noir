use std::path::Path;

use fm::FileManager;
use noirc_driver::{
    CompileOptions, CompiledProgram, CrateId,
    file_manager_with_stdlib, link_to_debug_crate,
};
use noirc_frontend::{
    debug::DebugInstrumenter,
    hir::{Context, FunctionNameMatch, ParsedFiles, def_map::TestFunction},
};

use crate::{
    errors::CompileError, insert_all_files_for_workspace_into_file_manager, package::Package,
    parse_all, prepare_package, workspace::Workspace,
};

use super::{
    compile_program, compile_program_with_debug_instrumenter,
};

pub struct TestDefinition {
    pub name: String,
    pub function: TestFunction,
}

/// Stub - debugging requires ACVM backend
pub fn get_test_function_for_debug(
    _crate_id: CrateId,
    _context: &Context,
    _test_name: &str,
) -> Result<TestDefinition, String> {
    Err("Debug execution is not available in Sensei (requires ZK backend)".to_string())
}

/// Stub - debugging requires ACVM backend
pub fn compile_options_for_debugging(
    _acir_mode: bool,
    _skip_instrumentation: bool,
    _expression_width: Option<String>,
    compile_options: CompileOptions,
) -> CompileOptions {
    // Return compile options unchanged since we can't do backend-specific debugging
    compile_options
}

/// Stub - debugging requires ACVM backend
pub fn compile_bin_package_for_debugging(
    _workspace: &Workspace,
    _package: &Package,
    _compile_options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    Err(CompileError::Generic("Debug compilation is not available in Sensei (requires ZK backend)".to_string()))
}

/// Stub - debugging requires ACVM backend
pub fn compile_test_fn_for_debugging(
    _test: &TestDefinition,
    _context: &mut Context,
    _package: &Package,
    _compile_options: CompileOptions,
) -> Result<CompiledProgram, Vec<CompileError>> {
    Err(vec![CompileError::Generic("Debug test compilation is not available in Sensei (requires ZK backend)".to_string())])
}

/// Stub - debugging requires ACVM backend
pub fn prepare_package_for_debug<'a, 'b>(
    file_manager: &'a FileManager,
    _parsed_files: &'b mut ParsedFiles,
    _package: &Package,
    _workspace: &Workspace,
) -> (Context<'a, 'b>, CrateId) {
    // Return new context and dummy crate ID
    (Context::new(file_manager.clone(), Default::default()), CrateId::dummy_id())
}

/// Stub - debugging requires ACVM backend
pub fn load_workspace_files(
    _workspace: &Workspace,
) -> (FileManager, ParsedFiles) {
    // Return empty file manager and parsed files
    let fm = FileManager::new(&Path::new("/"));
    let parsed = ParsedFiles::new();
    (fm, parsed)
}