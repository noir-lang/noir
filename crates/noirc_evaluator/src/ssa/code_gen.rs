use super::block::BlockId;
use super::context::SsaContext;
use super::node::{ConstrainOp, Instruction, Node, NodeId, Operation, Variable};
use super::{block, node, ssa_form};
use std::collections::HashMap;

use super::super::environment::Environment;
use super::super::errors::{RuntimeError, RuntimeErrorKind};
use crate::object::Object;
use acvm::FieldElement;
use noirc_frontend::hir::Context;
use noirc_frontend::hir_def::expr::{HirConstructorExpression, HirMemberAccess};
use noirc_frontend::hir_def::function::HirFunction;
use noirc_frontend::hir_def::stmt::{HirLValue, HirPattern};
use noirc_frontend::hir_def::{
    expr::{HirBinaryOp, HirBinaryOpKind, HirExpression, HirForExpression, HirLiteral},
    stmt::{HirConstrainStatement, HirLetStatement, HirStatement},
};
use noirc_frontend::node_interner::{ExprId, IdentId, NodeInterner, StmtId};
use noirc_frontend::util::vecmap;
use noirc_frontend::{FunctionKind, Type};

struct IRGenerator<'a> {
    context: SsaContext<'a>,
    value_names: HashMap<NodeId, u32>,

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
    fn unwrap_id(&self) -> NodeId {
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
    let block = main_func_body.block(this.def_interner());
    for stmt_id in block.statements() {
        this.evaluate_statement(env, stmt_id)?;
    }

    Ok(this.context)
}

impl<'a> IRGenerator<'a> {
    pub fn new(context: &Context) -> IRGenerator {
        IRGenerator {
            context: SsaContext::new(context),
            value_names: HashMap::new(),
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
        let o_type = self
            .context
            .context
            .def_interner
            .id_type(ident_def.unwrap());

        let var = match obj {
            Object::Array(a) => {
                let obj_type = o_type.into();
                //We should create an array from 'a' witnesses
                self.context.mem.create_array_from_object(
                    &a,
                    ident_def.unwrap(),
                    obj_type,
                    &ident_name,
                );
                let array_index = (self.context.mem.arrays.len() - 1) as u32;
                node::Variable {
                    id: NodeId::dummy(),
                    name: ident_name.clone(),
                    obj_type: node::ObjectType::Pointer(array_index),
                    root: None,
                    def: ident_def,
                    witness: None,
                    parent_block: self.context.current_block,
                }
            }
            _ => {
                let obj_type = node::ObjectType::get_type_from_object(&obj);
                //new variable - should be in a let statement? The let statement should set the type
                node::Variable {
                    id: NodeId::dummy(),
                    name: ident_name.clone(),
                    obj_type,
                    root: None,
                    def: ident_def,
                    witness: node::get_witness_from_object(&obj),
                    parent_block: self.context.current_block,
                }
            }
        };

        let v_id = self.context.add_variable(var, None);
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
    ) -> Result<NodeId, RuntimeError> {
        let ltype = self.context.get_object_type(lhs);
        //n.b. we do not verify rhs type as it should have been handled by the type checker.

        // Get the opcode from the infix operator
        let opcode = node::to_operation(op.kind, ltype);
        // Get the result type from the opcode
        let optype = self.context.get_result_type(opcode, ltype);
        if opcode == node::Operation::Ass {
            if let Some(lhs_ins) = self.context.try_get_mut_instruction(lhs) {
                if let node::Operation::Load(array) = lhs_ins.operator {
                    //make it a store rhs
                    lhs_ins.operator = node::Operation::Store(array);
                    lhs_ins.lhs = rhs;
                    return Ok(lhs);
                }
            }
        }
        Ok(self.context.new_instruction(lhs, rhs, opcode, optype))
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
                //////////////TODO name is needed because we don't parse main arguments
                let (ident_def, ident_name) = self.lvalue_ident_def_and_name(&assign_stmt.lvalue);

                let rhs = self.expression_to_object(env, &assign_stmt.expression)?;

                if let Some(lhs) = self.find_variable(ident_def) {
                    // We may be able to avoid cloning here if we change find_variable
                    // and assign_pattern to use only fields of self instead of `self` itself.
                    let lhs = lhs.clone();
                    self.assign_pattern(&lhs, rhs);
                } else {
                    //var is not defined,
                    //let's do it here for now...TODO
                    let typ = self.lvalue_type(&assign_stmt.lvalue);
                    self.bind_fresh_pattern(&ident_name, &typ, rhs);
                }

                Ok(())
            }
            HirStatement::Error => unreachable!(
                "ice: compiler did not exit before codegen when a statement failed to parse"
            ),
        }
    }

    fn lvalue_type(&self, lvalue: &HirLValue) -> Type {
        match lvalue {
            HirLValue::Ident(id) => self.def_interner().id_type(id),
            HirLValue::MemberAccess { .. } => unimplemented!(),
            HirLValue::Index { .. } => unimplemented!(),
        }
    }

    fn lvalue_ident_def_and_name(&self, lvalue: &HirLValue) -> (Option<IdentId>, String) {
        match lvalue {
            HirLValue::Ident(id) => {
                let def = self.def_interner().ident_def(id);
                (def, self.ident_name(id))
            }
            HirLValue::MemberAccess { .. } => unimplemented!(),
            HirLValue::Index { .. } => unimplemented!(),
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
        let lhs = self
            .expression_to_object(env, &constrain_stmt.0.lhs)?
            .unwrap_id();
        let rhs = self
            .expression_to_object(env, &constrain_stmt.0.rhs)?
            .unwrap_id();

        match constrain_stmt.0.operator.kind {
            // HirBinaryOpKind::Add => binary_op::handle_add_op(lhs, rhs, self),
            // HirBinaryOpKind::Subtract => binary_op::handle_sub_op(lhs, rhs, self),
            // HirBinaryOpKind::Multiply => binary_op::handle_mul_op(lhs, rhs, self),
            // HirBinaryOpKind::Divide => binary_op::handle_div_op(lhs, rhs, self),
            HirBinaryOpKind::NotEqual => Ok(self.context.new_instruction(
                lhs,
                rhs,
                node::Operation::Constrain(ConstrainOp::Neq),
                node::ObjectType::NotAnObject,
            )),
            HirBinaryOpKind::Equal => Ok(self.context.new_instruction(
                lhs,
                rhs,
                node::Operation::Constrain(ConstrainOp::Eq),
                node::ObjectType::NotAnObject,
            )),
            HirBinaryOpKind::And => todo!(),
            // HirBinaryOpKind::Xor => binary_op::handle_xor_op(lhs, rhs, self),
            HirBinaryOpKind::Less => todo!(), // Ok(self.new_instruction(lhs, rhs, node::Operation::LtGate, node::ObjectType::NotAnObject)),
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
                        assert_eq!(field_name, value_name);
                        let name = format!("{}.{}", basename, field_name);
                        let value = self.bind_fresh_pattern(&name, &field_type, field_value);
                        (field_name, value)
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

        if matches!(obj_type, node::ObjectType::Pointer(_)) {
            if let Ok(rhs_mut) = self.context.get_mut_variable(value_id) {
                rhs_mut.def = ident_def;
                rhs_mut.name = variable_name;
                return Value::Single(value_id);
            }
        }

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

    //same as update_variable but using the var index instead of var
    pub fn update_variable_id(&mut self, var_id: NodeId, new_var: NodeId, new_value: NodeId) {
        let root_id = self.context.get_root_value(var_id);
        let root = self.context.get_variable(root_id).unwrap();
        let root_name = root.name.clone();
        let cb = self.context.get_current_block_mut();
        cb.update_variable(var_id, new_value);
        let vname = self.value_names.entry(var_id).or_insert(0);
        *vname += 1;
        let variable_id = *vname;

        if let Ok(nvar) = self.context.get_mut_variable(new_var) {
            nvar.name = format!("{root_name}{variable_id}");
        }
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

                self.update_variable_id(ls_root, new_var_id, result); //update the name and the value map
            }
            (Value::Struct(lhs_fields), Value::Struct(rhs_fields)) => {
                assert_eq!(lhs_fields.len(), rhs_fields.len());
                for (lhs_field, rhs_field) in lhs_fields.iter().zip(rhs_fields) {
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
            HirExpression::Literal(HirLiteral::Integer(x)) => {
                let int_type = self.def_interner().id_type(expr_id);
                let element_type = int_type.into();
                Ok(Value::Single(self.context.get_or_create_const(x, element_type)))
            },
            HirExpression::Literal(HirLiteral::Array(arr_lit)) => {
                //We create a MemArray
                let arr_type = self.def_interner().id_type(expr_id);
                let element_type = arr_type.into();    //WARNING array type!

                let array_index = self.context.mem.create_new_array(arr_lit.length as u32, element_type, &String::new());
                //We parse the array definition
                let elements = self.expression_list_to_objects(env, &arr_lit.contents);
                let array = &mut self.context.mem.arrays[array_index as usize];
                let array_adr = array.adr;
                for (pos, object) in elements.into_iter().enumerate() {
                    //array.witness.push(node::get_witness_from_object(&object));
                    let lhs_adr = self.context.get_or_create_const(FieldElement::from((array_adr + pos as u32) as u128), node::ObjectType::Unsigned(32));
                    self.context.new_instruction(object, lhs_adr, node::Operation::Store(array_index), element_type);
                }
                //Finally, we create a variable pointing to this MemArray
                let new_var = node::Variable {
                    id: NodeId::dummy(),
                    obj_type : node::ObjectType::Pointer(array_index),
                    name: String::new(),
                    root: None,
                    def: None,
                    witness: None,
                    parent_block: self.context.current_block,
                };
                Ok(Value::Single(self.context.add_variable(new_var, None)))
            },
            HirExpression::Ident(x) =>  {
               Ok(self.evaluate_identifier(env, &x))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            },
            HirExpression::Infix(infx) => {
                // Note: using .into_id() here disallows structs/tuples in infix expressions.
                // The type checker currently disallows this as well but we may want to allow
                // for e.g. struct == struct in the future
                let lhs = self.expression_to_object(env, &infx.lhs)?.unwrap_id();
                let rhs = self.expression_to_object(env, &infx.rhs)?.unwrap_id();
                self.evaluate_infix_expression(lhs, rhs, infx.operator)
                    .map(Value::Single)
            },
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, &cast_expr.lhs)?.unwrap_id();
                let rtype = cast_expr.r#type.into();

                Ok(Value::Single(self.context.new_instruction(lhs, lhs, Operation::Cast, rtype)))

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
                let arr_def = self.def_interner().ident_def(&indexed_expr.collection_name);
                let arr_name = self.def_interner().ident_name(&indexed_expr.collection_name);
                let ident_span = self.def_interner().ident_span(&indexed_expr.collection_name);
                let arr_type = self.def_interner().id_type(arr_def.unwrap());
                let o_type = arr_type.into();
                let mut array_index = self.context.mem.arrays.len() as u32;
                let array = if let Some(moi) = self.context.mem.find_array(&arr_def) {
                    array_index= self.context.mem.get_array_index(moi).unwrap();
                    moi
                }
                 else if let Some(Value::Single(pointer)) = self.find_variable(arr_def) {
                    match self.context.get_object_type(*pointer) {
                        node::ObjectType::Pointer(a_id) => {
                            array_index = a_id;
                            &self.context.mem.arrays[a_id as usize]
                        }
                        _ => unreachable!(),
                    }
                 }
                else {
                    let arr = env.get_array(&arr_name).map_err(|kind|kind.add_span(ident_span)).unwrap();
                    self.context.mem.create_array_from_object(&arr, arr_def.unwrap(), o_type, &arr_name)
                };
                //let array = self.mem.get_or_create_array(&arr, arr_def.unwrap(), o_type, arr_name);
                let address = array.adr;

                // Evaluate the index expression
                let index_as_obj = self.expression_to_object(env, &indexed_expr.index)?.unwrap_id();

                let index_type = self.context.get_object_type(index_as_obj);
                let base_adr = self.context.get_or_create_const(FieldElement::from(address as i128), index_type);
                let adr_id = self.context.new_instruction(base_adr, index_as_obj, node::Operation::Add, index_type);
                Ok(Value::Single(self.context.new_instruction(adr_id, adr_id, node::Operation::Load(array_index), o_type)))
            },
            HirExpression::Call(call_expr) => {
                let func_meta = self.def_interner().function_meta(&call_expr.func_id);
                match func_meta.kind {
                    FunctionKind::Normal =>  {
                        //Function defined inside the Noir program.
                        todo!();
                    },
                    FunctionKind::LowLevel => {
                    // We use it's func name to find out what intrinsic function to call
                    todo!();
                    //    let attribute = func_meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                    //    let opcode_name = attribute.foreign().expect("ice: function marked as foreign, but attribute kind does not match this");
                    //    Ok(self.handle_stdlib(env, opcode_name, call_expr))
                    },
                    FunctionKind::Builtin => { todo!();
                    //     let attribute = func_meta.attributes.expect("all builtin functions must contain an attribute which contains the function name which it links to");
                    //     let builtin_name = attribute.builtin().expect("ice: function marked as a builtin, but attribute kind does not match this");
                    //     builtin::call_builtin(self, env, builtin_name, (call_expr,span))
                    },
                 }
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
            HirExpression::MethodCall(_) => unreachable!("Method calls should be desugared before codegen"),
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
    pub fn expression_list_to_objects(
        &mut self,
        env: &mut Environment,
        exprs: &[ExprId],
    ) -> Vec<NodeId> {
        exprs
            .iter()
            .map(|expr| {
                match self.expression_to_object(env, expr) {
                    Ok(Value::Single(id)) => id,
                    // TODO: Can we have arrays of structs? How should we store each element if
                    // structs don't exist in ssa?
                    other => panic!("Unexpected {:?} while codegening ssa array elements", other),
                }
            })
            .collect::<Vec<_>>()
    }

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
            .unwrap_id();
        let end_idx = self
            .expression_to_object(env, &for_expr.end_range)
            .map_err(|err| err.remove_span())
            .unwrap()
            .unwrap_id();
        //We support only const range for now
        let start = self.context.get_as_constant(start_idx).unwrap();
        //TODO how should we handle scope (cf. start/end_for_loop)?
        let iter_name = self.def_interner().ident_name(&for_expr.identifier);
        let iter_def = self.def_interner().ident_def(&for_expr.identifier);
        let int_type = self.def_interner().id_type(&for_expr.identifier);
        env.store(iter_name.clone(), Object::Constants(start));
        let iter_id = self.create_new_variable(iter_name, iter_def, env);
        let iter_var = self.context.get_mut_variable(iter_id).unwrap();
        iter_var.obj_type = int_type.into();
        let iter_type = self.context.get_object_type(iter_id);
        let iter_ass =
            self.context
                .new_instruction(iter_id, start_idx, node::Operation::Ass, iter_type);
        //We map the iterator to start_idx so that when we seal the join block, we will get the corrdect value.
        self.update_variable_id(iter_id, iter_ass, start_idx);

        //join block
        let join_idx =
            block::new_unsealed_block(&mut self.context, block::BlockType::ForJoin, true);
        let exit_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal);
        self.context.current_block = join_idx;
        //should parse a for_expr.condition statement that should evaluate to bool, but
        //we only supports i=start;i!=end for now
        //we generate the phi for the iterator because the iterator is manually created
        let phi = self.generate_empty_phi(join_idx, iter_id);
        self.update_variable_id(iter_id, iter_id, phi); //is it still needed?
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
