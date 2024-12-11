use super::fs::{inputs::read_inputs_from_file, load_hex_data};
use super::{NargoConfig, PackageOptions};
use crate::cli::compile_cmd::get_target_width;
use crate::errors::BackendError;
use crate::errors::CliError;

use clap::Args;
use nargo::constants::{PROOF_EXT, VERIFIER_INPUT_FILE};
use nargo::ops::{compile_program, report_errors};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo::{insert_all_files_for_workspace_into_file_manager, parse_all};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml};
use noirc_abi::input_parser::Format;
use noirc_driver::{
    file_manager_with_stdlib, CompileOptions, CompiledProgram, NOIR_ARTIFACT_VERSION_STRING,
};

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
#[clap(visible_alias = "v")]
pub(crate) struct VerifyCommand {
    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = VERIFIER_INPUT_FILE)]
    verifier_name: String,

    #[clap(flatten)]
    pub(super) package_options: PackageOptions,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: VerifyCommand, config: NargoConfig) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
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

        verify_package(&workspace, package, compiled_program, &args.verifier_name)?;
    }

    Ok(())
}

fn verify_package(
    workspace: &Workspace,
    package: &Package,
    compiled_program: CompiledProgram,
    verifier_name: &str,
) -> Result<(), CliError> {
    let public_abi = compiled_program.abi.public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(&package.root_dir, verifier_name, Format::Toml, &public_abi)?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;

    let proof_path =
        workspace.proofs_directory_path().join(package.name.to_string()).with_extension(PROOF_EXT);

    let proof = load_hex_data(&proof_path)?;

    let valid_proof =
        match compiled_program.plonky2_circuit.as_ref().unwrap().verify(&proof, public_inputs) {
            Ok(valid_proof) => valid_proof,
            Err(error) => {
                let error_message = format!("{:?}", error);
                return Err(CliError::BackendError(BackendError::UnfitBackend(error_message)));
            }
        };

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path))
    }
}
