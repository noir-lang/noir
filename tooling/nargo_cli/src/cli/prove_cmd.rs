use fm::FileManager;
use clap::Args;
use nargo::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};
use nargo::ops::{compile_program, report_errors};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::{Format, InputValue};
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};
use noirc_frontend::graph::CrateName;

use super::fs::{
    inputs::{read_inputs_from_file, write_inputs_to_file},
    proof::save_proof_to_dir,
};
use super::NargoConfig;
use crate::{backends::Backend, cli::execute_cmd::execute_program, errors::{CliError, FilesystemError}};

use std::collections::BTreeMap;

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "p")]
pub(crate) struct ProveCommand {
    /// The name of the toml file which contains the inputs for the prover
    #[clap(long, short, default_value = PROVER_INPUT_FILE)]
    prover_name: String,

    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = VERIFIER_INPUT_FILE)]
    verifier_name: String,

    /// Verify proof after proving
    #[arg(long)]
    verify: bool,

    /// The name of the package to prove
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Prove all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

pub(crate) fn run(
    backend: &Backend,
    args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);

    let workspace_with_inputs = workspace.into_iter().filter_map(|package| {
        if !package.is_binary() {
            return None;
        }
        Some(read_inputs_from_file(&package.root_dir, args.prover_name, Format::Toml, &compiled_program.abi).map(|(inputs_map, _)| (package, inputs_map)))
    });
    for proof_and_package_name_or_err in run_pure(backend, args, workspace, workspace_with_inputs, workspace_file_manager) {
        let (proof, package_name) = proof_and_package_name_or_err?;
        save_proof_to_dir(&proof, &String::from(&package_name), workspace.proofs_directory_path())?;
    }

    Ok(())
}

pub fn run_pure<'a, I>(
    backend: &'a Backend,
    args: ProveCommand,
    workspace: Workspace,
    workspace_iterator: I,
    workspace_file_manager: FileManager,
) -> impl Iterator<Item = Result<(Vec<u8>, String), CliError>> + 'a
where
    I: Iterator<Item = Result<(&'a Package, BTreeMap<String, InputValue>), FilesystemError>> + 'a,
{

    let parsed_files = parse_all(&workspace_file_manager);

    let expression_width = args
        .compile_options
        .expression_width
        .unwrap_or_else(|| backend.get_backend_info_or_default());
    workspace_iterator
        .map(move |package_and_inputs| {
            let (package, inputs_map) = package_and_inputs?;
            let compilation_result = compile_program(
                &workspace_file_manager,
                &parsed_files,
                package,
                &args.compile_options,
                None,
            );

            let compiled_program = report_errors(
                compilation_result,
                &workspace_file_manager,
                args.compile_options.deny_warnings,
                args.compile_options.silence_warnings,
            )?;

            let compiled_program = nargo::ops::transform_program(compiled_program, expression_width);

            prove_package_pure(
                backend,
                &workspace,
                package,
                compiled_program,
                &args.prover_name,
                &args.verifier_name,
                args.verify,
                args.oracle_resolver.as_deref(),
                inputs_map,
            )

        })

}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prove_package(
    backend: &Backend,
    workspace: &Workspace,
    package: &Package,
    compiled_program: CompiledProgram,
    prover_name: &str,
    verifier_name: &str,
    check_proof: bool,
    foreign_call_resolver_url: Option<&str>,
) -> Result<(), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &compiled_program.abi)?;

    let (proof, package_name) = prove_package_pure(backend, workspace, package, compiled_program, prover_name, verifier_name, check_proof, foreign_call_resolver_url, inputs_map)?;

    save_proof_to_dir(&proof, &package_name, workspace.proofs_directory_path())?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prove_package_pure(
    backend: &Backend,
    workspace: &Workspace,
    package: &Package,
    compiled_program: CompiledProgram,
    prover_name: &str,
    verifier_name: &str,
    check_proof: bool,
    foreign_call_resolver_url: Option<&str>,
    inputs_map: BTreeMap<String, InputValue>,
) -> Result<(Vec<u8>, String), CliError> {
    let witness_stack = execute_program(&compiled_program, &inputs_map, foreign_call_resolver_url)?;

    // Write public inputs into Verifier.toml
    let public_abi = compiled_program.abi.public_abi();
    // Get the entry point witness for the ABI
    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;
    let (public_inputs, return_value) = public_abi.decode(main_witness)?;

    write_inputs_to_file(
        &public_inputs,
        &return_value,
        &public_abi,
        &package.root_dir,
        verifier_name,
        Format::Toml,
    )?;

    let proof = backend.prove(&compiled_program.program, witness_stack)?;

    if check_proof {
        let public_inputs = public_abi.encode(&public_inputs, return_value)?;
        let valid_proof = backend.verify(&proof, public_inputs, &compiled_program.program)?;

        if !valid_proof {
            return Err(CliError::InvalidProof("".into()));
        }
    }

    Ok((proof, String::from(&package.name)))
}


