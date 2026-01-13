use std::hash::BuildHasher;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use fm::FileManager;
use nargo::ops::{collect_errors, compile_contract, compile_program, report_errors};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::PackageSelection;
use noir_artifact_cli::fs::artifact::{
    read_program_from_file, save_contract_to_file, save_program_to_file,
};
use noirc_artifacts::contract::CompiledContract;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;
use noirc_driver::{CompilationResult, CompileOptions};

use clap::Args;
use noirc_frontend::hir::ParsedFiles;
use notify::{EventKind, RecursiveMode, Watcher};
use notify_debouncer_full::new_debouncer;

use crate::errors::CliError;

use super::{LockType, PackageOptions, WorkspaceCommand};
use rayon::prelude::*;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub struct CompileCommand {
    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    pub(super) compile_options: CompileOptions,

    /// Watch workspace and recompile on changes.
    #[clap(long, hide = true)]
    watch: bool,
}

impl WorkspaceCommand for CompileCommand {
    fn package_selection(&self) -> PackageSelection {
        self.package_options.package_selection()
    }

    fn lock_type(&self) -> LockType {
        LockType::Exclusive
    }
}

pub(crate) fn run(args: CompileCommand, workspace: Workspace) -> Result<(), CliError> {
    if args.watch {
        if args.compile_options.debug_compile_stdin {
            return Err(CliError::CantWatchStdin);
        }
        watch_workspace(&workspace, &args.compile_options)
            .map_err(|err| CliError::Generic(err.to_string()))?;
    } else {
        let debug_compile_stdin = None;
        compile_workspace_full(&workspace, &args.compile_options, debug_compile_stdin)?;
    }
    Ok(())
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
    let debug_compile_stdin = None;
    let _ = compile_workspace_full(workspace, compile_options, debug_compile_stdin);
    for res in rx {
        let debounced_events = res.map_err(|mut err| err.remove(0))?;

        // We only want to trigger a rebuild if a noir source file has been modified.
        let noir_files_modified = debounced_events.iter().any(|event| {
            let mut event_paths = event.event.paths.iter();
            let event_affects_noir_file =
                event_paths.any(|path| path.extension().is_some_and(|ext| ext == "nr"));

            let is_relevant_event_kind = matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            );

            is_relevant_event_kind && event_affects_noir_file
        });

        if noir_files_modified {
            write!(screen, "{}{}", termion::cursor::Restore, termion::clear::AfterCursor).unwrap();
            screen.flush().unwrap();
            let debug_compile_stdin = None;
            let _ = compile_workspace_full(workspace, compile_options, debug_compile_stdin);
        }
    }

    screen.flush().unwrap();

    Ok(())
}

/// Parse all files in the workspace.
pub fn parse_workspace(
    workspace: &Workspace,
    debug_compile_stdin: Option<String>,
) -> (FileManager, ParsedFiles) {
    let mut file_manager = workspace.new_file_manager();

    if let Some(main_nr) = debug_compile_stdin {
        file_manager.add_file_with_source(Path::new("src/main.nr"), main_nr);
    } else {
        insert_all_files_for_workspace_into_file_manager(workspace, &mut file_manager);
    }

    let parsed_files = parse_all(&file_manager);
    (file_manager, parsed_files)
}

/// Parse and compile the entire workspace, then report errors.
/// This is the main entry point used by all other commands that need compilation.
pub fn compile_workspace_full(
    workspace: &Workspace,
    compile_options: &CompileOptions,
    debug_compile_stdin: Option<String>, // use this String as STDIN if present
) -> Result<(), CliError> {
    let mut debug_compile_stdin = debug_compile_stdin;
    if compile_options.debug_compile_stdin && debug_compile_stdin.is_none() {
        let mut main_nr = String::new();
        let stdin = std::io::stdin();
        let mut stdin_handle = stdin.lock();
        stdin_handle.read_to_string(&mut main_nr).expect("reading from stdin to succeed");
        debug_compile_stdin = Some(main_nr);
    }
    let (workspace_file_manager, parsed_files) = parse_workspace(workspace, debug_compile_stdin);

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
        read_program_from_file(&program_artifact_path)
            .ok()
            .filter(|p| p.noir_version == NOIR_ARTIFACT_VERSION_STRING)
            .map(|p| p.into())
    };

    let compile_package = |package| {
        let cached_program = load_cached_program(package);

        // Hash over the entire compiled program, including any post-compile transformations.
        // This is used to detect whether `cached_program` is returned by `compile_program`.
        let cached_hash =
            cached_program.as_ref().map(|prog| rustc_hash::FxBuildHasher.hash_one(prog));

        // Compile the program, or use the cached artifacts if it matches.
        let (program, warnings) = compile_program(
            file_manager,
            parsed_files,
            workspace,
            package,
            compile_options,
            cached_program,
        )?;

        // If the compiled program is the same as the cached one, we don't apply transformations again, unless the target width has changed.
        // The transformations might not be idempotent, which would risk creating witnesses that don't work with earlier versions,
        // based on which we might have generated a verifier already.
        if cached_hash == Some(rustc_hash::FxBuildHasher.hash_one(&program)) {
            return Ok(((), warnings));
        }
        // Run ACVM optimizations.
        let program = nargo::ops::optimize_program(program);
        // Check solvability.
        nargo::ops::check_program(&program)?;
        // Overwrite the build artifacts with the final circuit, which includes the backend specific transformations.
        save_program_to_file(&program.into(), &package.name, &workspace.target_directory_path())
            .expect("failed to save program");

        Ok(((), warnings))
    };

    // Configure a thread pool with a larger stack size to prevent overflowing stack in large programs.
    // Default is 2MB. Limit threads to the number of packages we actually need to compile.
    let num_threads = rayon::current_num_threads().min(binary_packages.len()).max(1);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .stack_size(4 * 1024 * 1024)
        .build()
        .unwrap();
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

            let contract = nargo::ops::optimize_contract(contract);
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
    )
    .expect("failed to save contract");
    if show_artifact_paths {
        println!("Saved contract artifact to: {}", artifact_path.display());
    }
}
