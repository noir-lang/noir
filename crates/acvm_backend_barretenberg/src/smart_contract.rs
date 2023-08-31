use super::proof_system::{serialize_circuit, write_to_file};
use crate::{
    bb::{ContractCommand, WriteVkCommand},
    proof_system::read_bytes_from_file,
    Backend, BackendError,
};
use acvm::acir::circuit::Circuit;
use tempfile::tempdir;

/// Embed the Solidity verifier file
const ULTRA_VERIFIER_CONTRACT: &str = include_str!("contract.sol");

impl Backend {
    pub fn eth_contract(&self, circuit: &Circuit) -> Result<String, BackendError> {
        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory_path = temp_directory.path();
        let temp_dir_path = temp_directory_path.to_str().unwrap();

        // Create a temporary file for the circuit
        let circuit_path = temp_directory_path.join("circuit").with_extension("bytecode");
        let serialized_circuit = serialize_circuit(circuit);
        write_to_file(serialized_circuit.as_bytes(), &circuit_path);

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory_path.join("vk").to_str().unwrap().to_string();
        WriteVkCommand {
            verbose: false,
            path_to_crs: temp_dir_path.to_string(),
            is_recursive: false,
            path_to_bytecode: circuit_path.as_os_str().to_str().unwrap().to_string(),
            path_to_vk_output: vk_path.clone(),
        }
        .run()
        .expect("write vk command failed");

        let path_to_contract = temp_directory_path.join("contract").to_str().unwrap().to_string();
        ContractCommand {
            verbose: false,
            path_to_crs: temp_dir_path.to_string(),
            path_to_vk: vk_path,
            path_to_contract: path_to_contract.clone(),
        }
        .run()
        .expect("contract command failed");

        let verification_key_library_bytes = read_bytes_from_file(&path_to_contract).unwrap();
        let verification_key_library = String::from_utf8(verification_key_library_bytes).unwrap();

        drop(temp_directory);
        Ok(format!("{verification_key_library}{ULTRA_VERIFIER_CONTRACT}"))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use acvm::acir::{
        circuit::{Circuit, Opcode, PublicInputs},
        native_types::{Expression, Witness},
    };

    #[test]
    #[serial_test::serial]
    fn test_smart_contract() {
        use crate::Backend;

        let expression = &(Witness(1) + Witness(2)) - &Expression::from(Witness(3));
        let constraint = Opcode::Arithmetic(expression);

        let circuit = Circuit {
            current_witness_index: 4,
            opcodes: vec![constraint],
            private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
        };

        let bb = Backend::default();

        let contract = bb.eth_contract(&circuit).unwrap();

        assert!(contract.contains("contract BaseUltraVerifier"));
        assert!(contract.contains("contract UltraVerifier"));
        assert!(contract.contains("library UltraVerificationKey"));
    }
}
