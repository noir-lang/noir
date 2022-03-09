use super::block::BlockId;
use super::context::SsaContext;
use super::node::{Instruction, Node, NodeId, Operation, Variable};
use super::{block, node, ssa_form};
use std::collections::HashMap;

use super::super::environment::Environment;
use super::super::errors::{RuntimeError, RuntimeErrorKind};
use crate::object::Object;
use acvm::FieldElement;
use noirc_frontend::hir::Context;
use noirc_frontend::hir_def::expr::{HirConstructorExpression, HirMemberAccess};
use noirc_frontend::hir_def::function::HirFunction;
use noirc_frontend::hir_def::stmt::HirPattern;
use noirc_frontend::hir_def::{
    expr::{HirBinaryOp, HirBinaryOpKind, HirExpression, HirForExpression, HirLiteral},
    stmt::{HirConstrainStatement, HirLetStatement, HirStatement},
};
use noirc_frontend::node_interner::{ExprId, IdentId, NodeInterner, StmtId};
use noirc_frontend::util::vecmap;
use noirc_frontend::Type;

struct IRGenerator<'a> {
    context: SsaContext<'a>,

    /// The current value of a variable. Used for flattening structs
    /// into multiple variables/values
    variable_values: HashMap<IdentId, Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Single(NodeId),
    Struct(Vec<(/*field_name:*/ String, Value)>),
}

impl Value {
    fn into_id(&self) -> NodeId {
        match self {
            Value::Single(id) => *id,
            Value::Struct(_) => panic!("Tried to unwrap a struct into a single value"),
        }
    }
}

////////////////PARSING THE AST//////////////////////////////////////////////
/// Compiles the AST into the intermediate format by evaluating the main function
pub fn evaluate_main<'a>(
    context: &'a Context,
    env: &mut Environment,
    main_func_body: HirFunction, //main function
) -> Result<SsaContext<'a>, RuntimeError> {
    let mut this = IRGenerator::new(context);
    let block = main_func_body.block(&this.def_interner());
    for stmt_id in block.statements() {
        this.evaluate_statement(env, stmt_id)?;
    }

    Ok(this.context)
}

impl<'a> IRGenerator<'a> {
    pub fn new(context: &Context) -> IRGenerator {
        IRGenerator {
            context: SsaContext::new(context),
            variable_values: HashMap::new(),
        }
    }

    fn find_variable(&self, variable_def: Option<IdentId>) -> Option<&Value> {
        variable_def.and_then(|def| self.variable_values.get(&def))
    }

    fn get_current_value(&mut self, value: &Value) -> Value {
        match value {
            Value::Single(id) => Value::Single(ssa_form::get_current_value(&mut self.context, *id)),
            Value::Struct(fields) => Value::Struct(vecmap(fields, |(name, value)| {
                let value = self.get_current_value(value);
                (name.clone(), value)
            })),
        }
    }

    fn evaluate_identifier(&mut self, env: &mut Environment, ident_id: &IdentId) -> Value {
        let ident_def = self.ident_def(ident_id);
        if let Some(value) = ident_def.and_then(|def| self.variable_values.get(&def)) {
            let value = value.clone();
            return self.get_current_value(&value);
        }

        let ident_name = dbg!(self.ident_name(ident_id));
        let obj = env.get(&ident_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);

        // TODO: Creating a new variable won't work for struct/tuple types here
        //new variable - should be in a let statement? The let statement should set the type
        let obj = node::Variable {
            id: NodeId::dummy(),
            name: ident_name.clone(),
            obj_type,
            root: None,
            def: ident_def,
            witness: node::get_witness_from_object(&obj),
            parent_block: self.context.current_block,
        };

        let v_id = self.context.add_variable(obj, None);
        self.context
            .get_current_block_mut()
            .update_variable(v_id, v_id);

        Value::Single(v_id)
    }

    fn def_interner(&self) -> &NodeInterner {
        &self.context.context.def_interner
    }

    fn evaluate_infix_expression(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        op: HirBinaryOp,
    ) -> Result<Value, RuntimeError> {
        let ltype = self.context.get_object_type(lhs);

        let optype = ltype; //n.b. we do not verify rhs type as it should have been handled by the typechecker.

        // Get the opcode from the infix operator
        let opcode = node::to_operation(op.kind, optype);
        let instruction = self.context.new_instruction(lhs, rhs, opcode, optype);
        Ok(Value::Single(instruction))
    }

    pub fn evaluate_statement(
        &mut self,
        env: &mut Environment,
        stmt_id: &StmtId,
    ) -> Result<(), RuntimeError> {
        let statement = self.def_interner().statement(stmt_id);
        match statement {
            HirStatement::Constrain(constrain_stmt) => {
                self.handle_constrain_statement(env, constrain_stmt)
            }
            HirStatement::Expression(expr) | HirStatement::Semi(expr) => {
                self.expression_to_object(env, &expr)?;
                Ok(())
            }
            HirStatement::Let(let_stmt) => {
                // let statements are used to declare a higher level object
                self.handle_let_statement(env, let_stmt)
            }
            HirStatement::Assign(assign_stmt) => {
                let ident_def = self.def_interner().ident_def(&assign_stmt.identifier);
                //////////////TODO temp this is needed because we don't parse main arguments
                let ident_name = self.ident_name(&assign_stmt.identifier);

                let rhs = self.expression_to_object(env, &assign_stmt.expression)?;

                if let Some(lhs) = self.find_variable(ident_def) {
                    // We may be able to avoid cloning here if we change find_variable
                    // and assign_pattern to use only fields of self instead of `self` itself.
                    let lhs = lhs.clone();
                    self.assign_pattern(&lhs, rhs);
                } else {
                    //var is not defined,
                    //let's do it here for now...TODO
                    let typ = self.def_interner().id_type(&assign_stmt.identifier);
                    self.bind_fresh_pattern(&ident_name, &typ, rhs);
                }

                Ok(())
            }
            HirStatement::Error => unreachable!(
                "ice: compiler did not exit before codegen when a statement failed to parse"
            ),
        }
    }

    fn create_new_variable(
        &mut self,
        var_name: String,
        def: Option<IdentId>,
        env: &mut Environment,
    ) -> NodeId {
        let obj = env.get(&var_name);
        let obj_type = node::ObjectType::get_type_from_object(&obj);
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type,
            name: var_name,
            root: None,
            def,
            witness: node::get_witness_from_object(&obj),
            parent_block: self.context.current_block,
        };
        self.context.add_variable(new_var, None)
    }

    // Add a constraint to constrain two expression together
    fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: HirConstrainStatement,
    ) -> Result<(), RuntimeError> {
        let lhs = self.expression_to_object(env, &constrain_stmt.0.lhs)?;
        let rhs = self.expression_to_object(env, &constrain_stmt.0.rhs)?;

        match constrain_stmt.0.operator.kind {
            // HirBinaryOpKind::Add => binary_op::handle_add_op(lhs, rhs, self),
            // HirBinaryOpKind::Subtract => binary_op::handle_sub_op(lhs, rhs, self),
            // HirBinaryOpKind::Multiply => binary_op::handle_mul_op(lhs, rhs, self),
            // HirBinaryOpKind::Divide => binary_op::handle_div_op(lhs, rhs, self),
            HirBinaryOpKind::NotEqual => todo!(),
            HirBinaryOpKind::Equal => {
                //TODO; the truncate strategy should benefit from this.
                //if one of them is a const, them we update the value array of the other to the same const
                // we should replace one with the other 'everywhere'
                // we should merge their property; min(max), min(bitsize),etc..
                Ok(self.context.new_instruction(
                    lhs.into_id(),
                    rhs.into_id(),
                    node::Operation::EqGate,
                    node::ObjectType::NotAnObject,
                ))
            }
            // HirBinaryOpKind::And => binary_op::handle_and_op(lhs, rhs, self),
            // HirBinaryOpKind::Xor => binary_op::handle_xor_op(lhs, rhs, self),
            HirBinaryOpKind::Less => todo!(),
            HirBinaryOpKind::LessEqual => todo!(),
            HirBinaryOpKind::Greater => todo!(),
            HirBinaryOpKind::GreaterEqual => {
                todo!();
            }
            HirBinaryOpKind::Assign => Err(RuntimeErrorKind::Spanless(
                "The Binary operation `=` can only be used in declaration statements".to_string(),
            )),
            HirBinaryOpKind::Or => Err(RuntimeErrorKind::Unimplemented(
                "The Or operation is currently not implemented. First implement in Barretenberg."
                    .to_owned(),
            )),
            _ => Err(RuntimeErrorKind::Unimplemented(
                "The operation is currently not supported in a constrain statement".to_owned(),
            )),
        }
        .map_err(|kind| kind.add_span(constrain_stmt.0.operator.span))?;

        Ok(())
    }

    /// Flatten the pattern and value, binding each identifier in the pattern
    /// to a single NodeId in the corresponding Value. This effectively flattens
    /// let bindings of struct variables, declaring a new variable for each field.
    fn bind_pattern(&mut self, pattern: &HirPattern, value: Value) {
        match (pattern, value) {
            (HirPattern::Identifier(ident_id), Value::Single(node_id)) => {
                let typ = self.def_interner().id_type(ident_id);
                let variable_name = self.ident_name(ident_id);
                let ident_def = self.ident_def(ident_id);
                let value = self.bind_variable(variable_name, ident_def, &typ, node_id);
                self.variable_values.insert(*ident_id, value);
            }
            (HirPattern::Identifier(ident_id), value @ Value::Struct(_)) => {
                let typ = self.def_interner().id_type(ident_id);
                let name = self.ident_name(ident_id);
                let value = self.bind_fresh_pattern(&name, &typ, value);
                self.variable_values.insert(*ident_id, value);
            }
            (HirPattern::Mutable(pattern, _), value) => self.bind_pattern(pattern, value),
            (pattern @ (HirPattern::Tuple(..) | HirPattern::Struct(..)), Value::Struct(exprs)) => {
                assert_eq!(pattern.field_count(), exprs.len());
                for ((pattern_name, pattern), (field_name, value)) in pattern
                    .iter_fields(&self.context.context.def_interner)
                    .zip(exprs)
                {
                    assert_eq!(pattern_name, field_name);
                    self.bind_pattern(pattern, value);
                }
            }
            _ => unreachable!(),
        }
    }

    /// This function is a recursive helper for bind_pattern which takes care
    /// of creating fresh variables to expand `ident = (a, b, ...)` to `(i_a, i_b, ...) = (a, b, ...)`
    ///
    /// This function could use a clearer name
    fn bind_fresh_pattern(&mut self, basename: &str, typ: &Type, value: Value) -> Value {
        match value {
            Value::Single(node_id) => self.bind_variable(basename.to_owned(), None, typ, node_id),
            Value::Struct(field_values) => {
                assert_eq!(field_values.len(), typ.num_elements());
                let values = typ
                    .iter_fields()
                    .zip(field_values)
                    .map(|((field_name, field_type), (value_name, field_value))| {
                        assert_eq!(field_name.as_ref(), &value_name);
                        let name = format!("{}.{}", basename, field_name);
                        let value = self.bind_fresh_pattern(&name, field_type, field_value);
                        (field_name.to_string(), value)
                    })
                    .collect();
                Value::Struct(values)
            }
        }
    }

    fn bind_variable(
        &mut self,
        variable_name: String,
        ident_def: Option<IdentId>,
        typ: &Type,
        value_id: NodeId,
    ) -> Value {
        let obj_type = typ.into();
        let new_var = Variable::new(
            obj_type,
            variable_name,
            ident_def,
            self.context.current_block,
        );
        let id = self.context.add_variable(new_var, None);

        //Assign rhs to lhs
        let result = self
            .context
            .new_instruction(id, value_id, node::Operation::Ass, obj_type);
        //This new variable should not be available in outer scopes.
        let cb = self.context.get_current_block_mut();
        cb.update_variable(id, result); //update the value array. n.b. we should not update the name as it is the first assignment (let)
        Value::Single(id)
    }

    /// Similar to bind_pattern but recursively creates Assignment instructions for
    /// each value rather than defining new variables.
    fn assign_pattern(&mut self, lhs: &Value, rhs: Value) {
        match (lhs, rhs) {
            (Value::Single(lhs_id), Value::Single(rhs_id)) => {
                let lhs = self.context.get_variable(*lhs_id).unwrap();

                //////////////////////////////----******************************************
                let new_var = node::Variable {
                    id: *lhs_id,
                    obj_type: lhs.obj_type,
                    name: String::new(),
                    root: None,
                    def: lhs.def,
                    witness: None,
                    parent_block: self.context.current_block,
                };
                let ls_root = lhs.get_root();

                //ssa: we create a new variable a1 linked to a
                let new_var_id = self.context.add_variable(new_var, Some(ls_root));

                let rhs = &self.context[rhs_id];
                let r_type = rhs.get_type();
                let result = self.context.new_instruction(
                    new_var_id,
                    rhs_id,
                    node::Operation::Ass,
                    r_type,
                );

                self.context.update_variable_id(ls_root, new_var_id, result); //update the name and the value map
            }
            (Value::Struct(lhs_fields), Value::Struct(rhs_fields)) => {
                assert_eq!(lhs_fields.len(), rhs_fields.len());
                for (lhs_field, rhs_field) in lhs_fields.into_iter().zip(rhs_fields) {
                    assert_eq!(lhs_field.0, rhs_field.0);
                    self.assign_pattern(&lhs_field.1, rhs_field.1);
                }
            }
            (Value::Single(_), Value::Struct(_)) => unreachable!("variables with tuple/struct types should already be decomposed into multiple variables"),
            (Value::Struct(_), Value::Single(_)) => unreachable!("Uncaught type error, tried to assign a single value to a tuple/struct type"),
        }
    }

    fn ident_name(&self, ident: &IdentId) -> String {
        self.context.context.def_interner.ident_name(ident)
    }

    fn ident_def(&self, ident: &IdentId) -> Option<IdentId> {
        self.context.context.def_interner.ident_def(ident)
    }

    // Let statements are used to declare higher level objects
    fn handle_let_statement(
        &mut self,
        env: &mut Environment,
        let_stmt: HirLetStatement,
    ) -> Result<(), RuntimeError> {
        let rhs = self.expression_to_object(env, &let_stmt.expression)?;
        self.bind_pattern(&let_stmt.pattern, rhs);
        Ok(())
    }

    pub(crate) fn expression_to_object(
        &mut self,
        env: &mut Environment,
        expr_id: &ExprId,
    ) -> Result<Value, RuntimeError> {
        let expr = self.def_interner().expression(expr_id);
        let span = self.def_interner().expr_span(expr_id);
        match expr {
            HirExpression::Literal(HirLiteral::Integer(x)) =>
            Ok(Value::Single(self.context.new_constant(x))),
            HirExpression::Literal(HirLiteral::Array(_arr_lit)) => {
                //TODO - handle arrays
                todo!();
                //Ok(Object::Array(Array::from(self, env, _arr_lit)?)) 
            },
            HirExpression::Ident(x) =>  {
                Ok(self.evaluate_identifier(env, &x))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            },
            HirExpression::Infix(infx) => {
                // Note: using .into_id() here disallows structs/tuples in infix expressions.
                // The type checker currently disallows this as well but we may want to allow
                // for e.g. struct == struct in the future
                let lhs = self.expression_to_object(env, &infx.lhs)?.into_id();
                let rhs = self.expression_to_object(env, &infx.rhs)?.into_id();
                self.evaluate_infix_expression(lhs, rhs, infx.operator)
            },
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, &cast_expr.lhs)?;
                let rtype = cast_expr.r#type.into();

                // Note: using .into_id here means structs/tuples are disallowed as the lhs of a cast expression
                Ok(Value::Single(self.context.new_cast_expression(lhs.into_id(), rtype)))

                //We should generate a cast instruction and handle properly type conversion:
                // unsigned integer to field ; ok, just checks if bit size over FieldElement::max_num_bits()
                // signed integer to field; ok; check bit size N, retrieve sign bit s and returns x*(1-s)+s*(p-2^N+x)
                // field to unsigned integer; returns x mod 2^N when N is the bit size of the result type
                // field to signed integer; ??
                // bool to integer or field, ok: returns if (x is true) 1 else 0
                // integer to field vers bool: ok, returns (x neq 0)
                // integer to other integer type: checks rust rules TODO
                // else... Not supported (for now).
                //binary_op::handle_cast_op(self,lhs, cast_expr.r#type).map_err(|kind|kind.add_span(span))
            },
            HirExpression::Index(indexed_expr) => {
                // Currently these only happen for arrays
                let arr_name = self.def_interner().ident_name(&indexed_expr.collection_name);
                let ident_span = self.def_interner().ident_span(&indexed_expr.collection_name);
                let _arr = env.get_array(&arr_name).map_err(|kind|kind.add_span(ident_span))?;
                //
                // Evaluate the index expression
                let index_as_obj = self.expression_to_object(env, &indexed_expr.index)?.into_id();
                let index_as_u128 = if let Some(index_as_constant) = self.context.get_as_constant(index_as_obj) {
                    index_as_constant.to_u128()
                }
                else {
                    panic!("Indexed expression does not evaluate to a constant");
                };
                dbg!(index_as_u128);
                todo!();
                //should return array + index
                // arr.get(index_as_u128).map_err(|kind|kind.add_span(span))
            },
            HirExpression::Call(call_expr) => {
                let _func_meta = self.def_interner().function_meta(&call_expr.func_id);
                todo!();
                //TODO generate a new block and checks whether how arguments should be passed (copy or ref)?
                // Choices are a low level func or an imported library function
                // If low level, then we use it's func name to find out what function to call
                // If not then we just call the library as usual with the function definition
                // todo..match func_meta.kind {
                //     FunctionKind::Normal => self.call_function(env, &call_expr, call_expr.func_id),
                //     FunctionKind::LowLevel => {
                //         let attribute = func_meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                //         let opcode_name = attribute.foreign().expect("ice: function marked as foreign, but attribute kind does not match this");
                //         low_level_function_impl::call_low_level(self, env, opcode_name, (call_expr, span))
                //     },
                //     FunctionKind::Builtin => {
                //         let attribute = func_meta.attributes.expect("all builtin functions must contain an attribute which contains the function name which it links to");
                //         let builtin_name = attribute.builtin().expect("ice: function marked as a builtin, but attribute kind does not match this");
                //         builtin::call_builtin(self, env, builtin_name, (call_expr,span))
                //     },
                // ...todo }
            },
            HirExpression::For(for_expr) => self.handle_for_expr(env,for_expr).map_err(|kind|kind.add_span(span)),
            HirExpression::Constructor(constructor) => self.handle_constructor(env, constructor),
            HirExpression::MemberAccess(access) => self.handle_member_access(env, access),
            HirExpression::Tuple(fields) => self.handle_tuple(env, fields),
            HirExpression::If(_) => todo!(),
            HirExpression::Prefix(_) => todo!(),
            HirExpression::Literal(_) => todo!(),
            HirExpression::Block(_) => todo!("currently block expressions not in for/if branches are not being evaluated. In the future, we should be able to unify the eval_block and all places which require block_expr here"),
            HirExpression::Error => todo!(),
        }
    }

    fn handle_constructor(
        &mut self,
        env: &mut Environment,
        constructor: HirConstructorExpression,
    ) -> Result<Value, RuntimeError> {
        let fields = constructor
            .fields
            .into_iter()
            .map(|(ident, field)| {
                let field_name = self.ident_name(&ident);
                Ok((field_name, self.expression_to_object(env, &field)?))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Value::Struct(fields))
    }

    /// A tuple is much the same as a constructor, we just give it fields with numbered names
    fn handle_tuple(
        &mut self,
        env: &mut Environment,
        fields: Vec<ExprId>,
    ) -> Result<Value, RuntimeError> {
        let fields = fields
            .into_iter()
            .enumerate()
            .map(|(i, field)| {
                // Tuple field names are 0..n-1 where n = the length of the tuple
                let field_name = i.to_string();
                Ok((field_name, self.expression_to_object(env, &field)?))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Value::Struct(fields))
    }

    fn handle_member_access(
        &mut self,
        env: &mut Environment,
        access: HirMemberAccess,
    ) -> Result<Value, RuntimeError> {
        match self.expression_to_object(env, &access.lhs)? {
            Value::Single(_) => unreachable!(
                "Runtime type error, expected struct but found a single value for {:?}",
                access
            ),
            Value::Struct(fields) => {
                dbg!(&access);
                let field = dbg!(fields)
                    .into_iter()
                    .find(|(field_name, _)| *field_name == access.rhs.0.contents);

                Ok(field.unwrap().1)
            }
        }
    }

    //TODO generate phi instructions
    fn handle_for_expr(
        &mut self,
        env: &mut Environment,
        for_expr: HirForExpression,
    ) -> Result<Value, RuntimeErrorKind> {
        //we add the ' i = start' instruction (in the block before the join)
        let start_idx = self
            .expression_to_object(env, &for_expr.start_range)
            .map_err(|err| err.remove_span())
            .unwrap()
            .into_id();
        let end_idx = self
            .expression_to_object(env, &for_expr.end_range)
            .map_err(|err| err.remove_span())
            .unwrap()
            .into_id();
        //We support only const range for now
        let start = self.context.get_as_constant(start_idx).unwrap();
        //TODO how should we handle scope (cf. start/end_for_loop)?
        let iter_name = self.def_interner().ident_name(&for_expr.identifier);
        let iter_def = self.def_interner().ident_def(&for_expr.identifier);
        env.store(iter_name.clone(), Object::Constants(start));
        let iter_id = self.create_new_variable(iter_name, iter_def, env); //TODO do we need to store and retrieve it ?
        let iter_var = self.context.get_mut_variable(iter_id).unwrap();
        iter_var.obj_type = node::ObjectType::Unsigned(32); //TODO create_new_variable should set the correct type
        let iter_type = self.context.get_object_type(iter_id);
        dbg!(iter_type);
        let iter_ass =
            self.context
                .new_instruction(iter_id, start_idx, node::Operation::Ass, iter_type);
        //We map the iterator to start_idx so that when we seal the join block, we will get the corrdect value.
        self.context
            .update_variable_id(iter_id, iter_ass, start_idx);

        //join block
        let join_idx =
            block::new_unsealed_block(&mut self.context, block::BlockType::ForJoin, true);
        let exit_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal);
        self.context.current_block = join_idx;
        //should parse a for_expr.condition statement that should evaluate to bool, but
        //we only supports i=start;i!=end for now
        //i1=phi(start);
        let i1 = node::Variable {
            id: iter_id,
            obj_type: iter_type,
            name: String::new(),
            root: None,
            def: None,
            witness: None,
            parent_block: join_idx,
        };
        let i1_id = self.context.add_variable(i1, Some(iter_id)); //TODO we do not need them
                                                                  //we generate the phi for the iterator because the iterator is manually created
        let phi = self.generate_empty_phi(join_idx, iter_id);
        self.context.update_variable_id(iter_id, i1_id, phi); //j'imagine que y'a plus besoin
        let cond =
            self.context
                .new_instruction(phi, end_idx, Operation::Ne, node::ObjectType::Boolean);
        let to_fix = self.context.new_instruction(
            cond,
            NodeId::dummy(),
            node::Operation::Jeq,
            node::ObjectType::NotAnObject,
        );

        //Body
        let body_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal);
        let block = match self.def_interner().expression(&for_expr.block) {
            HirExpression::Block(block_expr) => block_expr,
            _ => panic!("ice: expected a block expression"),
        };
        let body_block1 = &mut self.context[body_id];
        body_block1.update_variable(iter_id, phi); //TODO try with just a get_current_value(iter)
        let statements = block.statements();
        for stmt in statements {
            self.evaluate_statement(env, stmt).unwrap(); //TODO return the error
        }

        //increment iter
        let one = self
            .context
            .get_or_create_const(FieldElement::one(), iter_type);
        let incr = self
            .context
            .new_instruction(phi, one, node::Operation::Add, iter_type);
        let cur_block_id = self.context.current_block; //It should be the body block, except if the body has CFG statements
        let cur_block = &mut self.context[cur_block_id];
        cur_block.update_variable(iter_id, incr);

        //body.left = join
        cur_block.left = Some(join_idx);
        let join_mut = &mut self.context[join_idx];
        join_mut.predecessor.push(cur_block_id);
        //jump back to join
        self.context.new_instruction(
            NodeId::dummy(),
            self.context[join_idx].get_first_instruction(),
            node::Operation::Jmp,
            node::ObjectType::NotAnObject,
        );
        //seal join
        ssa_form::seal_block(&mut self.context, join_idx);

        //exit block
        self.context.current_block = exit_id;
        let exit_first = self.context.get_current_block().get_first_instruction();
        block::link_with_target(&mut self.context, join_idx, Some(exit_id), Some(body_id));
        let first_instruction = self.context[body_id].get_first_instruction();
        self.context.try_get_mut_instruction(to_fix).unwrap().rhs = first_instruction;
        Ok(Value::Single(exit_first)) //TODO what should we return???
    }

    pub fn generate_empty_phi(&mut self, target_block: BlockId, root: NodeId) -> NodeId {
        //Ensure there is not already a phi for the variable (n.b. probably not usefull)
        for i in &self.context[target_block].instructions {
            if let Some(ins) = self.context.try_get_instruction(*i) {
                if ins.operator == node::Operation::Phi && ins.rhs == root {
                    return *i;
                }
            }
        }

        let v_type = self.context.get_object_type(root);
        let new_phi = Instruction::new(Operation::Phi, root, root, v_type, Some(target_block));
        let phi_id = self.context.add_instruction(new_phi);
        self.context[target_block].instructions.insert(1, phi_id);
        phi_id
    }
}
