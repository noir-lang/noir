use super::fs::{create_named_dir, program::read_program_from_file, write_to_file};
use super::NargoConfig;
use crate::constants::{PKG_FILE, TARGET_DIR};
use crate::manifest::parse;
use crate::{cli::compile_cmd::compile_circuit, errors::CliError};
use clap::Args;
use nargo::manifest::PackageManifest;
use nargo::ops::{codegen_verifier, preprocess_program};
use noirc_driver::CompileOptions;
use std::path::PathBuf;

/// Generates a Solidity verifier smart contract for the program
#[derive(Debug, Clone, Args)]
pub(crate) struct CodegenVerifierCommand {
    /// The name of the circuit build files (ACIR, proving and verification keys)
    circuit_name: Option<String>,

    #[clap(flatten)]
    compile_options: CompileOptions,
}

pub(crate) fn run(args: CodegenVerifierCommand, config: NargoConfig) -> Result<(), CliError> {
    let backend = crate::backends::ConcreteBackend;

    // TODO(#1201): Should this be a utility function?
    let circuit_build_path = args
        .circuit_name
        .map(|circuit_name| config.program_dir.join(TARGET_DIR).join(circuit_name));

    let preprocessed_program = match circuit_build_path {
        Some(circuit_build_path) => read_program_from_file(circuit_build_path)?,
        None => {
            let compiled_program =
                compile_circuit(&backend, config.program_dir.as_ref(), &args.compile_options)?;
            preprocess_program(&backend, compiled_program)?
        }
    };

    let smart_contract_string = codegen_verifier(&backend, &preprocessed_program.verification_key)?;

    let manifest_path = config.program_dir.join(PKG_FILE);
    let manifest = parse(manifest_path)?;
    let contract_dir = verifier_out_dir(manifest, config.program_dir);
    create_named_dir(&contract_dir, contract_dir.file_name().unwrap().to_str().unwrap());
    let contract_path = contract_dir.join("plonk_vk").with_extension("sol");

    let path = write_to_file(smart_contract_string.as_bytes(), &contract_path);
    println!("Contract successfully created and located at {path}");
    Ok(())
}

/// TODO: Move this function to `PackageManifest` after tracking the root directory.
/// See also: https://github.com/noir-lang/noir/pull/1138#discussion_r1165351237
fn verifier_out_dir(manifest: PackageManifest, package_root: PathBuf) -> PathBuf {
    package_root.join(match manifest.codegen_verifier {
        Some(cv) => cv.out,
        None => "contract.".into(),
    })
}
