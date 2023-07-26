use std::{path::{PathBuf}, env};



use clap::Args;
use nameof::name_of;

use crate::constants;

#[derive(Debug, Clone, Args)]
pub(crate) struct ProverMetaArtifact {
    #[arg(env, long, default_value_os_t = default_pmeta_path())]
    pub(crate) nargo_prover_meta_path: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerifierMetaArtifact {
    #[arg(env, long, default_value_os_t = default_vmeta_path())]
    pub(crate) nargo_verifier_meta_path: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ProofArtifact {
    /// ACIR file desired location path
    #[arg(env, long, default_value_os_t = default_proof_path())]
    pub(crate) nargo_proof_path: PathBuf,

}

#[derive(Debug, Clone, Args)]
pub(crate) struct VerificationKeyArtifact {
    /// Witness file desired location path
    #[arg(env, long, default_value_os_t = default_vkey_path())]
    pub(crate) nargo_verification_key_path: PathBuf,
}


#[derive(clap::Args, Clone, Debug)]
pub(crate) struct PackageRootOption {
    #[arg(env, long, default_value_os_t = std::env::current_dir().unwrap())]
    pub(crate) nargo_package_root: PathBuf,    
}

#[derive(Debug, Clone, Args)]
pub(crate) struct WitnessArtifact {
    /// Witness file desired location path
    #[arg(env, long, default_value_os_t = crate::cli::arguments::default_witness_path())]
    pub(crate) nargo_witness_path: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub(crate) struct ContractArtifact {
    /// Witness file desired location path
    #[arg(env, long, default_value_os_t = crate::cli::arguments::default_contract_path())]
    pub(crate) nargo_contract_path: PathBuf,
}

#[non_exhaustive]
#[derive(clap::Args, Clone, Debug)]
pub(crate) struct NargoConfig {
    #[arg(env, long, default_value_os_t = std::env::current_dir().unwrap())]
    pub(crate) nargo_package_root: PathBuf,

    #[arg(env, long,  hide=true)]
    pub(crate) nargo_target_dir: Option<PathBuf>,

    #[arg(env, long, default_value_os_t = default_artifact_name())]
    pub(crate) nargo_artifact_name: String,

    /// Path to nargo artifact containing ACIR. Defaults to $NARGO_TARGET_DIR/target/${parent_folder_name}.acir.json
    #[arg(env, long, hide=true, global=true)]
    pub(crate) nargo_artifact_path: Option<PathBuf>,

    /// Path to solved wintess. Defaults to $NARGO_TARGET_DIR/target/${parent_folder_name}.tr
    #[arg(env, long, hide=true, global=true)]
    pub(crate) nargo_witness_path: Option<PathBuf>,

    /// Path to proof artifact. Defaults to $NARGO_TARGET_DIR/target/${parent_folder_name}.proof
    #[arg(env, long, hide=true)]
    pub(crate) nargo_proof_path: Option<PathBuf>,

    /// Path to proof verification key. Defaults to $NARGO_TARGET_DIR/target/${parent_folder_name}.vk
    #[arg(env, long, hide=true, global=true)]
    pub(crate) nargo_verification_key_path: Option<PathBuf>,

    /// Path to solved wintess. Defaults to $NARGO_TARGET_DIR/target/${parent_folder_name}.sol
    #[arg(env, long, hide=true, global=true)]
    pub(crate) nargo_contract_path: Option<PathBuf>,

}

fn default_artifact_name() -> String {
    
    let package_root_path = default_package_root();
    
    package_root_path
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_string_lossy()
        .to_string()

}


pub(crate) fn default_proof_path() -> PathBuf {

    let package_root_path = default_package_root();
    default_artifact_path(package_root_path, default_artifact_name(), constants::PROOF_EXT)

}

pub(crate) fn default_witness_path() -> PathBuf {

    let package_root_path = default_package_root();
    default_artifact_path(package_root_path, default_artifact_name(), constants::WITNESS_EXT)

}

pub(crate) fn default_contract_path() -> PathBuf {

    let package_root_path = default_package_root();
    default_artifact_path(package_root_path, default_artifact_name(), constants::CONTRACT_EXT)

}

pub(crate) fn default_vkey_path() -> PathBuf {

    let package_root_path = default_package_root();
    default_artifact_path(package_root_path, default_artifact_name(), constants::VERIFICATION_KEY_EXT)

}

pub(crate) fn default_pmeta_path() -> PathBuf {

    let package_root_path = default_package_root();
    let mut artifact_file_name = String::from(constants::PROVER_INPUT_FILE);
    artifact_file_name.push('.');
    artifact_file_name.push_str(constants::META_INPUT_EXT);

    package_root_path
        .join(artifact_file_name)

}

pub(crate) fn default_vmeta_path() -> PathBuf {

    let package_root_path = default_package_root();
    let mut artifact_file_name = String::from(constants::VERIFIER_INPUT_FILE);
    artifact_file_name.push('.');
    artifact_file_name.push_str(constants::META_INPUT_EXT);

    package_root_path
        .join(artifact_file_name)

}

pub(crate) fn default_artifact_path(
    package_root: PathBuf,
    artifact_name: String,
    ext: &str,
) -> PathBuf {
    
    let mut artifact_file_name = artifact_name;
    artifact_file_name.push('.');
    artifact_file_name.push_str(ext);
    
    package_root
        .join(constants::TARGET_DIR)
        .join(artifact_file_name)

}

fn default_package_root(
) -> PathBuf {
    let env_package_root_key = name_of!(nargo_package_root in PackageRootOption).to_uppercase();
    match env::var(env_package_root_key) {
        Ok(val) => PathBuf::from(val),
        Err(_) => env::current_dir().unwrap(),
    }
}
