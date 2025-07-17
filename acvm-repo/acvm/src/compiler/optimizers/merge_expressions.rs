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

use crate::compiler::CircuitSimulator;

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

    fn compute_used_witness(&mut self, circuit: &Circuit<F>) -> BTreeMap<Witness, BTreeSet<usize>> {
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
        used_witness
    }

    /// This pass analyzes the circuit and identifies intermediate variables that are
    /// only used in two arithmetic opcodes. It then merges the opcode which produces the
    /// intermediate variable into the second one that uses it
    /// Note: This pass is only relevant for backends that can handle unlimited width
    ///
    /// The first pass maps witnesses to the index of the opcodes using them.
    /// Public inputs are not considered because they cannot be simplified.
    /// Witnesses used by MemoryInit opcodes are put in a separate map and marked as used by a Brillig call
    /// if the memory block is an input to the call.
    ///
    /// The second pass looks for arithmetic opcodes having a witness which is only used by another arithmetic opcode.
    /// In that case, the opcode with the smallest index is merged into the other one via Gaussian elimination.
    /// For instance, if we have '_1' used only by these two opcodes,
    /// where `_{value}` refers to a witness and `{value}` refers to a constant:
    /// [(1, _2,_3), (2, _2), (2, _1), (1, _3)]
    /// [(2, _3, _4), (2,_1), (1, _4)]
    /// We will remove the first one and modify the second one like this:
    /// [(2, _3, _4), (1, _4), (-1, _2), (-1/2, _3), (-1/2, _2, _3)]
    ///
    /// This transformation is relevant for Plonk-ish backends although they have a limited width because
    /// they can potentially handle expressions with large linear combinations using 'big-add' gates.
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
                //index, value, and predicate
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

    /// Returns the 'updated' opcode at index 'g' in the circuit
    /// The modifications to the circuits are stored with 'deleted_gates' and 'modified_gates'
    /// These structures are used to give the 'updated' opcode.
    /// For instance, if the opcode has been deleted inside 'deleted_gates', then it returns None.
    fn get_opcode(&self, g: usize, circuit: &Circuit<F>) -> Option<Opcode<F>> {
        if self.deleted_gates.contains(&g) {
            return None;
        }
        self.modified_gates.get(&g).or(circuit.opcodes.get(g)).cloned()
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
