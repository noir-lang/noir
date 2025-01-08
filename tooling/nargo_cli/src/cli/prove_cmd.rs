use crate::cli::compile_cmd::get_target_width;
use clap::Args;
use nargo::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};
use nargo::ops::{compile_program, report_errors};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_abi::input_parser::Format;
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};

use super::fs::{
    inputs::{read_inputs_from_file, write_inputs_to_file},
    proof::save_proof_to_dir,
};
use super::{NargoConfig, PackageOptions};
use crate::{
    cli::execute_cmd::execute_program,
    errors::{BackendError, CliError},
};

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

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,

    /// JSON RPC url to solve oracle calls
    #[clap(long)]
    oracle_resolver: Option<String>,
}

pub(crate) fn run(args: ProveCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    // let default_selection = args.package_options.package_selection();
    let selection = args.package_options.package_selection();
    let workspace = resolve_workspace_from_toml(
        &toml_path,
        selection,
        Some(NOIR_ARTIFACT_VERSION_STRING.to_string()),
    )?;

    let mut workspace_file_manager = file_manager_with_stdlib(&workspace.root_dir);
    insert_all_files_for_workspace_into_file_manager(&workspace, &mut workspace_file_manager);
    let parsed_files = parse_all(&workspace_file_manager);

    let binary_packages = workspace.into_iter().filter(|package| package.is_binary());
    for package in binary_packages {
        let compilation_result = compile_program(
            &workspace_file_manager,
            &parsed_files,
            &workspace,
            package,
            &args.compile_options,
            None,
            true,
            false,
        );

        let compiled_program = report_errors(
            compilation_result,
            &workspace_file_manager,
            args.compile_options.deny_warnings,
            args.compile_options.silence_warnings,
        )?;

        let target_width =
            get_target_width(package.expression_width, args.compile_options.expression_width);
        let compiled_program = nargo::ops::transform_program(compiled_program, target_width);

        prove_package(
            &workspace,
            package,
            compiled_program,
            &args.prover_name,
            &args.verifier_name,
            args.verify,
            args.oracle_resolver.as_deref(),
            args.compile_options.pedantic_solving,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prove_package(
    workspace: &Workspace,
    package: &Package,
    compiled_program: CompiledProgram,
    prover_name: &str,
    verifier_name: &str,
    check_proof: bool,
    foreign_call_resolver_url: Option<&str>,
    pedantic_solving: bool,
) -> Result<(), CliError> {
    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &compiled_program.abi)?;

    let witness_stack = execute_program(
        &compiled_program,
        &inputs_map,
        foreign_call_resolver_url,
        Some(workspace.root_dir.clone()),
        Some(package.name.to_string()),
        pedantic_solving,
    )?;

    // Get the entry point witness for the ABI
    let main_witness =
        &witness_stack.peek().expect("Should have at least one witness on the stack").witness;
    let abi = compiled_program.abi;
    let (all_inputs, return_value) = abi.decode(main_witness)?;
    let public_abi = abi.public_abi();

    write_inputs_to_file(
        &all_inputs,
        &return_value,
        &public_abi,
        &package.root_dir,
        verifier_name,
        Format::Toml,
    )?;

    let proof = match compiled_program.plonky2_circuit.unwrap().prove(&inputs_map) {
        Ok(proof) => proof,
        Err(error) => {
            let error_message = format!("{:?}", error);
            return Err(CliError::BackendError(BackendError::UnfitBackend(error_message)));
        }
    };

    if check_proof {
        return Err(CliError::BackendError(BackendError::UnfitBackend("verify operation".into())));
    }

    save_proof_to_dir(&proof, &String::from(&package.name), workspace.proofs_directory_path())?;

    Ok(())
}
