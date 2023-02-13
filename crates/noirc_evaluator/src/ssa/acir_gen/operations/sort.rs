use acvm::{
    acir::native_types::Expression,
    acir::{circuit::opcodes::Opcode as AcirOpcode, native_types::Witness},
    FieldElement,
};

use crate::{
    ssa::acir_gen::{
        constraints::{add, mul_with_witness, subtract},
        expression_from_witness,
    },
    Evaluator,
};

// Generate gates which ensure that out_expr is a permutation of in_expr
// Returns the control bits of the sorting network used to generate the constrains
pub fn evaluate_permutation(
    in_expr: &Vec<Expression>,
    out_expr: &Vec<Expression>,
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    let (w, b) = permutation_layer(in_expr, evaluator);
    // we contrain the network output to out_expr
    for (b, o) in b.iter().zip(out_expr) {
        evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(b, FieldElement::one(), o)));
    }
    w
}

// Generates gates for a sorting network
// returns witness corresponding to the network configuration and the expressions corresponding to the network output
// in_expr: inputs of the sorting network
pub fn permutation_layer(
    in_expr: &Vec<Expression>,
    evaluator: &mut Evaluator,
) -> (Vec<Witness>, Vec<Expression>) {
    let n = in_expr.len();
    if n == 1 {
        return (Vec::new(), in_expr.clone());
    }
    let n1 = n / 2;
    let mut conf = Vec::new();
    // witness for the input switches
    for _ in 0..n1 {
        conf.push(evaluator.add_witness_to_cs());
    }
    // compute expressions after the input switches
    // If inputs are a1,a2, and the switch value is c, then we compute expresions b1,b2 where
    // b1 = a1+q, b2 = a2-q, q = c(a2-a1)
    let mut in_sub1 = Vec::new();
    let mut in_sub2 = Vec::new();
    for i in 0..n1 {
        //q = c*(a2-a1);
        let intermediate = mul_with_witness(
            evaluator,
            &expression_from_witness(conf[i]),
            &subtract(&in_expr[2 * i + 1], FieldElement::one(), &in_expr[2 * i]),
        );
        //b1=a1+q
        in_sub1.push(add(&intermediate, FieldElement::one(), &in_expr[2 * i]));
        //b2=a2-q
        in_sub2.push(subtract(&in_expr[2 * i + 1], FieldElement::one(), &intermediate));
    }
    if n % 2 == 1 {
        in_sub2.push(in_expr.last().unwrap().clone());
    }
    let mut out_expr = Vec::new();
    // compute results for the sub networks
    let (w1, b1) = permutation_layer(&in_sub1, evaluator);
    let (w2, b2) = permutation_layer(&in_sub2, evaluator);
    // apply the output swithces
    for i in 0..(n - 1) / 2 {
        let c = evaluator.add_witness_to_cs();
        conf.push(c);
        let intermediate = mul_with_witness(
            evaluator,
            &expression_from_witness(c),
            &subtract(&b2[i], FieldElement::one(), &b1[i]),
        );
        out_expr.push(add(&intermediate, FieldElement::one(), &b1[i]));
        out_expr.push(subtract(&b2[i], FieldElement::one(), &intermediate));
    }
    if n % 2 == 0 {
        out_expr.push(b1.last().unwrap().clone());
    }
    out_expr.push(b2.last().unwrap().clone());
    conf.extend(w1);
    conf.extend(w2);
    (conf, out_expr)
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acvm::{
        acir::{circuit::opcodes::BlackBoxFuncCall, native_types::Witness},
        FieldElement, OpcodeResolutionError, PartialWitnessGenerator,
    };

    use crate::{
        ssa::acir_gen::{expression_from_witness, operations::sort::evaluate_permutation},
        Evaluator,
    };
    use rand::prelude::*;

    struct MockBackend {}
    impl PartialWitnessGenerator for MockBackend {
        fn solve_black_box_function_call(
            _initial_witness: &mut BTreeMap<Witness, FieldElement>,
            _func_call: &BlackBoxFuncCall,
        ) -> Result<(), OpcodeResolutionError> {
            unreachable!();
        }
    }

    // Check that a random network constrains its output to be a permutation of any random input
    #[test]
    fn test_permutation() {
        let mut rng = rand::thread_rng();
        for n in 2..50 {
            let mut eval = Evaluator {
                current_witness_index: 0,
                num_witnesses_abi_len: 0,
                public_inputs: Vec::new(),
                opcodes: Vec::new(),
            };

            //we generate random inputs
            let mut input = Vec::new();
            let mut a_val = Vec::new();
            let mut b_wit = Vec::new();
            let mut solved_witness: BTreeMap<Witness, FieldElement> = BTreeMap::new();
            for i in 0..n {
                let w = eval.add_witness_to_cs();
                input.push(expression_from_witness(w));
                a_val.push(FieldElement::from(rng.next_u32() as i128));
                solved_witness.insert(w, a_val[i]);
            }

            let mut output = Vec::new();
            for _i in 0..n {
                let w = eval.add_witness_to_cs();
                b_wit.push(w);
                output.push(expression_from_witness(w));
            }
            //generate constraints for the inputs
            let w = evaluate_permutation(&input, &output, &mut eval);

            //we generate random network
            let mut c = Vec::new();
            for _i in 0..w.len() {
                c.push(rng.next_u32() % 2 != 0);
            }
            // intialise bits
            for i in 0..w.len() {
                solved_witness.insert(w[i], FieldElement::from(c[i] as i128));
            }
            // compute the network output by solving the constraints
            let backend = MockBackend {};
            backend
                .solve(&mut solved_witness, eval.opcodes.clone())
                .expect("Could not solve permutation constraints");
            let mut b_val = Vec::new();
            for i in 0..output.len() {
                b_val.push(solved_witness[&b_wit[i]]);
            }
            // ensure the outputs are a permutation of the inputs
            assert_eq!(a_val.sort(), b_val.sort());
        }
    }
}
