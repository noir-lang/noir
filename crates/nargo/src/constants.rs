// Directories
/// The directory for the `nargo contract` command output
pub(crate) const CONTRACT_DIR: &str = "contract";
/// The directory to store serialized circuit proofs.
pub(crate) const PROOFS_DIR: &str = "proofs";
/// The directory to store Noir source files
pub(crate) const SRC_DIR: &str = "src";
/// The directory to store circuits' serialized ACIR representations.
pub(crate) const TARGET_DIR: &str = "target";

// Files
/// The file from which Nargo pulls prover inputs
pub(crate) const PROVER_INPUT_FILE: &str = "Prover";
/// The file from which Nargo pulls verifier inputs
pub(crate) const VERIFIER_INPUT_FILE: &str = "Verifier";
/// The package definition file for a Noir project.
pub(crate) const PKG_FILE: &str = "Nargo.toml";

// Extensions
/// The extension for files containing circuit proofs.
pub(crate) const PROOF_EXT: &str = "proof";
/// The extension for files containing circuit ACIR representations.
pub(crate) const ACIR_EXT: &str = "acir";
/// The extension for files containing proof witnesses.
pub(crate) const WITNESS_EXT: &str = "tr";
