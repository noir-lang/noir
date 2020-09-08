use noir_evaluator::AndGate;
use noir_evaluator::Witness;
use noir_evaluator::XorGate;
use noir_field::FieldElement;
use std::collections::BTreeMap;

pub struct LogicSolver {}

impl LogicSolver {
    /// Derives the rest of the witness based on the initial low level variables
    fn solve_logic_gate<'a>(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        a: &Witness,
        b: &Witness,
        result: Witness,
        num_bits: u32,
        is_xor_gate: bool,
    ) {
        let w_l = initial_witness.get(a);
        let w_r = initial_witness.get(b);

        let (w_l_value, w_r_value) = match (w_l,w_r) {
            (Some(w_l_value), Some(w_r_value)) => { (w_l_value, w_r_value)
            },
            (_,_) => panic!("This should have been caught by the semantic analyser; or the gates were added in the wrong order. One of your wires are None for the logic gate") 
        };

        if is_xor_gate {
            let assignment = w_l_value.xor(w_r_value, num_bits);
            initial_witness.insert(result, assignment);
        } else {
            let assignment = w_l_value.and(w_r_value, num_bits);
            initial_witness.insert(result, assignment);
        }
    }

    pub fn solve_and_gate(initial_witness: &mut BTreeMap<Witness, FieldElement>, gate: &AndGate) {
        LogicSolver::solve_logic_gate(
            initial_witness,
            &gate.a,
            &gate.b,
            gate.result.clone(),
            gate.num_bits,
            false,
        )
    }
    pub fn solve_xor_gate(initial_witness: &mut BTreeMap<Witness, FieldElement>, gate: &XorGate) {
        LogicSolver::solve_logic_gate(
            initial_witness,
            &gate.a,
            &gate.b,
            gate.result.clone(),
            gate.num_bits,
            true,
        )
    }
}
