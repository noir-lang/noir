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
    GateSatisifed(FieldElement),
    GateSolveable(FieldElement, (FieldElement, Witness)),
    GateUnsolveable,
}


enum MulTerm {
    OneUnknown(FieldElement, Witness), // (qM * known_witness, unknown_witness)
    TooManyUnknowns,
    Solved(FieldElement)
}

impl ArithmeticSolver {


    /// Derives the rest of the witness based on the initial low level variables
    pub fn solve(initial_witness: &mut BTreeMap<Witness, FieldElement>, circuit: Circuit) {
        if circuit.0.len() == 0 {
            return;
        }

        let mut unsolved_gates = Circuit(Vec::new());

        for gate in circuit.0.into_iter() {
            // Evaluate multiplication term
            let mul_result = ArithmeticSolver::solve_mul_term(&gate, &initial_witness);
            // Evaluate the fan-in terms
            let gate_status = ArithmeticSolver::solve_fan_in_term(&gate, &initial_witness);
  
            match (mul_result, gate_status) {
                (MulTerm::TooManyUnknowns, _) => {
                    unsolved_gates.0.push(gate);
                    continue;
                },
                (_, GateStatus::GateUnsolveable) => {
                    unsolved_gates.0.push(gate);
                    continue;
                }
                (MulTerm::OneUnknown(_,_), GateStatus::GateSolveable(_,_)) =>{
                    unsolved_gates.0.push(gate);
                    continue;
                },
                (MulTerm::OneUnknown(partial_prod,unknown_var), GateStatus::GateSatisifed(sum)) =>{
                    // We have one unknown in the mul term and the fan-in terms are solved. 
                    // Hence the equation is solveable, since there is a single unknown
                    // The equation is: partial_prod * unknown_var + sum + qC = 0

                    let total_sum = sum + gate.q_C;
                    let assignment = -(total_sum / partial_prod);
                    // Add this into the witness assignments
                    initial_witness.insert(unknown_var, assignment);
                },
                (MulTerm::Solved(_), GateStatus::GateSatisifed(_)) =>{
                    // All the variables in the MulTerm are solved and the Fan-in is also solved
                    // There is nothing to solve
                    continue 
                },
                (MulTerm::Solved(total_prod), GateStatus::GateSolveable(partial_sum,(coeff, unknown_var))) =>{
                    // The variables in the MulTerm are solved nad there is one unknown in the Fan-in 
                    // Hence the equation is solveable, since we have one unknown
                    // The equation is total_prod + partial_sum + coeff * unknown_var + q_C = 0

                    let total_sum = total_prod + partial_sum + gate.q_C;
                    let assignment = -(total_sum / coeff);
                    // Add this into the witness assignments
                    initial_witness.insert(unknown_var, assignment);
                },
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
    ) -> MulTerm {
        // First note that the mul term can only contain one/zero term
        // We are assuming it has been optimised.
        match arith_gate.mul_terms.len() {
            0 => return MulTerm::Solved(FieldElement::zero()),
            1 => {
                let q_m = &arith_gate.mul_terms[0].0;
                let w_l = &arith_gate.mul_terms[0].1;
                let w_r = &arith_gate.mul_terms[0].2;

                // Check if these values are in the witness assignments
                let w_l_value = witness_assignments.get(w_l);
                let w_r_value = witness_assignments.get(w_r);

                match (w_l_value, w_r_value) {
                    (None, None) => {return MulTerm::TooManyUnknowns},
                    (Some(w_l), Some(w_r)) => {return MulTerm::Solved(*q_m * *w_l * *w_r)},
                    (None, Some(w_r)) => {return MulTerm::OneUnknown(*q_m * *w_r, w_l.clone())}
                    (Some(w_l),None) => {return MulTerm::OneUnknown(*q_m * *w_l, w_r.clone())}
                };
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
            return GateStatus::GateSatisifed(result);
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
    solved_witness.insert(zero, FieldElement::from(0));
    solved_witness.insert(x, FieldElement::from(0));
    solved_witness.insert(t, FieldElement::from(6));
    solved_witness.insert(z, FieldElement::from(6));

    let solver = ArithmeticSolver::solve(&mut solved_witness, circuit.clone());

    // Create constraint system
    let mut constraints: Vec<Constraint> = Vec::new();

    let mut witness_to_index: BTreeMap<Witness, usize> = BTreeMap::new();

    for gate in circuit.0.into_iter() {
        let mut a: i32 = 0;
        let mut b: i32 = 0;
        let mut c: i32 = 0;
        let mut qm: FieldElement = 0.into();
        let mut ql: FieldElement = 0.into();
        let mut qr: FieldElement = 0.into();
        let mut qo: FieldElement = 0.into();
        let mut qc: FieldElement = 0.into();

        // check mul gate
        if gate.mul_terms.len() != 0 {
            let mul_term = &gate.mul_terms[0];
            qm = mul_term.0;

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
            qo = qO_wO_term.0;

            let wO = &qO_wO_term.1;
            c = fetch_index(&mut witness_to_index, wO.clone());
        }

        // XXX: THis is a code smell. Refactor to be better. Maybe change barretenberg to take vectors
        // If there is more than one term,
        // Then add normally
        if gate.simplified_fan.len() == 2 {
            let qL_wL_term = &gate.simplified_fan[0];
            ql = qL_wL_term.0;

            let wL = &qL_wL_term.1;
            a = fetch_index(&mut witness_to_index, wL.clone());

            let qR_wR_term = &gate.simplified_fan[1];
            qr = qR_wR_term.0;

            let wR = &qR_wR_term.1;
            b = fetch_index(&mut witness_to_index, wR.clone());
        }

        if gate.simplified_fan.len() == 3 {
            let qL_wL_term = &gate.simplified_fan[0];
            ql = qL_wL_term.0;

            let wL = &qL_wL_term.1;
            a = fetch_index(&mut witness_to_index, wL.clone());

            let qR_wR_term = &gate.simplified_fan[1];
            qr = qR_wR_term.0;

            let wR = &qR_wR_term.1;
            b = fetch_index(&mut witness_to_index, wR.clone());

            let qO_wO_term = &gate.simplified_fan[2];
            qo = qO_wO_term.0;

            let wO = &qO_wO_term.1;
            c = fetch_index(&mut witness_to_index, wO.clone());
        }

        // Add the qc term
        qc = gate.q_C;

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

        sorted_witness.push(*i_th_witness);
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
