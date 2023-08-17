use super::NargoConfig;
use super::{
    compile_cmd::compile_package,
    fs::{
        common_reference_string::{
            read_cached_common_reference_string, update_common_reference_string,
            write_cached_common_reference_string,
        },
        inputs::read_inputs_from_file,
        load_hex_data,
        program::read_program_from_file,
    },
};
use crate::errors::CliError;

use acvm::Backend;
use clap::Args;
use nargo::constants::{PROOF_EXT, VERIFIER_INPUT_FILE};
use nargo::ops::{preprocess_program, verify_proof};
use nargo::{artifacts::program::PreprocessedProgram, package::Package};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;
use std::path::{Path, PathBuf};

/// Given a proof and a program, verify whether the proof is valid
#[derive(Debug, Clone, Args)]
pub(crate) struct VerifyCommand {
    /// The name of the toml file which contains the inputs for the verifier
    #[clap(long, short, default_value = VERIFIER_INPUT_FILE)]
    verifier_name: String,

    /// The name of the package verify
    #[clap(long, conflicts_with = "workspace")]
    package: Option<CrateName>,

    /// Verify all packages in the workspace
    #[clap(long, conflicts_with = "package")]
    workspace: bool,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: VerifyCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let proofs_dir = workspace.proofs_directory_path();

    for package in &workspace {
        let circuit_build_path = workspace.package_build_path(package);

        let proof_path = proofs_dir.join(String::from(&package.name)).with_extension(PROOF_EXT);

        verify_package(
            backend,
            package,
            &proof_path,
            circuit_build_path,
            &args.verifier_name,
            &args.compile_options,
        )?;
    }

    Ok(())
}

fn verify_package<B: Backend>(
    backend: &B,
    package: &Package,
    proof_path: &Path,
    circuit_build_path: PathBuf,
    verifier_name: &str,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let common_reference_string = read_cached_common_reference_string();

    let (common_reference_string, preprocessed_program) = if circuit_build_path.exists() {
        let program = read_program_from_file(circuit_build_path)?;
        let common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.bytecode)
                .map_err(CliError::CommonReferenceStringError)?;
        (common_reference_string, program)
    } else {
        let (_, program) = compile_package(backend, package, compile_options)?;
        let common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.circuit)
                .map_err(CliError::CommonReferenceStringError)?;
        let (program, _) = preprocess_program(backend, true, &common_reference_string, program)
            .map_err(CliError::ProofSystemCompilerError)?;
        (common_reference_string, program)
    };

    write_cached_common_reference_string(&common_reference_string);

    let PreprocessedProgram { abi, bytecode, verification_key, .. } = preprocessed_program;

    // Load public inputs (if any) from `verifier_name`.
    let public_abi = abi.public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(&package.root_dir, verifier_name, Format::Toml, &public_abi)?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;
    let proof = load_hex_data(proof_path)?;

    let verification_key = verification_key
        .expect("Verification key should exist as `true` is passed to `preprocess_program`");
    let valid_proof = verify_proof(
        backend,
        &common_reference_string,
        &bytecode,
        &proof,
        public_inputs,
        &verification_key,
    )
    .map_err(CliError::ProofSystemCompilerError)?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path.to_path_buf()))
    }
}
