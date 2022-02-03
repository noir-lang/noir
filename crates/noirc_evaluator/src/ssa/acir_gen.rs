use super::node::{Instruction, Operation};
use acvm::FieldElement;
use arena::Index;
use num_traits::ToPrimitive;
use std::collections::HashMap;
//use crate::acir::native_types::{Arithmetic, Witness};
use crate::ssa::{code_gen::IRGenerator, mem, node, node::Node};
use crate::Evaluator;
use crate::Gate;
use crate::RuntimeErrorKind;
use acvm::acir::circuit::gate::Directive;
use acvm::acir::native_types::{Arithmetic, Linear, Witness};
use num_bigint::BigUint;
use std::convert::TryInto;
pub struct Acir {
    pub arith_cache: HashMap<Index, InternalVar>,
    pub memory_map: HashMap<u32, InternalVar>, //maps memory adress to expression
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
        if self.idx.is_some() && b.idx.is_some() && self.idx.unwrap() == b.idx.unwrap() {
            return true;
        }
        if self.witness.is_some()
            && b.witness.is_some()
            && self.witness.unwrap() == b.witness.unwrap()
        {
            return true;
        }
        false //TODO should we check if content is the same??
    }

    pub fn to_const(&self) -> Option<FieldElement> {
        if self.expression.mul_terms.is_empty() && self.expression.linear_combinations.is_empty() {
            return Some(self.expression.q_c);
        }
        None
    }
}

impl Acir {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a nodeobj as an arithmetic expression
    fn substitute(
        arith_cache: &mut HashMap<Index, InternalVar>,
        idx: Index,
        evaluator: &mut Evaluator,
        cfg: &IRGenerator,
    ) -> InternalVar {
        if arith_cache.contains_key(&idx) {
            return arith_cache[&idx].clone();
        }
        let mut expr;
        let mut result: Option<InternalVar> = None;

        if let Some(obj) = cfg.get_object(idx) {
            match obj {
                node::NodeObj::Const(c) => {
                    let f_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()); //TODO const should be a field
                    expr = Arithmetic {
                        mul_terms: Vec::new(),
                        linear_combinations: Vec::new(),
                        q_c: f_value, //TODO handle other types
                    };
                    result = Some(InternalVar {
                        idx: Some(idx),
                        expression: expr,
                        witness: None,
                    });
                }
                node::NodeObj::Obj(v) => {
                    let w = if let Some(w1) = v.witness {
                        w1
                    } else {
                        evaluator.add_witness_to_cs()
                    };
                    expr = Arithmetic::from(&w);
                    result = Some(InternalVar {
                        idx: Some(idx),
                        expression: expr,
                        witness: Some(w),
                    });
                }
                _ => (),
            }
        }
        if result.is_none() {
            let w = evaluator.add_witness_to_cs();
            expr = Arithmetic::from(&w);
            result = Some(InternalVar {
                idx: Some(idx),
                expression: expr,
                witness: Some(w),
            });
        }
        arith_cache.insert(idx, result.unwrap());
        arith_cache[&idx].clone()
    }

    pub fn new() -> Acir {
        Acir {
            arith_cache: HashMap::new(),
            memory_map: HashMap::new(),
        }
    }

    pub fn evaluate_instruction(
        &mut self,
        ins: &Instruction,
        evaluator: &mut Evaluator,
        cfg: &IRGenerator,
    ) {
        if ins.operator == Operation::nop {
            return;
        }
        let mut output = Arithmetic::default();
        let l_c = Acir::substitute(&mut self.arith_cache, ins.lhs, evaluator, cfg);
        let r_c = Acir::substitute(&mut self.arith_cache, ins.rhs, evaluator, cfg);
        match ins.operator {
            Operation::add | Operation::sadd => {
                //output = &l_c.expression + &r_c.expression;
                output = add(&l_c.expression, FieldElement::one(), &r_c.expression);
                //  output.add(l_c.expression);
                //  output.add(r_c.expression);
            }
            Operation::sub | Operation::ssub => {
                //we need the type of rhs and its max value, then:
                //lhs-rhs+k*2^bit_size where k=ceil(max_value/2^bit_size)
                let bit_size = cfg.get_object(ins.rhs).unwrap().bits();
                assert!(bit_size < 128); //todo
                let r_mod = 1_u128 << bit_size;
                let k = (ins.max_value.to_f64().unwrap() / r_mod as f64).ceil() as i128;
                let mut f = FieldElement::from(k);
                f = f * FieldElement::from_be_bytes_reduce(&BigUint::from(r_mod).to_bytes_be());
                output = add(&l_c.expression, FieldElement::from(-1), &r_c.expression);
                output.q_c += f;
            }
            Operation::mul | Operation::smul => {
                output = evaluate_mul(&l_c, &r_c, evaluator);
            }
            Operation::udiv => {
                output = evaluate_udiv(&l_c, &r_c, evaluator);
            }
            Operation::sdiv => todo!(),
            Operation::urem => todo!(),
            Operation::srem => todo!(),
            Operation::fmod => todo!(),
            Operation::fneg => todo!(), //to check
            Operation::fdiv => todo!(),
            Operation::div => todo!(),
            Operation::eq => todo!(),
            Operation::ne => todo!(),
            Operation::ugt => todo!(),
            Operation::uge => todo!(),
            Operation::ult => todo!(),
            Operation::ule => todo!(),
            Operation::sgt => todo!(),
            Operation::sge => todo!(),
            Operation::slt => todo!(),
            Operation::sle => todo!(),
            Operation::lt => todo!(),
            Operation::gt => todo!(),
            Operation::lte => todo!(),
            Operation::gte => todo!(),
            Operation::and => todo!(),
            Operation::not => todo!(),
            Operation::or => todo!(),
            Operation::xor => todo!(),
            Operation::cast => {
                todo!()
            }
            Operation::ass | Operation::jne | Operation::jeq | Operation::jmp | Operation::phi => {
                todo!("invalid instruction");
            }
            Operation::trunc => {
                if is_const(&r_c.expression) {
                    output = evaluate_truncate(
                        l_c,
                        r_c.expression.q_c.to_u128().try_into().unwrap(),
                        ins.bit_size,
                        evaluator,
                    );
                } else {
                    todo!("Panic {:?}", r_c.expression);
                }
            }
            Operation::nop => (), //for now we skip..TODO todo!(),
            Operation::eq_gate => {
                dbg!(&l_c);
                output = add(&l_c.expression, FieldElement::from(-1), &r_c.expression);
                evaluator.gates.push(Gate::Arithmetic(output.clone())); //TODO should we create a witness??
            }
            Operation::load => {
                //retrieves the value from the map if address is known at compile time:
                //address = l_c and should be constant
                if let Some(val) = l_c.to_const() {
                    let address = mem::Memory::as_u32(val);
                    dbg!(address);
                    if self.memory_map.contains_key(&address) {
                        output = self.memory_map[&address].expression.clone();
                    } else {
                        //if not found, then it must be a witnes (else it is non-initialised memory)
                        let array = cfg.mem.get_array_adr(address);
                        let index = (address - array.adr) as usize;
                        let w = array.witness[index];
                        output = Arithmetic::from(Linear::from_witness(w));
                    }
                }
            }

            Operation::store => {
                //maps the address to the rhs if address is known at compile time
                if let Some(val) = r_c.to_const() {
                    let address = mem::Memory::as_u32(val);
                    self.memory_map.insert(address, l_c);
                    //we do not generate constraint, so no output.
                } else {
                    todo!();
                }
                
            }
        }

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
    let a_witness;
    //TODO: we should truncate the arithmetic expression (and so avoid having to create a witness)
    // if lhs is not a witness, but this requires a new truncate directive...TODO
    if lhs.witness.is_none() {
        a_witness = generate_witness(&lhs, evaluator);
    } else {
        a_witness = lhs.witness.unwrap();
    }

    let b_witness = evaluator.add_witness_to_cs();
    let c_witness = evaluator.add_witness_to_cs();
    //TODO not in master..
    evaluator.gates.push(Gate::Directive(Directive::Truncate {
        a: a_witness,
        b: b_witness,
        c: c_witness,
        bit_size: rhs,
    }));

    range_constraint(b_witness, rhs, evaluator);
    range_constraint(c_witness, max_bits - rhs, evaluator);

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
    if lhs.witness.is_some() {
        return lhs.witness.unwrap();
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
    let result;
    if is_const(&lhs.expression) {
        result = &(rhs.expression) * &(lhs.expression.q_c);
        return result;
    }
    if is_const(&rhs.expression) {
        result = &lhs.expression * &rhs.expression.q_c;
        return result;
    }
    //No multiplicative term
    if lhs.expression.mul_terms.is_empty() && rhs.expression.mul_terms.is_empty() {
        return mul(&lhs.expression, &rhs.expression);
    }
    //Generate intermediate variable
    //create new witness a and a gate: a = lhs
    let a = evaluator.add_witness_to_cs();
    let b;
    evaluator
        .gates
        .push(Gate::Arithmetic(&lhs.expression - &Arithmetic::from(&a)));
    //create new witness b et gate b = rhs
    if lhs.is_equal(rhs) {
        b = a;
    } else {
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
    let a_witness;
    //TODO: can we handle an arithmetic and not create a witness for a and b?
    if lhs.witness.is_none() {
        a_witness = generate_witness(lhs, evaluator); //TODO we should set lhs.witness = a.witness et lhs.expression= 1*w
    } else {
        a_witness = lhs.witness.unwrap();
    }
    let b_witness;
    //TODO: can we handle an arithmetic and not create a witness for a and b?
    if rhs.witness.is_none() {
        b_witness = generate_witness(rhs, evaluator);
    } else {
        b_witness = rhs.witness.unwrap();
    }
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
    range_constraint(q_witness, 32, evaluator); //todo bit size should be a.bits
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
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    let mut output = Arithmetic {
        mul_terms: Vec::new(),
        linear_combinations: Vec::new(),
        q_c: FieldElement::zero(),
    };

    //TODO a optimiser...
    while i1 < a.linear_combinations.len() {
        let single = single_mul(a.linear_combinations[i1].1, b);
        output = add(&output, a.linear_combinations[i1].0, &single);

        i1 += 1;
    }

    // while (i1< a.linear_combinations.len() && i2 < b.linear_combinations.len()) {
    //     let coef = a.linear_combinations[i1].0 * b.linear_combinations[i2].0;
    //     if a.linear_combinations[i1].1 < b.linear_combinations[i2].1 {
    //         output.mul_terms.push((coef, a.linear_combinations[i1].1, b.linear_combinations[i2].1));
    //     }
    //     else {
    //         output.mul_terms.push((coef, b.linear_combinations[i2].1, a.linear_combinations[i1].1));
    //     }
    //     next_mul_iter(&a.linear_combinations,&b.linear_combinations,&mut i1,&mut i2);
    //     dbg!(i1);
    //     dbg!(i2);
    // }

    //linear terms
    i1 = 0;
    //i2 = 0;
    //todo check it is correct
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        let coef_a = b.q_c * a.linear_combinations[i1].0;
        let coef_b = a.q_c * b.linear_combinations[i2].0;
        if a.linear_combinations[i1].1 < b.linear_combinations[i2].1 {
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
        } else if a.linear_combinations[i1].1 > b.linear_combinations[i2].1 {
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
        } else {
            if coef_a + coef_b != FieldElement::zero() {
                output
                    .linear_combinations
                    .push((coef_a + coef_b, a.linear_combinations[i1].1));
            }
            if (i1 + 1 >= a.linear_combinations.len()) && (i2 + 1 >= b.linear_combinations.len()) {
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
    //Constant term:
    output.q_c = a.q_c * b.q_c;
    //dbg!("mul result: {:?}", output.clone());
    output
}

// fn next_mul_iter(
//     a: &[(FieldElement, Witness)],
//     b: &[(FieldElement, Witness)],
//     i1: &mut usize,
//     i2: &mut usize,
// ) {
//     let mut next_a = None;
//     let mut next_b = None;
//     if *i1 + 1 >= a.len() && *i2 + 1 >= b.len() {
//         *i1 += 1;
//         *i2 += 1;
//         return;
//     }

//     if *i1 + 1 < a.len() {
//         if a[*i1 + 1].1 < b[*i2].1 {
//             next_a = Some((a[*i1 + 1].1, b[*i2].1));
//         } else {
//             next_a = Some((b[*i2].1, a[*i1 + 1].1));
//         }
//     }
//     if *i2 as usize + 1 < b.len() {
//         if a[*i1].1 < b[*i2 + 1].1 {
//             next_b = Some((a[*i1].1, b[*i2 + 1].1));
//         } else {
//             next_b = Some((b[*i2 + 1].1, a[*i1].1));
//         }
//     }

//     if next_a.is_none() {
//         *i2 += 1;
//         return;
//     }
//     if next_b.is_none() {
//         *i1 += 1;
//         return;
//     }
//     if next_a.unwrap() < next_b.unwrap() {
//         *i1 += 1;
//         return;
//     }
//     *i2 += 1;
// }

// returns a + k*b
pub fn add(a: &Arithmetic, k: FieldElement, b: &Arithmetic) -> Arithmetic {
    let mut output = Arithmetic::default();

    //linear combinations
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        if a.linear_combinations[i1].1 < b.linear_combinations[i2].1 {
            output.linear_combinations.push(a.linear_combinations[i1]);
            i1 += 1;
        } else if a.linear_combinations[i1].1 > b.linear_combinations[i2].1 {
            let coef = b.linear_combinations[i2].0 * k;
            if coef != FieldElement::zero() {
                output
                    .linear_combinations
                    .push((coef, b.linear_combinations[i2].1));
            }
            i2 += 1;
        } else {
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
        if (a.mul_terms[i1].1, a.mul_terms[i1].2) < (b.mul_terms[i2].1, b.mul_terms[i2].2) {
            output.mul_terms.push(a.mul_terms[i1]);
            i1 += 1;
        } else if (a.mul_terms[i1].1, a.mul_terms[i1].2) > (b.mul_terms[i2].1, b.mul_terms[i2].2) {
            let coef = b.mul_terms[i2].0 * k;
            if coef != FieldElement::zero() {
                output
                    .mul_terms
                    .push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
            }
            i2 += 1;
        } else {
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
    } else {
        // Note if the number of bits is odd, then Barretenberg will panic
        if num_bits % 2 == 1 {
            // new witnesses; r is constrained to num_bits-1 and b is 1 bit
            let r_witness = evaluator.add_witness_to_cs();
            let b_witness = evaluator.add_witness_to_cs();
            //TODO not in master...
            // evaluator.gates.push(Gate::Directive(Directive::Oddrange {
            //     a: witness,
            //     b: b_witness,
            //     r: r_witness,
            //     bit_size: num_bits,
            // }));
            range_constraint(r_witness, num_bits - 1, evaluator);
            range_constraint(b_witness, 1, evaluator);
            //Add the constraint a = r + 2^N*b
            let mut f = FieldElement::from(2_i128);
            f = f.pow(&FieldElement::from((num_bits - 1) as i128));
            let res = add(&from_witness(r_witness), f, &from_witness(b_witness));
            let my_constraint = add(&res, -FieldElement::one(), &from_witness(witness));
            evaluator.gates.push(Gate::Arithmetic(my_constraint));
        } else {
            evaluator.gates.push(Gate::Range(witness, num_bits));
        }
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
// n.b: we de NOT check here that a and b are indeed 'bits' size
// a < b <=> a+1<=b
fn bound_check(
    a: &InternalVar,
    b: &InternalVar,
    strict: bool,
    bits: u32,
    evaluator: &mut Evaluator,
)
//-> Result<RuntimeErrorKind>
{
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
                                  //range_check requires a witness - TODO may be this can be avoided?
    let w = evaluator.add_witness_to_cs();
    evaluator
        .gates
        .push(Gate::Arithmetic(&sub_expression - &Arithmetic::from(&w)));
    range_constraint(w, bits, evaluator);
    //Ok()
}

pub fn from_witness(witness: Witness) -> Arithmetic {
    Arithmetic {
        mul_terms: Vec::new(),
        linear_combinations: vec![(FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}
