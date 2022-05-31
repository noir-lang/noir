use super::context::SsaContext;
use super::node::{Binary, BinaryOp, ConstrainOp, Node, NodeId, ObjectType, Operation, Variable};
use super::{block, node, ssa_form};
use std::collections::HashMap;

use super::super::environment::Environment;
use super::super::errors::{RuntimeError, RuntimeErrorKind};
use crate::object::Object;

use crate::ssa::function;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::hir::Context;
use noirc_frontend::hir_def::expr::{
    HirCallExpression, HirConstructorExpression, HirIdent, HirMemberAccess, HirUnaryOp,
};
use noirc_frontend::hir_def::function::HirFunction;
use noirc_frontend::hir_def::stmt::{HirLValue, HirPattern};
use noirc_frontend::hir_def::{
    expr::{HirBinaryOp, HirBinaryOpKind, HirExpression, HirForExpression, HirLiteral},
    stmt::{HirConstrainStatement, HirLetStatement, HirStatement},
};
use noirc_frontend::node_interner::{DefinitionId, ExprId, NodeInterner, StmtId};
use noirc_frontend::util::vecmap;
use noirc_frontend::{FunctionKind, Type};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct IRGenerator<'a> {
    pub context: SsaContext<'a>,
    value_names: HashMap<NodeId, u32>,

    /// The current value of a variable. Used for flattening structs
    /// into multiple variables/values
    variable_values: HashMap<DefinitionId, Value>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Single(NodeId),
    Struct(Vec<(/*field_name:*/ String, Value)>),
}

impl Value {
    pub fn unwrap_id(&self) -> NodeId {
        match self {
            Value::Single(id) => *id,
            Value::Struct(_) => panic!("Tried to unwrap a struct into a single value"),
        }
    }
}

////////////////PARSING THE AST////////////////////////////////////////////////
/// Compiles the AST into the intermediate format by evaluating the main function
pub fn evaluate_main<'a>(
    igen: &mut IRGenerator<'a>,
    env: &mut Environment,
    main_func_body: HirFunction, //main function
) -> Result<(), RuntimeError> {
    let block = main_func_body.block(igen.def_interner());
    for stmt_id in block.statements() {
        igen.evaluate_statement(env, stmt_id)?;
    }
    Ok(())
}

impl<'a> IRGenerator<'a> {
    pub fn new(context: &Context) -> IRGenerator {
        IRGenerator {
            context: SsaContext::new(context),
            value_names: HashMap::new(),
            variable_values: HashMap::new(),
        }
    }

    pub fn find_variable(&self, variable_def: DefinitionId) -> Option<&Value> {
        self.variable_values.get(&variable_def)
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

    pub fn abi_array(
        &mut self,
        name: &str,
        ident_def: DefinitionId,
        el_type: Type,
        len: u128,
        witness: Vec<acvm::acir::native_types::Witness>,
    ) {
        self.context.mem.create_new_array(len as u32, el_type.into(), name);

        let array_idx = self.context.mem.last_id();

        self.context.mem[array_idx].def = ident_def;
        self.context.mem[array_idx].values = vecmap(witness, |w| w.into());
        let pointer = node::Variable {
            id: NodeId::dummy(),
            name: name.to_string(),
            obj_type: node::ObjectType::Pointer(array_idx),
            root: None,
            def: Some(ident_def),
            witness: None,
            parent_block: self.context.current_block,
        };
        let v_id = self.context.add_variable(pointer, None);
        self.context.get_current_block_mut().update_variable(v_id, v_id);

        let v_value = Value::Single(v_id);
        self.variable_values.insert(ident_def, v_value); //TODO ident_def or ident_id??
    }

    pub fn abi_var(
        &mut self,
        name: &str,
        ident_def: DefinitionId,
        obj_type: node::ObjectType,
        witness: acvm::acir::native_types::Witness,
    ) {
        //new variable - should be in a let statement? The let statement should set the type
        let var = node::Variable {
            id: NodeId::dummy(),
            name: name.to_string(),
            obj_type,
            root: None,
            def: Some(ident_def),
            witness: Some(witness),
            parent_block: self.context.current_block,
        };
        let v_id = self.context.add_variable(var, None);

        self.context.get_current_block_mut().update_variable(v_id, v_id);
        let v_value = Value::Single(v_id);
        self.variable_values.insert(ident_def, v_value); //TODO ident_def or ident_id??
    }

    fn evaluate_identifier(&mut self, env: &mut Environment, ident: HirIdent) -> Value {
        if let Some(value) = self.variable_values.get(&ident.id) {
            let value = value.clone();
            return self.get_current_value(&value);
        }

        let ident_name = self.ident_name(&ident);
        let obj = env.get(&ident_name);
        let o_type = self.context.context.def_interner.id_type(ident.id);

        let var = match obj {
            Object::Array(a) => {
                let obj_type = o_type.into();
                //We should create an array from 'a' witnesses
                let array =
                    self.context.mem.create_array_from_object(&a, ident.id, obj_type, &ident_name);

                node::Variable {
                    id: NodeId::dummy(),
                    name: ident_name.clone(),
                    obj_type: ObjectType::Pointer(array.id),
                    root: None,
                    def: Some(ident.id),
                    witness: None,
                    parent_block: self.context.current_block,
                }
            }
            _ => {
                let obj_type = ObjectType::get_type_from_object(&obj);
                //new variable - should be in a let statement? The let statement should set the type
                node::Variable {
                    id: NodeId::dummy(),
                    name: ident_name.clone(),
                    obj_type,
                    root: None,
                    def: Some(ident.id),
                    witness: node::get_witness_from_object(&obj),
                    parent_block: self.context.current_block,
                }
            }
        };

        let v_id = self.context.add_variable(var, None);
        self.context.get_current_block_mut().update_variable(v_id, v_id);

        Value::Single(v_id)
    }

    pub fn def_interner(&self) -> &NodeInterner {
        &self.context.context.def_interner
    }

    fn evaluate_prefix_expression(
        &mut self,
        rhs: NodeId,
        op: HirUnaryOp,
    ) -> Result<NodeId, RuntimeError> {
        let rtype = self.context.get_object_type(rhs);
        match op {
            HirUnaryOp::Minus => {
                let lhs = self.context.zero();
                // TODO: Perhaps this should be +infinity instead?
                let operator = BinaryOp::Sub { max_rhs_value: BigUint::zero() };
                let op = Operation::Binary(node::Binary { operator, lhs, rhs });
                Ok(self.context.new_instruction(op, rtype))
            }
            HirUnaryOp::Not => Ok(self.context.new_instruction(Operation::Not(rhs), rtype)),
        }
    }

    fn evaluate_infix_expression(&mut self, lhs: NodeId, rhs: NodeId, op: HirBinaryOp) -> NodeId {
        let ltype = self.context.get_object_type(lhs);
        //n.b. we do not verify rhs type as it should have been handled by the type checker.

        if let (HirBinaryOpKind::Assign, Some(lhs_ins)) =
            (op.kind, self.context.try_get_mut_instruction(lhs))
        {
            if let Operation::Load { array_id, index } = lhs_ins.operation {
                //make it a store rhs
                lhs_ins.operation = Operation::Store { array_id, index, value: rhs };
                return lhs;
            }
        }

        // Get the opcode from the infix operator
        let binary = Binary::from_hir(op.kind, ltype, lhs, rhs);
        let opcode = Operation::Binary(binary);

        let optype = self.context.get_result_type(&opcode, ltype);
        self.context.new_instruction(opcode, optype)
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
                    let result = self.assign_pattern(&lhs, rhs);
                    self.variable_values.insert(ident_def, result);
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
            HirLValue::Ident(ident) => self.def_interner().id_type(ident.id),
            HirLValue::MemberAccess { .. } => unimplemented!(),
            HirLValue::Index { .. } => unimplemented!(),
        }
    }

    fn lvalue_ident_def_and_name(&self, lvalue: &HirLValue) -> (DefinitionId, String) {
        match lvalue {
            HirLValue::Ident(ident) => (ident.id, self.ident_name(ident)),
            HirLValue::MemberAccess { .. } => unimplemented!(),
            HirLValue::Index { .. } => unimplemented!(),
        }
    }

    pub fn create_new_variable(
        &mut self,
        var_name: String,
        def: DefinitionId,
        obj_type: node::ObjectType,
        witness: Option<acvm::acir::native_types::Witness>,
    ) -> NodeId {
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type,
            name: var_name,
            root: None,
            def: Some(def),
            witness,
            parent_block: self.context.current_block,
        };
        let v_id = self.context.add_variable(new_var, None);
        let v_value = Value::Single(v_id);
        self.variable_values.insert(def, v_value);
        v_id
    }

    // Add a constraint to constrain two expression together
    fn handle_constrain_statement(
        &mut self,
        env: &mut Environment,
        constrain_stmt: HirConstrainStatement,
    ) -> Result<(), RuntimeError> {
        let lhs = self.expression_to_object(env, &constrain_stmt.0.lhs)?.unwrap_id();
        let rhs = self.expression_to_object(env, &constrain_stmt.0.rhs)?.unwrap_id();

        match constrain_stmt.0.operator.kind {
            // HirBinaryOpKind::Add => binary_op::handle_add_op(lhs, rhs, self),
            // HirBinaryOpKind::Subtract => binary_op::handle_sub_op(lhs, rhs, self),
            // HirBinaryOpKind::Multiply => binary_op::handle_mul_op(lhs, rhs, self),
            // HirBinaryOpKind::Divide => binary_op::handle_div_op(lhs, rhs, self),
            HirBinaryOpKind::NotEqual => Ok(self.context.new_instruction(
                Operation::binary(BinaryOp::Constrain(ConstrainOp::Neq), lhs, rhs),
                ObjectType::NotAnObject,
            )),
            HirBinaryOpKind::Equal => Ok(self.context.new_instruction(
                Operation::binary(BinaryOp::Constrain(ConstrainOp::Eq), lhs, rhs),
                ObjectType::NotAnObject,
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
            (HirPattern::Identifier(ident), Value::Single(node_id)) => {
                let otype = self.context.get_object_type(node_id);
                let variable_name = self.ident_name(ident);
                let value = self.bind_variable(variable_name, Some(ident.id), otype, node_id);
                self.variable_values.insert(ident.id, value);
            }
            (HirPattern::Identifier(ident), value @ Value::Struct(_)) => {
                let typ = self.def_interner().id_type(ident.id);
                let name = self.ident_name(ident);
                let value = self.bind_fresh_pattern(&name, &typ, value);
                self.variable_values.insert(ident.id, value);
            }
            (HirPattern::Mutable(pattern, _), value) => self.bind_pattern(pattern, value),
            (pattern @ (HirPattern::Tuple(..) | HirPattern::Struct(..)), Value::Struct(exprs)) => {
                assert_eq!(pattern.field_count(), exprs.len());
                for ((pattern_name, pattern), (field_name, value)) in
                    pattern.iter_fields().zip(exprs)
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
            Value::Single(node_id) => {
                let otype = self.context.get_object_type(node_id);
                self.bind_variable(basename.to_owned(), None, otype, node_id)
            }
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
        definition_id: Option<DefinitionId>,
        obj_type: node::ObjectType,
        value_id: NodeId,
    ) -> Value {
        if matches!(obj_type, ObjectType::Pointer(_)) {
            if let Ok(rhs_mut) = self.context.get_mut_variable(value_id) {
                rhs_mut.def = definition_id;
                rhs_mut.name = variable_name;
                return Value::Single(value_id);
            }
        }

        let new_var =
            Variable::new(obj_type, variable_name, definition_id, self.context.current_block);
        let id = self.context.add_variable(new_var, None);

        //Assign rhs to lhs
        let result = self.context.new_binary_instruction(BinaryOp::Assign, id, value_id, obj_type);

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
            nvar.name = format!("{}{}", root_name, variable_id);
        }
    }

    /// Similar to bind_pattern but recursively creates Assignment instructions for
    /// each value rather than defining new variables.
    fn assign_pattern(&mut self, lhs: &Value, rhs: Value) -> Value {
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

                let result_type = self.context[rhs_id].get_type();
                let operation = Operation::binary(BinaryOp::Assign, new_var_id, rhs_id);
                let result = self.context.new_instruction(operation, result_type);
                self.update_variable_id(ls_root, new_var_id, result); //update the name and the value map
                Value::Single(new_var_id)
            }
            (Value::Struct(lhs_fields), Value::Struct(rhs_fields)) => {
                assert_eq!(lhs_fields.len(), rhs_fields.len());
                let f = vecmap(lhs_fields.iter().zip(rhs_fields),
                |(lhs_field, rhs_field)| {
                     assert_eq!(lhs_field.0, rhs_field.0);
                (rhs_field.0, self.assign_pattern(&lhs_field.1, rhs_field.1))

            });
                Value::Struct(f)
            }
            (Value::Single(_), Value::Struct(_)) => unreachable!("variables with tuple/struct types should already be decomposed into multiple variables"),
            (Value::Struct(_), Value::Single(_)) => unreachable!("Uncaught type error, tried to assign a single value to a tuple/struct type"),
        }
    }

    pub fn def_to_name(&self, def: DefinitionId) -> String {
        self.context.context.def_interner.definition_name(def).to_owned()
    }

    pub fn ident_name(&self, ident: &HirIdent) -> String {
        self.def_to_name(ident.id)
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

                let array_id = self.context.mem.create_new_array(arr_lit.length as u32, element_type, &String::new());
                //We parse the array definition
                let elements = self.expression_list_to_objects(env, &arr_lit.contents);
                let array = &mut self.context.mem[array_id];
                let array_adr = array.adr;
                for (pos, object) in elements.into_iter().enumerate() {
                    //array.witness.push(node::get_witness_from_object(&object));
                    let lhs_adr = self.context.get_or_create_const(FieldElement::from((array_adr + pos as u32) as u128), ObjectType::Unsigned(32));
                    let store = Operation::Store {
                        array_id,
                        index: lhs_adr,
                        value: object,
                    };
                    self.context.new_instruction(store, element_type);
                }
                //Finally, we create a variable pointing to this MemArray
                let new_var = node::Variable {
                    id: NodeId::dummy(),
                    obj_type : ObjectType::Pointer(array_id),
                    name: String::new(),
                    root: None,
                    def: None,
                    witness: None,
                    parent_block: self.context.current_block,
                };
                Ok(Value::Single(self.context.add_variable(new_var, None)))
            },
            HirExpression::Ident(x) =>  {
               Ok(self.evaluate_identifier(env, x))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            },
            HirExpression::Infix(infx) => {
                // Note: using .into_id() here disallows structs/tuples in infix expressions.
                // The type checker currently disallows this as well but we may want to allow
                // for e.g. struct == struct in the future
                let lhs = self.expression_to_object(env, &infx.lhs)?.unwrap_id();
                let rhs = self.expression_to_object(env, &infx.rhs)?.unwrap_id();
                Ok(Value::Single(self.evaluate_infix_expression(lhs, rhs, infx.operator)))
            },
            HirExpression::Cast(cast_expr) => {
                let lhs = self.expression_to_object(env, &cast_expr.lhs)?.unwrap_id();
                let rtype = cast_expr.r#type.into();

                Ok(Value::Single(self.context.new_instruction(Operation::Cast(lhs), rtype)))

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
                let collection_name = match self.def_interner().expression(&indexed_expr.collection) {
                    HirExpression::Ident(id) => id,
                    other => todo!("Array indexing with an lhs of '{:?}' is unimplemented, you must use an expression in the form `identifier[expression]` for now.", other)
                };

                let arr_def = collection_name.id;
                let arr_name = self.def_interner().definition_name(arr_def).to_owned();
                let ident_span = collection_name.span;

                let arr_type = self.def_interner().id_type(arr_def);
                let o_type = arr_type.into();

                let array = if let Some(array) = self.context.mem.find_array(arr_def) {
                    array
                } else if let Some(Value::Single(pointer)) = self.find_variable(arr_def) {
                    match self.context.get_object_type(*pointer) {
                        ObjectType::Pointer(array_id) => &self.context.mem[array_id],
                        _ => unreachable!(),
                    }
                } else {
                    let arr = env.get_array(&arr_name).map_err(|kind|kind.add_span(ident_span)).unwrap();
                    self.context.mem.create_array_from_object(&arr, arr_def, o_type, &arr_name)
                };

                let array_id = array.id;
                let address = array.adr;

                // Evaluate the index expression
                let index_as_obj = self.expression_to_object(env, &indexed_expr.index)?.unwrap_id();

                let index_type = self.context.get_object_type(index_as_obj);
                let base_adr = self.context.get_or_create_const(FieldElement::from(address as i128), index_type);
                let adr_id = self.context.new_instruction(Operation::binary(BinaryOp::Add, base_adr, index_as_obj), index_type);

                let load = Operation::Load { array_id, index: adr_id };
                Ok(Value::Single(self.context.new_instruction(load, o_type)))
            },
            HirExpression::Call(call_expr) => {
                let func_meta = self.def_interner().function_meta(&call_expr.func_id);
                match func_meta.kind {
                    FunctionKind::Normal =>  {
                        if self.context.get_ssafunc(call_expr.func_id).is_none() {
                            let func = function::create_function(self, call_expr.func_id, self.context.context(), env, &func_meta.parameters);
                            self.context.functions.insert(call_expr.func_id, func);
                        }

                    //generate a call instruction to the function cfg
                    Ok(Value::Single(function::SSAFunction::call(call_expr.func_id ,&call_expr.arguments, self, env)))
                    },
                    FunctionKind::LowLevel => {
                    // We use it's func name to find out what intrinsic function to call
                    let attribute = func_meta.attributes.expect("all low level functions must contain an attribute which contains the opcode which it links to");
                    let opcode_name = attribute.foreign().expect("ice: function marked as foreign, but attribute kind does not match this");
                    Ok(Value::Single(self.handle_lowlevel(env, opcode_name, call_expr)))
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
            HirExpression::Prefix(prefix) => {
                let rhs = self.expression_to_object(env, &prefix.rhs)?.unwrap_id();
                self.evaluate_prefix_expression(rhs, prefix.operator).map(Value::Single)
            },
            HirExpression::Literal(l) => {
                Ok(Value::Single(self.handle_literal(&l)))
            },
            HirExpression::Block(_) => todo!("currently block expressions not in for/if branches are not being evaluated. In the future, we should be able to unify the eval_block and all places which require block_expr here"),
            HirExpression::Error => todo!(),
            HirExpression::MethodCall(_) => unreachable!("Method calls should be desugared before codegen"),
        }
    }

    pub fn handle_lowlevel(
        &mut self,
        env: &mut Environment,
        opcode_name: &str,
        call_expr: HirCallExpression,
    ) -> NodeId {
        let func = match OPCODE::lookup(opcode_name) {
            None => {
                let message = format!(
                    "cannot find a low level opcode with the name {} in the IR",
                    opcode_name
                );
                unreachable!("{}", message);
            }
            Some(func) => func,
        };
        function::call_low_level(func, call_expr, self, env)
    }

    pub fn handle_literal(&mut self, l: &HirLiteral) -> NodeId {
        match l {
            HirLiteral::Bool(b) => {
                if *b {
                    self.context
                        .get_or_create_const(FieldElement::one(), node::ObjectType::Unsigned(1))
                } else {
                    self.context
                        .get_or_create_const(FieldElement::zero(), node::ObjectType::Unsigned(1))
                }
            }
            HirLiteral::Integer(f) => {
                self.context.get_or_create_const(*f, node::ObjectType::NativeField)
                //TODO support integer literrals in the fronted: 30_u8
            }
            _ => todo!(), //todo: add support for Array(HirArrayLiteral), Str(String)
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
                let field_name = ident.0.contents;
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
        //TODO how should we handle scope (cf. start/end_for_loop)?
        let iter_name = self.def_interner().definition_name(for_expr.identifier.id).to_owned();
        let iter_def = for_expr.identifier.id;
        let int_type = self.def_interner().id_type(iter_def);
        let iter_type = int_type.into();
        let iter_id = self.create_new_variable(iter_name, iter_def, iter_type, None);
        let iter_var = self.context.get_mut_variable(iter_id).unwrap();
        iter_var.obj_type = iter_type;

        let assign = Operation::binary(BinaryOp::Assign, iter_id, start_idx);
        let iter_ass = self.context.new_instruction(assign, iter_type);

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
        let phi = self.context.generate_empty_phi(join_idx, iter_id);
        self.update_variable_id(iter_id, iter_id, phi); //is it still needed?

        let notequal = Operation::binary(BinaryOp::Ne, phi, end_idx);
        let cond = self.context.new_instruction(notequal, ObjectType::Boolean);

        let to_fix = self.context.new_instruction(Operation::Nop, ObjectType::NotAnObject);

        //Body
        let body_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal);
        self.context.try_get_mut_instruction(to_fix).unwrap().operation =
            Operation::Jeq(cond, body_id);

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
        let one = self.context.get_or_create_const(FieldElement::one(), iter_type);

        let incr_op = Operation::binary(BinaryOp::Add, phi, one);
        let incr = self.context.new_instruction(incr_op, iter_type);

        let cur_block_id = self.context.current_block; //It should be the body block, except if the body has CFG statements
        let cur_block = &mut self.context[cur_block_id];
        cur_block.update_variable(iter_id, incr);
        //body.left = join
        cur_block.left = Some(join_idx);
        let join_mut = &mut self.context[join_idx];
        join_mut.predecessor.push(cur_block_id);

        //jump back to join
        self.context.new_instruction(Operation::Jmp(join_idx), ObjectType::NotAnObject);

        //seal join
        ssa_form::seal_block(&mut self.context, join_idx);
        //exit block
        self.context.current_block = exit_id;
        let exit_first = self.context.get_current_block().get_first_instruction();
        block::link_with_target(&mut self.context, join_idx, Some(exit_id), Some(body_id));

        Ok(Value::Single(exit_first)) //TODO what should we return???
    }
}
