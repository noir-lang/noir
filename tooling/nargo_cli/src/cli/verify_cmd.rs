use super::NargoConfig;
use super::{
    compile_cmd::compile_bin_package,
    fs::{inputs::read_inputs_from_file, load_hex_data, program::read_program_from_file},
};
use crate::{backends::Backend, errors::CliError};

use acvm::acir::circuit::Opcode;
use acvm::Language;
use clap::Args;
use nargo::constants::{PROOF_EXT, VERIFIER_INPUT_FILE};
use nargo::package::PackageType;
use nargo::{artifacts::program::PreprocessedProgram, package::Package};
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::CompileOptions;
use noirc_frontend::graph::CrateName;
use std::path::{Path, PathBuf};

// TODO(#1388): pull this from backend.
const BACKEND_IDENTIFIER: &str = "acvm-backend-barretenberg";

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

pub(crate) fn run(
    backend: &Backend,
    args: VerifyCommand,
    config: NargoConfig,
) -> Result<(), CliError> {
    let toml_path = get_package_manifest(&config.program_dir)?;
    let default_selection =
        if args.workspace { PackageSelection::All } else { PackageSelection::DefaultOrAll };
    let selection = args.package.map_or(default_selection, PackageSelection::Selected);
    let workspace = resolve_workspace_from_toml(&toml_path, selection)?;
    let proofs_dir = workspace.proofs_directory_path();

    let (np_language, is_opcode_supported) = backend.get_backend_info()?;
    for package in &workspace {
        let circuit_build_path = workspace.package_build_path(package);

        let proof_path = proofs_dir.join(String::from(&package.name)).with_extension(PROOF_EXT);

        match package.package_type {
            PackageType::Binary => {
                verify_package(
                    backend,
                    package,
                    &proof_path,
                    circuit_build_path,
                    &args.verifier_name,
                    &args.compile_options,
                    np_language,
                    &is_opcode_supported,
                )?;
            }

            // Nargo does not support verifying proofs for these package types
            PackageType::Contract | PackageType::Library => (),
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn verify_package(
    backend: &Backend,
    package: &Package,
    proof_path: &Path,
    circuit_build_path: PathBuf,
    verifier_name: &str,
    compile_options: &CompileOptions,
    np_language: Language,
    is_opcode_supported: &impl Fn(&Opcode) -> bool,
) -> Result<(), CliError> {
    let preprocessed_program = if circuit_build_path.exists() {
        read_program_from_file(circuit_build_path)?
    } else {
        let program =
            compile_bin_package(package, compile_options, np_language, &is_opcode_supported)?;

        PreprocessedProgram {
            backend: String::from(BACKEND_IDENTIFIER),
            abi: program.abi,
            bytecode: program.circuit,
        }
    };

    let PreprocessedProgram { abi, bytecode, .. } = preprocessed_program;

    // Load public inputs (if any) from `verifier_name`.
    let public_abi = abi.public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(&package.root_dir, verifier_name, Format::Toml, &public_abi)?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;
    let proof = load_hex_data(proof_path)?;

    let valid_proof = backend.verify(&proof, public_inputs, &bytecode, false)?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path.to_path_buf()))
    }
}
