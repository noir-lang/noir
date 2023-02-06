use super::builtin::{self, Opcode};
use super::mem::{ArrayId, MemArray, Memory};
use super::node::{BinaryOp, Instruction, ObjectType, Operation};
use acvm::FieldElement;

use super::node::NodeId;

use num_traits::{One, Zero};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Mul, Neg};
//use crate::acir::native_types::{Arithmetic, Witness};
use crate::ssa::context::SsaContext;
use crate::ssa::node::Node;
use crate::ssa::{mem, node};
use crate::Evaluator;
use crate::RuntimeErrorKind;
use acvm::acir::circuit::directives::Directive;
use acvm::acir::circuit::opcodes::{BlackBoxFuncCall, FunctionInput};

use acvm::acir::circuit::Opcode as AcirOpcode;
use acvm::acir::native_types::{Expression, Linear, Witness};
use num_bigint::BigUint;

#[derive(Default)]
pub struct Acir {
    pub arith_cache: HashMap<NodeId, InternalVar>,
    pub memory_map: HashMap<u32, InternalVar>, //maps memory adress to expression
}

#[derive(Default, Clone, Debug)]
pub struct InternalVar {
    expression: Expression,
    //value: FieldElement,     //not used for now
    witness: Option<Witness>,
    id: Option<NodeId>,
}

impl InternalVar {
    fn new(expression: Expression, witness: Option<Witness>, id: NodeId) -> InternalVar {
        InternalVar { expression, witness, id: Some(id) }
    }

    pub fn to_const(&self) -> Option<FieldElement> {
        if self.expression.mul_terms.is_empty() && self.expression.linear_combinations.is_empty() {
            return Some(self.expression.q_c);
        }
        None
    }

    pub fn generate_witness(&mut self, evaluator: &mut Evaluator) -> Witness {
        if let Some(witness) = self.witness {
            return witness;
        }

        if self.expression.is_const() {
            todo!("Panic");
        }
        let witness = InternalVar::expression_to_witness(self.expression.clone(), evaluator);
        self.witness = Some(witness);
        witness
    }

    pub fn expression_to_witness(expr: Expression, evaluator: &mut Evaluator) -> Witness {
        if expr.mul_terms.is_empty()
            && expr.linear_combinations.len() == 1
            && expr.q_c == FieldElement::zero()
            && expr.linear_combinations[0].0 == FieldElement::one()
        {
            return expr.linear_combinations[0].1;
        }
        evaluator.create_intermediate_variable(expr)
    }
}

impl From<Expression> for InternalVar {
    fn from(arith: Expression) -> InternalVar {
        let w = is_unit(&arith);
        InternalVar { expression: arith, witness: w, id: None }
    }
}

impl From<Witness> for InternalVar {
    fn from(w: Witness) -> InternalVar {
        InternalVar { expression: from_witness(w), witness: Some(w), id: None }
    }
}

impl From<FieldElement> for InternalVar {
    fn from(f: FieldElement) -> InternalVar {
        InternalVar { expression: Expression::from_field(f), witness: None, id: None }
    }
}

impl Acir {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a nodeobj as an arithmetic expression
    fn substitute(
        &mut self,
        id: NodeId,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        if self.arith_cache.contains_key(&id) {
            return self.arith_cache[&id].clone();
        }
        let var = match ctx.try_get_node(id) {
            Some(node::NodeObj::Const(c)) => {
                let f_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be()); //TODO const should be a field
                let expr = Expression::from_field(f_value);
                InternalVar::new(expr, None, id)
            }
            Some(node::NodeObj::Obj(v)) => match v.get_type() {
                node::ObjectType::Pointer(_) => InternalVar::default(),
                _ => {
                    let w = v.witness.unwrap_or_else(|| evaluator.add_witness_to_cs());
                    let expr = Expression::from(&w);
                    InternalVar::new(expr, Some(w), id)
                }
            },
            _ => {
                let w = evaluator.add_witness_to_cs();
                let expr = Expression::from(&w);
                InternalVar::new(expr, Some(w), id)
            }
        };

        self.arith_cache.insert(id, var.clone());
        var
    }

    pub fn evaluate_instruction(
        &mut self,
        ins: &Instruction,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> Result<(), RuntimeErrorKind> {
        if ins.operation == Operation::Nop {
            return Ok(());
        }

        let mut output = match &ins.operation {
            Operation::Binary(binary) => self.evaluate_binary(binary, ins.res_type, evaluator, ctx),
            Operation::Constrain(value, ..) => {
                let value = self.substitute(*value, evaluator, ctx);
                let subtract = subtract(&Expression::one(), FieldElement::one(), &value.expression);
                evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract));
                value
            }
            Operation::Not(value) => {
                let a = (1_u128 << ins.res_type.bits()) - 1;
                let l_c = self.substitute(*value, evaluator, ctx);
                subtract(
                    &Expression {
                        mul_terms: Vec::new(),
                        linear_combinations: Vec::new(),
                        q_c: FieldElement::from(a),
                    },
                    FieldElement::one(),
                    &l_c.expression,
                )
                .into()
            }
            Operation::Cast(value) => self.substitute(*value, evaluator, ctx),
            i @ Operation::Jne(..)
            | i @ Operation::Jeq(..)
            | i @ Operation::Jmp(_)
            | i @ Operation::Phi { .. }
            | i @ Operation::Result { .. } => {
                unreachable!("Invalid instruction: {:?}", i);
            }
            Operation::Truncate { value, bit_size, max_bit_size } => {
                let value = self.substitute(*value, evaluator, ctx);
                evaluate_truncate(value, *bit_size, *max_bit_size, evaluator)
            }
            Operation::Intrinsic(opcode, args) => {
                let v = self.evaluate_opcode(ins.id, *opcode, args, ins.res_type, ctx, evaluator);
                InternalVar::from(v)
            }
            Operation::Call { .. } => unreachable!("call instruction should have been inlined"),
            Operation::Return(_) => todo!(), //return from main
            Operation::Cond { condition, val_true: lhs, val_false: rhs } => {
                let cond = self.substitute(*condition, evaluator, ctx);
                let l_c = self.substitute(*lhs, evaluator, ctx);
                let r_c = self.substitute(*rhs, evaluator, ctx);
                let sub = subtract(&l_c.expression, FieldElement::one(), &r_c.expression);
                let result = add(
                    &mul_with_witness(evaluator, &cond.expression, &sub),
                    FieldElement::one(),
                    &r_c.expression,
                );
                result.into()
            }
            Operation::Nop => InternalVar::default(),
            Operation::Load { array_id, index } => {
                //retrieves the value from the map if address is known at compile time:
                //address = l_c and should be constant
                let index = self.substitute(*index, evaluator, ctx);
                if let Some(index) = index.to_const() {
                    let idx = mem::Memory::as_u32(index);
                    let mem_array = &ctx.mem[*array_id];
                    let absolute_adr = mem_array.absolute_adr(idx);
                    if self.memory_map.contains_key(&absolute_adr) {
                        InternalVar::from(self.memory_map[&absolute_adr].expression.clone())
                    } else {
                        //if not found, then it must be a witness (else it is non-initialised memory)
                        let index = idx as usize;
                        if mem_array.values.len() > index {
                            mem_array.values[index].clone()
                        } else {
                            unreachable!("Could not find value at index {}", index);
                        }
                    }
                } else {
                    unimplemented!("dynamic arrays are not implemented yet");
                }
            }

            Operation::Store { array_id, index, value } => {
                //maps the address to the rhs if address is known at compile time
                let index = self.substitute(*index, evaluator, ctx);
                let value = self.substitute(*value, evaluator, ctx);

                if let Some(index) = index.to_const() {
                    let idx = mem::Memory::as_u32(index);
                    let absolute_adr = ctx.mem[*array_id].absolute_adr(idx);
                    self.memory_map.insert(absolute_adr, value);
                    //we do not generate constraint, so no output.
                    InternalVar::default()
                } else {
                    todo!("dynamic arrays are not implemented yet");
                }
            }
        };
        output.id = Some(ins.id);
        self.arith_cache.insert(ins.id, output);
        Ok(())
    }

    fn get_predicate(
        &mut self,
        binary: &node::Binary,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        if let Some(pred) = binary.predicate {
            self.substitute(pred, evaluator, ctx)
        } else {
            InternalVar::from(Expression::one())
        }
    }

    fn evaluate_binary(
        &mut self,
        binary: &node::Binary,
        res_type: ObjectType,
        evaluator: &mut Evaluator,
        ctx: &SsaContext,
    ) -> InternalVar {
        let l_c = self.substitute(binary.lhs, evaluator, ctx);
        let r_c = self.substitute(binary.rhs, evaluator, ctx);
        let r_size = ctx[binary.rhs].size_in_bits();
        let l_size = ctx[binary.lhs].size_in_bits();
        let max_size = u32::max(r_size, l_size);

        match &binary.operator {
            BinaryOp::Add | BinaryOp::SafeAdd => {
                InternalVar::from(add(&l_c.expression, FieldElement::one(), &r_c.expression))
            }
            BinaryOp::Sub { max_rhs_value } | BinaryOp::SafeSub { max_rhs_value } => {
                if res_type == node::ObjectType::NativeField {
                    InternalVar::from(subtract(
                        &l_c.expression,
                        FieldElement::one(),
                        &r_c.expression,
                    ))
                } else {
                    //we need the type of rhs and its max value, then:
                    //lhs-rhs+k*2^bit_size where k=ceil(max_value/2^bit_size)
                    let bit_size = r_size;
                    let r_big = BigUint::one() << bit_size;
                    let mut k = max_rhs_value / &r_big;
                    if max_rhs_value % &r_big != BigUint::zero() {
                        k = &k + BigUint::one();
                    }
                    k = &k * r_big;
                    let f = FieldElement::from_be_bytes_reduce(&k.to_bytes_be());
                    let mut sub_expr =
                        subtract(&l_c.expression, FieldElement::one(), &r_c.expression);
                    sub_expr.q_c += f;
                    let mut sub_var = sub_expr.into();
                    //TODO: uses interval analysis for more precise check
                    if let Some(lhs_const) = l_c.to_const() {
                        if max_rhs_value <= &BigUint::from_bytes_be(&lhs_const.to_be_bytes()) {
                            sub_var = InternalVar::from(subtract(
                                &l_c.expression,
                                FieldElement::one(),
                                &r_c.expression,
                            ));
                        }
                    }
                    sub_var
                }
            }
            BinaryOp::Mul | BinaryOp::SafeMul => {
                InternalVar::from(mul_with_witness(evaluator, &l_c.expression, &r_c.expression))
            }
            BinaryOp::Udiv => {
                let predicate = self.get_predicate(binary, evaluator, ctx);
                let (q_wit, _) = evaluate_udiv(&l_c, &r_c, max_size, &predicate, evaluator);
                InternalVar::from(q_wit)
            }
            BinaryOp::Sdiv => InternalVar::from(evaluate_sdiv(&l_c, &r_c, evaluator).0),
            BinaryOp::Urem => {
                let predicate = self.get_predicate(binary, evaluator, ctx);
                let (_, r_wit) = evaluate_udiv(&l_c, &r_c, max_size, &predicate, evaluator);
                InternalVar::from(r_wit)
            }
            BinaryOp::Srem => InternalVar::from(evaluate_sdiv(&l_c, &r_c, evaluator).1),
            BinaryOp::Div => {
                let predicate = self.get_predicate(binary, evaluator, ctx);
                let inverse = from_witness(evaluate_inverse(r_c, &predicate, evaluator));
                InternalVar::from(mul_with_witness(evaluator, &l_c.expression, &inverse))
            }
            BinaryOp::Eq => InternalVar::from(
                self.evaluate_eq(binary.lhs, binary.rhs, &l_c, &r_c, ctx, evaluator),
            ),
            BinaryOp::Ne => InternalVar::from(
                self.evaluate_neq(binary.lhs, binary.rhs, &l_c, &r_c, ctx, evaluator),
            ),
            BinaryOp::Ult => {
                let size = ctx[binary.lhs].get_type().bits();
                evaluate_cmp(&l_c, &r_c, size, false, evaluator).into()
            }
            BinaryOp::Ule => {
                let size = ctx[binary.lhs].get_type().bits();
                let e = evaluate_cmp(&r_c, &l_c, size, false, evaluator);
                subtract(&Expression::one(), FieldElement::one(), &e).into()
            }
            BinaryOp::Slt => {
                let s = ctx[binary.lhs].get_type().bits();
                evaluate_cmp(&l_c, &r_c, s, true, evaluator).into()
            }
            BinaryOp::Sle => {
                let s = ctx[binary.lhs].get_type().bits();
                let e = evaluate_cmp(&r_c, &l_c, s, true, evaluator);
                subtract(&Expression::one(), FieldElement::one(), &e).into()
            }
            BinaryOp::Lt => unimplemented!(
                "Field comparison is not implemented yet, try to cast arguments to integer type"
            ),
            BinaryOp::Lte => unimplemented!(
                "Field comparison is not implemented yet, try to cast arguments to integer type"
            ),
            BinaryOp::And => InternalVar::from(evaluate_and(l_c, r_c, res_type.bits(), evaluator)),
            BinaryOp::Or => InternalVar::from(evaluate_or(l_c, r_c, res_type.bits(), evaluator)),
            BinaryOp::Xor => InternalVar::from(evaluate_xor(l_c, r_c, res_type.bits(), evaluator)),
            BinaryOp::Shl | BinaryOp::Shr => unreachable!(),
            i @ BinaryOp::Assign => unreachable!("Invalid Instruction: {:?}", i),
        }
    }

    pub fn print_circuit(opcodes: &[AcirOpcode]) {
        for opcode in opcodes {
            println!("{opcode:?}");
        }
    }

    //Load array values into InternalVars
    //If create_witness is true, we create witnesses for values that do not have witness
    pub fn load_array(
        &mut self,
        array: &MemArray,
        create_witness: bool,
        evaluator: &mut Evaluator,
    ) -> Vec<InternalVar> {
        (0..array.len)
            .map(|i| {
                let address = array.adr + i;
                if let Some(memory) = self.memory_map.get_mut(&address) {
                    if create_witness && memory.witness.is_none() {
                        let w = evaluator.create_intermediate_variable(memory.expression.clone());
                        self.memory_map.get_mut(&address).unwrap().witness = Some(w);
                    }
                    self.memory_map[&address].clone()
                } else {
                    array.values[i as usize].clone()
                }
            })
            .collect()
    }

    //Map the outputs into the array
    fn map_array(&mut self, a: ArrayId, outputs: &[Witness], ctx: &SsaContext) {
        let array = &ctx.mem[a];
        let adr = array.adr;
        for i in 0..array.len {
            if i < outputs.len() as u32 {
                let var = InternalVar::from(outputs[i as usize]);
                self.memory_map.insert(adr + i, var);
            } else {
                let var = InternalVar::from(Expression::zero());
                self.memory_map.insert(adr + i, var);
            }
        }
    }

    pub fn evaluate_neq(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Expression {
        if let (Some(a), Some(b)) = (Memory::deref(ctx, lhs), Memory::deref(ctx, rhs)) {
            let array_a = &ctx.mem[a];
            let array_b = &ctx.mem[b];

            if array_a.len == array_b.len {
                let mut x = InternalVar::from(self.zero_eq_array_sum(array_a, array_b, evaluator));
                x.generate_witness(evaluator);
                from_witness(evaluate_zero_equality(&x, evaluator))
            } else {
                //If length are different, then the arrays are different
                Expression::one()
            }
        } else {
            if let (Some(l), Some(r)) = (l_c.to_const(), r_c.to_const()) {
                if l == r {
                    return Expression::default();
                } else {
                    return Expression::one();
                }
            }
            let mut x =
                InternalVar::from(subtract(&l_c.expression, FieldElement::one(), &r_c.expression));
            x.generate_witness(evaluator);
            from_witness(evaluate_zero_equality(&x, evaluator))
        }
    }

    pub fn evaluate_eq(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        l_c: &InternalVar,
        r_c: &InternalVar,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Expression {
        let neq = self.evaluate_neq(lhs, rhs, l_c, r_c, ctx, evaluator);
        subtract(&Expression::one(), FieldElement::one(), &neq)
    }

    //Generates gates for the expression: \sum_i(zero_eq(A[i]-B[i]))
    //N.b. We assumes the lenghts of a and b are the same but it is not checked inside the function.
    fn zero_eq_array_sum(
        &mut self,
        a: &MemArray,
        b: &MemArray,
        evaluator: &mut Evaluator,
    ) -> Expression {
        let mut sum = Expression::default();

        let a_values = self.load_array(a, false, evaluator);
        let b_values = self.load_array(b, false, evaluator);

        for (a_iter, b_iter) in a_values.into_iter().zip(b_values) {
            let diff_expr = subtract(&a_iter.expression, FieldElement::one(), &b_iter.expression);

            let diff_witness = evaluator.add_witness_to_cs();
            let diff_var = InternalVar {
                //in cache??
                expression: diff_expr.clone(),
                witness: Some(diff_witness),
                id: None,
            };
            evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
                &diff_expr,
                FieldElement::one(),
                &from_witness(diff_witness),
            )));
            //TODO: avoid creating witnesses for diff
            sum = add(
                &sum,
                FieldElement::one(),
                &from_witness(evaluate_zero_equality(&diff_var, evaluator)),
            );
        }
        sum
    }

    //Transform the arguments of intrinsic functions into witnesses
    pub fn prepare_inputs(
        &self,
        args: &[NodeId],
        cfg: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Vec<FunctionInput> {
        let mut inputs: Vec<FunctionInput> = Vec::new();

        for a in args {
            let l_obj = cfg.try_get_node(*a).unwrap();
            match l_obj {
                node::NodeObj::Obj(v) => {
                    match l_obj.get_type() {
                        node::ObjectType::Pointer(a) => {
                            let array = &cfg.mem[a];
                            let num_bits = array.element_type.bits();
                            for i in 0..array.len {
                                let address = array.adr + i;
                                if self.memory_map.contains_key(&address) {
                                    if let Some(wit) = self.memory_map[&address].witness {
                                        inputs.push(FunctionInput { witness: wit, num_bits });
                                    } else {
                                        //TODO we should store the witnesses somewhere, else if the inputs are re-used
                                        //we will duplicate the witnesses.
                                        let w = evaluator.create_intermediate_variable(
                                            self.memory_map[&address].expression.clone(),
                                        );
                                        inputs.push(FunctionInput { witness: w, num_bits });
                                    }
                                } else {
                                    inputs.push(FunctionInput {
                                        witness: array.values[i as usize].witness.unwrap(),
                                        num_bits,
                                    });
                                }
                            }
                        }
                        _ => {
                            if let Some(w) = v.witness {
                                inputs
                                    .push(FunctionInput { witness: w, num_bits: v.size_in_bits() });
                            } else {
                                todo!("generate a witness");
                            }
                        }
                    }
                }
                _ => {
                    if self.arith_cache.contains_key(a) {
                        let mut var = self.arith_cache[a].clone();
                        let witness =
                            var.witness.unwrap_or_else(|| var.generate_witness(evaluator));
                        inputs.push(FunctionInput { witness, num_bits: l_obj.size_in_bits() });
                    } else {
                        unreachable!("invalid input: {:?}", l_obj)
                    }
                }
            }
        }
        inputs
    }

    pub fn evaluate_opcode(
        &mut self,
        instruction_id: NodeId,
        opcode: builtin::Opcode,
        args: &[NodeId],
        res_type: ObjectType,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Expression {
        let outputs;
        match opcode {
            Opcode::ToBits => {
                let bit_size = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
                let l_c = self.substitute(args[0], evaluator, ctx);
                outputs = to_radix_base(&l_c, 2, bit_size, evaluator);
                if let node::ObjectType::Pointer(a) = res_type {
                    self.map_array(a, &outputs, ctx);
                }
            }
            Opcode::ToRadix => {
                let radix = ctx.get_as_constant(args[1]).unwrap().to_u128() as u32;
                let limb_size = ctx.get_as_constant(args[2]).unwrap().to_u128() as u32;
                let l_c = self.substitute(args[0], evaluator, ctx);
                outputs = to_radix_base(&l_c, radix, limb_size, evaluator);
                if let node::ObjectType::Pointer(a) = res_type {
                    self.map_array(a, &outputs, ctx);
                }
            }
            Opcode::LowLevel(op) => {
                let inputs = self.prepare_inputs(args, ctx, evaluator);
                let output_count = op.definition().output_size.0 as u32;
                outputs = self.prepare_outputs(instruction_id, output_count, ctx, evaluator);

                let call_gate = BlackBoxFuncCall {
                    name: op,
                    inputs,                   //witness + bit size
                    outputs: outputs.clone(), //witness
                };
                evaluator.opcodes.push(AcirOpcode::BlackBoxFuncCall(call_gate));
            }
            Opcode::Sort => {
                let mut in_expr = Vec::new();
                let array_id = Memory::deref(ctx, args[0]).unwrap();
                let array = &ctx.mem[array_id];
                let num_bits = array.element_type.bits();
                for i in 0..array.len {
                    let address = array.adr + i;
                    if self.memory_map.contains_key(&address) {
                        if let Some(wit) = self.memory_map[&address].witness {
                            in_expr.push(from_witness(wit))
                        } else {
                            in_expr.push(self.memory_map[&address].expression.clone());
                        }
                    } else {
                        in_expr.push(from_witness(array.values[i as usize].witness.unwrap()));
                    }
                }
                outputs = self.prepare_outputs(instruction_id, array.len, ctx, evaluator);
                let out_expr: Vec<Expression> = outputs.iter().map(|w| from_witness(*w)).collect();
                for i in 0..(out_expr.len() - 1) {
                    bound_constraint_with_offset(
                        &out_expr[i],
                        &out_expr[i + 1],
                        &Expression::zero(),
                        num_bits,
                        evaluator,
                    );
                }
                let bits = evaluate_permutation(&in_expr, &out_expr, evaluator);
                let inputs = in_expr.iter().map(|a| vec![a.clone()]).collect();
                evaluator.opcodes.push(AcirOpcode::Directive(Directive::PermutationSort {
                    inputs,
                    tuple: 1,
                    bits,
                    sort_by: vec![0],
                }));
                if let node::ObjectType::Pointer(a) = res_type {
                    self.map_array(a, &outputs, ctx);
                } else {
                    unreachable!();
                }
            }
        }

        if outputs.len() == 1 {
            from_witness(outputs[0])
        } else {
            //if there are more than one witness returned, the result is inside ins.res_type as a pointer to an array
            Expression::default()
        }
    }

    pub fn prepare_outputs(
        &mut self,
        pointer: NodeId,
        output_nb: u32,
        ctx: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Vec<Witness> {
        // Create fresh variables that will link to the output
        let mut outputs = Vec::with_capacity(output_nb as usize);
        for _ in 0..output_nb {
            let witness = evaluator.add_witness_to_cs();
            outputs.push(witness);
        }

        let l_obj = ctx.try_get_node(pointer).unwrap();
        if let node::ObjectType::Pointer(a) = l_obj.get_type() {
            self.map_array(a, &outputs, ctx);
        }
        outputs
    }
}

pub fn evaluate_sdiv(
    _lhs: &InternalVar,
    _rhs: &InternalVar,
    _evaluator: &mut Evaluator,
) -> (Expression, Expression) {
    todo!();
}

//Returns 1 if lhs < rhs
pub fn evaluate_cmp(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    signed: bool,
    evaluator: &mut Evaluator,
) -> Expression {
    if signed {
        //TODO use range_constraints instead of bit decomposition, like in the unsigned case
        let mut sub_expr = subtract(&lhs.expression, FieldElement::one(), &rhs.expression);
        let two_pow = BigUint::one() << (bit_size + 1);
        sub_expr.q_c += FieldElement::from_be_bytes_reduce(&two_pow.to_bytes_be());
        let bits = to_radix_base(&sub_expr.into(), 2, bit_size + 2, evaluator);
        from_witness(bits[(bit_size - 1) as usize])
    } else {
        let is_greater =
            from_witness(bound_check(&lhs.expression, &rhs.expression, bit_size, evaluator));
        subtract(&Expression::one(), FieldElement::one(), &is_greater)
    }
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}
pub fn bit_size_u32(a: u32) -> u32 where {
    num_bits::<u32>() as u32 - a.leading_zeros()
}

pub fn bit_size_u128(a: u128) -> u32 where {
    num_bits::<u128>() as u32 - a.leading_zeros()
}

//Decomposition into b-base: \sum ai b^i, where 0<=ai<b
// radix: the base, (it is a constant, not a witness)
// num_limbs: the number of elements in the decomposition
// output: (the elements of the decomposition as witness, the sum expression)
pub fn to_radix(
    radix: u32,
    num_limbs: u32,
    evaluator: &mut Evaluator,
) -> (Vec<Witness>, Expression) {
    let mut digits = Expression::default();
    let mut radix_pow = FieldElement::one();

    let shift = FieldElement::from(radix as i128);
    let mut result = Vec::new();
    let bit_size = bit_size_u32(radix);
    for _ in 0..num_limbs {
        let limb_witness = evaluator.add_witness_to_cs();
        result.push(limb_witness);
        let limb_expr = from_witness(limb_witness);
        digits = add(&digits, radix_pow, &limb_expr);
        radix_pow = radix_pow.mul(shift);

        if 1_u128 << (bit_size - 1) != radix as u128 {
            try_range_constraint(limb_witness, bit_size, evaluator);
        }
        bound_constraint_with_offset(
            &from_witness(limb_witness),
            &Expression::from_field(shift),
            &Expression::one(),
            bit_size,
            evaluator,
        );
    }

    (result, digits)
}

//decompose lhs onto radix-base with limb_size limbs
pub fn to_radix_base(
    lhs: &InternalVar,
    radix: u32,
    limb_size: u32,
    evaluator: &mut Evaluator,
) -> Vec<Witness> {
    // ensure there is no overflow
    let mut max = BigUint::from(radix);
    max = max.pow(limb_size) - BigUint::one();
    assert!(max < FieldElement::modulus());

    let (result, bytes) = to_radix(radix, limb_size, evaluator);
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::ToRadix {
        a: lhs.expression.clone(),
        b: result.clone(),
        radix,
    }));

    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
        &lhs.expression,
        FieldElement::one(),
        &bytes,
    )));

    result
}

fn const_and(
    var: InternalVar,
    b: FieldElement,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Expression {
    let a_bits = to_radix_base(&var, 2, bit_size, evaluator);
    let mut result = Expression::default();
    let mut k = FieldElement::one();
    let two = FieldElement::from(2_i128);
    for (a_iter, b_iter) in a_bits.into_iter().zip(b.bits().iter().rev()) {
        if *b_iter {
            result = add(&result, k, &from_witness(a_iter));
        }
        k = k.mul(two);
    }
    result
}

fn const_xor(
    var: InternalVar,
    b: FieldElement,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Expression {
    let a_bits = to_radix_base(&var, 2, bit_size, evaluator);
    let mut result = Expression::default();
    let mut k = FieldElement::one();
    let two = FieldElement::from(2_i128);
    for (a_iter, b_iter) in a_bits.into_iter().zip(b.bits().iter().rev()) {
        if *b_iter {
            let c = subtract(&Expression::one(), FieldElement::one(), &from_witness(a_iter));
            result = add(&result, k, &c);
        } else {
            result = add(&result, k, &from_witness(a_iter));
        }
        k = k.mul(two);
    }
    result
}

fn const_or(
    var: InternalVar,
    b: FieldElement,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Expression {
    if let Some(l_c) = var.to_const() {
        return Expression {
            mul_terms: Vec::new(),
            linear_combinations: Vec::new(),
            q_c: (l_c.to_u128() | b.to_u128()).into(),
        };
    }
    let a_bits = to_radix_base(&var, 2, bit_size, evaluator);
    let mut result = Expression::default();
    let mut k = FieldElement::one();
    let two = FieldElement::from(2_i128);
    let mut q_c = FieldElement::zero();
    for (a_iter, b_iter) in a_bits.into_iter().zip(b.bits().iter().rev()) {
        if *b_iter {
            q_c += k;
        } else {
            result = add(&result, k, &from_witness(a_iter));
        }
        k = k.mul(two);
    }
    result.q_c = q_c;
    result
}

pub fn evaluate_and(
    mut lhs: InternalVar,
    mut rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Expression {
    if let Some(r_c) = rhs.to_const() {
        return const_and(lhs, r_c, bit_size, evaluator);
    }
    if let Some(l_c) = lhs.to_const() {
        return const_and(rhs, l_c, bit_size, evaluator);
    }

    let a_witness = lhs.generate_witness(evaluator);
    let b_witness = rhs.generate_witness(evaluator);
    //TODO checks the cost of the gate vs bit_size (cf. #164)
    if bit_size == 1 {
        return Expression {
            mul_terms: vec![(FieldElement::one(), a_witness, b_witness)],
            linear_combinations: Vec::new(),
            q_c: FieldElement::zero(),
        };
    }
    let result = evaluator.add_witness_to_cs();
    let bsize = if bit_size % 2 == 1 { bit_size + 1 } else { bit_size };
    assert!(bsize < FieldElement::max_num_bits() - 1);

    let gate = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
        name: acvm::acir::BlackBoxFunc::AND,
        inputs: vec![
            FunctionInput { witness: a_witness, num_bits: bsize },
            FunctionInput { witness: b_witness, num_bits: bsize },
        ],
        outputs: vec![result],
    });
    evaluator.opcodes.push(gate);

    Expression::from(Linear::from_witness(result))
}

pub fn evaluate_xor(
    mut lhs: InternalVar,
    mut rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Expression {
    if let Some(r_c) = rhs.to_const() {
        return const_xor(lhs, r_c, bit_size, evaluator);
    }
    if let Some(l_c) = lhs.to_const() {
        return const_xor(rhs, l_c, bit_size, evaluator);
    }

    //TODO checks the cost of the gate vs bit_size (cf. #164)
    if bit_size == 1 {
        let sum = add(&lhs.expression, FieldElement::one(), &rhs.expression);
        let mul = mul_with_witness(evaluator, &lhs.expression, &rhs.expression);
        return subtract(&sum, FieldElement::from(2_i128), &mul);
    }
    let result = evaluator.add_witness_to_cs();
    let a_witness = lhs.generate_witness(evaluator);
    let b_witness = rhs.generate_witness(evaluator);
    let bsize = if bit_size % 2 == 1 { bit_size + 1 } else { bit_size };
    assert!(bsize < FieldElement::max_num_bits() - 1);

    let gate = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
        name: acvm::acir::BlackBoxFunc::XOR,
        inputs: vec![
            FunctionInput { witness: a_witness, num_bits: bsize },
            FunctionInput { witness: b_witness, num_bits: bsize },
        ],
        outputs: vec![result],
    });
    evaluator.opcodes.push(gate);

    from_witness(result)
}

pub fn evaluate_or(
    lhs: InternalVar,
    rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
) -> Expression {
    if let Some(r_c) = rhs.to_const() {
        return const_or(lhs, r_c, bit_size, evaluator);
    }
    if let Some(l_c) = lhs.to_const() {
        return const_or(rhs, l_c, bit_size, evaluator);
    }

    if bit_size == 1 {
        let sum = add(&lhs.expression, FieldElement::one(), &rhs.expression);
        let mul = mul_with_witness(evaluator, &lhs.expression, &rhs.expression);
        return subtract(&sum, FieldElement::one(), &mul);
    }

    let lhs_bits = to_radix_base(&lhs, 2, bit_size, evaluator);
    let rhs_bits = to_radix_base(&rhs, 2, bit_size, evaluator);
    let mut result = Expression::default();
    let mut k = FieldElement::one();
    let two = FieldElement::from(2_i128);
    for (l_bit, r_bit) in lhs_bits.into_iter().zip(rhs_bits) {
        let l_or_r = evaluate_or(l_bit.into(), r_bit.into(), 1, evaluator);
        result = add(&result, k, &l_or_r);
        k = k.mul(two);
    }
    result
}

//truncate lhs (a number whose value requires max_bits) into a rhs-bits number: i.e it returns b such that lhs mod 2^rhs is b
pub fn evaluate_truncate(
    lhs: InternalVar,
    rhs: u32,
    max_bits: u32,
    evaluator: &mut Evaluator,
) -> InternalVar {
    assert!(max_bits > rhs, "max_bits = {max_bits}, rhs = {rhs}");

    //0. Check for constant expression. This can happen through arithmetic simplifications
    if let Some(a_c) = lhs.to_const() {
        let mut a_big = BigUint::from_bytes_be(&a_c.to_be_bytes());
        let two = BigUint::from(2_u32);
        a_big %= two.pow(rhs);
        return InternalVar::from(FieldElement::from_be_bytes_reduce(&a_big.to_bytes_be()));
    }
    //1. Generate witnesses a,b,c
    let b_witness = evaluator.add_witness_to_cs();
    let c_witness = evaluator.add_witness_to_cs();

    try_range_constraint(b_witness, rhs, evaluator); //TODO propagate the error using ?
    try_range_constraint(c_witness, max_bits - rhs, evaluator);

    //2. Add the constraint a = b+2^Nc
    let mut f = FieldElement::from(2_i128);
    f = f.pow(&FieldElement::from(rhs as i128));
    let b_arith = from_witness(b_witness);
    let c_arith = from_witness(c_witness);
    let res = add(&b_arith, f, &c_arith); //b+2^Nc
    let my_constraint = add(&res, -FieldElement::one(), &lhs.expression);
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Truncate {
        a: lhs.expression,
        b: b_witness,
        c: c_witness,
        bit_size: rhs,
    }));
    evaluator.opcodes.push(AcirOpcode::Arithmetic(my_constraint));
    InternalVar::from(b_witness)
}

pub fn evaluate_udiv(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    predicate: &InternalVar,
    evaluator: &mut Evaluator,
) -> (Witness, Witness) {
    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();
    let pa = mul_with_witness(evaluator, &lhs.expression, &predicate.expression);
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Quotient {
        a: lhs.expression.clone(),
        b: rhs.expression.clone(),
        q: q_witness,
        r: r_witness,
        predicate: Some(predicate.expression.clone()),
    }));

    //r<b
    let r_expr = Expression::from(Linear::from_witness(r_witness));
    try_range_constraint(r_witness, bit_size, evaluator);
    bound_constraint_with_offset(
        &r_expr,
        &rhs.expression,
        &predicate.expression,
        bit_size,
        evaluator,
    );
    //range check q<=a
    try_range_constraint(q_witness, bit_size, evaluator);
    // a-b*q-r = 0
    let mut d = mul_with_witness(evaluator, &rhs.expression, &Expression::from(&q_witness));
    d = add(&d, FieldElement::one(), &Expression::from(&r_witness));
    d = mul_with_witness(evaluator, &d, &predicate.expression);
    let div_eucl = subtract(&pa, FieldElement::one(), &d);

    evaluator.opcodes.push(AcirOpcode::Arithmetic(div_eucl));
    (q_witness, r_witness)
}

//Zero Equality gate: returns 1 if x is not null and 0 else
pub fn evaluate_zero_equality(x: &InternalVar, evaluator: &mut Evaluator) -> Witness {
    let x_witness = x.witness.unwrap(); //todo we need a witness because of the directive, but we should use an expression

    let m = evaluator.add_witness_to_cs(); //'inverse' of x
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Invert { x: x_witness, result: m }));

    //y=x*m         y is 1 if x is not null, and 0 else
    let y_witness = evaluator.add_witness_to_cs();
    evaluator.opcodes.push(AcirOpcode::Arithmetic(Expression {
        mul_terms: vec![(FieldElement::one(), x_witness, m)],
        linear_combinations: vec![(FieldElement::one().neg(), y_witness)],
        q_c: FieldElement::zero(),
    }));

    //x=y*x
    let y_expr = from_witness(y_witness);
    let xy = mul(&from_witness(x_witness), &y_expr);
    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
        &xy,
        FieldElement::one(),
        &from_witness(x_witness),
    )));
    y_witness
}

/// Creates a new witness and constrains it to be the inverse of x
fn evaluate_inverse(
    mut x: InternalVar,
    predicate: &InternalVar,
    evaluator: &mut Evaluator,
) -> Witness {
    // Create a fresh witness - n.b we could check if x is constant or not
    let inverse_witness = evaluator.add_witness_to_cs();
    let inverse_expr = from_witness(inverse_witness);
    let x_witness = x.generate_witness(evaluator); //TODO avoid creating witnesses here.
    evaluator
        .opcodes
        .push(AcirOpcode::Directive(Directive::Invert { x: x_witness, result: inverse_witness }));

    //x*inverse = 1
    Expression::default();
    let one = mul(&from_witness(x_witness), &inverse_expr);
    let lhs = mul_with_witness(evaluator, &one, &predicate.expression);
    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
        &lhs,
        FieldElement::one(),
        &predicate.expression,
    )));
    inverse_witness
}

pub fn mul_with_witness(evaluator: &mut Evaluator, a: &Expression, b: &Expression) -> Expression {
    let a_arith;
    let a_arith = if !a.mul_terms.is_empty() && !b.is_const() {
        let a_witness = evaluator.add_witness_to_cs();
        a_arith = Expression::from(&a_witness);
        evaluator.opcodes.push(AcirOpcode::Arithmetic(a - &a_arith));
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
            evaluator.opcodes.push(AcirOpcode::Arithmetic(b - &b_arith));
            &b_arith
        }
    } else {
        b
    };
    mul(a_arith, b_arith)
}

//a*b
pub fn mul(a: &Expression, b: &Expression) -> Expression {
    let zero = Expression::zero();
    if a.is_const() {
        return add(&zero, a.q_c, b);
    } else if b.is_const() {
        return add(&zero, b.q_c, a);
    } else if !(a.is_linear() && b.is_linear()) {
        unreachable!("Can only multiply linear terms");
    }

    let mut output = Expression::from_field(a.q_c * b.q_c);

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
        match a.linear_combinations[i1].1.cmp(&b.linear_combinations[i2].1) {
            Ordering::Greater => {
                if coef_b != FieldElement::zero() {
                    output.linear_combinations.push((coef_b, b.linear_combinations[i2].1));
                }
                i2 += 1;
            }
            Ordering::Less => {
                if coef_a != FieldElement::zero() {
                    output.linear_combinations.push((coef_a, a.linear_combinations[i1].1));
                }
                i1 += 1;
            }
            Ordering::Equal => {
                if coef_a + coef_b != FieldElement::zero() {
                    output.linear_combinations.push((coef_a + coef_b, a.linear_combinations[i1].1));
                }

                i1 += 1;
                i2 += 1;
            }
        }
    }
    while i1 < a.linear_combinations.len() {
        let coef_a = b.q_c * a.linear_combinations[i1].0;
        output.linear_combinations.push((coef_a, a.linear_combinations[i1].1));
        i1 += 1;
    }
    while i2 < b.linear_combinations.len() {
        let coef_b = a.q_c * b.linear_combinations[i2].0;
        output.linear_combinations.push((coef_b, b.linear_combinations[i2].1));
        i2 += 1;
    }

    output
}

// returns a - k*b
pub fn subtract(a: &Expression, k: FieldElement, b: &Expression) -> Expression {
    add(a, k.neg(), b)
}

// returns a + k*b
pub fn add(a: &Expression, k: FieldElement, b: &Expression) -> Expression {
    let mut output = Expression::default();

    //linear combinations
    let mut i1 = 0; //a
    let mut i2 = 0; //b
    while i1 < a.linear_combinations.len() && i2 < b.linear_combinations.len() {
        match a.linear_combinations[i1].1.cmp(&b.linear_combinations[i2].1) {
            Ordering::Greater => {
                let coef = b.linear_combinations[i2].0 * k;
                if coef != FieldElement::zero() {
                    output.linear_combinations.push((coef, b.linear_combinations[i2].1));
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
                    output.linear_combinations.push((coef, a.linear_combinations[i1].1));
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
            output.linear_combinations.push((coef, b.linear_combinations[i2].1));
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
                    output.mul_terms.push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
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
                    output.mul_terms.push((coef, a.mul_terms[i1].1, a.mul_terms[i1].2));
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
            output.mul_terms.push((coef, b.mul_terms[i2].1, b.mul_terms[i2].2));
        }
        i2 += 1;
    }

    output.q_c = a.q_c + k * b.q_c;
    output
}

// returns w*b.linear_combinations
pub fn single_mul(w: Witness, b: &Expression) -> Expression {
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

pub fn boolean(witness: Witness) -> Expression {
    Expression {
        mul_terms: vec![(FieldElement::one(), witness, witness)],
        linear_combinations: vec![(-FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}

pub fn boolean_expr(expr: &Expression, evaluator: &mut Evaluator) -> Expression {
    subtract(&mul_with_witness(evaluator, expr, expr), FieldElement::one(), expr)
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
        evaluator.opcodes.push(AcirOpcode::Arithmetic(bool_constraint));
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
        evaluator.opcodes.push(AcirOpcode::Directive(Directive::OddRange {
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
        let res = add(&from_witness(r_witness), f, &from_witness(b_witness));
        let my_constraint = add(&res, -FieldElement::one(), &from_witness(witness));
        evaluator.opcodes.push(AcirOpcode::Arithmetic(my_constraint));
    } else {
        let gate = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
            name: acvm::acir::BlackBoxFunc::RANGE,
            inputs: vec![FunctionInput { witness, num_bits }],
            outputs: vec![],
        });
        evaluator.opcodes.push(gate);
    }

    Ok(())
}

// returns a witness of a>=b
fn bound_check(
    a: &Expression,
    b: &Expression,
    max_bits: u32,
    evaluator: &mut Evaluator,
) -> Witness {
    assert!(max_bits + 1 < FieldElement::max_num_bits()); //n.b what we really need is 2^{max_bits+1}<p
    let mut sub = subtract(a, FieldElement::one(), b);
    let two = FieldElement::from(2_i128);
    let two_s = two.pow(&FieldElement::from(max_bits as i128));
    sub.q_c += two_s;
    let q_witness = evaluator.add_witness_to_cs();
    let r_witness = evaluator.add_witness_to_cs();
    //2^s+a-b=q*2^s +r
    let expr = add(&from_witness(r_witness), two_s, &from_witness(q_witness));
    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(&sub, FieldElement::one(), &expr)));
    evaluator.opcodes.push(AcirOpcode::Directive(Directive::Truncate {
        a: sub,
        b: r_witness,
        c: q_witness,
        bit_size: max_bits,
    }));
    try_range_constraint(r_witness, max_bits, evaluator);
    evaluator.opcodes.push(AcirOpcode::Arithmetic(boolean(q_witness)));
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
fn bound_constraint_with_offset(
    a: &Expression,
    b: &Expression,
    offset: &Expression,
    bits: u32,
    evaluator: &mut Evaluator,
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
                0 => evaluator.opcodes.push(AcirOpcode::Arithmetic(aof)),
                1 => {
                    let expr = boolean_expr(&aof, evaluator);
                    evaluator.opcodes.push(AcirOpcode::Arithmetic(expr))
                }
                2 => {
                    let y = InternalVar::expression_to_witness(
                        boolean_expr(&aof, evaluator),
                        evaluator,
                    );
                    let two = FieldElement::from(2_i128);
                    let y_expr = from_witness(y);
                    let eee = subtract(&mul_with_witness(evaluator, &aof, &y_expr), two, &y_expr);
                    evaluator.opcodes.push(AcirOpcode::Arithmetic(eee));
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

fn try_range_constraint(w: Witness, bits: u32, evaluator: &mut Evaluator) {
    if let Err(err) = range_constraint(w, bits, evaluator) {
        eprintln!("{err}");
    }
}

pub fn is_unit(arith: &Expression) -> Option<Witness> {
    if arith.mul_terms.is_empty()
        && arith.linear_combinations.len() == 1
        && arith.linear_combinations[0].0 == FieldElement::one()
        && arith.q_c == FieldElement::zero()
    {
        return Some(arith.linear_combinations[0].1);
    }
    if arith.mul_terms.is_empty() && arith.linear_combinations.len() == 1 {
        //todo!("should be simplified");
    }
    None
}
pub fn from_witness(witness: Witness) -> Expression {
    Expression {
        mul_terms: Vec::new(),
        linear_combinations: vec![(FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}

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
            &from_witness(conf[i]),
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
            &from_witness(c),
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

    use crate::{ssa::acir_gen::evaluate_permutation, Evaluator};
    use rand::prelude::*;

    use super::from_witness;

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
                input.push(from_witness(w));
                a_val.push(FieldElement::from(rng.next_u32() as i128));
                solved_witness.insert(w, a_val[i]);
            }

            let mut output = Vec::new();
            for _i in 0..n {
                let w = eval.add_witness_to_cs();
                b_wit.push(w);
                output.push(from_witness(w));
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
