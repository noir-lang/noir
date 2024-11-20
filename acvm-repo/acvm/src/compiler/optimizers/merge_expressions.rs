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

pub(crate) struct MergeExpressionsOptimizer {
    resolved_blocks: HashMap<BlockId, BTreeSet<Witness>>,
}

impl MergeExpressionsOptimizer {
    pub(crate) fn new() -> Self {
        MergeExpressionsOptimizer { resolved_blocks: HashMap::new() }
    }
    /// This pass analyzes the circuit and identifies intermediate variables that are
    /// only used in two gates. It then merges the gate that produces the
    /// intermediate variable into the second one that uses it
    /// Note: This pass is only relevant for backends that can handle unlimited width
    pub(crate) fn eliminate_intermediate_variable<F: AcirField>(
        &mut self,
        circuit: &Circuit<F>,
        acir_opcode_positions: Vec<usize>,
    ) -> (Vec<Opcode<F>>, Vec<usize>) {
        // Keep track, for each witness, of the gates that use it
        let circuit_inputs = circuit.circuit_arguments();
        self.resolved_blocks = HashMap::new();
        let mut used_witness: BTreeMap<Witness, BTreeSet<usize>> = BTreeMap::new();
        for (i, opcode) in circuit.opcodes.iter().enumerate() {
            let witnesses = self.witness_inputs(opcode);
            if let Opcode::MemoryInit { block_id, .. } = opcode {
                self.resolved_blocks.insert(*block_id, witnesses.clone());
            }
            for w in witnesses {
                // We do not simplify circuit inputs
                if !circuit_inputs.contains(&w) {
                    used_witness.entry(w).or_default().insert(i);
                }
            }
        }

        let mut modified_gates: HashMap<usize, Opcode<F>> = HashMap::new();
        let mut new_circuit = Vec::new();
        let mut new_acir_opcode_positions = Vec::new();
        // For each opcode, try to get a target opcode to merge with
        for (i, opcode) in circuit.opcodes.iter().enumerate() {
            if !matches!(opcode, Opcode::AssertZero(_)) {
                new_circuit.push(opcode.clone());
                new_acir_opcode_positions.push(acir_opcode_positions[i]);
                continue;
            }
            let opcode = modified_gates.get(&i).unwrap_or(opcode).clone();
            let mut to_keep = true;
            let input_witnesses = self.witness_inputs(&opcode);
            for w in input_witnesses.clone() {
                let empty_gates = BTreeSet::new();
                let gates_using_w = used_witness.get(&w).unwrap_or(&empty_gates);
                // We only consider witness which are used in exactly two arithmetic gates
                if gates_using_w.len() == 2 {
                    let gates_using_w: Vec<_> = gates_using_w.iter().collect();
                    let mut b = *gates_using_w[1];
                    if b == i {
                        b = *gates_using_w[0];
                    } else {
                        // sanity check
                        assert!(i == *gates_using_w[0]);
                    }
                    let second_gate = modified_gates.get(&b).unwrap_or(&circuit.opcodes[b]).clone();
                    if let (Opcode::AssertZero(expr_define), Opcode::AssertZero(expr_use)) =
                        (opcode.clone(), second_gate)
                    {
                        // We cannot merge an expression into an earlier opcode, because this
                        // would break the 'execution ordering' of the opcodes
                        // This case can happen because a previous merge would change an opcode
                        // and eliminate a witness from it, giving new opportunities for this
                        // witness to be used in only two expressions
                        // TODO: the missed optimization for the i>b case can be handled by
                        // - doing this pass again until there is no change, or
                        // - merging 'b' into 'i' instead
                        if i < b {
                            if let Some(expr) = Self::merge(&expr_use, &expr_define, w) {
                                modified_gates.insert(b, Opcode::AssertZero(expr));
                                to_keep = false;
                                // Update the 'used_witness' map to account for the merge.
                                for w2 in CircuitSimulator::expr_wit(&expr_define) {
                                    if !circuit_inputs.contains(&w2) {
                                        let mut v = used_witness[&w2].clone();
                                        v.insert(b);
                                        v.remove(&i);
                                        used_witness.insert(w2, v);
                                    }
                                }
                                // We need to stop here and continue with the next opcode
                                // because the merge invalidates the current opcode.
                                break;
                            }
                        }
                    }
                }
            }

            if to_keep {
                if modified_gates.contains_key(&i) {
                    new_circuit.push(modified_gates[&i].clone());
                } else {
                    new_circuit.push(opcode.clone());
                }
                new_acir_opcode_positions.push(acir_opcode_positions[i]);
            }
        }
        (new_circuit, new_acir_opcode_positions)
    }

    fn brillig_input_wit<F>(&self, input: &BrilligInputs<F>) -> BTreeSet<Witness> {
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
    fn witness_inputs<F: AcirField>(&self, opcode: &Opcode<F>) -> BTreeSet<Witness> {
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
    fn merge<F: AcirField>(
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
            Opcode::BlackBoxFuncCall(acir::circuit::opcodes::BlackBoxFuncCall::RANGE {
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
