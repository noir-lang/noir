use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode,
        brillig::{BrilligInputs, BrilligOutputs},
        opcodes::{BlockId, FunctionInput},
    },
    native_types::{Expression, Witness},
};
use std::collections::HashSet;

/// Simulate solving a circuit symbolically
/// Instead of evaluating witness values from the inputs, like the PWG module is doing,
/// this pass simply marks the witness that can be evaluated, from the known inputs,
/// and incrementally from the previously marked witnesses.
/// This avoids any computation on a big field which makes the process efficient.
/// When all the witness of an opcode are marked as solvable, it means that the
/// opcode is solvable.
#[derive(Default)]
pub struct CircuitSimulator {
    /// Track the witnesses that can be solved
    solvable_witnesses: HashSet<Witness>,

    /// Track whether a [`BlockId`] has been initialized
    initialized_blocks: HashSet<BlockId>,
}

impl CircuitSimulator {
    /// Simulate solving a circuit symbolically by keeping track of the witnesses that can be solved.
    /// Returns the index of an opcode that cannot be solved, if any.
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn check_circuit<F: AcirField>(&mut self, circuit: &Circuit<F>) -> Option<usize> {
        let circuit_inputs = circuit.circuit_arguments();
        self.solvable_witnesses.extend(circuit_inputs.iter());
        for (i, op) in circuit.opcodes.iter().enumerate() {
            if !self.try_solve(op) {
                return Some(i);
            }
        }
        None
    }

    /// Check if the Opcode can be solved, and if yes, add the solved witness to set of solvable witness
    fn try_solve<F: AcirField>(&mut self, opcode: &Opcode<F>) -> bool {
        match opcode {
            Opcode::AssertZero(expr) => {
                let mut unresolved = HashSet::new();
                for (_, w1, w2) in &expr.mul_terms {
                    if !self.solvable_witnesses.contains(w1) {
                        if !self.solvable_witnesses.contains(w2) {
                            return false;
                        }
                        unresolved.insert(*w1);
                    }
                    if !self.solvable_witnesses.contains(w2) && w1 != w2 {
                        unresolved.insert(*w2);
                    }
                }
                for (_, w) in &expr.linear_combinations {
                    if !self.solvable_witnesses.contains(w) {
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
            Opcode::MemoryOp { block_id, op } => {
                if !self.initialized_blocks.contains(block_id) {
                    // Memory must be initialized before it can be used.
                    return false;
                }
                if !self.can_solve_expression(&op.index) {
                    return false;
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
                    if !self.solvable_witnesses.contains(w) {
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
                    if !self.solvable_witnesses.contains(w) {
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
        self.solvable_witnesses.insert(witness);
    }

    pub fn can_solve_function_input<F: AcirField>(&self, input: &FunctionInput<F>) -> bool {
        if let FunctionInput::Witness(w) = input {
            return self.solvable_witnesses.contains(w);
        }
        true
    }

    fn can_solve_expression<F>(&self, expr: &Expression<F>) -> bool {
        for w in Self::expr_witness(expr) {
            if !self.solvable_witnesses.contains(&w) {
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

    pub(crate) fn expr_witness<F>(expr: &Expression<F>) -> impl Iterator<Item = Witness> {
        expr.mul_terms
            .iter()
            .flat_map(|i| [i.1, i.2])
            .chain(expr.linear_combinations.iter().map(|i| i.1))
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler::CircuitSimulator;
    use acir::{
        FieldElement,
        acir_field::AcirField,
        circuit::{
            Circuit, Opcode, PublicInputs,
            opcodes::{BlackBoxFuncCall, FunctionInput},
        },
        native_types::{Expression, Witness},
    };
    use std::collections::BTreeSet;

    // TODO(https://github.com/noir-lang/noir/issues/10122): Use `Circuit::from_str` to test ACVM's `CircuitSimulator`
    fn test_circuit(
        opcodes: Vec<Opcode<FieldElement>>,
        private_parameters: BTreeSet<Witness>,
        public_parameters: PublicInputs,
    ) -> Circuit<FieldElement> {
        Circuit {
            function_name: "test_circuit".to_string(),
            current_witness_index: 1,
            opcodes,
            private_parameters,
            public_parameters,
            return_values: PublicInputs::default(),
            assert_messages: Default::default(),
        }
    }

    #[test]
    fn reports_none_for_empty_circuit() {
        let src = "
        private parameters: []
        public parameters: []
        return values: []
        ";
        let empty_circuit = Circuit::from_str(src).unwrap();
        assert!(CircuitSimulator::default().check_circuit(&empty_circuit).is_none());
    }

    #[test]
    fn reports_none_for_connected_circuit() {
        let src = "
        private parameters: [w1]
        public parameters: []
        return values: []
        ASSERT w2 = w1
        ";
        let connected_circuit = Circuit::from_str(src).unwrap();
        assert!(CircuitSimulator::default().check_circuit(&connected_circuit).is_none());
    }

    #[test]
    fn reports_true_for_connected_circuit_with_range() {
        let connected_circuit = test_circuit(
            vec![
                Opcode::AssertZero(Expression {
                    mul_terms: Vec::new(),
                    linear_combinations: vec![
                        (FieldElement::one(), Witness(1)),
                        (-FieldElement::one(), Witness(2)),
                    ],
                    q_c: FieldElement::zero(),
                }),
                Opcode::BlackBoxFuncCall(BlackBoxFuncCall::RANGE {
                    input: FunctionInput::Witness(Witness(3)),
                    num_bits: 8,
                }),
            ],
            BTreeSet::from([Witness(1), Witness(3)]),
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
    fn reports_none_for_blackbox_output() {
        let src = "
        private parameters: [w0, w1]
        public parameters: []
        return values: []
        BLACKBOX::AND lhs: w0, rhs: w1, output: w2, bits: 32
        ASSERT w3 = w2
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert!(CircuitSimulator::default().check_circuit(&circuit).is_none());
    }

    #[test]
    fn reports_none_for_read_memory() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: []
        INIT b0 = [w0]
        READ w1 = b0[0]
        ASSERT w2 = w1
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert!(CircuitSimulator::default().check_circuit(&circuit).is_none());
    }

    #[test]
    fn reports_none_for_call_output() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: []
        CALL func: 0, inputs: [w0], outputs: [w1]
        ASSERT w2 = w1
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert!(CircuitSimulator::default().check_circuit(&circuit).is_none());
    }

    #[test]
    fn reports_none_for_brillig_call_output() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: []
        BRILLIG CALL func: 0, inputs: [w0], outputs: [w1]
        ASSERT w2 = w1
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert!(CircuitSimulator::default().check_circuit(&circuit).is_none());
    }

    #[test]
    fn reports_some_for_disconnected_circuit() {
        let src = "
        private parameters: [w1]
        public parameters: []
        return values: []
        ASSERT w2 = w1
        ASSERT w4 = w3
        ";
        let disconnected_circuit = Circuit::from_str(src).unwrap();
        assert_eq!(CircuitSimulator::default().check_circuit(&disconnected_circuit), Some(1));
    }

    #[test]
    fn reports_some_when_memory_block_is_passed_an_unknown_witness() {
        let src = "
        private parameters: [w1]
        public parameters: []
        return values: []
        ASSERT w1 = 0
        INIT b0 = [w0]
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert_eq!(CircuitSimulator::default().check_circuit(&circuit), Some(1));
    }

    #[test]
    fn reports_some_when_attempting_to_reinitialize_memory_block() {
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: []
        INIT b0 = [w0]
        INIT b0 = [w0]
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert_eq!(CircuitSimulator::default().check_circuit(&circuit), Some(1));
    }

    #[test]
    fn reports_some_when_unknown_witness_is_multiplied_by_itself() {
        // If an AssertZero contains just one unknown witness, it might still not possible
        // to solve if: if that unknown witness is being multiplied by itself.
        let src = "
        private parameters: [w0]
        public parameters: []
        return values: []
        ASSERT w0 = w1*w1
        ";
        let circuit = Circuit::from_str(src).unwrap();
        assert_eq!(CircuitSimulator::default().check_circuit(&circuit), Some(0));
    }
}
