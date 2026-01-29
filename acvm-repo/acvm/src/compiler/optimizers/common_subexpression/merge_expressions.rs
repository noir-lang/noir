use std::collections::{BTreeMap, BTreeSet, HashMap};

use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode,
        brillig::{BrilligInputs, BrilligOutputs},
        opcodes::BlockId,
    },
    native_types::{Expression, Witness},
};

use crate::compiler::{CircuitSimulator, optimizers::GeneralOptimizer};

pub(crate) struct MergeExpressionsOptimizer<F: AcirField> {
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
    /// only used in two AssertZero opcodes. It then merges the opcode which produces the
    /// intermediate variable into the second one that uses it
    ///
    /// The first pass maps witnesses to the indices of the opcodes using them.
    /// Public inputs are not considered because they cannot be simplified.
    /// Witnesses used by MemoryInit opcodes are put in a separate map and marked as used by a Brillig call
    /// if the memory block is an input to the call.
    ///
    /// The second pass looks for AssertZero opcodes having a witness which is only used by another arithmetic opcode.
    /// In that case, the opcode with the smallest index is merged into the other one via Gaussian elimination.
    /// For instance, if we have 'w1' used only by these two opcodes,
    /// `5*w2*w3` and `w1`:
    /// w2*w3 + 2*w2 + w1 + w3 = 0   // This opcode 'defines' the variable w1
    /// 2*w3*w4 + w1 + w4 = 0        // which is only used here
    ///
    /// For w1 we can say:
    /// w1 = -1/2*w2*w3 - w2 - 1/2*w3
    ///
    /// Then we will remove the first one and modify the second one like this:
    /// 2*w3*w4 + w4 - w2 - 1/2*w3 - 1/2*w2*w3 = 0
    ///
    /// Pre-condition:
    /// - This pass is relevant for backends that can handle unlimited width and
    ///   Plonk-ish backends. Although they have a limited width, they can potentially
    ///   handle expressions with large linear combinations using 'big-add' gates.
    /// - The CSAT pass should have been run prior to this one.
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

        let mut used_witnesses: BTreeMap<Witness, BTreeSet<usize>> = BTreeMap::new();
        for (i, opcode) in circuit.opcodes.iter().enumerate() {
            let witnesses = self.witness_inputs(opcode);
            if let Opcode::MemoryInit { block_id, .. } = opcode {
                self.resolved_blocks.insert(*block_id, witnesses.clone());
            }
            for w in witnesses {
                // We do not simplify circuit inputs and outputs
                if !circuit_io.contains(&w) {
                    used_witnesses.entry(w).or_default().insert(i);
                }
            }
        }

        // For each opcode, try to get a target opcode to merge with
        for (op1, opcode) in circuit.opcodes.iter().enumerate() {
            if !matches!(opcode, Opcode::AssertZero(_)) {
                continue;
            }
            if let Some(opcode) = self.get_opcode(op1, circuit) {
                let input_witnesses = self.witness_inputs(&opcode);
                for w in input_witnesses {
                    let Some(gates_using_w) = used_witnesses.get(&w) else {
                        continue;
                    };
                    // We only consider witness which are used in exactly two arithmetic gates
                    if gates_using_w.len() == 2 {
                        let first = *gates_using_w.first().expect("gates_using_w.len == 2");
                        let second = *gates_using_w.last().expect("gates_using_w.len == 2");
                        let op2 = if second == op1 {
                            first
                        } else {
                            // sanity check
                            assert!(op1 == first);
                            second
                        };

                        // Merge the opcode with smaller index into the other one
                        // by updating modified_gates/deleted_gates/used_witnesses
                        // returns false if it could not merge them
                        if op1 != op2 {
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
                                    // Update the 'used_witnesses' map to account for the merge.
                                    let witness_list = CircuitSimulator::expr_witness(&expr_use);
                                    let witness_list = witness_list
                                        .chain(CircuitSimulator::expr_witness(&expr_define));

                                    for w2 in witness_list {
                                        if !circuit_io.contains(&w2) {
                                            used_witnesses.entry(w2).and_modify(|v| {
                                                v.insert(target);
                                                v.remove(&source);
                                            });
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
            }
        }

        // Construct the new circuit from modified/deleted gates
        let mut new_circuit = Vec::new();
        let mut new_acir_opcode_positions = Vec::new();

        for (i, opcode_position) in acir_opcode_positions.iter().enumerate() {
            if let Some(opcode) = self.get_opcode(i, circuit) {
                new_circuit.push(opcode);
                new_acir_opcode_positions.push(*opcode_position);
            }
        }
        (new_circuit, new_acir_opcode_positions)
    }

    fn for_each_brillig_input_witness(&self, input: &BrilligInputs<F>, mut f: impl FnMut(Witness)) {
        match input {
            BrilligInputs::Single(expr) => {
                for witness in CircuitSimulator::expr_witness(expr) {
                    f(witness);
                }
            }
            BrilligInputs::Array(exprs) => {
                for expr in exprs {
                    for witness in CircuitSimulator::expr_witness(expr) {
                        f(witness);
                    }
                }
            }
            BrilligInputs::MemoryArray(block_id) => {
                for witness in self.resolved_blocks.get(block_id).expect("Unknown block id") {
                    f(*witness);
                }
            }
        }
    }

    fn for_each_brillig_output_witness(&self, output: &BrilligOutputs, mut f: impl FnMut(Witness)) {
        match output {
            BrilligOutputs::Simple(witness) => f(*witness),
            BrilligOutputs::Array(witnesses) => {
                for witness in witnesses {
                    f(*witness);
                }
            }
        }
    }

    // Returns the input witnesses used by the opcode
    fn witness_inputs(&self, opcode: &Opcode<F>) -> BTreeSet<Witness> {
        match opcode {
            Opcode::AssertZero(expr) => CircuitSimulator::expr_witness(expr).collect(),
            Opcode::BlackBoxFuncCall(bb_func) => {
                let mut witnesses = bb_func.get_input_witnesses();
                witnesses.extend(bb_func.get_outputs_vec());
                if let Some(w) = bb_func.get_predicate() {
                    witnesses.insert(w);
                }
                witnesses
            }
            Opcode::MemoryOp { block_id: _, op } => CircuitSimulator::expr_witness(&op.operation)
                .chain(CircuitSimulator::expr_witness(&op.index))
                .chain(CircuitSimulator::expr_witness(&op.value))
                .collect(),

            Opcode::MemoryInit { block_id: _, init, block_type: _ } => {
                init.iter().cloned().collect()
            }
            Opcode::BrilligCall { inputs, outputs, predicate, .. } => {
                let mut witnesses = BTreeSet::new();
                for i in inputs {
                    self.for_each_brillig_input_witness(i, |witness| {
                        witnesses.insert(witness);
                    });
                }
                witnesses.extend(CircuitSimulator::expr_witness(predicate));
                for i in outputs {
                    self.for_each_brillig_output_witness(i, |witness| {
                        witnesses.insert(witness);
                    });
                }
                witnesses
            }
            Opcode::Call { id: _, inputs, outputs, predicate } => {
                let mut witnesses: BTreeSet<Witness> = inputs.iter().copied().collect();
                witnesses.extend(outputs);
                witnesses.extend(CircuitSimulator::expr_witness(predicate));
                witnesses
            }
        }
    }

    // Merge 'expr' into 'target' via Gaussian elimination on 'w'
    // Returns None if the expressions cannot be merged
    fn merge_expression(
        target: &Expression<F>,
        expr: &Expression<F>,
        witness: Witness,
    ) -> Option<Expression<F>> {
        // Check that the witness is not part of multiplication terms
        for m in &target.mul_terms {
            if m.1 == witness || m.2 == witness {
                return None;
            }
        }
        for m in &expr.mul_terms {
            if m.1 == witness || m.2 == witness {
                return None;
            }
        }

        for k in &target.linear_combinations {
            if k.1 == witness {
                for i in &expr.linear_combinations {
                    if i.1 == witness {
                        assert!(
                            i.0 != F::zero(),
                            "merge_expression: attempting to divide k.0 by F::zero"
                        );
                        let expr = target.add_mul(-(k.0 / i.0), expr);
                        let expr = GeneralOptimizer::optimize(expr);
                        return Some(expr);
                    }
                }
            }
        }
        None
    }

    /// Returns the 'updated' opcode at the given index in the circuit
    /// The modifications to the circuits are stored with 'deleted_gates' and 'modified_gates'
    /// These structures are used to give the 'updated' opcode.
    /// For instance, if the opcode has been deleted inside 'deleted_gates', then it returns None.
    fn get_opcode(&self, index: usize, circuit: &Circuit<F>) -> Option<Opcode<F>> {
        if self.deleted_gates.contains(&index) {
            return None;
        }
        self.modified_gates.get(&index).or(circuit.opcodes.get(index)).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_circuit_snapshot,
        compiler::{
            CircuitSimulator,
            optimizers::common_subexpression::merge_expressions::MergeExpressionsOptimizer,
        },
    };
    use acir::{
        AcirField, FieldElement,
        circuit::Circuit,
        native_types::{Expression, Witness},
    };

    fn merge_expressions(circuit: Circuit<FieldElement>) -> Circuit<FieldElement> {
        assert!(CircuitSimulator::check_circuit(&circuit).is_none());
        let mut merge_optimizer = MergeExpressionsOptimizer::new();
        let acir_opcode_positions = vec![0; 20];
        let (opcodes, _) =
            merge_optimizer.eliminate_intermediate_variable(&circuit, acir_opcode_positions);
        let mut optimized_circuit = circuit;
        optimized_circuit.opcodes = opcodes;

        // check that the circuit is still valid after optimization
        assert!(CircuitSimulator::check_circuit(&optimized_circuit).is_none());
        optimized_circuit
    }

    #[test]
    fn merges_expressions() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: [w2]
        ASSERT 2*w1 = w0 + 5
        ASSERT w2 = 4*w1 + 4
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = merge_expressions(circuit.clone());
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0]
        public parameters: []
        return values: [w2]
        ASSERT w2 = 2*w0 + 14
        ");
    }

    #[test]
    fn does_not_eliminate_witnesses_returned_from_brillig() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: []
        BRILLIG CALL func: 0, predicate: 1, inputs: [], outputs: [w1]
        ASSERT 2*w0 + 3*w1 + w2 + 1 = 0
        ASSERT 2*w0 + 2*w1 + w5 + 1 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = merge_expressions(circuit.clone());
        assert_eq!(circuit, optimized_circuit);
    }

    #[test]
    fn does_not_eliminate_witnesses_returned_from_circuit() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: [w1, w2]
        ASSERT -w0*w0 + w1 = 0
        ASSERT -w1 + w2 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = merge_expressions(circuit.clone());
        assert_eq!(circuit, optimized_circuit);
    }

    #[test]
    fn does_not_attempt_to_merge_into_previous_opcodes() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT w0*w0 - w4 = 0
        ASSERT w0*w1 + w5 = 0
        ASSERT -w2 + w4 + w5 = 0
        ASSERT w2 - w3 + w4 + w5 = 0
        BLACKBOX::RANGE input: w3, bits: 32
        ";
        let circuit = Circuit::from_str(src).unwrap();

        let optimized_circuit = merge_expressions(circuit);
        assert_circuit_snapshot!(optimized_circuit, @r"
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        ASSERT w5 = -w0*w1
        ASSERT w3 = 2*w0*w0 + 2*w5
        BLACKBOX::RANGE input: w3, bits: 32
        ");
    }

    #[test]
    fn takes_blackbox_opcode_outputs_into_account() {
        // Regression test for https://github.com/noir-lang/noir/issues/6527
        // Previously we would not track the usage of witness 4 in the output of the blackbox function.
        // We would then merge the final two opcodes losing the check that the brillig call must match
        // with `w0 ^ w1`.
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: [w2]
        BRILLIG CALL func: 0, predicate: 1, inputs: [], outputs: [w3]
        BLACKBOX::AND lhs: w0, rhs: w1, output: w4, bits: 8
        ASSERT w3 - w4 = 0
        ASSERT -w2 + w4 = 0
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = merge_expressions(circuit.clone());
        assert_eq!(circuit, optimized_circuit);
    }

    #[test]
    #[should_panic(expected = "merge_expression: attempting to divide k.0 by F::zero")]
    fn merge_expression_on_zero_linear_combination_panics() {
        let opcode_a = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::one(), Witness(0))],
            q_c: FieldElement::zero(),
        };
        let opcode_b = Expression {
            mul_terms: vec![],
            linear_combinations: vec![(FieldElement::zero(), Witness(0))],
            q_c: FieldElement::zero(),
        };
        assert_eq!(
            MergeExpressionsOptimizer::merge_expression(&opcode_a, &opcode_b, Witness(0),),
            Some(opcode_a)
        );
    }

    #[test]
    fn does_not_eliminate_witnesses_used_in_brillig_call_predicates() {
        let src = "
        private parameters: [w2]
        public parameters: [w0, w1]
        return values: [w3]
        BLACKBOX::RANGE input: w0, bits: 1
        BLACKBOX::RANGE input: w1, bits: 1
        BLACKBOX::RANGE input: w2, bits: 1
        ASSERT w4 = w0*w1
        ASSERT w5 = -w2 + 1
        BRILLIG CALL func: 0, predicate: w4*w5, inputs: [w2], outputs: [w6]
        ASSERT w3 = -w5 + 1
        ";
        let circuit = Circuit::from_str(src).unwrap();
        let optimized_circuit = merge_expressions(circuit.clone());
        assert_eq!(circuit, optimized_circuit);
    }
}
