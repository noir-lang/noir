use super::proof_system::{serialize_circuit, write_to_file};
use crate::{
    assert_binary_exists,
    cli::{ContractCommand, WriteVkCommand},
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
        let temp_directory_path = temp_directory.path().to_path_buf();

        // Create a temporary file for the circuit
        let bytecode_path = temp_directory_path.join("circuit").with_extension("bytecode");
        let serialized_circuit = serialize_circuit(circuit);
        write_to_file(serialized_circuit.as_bytes(), &bytecode_path);

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory_path.join("vk");

        let binary_path = assert_binary_exists(self);
        WriteVkCommand {
            verbose: false,
            crs_path: self.backend_directory(),
            is_recursive: false,
            bytecode_path,
            vk_path_output: vk_path.clone(),
        }
        .run(&binary_path)?;

        let contract_path = temp_directory_path.join("contract");
        ContractCommand {
            verbose: false,
            crs_path: self.backend_directory(),
            vk_path,
            contract_path: contract_path.clone(),
        }
        .run(&binary_path)?;

        let verification_key_library_bytes = read_bytes_from_file(&contract_path).unwrap();
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

    use crate::get_bb;

    #[test]
    #[serial_test::serial]
    fn test_smart_contract() {
        let expression = &(Witness(1) + Witness(2)) - &Expression::from(Witness(3));
        let constraint = Opcode::Arithmetic(expression);

        let circuit = Circuit {
            current_witness_index: 4,
            opcodes: vec![constraint],
            private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
        };

        let contract = get_bb().eth_contract(&circuit).unwrap();

        assert!(contract.contains("contract BaseUltraVerifier"));
        assert!(contract.contains("contract UltraVerifier"));
        assert!(contract.contains("library UltraVerificationKey"));
    }
}
