use super::node::{Instruction, Operation};
use acvm::FieldElement;
use arena::Index;
use num_traits::One;
use num_traits::Zero;
use std::cmp::Ordering;
use std::collections::HashMap;
//use crate::acir::native_types::{Arithmetic, Witness};
use crate::ssa::code_gen::IRGenerator;
use crate::ssa::node;
use crate::ssa::node::Node;
use crate::Evaluator;
use crate::Gate;
use crate::RuntimeErrorKind;
use acvm::acir::circuit::gate::Directive;
use acvm::acir::native_types::{Arithmetic, Linear, Witness};
use num_bigint::BigUint;
use std::convert::TryInto;
pub struct Acir {
    pub arith_cache: HashMap<Index, InternalVar>,
}

#[derive(Clone, Debug)]
pub struct InternalVar {
    expression: Arithmetic,
    //value: FieldElement,     //not used for now
    witness: Option<Witness>,
    idx: Option<Index>,
}
impl InternalVar {
    pub fn is_equal(&self, b: &InternalVar) -> bool {
        (self.idx.is_some() && self.idx == b.idx)
            || (self.witness.is_some() && self.witness == b.witness)
            || self.expression == b.expression
    }

    fn new(expression: Arithmetic, witness: Option<Witness>, id: Index) -> InternalVar {
        InternalVar {
            expression,
            witness,
            idx: Some(id),
        }
    }
}

impl Acir {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a nodeobj as an arithmetic expression
    fn substitute(
        &mut self,
        idx: Index,
        evaluator: &mut Evaluator,
        cfg: &IRGenerator,
    ) -> InternalVar {
        if self.arith_cache.contains_key(&idx) {
            return self.arith_cache[&idx].clone();
        }
        let var = match cfg.get_object(idx) {
            Some(node::NodeObj::Const(c)) => {
                let f_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()); //TODO const should be a field
                let expr = Arithmetic {
                    mul_terms: Vec::new(),
                    linear_combinations: Vec::new(),
                    q_c: f_value, //TODO handle other types
                };
                InternalVar::new(expr, None, idx)
            }
            Some(node::NodeObj::Obj(v)) => {
                let w = if let Some(w1) = v.witness {
                    w1
                } else {
                    evaluator.add_witness_to_cs()
                };
                let expr = Arithmetic::from(&w);
                InternalVar::new(expr, Some(w), idx)
            }
            _ => {
                let w = evaluator.add_witness_to_cs();
                let expr = Arithmetic::from(&w);
                InternalVar::new(expr, Some(w), idx)
            }
        };
        self.arith_cache.insert(idx, var);
        self.arith_cache[&idx].clone()
    }

    pub fn new() -> Acir {
        Acir {
            arith_cache: HashMap::new(),
        }
    }

    pub fn evaluate_instruction(
        &mut self,
        ins: &Instruction,
        evaluator: &mut Evaluator,
        cfg: &IRGenerator,
    ) {
        if ins.operator == Operation::Nop {
            return;
        }
        let l_c = self.substitute(ins.lhs, evaluator, cfg);
        let r_c = self.substitute(ins.rhs, evaluator, cfg);
        let output = match ins.operator {
            Operation::Add | Operation::SafeAdd => {
                add(&l_c.expression, FieldElement::one(), &r_c.expression)
            }
            Operation::Sub | Operation::SafeSub => {
                //we need the type of rhs and its max value, then:
                //lhs-rhs+k*2^bit_size where k=ceil(max_value/2^bit_size)
                let bit_size = cfg.get_object(ins.rhs).unwrap().bits();
                let r_big = BigUint::one() << bit_size;
                let mut k = &ins.max_value / &r_big;
                if &ins.max_value % &r_big != BigUint::zero() {
                    k = &k + BigUint::one();
                }
                k = &k * r_big;
                let f = FieldElement::from_be_bytes_reduce(&k.to_bytes_be());
                let mut output = add(
                    &l_c.expression,
                    FieldElement::from(-1_i128),
                    &r_c.expression,
                );
                output.q_c += f;
                output
            }
            Operation::Mul | Operation::SafeMul => evaluate_mul(&l_c, &r_c, evaluator),
            Operation::Udiv => evaluate_udiv(&l_c, &r_c, evaluator),
            Operation::Sdiv => evaluate_sdiv(&l_c, &r_c, evaluator),
            Operation::Urem => todo!(),
            Operation::Srem => todo!(),
            Operation::Div => todo!(),
            Operation::Eq => todo!(),
            Operation::Ne => todo!(),
            Operation::Ugt => todo!(),
            Operation::Uge => todo!(),
            Operation::Ult => todo!(),
            Operation::Ule => todo!(),
            Operation::Sgt => todo!(),
            Operation::Sge => todo!(),
            Operation::Slt => todo!(),
            Operation::Sle => todo!(),
            Operation::Lt => todo!(),
            Operation::Gt => todo!(),
            Operation::Lte => todo!(),
            Operation::Gte => todo!(),
            Operation::And => {
                todo!();
                //use AndGate
            }
            Operation::Not => todo!(),
            Operation::Or => todo!(),
            Operation::Xor => todo!(),
            Operation::Cast => l_c.expression,
            Operation::Ass | Operation::Jne | Operation::Jeq | Operation::Jmp | Operation::Phi => {
                todo!("invalid instruction");
            }
            Operation::Trunc => {
                assert!(is_const(&r_c.expression));
                evaluate_truncate(
                    l_c,
                    r_c.expression.q_c.to_u128().try_into().unwrap(),
                    ins.bit_size,
                    evaluator,
                )
            }
            Operation::Nop => Arithmetic::default(),
            Operation::EqGate => {
                let output = add(
                    &l_c.expression,
                    FieldElement::from(-1_i128),
                    &r_c.expression,
                );
                evaluator.gates.push(Gate::Arithmetic(output.clone())); //TODO should we create a witness??
                output
            }
        };

        let output_var = InternalVar {
            expression: output,
            //value: FieldElement::from(0_u32),
            idx: Some(ins.idx),
            witness: None, //TODO put the witness when it exist
        };

        self.arith_cache.insert(ins.idx, output_var);
    }
}

//truncate lhs (a number whose value requires max_bits) into a rhs-bits number: i.e it returns b such that lhs mod 2^rhs is b
pub fn evaluate_truncate(
    lhs: InternalVar,
    rhs: u32,
    max_bits: u32,
    evaluator: &mut Evaluator,
) -> Arithmetic {
    assert!(max_bits > rhs);
    //1. Generate witnesses a,b,c
    //TODO: we should truncate the arithmetic expression (and so avoid having to create a witness)
    // if lhs is not a witness, but this requires a new truncate directive...TODO
    let a_witness = lhs
        .witness
        .unwrap_or_else(|| generate_witness(&lhs, evaluator));
    let b_witness = evaluator.add_witness_to_cs();
    let c_witness = evaluator.add_witness_to_cs();
    //TODO not in master..
    evaluator.gates.push(Gate::Directive(Directive::Truncate {
        a: a_witness,
        b: b_witness,
        c: c_witness,
        bit_size: rhs,
    }));

    range_constraint(b_witness, rhs, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    }); //TODO propagate the error using ?
    range_constraint(c_witness, max_bits - rhs, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    });

    //2. Add the constraint a = b+2^Nc
    let mut f = FieldElement::from(2_i128);
    f = f.pow(&FieldElement::from(rhs as i128));
    let b_arith = from_witness(b_witness);
    let c_arith = from_witness(c_witness);
    let res = add(&b_arith, f, &c_arith); //b+2^Nc
    let a = &Arithmetic::from(Linear::from_witness(a_witness));
    let my_constraint = add(&res, -FieldElement::one(), a);
    evaluator.gates.push(Gate::Arithmetic(my_constraint));

    Arithmetic::from(Linear::from_witness(b_witness))
}

pub fn generate_witness(lhs: &InternalVar, evaluator: &mut Evaluator) -> Witness {
    if let Some(witness) = lhs.witness {
        return witness;
    }

    if is_const(&lhs.expression) {
        todo!("Panic");
    }
    if lhs.expression.mul_terms.is_empty() && lhs.expression.linear_combinations.len() == 1 {
        todo!("optimisation ??!?");
    }
    let w = evaluator.add_witness_to_cs(); //TODO  set lhs.witness = w
    evaluator
        .gates
        .push(Gate::Arithmetic(&lhs.expression - &Arithmetic::from(&w)));
    w
}

pub fn evaluate_mul(lhs: &InternalVar, rhs: &InternalVar, evaluator: &mut Evaluator) -> Arithmetic {
    if is_const(&lhs.expression) {
        return &rhs.expression * &lhs.expression.q_c;
    }
    if is_const(&rhs.expression) {
        return &lhs.expression * &rhs.expression.q_c;
    }
    //No multiplicative term
    if lhs.expression.mul_terms.is_empty() && rhs.expression.mul_terms.is_empty() {
        return mul(&lhs.expression, &rhs.expression);
    }
    //Generate intermediate variable
    //create new witness a and a gate: a = lhs
    let a = evaluator.add_witness_to_cs();
    evaluator
        .gates
        .push(Gate::Arithmetic(&lhs.expression - &Arithmetic::from(&a)));
    //create new witness b and gate b = rhs
    let mut b = a;
    if !lhs.is_equal(rhs) {
        b = evaluator.add_witness_to_cs();
        evaluator
            .gates
            .push(Gate::Arithmetic(&rhs.expression - &Arithmetic::from(&b)));
    }

    //return arith(mul=a*b)
    mul(&Arithmetic::from(&a), &Arithmetic::from(&b)) //TODO  &lhs.expression * &rhs.expression
}

pub fn evaluate_udiv(
    lhs: &InternalVar,
    rhs: &InternalVar,
    evaluator: &mut Evaluator,
) -> Arithmetic {
    //a = q*b+r, a= lhs, et b= rhs
    //result = q
    //n.b a et b MUST have proper bit size
    //we need to know a bit size (so that q has the same)
    //generate witnesses

    //TODO: can we handle an arithmetic and not create a witness for a and b?
    let a_witness = if let Some(lhs_witness) = lhs.witness {
        lhs_witness
    } else {
        generate_witness(lhs, evaluator) //TODO we should set lhs.witness = a.witness and lhs.expression= 1*w
    };

    //TODO: can we handle an arithmetic and not create a witness for a and b?
    let b_witness = if let Some(rhs_witness) = rhs.witness {
        rhs_witness
    } else {
        generate_witness(rhs, evaluator)
    };
    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();

    //TODO not in master...
    evaluator.gates.push(Gate::Directive(Directive::Quotient {
        a: a_witness,
        b: b_witness,
        q: q_witness,
        r: r_witness,
    }));
    //r<b
    let r_expr = Arithmetic::from(Linear::from_witness(r_witness));
    let r_var = InternalVar {
        expression: r_expr,
        witness: Some(r_witness),
        idx: None,
    };
    bound_check(&r_var, rhs, true, 32, evaluator); //TODO bit size! should be max(a.bit, b.bit)
                                                   //range check q<=a
    range_constraint(q_witness, 32, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    }); //todo bit size should be a.bits
        // a-b*q-r = 0
    let div_eucl = add(
        &lhs.expression,
        -FieldElement::one(),
        &Arithmetic {
            mul_terms: vec![(FieldElement::one(), b_witness, q_witness)],
            linear_combinations: vec![(FieldElement::one(), r_witness)],
            q_c: FieldElement::zero(),
        },
    );

    evaluator.gates.push(Gate::Arithmetic(div_eucl));
    Arithmetic::from(Linear::from_witness(q_witness)) //todo witness, arith, var??
}

pub fn evaluate_sdiv(
    lhs: &InternalVar,
    rhs: &InternalVar,
    evaluator: &mut Evaluator,
) -> Arithmetic {
    //TODO
    evaluate_udiv(lhs, rhs, evaluator);
    todo!();
}

pub fn is_const(expr: &Arithmetic) -> bool {
    expr.mul_terms.is_empty() && expr.linear_combinations.is_empty()
}

//a*b
pub fn mul(a: &Arithmetic, b: &Arithmetic) -> Arithmetic {
    if !(a.mul_terms.is_empty() && b.mul_terms.is_empty()) {
        todo!("PANIC");
    }

    let mut output = Arithmetic {
        mul_terms: Vec::new(),
        linear_combinations: Vec::new(),
        q_c: FieldElement::zero(),
    };

    //TODO to optimise...
    for lc in &a.linear_combinations {
        let single = single_mul(lc.1, b);
        output = add(&output, lc.0, &single);
    }

    //linear terms
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        let coef_a = b.q_c * a.linear_combinations[i1].0;
        let coef_b = a.q_c * b.linear_combinations[i2].0;
        match a.linear_combinations[i1]
            .1
            .cmp(&b.linear_combinations[i2].1)
        {
            Ordering::Greater => {
                if coef_b != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef_b, b.linear_combinations[i2].1));
                }
                if i2 + 1 >= b.linear_combinations.len() {
                    i1 += 1;
                } else {
                    i2 += 1;
                }
            }
            Ordering::Less => {
                if coef_a != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef_a, a.linear_combinations[i1].1));
                }
                if i1 + 1 >= a.linear_combinations.len() {
                    i2 += 1;
                } else {
                    i1 += 1;
                }
            }
            Ordering::Equal => {
                if coef_a + coef_b != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef_a + coef_b, a.linear_combinations[i1].1));
                }
                if (i1 + 1 >= a.linear_combinations.len())
                    && (i2 + 1 >= b.linear_combinations.len())
                {
                    i1 += 1;
                    i2 += 1;
                } else {
                    if i1 + 1 < a.linear_combinations.len() {
                        i1 += 1;
                    }
                    if i2 + 1 < a.linear_combinations.len() {
                        i2 += 1;
                    }
                }
            }
        }
    }
    //Constant term:
    output.q_c = a.q_c * b.q_c;
    output
}

// returns a + k*b
pub fn add(a: &Arithmetic, k: FieldElement, b: &Arithmetic) -> Arithmetic {
    let mut output = Arithmetic::default();

    //linear combinations
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        match a.linear_combinations[i1]
            .1
            .cmp(&b.linear_combinations[i2].1)
        {
            Ordering::Greater => {
                let coef = b.linear_combinations[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef, b.linear_combinations[i2].1));
                }
                i2 += 1;
            }
            Ordering::Less => {
                output.linear_combinations.push(a.linear_combinations[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                let coef = a.linear_combinations[i1].0 + b.linear_combinations[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .linear_combinations
                        .push((coef, a.linear_combinations[i1].1));
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
        let coef = b.linear_combinations[i2].0 * k;
        if coef != FieldElement::zero() {
            output
                .linear_combinations
                .push((coef, b.linear_combinations[i2].1));
        }
        i2 += 1;
    }

    //mul terms

    i1 = 0; //a
    i2 = 0; //b

    while i1 < a.mul_terms.len() && i2 < b.mul_terms.len() {
        match (a.mul_terms[i1].1, a.mul_terms[i1].2).cmp(&(b.mul_terms[i2].1, b.mul_terms[i2].2)) {
            Ordering::Greater => {
                let coef = b.mul_terms[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .mul_terms
                        .push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
                }
                i2 += 1;
            }
            Ordering::Less => {
                output.mul_terms.push(a.mul_terms[i1]);
                i1 += 1;
            }
            Ordering::Equal => {
                let coef = a.mul_terms[i1].0 + b.mul_terms[i2].0 * k;
                if coef != FieldElement::zero() {
                    output
                        .mul_terms
                        .push((coef, a.mul_terms[i1].1, a.mul_terms[i1].2));
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
        let coef = b.mul_terms[i2].0 * k;
        if coef != FieldElement::zero() {
            output
                .mul_terms
                .push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
        }
        i2 += 1;
    }

    output.q_c = a.q_c + k * b.q_c;
    output
}

// returns w*b.linear_combinations
pub fn single_mul(w: Witness, b: &Arithmetic) -> Arithmetic {
    let mut output = Arithmetic::default();
    let mut i1 = 0;
    while i1 < b.linear_combinations.len() {
        if (w, b.linear_combinations[i1].1) < (b.linear_combinations[i1].1, w) {
            output
                .mul_terms
                .push((b.linear_combinations[i1].0, w, b.linear_combinations[i1].1));
        } else {
            output
                .mul_terms
                .push((b.linear_combinations[i1].0, b.linear_combinations[i1].1, w));
        }
        i1 += 1;
    }
    output
}

pub fn boolean(witness: Witness) -> Arithmetic {
    Arithmetic {
        mul_terms: vec![(FieldElement::one(), witness, witness)],
        linear_combinations: vec![(-FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}

//contrain witness a to be num_bits-size integer, i.e between 0 and 2^num_bits-1
pub fn range_constraint(
    witness: Witness,
    num_bits: u32,
    evaluator: &mut Evaluator,
) -> Result<(), RuntimeErrorKind> {
    if num_bits == 1 {
        // Add a bool gate
        let bool_constraint = boolean(witness);
        evaluator.gates.push(Gate::Arithmetic(bool_constraint));
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
        evaluator.gates.push(Gate::Directive(Directive::Oddrange {
            a: witness,
            b: b_witness,
            r: r_witness,
            bit_size: num_bits,
        }));
        range_constraint(r_witness, num_bits - 1, evaluator).unwrap_or_else(|err| {
            dbg!(err);
        });
        range_constraint(b_witness, 1, evaluator).unwrap_or_else(|err| {
            dbg!(err);
        });
        //Add the constraint a = r + 2^N*b
        let mut f = FieldElement::from(2_i128);
        f = f.pow(&FieldElement::from((num_bits - 1) as i128));
        let res = add(&from_witness(r_witness), f, &from_witness(b_witness));
        let my_constraint = add(&res, -FieldElement::one(), &from_witness(witness));
        evaluator.gates.push(Gate::Arithmetic(my_constraint));
    } else {
        evaluator.gates.push(Gate::Range(witness, num_bits));
    }

    Ok(())
}

// Generate constraints that are satisfied iff
// a < b , when strict is true, or
// a <= b, when strict is false
// bits is the bit size of a and b (or an upper bound of the bit size)
///////////////
// a<=b is done by constraining b-a to a bit size of 'bits':
// if a<=b, 0 <= b-a <= b < 2^bits
// if a>b, b-a = p+b-a > p-2^bits >= 2^bits  (if log(p) >= bits + 1)
// n.b: we do NOT check here that a and b are indeed 'bits' size
// a < b <=> a+1<=b
fn bound_check(
    a: &InternalVar,
    b: &InternalVar,
    strict: bool,
    bits: u32,
    evaluator: &mut Evaluator,
) {
    //todo appeler bound_constrains et rajouter les gates a l'evaluator
    if bits > FieldElement::max_num_bits() - 1
    //TODO max_num_bits() is not log(p)?
    {
        todo!("ERROR");
    }
    let offset = if strict {
        FieldElement::one()
    } else {
        FieldElement::zero()
    };
    let mut sub_expression = add(&b.expression, -FieldElement::one(), &a.expression); //a-b
    sub_expression.q_c += offset; //a-b+offset
    let w = evaluator.add_witness_to_cs(); //range_check requires a witness - TODO may be this can be avoided?
    evaluator
        .gates
        .push(Gate::Arithmetic(&sub_expression - &Arithmetic::from(&w)));
    range_constraint(w, bits, evaluator).unwrap_or_else(|err| {
        dbg!(err);
    });
}

pub fn from_witness(witness: Witness) -> Arithmetic {
    Arithmetic {
        mul_terms: Vec::new(),
        linear_combinations: vec![(FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}
