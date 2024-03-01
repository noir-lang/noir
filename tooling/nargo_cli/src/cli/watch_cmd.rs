use std::io::Write;
use std::time::Duration;

use nargo::ops::compile_workspace;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_driver::file_manager_with_stdlib;
use noirc_driver::CompileOptions;
use noirc_driver::NOIR_ARTIFACT_VERSION_STRING;

use noirc_frontend::graph::CrateName;

use clap::Args;
use notify::{EventKind, RecursiveMode, Watcher};

use crate::errors::CliError;

use super::NargoConfig;

use notify_debouncer_full::new_debouncer;

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct WatchCommand {
    /// The name of the package to compile
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Compile all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: WatchCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);

    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_owned()),
    )?;

    watch_workspace(&workspace, &args.compile_options)
        .map_err(|err| CliError::Generic(err.to_string()))?;

    Ok(())
}

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
    compile_workspace_for_diagnostics(workspace, compile_options);
    for res in rx {
        let debounced_events = res.map_err(|mut err| err.remove(0))?;

        if debounced_events.iter().all(|event| {
            !matches!(event.kind, EventKind::Modify(_))
                || event
                    .event
                    .paths
                    .iter()
                    .all(|path| path.extension().map_or(false, |ext| ext != "nr"))
        }) {
            continue;
        }
        write!(screen, "{}{}", termion::cursor::Restore, termion::clear::AfterCursor).unwrap();
        screen.flush().unwrap();
        compile_workspace_for_diagnostics(workspace, compile_options);
    }

    screen.flush().unwrap();

    Ok(())
}

fn compile_workspace_for_diagnostics(workspace: &Workspace, compile_options: &CompileOptions) {
    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let compiled_workspace =
        compile_workspace(&workspace_file_manager, &parsed_files, workspace, compile_options);

    let warnings_and_errors =
        compiled_workspace.map(|(_, warnings)| warnings).unwrap_or_else(|errors| errors);
    let reported_errors = noirc_errors::reporter::report_all(
        workspace_file_manager.as_file_map(),
        &warnings_and_errors,
        compile_options.deny_warnings,
        compile_options.silence_warnings,
    );

    if reported_errors.error_count == 0 {
        println!("Successfully compiled without errors");
    } else {
        println!("Compiled with {} warnings/errors", reported_errors.error_count);
    }
}
