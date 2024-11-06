use std::collections::{BTreeMap, BTreeSet, HashMap};

use acir::{
    circuit::{brillig::BrilligInputs, directives::Directive, opcodes::BlockId, Circuit, Opcode},
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

    fn compute_used_witness(&mut self, circuit: &Circuit<F>) -> BTreeMap<Witness, BTreeSet<usize>> {
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
        used_witness
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
        let mut used_witness = self.compute_used_witness(circuit);
        let circuit_inputs = circuit.circuit_arguments();

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
                            for w2 in CircuitSimulator::expr_wit(&expr_define) {
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

    fn expr_wit(expr: &Expression<F>) -> BTreeSet<Witness> {
        let mut result = BTreeSet::new();
        result.extend(expr.mul_terms.iter().flat_map(|i| vec![i.1, i.2]));
        result.extend(expr.linear_combinations.iter().map(|i| i.1));
        result
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

    // Returns the input witnesses used by the opcode
    fn witness_inputs(&self, opcode: &Opcode<F>) -> BTreeSet<Witness> {
        let mut witnesses = BTreeSet::new();
        match opcode {
            Opcode::AssertZero(expr) => CircuitSimulator::expr_wit(expr),
            Opcode::BlackBoxFuncCall(bb_func) => bb_func.get_input_witnesses(),
            Opcode::Directive(Directive::ToLeRadix { a, .. }) => CircuitSimulator::expr_wit(a),
            Opcode::MemoryOp { block_id: _, op, predicate } => {
                //index et value, et predicate
                let mut witnesses = BTreeSet::new();
                witnesses.extend(CircuitSimulator::expr_wit(&op.index));
                witnesses.extend(CircuitSimulator::expr_wit(&op.value));
                if let Some(p) = predicate {
                    witnesses.extend(CircuitSimulator::expr_wit(p));
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
                    witnesses.extend(CircuitSimulator::expr_wit(p));
                }
                witnesses
            }
        }
    }

    /// Merge 'expr' into 'target' via Gaussian elimination on 'w'
    /// It supports the case where w is in a target's multiplication term:
    /// - If w is only linear in expr and target, it's just a Gaussian elimination
    /// - If w is in a expr's mul term: merge is not allowed
    /// - If w is in a target's mul term AND expr has no mul term, then we do the Gaussian elimination in target's linear and mul terms
    fn merge(target: &Expression<F>, expr: &Expression<F>, w: Witness) -> Option<Expression<F>> {
        // Check that the witness is not part of expr multiplication terms
        for m in &expr.mul_terms {
            if m.1 == w || m.2 == w {
                return None;
            }
        }
        // w must be in expr linear terms, we use expr to 'solve w'
        let mut solved_w = Expression::zero();
        let w_idx = expr.linear_combinations.iter().position(|x| x.1 == w).unwrap();
        solved_w.linear_combinations.push((F::one(), w));
        solved_w = solved_w.add_mul(-(F::one() / expr.linear_combinations[w_idx].0), expr);

        // Solve w in target multiplication terms
        let mut result: Expression<F> = Expression::zero();
        result.linear_combinations = target.linear_combinations.clone();
        result.q_c = target.q_c;
        for mul in &target.mul_terms {
            if mul.1 == w || mul.2 == w {
                if !expr.mul_terms.is_empty() || mul.1 == mul.2 {
                    // the result will be of degree 3, so this case does not work
                    return None;
                } else {
                    let x = if mul.1 == w { mul.2 } else { mul.1 };

                    // replace w by solved_w in the mul: x * w = x * solved_w
                    let mut solved_mul = Expression::zero();
                    for lin in &solved_w.linear_combinations {
                        solved_mul.mul_terms.push((mul.0 * lin.0, x, lin.1));
                    }
                    solved_mul.linear_combinations.push((solved_w.q_c, x));
                    solved_mul.sort();
                    result = result.add_mul(F::one(), &solved_mul);
                }
            } else {
                result.mul_terms.push(*mul);
                result.sort();
            }
        }

        // Solve w in target linear terms
        let mut w_coefficient = F::zero();
        for k in &result.linear_combinations {
            if k.1 == w {
                w_coefficient = -(k.0 / expr.linear_combinations[w_idx].0);
                break;
            }
        }
        result = result.add_mul(w_coefficient, expr);
        Some(result)
    }

    fn is_free(opcode: Opcode<F>, width: usize) -> Option<Expression<F>> {
        if let Opcode::AssertZero(expr) = opcode {
            if expr.mul_terms.len() <= 1
                && expr.linear_combinations.len() < width
                && !expr.linear_combinations.is_empty()
            {
                return Some(expr);
            }
        }
        None
    }

    fn get_opcode(&self, g: usize, circuit: &Circuit<F>) -> Option<Opcode<F>> {
        if self.deleted_gates.contains(&g) {
            return None;
        }
        Some(self.modified_gates.get(&g).unwrap_or(&circuit.opcodes[g]).clone())
    }

    fn fits(expr: &Expression<F>, width: usize) -> bool {
        if expr.mul_terms.len() > 1 || expr.linear_combinations.len() > width {
            return false;
        }
        if expr.mul_terms.len() == 1 {
            let mut used = 2;
            let mut contains_a = false;
            let mut contains_b = false;
            for lin in &expr.linear_combinations {
                if lin.1 == expr.mul_terms[0].1 {
                    contains_a = true;
                }
                if lin.1 == expr.mul_terms[0].2 {
                    contains_b = true;
                }
                if contains_a && contains_b {
                    break;
                }
            }
            if contains_a {
                used -= 1;
            }
            if (expr.mul_terms[0].1 != expr.mul_terms[0].2) && contains_b {
                used -= 1;
            }
            return expr.linear_combinations.len() + used <= width;
        }
        true
    }

    /// Simplify 'small expression'
    /// Small expressions, even if they are re-used several times in other expressions, can still be simplified.
    /// for example in the case where we have c=ab and the expressions using c do not have a multiplication term: c = ab; a+b+c =0; d+e-c = 0;
    /// Then it can be simplified into two expressions: ab+a+c=0; -ab+d+e=0;
    ///
    /// If we enforce that ALL results satisfies the width, then we are ensured that it will always be an improvement.
    /// However in practice the improvement is very small, so instead we allow for some over-fitting. As a result, optimisation is not guaranteed
    /// and in some cases the result can be worse than the original circuit.
    pub(crate) fn simply_small_expression(
        &mut self,
        circuit: &Circuit<F>,
        acir_opcode_positions: Vec<usize>,
        width: usize,
    ) -> (Vec<Opcode<F>>, Vec<usize>) {
        let mut used_witness = self.compute_used_witness(circuit);

        let mut new_circuit = Vec::new();
        let mut new_acir_opcode_positions = Vec::new();
        self.modified_gates.clear();
        self.deleted_gates.clear();

        // For each opcode, we try to simplify 'small' expressions
        // If it works, we update modified_gates and deleted_gates to store the result of the simplification
        for (i, _) in circuit.opcodes.iter().enumerate() {
            let mut to_keep = true;
            if let Some(opcode) = self.get_opcode(i, circuit) {
                let mut merged = Vec::new();
                let empty_gates = BTreeSet::new();

                // If the current expression current_expr is a 'small' expression
                if let Some(current_expr) = Self::is_free(opcode.clone(), width) {
                    // we try to simplify it doing Gaussian elimination on one of its linear witness
                    // We try each witness until a simplification works.
                    for (_, w) in &current_expr.linear_combinations {
                        let gates_using_w = used_witness.get(w).unwrap_or(&empty_gates).clone();
                        let gates: Vec<&usize> = gates_using_w
                            .iter()
                            .filter(|g| **g != i && !self.deleted_gates.contains(g))
                            .collect();
                        merged.clear();
                        for g in gates {
                            if let Some(g_update) = self.get_opcode(*g, circuit) {
                                if let Opcode::AssertZero(g_expr) = g_update.clone() {
                                    let merged_expr = Self::merge(&g_expr, &current_expr, *w);
                                    if merged_expr.is_none()
                                        || !Self::fits(&merged_expr.clone().unwrap(), width * 2)
                                    {
                                        // Do not simplify if merge failed or the result does not fit
                                        to_keep = true;
                                        break;
                                    }
                                    if *g <= i {
                                        // This case is not supported, as it would break gates execution ordering
                                        to_keep = true;
                                        break;
                                    }
                                    merged.push((*g, merged_expr.clone().unwrap()));
                                } else {
                                    // Do not simplify if w is used in a non-arithmetic opcode
                                    to_keep = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !to_keep {
                        for m in &merged {
                            self.modified_gates.insert(m.0, Opcode::AssertZero(m.1.clone()));
                            // Update the used_witness map
                            let expr_witnesses = Self::expr_wit(&m.1);
                            for w in expr_witnesses {
                                used_witness.entry(w).or_default().insert(m.0);
                            }
                        }
                        self.deleted_gates.insert(i);
                    }
                }
            }
        }
        #[allow(clippy::needless_range_loop)]
        for i in 0..circuit.opcodes.len() {
            if let Some(op) = self.get_opcode(i, circuit) {
                new_circuit.push(op);
                new_acir_opcode_positions.push(acir_opcode_positions[i]);
            }
        }
        (new_circuit, new_acir_opcode_positions)
    }
}
