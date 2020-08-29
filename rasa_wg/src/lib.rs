use rasa_evaluator::polynomial::Arithmetic;
use rasa_evaluator::{Circuit, Witness};
use rasa_field::FieldElement;
use std::collections::BTreeMap;

/// An Arithmetic solver will take a Circuit's arithmetic gates with witness assignments
/// and create the other witness variables
pub struct ArithmeticSolver {
    witness: BTreeMap<Witness, FieldElement>,
    circuit: Circuit,
}

enum GateStatus {
    GateSatisifed,
    GateSolveable(FieldElement, (FieldElement, Witness)),
    GateUnsolveable,
}

impl ArithmeticSolver {

    /// Derives the rest of the witness based on the initial low level variables
    pub fn solve(initial_witness: &mut BTreeMap<Witness, FieldElement>, circuit: Circuit) {
        if circuit.0.len() == 0 {
            return;
        }

        let mut unsolved_gates = Circuit(Vec::new());

        for gate in circuit.0.into_iter() {
            // XXX: Assuming that each gate has at least one element in the fan-in. Check correctness.
            // qM * w_L * w_R + q_C
            // For example, the above is skipped
            if gate.simplified_fan.len() == 0 {
                continue;
            }

            let mut result = FieldElement::zero();

            // Evaluate multiplication term
            // Note: We assume that if any terms are missing in the mul situation, then the gate cannot be solved
            let mul_result = ArithmeticSolver::solve_mul_term(&gate, &initial_witness);
            if mul_result.is_none() {
                unsolved_gates.0.push(gate);
                continue;
            }
            result += mul_result.unwrap();

            // Evaluate the fan-in terms
            let gate_status = ArithmeticSolver::solve_fan_in_term(&gate, &initial_witness);

            match gate_status {
                GateStatus::GateUnsolveable => {
                    unsolved_gates.0.push(gate);
                    continue;
                }
                GateStatus::GateSatisifed => {
                    continue;
                }
                GateStatus::GateSolveable(sum, unknown_var) => {
                    // If we are here, it means we have a single unknown
                    result += sum;

                    // Add the constant term
                    result += gate.q_C;

                    // What we now have is:
                    // result + Ax = 0
                    // solving for x: x = - result/A
                    let A = unknown_var.0;
                    let x = unknown_var.1;
                    let assignment = -FieldElement(result.0 / A.0); // XXX: Need to change this to use proper field elements. Division may not always be correct

                    // Add this into the witness assignments
                    initial_witness.insert(x, assignment);
                }
            };
        }
        ArithmeticSolver::solve(initial_witness, unsolved_gates)
    }

    /// Returns the evaluation of the multiplication term in the arithmetic gate
    /// If the witness values are not known, then the function returns a None
    /// XXX: Do we need to account for the case where 5xy + 6x = 0 ? We do not know y, but it can be solved given x . But I believe x can be solved with another gate  
    /// XXX: What about making a mul gate = a constant 5xy + 7 = 0 ? This is the same as the above.
    fn solve_mul_term(
        arith_gate: &Arithmetic,
        witness_assignments: &BTreeMap<Witness, FieldElement>,
    ) -> Option<FieldElement> {
        // First note that the mul term can only contain one/zero term
        // We are assuming it has been optimised.
        match arith_gate.mul_terms.len() {
            0 => return Some(FieldElement::zero()),
            1 => {
                let q_m = &arith_gate.mul_terms[0].0;
                let w_l = &arith_gate.mul_terms[0].1;
                let w_r = &arith_gate.mul_terms[0].2;

                // Check if these values are in the witness assignments
                let w_l_value = *witness_assignments.get(w_l)?;
                let w_r_value = *witness_assignments.get(w_r)?;

                let result = *q_m * w_l_value * w_r_value;

                return Some(result);
            }
            _ => panic!("Mul term in the arithmetic gate must contain either zero or one term"),
        }
    }

    /// Returns the summation of all of the variables, plus the unknown variable
    /// Returns None, if there is more than one unknown variable
    /// We cannot assign
    fn solve_fan_in_term(
        arith_gate: &Arithmetic,
        witness_assignments: &BTreeMap<Witness, FieldElement>,
    ) -> GateStatus {
        // This is assuming that the fan-in is more than 0

        // This is the variable that we want to assign the value to
        let mut unknown_variable = (FieldElement::zero(), Witness::default());
        let mut num_unknowns = 0;
        // This is the sum of all of the known variables
        let mut result = FieldElement::zero();

        for term in arith_gate.simplified_fan.iter() {
            let q_l = term.0;
            let w_l = &term.1;

            // Check if we have w_l
            let w_l_value = witness_assignments.get(w_l);

            match w_l_value {
                Some(a) => result += q_l * *a,
                None => {
                    unknown_variable = term.clone();
                    num_unknowns += 1;
                }
            };

            // If we have more than 1 unknown, then we cannot solve this equation
            if num_unknowns > 1 {
                return GateStatus::GateUnsolveable;
            }
        }

        if num_unknowns == 0 {
            return GateStatus::GateSatisifed;
        }

        GateStatus::GateSolveable(result, unknown_variable)
    }
}

#[test]
fn name() {
    use rasa_evaluator::circuit::Witness;

    use rasa_evaluator::{Environment, Evaluator};
    use librasac_lexer::lexer::Lexer;
    use librasac_parser::Parser;
    use barretenberg_rs::composer::{Assignments, Constraint, ConstraintSystem, StandardComposer};

    let input = "
  
    fn main(x : Witness, z : Witness, t : Witness) {
        priv y = x * z;
        const a = 2;
        constrain y == x + 2;
        constrain z == y * x; 
    }
    ";

    let mut parser = Parser::new(Lexer::new(&input));
    let program = parser.parse_program();
    let evaluator = Evaluator::new(program);

    let circuit = evaluator.evaluate(&mut Environment::new());
}

#[test]
fn test_simple_circuit() {
    use rasa_evaluator::circuit::Witness;

    use rasa_evaluator::{Environment, Evaluator};
    use librasac_lexer::lexer::Lexer;
    use librasac_parser::Parser;
    use barretenberg_rs::composer::{Assignments, Constraint, ConstraintSystem, StandardComposer};

    let input = "
    fn hello(y : Witness,e : Witness) {
        (y+e) * e
    }
    fn main(x : Witness, z : Witness, t : Witness) {
        priv y = x * z;
        constrain y == hello(y,x);
    }
    ";

    let mut parser = Parser::new(Lexer::new(&input));
    let program = parser.parse_program();
    librasac_analyser::check(&program);
    let evaluator = Evaluator::new(program);

    let (circuit, _) = evaluator.evaluate(&mut Environment::new());

    // Parameters to main function
    let x = Witness("x".to_string(),1);
    let t = Witness("t".to_string(),2);
    let z = Witness("z".to_string(),3);
    let zero = Witness("0".to_string(),0);

    let mut solved_witness = BTreeMap::new();
    solved_witness.insert(zero, FieldElement(0));
    solved_witness.insert(x, FieldElement(0));
    solved_witness.insert(t, FieldElement(6));
    solved_witness.insert(z, FieldElement(6));

    let solver = ArithmeticSolver::solve(&mut solved_witness, circuit.clone());

    // Create constraint system
    let mut constraints: Vec<Constraint> = Vec::new();

    let mut witness_to_index: BTreeMap<Witness, usize> = BTreeMap::new();

    for gate in circuit.0.into_iter() {
        let mut a: i32 = 0;
        let mut b: i32 = 0;
        let mut c: i32 = 0;
        let mut qm: i32 = 0;
        let mut ql: i32 = 0;
        let mut qr: i32 = 0;
        let mut qo: i32 = 0;
        let mut qc: i32 = 0;

        // check mul gate
        if gate.mul_terms.len() != 0 {
            let mul_term = &gate.mul_terms[0];
            qm = (mul_term.0).0 as i32;

            // Get wL term
            let wL = &mul_term.1;
            a = fetch_index(&mut witness_to_index, wL.clone());

            // Get wR term
            let wR = &mul_term.2;
            b = fetch_index(&mut witness_to_index, wR.clone());
        }

        // If there is only one simplified fan term,
        // then put it in qO * wO
        // This is incase, the qM term is non-zero
        if gate.simplified_fan.len() == 1 {
            let qO_wO_term = &gate.simplified_fan[0];
            qo = (qO_wO_term.0).0 as i32;

            let wO = &qO_wO_term.1;
            c = fetch_index(&mut witness_to_index, wO.clone());
        }

        // XXX: THis is a code smell. Refactor to be better. Maybe change barretenberg to take vectors
        // If there is more than one term,
        // Then add normally
        if gate.simplified_fan.len() == 2 {
            let qL_wL_term = &gate.simplified_fan[0];
            ql = (qL_wL_term.0).0 as i32;

            let wL = &qL_wL_term.1;
            a = fetch_index(&mut witness_to_index, wL.clone());

            let qR_wR_term = &gate.simplified_fan[1];
            qr = (qR_wR_term.0).0 as i32;

            let wR = &qR_wR_term.1;
            b = fetch_index(&mut witness_to_index, wR.clone());
        }

        if gate.simplified_fan.len() == 3 {
            let qL_wL_term = &gate.simplified_fan[0];
            ql = (qL_wL_term.0).0 as i32;

            let wL = &qL_wL_term.1;
            a = fetch_index(&mut witness_to_index, wL.clone());

            let qR_wR_term = &gate.simplified_fan[1];
            qr = (qR_wR_term.0).0 as i32;

            let wR = &qR_wR_term.1;
            b = fetch_index(&mut witness_to_index, wR.clone());

            let qO_wO_term = &gate.simplified_fan[2];
            qo = (qO_wO_term.0).0 as i32;

            let wO = &qO_wO_term.1;
            c = fetch_index(&mut witness_to_index, wO.clone());
        }

        // Add the qc term
        qc = (gate.q_C.0) as i32;

        let constraint = Constraint {
            a,
            b,
            c,
            qm,
            ql,
            qr,
            qo,
            qc,
        };
        constraints.push(constraint);
    }

    // Create constraint system
    let constraint_system = ConstraintSystem {
        var_num: solved_witness.len() as u32,
        pub_var_num: 0,
        constraints: constraints,
    };

    let mut composer = StandardComposer::new(constraint_system.size());

    // Add witnesses
    let mut sorted_witness = Assignments::new();
    for i in 0..witness_to_index.len() {
        dbg!(i);
        let witness = find_key_for_value(&witness_to_index, i + 1).unwrap();

        let i_th_witness = solved_witness.get(witness).unwrap();

        sorted_witness.push(i_th_witness.0 as i32);
    }

    let proof = composer.create_proof(&constraint_system, sorted_witness);

    let public_inputs = None;
    let verified = composer.verify(&constraint_system, &proof, public_inputs);

    println!("Proof verified : {}\n", verified);

    // dbg!(solved_witness);
}

fn find_key_for_value(map: &BTreeMap<Witness, usize>, value: usize) -> Option<&Witness> {
    map.iter()
        .find_map(|(key, &val)| if val == value { Some(key) } else { None })
}

fn fetch_index(map: &mut BTreeMap<Witness, usize>, witness: Witness) -> i32 {
    let possible_index = map.get(&witness);
    let num_witnesses = map.len();
    match possible_index {
        Some(index) => *index as i32,
        None => {
            // XXX: plus one is due to the constraint system starting from 1
            map.insert(witness, num_witnesses + 1);
            (num_witnesses + 1) as i32
        }
    }
}
