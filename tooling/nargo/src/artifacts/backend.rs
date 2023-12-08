#[derive(Debug)]
pub struct Circuit {
    bytecode: Vec<u8>,
}

#[derive(Debug)]
pub struct SolvedCircuit {
    solved_circuit: Vec<u8>,
}

// The `Context` struct holds the context for zk-SNARK operations.
#[derive(Debug)]
pub struct Context {
    circuit: Circuit,
}

// The `ZKSnarkBackend` trait defines the interface for zk-SNARK operations.
pub trait ZKSnarkBackend {
    /// Generates a cryptographic proof for a `solved_circuit`.
    ///
    /// # Arguments
    ///
    /// * `solved_circuit` - A `SolvedCircuit` instance containing the circuit to be proved.
    /// * `ctx` - A `Context` instance containing the circuit proving context.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<u8>` representing the generated cryptographic proof.
    fn prove(solved_circuit: SolvedCircuit, ctx: Context) -> Vec<u8>;

    /// Verifies a cryptographic `proof` against a given `circuit`.
    ///
    /// # Arguments
    ///
    /// * `proof` - A `Vec<u8>` representing the cryptographic proof to be verified.
    /// * `ctx` - A `Context` instance containing the circuit verifying context.
    ///
    /// # Returns
    ///
    /// Returns `true` if the proof is valid for the given circuit, otherwise `false`.
    fn verify(proof: Vec<u8>, ctx: Context) -> bool;
}
