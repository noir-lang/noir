mod arithmetic;
mod gadget_call;
mod logic;

pub use arithmetic::ArithmeticSolver;
pub use gadget_call::GadgetCaller;
pub use logic::LogicSolver;

use noir_evaluator::Gate;
use noir_evaluator::{Circuit, Witness};
use noir_field::FieldElement;
use std::collections::BTreeMap;

pub struct Solver {}

impl Solver {
    /// Derives the rest of the witness based on the initial low level variables
    pub fn solve(initial_witness: &mut BTreeMap<Witness, FieldElement>, circuit: Circuit) {
        if circuit.0.len() == 0 {
            return;
        }

        let mut unsolved_gates = Circuit(Vec::new());

        for gate in circuit.0.into_iter() {
            let unsolved = match &gate {
                Gate::Arithmetic(arith) => {
                    ArithmeticSolver::solve(initial_witness, &arith).is_some()
                }
                Gate::Range(_, _) => {
                    // We do not need to solve for this gate, we have passed responsibility to the underlying
                    // proof system for intermediate witness generation
                    false
                }
                Gate::And(and_gate) => {
                    LogicSolver::solve_and_gate(initial_witness, and_gate);

                    // We compute the result because the other gates may want to use the assignment to generate their assignments
                    false
                }
                Gate::Xor(xor_gate) => {
                    LogicSolver::solve_xor_gate(initial_witness, xor_gate);

                    // We compute the result because the other gates may want to use the assignment to generate their assignments
                    false
                }
                Gate::GadgetCall(gc) => {
                    GadgetCaller::solve_gadget_call(initial_witness, gc);

                    false
                }
                gate => panic!(
                    "Solver does not know how to deal with this Gate: {:?}",
                    gate
                ),
            };
            if unsolved {
                unsolved_gates.0.push(gate);
            }
        }
        Solver::solve(initial_witness, unsolved_gates)
    }
}

#[test]
fn name() {
    use noir_evaluator::circuit::Witness;

    use libnoirc_lexer::lexer::Lexer;
    use libnoirc_parser::Parser;
    use noir_evaluator::{Environment, Evaluator};
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
    let symbol_table = libnoirc_analyser::build_symbol_table(&program);
    let evaluator = Evaluator::new(program, symbol_table);

    let circuit = evaluator.evaluate(&mut Environment::new());
}

#[test]
fn test_simple_circuit() {
    use noir_field::FieldElement;
    use std::collections::BTreeMap;

    use noir_evaluator::circuit::Witness;

    use libnoirc_lexer::lexer::Lexer;
    use libnoirc_parser::Parser;
    use noir_evaluator::{Environment, Evaluator};

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
    let symbol_table = libnoirc_analyser::build_symbol_table(&program);
    let checked_program = libnoirc_analyser::check(program);
    let evaluator = Evaluator::new(checked_program, symbol_table);

    let (circuit, _, num_pub_inputs) = evaluator.evaluate(&mut Environment::new());

    // Parameters to main function
    let x = Witness("x".to_string(), 1);
    let t = Witness("t".to_string(), 2);
    let z = Witness("z".to_string(), 3);
    let zero = Witness("0".to_string(), 0);

    let mut solved_witness = BTreeMap::new();
    solved_witness.insert(zero, FieldElement::from(0));
    solved_witness.insert(x, FieldElement::from(0));
    solved_witness.insert(t, FieldElement::from(6));
    solved_witness.insert(z, FieldElement::from(6));

    Solver::solve(&mut solved_witness, circuit.clone());

    // Create constraint system
    let constraint_system =
        noir_serialiser::serialise_circuit(&circuit, solved_witness.len(), num_pub_inputs);
}
