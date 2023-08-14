use std::path::{Path, PathBuf};

use acvm::Backend;
use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};
use nargo::ops::{preprocess_program, prove_execution, verify_proof};
use nargo::package::Package;
use nargo_toml::{find_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;

use super::compile_cmd::compile_package;
use super::fs::{
    common_reference_string::{
        read_cached_common_reference_string, update_common_reference_string,
        write_cached_common_reference_string,
    },
    inputs::{read_inputs_from_file, write_inputs_to_file},
    program::read_program_from_file,
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
        let circuit_build_path = workspace.package_build_path(package);

        prove_package(
            backend,
            package,
            &args.prover_name,
            &args.verifier_name,
            &proof_dir,
            circuit_build_path,
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
    circuit_build_path: PathBuf,
    check_proof: bool,
    compile_options: &CompileOptions,
) -> Result<(), CliError<B>> {
    let common_reference_string = read_cached_common_reference_string();

    let (common_reference_string, preprocessed_program, debug_data) = if circuit_build_path.exists()
    {
        let program = read_program_from_file(circuit_build_path)?;
        let common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.bytecode)
                .map_err(CliError::CommonReferenceStringError)?;
        (common_reference_string, program, None)
    } else {
        let (context, program) = compile_package(backend, package, compile_options)?;
        let common_reference_string =
            update_common_reference_string(backend, &common_reference_string, &program.circuit)
                .map_err(CliError::CommonReferenceStringError)?;
        let (program, debug) = preprocess_program(backend, true, &common_reference_string, program)
            .map_err(CliError::ProofSystemCompilerError)?;
        (common_reference_string, program, Some((debug, context)))
    };

    write_cached_common_reference_string(&common_reference_string);

    let PreprocessedProgram { abi, bytecode, proving_key, verification_key, .. } =
        preprocessed_program;

    // Parse the initial witness values from Prover.toml
    let (inputs_map, _) =
        read_inputs_from_file(&package.root_dir, prover_name, Format::Toml, &abi)?;

    let solved_witness = execute_program(backend, bytecode.clone(), &abi, &inputs_map, debug_data)?;

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

    let proving_key =
        proving_key.expect("Proving key should exist as `true` is passed to `preprocess_program`");

    let proof =
        prove_execution(backend, &common_reference_string, &bytecode, solved_witness, &proving_key)
            .map_err(CliError::ProofSystemCompilerError)?;

    if check_proof {
        let public_inputs = public_abi.encode(&public_inputs, return_value)?;
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

        if !valid_proof {
            return Err(CliError::InvalidProof("".into()));
        }
    }

    save_proof_to_dir(&proof, &String::from(&package.name), proof_dir)?;

    Ok(())
}
