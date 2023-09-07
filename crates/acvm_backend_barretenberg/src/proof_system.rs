use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use acvm::acir::circuit::Opcode;
use acvm::acir::{circuit::Circuit, native_types::WitnessMap};
use acvm::FieldElement;
use acvm::Language;
use tempfile::tempdir;

use crate::cli::{GatesCommand, InfoCommand, ProveCommand, VerifyCommand, WriteVkCommand};
use crate::{assert_binary_exists, Backend, BackendError};

impl Backend {
    pub fn get_exact_circuit_size(&self, circuit: &Circuit) -> Result<u32, BackendError> {
        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory = temp_directory.path().to_path_buf();

        // Create a temporary file for the circuit
        let circuit_path = temp_directory.join("circuit").with_extension("bytecode");
        let serialized_circuit = serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &circuit_path);

        let binary_path = assert_binary_exists(self);
        GatesCommand { crs_path: self.crs_directory(), bytecode_path: circuit_path }
            .run(&binary_path)
    }

    pub fn get_backend_info(
        &self,
    ) -> Result<(Language, Box<impl Fn(&Opcode) -> bool>), BackendError> {
        let binary_path = assert_binary_exists(self);
        InfoCommand { crs_path: self.crs_directory() }.run(&binary_path)
    }

    pub fn prove(
        &self,
        circuit: &Circuit,
        witness_values: WitnessMap,
        is_recursive: bool,
    ) -> Result<Vec<u8>, BackendError> {
        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory = temp_directory.path().to_path_buf();

        // Create a temporary file for the witness
        let serialized_witnesses: Vec<u8> =
            witness_values.try_into().expect("could not serialize witness map");
        let witness_path = temp_directory.join("witness").with_extension("tr");
        write_to_file(&serialized_witnesses, &witness_path);

        // Create a temporary file for the circuit
        //
        let bytecode_path = temp_directory.join("circuit").with_extension("bytecode");
        let serialized_circuit = serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &bytecode_path);

        let proof_path = temp_directory.join("proof").with_extension("proof");

        let binary_path = assert_binary_exists(self);
        // Create proof and store it in the specified path
        ProveCommand {
            crs_path: self.crs_directory(),
            is_recursive,
            bytecode_path,
            witness_path,
            proof_path: proof_path.clone(),
        }
        .run(&binary_path)?;

        let proof_with_public_inputs = read_bytes_from_file(&proof_path).unwrap();

        // Barretenberg return the proof prepended with the public inputs.
        //
        // This is not how the API expects the proof to be formatted,
        // so we remove the public inputs from the proof.
        //
        // TODO: As noted in the verification procedure, this is an abstraction leak
        // TODO: and will need modifications to barretenberg
        let proof =
            remove_public_inputs(circuit.public_inputs().0.len(), &proof_with_public_inputs);
        Ok(proof)
    }

    pub fn verify(
        &self,
        proof: &[u8],
        public_inputs: WitnessMap,
        circuit: &Circuit,
        is_recursive: bool,
    ) -> Result<bool, BackendError> {
        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory = temp_directory.path().to_path_buf();

        // Unlike when proving, we omit any unassigned witnesses.
        // Witness values should be ordered by their index but we skip over any indices without an assignment.
        let flattened_public_inputs: Vec<FieldElement> =
            public_inputs.into_iter().map(|(_, el)| el).collect();

        // Barretenberg expects the proof to be prepended with the public inputs.
        //
        // TODO: This is an abstraction leak and barretenberg's API should accept the public inputs
        // TODO: separately and then prepend them internally
        let proof_with_public_inputs =
            prepend_public_inputs(proof.to_vec(), flattened_public_inputs.to_vec());

        // Create a temporary file for the proof
        let proof_path = temp_directory.join("proof").with_extension("proof");
        write_to_file(&proof_with_public_inputs, &proof_path);

        // Create a temporary file for the circuit
        let bytecode_path = temp_directory.join("circuit").with_extension("bytecode");
        let serialized_circuit = serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &bytecode_path);

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory.join("vk");

        let binary_path = assert_binary_exists(self);
        WriteVkCommand {
            crs_path: self.crs_directory(),
            is_recursive,
            bytecode_path,
            vk_path_output: vk_path.clone(),
        }
        .run(&binary_path)?;

        // Verify the proof
        let valid_proof =
            VerifyCommand { crs_path: self.crs_directory(), is_recursive, proof_path, vk_path }
                .run(&binary_path);

        Ok(valid_proof)
    }
}

pub(super) fn write_to_file(bytes: &[u8], path: &Path) -> String {
    let display = path.display();

    let mut file = match File::create(path) {
        Err(why) => panic!("couldn't create {display}: {why}"),
        Ok(file) => file,
    };

    match file.write_all(bytes) {
        Err(why) => panic!("couldn't write to {display}: {why}"),
        Ok(_) => display.to_string(),
    }
}

pub(super) fn read_bytes_from_file(path: &Path) -> std::io::Result<Vec<u8>> {
    // Open the file for reading.
    let mut file = File::open(path)?;

    // Create a buffer to store the bytes.
    let mut buffer = Vec::new();

    // Read bytes from the file.
    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

/// Removes the public inputs which are prepended to a proof by Barretenberg.
fn remove_public_inputs(num_pub_inputs: usize, proof: &[u8]) -> Vec<u8> {
    // Barretenberg prepends the public inputs onto the proof so we need to remove
    // the first `num_pub_inputs` field elements.
    let num_bytes_to_remove = num_pub_inputs * (FieldElement::max_num_bytes() as usize);
    proof[num_bytes_to_remove..].to_vec()
}

/// Prepends a set of public inputs to a proof.
fn prepend_public_inputs(proof: Vec<u8>, public_inputs: Vec<FieldElement>) -> Vec<u8> {
    if public_inputs.is_empty() {
        return proof;
    }

    let public_inputs_bytes =
        public_inputs.into_iter().flat_map(|assignment| assignment.to_be_bytes());

    public_inputs_bytes.chain(proof.into_iter()).collect()
}

// TODO: See nargo/src/artifacts/mod.rs
// TODO: This method should live in ACVM and be the default method for serializing/deserializing circuits
pub(super) fn serialize_circuit(circuit: &Circuit) -> Vec<u8> {
    let mut circuit_bytes: Vec<u8> = Vec::new();
    circuit.write(&mut circuit_bytes).unwrap();
    circuit_bytes
}
