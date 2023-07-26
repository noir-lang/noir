// Directories
/// Contains all things `nargo` in home folder
pub(crate) const NARGO_HOME_FOLDER_NAME: &str = ".nargo";
/// Contains backend related files
pub(crate) const NARGO_BACKENDS_FOLDER_NAME: &str = "backends";
// / The directory to store serialized circuit proofs.
// pub(crate) const PROOFS_DIR: &str = "target";
/// The directory to store Noir source files
pub(crate) const SRC_DIR: &str = "src";
/// The directory to store circuits' serialized ACIR representations.
pub(crate) const TARGET_DIR: &str = "target";

// Files
/// The file from which Nargo pulls prover inputs
pub(crate) const PROVER_INPUT_FILE: &str = "Prover";
/// The file from which Nargo pulls verifier inputs
pub(crate) const VERIFIER_INPUT_FILE: &str = "Verifier";
/// The file from which Nargo pulls verifier inputs
pub(crate) const META_INPUT_EXT: &str = "toml";

/// The package definition file for a Noir project.
pub(crate) const PKG_FILE: &str = "Nargo.toml";
/// Global config file name
pub(crate) const NARGO_GLOBAL_CONFIG_FILENAME: &str = "nargo.toml";

// Extensions
/// The extension for files containing circuit proofs.
pub(crate) const PROOF_EXT: &str = "proof";
/// The extension for files containing proof witnesses.
pub(crate) const WITNESS_EXT: &str = "tr";
/// The extension for files containing ACIR code.
pub(crate) const ACIR_EXT: &str = "acir.json";
/// The extension for files containing ACIR code.
pub(crate) const VERIFICATION_KEY_EXT: &str = "vk";
/// The extension for files Solidity contract code.
pub(crate) const CONTRACT_EXT: &str = "sol";

// Backend Vendor Commands
/// Prove command 
pub(crate) const PROVE_SUB_CMD: &str = "prove";
/// Verify command 
pub(crate) const VERIFY_SUB_CMD: &str = "verify";
/// Gates command 
pub(crate) const GATES_SUB_CMD: &str = "gates";
/// Contract command 
pub(crate) const CONTRACT_SUB_CMD: &str = "contract";
