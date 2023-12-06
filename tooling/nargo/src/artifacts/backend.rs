#[allow(unused_variables)]
use std::path::Path;

#[derive(Debug)]
pub struct SRS {
    data: Vec<u8>,
    g2_data: Vec<u8>,
    num_points: u32,
}

impl SRS {
    /// Downloads an SRS from network, for the provided `circuit_metadata`.
    ///
    /// # Arguments
    ///
    /// * `circuit_metadata` - Metadata of the circuit associated with the SRS.
    ///
    /// # Returns
    ///
    /// Returns an `SRS` instance.
    pub fn load_network(circuit_metadata: CircuitMetadata) -> Self {
        todo!()
    }

    /// Loads an SRS from a file located at the specified `path`, using the provided `circuit_metadata`.
    ///
    /// # Arguments
    ///
    /// * `circuit_metadata` - Metadata of the circuit associated with the SRS.
    /// * `path` - The file path where the SRS data is stored.
    ///
    /// # Returns
    ///
    /// Returns an `SRS` instance loaded from the specified file.
    pub fn load_file(circuit_metadata: CircuitMetadata, path: &Path) -> Self {
        todo!()
    }

    /// Saves the SRS to a file located at the specified `path`.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where the SRS data should be saved.
    ///
    /// # Remarks
    ///
    /// This method consumes the `SRS` instance (`self`) to prevent further use after saving.
    pub fn save_file(self, path: &Path) {
        todo!()
    }
}

#[derive(Debug)]
pub struct Circuit {
    bytecode: Vec<u8>,
}

#[derive(Debug)]
pub struct SolvedCircuit {
    solved_witness: Vec<u8>,
}

#[derive(Debug)]
pub struct CircuitMetadata {
    exact_size: u32,
    total_size: u32,
    subgroup_size: u32,
}

pub trait ZKSnarkBackend {
    /// Asks backend about metadata for a given `circuit`.
    ///
    /// This function is responsible for analyzing the provided `circuit`
    /// and extracting relevant metadata such as size, subgroup size, etc.
    ///
    /// # Arguments
    ///
    /// * `circuit` - A `Circuit` instance from which metadata is to be extracted.
    ///
    /// # Returns
    ///
    /// Returns `CircuitMetadata` containing details about the `circuit`.
    fn metadata(circuit: Circuit) -> CircuitMetadata;

    /// Generates a cryptographic proof for a `solved_circuit` using a given `srs` (Structured Reference String).
    ///
    /// This function is used to create a zero-knowledge proof, which proves that
    /// the `solved_circuit` is correct without revealing its contents.
    ///
    /// # Arguments
    ///
    /// * `solved_circuit` - A `SolvedCircuit` instance containing the solution to be proved.
    /// * `srs` - An `SRS` instance used in the proof generation process.
    ///
    /// # Returns
    ///
    /// Returns a `Vec<u8>` representing the generated cryptographic proof.
    fn prove(solved_circuit: SolvedCircuit, srs: SRS) -> Vec<u8>;

    /// Verifies a cryptographic `proof` against a given `circuit` and `srs`.
    ///
    /// This function checks whether the provided `proof` is valid for the given
    /// `circuit` using the specified `srs`. It is an essential part of ensuring
    /// the integrity and correctness of the zero-knowledge proof system.
    ///
    /// # Arguments
    ///
    /// * `proof` - A `Vec<u8>` representing the cryptographic proof to be verified.
    /// * `circuit` - A `Circuit` instance against which the proof is to be verified.
    /// * `srs` - An `SRS` instance used in the verification process.
    ///
    /// # Returns
    ///
    /// Returns `true` if the proof is valid for the given circuit and SRS, otherwise `false`.
    fn verify(proof: Vec<u8>, circuit: Circuit, srs: SRS) -> bool;
}
