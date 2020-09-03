mod arithmetic;
mod directive;
pub use arithmetic::ArithmeticSolver;

use rasa_evaluator::Gate;
use rasa_evaluator::{Circuit, Witness};
use rasa_field::FieldElement;
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
                Gate::Directive(direct) => {
                    directive::DirectiveSolver::solve(initial_witness, &direct).is_some()
                }
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
    use rasa_evaluator::circuit::Witness;

    use librasac_lexer::lexer::Lexer;
    use librasac_parser::Parser;
    use rasa_evaluator::{Environment, Evaluator};
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
    use rasa_field::FieldElement;
    use std::collections::BTreeMap;

    use rasa_evaluator::circuit::Witness;

    use librasac_lexer::lexer::Lexer;
    use librasac_parser::Parser;
    use rasa_evaluator::{Environment, Evaluator};

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
        rasa_serialiser::serialise_circuit(&circuit, solved_witness.len(), num_pub_inputs);
}
