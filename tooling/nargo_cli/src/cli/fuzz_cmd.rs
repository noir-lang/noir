use clap::Args;

use nargo::artifacts::program::ProgramArtifact;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noir_fuzzer::FuzzedExecutor;
use noirc_driver::{CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING};
use noirc_frontend::graph::CrateName;
use proptest::test_runner::TestRunner;

use super::compile_cmd::compile_workspace_full;
use super::NargoConfig;
use crate::cli::fs::program::read_program_from_file;
use crate::errors::CliError;

/// Executes a circuit to calculate its return value
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "e")]
pub(crate) struct FuzzCommand {
    /// The name of the package to execute
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Execute all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

pub(crate) fn run(args: FuzzCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    // Compile the full workspace in order to generate any build artifacts.
    compile_workspace_full(&workspace, &args.compile_options)?;

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let program_artifact_path = workspace.package_build_path(package);
        let program: CompiledProgram = read_program_from_file(program_artifact_path)?.into();

        fuzz_program(program.into(), args.oracle_resolver.as_deref())?;
    }
    Ok(())
}

fn fuzz_program(
    compiled_program: ProgramArtifact,
    _foreign_call_resolver_url: Option<&str>,
) -> Result<(), CliError> {
    let runner = TestRunner::default();
    let fuzzer = FuzzedExecutor::new(compiled_program, runner);

    let result = fuzzer.fuzz();

    println!("{result:?}");

    Ok(())
}
