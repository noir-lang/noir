use nargo::ops::{check_crate_and_report_errors, report_errors};
use noir_artifact_cli::fs::artifact::save_program_to_file;
use noirc_artifacts::program::CompiledProgram;
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::ParsedFiles;
use rayon::prelude::*;
use std::any::Any;
use std::panic::{self, AssertUnwindSafe};
use std::path::Path;
use std::sync::Arc;

use fm::FileManager;
use nargo::package::Package;
use nargo::prepare_package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::PackageSelection;
use noirc_driver::{CompileOptions, compile_no_check};

use clap::Args;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};

#[allow(rustdoc::broken_intra_doc_links)]
/// Exports functions marked with #[export] attribute
#[derive(Debug, Clone, Args)]
pub(crate) struct ExportCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

impl WorkspaceCommand for ExportCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        // Writes the exported functions.
        LockType::Exclusive
    }
}

pub(crate) fn run(args: ExportCommand, workspace: Workspace) -> Result<(), CliError> {
    let mut workspace_file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = Arc::new(parse_all(&workspace_file_manager));
    let workspace_file_manager = Arc::new(workspace_file_manager);

    let library_packages: Vec<_> =
        workspace.into_iter().filter(|package| package.is_library()).collect();

    let compile_package = |package: &&Package| {
        compile_exported_functions(
            &workspace_file_manager,
            &parsed_files,
            &workspace,
            package,
            &args.compile_options,
        )
    };

    // Packages, and each package's exported functions within them, compile in parallel on this
    // one pool, unless an option prints the compilation steps, which would then interleave. Its
    // threads get the stack `nargo compile` gives its threads, since deep functions overflow
    // rayon's default.
    let pool = rayon::ThreadPoolBuilder::new()
        .stack_size(nargo::compilation_stack_size())
        .build()
        .expect("failed to build the export thread pool");
    pool.install(|| {
        if args.compile_options.prints_compilation_steps() {
            library_packages.iter().try_for_each(compile_package)
        } else {
            library_packages.par_iter().map(compile_package).collect()
        }
    })
}

/// What compiling one exported function produced: a program, an ordinary error, or a panic.
type CompiledFunction = std::thread::Result<Result<CompiledProgram, CustomDiagnostic>>;

/// The exported functions that compiled, the errors of those that didn't, and the payload of the
/// first function that panicked, all in the order of `compiled`.
struct ExportResults {
    programs: Vec<(String, CompiledProgram)>,
    errors: Vec<CustomDiagnostic>,
    first_panic: Option<Box<dyn Any + Send>>,
}

/// Reports `errors`, if any, with `report`, and then raises `first_panic` again if there was one.
/// The errors are reported first, so a panic in one function doesn't hide another's errors.
fn report_then_resume_panic<E>(
    errors: Vec<CustomDiagnostic>,
    first_panic: Option<Box<dyn Any + Send>>,
    report: impl FnOnce(Vec<CustomDiagnostic>) -> Result<(), E>,
) -> Result<(), E> {
    let reported = if errors.is_empty() { Ok(()) } else { report(errors) };
    if let Some(payload) = first_panic {
        panic::resume_unwind(payload);
    }
    reported
}

fn split_results(compiled: Vec<(String, CompiledFunction)>) -> ExportResults {
    let mut results = ExportResults { programs: Vec::new(), errors: Vec::new(), first_panic: None };
    for (function_name, program) in compiled {
        match program {
            Ok(Ok(program)) => results.programs.push((function_name, program)),
            Ok(Err(error)) => results.errors.push(error),
            Err(payload) => {
                results.first_panic.get_or_insert(payload);
            }
        }
    }
    results
}

fn compile_exported_functions(
    file_manager: &Arc<FileManager>,
    parsed_files: &Arc<ParsedFiles>,
    workspace: &Workspace,
    package: &Package,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let (context, crate_id) = prepare_package(file_manager, parsed_files, package);
    let context = check_crate_and_report_errors(context, crate_id, compile_options)?;

    let exported_functions = context.get_all_exported_functions_in_crate(&crate_id);

    // Each exported function is compiled on its own, reading `context` without writing to it, so
    // they are compiled in parallel, unless an option prints the compilation steps, which would
    // then interleave. A panic is caught so that the other functions' errors are still reported
    // before it is raised again.
    let compile = |(function_name, function_id): (String, _)| -> (String, CompiledFunction) {
        let program = panic::catch_unwind(AssertUnwindSafe(|| {
            compile_no_check(
                &context,
                compile_options,
                Path::new(""),
                false,
                function_id,
                None,
                false,
            )
            .map_err(CustomDiagnostic::from)
        }));
        (function_name, program)
    };
    let compiled: Vec<_> = if compile_options.prints_compilation_steps() {
        exported_functions.into_iter().map(compile).collect()
    } else {
        exported_functions.into_par_iter().map(compile).collect()
    };

    // Every function was compiled, so every failure is reported, in the order
    // `get_all_exported_functions_in_crate` returned the functions.
    let ExportResults { programs, errors, first_panic } = split_results(compiled);
    report_then_resume_panic(errors, first_panic, |errors| {
        report_errors::<()>(
            Err(errors),
            &context.file_manager,
            &context.parsed_files,
            compile_options.deny_warnings,
            compile_options.silence_warnings,
        )
    })?;

    let export_dir = workspace.export_directory_path();
    for (function_name, program) in programs {
        save_program_to_file(&program.into(), &function_name.parse().unwrap(), &export_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use noirc_errors::CustomDiagnostic;

    use super::{CompiledFunction, ExportResults, report_then_resume_panic, split_results};

    #[test]
    fn a_panicking_function_keeps_the_other_functions_errors() {
        let error = CustomDiagnostic::from_message("b fails", fm::FileId::dummy());
        let compiled: Vec<(String, CompiledFunction)> = vec![
            ("a".to_string(), Err(Box::new("a panics"))),
            ("b".to_string(), Ok(Err(error))),
            ("c".to_string(), Err(Box::new("c panics"))),
        ];

        let ExportResults { programs, errors, first_panic } = split_results(compiled);

        assert!(programs.is_empty());
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "b fails");
        let payload = first_panic.expect("the panic should be kept");
        assert_eq!(payload.downcast_ref::<&str>(), Some(&"a panics"), "the first panic is kept");
    }

    #[test]
    fn errors_are_reported_before_a_panic_is_raised_again() {
        let error = CustomDiagnostic::from_message("b fails", fm::FileId::dummy());
        let mut reported = Vec::new();

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            report_then_resume_panic::<()>(vec![error], Some(Box::new("a panics")), |errors| {
                reported.extend(errors.into_iter().map(|error| error.message));
                Err(())
            })
        }))
        .expect_err("the panic should be raised again");

        assert_eq!(reported, vec!["b fails".to_string()]);
        assert_eq!(panic.downcast_ref::<&str>(), Some(&"a panics"));
    }
}
