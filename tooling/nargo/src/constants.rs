// Directories
/// The directory for the `nargo contract` command output
pub const CONTRACT_DIR: &str = "contract";
/// The directory to store serialized circuit proofs.
pub const PROOFS_DIR: &str = "proofs";
/// The directory to store Noir source files
pub const SRC_DIR: &str = "src";
/// The directory to store circuits' serialized ACIR representations.
pub const TARGET_DIR: &str = "target";
/// The directory to store serialized ACIR representations of exported library functions.
pub const EXPORT_DIR: &str = "export";

// Files
/// The file from which Nargo pulls prover inputs
pub const PROVER_INPUT_FILE: &str = "Prover";
/// The package definition file for a Noir project.
pub const PKG_FILE: &str = "Nargo.toml";

// Extensions
/// The extension for files containing circuit proofs.
pub const PROOF_EXT: &str = "proof";
/// The extension for files containing proof witnesses.
pub const WITNESS_EXT: &str = "gz";
