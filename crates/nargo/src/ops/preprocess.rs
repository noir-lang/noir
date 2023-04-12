use acvm::acir::circuit::Circuit;
use acvm::{checksum_constraint_system, ProofSystemCompiler};

use crate::NargoError;

pub fn checksum_acir(circuit: &Circuit) -> [u8; 4] {
    checksum_constraint_system(circuit).to_be_bytes()
}

/// The result of preprocessing the ACIR bytecode.
/// The proving, verification key and circuit are backend specific.
///
/// The circuit is backend specific because at the end of compilation
/// an optimization pass is applied which will transform the bytecode into
/// a format that the backend will accept; removing unsupported gates
/// is one example of this.
pub struct PreprocessedData {
    pub proving_key: Vec<u8>,
    pub verification_key: Vec<u8>,
    pub program_checksum: [u8; 4],
}

pub fn preprocess_circuit(
    backend: &impl ProofSystemCompiler,
    circuit: &Circuit,
) -> Result<PreprocessedData, NargoError> {
    let (proving_key, verification_key) = backend.preprocess(circuit);
    let program_checksum = checksum_acir(circuit);

    Ok(PreprocessedData { proving_key, verification_key, program_checksum })
}
