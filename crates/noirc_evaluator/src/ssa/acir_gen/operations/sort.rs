use acvm::{
    acir::native_types::Expression,
    acir::{circuit::opcodes::Opcode as AcirOpcode, native_types::Witness},
    FieldElement,
};

use crate::{
    ssa::acir_gen::constraints::{add, mul_with_witness, subtract},
    Evaluator,
};

// Generate gates which ensure that out_expr is a permutation of in_expr
// Returns the control bits of the sorting network used to generate the constrains
pub(crate) fn evaluate_permutation(
    in_expr: &[Expression],
    out_expr: &[Expression],
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    let bits = Vec::new();
    let (w, b) = permutation_layer(in_expr, &bits, true, evaluator);
    // we constrain the network output to out_expr
    for (b, o) in b.iter().zip(out_expr) {
        evaluator.push_opcode(AcirOpcode::Arithmetic(subtract(b, FieldElement::one(), o)));
    }
    w
}

// Same as evaluate_permutation() but uses the provided witness as network control bits
pub(crate) fn evaluate_permutation_with_witness(
    in_expr: &[Expression],
    out_expr: &[Expression],
    bits: &[Witness],
    evaluator: &mut Evaluator,
) {
    let (w, b) = permutation_layer(in_expr, bits, false, evaluator);
    debug_assert_eq!(w, *bits);
    // we constrain the network output to out_expr
    for (b, o) in b.iter().zip(out_expr) {
        evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(b, FieldElement::one(), o)));
    }
}

// Generates gates for a sorting network
// returns witness corresponding to the network configuration and the expressions corresponding to the network output
// in_expr: inputs of the sorting network
// if generate_witness is false, it uses the witness provided in bits instead of generating them
// in both cases it returns the witness of the network configuration
// if generate_witness is true, bits is ignored
fn permutation_layer(
    in_expr: &[Expression],
    bits: &[Witness],
    generate_witness: bool,
    evaluator: &mut Evaluator,
) -> (Vec<Witness>, Vec<Expression>) {
    let n = in_expr.len();
    if n == 1 {
        return (Vec::new(), in_expr.to_vec());
    }
    let n1 = n / 2;

    // witness for the input switches
    let mut conf = iter_extended::vecmap(0..n1, |i| {
        if generate_witness {
            evaluator.add_witness_to_cs()
        } else {
            bits[i]
        }
    });

    // compute expressions after the input switches
    // If inputs are a1,a2, and the switch value is c, then we compute expressions b1,b2 where
    // b1 = a1+q, b2 = a2-q, q = c(a2-a1)
    let mut in_sub1 = Vec::new();
    let mut in_sub2 = Vec::new();
    for i in 0..n1 {
        //q = c*(a2-a1);
        let intermediate = mul_with_witness(
            evaluator,
            &conf[i].into(),
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
    let bits1 = if generate_witness { bits } else { &bits[n1 + (n - 1) / 2..] };
    let (w1, b1) = permutation_layer(&in_sub1, bits1, generate_witness, evaluator);
    let bits2 = if generate_witness { bits } else { &bits[n1 + (n - 1) / 2 + w1.len()..] };
    let (w2, b2) = permutation_layer(&in_sub2, bits2, generate_witness, evaluator);
    // apply the output switches
    for i in 0..(n - 1) / 2 {
        let c = if generate_witness { evaluator.add_witness_to_cs() } else { bits[n1 + i] };
        conf.push(c);
        let intermediate =
            mul_with_witness(evaluator, &c.into(), &subtract(&b2[i], FieldElement::one(), &b1[i]));
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
    use acvm::{
        acir::native_types::WitnessMap,
        pwg::{ACVMStatus, ACVM},
        BlackBoxFunctionSolver, BlackBoxResolutionError, FieldElement,
    };

    use crate::{
        ssa::acir_gen::operations::sort::{evaluate_permutation, permutation_layer},
        Evaluator,
    };
    use rand::prelude::*;

    struct MockBackend {}
    impl BlackBoxFunctionSolver for MockBackend {
        fn schnorr_verify(
            &self,
            _public_key_x: &FieldElement,
            _public_key_y: &FieldElement,
            _signature: &[u8],
            _message: &[u8],
        ) -> Result<bool, BlackBoxResolutionError> {
            panic!("Path not trodden by this test")
        }
        fn pedersen(
            &self,
            _inputs: &[FieldElement],
            _domain_separator: u32,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            panic!("Path not trodden by this test")
        }
        fn fixed_base_scalar_mul(
            &self,
            _input: &FieldElement,
        ) -> Result<(FieldElement, FieldElement), BlackBoxResolutionError> {
            panic!("Path not trodden by this test")
        }
    }

    // Check that a random network constrains its output to be a permutation of any random input
    #[test]
    fn test_permutation() {
        let mut rng = rand::thread_rng();
        for n in 2..50 {
            let mut eval = Evaluator::default();

            //we generate random inputs
            let mut input = Vec::new();
            let mut a_val = Vec::new();
            let mut b_wit = Vec::new();
            let mut initial_witness = WitnessMap::new();
            for i in 0..n {
                let w = eval.add_witness_to_cs();
                input.push(w.into());
                a_val.push(FieldElement::from(rng.next_u32() as i128));
                initial_witness.insert(w, a_val[i]);
            }

            let mut output = Vec::new();
            for _i in 0..n {
                let w = eval.add_witness_to_cs();
                b_wit.push(w);
                output.push(w.into());
            }
            //generate constraints for the inputs
            let w = evaluate_permutation(&input, &output, &mut eval);
            //checks that it generate the same witness
            let (w1, _) = permutation_layer(&input, &w, false, &mut eval);
            assert_eq!(w, w1);
            //we generate random network
            let mut c = Vec::new();
            for _i in 0..w.len() {
                c.push(rng.next_u32() % 2 != 0);
            }
            // initialize bits
            for i in 0..w.len() {
                initial_witness.insert(w[i], FieldElement::from(c[i] as i128));
            }
            // compute the network output by solving the constraints
            let backend = MockBackend {};
            let mut acvm = ACVM::new(backend, eval.opcodes.clone(), initial_witness);
            let solver_status = acvm.solve();
            assert_eq!(solver_status, ACVMStatus::Solved, "Incomplete solution");
            let solved_witness = acvm.finalize();

            let mut b_val = Vec::new();
            for i in 0..output.len() {
                b_val.push(solved_witness[&b_wit[i]]);
            }
            // ensure the outputs are a permutation of the inputs
            a_val.sort();
            b_val.sort();
            assert_eq!(a_val, b_val);
        }
    }
}
