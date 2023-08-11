use std::path::Path;

use acvm::Backend;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};
use nargo::ops::{prove_execution, verify_proof};
use nargo::package::Package;
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;

use super::compile_cmd::compile_package_and_save;
use super::fs::common_reference_string::update_common_reference_string;
use super::fs::{
    common_reference_string::read_cached_common_reference_string,
    inputs::{read_inputs_from_file, write_inputs_to_file},
    proof::save_proof_to_dir,
};
use super::NargoConfig;
use crate::{cli::execute_cmd::execute_program, errors::CliError};

/// Create proof for this program. The proof is returned as a hex encoded string.
#[derive(Debug, Clone, Args)]
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
}

pub(crate) fn run<B: Backend>(
    backend: &B,
    args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError<B>> {
    let toml_path = find_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let proof_dir = workspace.proofs_directory_path();

    for package in &workspace {
        prove_package(
            backend,
            package,
            &args.prover_name,
            &args.verifier_name,
            &proof_dir,
            args.verify,
            &args.compile_options,
        )?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn prove_package<B: Backend>(
    backend: &B,
    package: &Package,
    prover_name: &str,
    verifier_name: &str,
    proof_dir: &Path,
    check_proof: bool,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let (debug_and_context, preprocessed_program) =
        compile_package_and_save(backend, package, compile_options, false)?;

    let common_reference_string = read_cached_common_reference_string();
    let common_reference_string = update_common_reference_string(
        backend,
        &common_reference_string,
        &preprocessed_program.bytecode,
    )
    .map_err(CliError::CommonReferenceStringError)?;

    let (proving_key, verification_key) = backend
        .preprocess(&common_reference_string, &preprocessed_program.bytecode)
        .map_err(CliError::ProofSystemCompilerError)?;

    let PreprocessedProgram { abi, bytecode, .. } = preprocessed_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &abi)?;

    let solved_witness =
        execute_program(backend, bytecode.clone(), &abi, &inputs_map, Some(debug_and_context))?;

    // Write public inputs into Verifier.toml
    let public_abi = abi.public_abi();
    let (public_inputs, return_value) = public_abi.decode(&solved_witness)?;

    write_inputs_to_file(
        &public_inputs,
        &return_value,
        &public_abi,
        &package.root_dir,
        verifier_name,
        Format::Toml,
    )?;

    let proof =
        prove_execution(backend, &common_reference_string, &bytecode, solved_witness, &proving_key)
            .map_err(CliError::ProofSystemCompilerError)?;

    if check_proof {
        let public_inputs = public_abi.encode(&public_inputs, return_value)?;
        let valid_proof = verify_proof(
            backend,
            &common_reference_string,
            &bytecode,
            &proof,
            public_inputs,
            &verification_key,
        )
        .map_err(CliError::ProofSystemCompilerError)?;

        if !valid_proof {
            return Err(CliError::InvalidProof("".into()));
        }
    }

    save_proof_to_dir(&proof, &String::from(&package.name), proof_dir)?;

    Ok(())
}
