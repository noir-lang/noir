use std::fs::File;
use std::io::Write;
use std::path::Path;

use acvm::acir::{
    circuit::{Circuit, ExpressionWidth},
    native_types::WitnessMap,
};
use acvm::FieldElement;
use tempfile::tempdir;
use tracing::warn;

use crate::cli::{
    GatesCommand, InfoCommand, ProofAsFieldsCommand, ProveCommand, VerifyCommand,
    VkAsFieldsCommand, WriteVkCommand,
};
use crate::{Backend, BackendError};

impl Backend {
    pub fn get_exact_circuit_size(&self, circuit: &Circuit) -> Result<u32, BackendError> {
        let binary_path = self.assert_binary_exists()?;
        self.assert_correct_version()?;

        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory = temp_directory.path().to_path_buf();

        // Create a temporary file for the circuit
        let circuit_path = temp_directory.join("circuit").with_extension("bytecode");
        let serialized_circuit = Circuit::serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &circuit_path);

        GatesCommand { crs_path: self.crs_directory(), bytecode_path: circuit_path }
            .run(binary_path)
    }

    pub fn get_backend_info(&self) -> Result<ExpressionWidth, BackendError> {
        let binary_path = self.assert_binary_exists()?;
        self.assert_correct_version()?;
        InfoCommand { crs_path: self.crs_directory() }.run(binary_path)
    }

    /// If we cannot get a valid backend, returns `ExpressionWidth::Bound { width: 3 }``
    /// The function also prints a message saying we could not find a backend
    pub fn get_backend_info_or_default(&self) -> ExpressionWidth {
        if let Ok(expression_width) = self.get_backend_info() {
            expression_width
        } else {
            warn!(
                "No valid backend found, ExpressionWidth defaulting to Bounded with a width of 3"
            );
            ExpressionWidth::Bounded { width: 3 }
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn prove(
        &self,
        circuit: &Circuit,
        witness_values: WitnessMap,
    ) -> Result<Vec<u8>, BackendError> {
        let binary_path = self.assert_binary_exists()?;
        self.assert_correct_version()?;

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
        let serialized_circuit = Circuit::serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &bytecode_path);

        // Create proof and store it in the specified path
        let proof_with_public_inputs =
            ProveCommand { crs_path: self.crs_directory(), bytecode_path, witness_path }
                .run(binary_path)?;

        let proof = bb_abstraction_leaks::remove_public_inputs(
            circuit.public_inputs().0.len(),
            &proof_with_public_inputs,
        );
        Ok(proof)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn verify(
        &self,
        proof: &[u8],
        public_inputs: WitnessMap,
        circuit: &Circuit,
    ) -> Result<bool, BackendError> {
        let binary_path = self.assert_binary_exists()?;
        self.assert_correct_version()?;

        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory = temp_directory.path().to_path_buf();

        // Create a temporary file for the proof
        let proof_with_public_inputs =
            bb_abstraction_leaks::prepend_public_inputs(proof.to_vec(), public_inputs);
        let proof_path = temp_directory.join("proof").with_extension("proof");
        write_to_file(&proof_with_public_inputs, &proof_path);

        // Create a temporary file for the circuit
        let bytecode_path = temp_directory.join("circuit").with_extension("bytecode");
        let serialized_circuit = Circuit::serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &bytecode_path);

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory.join("vk");

        WriteVkCommand {
            crs_path: self.crs_directory(),
            bytecode_path,
            vk_path_output: vk_path.clone(),
        }
        .run(binary_path)?;

        // Verify the proof
        VerifyCommand { crs_path: self.crs_directory(), proof_path, vk_path }.run(binary_path)
    }

    pub fn get_intermediate_proof_artifacts(
        &self,
        circuit: &Circuit,
        proof: &[u8],
        public_inputs: WitnessMap,
    ) -> Result<(Vec<FieldElement>, FieldElement, Vec<FieldElement>), BackendError> {
        let binary_path = self.assert_binary_exists()?;
        self.assert_correct_version()?;

        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory = temp_directory.path().to_path_buf();

        // Create a temporary file for the circuit
        //
        let bytecode_path = temp_directory.join("circuit").with_extension("bytecode");
        let serialized_circuit = Circuit::serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &bytecode_path);

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory.join("vk");

        WriteVkCommand {
            crs_path: self.crs_directory(),
            bytecode_path,
            vk_path_output: vk_path.clone(),
        }
        .run(binary_path)?;

        // Create a temporary file for the proof

        let proof_with_public_inputs =
            bb_abstraction_leaks::prepend_public_inputs(proof.to_vec(), public_inputs);
        let proof_path = temp_directory.join("proof").with_extension("proof");
        write_to_file(&proof_with_public_inputs, &proof_path);

        // Now ready to generate intermediate artifacts.

        let proof_as_fields =
            ProofAsFieldsCommand { proof_path, vk_path: vk_path.clone() }.run(binary_path)?;

        let (vk_hash, vk_as_fields) = VkAsFieldsCommand { vk_path }.run(binary_path)?;

        Ok((proof_as_fields, vk_hash, vk_as_fields))
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
