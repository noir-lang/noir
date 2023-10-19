use super::NargoConfig;
use super::{
    compile_cmd::compile_bin_package,
    fs::{inputs::read_inputs_from_file, load_hex_data},
};
use crate::{backends::Backend, errors::CliError};

use clap::Args;
use nargo::constants::{PROOF_EXT, VERIFIER_INPUT_FILE};
use nargo::package::Package;
use nargo::workspace::Workspace;
use nargo_toml::{get_package_manifest, resolve_workspace_from_toml, PackageSelection};
use noirc_abi::input_parser::Format;
use noirc_driver::{CompileOptions, CompiledProgram};
use noirc_frontend::graph::CrateName;

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

    let (np_language, opcode_support) = backend.get_backend_info()?;
    for package in &workspace {
        let program = compile_bin_package(
            &workspace,
            package,
            &args.compile_options,
            false,
            np_language,
            &|opcode| opcode_support.is_opcode_supported(opcode),
        )?;

        verify_package(backend, &workspace, package, program, &args.verifier_name)?;
    }

    Ok(())
}

fn verify_package(
    backend: &Backend,
    workspace: &Workspace,
    package: &Package,
    compiled_program: CompiledProgram,
    verifier_name: &str,
) -> Result<(), CliError> {
    // Load public inputs (if any) from `verifier_name`.
    let public_abi = compiled_program.abi.public_abi();
    let (public_inputs_map, return_value) =
        read_inputs_from_file(&package.root_dir, verifier_name, Format::Toml, &public_abi)?;

    let public_inputs = public_abi.encode(&public_inputs_map, return_value)?;

    let proof_path =
        workspace.proofs_directory_path().join(package.name.to_string()).with_extension(PROOF_EXT);

    let proof = load_hex_data(&proof_path)?;

    let valid_proof = backend.verify(&proof, public_inputs, &compiled_program.circuit, false)?;

    if valid_proof {
        Ok(())
    } else {
        Err(CliError::InvalidProof(proof_path))
    }
}
