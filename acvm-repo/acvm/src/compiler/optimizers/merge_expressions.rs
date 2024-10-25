use std::collections::{BTreeMap, BTreeSet, HashMap};

use acir::{
    circuit::{brillig::BrilligInputs, directives::Directive, opcodes::BlockId, Circuit, Opcode},
    native_types::{Expression, Witness},
    AcirField,
};

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
                        if let Some(expr) = Self::merge(&expr_use, &expr_define, w) {
                            // sanity check
                            assert!(i < b);
                            modified_gates.insert(b, Opcode::AssertZero(expr));
                            to_keep = false;
                            // Update the 'used_witness' map to account for the merge.
                            for w2 in Self::expr_wit(&expr_define) {
                                if !circuit_inputs.contains(&w2) {
                                    let mut v = used_witness[&w2].clone();
                                    v.insert(b);
                                    v.remove(&i);
                                    used_witness.insert(w2, v);
                                }
                            }
                            // We need to stop here and continue with the next opcode
                            // because the merge invalidate the current opcode
                            break;
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

    fn expr_wit<F>(expr: &Expression<F>) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        result.extend(expr.mul_terms.iter().flat_map(|i| vec![i.1, i.2]));
        result.extend(expr.linear_combinations.iter().map(|i| i.1));
        result
    }

    fn brillig_input_wit<F>(&self, input: &BrilligInputs<F>) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        match input {
            BrilligInputs::Single(expr) => {
                result.extend(Self::expr_wit(expr));
            }
            BrilligInputs::Array(exprs) => {
                for expr in exprs {
                    result.extend(Self::expr_wit(expr));
                }
            }
            BrilligInputs::MemoryArray(block_id) => {
                let witnesses = self.resolved_blocks.get(block_id).expect("Unknown block id");
                result.extend(witnesses);
            }
        }
        result
    }

    // Returns the input witnesses used by the opcode
    fn witness_inputs<F: AcirField>(&self, opcode: &Opcode<F>) -> BTreeSet<Witness> {
        let mut witnesses = BTreeSet::new();
        match opcode {
            Opcode::AssertZero(expr) => Self::expr_wit(expr),
            Opcode::BlackBoxFuncCall(bb_func) => bb_func.get_input_witnesses(),
            Opcode::Directive(Directive::ToLeRadix { a, .. }) => Self::expr_wit(a),
            Opcode::MemoryOp { block_id: _, op, predicate } => {
                //index et value, et predicate
                let mut witnesses = BTreeSet::new();
                witnesses.extend(Self::expr_wit(&op.index));
                witnesses.extend(Self::expr_wit(&op.value));
                if let Some(p) = predicate {
                    witnesses.extend(Self::expr_wit(p));
                }
                witnesses
            }

            Opcode::MemoryInit { block_id: _, init, block_type: _ } => {
                init.iter().cloned().collect()
            }
            Opcode::BrilligCall { inputs, .. } => {
                for i in inputs {
                    witnesses.extend(self.brillig_input_wit(i));
                }
                witnesses
            }
            Opcode::Call { id: _, inputs, outputs: _, predicate } => {
                for i in inputs {
                    witnesses.insert(*i);
                }
                if let Some(p) = predicate {
                    witnesses.extend(Self::expr_wit(p));
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
