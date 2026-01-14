use crate::pwg::{
    ErrorLocation, OpcodeNotSolvable, OpcodeResolutionError, ResolvedAssertionPayload,
    arithmetic::ExpressionSolver,
    blackbox::{
        self,
        embedded_curve_ops::{execute_embedded_curve_add, execute_multi_scalar_mul},
        hash::get_hash_input,
    },
    get_value, input_to_value,
    memory_op::MemoryOpSolver,
    witness_to_value,
};
use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode, OpcodeLocation,
        opcodes::{BlackBoxFuncCall, BlockId, MemOp},
    },
    native_types::{Witness, WitnessMap},
};
use acvm_blackbox_solver::{
    BlackBoxFunctionSolver, bit_and, bit_xor, blake2s, blake3, keccakf1600,
};
use std::collections::HashMap;

fn unsatisfied_constraint<F>(opcode_index: usize, message: String) -> OpcodeResolutionError<F> {
    OpcodeResolutionError::UnsatisfiedConstrain {
        opcode_location: ErrorLocation::Resolved(OpcodeLocation::Acir(opcode_index)),
        payload: Some(ResolvedAssertionPayload::String(message)),
    }
}

fn witness_value<F: AcirField>(
    w: &Witness,
    witness_map: &WitnessMap<F>,
) -> Result<F, OpcodeResolutionError<F>> {
    Ok(*witness_map.get(w).ok_or(OpcodeNotSolvable::MissingAssignment(w.witness_index()))?)
}

pub fn validate_witness<F: AcirField>(
    backend: &impl BlackBoxFunctionSolver<F>,
    witness_map: WitnessMap<F>,
    circuit: &Circuit<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    let mut block_solvers: HashMap<BlockId, MemoryOpSolver<F>> = HashMap::new();

    for (opcode_index, opcode) in circuit.opcodes.iter().enumerate() {
        match opcode {
            Opcode::AssertZero(expression) => {
                let result = &ExpressionSolver::evaluate(expression, &witness_map);
                if !result.is_zero() {
                    return Err(unsatisfied_constraint(
                        opcode_index,
                        format!("Invalid witness assignment: {expression}"),
                    ));
                }
            }
            Opcode::BlackBoxFuncCall(black_box_func_call) => {
                match black_box_func_call {
                    BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs } => {
                        let ciphertext = blackbox::aes128::execute_aes128_encryption_opcode(
                            &witness_map,
                            inputs,
                            iv,
                            key,
                        )?;
                        assert_eq!(outputs.len(), ciphertext.len());
                        for (output_witness, value) in outputs.iter().zip(ciphertext.into_iter()) {
                            let witness_value = witness_value(output_witness, &witness_map)?;
                            let output_value = F::from(u128::from(value));
                            if witness_value != output_value {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "AES128 opcode violation: expected {output_value} but found {witness_value} for output witness {output_witness}",
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::AND { lhs, rhs, num_bits, output } => {
                        let lhs_value = input_to_value(&witness_map, *lhs)?;
                        let rhs_value = input_to_value(&witness_map, *rhs)?;
                        let and_result = bit_and(lhs_value, rhs_value, *num_bits);
                        let output_value = witness_map
                            .get(output)
                            .ok_or(OpcodeNotSolvable::MissingAssignment(output.0))?;
                        if and_result != *output_value {
                            return Err(unsatisfied_constraint(
                                opcode_index,
                                format!(
                                    "AND opcode violation: {lhs_value} AND {rhs_value} != {output_value} for {num_bits} bits"
                                ),
                            ));
                        }
                    }
                    BlackBoxFuncCall::XOR { lhs, rhs, num_bits, output } => {
                        let lhs_value = input_to_value(&witness_map, *lhs)?;
                        let rhs_value = input_to_value(&witness_map, *rhs)?;
                        let xor_result = bit_xor(lhs_value, rhs_value, *num_bits);
                        let output_value = witness_map
                            .get(output)
                            .ok_or(OpcodeNotSolvable::MissingAssignment(output.0))?;
                        if xor_result != *output_value {
                            return Err(unsatisfied_constraint(
                                opcode_index,
                                format!(
                                    "XOR opcode violation: {lhs_value} XOR {rhs_value} != {output_value} for {num_bits} bits"
                                ),
                            ));
                        }
                    }
                    BlackBoxFuncCall::RANGE { input, num_bits } => {
                        let value = input_to_value(&witness_map, *input)?;
                        if value.num_bits() > *num_bits {
                            return Err(unsatisfied_constraint(
                                opcode_index,
                                format!(
                                    "RANGE opcode violation: value {value} does not fit in {num_bits} bits"
                                ),
                            ));
                        }
                    }
                    BlackBoxFuncCall::Blake2s { inputs, outputs } => {
                        let message_input = get_hash_input(&witness_map, inputs, None, 8)?;
                        let digest: [u8; 32] = blake2s(&message_input)?;
                        for i in 0..32 {
                            let output_witness = &outputs[i];
                            let witness_value = witness_map
                                .get(output_witness)
                                .ok_or(OpcodeNotSolvable::MissingAssignment(output_witness.0))?;
                            if *witness_value != F::from_be_bytes_reduce(&[digest[i]]) {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "BLAKE2s opcode violation: expected {:?} but found {:?} for output witness {:?}",
                                        F::from_be_bytes_reduce(&[digest[i]]),
                                        witness_value,
                                        output_witness
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::Blake3 { inputs, outputs } => {
                        let message_input = get_hash_input(&witness_map, inputs, None, 8)?;
                        let digest: [u8; 32] = blake3(&message_input)?;
                        for i in 0..32 {
                            let output_witness = &outputs[i];
                            let witness_value = witness_value(output_witness, &witness_map)?;
                            if witness_value != F::from_be_bytes_reduce(&[digest[i]]) {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "BLAKE3 opcode violation: expected {:?} but found {:?} for output witness {:?}",
                                        F::from_be_bytes_reduce(&[digest[i]]),
                                        witness_value,
                                        output_witness
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::EcdsaSecp256k1 {
                        public_key_x,
                        public_key_y,
                        signature,
                        hashed_message,
                        predicate,
                        output,
                    } => {
                        let predicate_value = input_to_value(&witness_map, *predicate)?.is_one();
                        if predicate_value {
                            let is_valid = blackbox::signature::ecdsa::execute_ecdsa(
                                &witness_map,
                                public_key_x,
                                public_key_y,
                                signature,
                                hashed_message,
                                predicate,
                                true,
                            )?;
                            let output_value = witness_value(output, &witness_map)?;
                            if output_value != F::from(is_valid) {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "EcdsaSecp256k1 opcode violation: expected {:?} but found {:?} for output witness {:?}",
                                        F::from(is_valid),
                                        output_value,
                                        output
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::EcdsaSecp256r1 {
                        public_key_x,
                        public_key_y,
                        signature,
                        hashed_message,
                        predicate,
                        output,
                    } => {
                        let predicate_value = input_to_value(&witness_map, *predicate)?.is_one();
                        if predicate_value {
                            let is_valid = blackbox::signature::ecdsa::execute_ecdsa(
                                &witness_map,
                                public_key_x,
                                public_key_y,
                                signature,
                                hashed_message,
                                predicate,
                                false,
                            )?;
                            let output_value = witness_value(output, &witness_map)?;
                            if output_value != F::from(is_valid) {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "EcdsaSecp256r1 opcode violation: expected {:?} but found {:?} for output witness {:?}",
                                        F::from(is_valid),
                                        output_value,
                                        output
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::MultiScalarMul { points, scalars, predicate, outputs } => {
                        let predicate_value = input_to_value(&witness_map, *predicate)?.is_one();
                        if predicate_value {
                            let (res_x, res_y, res_infinite) = execute_multi_scalar_mul(
                                backend,
                                &witness_map,
                                points,
                                scalars,
                                *predicate,
                            )?;
                            let output_x_value = witness_value(&outputs.0, &witness_map)?;
                            let output_y_value = witness_value(&outputs.1, &witness_map)?;
                            let output_infinite_value = witness_value(&outputs.2, &witness_map)?;
                            if res_x != output_x_value
                                || res_y != output_y_value
                                || res_infinite != output_infinite_value
                            {
                                //TODO: on check pas les x,y si infinite est true
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "MultiScalarMul opcode violation: expected ({res_x}, {res_y}, {res_infinite}) but found ({output_x_value}, {output_y_value}, {output_infinite_value})"
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, predicate, outputs } => {
                        let predicate_value = input_to_value(&witness_map, *predicate)?.is_one();
                        if predicate_value {
                            let (res_x, res_y, res_infinite) = execute_embedded_curve_add(
                                backend,
                                &witness_map,
                                **input1,
                                **input2,
                                *predicate,
                            )?;
                            let output_x_value = witness_value(&outputs.0, &witness_map)?;
                            let output_y_value = witness_value(&outputs.1, &witness_map)?;
                            let output_infinite_value = witness_value(&outputs.2, &witness_map)?;
                            if res_x != output_x_value
                                || res_y != output_y_value
                                || res_infinite != output_infinite_value
                            {
                                //TODO: on check pas les x,y si infinite est true
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "EmbeddedCurveAdd opcode violation: expected ({res_x}, {res_y}, {res_infinite}) but found ({output_x_value}, {output_y_value}, {output_infinite_value})"
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
                        let mut state = [0; 25];
                        for (it, input) in state.iter_mut().zip(inputs.as_ref()) {
                            let witness_assignment = input_to_value(&witness_map, *input)?;
                            let lane = witness_assignment.try_to_u64();
                            *it = lane.unwrap();
                        }
                        let output_state = keccakf1600(state)?;
                        for (output_witness, value) in outputs.iter().zip(output_state.into_iter())
                        {
                            let witness_value = witness_value(output_witness, &witness_map)?;
                            if witness_value != F::from(u128::from(value)) {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "Keccakf1600 opcode violation: expected {value} but found {witness_value} for output witness {output_witness}",
                                    ),
                                ));
                            }
                        }
                    }
                    // Recursive aggregation is checked outside of ACVM
                    BlackBoxFuncCall::RecursiveAggregation { .. } => (),
                    BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs } => {
                        let state = blackbox::hash::execute_poseidon2_permutation_opcode(
                            backend,
                            &witness_map,
                            inputs,
                        )?;

                        for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
                            let witness_value = witness_map
                                .get(output_witness)
                                .ok_or(OpcodeNotSolvable::MissingAssignment(output_witness.0))?;
                            if *witness_value != value {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "Poseidon2 opcode violation: expected {value} but found {witness_value} for output witness {output_witness}",
                                    ),
                                ));
                            }
                        }
                    }
                    BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
                        let state = blackbox::hash::execute_sha_256_permutation_opcode(
                            &witness_map,
                            inputs,
                            hash_values,
                        )?;

                        for (output_witness, value) in outputs.iter().zip(state.into_iter()) {
                            let witness_value = witness_map
                                .get(output_witness)
                                .ok_or(OpcodeNotSolvable::MissingAssignment(output_witness.0))?;
                            if *witness_value != F::from(u128::from(value)) {
                                return Err(unsatisfied_constraint(
                                    opcode_index,
                                    format!(
                                        "SHA256 Compression opcode violation: expected {:?} but found {:?} for output witness {:?}",
                                        F::from(u128::from(value)),
                                        witness_value,
                                        output_witness
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
            Opcode::MemoryOp { block_id, op } => {
                let solver = block_solvers
                    .get_mut(block_id)
                    .expect("Memory block should have been initialized");
                solver.check_memory_op(op, &witness_map, opcode_index)?;
            }
            Opcode::MemoryInit { block_id, init, .. } => {
                MemoryOpSolver::new(init, &witness_map).map(|solver| {
                    let existing_block_id = block_solvers.insert(*block_id, solver);
                    assert!(existing_block_id.is_none(), "Memory block already initialized");
                })?;
            }
            // BrilligCall is unconstrained
            Opcode::BrilligCall { .. } => (),
            Opcode::Call { id: _, inputs, outputs, predicate } => {
                // Skip validation when predicate is false
                if let Some(pred) = predicate {
                    let pred_value = get_value(pred, &witness_map)?;
                    if pred_value.is_zero() {
                        continue;
                    }
                }

                // Verify input witnesses exist
                for input in inputs {
                    if witness_map.get(input).is_none() {
                        return Err(OpcodeNotSolvable::MissingAssignment(input.0).into());
                    }
                }

                // Verify output witnesses exist (value should have been validated by the called function)
                for output in outputs {
                    if witness_map.get(output).is_none() {
                        return Err(OpcodeNotSolvable::MissingAssignment(output.0).into());
                    }
                }
            }
        }
    }

    Ok(())
}

impl<F: AcirField> MemoryOpSolver<F> {
    pub(crate) fn check_memory_op(
        &mut self,
        op: &MemOp,
        witness_map: &WitnessMap<F>,
        opcode_index: usize,
    ) -> Result<(), OpcodeResolutionError<F>> {
        // Find the memory index associated with this memory operation.
        let index = witness_to_value(witness_map, op.index)?;
        let memory_index = self.index_from_field(*index)?;

        // Calculate the value associated with this memory operation.
        let value = *witness_to_value(witness_map, op.value)?;

        // `operation == 0` for read operation, `operation == 1` for write operation.
        let is_read_operation = !op.operation;

        if is_read_operation {
            // `value = arr[memory_index]`
            let value_in_array = self.read_memory_index(memory_index)?;
            if value != value_in_array {
                return Err(unsatisfied_constraint(
                    opcode_index,
                    format!(
                        "Memory read opcode violation at index {memory_index}: expected {value_in_array} but found {value}",
                    ),
                ));
            }
            Ok(())
        } else {
            // `arr[memory_index] = value`
            self.write_memory_index(memory_index, value)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        AcirField, FieldElement,
        circuit::{
            Circuit, Opcode, PublicInputs,
            opcodes::{BlackBoxFuncCall, FunctionInput},
        },
        native_types::{Expression, Witness, WitnessMap},
    };
    use bn254_blackbox_solver::Bn254BlackBoxSolver;

    use super::validate_witness;

    /// Helper to create a simple circuit with the given opcodes
    fn make_circuit(opcodes: Vec<Opcode<FieldElement>>) -> Circuit<FieldElement> {
        Circuit {
            current_witness_index: 10,
            opcodes,
            private_parameters: Default::default(),
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
            function_name: "test".to_string(),
        }
    }

    #[test]
    fn test_assert_zero_valid() {
        // w1 + w2 - w3 = 0, where w1=2, w2=3, w3=5
        let expr = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (FieldElement::one(), Witness(1)),
                (FieldElement::one(), Witness(2)),
                (-FieldElement::one(), Witness(3)),
            ],
            q_c: FieldElement::zero(),
        };

        let circuit = make_circuit(vec![Opcode::AssertZero(expr)]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
            (Witness(3), FieldElement::from(5u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_assert_zero_invalid() {
        // w1 + w2 - w3 = 0, but w1=2, w2=3, w3=6 (should be 5)
        let expr = Expression {
            mul_terms: vec![],
            linear_combinations: vec![
                (FieldElement::one(), Witness(1)),
                (FieldElement::one(), Witness(2)),
                (-FieldElement::one(), Witness(3)),
            ],
            q_c: FieldElement::zero(),
        };

        let circuit = make_circuit(vec![Opcode::AssertZero(expr)]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(2u128)),
            (Witness(2), FieldElement::from(3u128)),
            (Witness(3), FieldElement::from(6u128)), // Wrong value!
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_assert_zero_with_multiplication() {
        // w1 * w2 - w3 = 0, where w1=3, w2=4, w3=12
        let expr = Expression {
            mul_terms: vec![(FieldElement::one(), Witness(1), Witness(2))],
            linear_combinations: vec![(-FieldElement::one(), Witness(3))],
            q_c: FieldElement::zero(),
        };

        let circuit = make_circuit(vec![Opcode::AssertZero(expr)]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(3u128)),
            (Witness(2), FieldElement::from(4u128)),
            (Witness(3), FieldElement::from(12u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_range_valid() {
        // w1 should fit in 8 bits
        let circuit = make_circuit(vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput::Witness(Witness(1)),
            num_bits: 8,
        })]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(255u128)), // Max 8-bit value
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_range_invalid() {
        // w1 should fit in 8 bits, but 256 doesn't
        let circuit = make_circuit(vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
            input: FunctionInput::Witness(Witness(1)),
            num_bits: 8,
        })]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(256u128)), // Too large for 8 bits
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_and_valid() {
        // w1 AND w2 = w3, where w1=0b1010, w2=0b1100, w3=0b1000
        let circuit = make_circuit(vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput::Witness(Witness(1)),
            rhs: FunctionInput::Witness(Witness(2)),
            num_bits: 8,
            output: Witness(3),
        })]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(0b1010u128)),
            (Witness(2), FieldElement::from(0b1100u128)),
            (Witness(3), FieldElement::from(0b1000u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_and_invalid() {
        // w1 AND w2 = w3, but w3 has wrong value
        let circuit = make_circuit(vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
            lhs: FunctionInput::Witness(Witness(1)),
            rhs: FunctionInput::Witness(Witness(2)),
            num_bits: 8,
            output: Witness(3),
        })]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(0b1010u128)),
            (Witness(2), FieldElement::from(0b1100u128)),
            (Witness(3), FieldElement::from(0b1111u128)), // Wrong!
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_xor_valid() {
        // w1 XOR w2 = w3, where w1=0b1010, w2=0b1100, w3=0b0110
        let circuit = make_circuit(vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
            lhs: FunctionInput::Witness(Witness(1)),
            rhs: FunctionInput::Witness(Witness(2)),
            num_bits: 8,
            output: Witness(3),
        })]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(0b1010u128)),
            (Witness(2), FieldElement::from(0b1100u128)),
            (Witness(3), FieldElement::from(0b0110u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_xor_invalid() {
        // w1 XOR w2 = w3, but w3 has wrong value
        let circuit = make_circuit(vec![Opcode::BlackBoxFuncCall(BlackBoxFuncCall::XOR {
            lhs: FunctionInput::Witness(Witness(1)),
            rhs: FunctionInput::Witness(Witness(2)),
            num_bits: 8,
            output: Witness(3),
        })]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(0b1010u128)),
            (Witness(2), FieldElement::from(0b1100u128)),
            (Witness(3), FieldElement::from(0b1111u128)), // Wrong!
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_missing_witness_in_expression() {
        let expr = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), Witness(1))],
            q_c: FieldElement::zero(),
        };

        let circuit = make_circuit(vec![Opcode::AssertZero(expr)]);

        // Empty witness map - missing w1
        let witness_map = WitnessMap::default();

        let backend = Bn254BlackBoxSolver(false);
        // The expression evaluates with missing witness, but won't be zero
        // so this should fail
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_call_opcode_valid() {
        use acir::circuit::opcodes::AcirFunctionId;

        let circuit = make_circuit(vec![Opcode::Call {
            id: AcirFunctionId(1),
            inputs: vec![Witness(1), Witness(2)],
            outputs: vec![Witness(3)],
            predicate: None,
        }]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(2u128)),
            (Witness(3), FieldElement::from(3u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_call_opcode_missing_input() {
        use acir::circuit::opcodes::AcirFunctionId;

        let circuit = make_circuit(vec![Opcode::Call {
            id: AcirFunctionId(1),
            inputs: vec![Witness(1), Witness(2)],
            outputs: vec![Witness(3)],
            predicate: None,
        }]);

        // Missing Witness(2)
        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(3u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_call_opcode_missing_output() {
        use acir::circuit::opcodes::AcirFunctionId;

        let circuit = make_circuit(vec![Opcode::Call {
            id: AcirFunctionId(1),
            inputs: vec![Witness(1), Witness(2)],
            outputs: vec![Witness(3)],
            predicate: None,
        }]);

        // Missing Witness(3) output
        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(2u128)),
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_call_opcode_skipped_with_zero_predicate() {
        use acir::circuit::opcodes::AcirFunctionId;

        // Predicate is zero, so call should be skipped even with missing witnesses
        let circuit = make_circuit(vec![Opcode::Call {
            id: AcirFunctionId(1),
            inputs: vec![Witness(1), Witness(2)],
            outputs: vec![Witness(3)],
            predicate: Some(Expression {
                mul_terms: vec![],
                linear_combinations: vec![(FieldElement::one(), Witness(4))],
                q_c: FieldElement::zero(),
            }),
        }]);

        // Witness(4) = 0, so predicate is false, call is skipped
        // Missing input/output witnesses should not cause an error
        let witness_map =
            WitnessMap::from(BTreeMap::from_iter([(Witness(4), FieldElement::zero())]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_memory_init_and_read() {
        use acir::circuit::opcodes::{BlockId, MemOp};

        let block_id = BlockId(0);

        let circuit = make_circuit(vec![
            // Initialize memory block with witnesses 1 and 2
            Opcode::MemoryInit {
                block_id,
                init: vec![Witness(1), Witness(2)],
                block_type: acir::circuit::opcodes::BlockType::Memory,
            },
            // Read from index 0 into witness 3
            Opcode::MemoryOp { block_id, op: MemOp::read_at_mem_index(Witness(0), Witness(3)) },
        ]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(0), FieldElement::from(0u128)),
            (Witness(1), FieldElement::from(42u128)),
            (Witness(2), FieldElement::from(43u128)),
            (Witness(3), FieldElement::from(42u128)), // Should match value at index 0
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_memory_read_wrong_value() {
        use acir::circuit::opcodes::{BlockId, MemOp};

        let block_id = BlockId(0);

        let circuit = make_circuit(vec![
            Opcode::MemoryInit {
                block_id,
                init: vec![Witness(1), Witness(2)],
                block_type: acir::circuit::opcodes::BlockType::Memory,
            },
            Opcode::MemoryOp { block_id, op: MemOp::read_at_mem_index(Witness(0), Witness(3)) },
        ]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(0), FieldElement::from(0u128)),
            (Witness(1), FieldElement::from(42u128)),
            (Witness(2), FieldElement::from(43u128)),
            (Witness(3), FieldElement::from(99u128)), // Wrong! Should be 42
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_err());
    }

    #[test]
    fn test_memory_write_then_read() {
        use acir::circuit::opcodes::{BlockId, MemOp};

        let block_id = BlockId(0);

        let circuit = make_circuit(vec![
            // Initialize memory block
            Opcode::MemoryInit {
                block_id,
                init: vec![Witness(1), Witness(2)],
                block_type: acir::circuit::opcodes::BlockType::Memory,
            },
            // Write value from witness 3 to index 0
            Opcode::MemoryOp { block_id, op: MemOp::write_to_mem_index(Witness(0), Witness(3)) },
            // Read from index 0 into witness 4
            Opcode::MemoryOp { block_id, op: MemOp::read_at_mem_index(Witness(0), Witness(4)) },
        ]);

        let witness_map = WitnessMap::from(BTreeMap::from_iter([
            (Witness(0), FieldElement::from(0u128)),
            (Witness(1), FieldElement::from(42u128)), // Initial value at index 0
            (Witness(2), FieldElement::from(43u128)), // Initial value at index 1
            (Witness(3), FieldElement::from(100u128)), // Value to write
            (Witness(4), FieldElement::from(100u128)), // Read should get written value
        ]));

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }

    #[test]
    fn test_brillig_call_with_empty_witness_map() {
        use acir::circuit::brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs};

        // Create a BrilligCall opcode with input and output witnesses
        // Brillig calls are unconstrained and should be skipped during validation,
        // so this should pass even with an empty witness map
        let circuit = make_circuit(vec![Opcode::BrilligCall {
            id: BrilligFunctionId(0),
            inputs: vec![
                BrilligInputs::Single(Witness(1).into()),
                BrilligInputs::Single(Witness(2).into()),
            ],
            outputs: vec![BrilligOutputs::Simple(Witness(3))],
            predicate: None,
        }]);

        // Empty witness map
        let witness_map = WitnessMap::default();

        let backend = Bn254BlackBoxSolver(false);
        assert!(validate_witness(&backend, witness_map, &circuit).is_ok());
    }
}
