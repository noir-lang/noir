use std::io::Write;
use std::path::Path;
use std::time::Duration;

use acvm::acir::circuit::ExpressionWidth;
use fm::FileManager;
use nargo::ops::{collect_errors, compile_contract, compile_program, report_errors};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{
    get_package_manifest, resolve_workspace_from_toml, ManifestError, PackageSelection,
};
use noirc_driver::DEFAULT_EXPRESSION_WIDTH;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_driver::{CompilationResult, CompileOptions, CompiledContract};

use clap::Args;
use noirc_frontend::hir::ParsedFiles;
use notify::{EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;

use crate::errors::CliError;

use super::fs::program::{read_program_from_file, save_contract_to_file, save_program_to_file};
use super::{NargoConfig, PackageOptions};
use rayon::prelude::*;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// Watch workspace and recompile on changes.
    #[clap(long, hide = true)]
    watch: bool,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let selection = args.package_options.package_selection();
    let workspace = read_workspace(&config.program_dir, selection)?;

    if args.watch {
        watch_workspace(&workspace, &args.compile_options)
            .map_err(|err| CliError::Generic(err.to_string()))?;
    } else {
        compile_workspace_full(&workspace, &args.compile_options)?;
    }

    Ok(())
}

/// Read a given program directory into a workspace.
fn read_workspace(
    program_dir: &Path,
    selection: PackageSelection,
) -> Result<Workspace, ManifestError> {
    let toml_path = get_package_manifest(program_dir)?;

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;

    Ok(workspace)
}

/// Continuously recompile the workspace on any Noir file change event.
fn watch_workspace(workspace: &Workspace, compile_options: &CompileOptions) -> notify::Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();

    // No specific tickrate, max debounce time 1 seconds
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, tx)?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    debouncer.watcher().watch(&workspace.root_dir, RecursiveMode::Recursive)?;

    let mut screen = std::io::stdout();
    write!(screen, "{}", termion::cursor::Save).unwrap();
    screen.flush().unwrap();
    let _ = compile_workspace_full(workspace, compile_options);
    for res in rx {
        let debounced_events = res.map_err(|mut err| err.remove(0))?;

        // We only want to trigger a rebuild if a noir source file has been modified.
        let noir_files_modified = debounced_events.iter().any(|event| {
            let mut event_paths = event.event.paths.iter();
            let event_affects_noir_file =
                event_paths.any(|path| path.extension().map_or(false, |ext| ext == "nr"));

            let is_relevant_event_kind = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            );

            is_relevant_event_kind && event_affects_noir_file
        });

        if noir_files_modified {
            write!(screen, "{}{}", termion::cursor::Restore, termion::clear::AfterCursor).unwrap();
            screen.flush().unwrap();
            let _ = compile_workspace_full(workspace, compile_options);
        }
    }

    screen.flush().unwrap();

    Ok(())
}

/// Parse all files in the workspace.
fn parse_workspace(workspace: &Workspace) -> (FileManager, ParsedFiles) {
    let mut file_manager = workspace.new_file_manager();
    insert_all_files_for_workspace_into_file_manager(workspace, &mut file_manager);
    let parsed_files = parse_all(&file_manager);
    (file_manager, parsed_files)
}

/// Parse and compile the entire workspace, then report errors.
/// This is the main entry point used by all other commands that need compilation.
pub(super) fn compile_workspace_full(
    workspace: &Workspace,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let (workspace_file_manager, parsed_files) = parse_workspace(workspace);

    let compiled_workspace =
        compile_workspace(&workspace_file_manager, &parsed_files, workspace, compile_options);

    report_errors(
        compiled_workspace,
        &workspace_file_manager,
        compile_options.deny_warnings,
        compile_options.silence_warnings,
    )?;

    Ok(())
}

/// Compile binary and contract packages.
/// Returns the merged warnings or errors.
fn compile_workspace(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    compile_options: &CompileOptions,
) -> CompilationResult<()> {
    let (binary_packages, contract_packages): (Vec<_>, Vec<_>) = workspace
        .into_iter()
        .filter(|package| !package.is_library())
        .cloned()
        .partition(|package| package.is_binary());

    // Compile all of the packages in parallel.
    let program_warnings_or_errors: CompilationResult<()> =
        compile_programs(file_manager, parsed_files, workspace, &binary_packages, compile_options);

    let contract_warnings_or_errors: CompilationResult<()> = compile_contracts(
        file_manager,
        parsed_files,
        &contract_packages,
        compile_options,
        &workspace.target_directory_path(),
    );

    match (program_warnings_or_errors, contract_warnings_or_errors) {
        (Ok((_, program_warnings)), Ok((_, contract_warnings))) => {
            let warnings = [program_warnings, contract_warnings].concat();
            Ok(((), warnings))
        }
        (Err(program_errors), Err(contract_errors)) => {
            Err([program_errors, contract_errors].concat())
        }
        (Err(errors), _) | (_, Err(errors)) => Err(errors),
    }
}

/// Compile the given binary packages in the workspace.
fn compile_programs(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    workspace: &Workspace,
    binary_packages: &[Package],
    compile_options: &CompileOptions,
) -> CompilationResult<()> {
    // Load any existing artifact for a given package, _iff_ it was compiled with the same nargo version.
    // The loaded circuit includes backend specific transformations, which might be different from the current target.
    let load_cached_program = |package| {
        let program_artifact_path = workspace.package_build_path(package);
        read_program_from_file(program_artifact_path)
            .ok()
            .filter(|p| p.noir_version == NOIR_ARTIFACT_VERSION_STRING)
            .map(|p| p.into())
    };

    let compile_package = |package| {
        let cached_program = load_cached_program(package);

        // Hash over the entire compiled program, including any post-compile transformations.
        // This is used to detect whether `cached_program` is returned by `compile_program`.
        let cached_hash = cached_program.as_ref().map(fxhash::hash64);

        // Compile the program, or use the cached artifacts if it matches.
        let (program, warnings) = compile_program(
            file_manager,
            parsed_files,
            workspace,
            package,
            compile_options,
            cached_program,
        )?;

        if compile_options.check_non_determinism {
            let (program_two, _) = compile_program(
                file_manager,
                parsed_files,
                workspace,
                package,
                compile_options,
                load_cached_program(package),
            )?;
            if fxhash::hash64(&program) != fxhash::hash64(&program_two) {
                panic!("Non deterministic result compiling {}", package.name);
            }
        }

        // Choose the target width for the final, backend specific transformation.
        let target_width =
            get_target_width(package.expression_width, compile_options.expression_width);

        // If the compiled program is the same as the cached one, we don't apply transformations again, unless the target width has changed.
        // The transformations might not be idempotent, which would risk creating witnesses that don't work with earlier versions,
        // based on which we might have generated a verifier already.
        if cached_hash == Some(fxhash::hash64(&program)) {
            let width_matches = program
                .program
                .functions
                .iter()
                .all(|circuit| circuit.expression_width == target_width);

            if width_matches {
                return Ok(((), warnings));
            }
        }
        // Run ACVM optimizations and set the target width.
        let program = nargo::ops::transform_program(program, target_width);
        // Check solvability.
        nargo::ops::check_program(&program)?;
        // Overwrite the build artifacts with the final circuit, which includes the backend specific transformations.
        save_program_to_file(&program.into(), &package.name, workspace.target_directory_path());

        Ok(((), warnings))
    };

    // Configure a thread pool with a larger stack size to prevent overflowing stack in large programs.
    // Default is 2MB.
    let pool = rayon::ThreadPoolBuilder::new().stack_size(4 * 1024 * 1024).build().unwrap();
    let program_results: Vec<CompilationResult<()>> =
        pool.install(|| binary_packages.par_iter().map(compile_package).collect());

    // Collate any warnings/errors which were encountered during compilation.
    collect_errors(program_results).map(|(_, warnings)| ((), warnings))
}

/// Compile the given contracts in the workspace.
fn compile_contracts(
    file_manager: &FileManager,
    parsed_files: &ParsedFiles,
    contract_packages: &[Package],
    compile_options: &CompileOptions,
    target_dir: &Path,
) -> CompilationResult<()> {
    let contract_results: Vec<CompilationResult<()>> = contract_packages
        .par_iter()
        .map(|package| {
            let (contract, warnings) =
                compile_contract(file_manager, parsed_files, package, compile_options)?;
            let target_width =
                get_target_width(package.expression_width, compile_options.expression_width);
            let contract = nargo::ops::transform_contract(contract, target_width);
            save_contract(contract, package, target_dir, compile_options.show_artifact_paths);
            Ok(((), warnings))
        })
        .collect();

    // Collate any warnings/errors which were encountered during compilation.
    collect_errors(contract_results).map(|(_, warnings)| ((), warnings))
}

fn save_contract(
    contract: CompiledContract,
    package: &Package,
    target_dir: &Path,
    show_artifact_paths: bool,
) {
    let contract_name = contract.name.clone();
    let artifact_path = save_contract_to_file(
        &contract.into(),
        &format!("{}-{}", package.name, contract_name),
        target_dir,
    );
    if show_artifact_paths {
        println!("Saved contract artifact to: {}", artifact_path.display());
    }
}

/// If a target width was not specified in the CLI we can safely override the default.
pub(crate) fn get_target_width(
    package_default_width: Option<ExpressionWidth>,
    compile_options_width: Option<ExpressionWidth>,
) -> ExpressionWidth {
    if let (Some(manifest_default_width), None) = (package_default_width, compile_options_width) {
        manifest_default_width
    } else {
        compile_options_width.unwrap_or(DEFAULT_EXPRESSION_WIDTH)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        str::FromStr,
    };

    use clap::Parser;
    use nargo::ops::compile_program;
    use nargo_toml::PackageSelection;
    use noirc_driver::{CompileOptions, CrateName};
    use rayon::prelude::*;

    use crate::cli::compile_cmd::{get_target_width, parse_workspace, read_workspace};

    /// Try to find the directory that Cargo sets when it is running;
    /// otherwise fallback to assuming the CWD is the root of the repository
    /// and append the crate path.
    fn test_programs_dir() -> PathBuf {
        let root_dir = match std::env::var("CARGO_MANIFEST_DIR") {
            Ok(dir) => PathBuf::from(dir).parent().unwrap().parent().unwrap().to_path_buf(),
            Err(_) => std::env::current_dir().unwrap(),
        };
        root_dir.join("test_programs")
    }

    /// Collect the test programs under a sub-directory.
    fn read_test_program_dirs(
        test_programs_dir: &Path,
        test_sub_dir: &str,
    ) -> impl Iterator<Item = PathBuf> {
        let test_case_dir = test_programs_dir.join(test_sub_dir);
        std::fs::read_dir(test_case_dir)
            .unwrap()
            .flatten()
            .filter(|c| c.path().is_dir())
            .map(|c| c.path())
    }

    #[derive(Parser, Debug)]
    #[command(ignore_errors = true)]
    struct Options {
        /// Test name to filter for.
        ///
        /// For example:
        /// ```text
        /// cargo test -p nargo_cli -- test_transform_program_is_idempotent slice_loop
        /// ```
        args: Vec<String>,
    }

    impl Options {
        fn package_selection(&self) -> PackageSelection {
            match self.args.as_slice() {
                [_test_name, test_program] => {
                    PackageSelection::Selected(CrateName::from_str(test_program).unwrap())
                }
                _ => PackageSelection::DefaultOrAll,
            }
        }
    }

    /// Check that `nargo::ops::transform_program` is idempotent by compiling the
    /// test programs and running them through the optimizer twice.
    ///
    /// This test is here purely because of the convenience of having access to
    /// the utility functions to process workspaces.
    #[test]
    fn test_transform_program_is_idempotent() {
        let opts = Options::parse();

        let sel = opts.package_selection();
        let verbose = matches!(sel, PackageSelection::Selected(_));

        let test_workspaces = read_test_program_dirs(&test_programs_dir(), "execution_success")
            .filter_map(|dir| read_workspace(&dir, sel.clone()).ok())
            .collect::<Vec<_>>();

        assert!(!test_workspaces.is_empty(), "should find some test workspaces");

        test_workspaces.par_iter().for_each(|workspace| {
            let (file_manager, parsed_files) = parse_workspace(workspace);
            let binary_packages = workspace.into_iter().filter(|package| package.is_binary());

            for package in binary_packages {
                let (program_0, _warnings) = compile_program(
                    &file_manager,
                    &parsed_files,
                    workspace,
                    package,
                    &CompileOptions::default(),
                    None,
                )
                .expect("failed to compile");

                let width = get_target_width(package.expression_width, None);

                let program_1 = nargo::ops::transform_program(program_0, width);
                let program_2 = nargo::ops::transform_program(program_1.clone(), width);

                if verbose {
                    // Compare where the most likely difference is.
                    similar_asserts::assert_eq!(
                        format!("{}", program_1.program),
                        format!("{}", program_2.program),
                        "optimization not idempotent for test program '{}'",
                        package.name
                    );
                    assert_eq!(
                        program_1.program, program_2.program,
                        "optimization not idempotent for test program '{}'",
                        package.name
                    );

                    // Compare the whole content.
                    similar_asserts::assert_eq!(
                        serde_json::to_string_pretty(&program_1).unwrap(),
                        serde_json::to_string_pretty(&program_2).unwrap(),
                        "optimization not idempotent for test program '{}'",
                        package.name
                    );
                } else {
                    // Just compare hashes, which would just state that the program failed.
                    // Then we can use the filter option to zoom in one one to see why.
                    assert!(
                        fxhash::hash64(&program_1) == fxhash::hash64(&program_2),
                        "optimization not idempotent for test program '{}'",
                        package.name
                    );
                }
            }
        });
    }
}
