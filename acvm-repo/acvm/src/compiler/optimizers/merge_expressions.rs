use std::collections::{BTreeMap, BTreeSet, HashMap};

use acir::{
    circuit::{
        brillig::{BrilligInputs, BrilligOutputs},
        opcodes::BlockId,
        Circuit, Opcode,
    },
    native_types::{Expression, Witness},
    AcirField,
};

use crate::compiler::CircuitSimulator;

pub(crate) struct MergeExpressionsOptimizer<F> {
    resolved_blocks: HashMap<BlockId, BTreeSet<Witness>>,
    modified_gates: HashMap<usize, Opcode<F>>,
    deleted_gates: BTreeSet<usize>,
}

impl<F: AcirField> MergeExpressionsOptimizer<F> {
    pub(crate) fn new() -> Self {
        MergeExpressionsOptimizer {
            resolved_blocks: HashMap::new(),
            modified_gates: HashMap::new(),
            deleted_gates: BTreeSet::new(),
        }
    }
    /// This pass analyzes the circuit and identifies intermediate variables that are
    /// only used in two gates. It then merges the gate that produces the
    /// intermediate variable into the second one that uses it
    /// Note: This pass is only relevant for backends that can handle unlimited width
    pub(crate) fn eliminate_intermediate_variable(
        &mut self,
        circuit: &Circuit<F>,
        acir_opcode_positions: Vec<usize>,
    ) -> (Vec<Opcode<F>>, Vec<usize>) {
        // Initialization
        self.modified_gates.clear();
        self.deleted_gates.clear();
        self.resolved_blocks.clear();

        // Keep track, for each witness, of the gates that use it
        let circuit_io: BTreeSet<Witness> =
            circuit.circuit_arguments().union(&circuit.public_inputs().0).cloned().collect();

        let mut used_witness: BTreeMap<Witness, BTreeSet<usize>> = BTreeMap::new();
        for (i, opcode) in circuit.opcodes.iter().enumerate() {
            let witnesses = self.witness_inputs(opcode);
            if let Opcode::MemoryInit { block_id, .. } = opcode {
                self.resolved_blocks.insert(*block_id, witnesses.clone());
            }
            for w in witnesses {
                // We do not simplify circuit inputs and outputs
                if !circuit_io.contains(&w) {
                    used_witness.entry(w).or_default().insert(i);
                }
            }
        }

        // For each opcode, try to get a target opcode to merge with
        for (i, opcode) in circuit.opcodes.iter().enumerate() {
            if !matches!(opcode, Opcode::AssertZero(_)) {
                continue;
            }
            if let Some(opcode) = self.get_opcode(i, circuit) {
                let input_witnesses = self.witness_inputs(&opcode);
                for w in input_witnesses {
                    let Some(gates_using_w) = used_witness.get(&w) else {
                        continue;
                    };
                    // We only consider witness which are used in exactly two arithmetic gates
                    if gates_using_w.len() == 2 {
                        let first = *gates_using_w.first().expect("gates_using_w.len == 2");
                        let second = *gates_using_w.last().expect("gates_using_w.len == 2");
                        let b = if second == i {
                            first
                        } else {
                            // sanity check
                            assert!(i == first);
                            second
                        };
                        // Merge the opcode with smaller index into the other one
                        // by updating modified_gates/deleted_gates/used_witness
                        // returns false if it could not merge them
                        let mut merge_opcodes = |op1, op2| -> bool {
                            if op1 == op2 {
                                return false;
                            }
                            let (source, target) = if op1 < op2 { (op1, op2) } else { (op2, op1) };
                            let source_opcode = self.get_opcode(source, circuit);
                            let target_opcode = self.get_opcode(target, circuit);
                            if let (
                                Some(Opcode::AssertZero(expr_use)),
                                Some(Opcode::AssertZero(expr_define)),
                            ) = (target_opcode, source_opcode)
                            {
                                if let Some(expr) =
                                    Self::merge_expression(&expr_use, &expr_define, w)
                                {
                                    self.modified_gates.insert(target, Opcode::AssertZero(expr));
                                    self.deleted_gates.insert(source);
                                    // Update the 'used_witness' map to account for the merge.
                                    let mut witness_list = CircuitSimulator::expr_wit(&expr_use);
                                    witness_list.extend(CircuitSimulator::expr_wit(&expr_define));
                                    for w2 in witness_list {
                                        if !circuit_io.contains(&w2) {
                                            used_witness.entry(w2).and_modify(|v| {
                                                v.insert(target);
                                                v.remove(&source);
                                            });
                                        }
                                    }
                                    return true;
                                }
                            }
                            false
                        };

                        if merge_opcodes(b, i) {
                            // We need to stop here and continue with the next opcode
                            // because the merge invalidates the current opcode.
                            break;
                        }
                    }
                }
            }
        }

        // Construct the new circuit from modified/deleted gates
        let mut new_circuit = Vec::new();
        let mut new_acir_opcode_positions = Vec::new();

        for (i, opcode_position) in acir_opcode_positions.iter().enumerate() {
            if let Some(op) = self.get_opcode(i, circuit) {
                new_circuit.push(op);
                new_acir_opcode_positions.push(*opcode_position);
            }
        }
        (new_circuit, new_acir_opcode_positions)
    }

    fn brillig_input_wit(&self, input: &BrilligInputs<F>) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        match input {
            BrilligInputs::Single(expr) => {
                result.extend(CircuitSimulator::expr_wit(expr));
            }
            BrilligInputs::Array(exprs) => {
                for expr in exprs {
                    result.extend(CircuitSimulator::expr_wit(expr));
                }
            }
            BrilligInputs::MemoryArray(block_id) => {
                let witnesses = self.resolved_blocks.get(block_id).expect("Unknown block id");
                result.extend(witnesses);
            }
        }
        result
    }

    fn brillig_output_wit(&self, output: &BrilligOutputs) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        match output {
            BrilligOutputs::Simple(witness) => {
                result.insert(*witness);
            }
            BrilligOutputs::Array(witnesses) => {
                result.extend(witnesses);
            }
        }
        result
    }

    // Returns the input witnesses used by the opcode
    fn witness_inputs(&self, opcode: &Opcode<F>) -> BTreeSet<Witness> {
        match opcode {
            Opcode::AssertZero(expr) => CircuitSimulator::expr_wit(expr),
            Opcode::BlackBoxFuncCall(bb_func) => {
                let mut witnesses = bb_func.get_input_witnesses();
                witnesses.extend(bb_func.get_outputs_vec());

                witnesses
            }
            Opcode::MemoryOp { block_id: _, op, predicate } => {
                //index et value, et predicate
                let mut witnesses = CircuitSimulator::expr_wit(&op.index);
                witnesses.extend(CircuitSimulator::expr_wit(&op.value));
                if let Some(p) = predicate {
                    witnesses.extend(CircuitSimulator::expr_wit(p));
                }
                witnesses
            }

            Opcode::MemoryInit { block_id: _, init, block_type: _ } => {
                init.iter().cloned().collect()
            }
            Opcode::BrilligCall { inputs, outputs, .. } => {
                let mut witnesses = BTreeSet::new();
                for i in inputs {
                    witnesses.extend(self.brillig_input_wit(i));
                }
                for i in outputs {
                    witnesses.extend(self.brillig_output_wit(i));
                }
                witnesses
            }
            Opcode::Call { id: _, inputs, outputs, predicate } => {
                let mut witnesses: BTreeSet<Witness> = BTreeSet::from_iter(inputs.iter().copied());
                witnesses.extend(outputs);

                if let Some(p) = predicate {
                    witnesses.extend(CircuitSimulator::expr_wit(p));
                }
                witnesses
            }
        }
    }

    // Merge 'expr' into 'target' via Gaussian elimination on 'w'
    // Returns None if the expressions cannot be merged
    fn merge_expression(
        target: &Expression<F>,
        expr: &Expression<F>,
        w: Witness,
    ) -> Option<Expression<F>> {
        // Check that the witness is not part of multiplication terms
        for m in &target.mul_terms {
            if m.1 == w || m.2 == w {
                return None;
            }
        }
        for m in &expr.mul_terms {
            if m.1 == w || m.2 == w {
                return None;
            }
        }

        for k in &target.linear_combinations {
            if k.1 == w {
                for i in &expr.linear_combinations {
                    if i.1 == w {
                        return Some(target.add_mul(-(k.0 / i.0), expr));
                    }
                }
            }
        }
        None
    }

    fn get_opcode(&self, g: usize, circuit: &Circuit<F>) -> Option<Opcode<F>> {
        if self.deleted_gates.contains(&g) {
            return None;
        }
        self.modified_gates.get(&g).or(circuit.opcodes.get(g)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::{optimizers::MergeExpressionsOptimizer, CircuitSimulator};
    use acir::{
        acir_field::AcirField,
        circuit::{
            brillig::{BrilligFunctionId, BrilligOutputs},
            opcodes::{BlackBoxFuncCall, FunctionInput},
            Circuit, ExpressionWidth, Opcode, PublicInputs,
        },
        native_types::{Expression, Witness},
        FieldElement,
    };
    use std::collections::BTreeSet;

    fn check_circuit(circuit: Circuit<FieldElement>) -> Circuit<FieldElement> {
        assert!(CircuitSimulator::default().check_circuit(&circuit));
        let mut merge_optimizer = MergeExpressionsOptimizer::new();
        let acir_opcode_positions = vec![0; 20];
        let (opcodes, _) =
            merge_optimizer.eliminate_intermediate_variable(&circuit, acir_opcode_positions);
        let mut optimized_circuit = circuit;
        optimized_circuit.opcodes = opcodes;
        // check that the circuit is still valid after optimization
        assert!(CircuitSimulator::default().check_circuit(&optimized_circuit));
        optimized_circuit
    }

    #[test]
    fn does_not_eliminate_witnesses_returned_from_brillig() {
        let opcodes = vec![
            Opcode::BrilligCall {
                id: BrilligFunctionId::default(),
                inputs: Vec::new(),
                outputs: vec![BrilligOutputs::Simple(Witness(1))],
                predicate: None,
            },
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::from(2_u128), Witness(0)),
                    (FieldElement::from(3_u128), Witness(1)),
                    (FieldElement::from(1_u128), Witness(2)),
                ],
                q_c: FieldElement::one(),
            }),
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::from(2_u128), Witness(0)),
                    (FieldElement::from(2_u128), Witness(1)),
                    (FieldElement::from(1_u128), Witness(5)),
                ],
                q_c: FieldElement::one(),
            }),
        ];

        let mut private_parameters = BTreeSet::new();
        private_parameters.insert(Witness(0));

        let circuit = Circuit {
            current_witness_index: 1,
            expression_width: ExpressionWidth::Bounded { width: 4 },
            opcodes,
            private_parameters,
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        };
        check_circuit(circuit);
    }

    #[test]
    fn does_not_eliminate_witnesses_returned_from_circuit() {
        let opcodes = vec![
            Opcode::AssertZero(Expression {
                mul_terms: vec![(FieldElement::from(-1i128), Witness(0), Witness(0))],
                linear_combinations: vec![(FieldElement::from(1i128), Witness(1))],
                q_c: FieldElement::zero(),
            }),
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::from(-1i128), Witness(1)),
                    (FieldElement::from(1i128), Witness(2)),
                ],
                q_c: FieldElement::zero(),
            }),
        ];
        // Witness(1) could be eliminated because it's only used by 2 opcodes.

        let mut private_parameters = BTreeSet::new();
        private_parameters.insert(Witness(0));

        let mut return_values = BTreeSet::new();
        return_values.insert(Witness(1));
        return_values.insert(Witness(2));

        let circuit = Circuit {
            current_witness_index: 2,
            expression_width: ExpressionWidth::Bounded { width: 4 },
            opcodes,
            private_parameters,
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs(return_values),
            assert_messages: Default::default(),
        };

        let mut merge_optimizer = MergeExpressionsOptimizer::new();
        let acir_opcode_positions = vec![0; 20];
        let (opcodes, _) =
            merge_optimizer.eliminate_intermediate_variable(&circuit, acir_opcode_positions);

        assert_eq!(opcodes.len(), 2);
    }

    #[test]
    fn does_not_attempt_to_merge_into_previous_opcodes() {
        let opcodes = vec![
            Opcode::AssertZero(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(0), Witness(0))],
                linear_combinations: vec![(-FieldElement::one(), Witness(4))],
                q_c: FieldElement::zero(),
            }),
            Opcode::AssertZero(Expression {
                mul_terms: vec![(FieldElement::one(), Witness(0), Witness(1))],
                linear_combinations: vec![(FieldElement::one(), Witness(5))],
                q_c: FieldElement::zero(),
            }),
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (-FieldElement::one(), Witness(2)),
                    (FieldElement::one(), Witness(4)),
                    (FieldElement::one(), Witness(5)),
                ],
                q_c: FieldElement::zero(),
            }),
            Opcode::AssertZero(Expression {
                mul_terms: Vec::new(),
                linear_combinations: vec![
                    (FieldElement::one(), Witness(2)),
                    (-FieldElement::one(), Witness(3)),
                    (FieldElement::one(), Witness(4)),
                    (FieldElement::one(), Witness(5)),
                ],
                q_c: FieldElement::zero(),
            }),
            Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                input: FunctionInput::witness(Witness(3), 32),
            }),
        ];

        let mut private_parameters = BTreeSet::new();
        private_parameters.insert(Witness(0));
        private_parameters.insert(Witness(1));
        let circuit = Circuit {
            current_witness_index: 5,
            expression_width: ExpressionWidth::Bounded { width: 4 },
            opcodes,
            private_parameters,
            public_parameters: PublicInputs::default(),
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        };
        check_circuit(circuit);
    }

    #[test]
    fn takes_blackbox_opcode_outputs_into_account() {
        // Regression test for https://github.com/noir-lang/noir/issues/6527
        // Previously we would not track the usage of witness 4 in the output of the blackbox function.
        // We would then merge the final two opcodes losing the check that the brillig call must match
        // with `_0 ^ _1`.

        let circuit: Circuit<FieldElement> = Circuit {
            current_witness_index: 7,
            opcodes: vec![
                Opcode::BrilligCall {
                    id: BrilligFunctionId(0),
                    inputs: Vec::new(),
                    outputs: vec![BrilligOutputs::Simple(Witness(3))],
                    predicate: None,
                },
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::AND {
                    lhs: FunctionInput::witness(Witness(0), 8),
                    rhs: FunctionInput::witness(Witness(1), 8),
                    output: Witness(4),
                }),
                Opcode::AssertZero(Expression {
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(3)),
                        (-FieldElement::one(), Witness(4)),
                    ],
                    ..Default::default()
                }),
                Opcode::AssertZero(Expression {
                    linear_combinations: vec![
                        (-FieldElement::one(), Witness(2)),
                        (FieldElement::one(), Witness(4)),
                    ],
                    ..Default::default()
                }),
            ],
            expression_width: ExpressionWidth::Bounded { width: 4 },
            private_parameters: BTreeSet::from([Witness(0), Witness(1)]),
            return_values: PublicInputs(BTreeSet::from([Witness(2)])),
            ..Default::default()
        };

        let new_circuit = check_circuit(circuit.clone());
        assert_eq!(circuit, new_circuit);
    }
}
