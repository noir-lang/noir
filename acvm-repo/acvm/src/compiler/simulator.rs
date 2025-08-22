use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode,
        brillig::{BrilligInputs, BrilligOutputs},
        opcodes::{BlockId, FunctionInput},
    },
    native_types::{Expression, Witness},
};
use std::collections::{BTreeSet, HashSet};

/// Simulate a symbolic solve for a circuit
#[derive(Default)]
pub struct CircuitSimulator {
    /// Track the witnesses that can be solved
    solvable_witness: HashSet<Witness>,

    /// Track whether a [`BlockId`] has been initialized
    initialized_blocks: HashSet<BlockId>,
}

impl CircuitSimulator {
    /// Simulate a symbolic solve for a circuit by keeping track of the witnesses that can be solved.
    /// Returns the index of the opcode that cannot be solved, if any.
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn check_circuit<F: AcirField>(&mut self, circuit: &Circuit<F>) -> Option<usize> {
        let circuit_inputs = circuit.circuit_arguments();
        self.solvable_witness.extend(circuit_inputs.iter());
        for (i, op) in circuit.opcodes.iter().enumerate() {
            if !self.try_solve(op) {
                return Some(i);
            }
        }
        None
    }

    /// Check if the Opcode can be solved, and if yes, add the solved witness to set of solvable witness
    fn try_solve<F: AcirField>(&mut self, opcode: &Opcode<F>) -> bool {
        let mut unresolved = HashSet::new();
        match opcode {
            Opcode::AssertZero(expr) => {
                for (_, w1, w2) in &expr.mul_terms {
                    if !self.solvable_witness.contains(w1) {
                        if !self.solvable_witness.contains(w2) {
                            return false;
                        }
                        unresolved.insert(*w1);
                    }
                    if !self.solvable_witness.contains(w2) && w1 != w2 {
                        unresolved.insert(*w2);
                    }
                }
                for (_, w) in &expr.linear_combinations {
                    if !self.solvable_witness.contains(w) {
                        unresolved.insert(*w);
                    }
                }
                if unresolved.len() == 1 {
                    self.mark_solvable(*unresolved.iter().next().unwrap());
                    return true;
                }
                unresolved.is_empty()
            }
            Opcode::BlackBoxFuncCall(black_box_func_call) => {
                let inputs = black_box_func_call.get_inputs_vec();
                for input in inputs {
                    if !self.can_solve_function_input(&input) {
                        return false;
                    }
                }
                let outputs = black_box_func_call.get_outputs_vec();
                for output in outputs {
                    self.mark_solvable(output);
                }
                true
            }
            Opcode::MemoryOp { block_id, op, predicate } => {
                if !self.initialized_blocks.contains(block_id) {
                    // Memory must be initialized before it can be used.
                    return false;
                }

                if !self.can_solve_expression(&op.index) {
                    return false;
                }
                if let Some(predicate) = predicate {
                    if !self.can_solve_expression(predicate) {
                        return false;
                    }
                }
                if op.operation.is_zero() {
                    let Some(w) = op.value.to_witness() else {
                        return false;
                    };
                    self.mark_solvable(w);
                    true
                } else {
                    self.try_solve(&Opcode::AssertZero(op.value.clone()))
                }
            }
            Opcode::MemoryInit { block_id, init, .. } => {
                for w in init {
                    if !self.solvable_witness.contains(w) {
                        return false;
                    }
                }
                self.initialized_blocks.insert(*block_id)
            }
            Opcode::BrilligCall { id: _, inputs, outputs, predicate } => {
                for input in inputs {
                    if !self.can_solve_brillig_input(input) {
                        return false;
                    }
                }
                if let Some(predicate) = predicate {
                    if !self.can_solve_expression(predicate) {
                        return false;
                    }
                }
                for output in outputs {
                    match output {
                        BrilligOutputs::Simple(w) => self.mark_solvable(*w),
                        BrilligOutputs::Array(arr) => {
                            for w in arr {
                                self.mark_solvable(*w);
                            }
                        }
                    }
                }
                true
            }
            Opcode::Call { id: _, inputs, outputs, predicate } => {
                for w in inputs {
                    if !self.solvable_witness.contains(w) {
                        return false;
                    }
                }
                if let Some(predicate) = predicate {
                    if !self.can_solve_expression(predicate) {
                        return false;
                    }
                }
                for w in outputs {
                    self.mark_solvable(*w);
                }
                true
            }
        }
    }

    /// Adds the witness to set of solvable witness
    pub(crate) fn mark_solvable(&mut self, witness: Witness) {
        self.solvable_witness.insert(witness);
    }

    pub fn can_solve_function_input<F: AcirField>(&self, input: &FunctionInput<F>) -> bool {
        if !input.is_constant() {
            return self.solvable_witness.contains(&input.to_witness());
        }
        true
    }
    fn can_solve_expression<F>(&self, expr: &Expression<F>) -> bool {
        for w in Self::expr_wit(expr) {
            if !self.solvable_witness.contains(&w) {
                return false;
            }
        }
        true
    }
    fn can_solve_brillig_input<F>(&mut self, input: &BrilligInputs<F>) -> bool {
        match input {
            BrilligInputs::Single(expr) => self.can_solve_expression(expr),
            BrilligInputs::Array(exprs) => {
                for expr in exprs {
                    if !self.can_solve_expression(expr) {
                        return false;
                    }
                }
                true
            }

            BrilligInputs::MemoryArray(block_id) => self.initialized_blocks.contains(block_id),
        }
    }

    pub(crate) fn expr_wit<F>(expr: &Expression<F>) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        result.extend(expr.mul_terms.iter().flat_map(|i| vec![i.1, i.2]));
        result.extend(expr.linear_combinations.iter().map(|i| i.1));
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use crate::compiler::CircuitSimulator;
    use acir::{
        FieldElement,
        acir_field::AcirField,
        circuit::{
            Circuit, ExpressionWidth, Opcode, PublicInputs,
            brillig::{BrilligFunctionId, BrilligInputs},
            opcodes::{BlockId, BlockType, MemOp},
        },
        native_types::{Expression, Witness},
    };

    fn test_circuit(
        opcodes: Vec<Opcode<FieldElement>>,
        private_parameters: BTreeSet<Witness>,
        public_parameters: PublicInputs,
    ) -> Circuit<FieldElement> {
        Circuit {
            current_witness_index: 1,
            expression_width: ExpressionWidth::Bounded { width: 4 },
            opcodes,
            private_parameters,
            public_parameters,
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        }
    }

    #[test]
    fn reports_true_for_empty_circuit() {
        let empty_circuit = test_circuit(vec![], BTreeSet::default(), PublicInputs::default());

        assert!(CircuitSimulator::default().check_circuit(&empty_circuit).is_none());
    }

    #[test]
    fn reports_true_for_connected_circuit() {
        let connected_circuit = test_circuit(
            vec![Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::one(), Witness(1)),
                    (-FieldElement::one(), Witness(2)),
                ],
                q_c: FieldElement::zero(),
            })],
            BTreeSet::from([Witness(1)]),
            PublicInputs::default(),
        );

        assert!(CircuitSimulator::default().check_circuit(&connected_circuit).is_none());
    }

    #[test]
    fn reports_false_for_disconnected_circuit() {
        let disconnected_circuit = test_circuit(
            vec![
                Opcode::AssertZero(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(1)),
                        (-FieldElement::one(), Witness(2)),
                    ],
                    q_c: FieldElement::zero(),
                }),
                Opcode::AssertZero(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(3)),
                        (-FieldElement::one(), Witness(4)),
                    ],
                    q_c: FieldElement::zero(),
                }),
            ],
            BTreeSet::from([Witness(1)]),
            PublicInputs::default(),
        );

        assert!(CircuitSimulator::default().check_circuit(&disconnected_circuit).is_some());
    }

    #[test]
    fn reports_true_when_memory_block_passed_to_brillig_and_then_written_to() {
        let circuit = test_circuit(
            vec![
                Opcode::AssertZero(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![(FieldElement::one(), Witness(1))],
                    q_c: FieldElement::zero(),
                }),
                Opcode::MemoryInit {
                    block_id: BlockId(0),
                    init: vec![Witness(0)],
                    block_type: BlockType::Memory,
                },
                Opcode::BrilligCall {
                    id: BrilligFunctionId(0),
                    inputs: vec![BrilligInputs::MemoryArray(BlockId(0))],
                    outputs: Vec::new(),
                    predicate: None,
                },
                Opcode::MemoryOp {
                    block_id: BlockId(0),
                    op: MemOp::read_at_mem_index(
                        Expression {
                            mul_terms: Vec::new(),
                            linear_combinations: Vec::new(),
                            q_c: FieldElement::one(),
                        },
                        Witness(2),
                    ),
                    predicate: None,
                },
            ],
            BTreeSet::from([Witness(1)]),
            PublicInputs::default(),
        );

        assert!(CircuitSimulator::default().check_circuit(&circuit).is_some());
    }

    #[test]
    fn reports_false_when_attempting_to_reinitialize_memory_block() {
        let circuit = test_circuit(
            vec![
                Opcode::MemoryInit {
                    block_id: BlockId(0),
                    init: vec![Witness(0)],
                    block_type: BlockType::Memory,
                },
                Opcode::MemoryInit {
                    block_id: BlockId(0),
                    init: vec![Witness(0)],
                    block_type: BlockType::Memory,
                },
            ],
            BTreeSet::from([Witness(0)]),
            PublicInputs::default(),
        );

        assert!(CircuitSimulator::default().check_circuit(&circuit).is_some());
    }
}
