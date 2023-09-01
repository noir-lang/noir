use std::path::{Path, PathBuf};

use clap::Args;
use nargo::artifacts::program::PreprocessedProgram;
use nargo::constants::{PROVER_INPUT_FILE, VERIFIER_INPUT_FILE};
use nargo::package::Package;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;

use super::compile_cmd::compile_package;
use super::fs::{
    inputs::{read_inputs_from_file, write_inputs_to_file},
    program::read_program_from_file,
    proof::save_proof_to_dir,
};
use super::NargoConfig;
use crate::{backends::Backend, cli::execute_cmd::execute_program, errors::CliError};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

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

pub(crate) fn run(
    backend: &Backend,
    args: ProveCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
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
pub(crate) fn prove_package(
    backend: &Backend,
    package: &Package,
    prover_name: &str,
    verifier_name: &str,
    proof_dir: &Path,
    circuit_build_path: PathBuf,
    check_proof: bool,
    compile_options: &CompileOptions,
) -> Result<(), CliError> {
    let (preprocessed_program, debug_data) = if circuit_build_path.exists() {
        let program = read_program_from_file(circuit_build_path)?;

        (program, None)
    } else {
        let (context, program) = compile_package(backend, package, compile_options)?;
        let preprocessed_program = PreprocessedProgram {
            backend: String::from(BACKEND_IDENTIFIER),
            abi: program.abi,
            bytecode: program.circuit,
        };
        (preprocessed_program, Some((program.debug, context)))
    };

    let PreprocessedProgram { abi, bytecode, .. } = preprocessed_program;

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

    let proof = backend.prove(&bytecode, solved_witness, false)?;

    if check_proof {
        let public_inputs = public_abi.encode(&public_inputs, return_value)?;
        let valid_proof = backend.verify(&proof, public_inputs, &bytecode, false)?;

        if !valid_proof {
            return Err(CliError::InvalidProof("".into()));
        }
    }

    save_proof_to_dir(&proof, &String::from(&package.name), proof_dir)?;

    Ok(())
}
