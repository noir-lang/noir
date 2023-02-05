use crate::{
    errors::RuntimeErrorKind,
    ssa::acir_gen::{const_from_expression, expression_from_witness, InternalVar},
    Evaluator,
};
use acvm::{
    acir::{
        circuit::{
            directives::Directive,
            opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
        },
        native_types::{Expression, Linear, Witness},
    },
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::One;
use std::{
    cmp::Ordering,
    ops::{Mul, Neg},
};

// Code in this file, will generate constraints without
// using only the Evaluator and ACIR Expression types

// This interface abstracts away the functionality needed
// by the `Evaluator` to create constraints.
pub trait ACIRState {
    // Adds a new `Witness` into the constraint system.
    // This can now be used inside of ACIR opcodes.
    fn add_witness_to_cs(&mut self) -> Witness;
    // Creates a `Witness` variable that is equivalent to the
    // expression passed in.
    fn create_intermediate_variable(&mut self, expression: Expression) -> Witness;
    // Stores an ACIR Opcode
    fn add_opcode(&mut self, opcode: AcirOpcode);
}

impl ACIRState for Evaluator {
    fn add_witness_to_cs(&mut self) -> Witness {
        Evaluator::add_witness_to_cs(self)
    }

    fn create_intermediate_variable(&mut self, expression: Expression) -> Witness {
        Evaluator::create_intermediate_variable(self, expression)
    }

    fn add_opcode(&mut self, opcode: AcirOpcode) {
        self.opcodes.push(opcode)
    }
}

pub(crate) fn mul_with_witness<A: ACIRState>(
    evaluator: &mut A,
    a: &Expression,
    b: &Expression,
) -> Expression {
    let a_arith;
    let a_arith = if !a.mul_terms.is_empty() && !b.is_const() {
        let a_witness = evaluator.add_witness_to_cs();
        a_arith = Expression::from(&a_witness);
        evaluator.add_opcode(AcirOpcode::Arithmetic(a - &a_arith));
        &a_arith
    } else {
        a
    };
    let b_arith;
    let b_arith = if !b.mul_terms.is_empty() && !a.is_const() {
        if a == b {
            a_arith
        } else {
            let b_witness = evaluator.add_witness_to_cs();
            b_arith = Expression::from(&b_witness);
            evaluator.add_opcode(AcirOpcode::Arithmetic(b - &b_arith));
            &b_arith
        }
    } else {
        b
    };
    mul(a_arith, b_arith)
}

//a*b
pub(crate) fn mul(a: &Expression, b: &Expression) -> Expression {
    let zero = Expression::zero();
    if a.is_const() {
        return add(&zero, a.q_c, b);
    } else if b.is_const() {
        return add(&zero, b.q_c, a);
    } else if !(a.is_linear() && b.is_linear()) {
        unreachable!("Can only multiply linear terms");
    }

    let mut output = Expression::from_field(a.q_c * b.q_c);

    //TODO to optimize...
    for lc in &a.linear_combinations {
        let single = single_mul(lc.1, b);
        output = add(&output, lc.0, &single);
    }

    //linear terms
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        let coeff_a = b.q_c * a.linear_combinations[i1].0;
        let coeff_b = a.q_c * b.linear_combinations[i2].0;
        match a.linear_combinations[i1].1.cmp(&b.linear_combinations[i2].1) {
            Ordering::Greater => {
                if coeff_b != FieldElement::zero() {
                    output.linear_combinations.push((coeff_b, b.linear_combinations[i2].1));
                }
                i2 += 1;
            }
            Ordering::Less => {
                if coeff_a != FieldElement::zero() {
                    output.linear_combinations.push((coeff_a, a.linear_combinations[i1].1));
                }
                i1 += 1;
            }
            Ordering::Equal => {
                if coeff_a + coeff_b != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coeff_a + coeff_b, a.linear_combinations[i1].1));
                }

                i1 += 1;
                i2 += 1;
            }
        }
    }
    while i1 < a.linear_combinations.len() {
        let coeff_a = b.q_c * a.linear_combinations[i1].0;
        output.linear_combinations.push((coeff_a, a.linear_combinations[i1].1));
        i1 += 1;
    }
    while i2 < b.linear_combinations.len() {
        let coeff_b = a.q_c * b.linear_combinations[i2].0;
        output.linear_combinations.push((coeff_b, b.linear_combinations[i2].1));
        i2 += 1;
    }

    output
}

// returns a - k*b
pub(crate) fn subtract(a: &Expression, k: FieldElement, b: &Expression) -> Expression {
    add(a, k.neg(), b)
}

// returns a + k*b
pub(crate) fn add(a: &Expression, k: FieldElement, b: &Expression) -> Expression {
    let mut output = Expression::default();

    //linear combinations
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        match a.linear_combinations[i1].1.cmp(&b.linear_combinations[i2].1) {
            Ordering::Greater => {
                let coeff = b.linear_combinations[i2].0 * k;
                if coeff != FieldElement::zero() {
                    output.linear_combinations.push((coeff, b.linear_combinations[i2].1));
                }
                i2 += 1;
            }
            Ordering::Less => {
                output.linear_combinations.push(a.linear_combinations[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                let coeff = a.linear_combinations[i1].0 + b.linear_combinations[i2].0 * k;
                if coeff != FieldElement::zero() {
                    output.linear_combinations.push((coeff, a.linear_combinations[i1].1));
                }
                i2 += 1;
                i1 += 1;
            }
        }
    }
    while i1 < a.linear_combinations.len() {
        output.linear_combinations.push(a.linear_combinations[i1]);
        i1 += 1;
    }
    while i2 < b.linear_combinations.len() {
        let coeff = b.linear_combinations[i2].0 * k;
        if coeff != FieldElement::zero() {
            output.linear_combinations.push((coeff, b.linear_combinations[i2].1));
        }
        i2 += 1;
    }

    //mul terms

    i1 = 0; //a
    i2 = 0; //b

    while i1 < a.mul_terms.len() && i2 < b.mul_terms.len() {
        match (a.mul_terms[i1].1, a.mul_terms[i1].2).cmp(&(b.mul_terms[i2].1, b.mul_terms[i2].2)) {
            Ordering::Greater => {
                let coeff = b.mul_terms[i2].0 * k;
                if coeff != FieldElement::zero() {
                    output.mul_terms.push((coeff, b.mul_terms[i2].1, b.mul_terms[i2].2));
                }
                i2 += 1;
            }
            Ordering::Less => {
                output.mul_terms.push(a.mul_terms[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                let coeff = a.mul_terms[i1].0 + b.mul_terms[i2].0 * k;
                if coeff != FieldElement::zero() {
                    output.mul_terms.push((coeff, a.mul_terms[i1].1, a.mul_terms[i1].2));
                }
                i2 += 1;
                i1 += 1;
            }
        }
    }
    while i1 < a.mul_terms.len() {
        output.mul_terms.push(a.mul_terms[i1]);
        i1 += 1;
    }

    while i2 < b.mul_terms.len() {
        let coeff = b.mul_terms[i2].0 * k;
        if coeff != FieldElement::zero() {
            output.mul_terms.push((coeff, b.mul_terms[i2].1, b.mul_terms[i2].2));
        }
        i2 += 1;
    }

    output.q_c = a.q_c + k * b.q_c;
    output
}

// returns w*b.linear_combinations
pub(crate) fn single_mul(w: Witness, b: &Expression) -> Expression {
    let mut output = Expression::default();
    let mut i1 = 0;
    while i1 < b.linear_combinations.len() {
        if (w, b.linear_combinations[i1].1) < (b.linear_combinations[i1].1, w) {
            output.mul_terms.push((b.linear_combinations[i1].0, w, b.linear_combinations[i1].1));
        } else {
            output.mul_terms.push((b.linear_combinations[i1].0, b.linear_combinations[i1].1, w));
        }
        i1 += 1;
    }
    output
}

pub(crate) fn boolean(witness: Witness) -> Expression {
    Expression {
        mul_terms: vec![(FieldElement::one(), witness, witness)],
        linear_combinations: vec![(-FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}

pub(crate) fn boolean_expr<A: ACIRState>(expr: &Expression, evaluator: &mut A) -> Expression {
    subtract(&mul_with_witness(evaluator, expr, expr), FieldElement::one(), expr)
}

//constrain witness a to be num_bits-size integer, i.e between 0 and 2^num_bits-1
pub fn range_constraint<A: ACIRState>(
    witness: Witness,
    num_bits: u32,
    evaluator: &mut A,
) -> Result<(), RuntimeErrorKind> {
    if num_bits == 1 {
        // Add a bool gate
        let bool_constraint = boolean(witness);
        evaluator.add_opcode(AcirOpcode::Arithmetic(bool_constraint));
    } else if num_bits == FieldElement::max_num_bits() {
        // Don't apply any constraints if the range is for the maximum number of bits
        let message = format!(
            "All Witnesses are by default u{}. Applying this type does not apply any constraints.",
            FieldElement::max_num_bits()
        );
        return Err(RuntimeErrorKind::UnstructuredError { message });
    } else if num_bits % 2 == 1 {
        // Note if the number of bits is odd, then Barretenberg will panic
        // new witnesses; r is constrained to num_bits-1 and b is 1 bit
        let r_witness = evaluator.add_witness_to_cs();
        let b_witness = evaluator.add_witness_to_cs();
        evaluator.add_opcode(AcirOpcode::Directive(Directive::OddRange {
            a: witness,
            b: b_witness,
            r: r_witness,
            bit_size: num_bits,
        }));

        try_range_constraint(r_witness, num_bits - 1, evaluator);
        try_range_constraint(b_witness, 1, evaluator);

        //Add the constraint a = r + 2^N*b
        let mut f = FieldElement::from(2_i128);
        f = f.pow(&FieldElement::from((num_bits - 1) as i128));
        let res = add(&expression_from_witness(r_witness), f, &expression_from_witness(b_witness));
        let my_constraint = add(&res, -FieldElement::one(), &expression_from_witness(witness));
        evaluator.add_opcode(AcirOpcode::Arithmetic(my_constraint));
    } else {
        let gate = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: acvm::acir::BlackBoxFunc::RANGE,
            inputs: vec![FunctionInput { witness, num_bits }],
            outputs: vec![],
        });
        evaluator.add_opcode(gate);
    }

    Ok(())
}

// returns a witness of a>=b
pub(crate) fn bound_check<A: ACIRState>(
    a: &Expression,
    b: &Expression,
    max_bits: u32,
    evaluator: &mut A,
) -> Witness {
    assert!(max_bits + 1 < FieldElement::max_num_bits()); //n.b what we really need is 2^{max_bits+1}<p
    let mut sub = subtract(a, FieldElement::one(), b);
    let two = FieldElement::from(2_i128);
    let two_s = two.pow(&FieldElement::from(max_bits as i128));
    sub.q_c += two_s;
    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();
    //2^s+a-b=q*2^s +r
    let expr = add(&expression_from_witness(r_witness), two_s, &expression_from_witness(q_witness));
    evaluator.add_opcode(AcirOpcode::Arithmetic(subtract(&sub, FieldElement::one(), &expr)));
    evaluator.add_opcode(AcirOpcode::Directive(Directive::Truncate {
        a: sub,
        b: r_witness,
        c: q_witness,
        bit_size: max_bits,
    }));
    try_range_constraint(r_witness, max_bits, evaluator);
    evaluator.add_opcode(AcirOpcode::Arithmetic(boolean(q_witness)));
    q_witness
}

// Generate constraints that are satisfied iff
// a < b , when offset is 1, or
// a <= b, when offset is 0
// bits is the bit size of a and b (or an upper bound of the bit size)
///////////////
// a<=b is done by constraining b-a to a bit size of 'bits':
// if a<=b, 0 <= b-a <= b < 2^bits
// if a>b, b-a = p+b-a > p-2^bits >= 2^bits  (if log(p) >= bits + 1)
// n.b: we do NOT check here that a and b are indeed 'bits' size
// a < b <=> a+1<=b
pub(crate) fn bound_constraint_with_offset<A: ACIRState>(
    a: &Expression,
    b: &Expression,
    offset: &Expression,
    bits: u32,
    evaluator: &mut A,
) {
    assert!(
        bits < FieldElement::max_num_bits(),
        "range check with bit size of the prime field is not implemented yet"
    );

    let mut aof = add(a, FieldElement::one(), offset);

    if b.is_const() && b.q_c.fits_in_u128() {
        let f = if *offset == Expression::one() {
            aof = a.clone();
            assert!(b.q_c.to_u128() >= 1);
            b.q_c.to_u128() - 1
        } else {
            b.q_c.to_u128()
        };

        if f < 3 {
            match f {
                0 => evaluator.add_opcode(AcirOpcode::Arithmetic(aof)),
                1 => {
                    let expr = boolean_expr(&aof, evaluator);
                    evaluator.add_opcode(AcirOpcode::Arithmetic(expr))
                }
                2 => {
                    let y = InternalVar::expression_to_witness(
                        boolean_expr(&aof, evaluator),
                        evaluator,
                    );
                    let two = FieldElement::from(2_i128);
                    let y_expr = expression_from_witness(y);
                    let eee = subtract(&mul_with_witness(evaluator, &aof, &y_expr), two, &y_expr);
                    evaluator.add_opcode(AcirOpcode::Arithmetic(eee));
                }
                _ => unreachable!(),
            }
            return;
        }
        let bit_size = bit_size_u128(f);
        if bit_size < 128 {
            let r = (1_u128 << bit_size) - f - 1;
            assert!(bits + bit_size < FieldElement::max_num_bits()); //we need to ensure a+r does not overflow
            let aor = add(&aof, FieldElement::from(r), &Expression::one());
            let witness = InternalVar::expression_to_witness(aor, evaluator);
            try_range_constraint(witness, bit_size, evaluator);
            return;
        }
    }

    let sub_expression = subtract(b, FieldElement::one(), &aof); //b-(a+offset)
    let w = InternalVar::expression_to_witness(sub_expression, evaluator);
    try_range_constraint(w, bits, evaluator);
}

pub(crate) fn try_range_constraint<A: ACIRState>(w: Witness, bits: u32, evaluator: &mut A) {
    if let Err(err) = range_constraint(w, bits, evaluator) {
        eprintln!("{err}");
    }
}

//decompose lhs onto radix-base with limb_size limbs
pub(crate) fn to_radix_base<A: ACIRState>(
    lhs: &Expression,
    radix: u32,
    limb_size: u32,
    evaluator: &mut A,
) -> Vec<Witness> {
    // ensure there is no overflow
    let mut max = BigUint::from(radix);
    max = max.pow(limb_size) - BigUint::one();
    assert!(max < FieldElement::modulus());

    let (result, bytes) = to_radix(radix, limb_size, evaluator);
    evaluator.add_opcode(AcirOpcode::Directive(Directive::ToRadix {
        a: lhs.clone(),
        b: result.clone(),
        radix,
    }));

    evaluator.add_opcode(AcirOpcode::Arithmetic(subtract(lhs, FieldElement::one(), &bytes)));

    result
}

//Decomposition into b-base: \sum ai b^i, where 0<=ai<b
// radix: the base, (it is a constant, not a witness)
// num_limbs: the number of elements in the decomposition
// output: (the elements of the decomposition as witness, the sum expression)
pub(crate) fn to_radix<A: ACIRState>(
    radix: u32,
    num_limbs: u32,
    evaluator: &mut A,
) -> (Vec<Witness>, Expression) {
    let mut digits = Expression::default();
    let mut radix_pow = FieldElement::one();

    let shift = FieldElement::from(radix as i128);
    let mut result = Vec::new();
    let bit_size = bit_size_u32(radix);
    for _ in 0..num_limbs {
        let limb_witness = evaluator.add_witness_to_cs();
        result.push(limb_witness);
        let limb_expr = expression_from_witness(limb_witness);
        digits = add(&digits, radix_pow, &limb_expr);
        radix_pow = radix_pow.mul(shift);

        if 1_u128 << (bit_size - 1) != radix as u128 {
            try_range_constraint(limb_witness, bit_size, evaluator);
        }
        bound_constraint_with_offset(
            &expression_from_witness(limb_witness),
            &Expression::from_field(shift),
            &Expression::one(),
            bit_size,
            evaluator,
        );
    }

    (result, digits)
}

//Returns 1 if lhs < rhs
pub(crate) fn evaluate_cmp<A: ACIRState>(
    lhs: &Expression,
    rhs: &Expression,
    bit_size: u32,
    signed: bool,
    evaluator: &mut A,
) -> Expression {
    if signed {
        //TODO use range_constraints instead of bit decomposition, like in the unsigned case
        let mut sub_expr = subtract(lhs, FieldElement::one(), rhs);
        let two_pow = BigUint::one() << (bit_size + 1);
        sub_expr.q_c += FieldElement::from_be_bytes_reduce(&two_pow.to_bytes_be());
        let bits = to_radix_base(&sub_expr, 2, bit_size + 2, evaluator);
        expression_from_witness(bits[(bit_size - 1) as usize])
    } else {
        let is_greater = expression_from_witness(bound_check(lhs, rhs, bit_size, evaluator));
        subtract(&Expression::one(), FieldElement::one(), &is_greater)
    }
}

//truncate lhs (a number whose value requires max_bits) into a rhs-bits number: i.e it returns b such that lhs mod 2^rhs is b
pub(crate) fn evaluate_truncate<A: ACIRState>(
    lhs: &Expression,
    rhs: u32,
    max_bits: u32,
    evaluator: &mut A,
) -> Expression {
    assert!(max_bits > rhs, "max_bits = {max_bits}, rhs = {rhs}");

    //0. Check for constant expression. This can happen through arithmetic simplifications
    if let Some(a_c) = const_from_expression(lhs) {
        let mut a_big = BigUint::from_bytes_be(&a_c.to_be_bytes());
        let two = BigUint::from(2_u32);
        a_big %= two.pow(rhs);
        return Expression::from(&FieldElement::from_be_bytes_reduce(&a_big.to_bytes_be()));
    };

    //1. Generate witnesses a,b,c
    let b_witness = evaluator.add_witness_to_cs();
    let c_witness = evaluator.add_witness_to_cs();

    try_range_constraint(b_witness, rhs, evaluator); //TODO propagate the error using ?
    try_range_constraint(c_witness, max_bits - rhs, evaluator);

    //2. Add the constraint a = b+2^Nc
    let mut f = FieldElement::from(2_i128);
    f = f.pow(&FieldElement::from(rhs as i128));
    let b_arith = expression_from_witness(b_witness);
    let c_arith = expression_from_witness(c_witness);
    let res = add(&b_arith, f, &c_arith); //b+2^Nc
    let my_constraint = add(&res, -FieldElement::one(), lhs);
    evaluator.add_opcode(AcirOpcode::Directive(Directive::Truncate {
        a: lhs.clone(),
        b: b_witness,
        c: c_witness,
        bit_size: rhs,
    }));
    evaluator.add_opcode(AcirOpcode::Arithmetic(my_constraint));
    Expression::from(&b_witness)
}

pub(crate) fn evaluate_udiv<A: ACIRState>(
    lhs: &Expression,
    rhs: &Expression,
    bit_size: u32,
    predicate: &Expression,
    evaluator: &mut A,
) -> (Witness, Witness) {
    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();
    let pa = mul_with_witness(evaluator, lhs, predicate);
    evaluator.add_opcode(AcirOpcode::Directive(Directive::Quotient {
        a: lhs.clone(),
        b: rhs.clone(),
        q: q_witness,
        r: r_witness,
        predicate: Some(predicate.clone()),
    }));

    //r<b
    let r_expr = Expression::from(Linear::from_witness(r_witness));
    try_range_constraint(r_witness, bit_size, evaluator);
    bound_constraint_with_offset(&r_expr, rhs, predicate, bit_size, evaluator);
    //range check q<=a
    try_range_constraint(q_witness, bit_size, evaluator);
    // a-b*q-r = 0
    let mut d = mul_with_witness(evaluator, rhs, &Expression::from(&q_witness));
    d = add(&d, FieldElement::one(), &Expression::from(&r_witness));
    d = mul_with_witness(evaluator, &d, predicate);
    let div_euclidean = subtract(&pa, FieldElement::one(), &d);

    evaluator.add_opcode(AcirOpcode::Arithmetic(div_euclidean));
    (q_witness, r_witness)
}

/// Creates a new witness and constrains it to be the inverse of x
pub(crate) fn evaluate_inverse<A: ACIRState>(
    x_witness: Witness,
    predicate: &Expression,
    evaluator: &mut A,
) -> Witness {
    // Create a fresh witness - n.b we could check if x is constant or not
    let inverse_witness = evaluator.add_witness_to_cs();
    let inverse_expr = expression_from_witness(inverse_witness);
    evaluator.add_opcode(AcirOpcode::Directive(Directive::Invert {
        x: x_witness,
        result: inverse_witness,
    }));

    //x*inverse = 1
    let one = mul(&expression_from_witness(x_witness), &inverse_expr);
    let lhs = mul_with_witness(evaluator, &one, predicate);
    evaluator.add_opcode(AcirOpcode::Arithmetic(subtract(&lhs, FieldElement::one(), predicate)));
    inverse_witness
}

//Zero Equality gate: returns 1 if x is not null and 0 else
pub(crate) fn evaluate_zero_equality<A: ACIRState>(
    x_witness: Witness,
    evaluator: &mut A,
) -> Witness {
    let m = evaluator.add_witness_to_cs(); //'inverse' of x
    evaluator.add_opcode(AcirOpcode::Directive(Directive::Invert { x: x_witness, result: m }));

    //y=x*m         y is 1 if x is not null, and 0 else
    let y_witness = evaluator.add_witness_to_cs();
    evaluator.add_opcode(AcirOpcode::Arithmetic(Expression {
        mul_terms: vec![(FieldElement::one(), x_witness, m)],
        linear_combinations: vec![(-FieldElement::one(), y_witness)],
        q_c: FieldElement::zero(),
    }));

    //x=y*x
    let y_expr = expression_from_witness(y_witness);
    let xy = mul(&expression_from_witness(x_witness), &y_expr);
    evaluator.add_opcode(AcirOpcode::Arithmetic(subtract(
        &xy,
        FieldElement::one(),
        &expression_from_witness(x_witness),
    )));
    y_witness
}

// Given two lists, `A` and `B` of `Expression`s
// We generate constraints that A and B are equal
// An `Expression` is returned that indicates whether this
// was true.
//
// This method does not check the arrays length.
// We assume this has been checked by the caller.
pub(crate) fn arrays_eq_predicate<A: ACIRState>(
    a_values: &[Expression],
    b_values: &[Expression],
    evaluator: &mut A,
) -> Expression {
    let mut sum = Expression::default();

    for (a_iter, b_iter) in a_values.iter().zip(b_values) {
        let diff_expr = subtract(a_iter, FieldElement::one(), b_iter);

        let diff_witness = evaluator.add_witness_to_cs();

        evaluator.add_opcode(AcirOpcode::Arithmetic(subtract(
            &diff_expr,
            FieldElement::one(),
            &expression_from_witness(diff_witness),
        )));
        //TODO: avoid creating witnesses for diff
        sum = add(
            &sum,
            FieldElement::one(),
            &expression_from_witness(evaluate_zero_equality(diff_witness, evaluator)),
        );
    }
    sum
}

// TODO: An issue should be created for this
pub(crate) fn evaluate_sdiv<A: ACIRState>(
    _lhs: &Expression,
    _rhs: &Expression,
    _evaluator: &mut A,
) -> (Expression, Expression) {
    todo!();
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}

fn bit_size_u128(a: u128) -> u32 where {
    num_bits::<u128>() as u32 - a.leading_zeros()
}

fn bit_size_u32(a: u32) -> u32 where {
    num_bits::<u32>() as u32 - a.leading_zeros()
}
