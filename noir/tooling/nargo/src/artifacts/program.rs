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
    pub noir_version: String,

    /// Hash of the [`Program`][noirc_frontend::monomorphization::ast::Program] from which this [`PreprocessedProgram`]
    /// was compiled.
    ///
    /// Used to short-circuit compilation in the case of the source code not changing since the last compilation.
    pub hash: u64,

    pub abi: Abi,

    #[serde(
        serialize_with = "Circuit::serialize_circuit_base64",
        deserialize_with = "Circuit::deserialize_circuit_base64"
    )]
    pub bytecode: Circuit,
}
