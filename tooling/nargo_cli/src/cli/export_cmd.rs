use itertools::Itertools;
use nargo::ops::{check_crate_and_report_errors, report_errors};
use noir_artifact_cli::fs::artifact::save_program_to_file;
use noirc_errors::CustomDiagnostic;
use noirc_frontend::hir::ParsedFiles;
use rayon::prelude::*;
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
    // then interleave.
    let compile = |(function_name, function_id): (String, _)| {
        compile_no_check(&context, compile_options, Path::new(""), false, function_id, None, false)
            .map(|program| (function_name, program))
            .map_err(CustomDiagnostic::from)
    };
    let compiled: Vec<_> = if compile_options.prints_compilation_steps() {
        exported_functions.into_iter().map(compile).collect()
    } else {
        exported_functions.into_par_iter().map(compile).collect()
    };

    // Every function was compiled, so every failure is reported, in the order
    // `get_all_exported_functions_in_crate` returned the functions.
    let (programs, errors): (Vec<_>, Vec<_>) = compiled.into_iter().partition_result();
    if !errors.is_empty() {
        report_errors::<()>(
            Err(errors),
            &context.file_manager,
            &context.parsed_files,
            compile_options.deny_warnings,
            compile_options.silence_warnings,
        )?;
    }

    let export_dir = workspace.export_directory_path();
    for (function_name, program) in programs {
        save_program_to_file(&program.into(), &function_name.parse().unwrap(), &export_dir)?;
    }
    Ok(())
}
