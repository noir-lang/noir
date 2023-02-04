use crate::ssa::{
    builtin::Opcode,
    context::SsaContext,
    mem::{ArrayId, MemArray, Memory},
    node::{BinaryOp, Instruction, Node, NodeId, ObjectType, Operation},
    {builtin, mem, node},
};
use crate::{Evaluator, RuntimeErrorKind};
use acvm::{
    acir::circuit::{
        directives::Directive,
        opcodes::{BlackBoxFuncCall, FunctionInput, Opcode as AcirOpcode},
    },
    acir::native_types::{Expression, Linear, Witness},
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};
use std::{
    cmp::Ordering,
    collections::HashMap,
    ops::{Mul, Neg},
};

#[derive(Default)]
pub struct Acir {
    pub arith_cache: HashMap<NodeId, InternalVar>,
    pub memory_map: HashMap<u32, InternalVar>, //maps memory address to expression
}

#[derive(Default, Clone, Debug, Eq)]
pub struct InternalVar {
    // A multi-variate degree-2 polynomial
    expression: Expression,
    // A witness associated to `expression`.
    // semantically `cached_witness` and `expression`
    // should be the equal.
    //
    // Example: `z = x + y`
    // `z` can be seen as the same to `x + y`
    // ie if `z = 10` , then `x+y` must be equal to
    // 10.
    // There are places where we can use `cached_witness`
    // in place of `expression` for performance reasons
    // due to the fact that `cached_witness` is a single variable
    // whereas `expression` is a multi-variate polynomial which can
    // contain many degree-2 terms.
    //
    // TODO: What if the cached_witness becomes `dirty`
    // TODO ie the expression is updated, but the cached_witness
    // TODO refers to an old version of it. Can we add tests for this?
    // TODO: We can guarantee this, if an InternalVar is immutable after
    // TODO creation.
    cached_witness: Option<Witness>,
    id: Option<NodeId>,
}

impl InternalVar {
    fn new(expression: Expression, cached_witness: Option<Witness>, id: NodeId) -> InternalVar {
        InternalVar { expression, cached_witness, id: Some(id) }
    }

    /// If the InternalVar holds a constant expression
    /// Return that constant.Otherwise, return None.
    // TODO: we should have a method in ACVM
    // TODO which returns the constant term if its a constant
    // TODO expression. ie `self.expression.to_const()`
    fn to_const(&self) -> Option<FieldElement> {
        if self.is_const_expression() {
            return Some(self.expression.q_c);
        }

        None
    }

    /// The expression term is degree-2 multi-variate polynomial, so
    /// in order to check if if represents a constant,
    /// we need to check that the degree-2 terms `mul_terms`
    /// and the degree-1 terms `linear_combinations` do not exist.
    ///
    /// Example: f(x,y) = xy + 5
    /// `f` is not a constant expression because there
    /// is a bi-variate term `xy`.
    /// Example: f(x,y) = x + y + 5
    /// `f` is not a constant expression because there are
    /// two uni-variate terms `x` and `y`
    /// Example: f(x,y) = 10
    /// `f` is a constant expression because there are no
    /// bi-variate or uni-variate terms, just a constant.
    fn is_const_expression(&self) -> bool {
        self.expression.is_const()
    }

    /// Creates an `InternalVar` from an `Expression`.
    /// If `Expression` represents a degree-1 polynomial
    /// then we also assign it to the `cached_witness`
    fn from_expression(expression: Expression) -> InternalVar {
        let witness = witness_from_expression(&expression);
        InternalVar { expression, cached_witness: witness, id: None }
    }

    /// Creates an `InternalVar` from a `Witness`.
    /// Since a `Witness` can alway be coerced into an
    /// Expression, this method is infallible.
    fn from_witness(witness: Witness) -> InternalVar {
        InternalVar {
            expression: expression_from_witness(witness),
            cached_witness: Some(witness),
            id: None,
        }
    }

    /// Creates an `InternalVar` from a `FieldElement`.
    fn from_constant(constant: FieldElement) -> InternalVar {
        InternalVar { expression: Expression::from_field(constant), cached_witness: None, id: None }
    }

    /// Generates a `Witness` that is equal to the `expression`.
    /// - If a `Witness` has previously been generated
    /// we return that.
    /// - If the Expression represents a constant, we return None.
    fn witness(&mut self, evaluator: &mut Evaluator) -> Option<Witness> {
        // Check if we've already generated a `Witness` which is equal to
        // the stored `Expression`
        if let Some(witness) = self.cached_witness {
            return Some(witness);
        }

        // If we have a constant expression, we do not
        // create a witness equal to it and instead
        // panic (TODO change)
        // TODO: why don't we create a witness for the constant expression here?
        if self.is_const_expression() {
            return None;
        }

        self.cached_witness =
            Some(InternalVar::expression_to_witness(self.expression.clone(), evaluator));

        self.cached_witness
    }

    /// Converts an `Expression` into a `Witness`
    /// - If the `Expression` is a degree-1 univariate polynomial
    /// then this conversion is a simple coercion.
    /// - Otherwise, we create a new `Witness` and set it to be equal to the
    /// `Expression`.
    fn expression_to_witness(expr: Expression, evaluator: &mut Evaluator) -> Witness {
        match witness_from_expression(&expr) {
            Some(witness) => witness,
            None => evaluator.create_intermediate_variable(expr),
        }
    }
}

impl PartialEq for InternalVar {
    fn eq(&self, other: &Self) -> bool {
        // An InternalVar is Equal to another InternalVar if _any_ of the fields
        // in the InternalVar are equal.

        let expressions_are_same = self.expression == other.expression;

        // Check if `cached_witnesses` are the same
        //
        // This may happen if the expressions are the same
        // but one is simplified and the other is not.
        //
        // The caller whom created both `InternalVar` objects
        // may have known this and set their cached_witnesses to
        // be the same.
        // Example:
        // z = 2*x + y
        // t = x + x + y
        // After simplification, it is clear that both RHS are the same
        // However, since when we check for equality, we do not check for
        // simplification, or in particular, we may eagerly assume
        // the two expressions of the RHS are different.
        //
        // The caller can notice this and thus do:
        // InternalVar::new(expr: 2*x + y, witness: z)
        // InternalVar::new(expr: x + x + y, witness: z)
        let cached_witnesses_same =
            self.cached_witness.is_some() && self.cached_witness == other.cached_witness;

        let node_ids_same = self.id.is_some() && self.id == other.id;

        expressions_are_same || cached_witnesses_same || node_ids_same
    }
}

impl From<Expression> for InternalVar {
    fn from(expression: Expression) -> InternalVar {
        InternalVar::from_expression(expression)
    }
}

impl From<Witness> for InternalVar {
    fn from(witness: Witness) -> InternalVar {
        InternalVar::from_witness(witness)
    }
}

impl From<FieldElement> for InternalVar {
    fn from(constant: FieldElement) -> InternalVar {
        InternalVar::from_constant(constant)
    }
}

impl Acir {
    //This function stores the substitution with the arithmetic expression in the cache
    //When an instruction performs arithmetic operation, its output can be represented as an arithmetic expression of its arguments
    //Substitute a node object as an arithmetic expression
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
            Some(node::NodeObject::Const(c)) => {
                let f_value = FieldElement::from_be_bytes_reduce(&c.value.to_bytes_be());
                let expr = Expression::from_field(f_value);
                InternalVar::new(expr, None, id)
            }
            Some(node::NodeObject::Obj(v)) => match v.get_type() {
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
            Operation::Return(node_ids) => {
                // XXX: When we return a node_id that was created from
                // the UnitType, there is a witness associated with it
                // Ideally no witnesses are created for such types.

                // This can only ever be called in the main context.
                // In all other context's, the return operation is transformed.

                for node_id in node_ids {
                    // An array produces a single node_id
                    // We therefore need to check if the node_id is referring to an array
                    // and deference to get the elements
                    let objects = match Memory::deref(ctx, *node_id) {
                        Some(a) => {
                            let array = &ctx.mem[a];
                            self.load_array(array, false, evaluator)
                        }
                        None => vec![self.substitute(*node_id, evaluator, ctx)],
                    };

                    for mut object in objects {
                        let witness = if object.expression.is_const() {
                            evaluator.create_intermediate_variable(object.expression)
                        } else {
                            object.witness(evaluator).expect("unexpected constant expression")
                        };

                        // Before pushing to the public inputs, we need to check that
                        // it was not a private ABI input
                        if evaluator.is_private_abi_input(witness) {
                            return Err(RuntimeErrorKind::Spanless(String::from(
                                "we do not allow private ABI inputs to be returned as public outputs",
                            )));
                        }
                        evaluator.public_inputs.push(witness);
                    }
                }

                InternalVar::default()
            }
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
                        //if not found, then it must be a witness (else it is non-initialized memory)
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
                let inverse = expression_from_witness(evaluate_inverse(r_c, &predicate, evaluator));
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
            BinaryOp::And => InternalVar::from(evaluate_bitwise(
                l_c,
                r_c,
                res_type.bits(),
                evaluator,
                BinaryOp::And,
            )),
            BinaryOp::Or => InternalVar::from(evaluate_bitwise(
                l_c,
                r_c,
                res_type.bits(),
                evaluator,
                BinaryOp::Or,
            )),
            BinaryOp::Xor => InternalVar::from(evaluate_bitwise(
                l_c,
                r_c,
                res_type.bits(),
                evaluator,
                BinaryOp::Xor,
            )),
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
    fn load_array(
        &mut self,
        array: &MemArray,
        create_witness: bool,
        evaluator: &mut Evaluator,
    ) -> Vec<InternalVar> {
        (0..array.len)
            .map(|i| {
                let address = array.adr + i;
                if let Some(memory) = self.memory_map.get_mut(&address) {
                    if create_witness && memory.cached_witness.is_none() {
                        let w = evaluator.create_intermediate_variable(memory.expression.clone());
                        self.memory_map.get_mut(&address).unwrap().cached_witness = Some(w);
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

    fn evaluate_neq(
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
                x.witness(evaluator);
                expression_from_witness(evaluate_zero_equality(&x, evaluator))
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
            x.witness(evaluator);
            expression_from_witness(evaluate_zero_equality(&x, evaluator))
        }
    }

    fn evaluate_eq(
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
    //N.b. We assumes the lengths of a and b are the same but it is not checked inside the function.
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
                cached_witness: Some(diff_witness),
                id: None,
            };
            evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
                &diff_expr,
                FieldElement::one(),
                &expression_from_witness(diff_witness),
            )));
            //TODO: avoid creating witnesses for diff
            sum = add(
                &sum,
                FieldElement::one(),
                &expression_from_witness(evaluate_zero_equality(&diff_var, evaluator)),
            );
        }
        sum
    }

    //Transform the arguments of intrinsic functions into witnesses
    fn prepare_inputs(
        &mut self,
        args: &[NodeId],
        cfg: &SsaContext,
        evaluator: &mut Evaluator,
    ) -> Vec<FunctionInput> {
        let mut inputs: Vec<FunctionInput> = Vec::new();

        for a in args {
            let l_obj = cfg.try_get_node(*a).unwrap();
            match l_obj {
                node::NodeObject::Obj(v) => match l_obj.get_type() {
                    node::ObjectType::Pointer(a) => {
                        let array = &cfg.mem[a];
                        let num_bits = array.element_type.bits();
                        for i in 0..array.len {
                            let address = array.adr + i;
                            if self.memory_map.contains_key(&address) {
                                if let Some(wit) = self.memory_map[&address].cached_witness {
                                    inputs.push(FunctionInput { witness: wit, num_bits });
                                } else {
                                    let mut var = self.memory_map[&address].clone();
                                    if var.expression.is_const() {
                                        let w = evaluator.create_intermediate_variable(
                                            self.memory_map[&address].expression.clone(),
                                        );
                                        var.cached_witness = Some(w);
                                    }
                                    let w = var
                                        .witness(evaluator)
                                        .expect("unexpected constant expression");
                                    self.memory_map.insert(address, var);

                                    inputs.push(FunctionInput { witness: w, num_bits });
                                }
                            } else {
                                inputs.push(FunctionInput {
                                    witness: array.values[i as usize].cached_witness.unwrap(),
                                    num_bits,
                                });
                            }
                        }
                    }
                    _ => {
                        if let Some(w) = v.witness {
                            inputs.push(FunctionInput { witness: w, num_bits: v.size_in_bits() });
                        } else {
                            todo!("generate a witness");
                        }
                    }
                },
                _ => {
                    if self.arith_cache.contains_key(a) {
                        let mut var = self.arith_cache[a].clone();
                        let witness = var.cached_witness.unwrap_or_else(|| {
                            var.witness(evaluator).expect("unexpected constant expression")
                        });
                        inputs.push(FunctionInput { witness, num_bits: l_obj.size_in_bits() });
                    } else {
                        unreachable!("invalid input: {:?}", l_obj)
                    }
                }
            }
        }
        inputs
    }

    fn evaluate_opcode(
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
        }

        if outputs.len() == 1 {
            expression_from_witness(outputs[0])
        } else {
            //if there are more than one witness returned, the result is inside ins.res_type as a pointer to an array
            Expression::default()
        }
    }

    fn prepare_outputs(
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

fn evaluate_sdiv(
    _lhs: &InternalVar,
    _rhs: &InternalVar,
    _evaluator: &mut Evaluator,
) -> (Expression, Expression) {
    todo!();
}

//Returns 1 if lhs < rhs
fn evaluate_cmp(
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
        expression_from_witness(bits[(bit_size - 1) as usize])
    } else {
        let is_greater = expression_from_witness(bound_check(
            &lhs.expression,
            &rhs.expression,
            bit_size,
            evaluator,
        ));
        subtract(&Expression::one(), FieldElement::one(), &is_greater)
    }
}

const fn num_bits<T>() -> usize {
    std::mem::size_of::<T>() * 8
}
fn bit_size_u32(a: u32) -> u32 where {
    num_bits::<u32>() as u32 - a.leading_zeros()
}

fn bit_size_u128(a: u128) -> u32 where {
    num_bits::<u128>() as u32 - a.leading_zeros()
}

//Decomposition into b-base: \sum ai b^i, where 0<=ai<b
// radix: the base, (it is a constant, not a witness)
// num_limbs: the number of elements in the decomposition
// output: (the elements of the decomposition as witness, the sum expression)
fn to_radix(radix: u32, num_limbs: u32, evaluator: &mut Evaluator) -> (Vec<Witness>, Expression) {
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

//decompose lhs onto radix-base with limb_size limbs
fn to_radix_base(
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

fn simplify_bitwise(
    lhs: &InternalVar,
    rhs: &InternalVar,
    bit_size: u32,
    opcode: &BinaryOp,
) -> Option<InternalVar> {
    if lhs == rhs {
        //simplify bitwise operation of the form: a OP a
        return Some(match opcode {
            BinaryOp::And => lhs.clone(),
            BinaryOp::Or => lhs.clone(),
            BinaryOp::Xor => InternalVar::from(FieldElement::zero()),
            _ => unreachable!(),
        });
    }

    assert!(bit_size < FieldElement::max_num_bits());
    let max = FieldElement::from((1_u128 << bit_size) - 1);
    let mut field = None;
    let mut var = lhs;
    if let Some(l_c) = lhs.to_const() {
        if l_c == FieldElement::zero() || l_c == max {
            field = Some(l_c);
            var = rhs
        }
    } else if let Some(r_c) = rhs.to_const() {
        if r_c == FieldElement::zero() || r_c == max {
            field = Some(r_c);
        }
    }
    if let Some(field) = field {
        //simplify bitwise operation of the form: 0 OP var or 1 OP var
        return Some(match opcode {
            BinaryOp::And => {
                if field.is_zero() {
                    InternalVar::from(field)
                } else {
                    var.clone()
                }
            }
            BinaryOp::Xor => {
                if field.is_zero() {
                    var.clone()
                } else {
                    InternalVar::from(subtract(
                        &Expression::from_field(field),
                        FieldElement::one(),
                        &var.expression,
                    ))
                }
            }
            BinaryOp::Or => {
                if field.is_zero() {
                    var.clone()
                } else {
                    InternalVar::from(field)
                }
            }
            _ => unreachable!(),
        });
    }

    None
}

fn evaluate_bitwise(
    mut lhs: InternalVar,
    mut rhs: InternalVar,
    bit_size: u32,
    evaluator: &mut Evaluator,
    opcode: BinaryOp,
) -> Expression {
    if let Some(var) = simplify_bitwise(&lhs, &rhs, bit_size, &opcode) {
        return var.expression;
    }
    if bit_size == 1 {
        match opcode {
            BinaryOp::And => return mul_with_witness(evaluator, &lhs.expression, &rhs.expression),
            BinaryOp::Xor => {
                let sum = add(&lhs.expression, FieldElement::one(), &rhs.expression);
                let mul = mul_with_witness(evaluator, &lhs.expression, &rhs.expression);
                return subtract(&sum, FieldElement::from(2_i128), &mul);
            }
            BinaryOp::Or => {
                let sum = add(&lhs.expression, FieldElement::one(), &rhs.expression);
                let mul = mul_with_witness(evaluator, &lhs.expression, &rhs.expression);
                return subtract(&sum, FieldElement::one(), &mul);
            }
            _ => unreachable!(),
        }
    }
    //We generate witness from const values in order to use the ACIR bitwise gates
    // If the gate is implemented, it is expected to be better than going through bit decomposition, even if one of the operand is a constant
    // If the gate is not implemented, we rely on the ACIR simplification to remove these witnesses
    if rhs.to_const().is_some() && rhs.cached_witness.is_none() {
        rhs.cached_witness = Some(evaluator.create_intermediate_variable(rhs.expression.clone()));
        assert!(lhs.to_const().is_none());
    } else if lhs.to_const().is_some() && lhs.cached_witness.is_none() {
        assert!(rhs.to_const().is_none());
        lhs.cached_witness = Some(evaluator.create_intermediate_variable(lhs.expression.clone()));
    }

    let mut a_witness = lhs.witness(evaluator).expect("unexpected constant expression");
    let mut b_witness = rhs.witness(evaluator).expect("unexpected constant expression");

    let result = evaluator.add_witness_to_cs();
    let bit_size = if bit_size % 2 == 1 { bit_size + 1 } else { bit_size };
    assert!(bit_size < FieldElement::max_num_bits() - 1);
    let max = FieldElement::from((1_u128 << bit_size) - 1);
    let bit_gate = match opcode {
        BinaryOp::And => acvm::acir::BlackBoxFunc::AND,
        BinaryOp::Xor => acvm::acir::BlackBoxFunc::XOR,
        BinaryOp::Or => {
            a_witness = evaluator.create_intermediate_variable(subtract(
                &Expression::from_field(max),
                FieldElement::one(),
                &lhs.expression,
            ));
            b_witness = evaluator.create_intermediate_variable(subtract(
                &Expression::from_field(max),
                FieldElement::one(),
                &rhs.expression,
            ));
            acvm::acir::BlackBoxFunc::AND
        }
        _ => unreachable!(),
    };

    let gate = AcirOpcode::BlackBoxFuncCall(BlackBoxFuncCall {
        name: bit_gate,
        inputs: vec![
            FunctionInput { witness: a_witness, num_bits: bit_size },
            FunctionInput { witness: b_witness, num_bits: bit_size },
        ],
        outputs: vec![result],
    });
    evaluator.opcodes.push(gate);

    if opcode == BinaryOp::Or {
        subtract(
            &Expression::from_field(max),
            FieldElement::one(),
            &expression_from_witness(result),
        )
    } else {
        expression_from_witness(result)
    }
}

//truncate lhs (a number whose value requires max_bits) into a rhs-bits number: i.e it returns b such that lhs mod 2^rhs is b
fn evaluate_truncate(
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
    let b_arith = expression_from_witness(b_witness);
    let c_arith = expression_from_witness(c_witness);
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

fn evaluate_udiv(
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
    let div_euclidean = subtract(&pa, FieldElement::one(), &d);

    evaluator.opcodes.push(AcirOpcode::Arithmetic(div_euclidean));
    (q_witness, r_witness)
}

//Zero Equality gate: returns 1 if x is not null and 0 else
fn evaluate_zero_equality(x: &InternalVar, evaluator: &mut Evaluator) -> Witness {
    let x_witness = x.cached_witness.unwrap(); //todo we need a witness because of the directive, but we should use an expression

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
    let y_expr = expression_from_witness(y_witness);
    let xy = mul(&expression_from_witness(x_witness), &y_expr);
    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
        &xy,
        FieldElement::one(),
        &expression_from_witness(x_witness),
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
    let inverse_expr = expression_from_witness(inverse_witness);
    let x_witness = x.witness(evaluator).expect("unexpected constant expression"); //TODO avoid creating witnesses here.
    evaluator
        .opcodes
        .push(AcirOpcode::Directive(Directive::Invert { x: x_witness, result: inverse_witness }));

    //x*inverse = 1
    Expression::default();
    let one = mul(&expression_from_witness(x_witness), &inverse_expr);
    let lhs = mul_with_witness(evaluator, &one, &predicate.expression);
    evaluator.opcodes.push(AcirOpcode::Arithmetic(subtract(
        &lhs,
        FieldElement::one(),
        &predicate.expression,
    )));
    inverse_witness
}

fn mul_with_witness(evaluator: &mut Evaluator, a: &Expression, b: &Expression) -> Expression {
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
fn mul(a: &Expression, b: &Expression) -> Expression {
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
fn subtract(a: &Expression, k: FieldElement, b: &Expression) -> Expression {
    add(a, k.neg(), b)
}

// returns a + k*b
fn add(a: &Expression, k: FieldElement, b: &Expression) -> Expression {
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
fn single_mul(w: Witness, b: &Expression) -> Expression {
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

fn boolean(witness: Witness) -> Expression {
    Expression {
        mul_terms: vec![(FieldElement::one(), witness, witness)],
        linear_combinations: vec![(-FieldElement::one(), witness)],
        q_c: FieldElement::zero(),
    }
}

fn boolean_expr(expr: &Expression, evaluator: &mut Evaluator) -> Expression {
    subtract(&mul_with_witness(evaluator, expr, expr), FieldElement::one(), expr)
}

//constrain witness a to be num_bits-size integer, i.e between 0 and 2^num_bits-1
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
        let res = add(&expression_from_witness(r_witness), f, &expression_from_witness(b_witness));
        let my_constraint = add(&res, -FieldElement::one(), &expression_from_witness(witness));
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
    let expr = add(&expression_from_witness(r_witness), two_s, &expression_from_witness(q_witness));
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
                    let y_expr = expression_from_witness(y);
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

// Returns a `Witness` if the `Expression` can be represented as a degree-1
// univariate polynomial. Otherwise, Return None.
//
// Note that `Witness` is only capable of expressing polynomials of the form
// f(x) = x and not polynomials of the form f(x) = mx+c , so this method has
// extra checks to ensure that m=1 and c=0
//
// TODO: move to ACVM repo
fn witness_from_expression(arith: &Expression) -> Option<Witness> {
    let is_deg_one_univariate = expression_is_deg_one_univariate(arith);

    if is_deg_one_univariate {
        // If we get here, we know that our expression is of the form `f(x) = mx+c`
        // We want to now restrict ourselves to expressions of the form f(x) = x
        // ie where the constant term is 0 and the coefficient in front of the variable is
        // one.
        let coefficient = arith.linear_combinations[0].0;
        let variable = arith.linear_combinations[0].1;
        let constant = arith.q_c;

        let coefficient_is_one = coefficient.is_one();
        let constant_term_is_zero = constant.is_zero();

        if coefficient_is_one && constant_term_is_zero {
            return Some(variable);
        }
    }

    None
}
// Returns true if highest degree term in the expression is one.
//
// - `mul_term` in an expression contains degree-2 terms
// - `linear_combinations` contains degree-1 terms
// Hence, it is sufficient to check that there are no `mul_terms`
//
// Examples:
// -  f(x, y) = x + y would return true
// -  f(x, y) = xy would return false, the degree here is 2
// -  f(x,y) = 0 would return true, the degree is 0
//
// TODO: move to ACVM repo
fn expression_is_degree_1(expression: &Expression) -> bool {
    expression.mul_terms.is_empty()
}
// Returns true if the expression can be seen as a degree-1 univariate polynomial
//
// - `mul_terms` in an expression can be univariate, however unless the coefficient
// is zero, it is always degree-2.
// - `linear_combinations` contains the sum of degree-1 terms, these terms do not
// need to contain the same variable and so it can be multivariate. However, we
// have thus far only checked if `linear_combinations` contains one term, so this
// method will return false, if the `Expression` has not been simplified.
//
// Hence, we check in the simplest case if an expression is a degree-1 univariate,
// by checking if it contains no `mul_terms` and it contains one `linear_combination` term.
//
// Examples:
// - f(x,y) = x would return true
// - f(x,y) = x + 6 would return true
// - f(x,y) = 2*y + 6 would return true
// - f(x,y) = x + y would return false
// - f(x, y) = x + x should return true, but we return false *** (we do not simplify)
//
// TODO move to ACVM repo
// TODO: ACVM has a method called is_linear, we should change this to `max_degree_one`
fn expression_is_deg_one_univariate(expression: &Expression) -> bool {
    let has_one_univariate_term = expression.linear_combinations.len() == 1;
    expression_is_degree_1(expression) && has_one_univariate_term
}

// Creates an Expression from a Witness.
//
// This is infallible since an expression is
// a multi-variate polynomial and a Witness
// can be seen as a univariate polynomial
//
// TODO: Possibly remove this small shim.
// TODO: Lets first see how the rest of the code looks after
// TODO further refactor.
fn expression_from_witness(witness: Witness) -> Expression {
    Expression::from(&witness)
}

#[test]
fn internal_var_const_expression() {
    let mut evaluator = Evaluator::new();

    let expected_constant = FieldElement::from(123456789u128);

    // Initialize an InternalVar with a FieldElement
    let mut internal_var = InternalVar::from_constant(expected_constant);

    // We currently do not create witness when the InternalVar was created using a constant
    let witness = internal_var.witness(&mut evaluator);
    assert!(witness.is_none());

    match internal_var.to_const() {
        Some(got_constant) => assert_eq!(got_constant, expected_constant),
        None => {
            panic!("`InternalVar` was initialized with a constant, so a field element was expected")
        }
    }
}
#[test]
fn internal_var_from_witness() {
    let mut evaluator = Evaluator::new();

    let expected_witness = Witness(1234);
    // Initialize an InternalVar with a `Witness`
    let mut internal_var = InternalVar::from_witness(expected_witness);

    // We should get back the same `Witness`
    let got_witness = internal_var.witness(&mut evaluator);
    match got_witness {
        Some(got_witness) => assert_eq!(got_witness, expected_witness),
        None => panic!("expected a `Witness` value"),
    }
}
