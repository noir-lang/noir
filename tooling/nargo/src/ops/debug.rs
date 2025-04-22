use std::path::Path;

use acvm::acir::circuit::ExpressionWidth;
use fm::FileManager;
use noirc_driver::{
    CompileOptions, CompiledProgram, CrateId, DEFAULT_EXPRESSION_WIDTH, compile_no_check,
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
    compile_program, compile_program_with_debug_instrumenter, report_errors, transform_program,
};

pub struct TestDefinition {
    pub name: String,
    pub function: TestFunction,
}
pub fn get_test_function_for_debug(
    crate_id: CrateId,
    context: &Context,
    test_name: &str,
) -> Result<TestDefinition, String> {
    let test_pattern = FunctionNameMatch::Contains(vec![test_name.into()]);

    let test_functions = context.get_all_test_functions_in_crate_matching(&crate_id, &test_pattern);

    let (test_name, test_function) = match test_functions {
        matchings if matchings.is_empty() => {
            return Err(format!("`{}` does not match with any test function", test_name));
        }
        matchings if matchings.len() == 1 => matchings.into_iter().next().unwrap(),
        matchings => {
            let exact_match_op = matchings
                .into_iter()
                .filter(|(name, _)| name.split("::").last() == Some(test_name))
                .collect::<Vec<(String, TestFunction)>>();
            // There can be multiple matches but only one that matches exactly
            // this would be the case of tests names that englobe others
            // i.e.:
            //  - test_something
            //  - unconstrained_test_something
            // in this case, looking up "test_something" throws two matchings
            // but only one matches exact
            if exact_match_op.len() == 1 {
                exact_match_op.into_iter().next().unwrap()
            } else {
                return Err(format!("`{}` matches with more than one test function", test_name));
            }
        }
    };

    let test_function_has_arguments = !context
        .def_interner
        .function_meta(&test_function.get_id())
        .function_signature()
        .0
        .is_empty();

    if test_function_has_arguments {
        return Err(String::from("Cannot debug tests with arguments"));
    }
    Ok(TestDefinition { name: test_name, function: test_function })
}

pub fn compile_test_fn_for_debugging(
    test_def: &TestDefinition,
    context: &mut Context,
    package: &Package,
    compile_options: CompileOptions,
) -> Result<CompiledProgram, noirc_driver::CompileError> {
    let compiled_program =
        compile_no_check(context, &compile_options, test_def.function.get_id(), None, false)?;
    let expression_width =
        get_target_width(package.expression_width, compile_options.expression_width);
    let compiled_program = transform_program(compiled_program, expression_width);
    Ok(compiled_program)
}

pub fn compile_bin_package_for_debugging(
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<CompiledProgram, CompileError> {
    let (workspace_file_manager, mut parsed_files) = load_workspace_files(workspace);

    let expression_width =
        get_target_width(package.expression_width, compile_options.expression_width);

    let compilation_result = if compile_options.instrument_debug {
        let debug_state =
            instrument_package_files(&mut parsed_files, &workspace_file_manager, package);

        compile_program_with_debug_instrumenter(
            &workspace_file_manager,
            &parsed_files,
            workspace,
            package,
            compile_options,
            None,
            debug_state,
        )
    } else {
        compile_program(
            &workspace_file_manager,
            &parsed_files,
            workspace,
            package,
            compile_options,
            None,
        )
    };

    report_errors(
        compilation_result,
        &workspace_file_manager,
        compile_options.deny_warnings,
        compile_options.silence_warnings,
    )
    .map(|compiled_program| transform_program(compiled_program, expression_width))
}

pub fn compile_options_for_debugging(
    acir_mode: bool,
    skip_instrumentation: bool,
    expression_width: Option<ExpressionWidth>,
    compile_options: CompileOptions,
) -> CompileOptions {
    CompileOptions {
        // Compilation warnings are disabled when
        // compiling for debugging
        //
        // For instrumenting the program the debugger
        // will import functions that may not be used,
        // which would generate compilation warnings
        silence_warnings: true,
        deny_warnings: false,
        instrument_debug: !skip_instrumentation,
        force_brillig: !acir_mode,
        expression_width,
        ..compile_options
    }
}

pub fn prepare_package_for_debug<'a>(
    file_manager: &'a FileManager,
    parsed_files: &'a mut ParsedFiles,
    package: &'a Package,
    workspace: &Workspace,
) -> (Context<'a, 'a>, CrateId) {
    let debug_instrumenter = instrument_package_files(parsed_files, file_manager, package);

    // -- This :down: is from nargo::ops(compile).compile_program_with_debug_instrumenter
    let (mut context, crate_id) = prepare_package(file_manager, parsed_files, package);
    link_to_debug_crate(&mut context, crate_id);
    context.debug_instrumenter = debug_instrumenter;
    context.package_build_path = workspace.package_build_path(package);
    (context, crate_id)
}

pub fn load_workspace_files(workspace: &Workspace) -> (FileManager, ParsedFiles) {
    let mut file_manager = file_manager_with_stdlib(Path::new(""));
    insert_all_files_for_workspace_into_file_manager(workspace, &mut file_manager);

    let parsed_files = parse_all(&file_manager);
    (file_manager, parsed_files)
}

/// Add debugging instrumentation to all parsed files belonging to the package
/// being compiled
fn instrument_package_files(
    parsed_files: &mut ParsedFiles,
    file_manager: &FileManager,
    package: &Package,
) -> DebugInstrumenter {
    // Start off at the entry path and read all files in the parent directory.
    let entry_path_parent = package
        .entry_path
        .parent()
        .unwrap_or_else(|| panic!("The entry path is expected to be a single file within a directory and so should have a parent {:?}", package.entry_path));

    let mut debug_instrumenter = DebugInstrumenter::default();

    for (file_id, parsed_file) in parsed_files.iter_mut() {
        let file_path =
            file_manager.path(*file_id).expect("Parsed file ID not found in file manager");
        for ancestor in file_path.ancestors() {
            if ancestor == entry_path_parent {
                // file is in package
                debug_instrumenter.instrument_module(&mut parsed_file.0, *file_id);
            }
        }
    }

    debug_instrumenter
}

// This is the same as in compile_cmd, perhaps we should move it to ops::compile?
fn get_target_width(
    package_default_width: Option<ExpressionWidth>,
    compile_options_width: Option<ExpressionWidth>,
) -> ExpressionWidth {
    if let (Some(manifest_default_width), None) = (package_default_width, compile_options_width) {
        manifest_default_width
    } else {
        compile_options_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH)
    }
}
