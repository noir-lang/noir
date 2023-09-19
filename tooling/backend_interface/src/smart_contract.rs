use super::proof_system::{serialize_circuit, write_to_file};
use crate::{
    cli::{ContractCommand, WriteVkCommand},
    Backend, BackendError,
};
use acvm::acir::circuit::Circuit;
use tempfile::tempdir;

impl Backend {
    pub fn eth_contract(&self, circuit: &Circuit) -> Result<String, BackendError> {
        let binary_path = self.assert_binary_exists()?;

        let temp_directory = tempdir().expect("could not create a temporary directory");
        let temp_directory_path = temp_directory.path().to_path_buf();

        // Create a temporary file for the circuit
        let bytecode_path = temp_directory_path.join("circuit").with_extension("bytecode");
        let serialized_circuit = serialize_circuit(circuit);
        write_to_file(&serialized_circuit, &bytecode_path);

        // Create the verification key and write it to the specified path
        let vk_path = temp_directory_path.join("vk");

        WriteVkCommand {
            crs_path: self.crs_directory(),
            is_recursive: false,
            bytecode_path,
            vk_path_output: vk_path.clone(),
        }
        .run(binary_path)?;

        ContractCommand { crs_path: self.crs_directory(), vk_path }.run(binary_path)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use acvm::acir::{
        circuit::{Circuit, Opcode, PublicInputs},
        native_types::{Expression, Witness},
    };

    use crate::{get_mock_backend, BackendError};

    #[test]
    fn test_smart_contract() -> Result<(), BackendError> {
        let expression = &(Witness(1) + Witness(2)) - &Expression::from(Witness(3));
        let constraint = Opcode::Arithmetic(expression);

        let circuit = Circuit {
            current_witness_index: 4,
            opcodes: vec![constraint],
            private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        };

        let contract = get_mock_backend()?.eth_contract(&circuit)?;

        assert!(contract.contains("contract VerifierContract"));

        Ok(())
    }
}
