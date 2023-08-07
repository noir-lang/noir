use acvm::acir::circuit::Circuit;
use noirc_abi::Abi;
use serde::{Deserialize, Serialize};

/// `PreprocessedProgram` represents a Noir program which has been preprocessed by a particular backend proving system.
///
/// This differs from a generic Noir program artifact in that:
/// - The ACIR bytecode has had an optimization pass applied to tailor it for the backend.
/// - Proving and verification keys have been pregenerated based on this ACIR.
#[derive(Serialize, Deserialize, Debug)]
pub struct PreprocessedProgram {
    pub backend: String,
    pub abi: Abi,

    #[serde(
        serialize_with = "super::serialize_circuit",
        deserialize_with = "super::deserialize_circuit"
    )]
    pub bytecode: Circuit,

    pub proving_key: Option<Vec<u8>>,
    pub verification_key: Option<Vec<u8>>,
}
