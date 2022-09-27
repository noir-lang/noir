use super::context::SsaContext;
use super::function::FuncIndex;
use super::mem::ArrayId;
use super::node::{Binary, BinaryOp, NodeId, ObjectType, Operation, Variable};
use super::{block, node, ssa_form};
use std::collections::HashMap;
use std::convert::TryInto;

use super::super::environment::Environment;
use super::super::errors::RuntimeError;

use crate::ssa::block::BlockType;
use crate::ssa::function;
use acvm::acir::OPCODE;
use acvm::FieldElement;
use noirc_frontend::monomorphisation::ast::*;
use noirc_frontend::util::vecmap;
use noirc_frontend::{BinaryOpKind, UnaryOp};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct IRGenerator {
    pub context: SsaContext,
    pub function_context: Option<FuncIndex>,

    /// The current value of a variable. Used for flattening structs
    /// into multiple variables/values
    variable_values: HashMap<DefinitionId, Value>,

    pub program: Functions,
}

#[derive(Debug, Clone)]
pub enum Value {
    Single(NodeId),
    Tuple(Vec<Value>),
}

impl Value {
    pub fn unwrap_id(&self) -> NodeId {
        match self {
            Value::Single(id) => *id,
            Value::Tuple(_) => panic!("Tried to unwrap a struct into a single value"),
        }
    }

    pub fn dummy() -> Value {
        Value::Single(NodeId::dummy())
    }

    pub fn to_node_ids(&self) -> Vec<NodeId> {
        match self {
            Value::Single(id) => vec![*id],
            Value::Tuple(v) => v.iter().flat_map(|i| i.to_node_ids()).collect(),
        }
    }

    pub fn into_field_member(self, field_index: usize) -> Value {
        match self {
            Value::Single(_) => {
                unreachable!("Runtime type error, expected struct but found a single value")
            }
            Value::Tuple(mut fields) => fields.remove(field_index),
        }
    }

    pub fn get_field_member(&self, field_index: usize) -> &Value {
        match self {
            Value::Single(_) => {
                unreachable!("Runtime type error, expected struct but found a single value")
            }
            Value::Tuple(fields) => &fields[field_index],
        }
    }
}

impl IRGenerator {
    pub fn new(program: Functions) -> IRGenerator {
        IRGenerator {
            context: SsaContext::new(),
            variable_values: HashMap::new(),
            function_context: None,
            program,
        }
    }

    pub fn codegen_main(&mut self, env: &mut Environment) -> Result<(), RuntimeError> {
        let main_body = self.program.take_main_body();
        self.codegen_expression(env, &main_body)?;
        Ok(())
    }

    pub fn find_variable(&self, variable_def: DefinitionId) -> Option<&Value> {
        self.variable_values.get(&variable_def)
    }

    pub fn get_current_value(&mut self, value: &Value) -> Value {
        match value {
            Value::Single(id) => Value::Single(ssa_form::get_current_value(&mut self.context, *id)),
            Value::Tuple(fields) => {
                Value::Tuple(vecmap(fields, |value| self.get_current_value(value)))
            }
        }
    }

    pub fn abi_array(
        &mut self,
        name: &str,
        ident_def: DefinitionId,
        el_type: &noirc_abi::AbiType,
        len: u128,
        witness: Vec<acvm::acir::native_types::Witness>,
    ) {
        let element_type = match el_type {
            noirc_abi::AbiType::Field(_) => ObjectType::NativeField,
            noirc_abi::AbiType::Integer { sign, width, .. } => match sign {
                noirc_abi::Sign::Unsigned => ObjectType::Unsigned(*width),
                noirc_abi::Sign::Signed => ObjectType::Signed(*width),
            },
            noirc_abi::AbiType::Array { .. } => unreachable!(),
        };

        let v_id = self.new_array(name, element_type, len as u32, Some(ident_def));
        let array_idx = self.context.mem.last_id();
        self.context.mem[array_idx].values = vecmap(witness, |w| w.into());
        self.context.get_current_block_mut().update_variable(v_id, v_id);
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

    fn codegen_identifier(&mut self, ident: &Ident) -> Value {
        let value = self.variable_values[&ident.id].clone();
        self.get_current_value(&value)
    }

    fn codegen_prefix_expression(
        &mut self,
        rhs: NodeId,
        op: UnaryOp,
    ) -> Result<NodeId, RuntimeError> {
        let rtype = self.context.get_object_type(rhs);
        match op {
            UnaryOp::Minus => {
                let lhs = self.context.zero_with_type(rtype);
                let operator = BinaryOp::Sub { max_rhs_value: BigUint::zero() };
                let op = Operation::Binary(node::Binary { operator, lhs, rhs });
                self.context.new_instruction(op, rtype)
            }
            UnaryOp::Not => self.context.new_instruction(Operation::Not(rhs), rtype),
        }
    }

    fn codegen_infix_expression(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        op: BinaryOpKind,
    ) -> Result<NodeId, RuntimeError> {
        let ltype = self.context.get_object_type(lhs);
        // Get the opcode from the infix operator
        let opcode = Operation::Binary(Binary::from_ast(op, ltype, lhs, rhs));
        let optype = self.context.get_result_type(&opcode, ltype);
        self.context.new_instruction(opcode, optype)
    }

    fn codegen_indexed_value(
        &mut self,
        array: &LValue,
        index: &Expression,
        env: &mut Environment,
    ) -> Result<(ArrayId, NodeId), RuntimeError> {
        let ident_def = Self::lvalue_ident_def(array);
        let val = self.find_variable(ident_def).unwrap();
        let lhs = val.unwrap_id();

        let a_id = self.context.get_object_type(lhs).type_to_pointer();
        let index = self.codegen_expression(env, index)?.unwrap_id();
        Ok((a_id, index))
    }

    fn lvalue_ident_def(lvalue: &LValue) -> DefinitionId {
        match lvalue {
            LValue::Ident(ident) => ident.id,
            LValue::Index { array, .. } => Self::lvalue_ident_def(array.as_ref()),
            LValue::MemberAccess { object, .. } => Self::lvalue_ident_def(object.as_ref()),
        }
    }

    pub fn create_new_variable(
        &mut self,
        var_name: String,
        def: Option<DefinitionId>,
        obj_type: node::ObjectType,
        witness: Option<acvm::acir::native_types::Witness>,
    ) -> NodeId {
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type,
            name: var_name,
            root: None,
            def,
            witness,
            parent_block: self.context.current_block,
        };
        let v_id = self.context.add_variable(new_var, None);
        let v_value = Value::Single(v_id);
        if let Some(def) = def {
            self.variable_values.insert(def, v_value);
        }
        v_id
    }

    //Helper function for create_new_value()
    fn insert_new_struct(&mut self, def: Option<DefinitionId>, values: Vec<Value>) -> Value {
        let result = Value::Tuple(values);
        if let Some(def_id) = def {
            self.variable_values.insert(def_id, result.clone());
        }
        result
    }

    pub fn create_new_value(
        &mut self,
        typ: &Type,
        base_name: &str,
        def: Option<DefinitionId>,
    ) -> Value {
        match typ {
            Type::Tuple(fields) => {
                let values = vecmap(fields.iter().enumerate(), |(i, field)| {
                    let name = format!("{}.{}", base_name, i);
                    self.create_new_value(field, &name, None)
                });
                self.insert_new_struct(def, values)
            }
            Type::Array(len, _) => {
                //TODO support array of structs
                let obj_type = node::ObjectType::from(typ);
                let len = *len;
                let v_id = self.new_array(base_name, obj_type, len.try_into().unwrap(), def);
                Value::Single(v_id)
            }
            _ => {
                let obj_type = node::ObjectType::from(typ);
                let v_id = self.create_new_variable(base_name.to_string(), def, obj_type, None);
                self.context.get_current_block_mut().update_variable(v_id, v_id);
                Value::Single(v_id)
            }
        }
    }

    pub fn new_array(
        &mut self,
        name: &str,
        element_type: ObjectType,
        len: u32,
        def_id: Option<DefinitionId>,
    ) -> NodeId {
        let id = self.context.new_array(name, element_type, len, def_id);
        if let Some(def) = def_id {
            self.variable_values.insert(def, super::code_gen::Value::Single(id));
        }
        id
    }

    // Add a constraint to constrain two expression together
    fn codegen_constrain(
        &mut self,
        env: &mut Environment,
        expr: &Expression,
        location: noirc_errors::Location,
    ) -> Result<Value, RuntimeError> {
        let cond = self.codegen_expression(env, expr)?.unwrap_id();
        let operation = Operation::Constrain(cond, location);
        self.context.new_instruction(operation, ObjectType::NotAnObject)?;
        Ok(Value::dummy())
    }

    /// Bind the given DefinitionId to the given Value. This will flatten the Value as needed,
    /// expanding each field of the value to a new variable.
    fn bind_id(&mut self, id: DefinitionId, value: Value, name: &str) -> Result<(), RuntimeError> {
        match value {
            Value::Single(node_id) => {
                let otype = self.context.get_object_type(node_id);
                let value = self.bind_variable(name.to_owned(), Some(id), otype, node_id)?;
                self.variable_values.insert(id, value);
            }
            value @ Value::Tuple(_) => {
                let value = self.bind_fresh_pattern(name, value)?;
                self.variable_values.insert(id, value);
            }
        }
        Ok(())
    }

    /// This function is a recursive helper for bind_pattern which takes care
    /// of creating fresh variables to expand `ident = (a, b, ...)` to `(i_a, i_b, ...) = (a, b, ...)`
    ///
    /// This function could use a clearer name
    fn bind_fresh_pattern(&mut self, basename: &str, value: Value) -> Result<Value, RuntimeError> {
        match value {
            Value::Single(node_id) => {
                let otype = self.context.get_object_type(node_id);
                self.bind_variable(basename.to_owned(), None, otype, node_id)
            }
            Value::Tuple(field_values) => {
                let values = field_values
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| {
                        let name = format!("{}.{}", basename, i);
                        self.bind_fresh_pattern(&name, value)
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Value::Tuple(values))
            }
        }
    }

    fn bind_variable(
        &mut self,
        variable_name: String,
        definition_id: Option<DefinitionId>,
        obj_type: node::ObjectType,
        value_id: NodeId,
    ) -> Result<Value, RuntimeError> {
        let id = if let node::ObjectType::Pointer(a) = obj_type {
            let len = self.context.mem[a].len;
            let el_type = self.context.mem[a].element_type;
            self.context.new_array(&variable_name, el_type, len, definition_id)
        } else {
            let new_var =
                Variable::new(obj_type, variable_name, definition_id, self.context.current_block);
            self.context.add_variable(new_var, None)
        };
        //Assign rhs to lhs
        Ok(Value::Single(self.context.handle_assign(id, None, value_id)?))
    }

    //same as update_variable but using the var index instead of var
    pub fn update_variable_id(&mut self, var_id: NodeId, new_var: NodeId, new_value: NodeId) {
        self.context.update_variable_id(var_id, new_var, new_value);
    }

    fn codegen_assign(
        &mut self,
        lvalue: &LValue,
        expression: &Expression,
        env: &mut Environment,
    ) -> Result<Value, RuntimeError> {
        let ident_def = Self::lvalue_ident_def(lvalue);
        let rhs = self.codegen_expression(env, expression)?;

        match lvalue {
            LValue::Ident(_) => {
                let lhs = self.find_variable(ident_def).unwrap();
                // We may be able to avoid cloning here if we change find_variable
                // and assign_pattern to use only fields of self instead of `self` itself.
                let lhs = lhs.clone();
                let result = self.assign_pattern(&lhs, rhs)?;
                self.variable_values.insert(ident_def, result);
            }
            LValue::Index { array, index } => {
                let (_, array_idx) = self.codegen_indexed_value(array.as_ref(), index, env)?;
                let val = self.find_variable(ident_def).unwrap();
                let rhs_id = rhs.unwrap_id();
                let lhs_id = val.unwrap_id();
                self.context.handle_assign(lhs_id, Some(array_idx), rhs_id)?;
            }
            LValue::MemberAccess { object: _, field_index } => {
                // TODO: This is incorrect for nested structs
                let val = self.find_variable(ident_def).unwrap();
                let value = val.get_field_member(*field_index).clone();
                self.assign_pattern(&value, rhs)?;
            }
        }
        Ok(Value::dummy())
    }

    /// Similar to bind_pattern but recursively creates Assignment instructions for
    /// each value rather than defining new variables.
    fn assign_pattern(&mut self, lhs: &Value, rhs: Value) -> Result<Value, RuntimeError> {
        match (lhs, rhs) {
            (Value::Single(lhs_id), Value::Single(rhs_id)) => {
                Ok(Value::Single(self.context.handle_assign(*lhs_id, None, rhs_id)?))
            }
            (Value::Tuple(lhs_fields), Value::Tuple(rhs_fields)) => {
                assert_eq!(lhs_fields.len(), rhs_fields.len());
                let fields = lhs_fields.iter().zip(rhs_fields).map(|(lhs_field, rhs_field)| {
                    self.assign_pattern(lhs_field, rhs_field)
                }).collect::<Result<Vec<_>, _>>()?;

                Ok(Value::Tuple(fields))
            }
            (Value::Single(_), Value::Tuple(_)) => unreachable!("variables with tuple/struct types should already be decomposed into multiple variables"),
            (Value::Tuple(_), Value::Single(_)) => unreachable!("Uncaught type error, tried to assign a single value to a tuple/struct type"),
        }
    }

    // Let statements are used to declare higher level objects
    fn codegen_let(
        &mut self,
        env: &mut Environment,
        let_expr: &Let,
    ) -> Result<Value, RuntimeError> {
        let rhs = self.codegen_expression(env, &let_expr.expression)?;
        self.bind_id(let_expr.id, rhs, &let_expr.name)?;
        Ok(Value::dummy())
    }

    pub(crate) fn codegen_expression(
        &mut self,
        env: &mut Environment,
        expr: &Expression,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Literal(Literal::Integer(x, typ)) => {
                Ok(Value::Single(self.context.get_or_create_const(*x, typ.into())))
            }
            Expression::Literal(Literal::Array(arr_lit)) => {
                let element_type = ObjectType::from(&arr_lit.element_type);
                let new_var = self.context.new_array("", element_type, arr_lit.length as u32, None);
                let array_id = self.context.mem.last_id();

                //We parse the array definition
                let elements = self.codegen_expression_list(env, &arr_lit.contents);
                for (pos, object) in elements.into_iter().enumerate() {
                    let lhs_adr = self.context.get_or_create_const(
                        FieldElement::from((pos as u32) as u128),
                        ObjectType::NativeField,
                    );
                    let store = Operation::Store { array_id, index: lhs_adr, value: object };
                    self.context.new_instruction(store, element_type)?;
                }
                Ok(Value::Single(new_var))
            }
            Expression::Ident(ident) => {
                Ok(self.codegen_identifier(ident))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            }
            Expression::Binary(binary) => {
                // Note: using .into_id() here disallows structs/tuples in infix expressions.
                // The type checker currently disallows this as well but we may want to allow
                // for e.g. struct == struct in the future
                let lhs = self.codegen_expression(env, &binary.lhs)?.unwrap_id();
                let rhs = self.codegen_expression(env, &binary.rhs)?.unwrap_id();
                Ok(Value::Single(self.codegen_infix_expression(lhs, rhs, binary.operator)?))
            }
            Expression::Cast(cast_expr) => {
                let lhs = self.codegen_expression(env, &cast_expr.lhs)?.unwrap_id();
                let rtype = ObjectType::from(&cast_expr.r#type);

                Ok(Value::Single(self.context.new_instruction(Operation::Cast(lhs), rtype)?))

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
            }
            Expression::Index(indexed_expr) => {
                // Currently these only happen for arrays
                let collection = match indexed_expr.collection.as_ref() {
                    Expression::Ident(id) => id,
                    other => todo!("Array indexing with an lhs of '{:?}' is unimplemented, you must use an expression in the form `identifier[expression]` for now.", other)
                };

                let arr_def = collection.id;
                let o_type = ObjectType::from(&collection.typ);
                let e_type = o_type.deref(&self.context);

                let array = if let Some(array) = self.context.mem.find_array(arr_def) {
                    array
                } else if let Some(Value::Single(pointer)) = self.find_variable(arr_def) {
                    match self.context.get_object_type(*pointer) {
                        ObjectType::Pointer(array_id) => &self.context.mem[array_id],
                        other => unreachable!("Expected Pointer type, found {:?}", other),
                    }
                } else {
                    let arr = env.get_array(&collection.name).unwrap();
                    self.context.create_array_from_object(&arr, arr_def, o_type, &collection.name);
                    let array_id = self.context.mem.last_id();
                    &self.context.mem[array_id]
                };

                let array_id = array.id;
                // Evaluate the index expression
                let index_as_obj = self.codegen_expression(env, &indexed_expr.index)?.unwrap_id();
                let load = Operation::Load { array_id, index: index_as_obj };
                Ok(Value::Single(self.context.new_instruction(load, e_type)?))
            }
            Expression::Call(call_expr) => {
                if self.context.get_ssafunc(call_expr.func_id).is_none() {
                    let index = self.context.get_function_index();
                    self.create_function(call_expr.func_id, env, index)?;
                }

                let callee = self.context.get_ssafunc(call_expr.func_id).unwrap().idx;
                //generate a call instruction to the function cfg
                if let Some(caller) = self.function_context {
                    function::update_call_graph(&mut self.context.call_graph, caller, callee);
                }
                let results = self.call(call_expr, env)?;

                let function = &self.program[call_expr.func_id];
                Ok(match &function.return_type {
                    Type::Tuple(_) => Value::Tuple(vecmap(results, Value::Single)),
                    _ => {
                        assert_eq!(results.len(), 1);
                        Value::Single(results[0])
                    }
                })
            }
            Expression::CallLowLevel(call) => Ok(Value::Single(self.codegen_lowlevel(env, call)?)),
            Expression::CallBuiltin(_call) => {
                todo!()
                // let attribute = func_meta.attributes.expect("all builtin functions must contain an attribute which contains the function name which it links to");
                // let builtin_name = attribute.builtin().expect("ice: function marked as a builtin, but attribute kind does not match this");
                // builtin::call_builtin(self, env, builtin_name, (call_expr,span))
            }
            Expression::For(for_expr) => self.codegen_for(env, for_expr),
            Expression::Tuple(fields) => self.codegen_tuple(env, fields),
            Expression::If(if_expr) => self.handle_if_expr(env, if_expr),
            Expression::Unary(prefix) => {
                let rhs = self.codegen_expression(env, &prefix.rhs)?.unwrap_id();
                self.codegen_prefix_expression(rhs, prefix.operator).map(Value::Single)
            }
            Expression::Literal(l) => Ok(Value::Single(self.codegen_literal(l))),
            Expression::Block(block) => self.codegen_block(block, env),
            Expression::ExtractTupleField(expr, field) => {
                let tuple = self.codegen_expression(env, expr.as_ref())?;
                Ok(tuple.into_field_member(*field))
            }
            Expression::Let(let_expr) => self.codegen_let(env, let_expr),
            Expression::Constrain(expr, location) => {
                self.codegen_constrain(env, expr.as_ref(), *location)
            }
            Expression::Assign(assign) => {
                self.codegen_assign(&assign.lvalue, assign.expression.as_ref(), env)
            }
            Expression::Semi(expr) => {
                self.codegen_expression(env, expr.as_ref())?;
                Ok(Value::dummy())
            }
        }
    }

    fn codegen_lowlevel(
        &mut self,
        env: &mut Environment,
        call: &CallLowLevel,
    ) -> Result<NodeId, RuntimeError> {
        match OPCODE::lookup(&call.opcode) {
            Some(func) => self.call_low_level(func, call, env),
            None => {
                unreachable!(
                    "cannot find a low level opcode with the name {} in the IR",
                    &call.opcode
                )
            }
        }
    }

    fn codegen_literal(&mut self, l: &Literal) -> NodeId {
        match l {
            Literal::Bool(b) => {
                if *b {
                    self.context.one()
                } else {
                    self.context.zero()
                }
            }
            Literal::Integer(f, typ) => self.context.get_or_create_const(*f, typ.into()),
            _ => todo!(), //todo: add support for Array(ArrayLiteral), Str(String)
        }
    }

    /// A tuple is much the same as a constructor, we just give it fields with numbered names
    fn codegen_tuple(
        &mut self,
        env: &mut Environment,
        fields: &[Expression],
    ) -> Result<Value, RuntimeError> {
        let fields = fields
            .iter()
            .map(|field| self.codegen_expression(env, field))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Value::Tuple(fields))
    }

    pub fn codegen_expression_list(
        &mut self,
        env: &mut Environment,
        exprs: &[Expression],
    ) -> Vec<NodeId> {
        let mut result = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let value = self.codegen_expression(env, expr);
            result.extend(value.unwrap().to_node_ids());
        }
        result
    }

    fn codegen_for(
        &mut self,
        env: &mut Environment,
        for_expr: &For,
    ) -> Result<Value, RuntimeError> {
        //we add the 'i = start' instruction (in the block before the join)
        let start_idx = self.codegen_expression(env, &for_expr.start_range).unwrap().unwrap_id();
        let end_idx = self.codegen_expression(env, &for_expr.end_range).unwrap().unwrap_id();

        //We support only const range for now
        //TODO how should we handle scope (cf. start/end_for_loop)?
        let iter_def = for_expr.index_variable;
        let iter_type = ObjectType::from(&for_expr.index_type);
        let index_name = for_expr.index_name.clone();

        let iter_id = self.create_new_variable(index_name, Some(iter_def), iter_type, None);
        let iter_var = self.context.get_mut_variable(iter_id).unwrap();
        iter_var.obj_type = iter_type;

        let assign = Operation::binary(BinaryOp::Assign, iter_id, start_idx);
        let iter_ass = self.context.new_instruction(assign, iter_type)?;

        //We map the iterator to start_idx so that when we seal the join block, we will get the correct value.
        self.update_variable_id(iter_id, iter_ass, start_idx);

        //join block
        let join_idx =
            block::new_unsealed_block(&mut self.context, block::BlockType::ForJoin, true);
        let exit_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);
        self.context.current_block = join_idx;

        //should parse a for_expr.condition statement that should evaluate to bool, but
        //we only supports i=start;i!=end for now
        //we generate the phi for the iterator because the iterator is manually created
        let phi = self.context.generate_empty_phi(join_idx, iter_id);
        self.update_variable_id(iter_id, iter_id, phi); //is it still needed?

        let notequal = Operation::binary(BinaryOp::Ne, phi, end_idx);
        let cond = self.context.new_instruction(notequal, ObjectType::Boolean)?;

        let to_fix = self.context.new_instruction(Operation::Nop, ObjectType::NotAnObject)?;

        //Body
        let body_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);
        self.context.try_get_mut_instruction(to_fix).unwrap().operation =
            Operation::Jeq(cond, body_id);

        let body_block1 = &mut self.context[body_id];
        body_block1.update_variable(iter_id, phi); //TODO try with just a get_current_value(iter)

        self.codegen_expression(env, for_expr.block.as_ref())?;

        //increment iter
        let one = self.context.get_or_create_const(FieldElement::one(), iter_type);

        let incr_op = Operation::binary(BinaryOp::Add, phi, one);
        let incr = self.context.new_instruction(incr_op, iter_type)?;

        let cur_block_id = self.context.current_block; //It should be the body block, except if the body has CFG statements
        let cur_block = &mut self.context[cur_block_id];
        cur_block.update_variable(iter_id, incr);

        //body.left = join
        cur_block.left = Some(join_idx);
        let join_mut = &mut self.context[join_idx];
        join_mut.predecessor.push(cur_block_id);

        //jump back to join
        self.context.new_instruction(Operation::Jmp(join_idx), ObjectType::NotAnObject)?;

        //seal join
        ssa_form::seal_block(&mut self.context, join_idx);

        //exit block
        self.context.current_block = exit_id;
        let exit_first = self.context.get_current_block().get_first_instruction();
        block::link_with_target(&mut self.context, join_idx, Some(exit_id), Some(body_id));

        Ok(Value::Single(exit_first)) //TODO what should we return???
    }

    //Parse a block of AST statements into ssa form
    pub fn codegen_block(
        &mut self,
        block: &[Expression],
        env: &mut Environment,
    ) -> Result<Value, RuntimeError> {
        let mut last_value = Value::dummy();
        for expr in block {
            last_value = self.codegen_expression(env, expr)?;
        }
        Ok(last_value)
    }

    fn handle_if_expr(
        &mut self,
        env: &mut Environment,
        if_expr: &If,
    ) -> Result<Value, RuntimeError> {
        //jump instruction
        let mut entry_block = self.context.current_block;
        if self.context[entry_block].kind != BlockType::Normal {
            entry_block =
                block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);
        }

        let condition = self.codegen_expression(env, if_expr.condition.as_ref())?.unwrap_id();

        if let Some(cond) = node::NodeEval::from_id(&self.context, condition).into_const_value() {
            if cond.is_zero() {
                if let Some(alt) = &if_expr.alternative {
                    return self.codegen_expression(env, alt);
                } else {
                    return Ok(Value::dummy());
                }
            } else {
                return self.codegen_expression(env, if_expr.consequence.as_ref());
            }
        }

        let jump_op = Operation::Jeq(condition, block::BlockId::dummy());
        let jump_ins = self.context.new_instruction(jump_op, ObjectType::NotAnObject).unwrap();

        //Then block
        block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);

        self.codegen_expression(env, if_expr.consequence.as_ref())?;

        //Exit block
        let exit_block =
            block::new_unsealed_block(&mut self.context, block::BlockType::IfJoin, true);

        self.context[entry_block].dominated.push(exit_block);

        //Else block
        self.context.current_block = entry_block;
        let block2 = block::new_sealed_block(&mut self.context, block::BlockType::Normal, false);
        self.context[entry_block].right = Some(block2);

        //Fixup the jump
        if let node::Instruction { operation: Operation::Jeq(_, target), .. } =
            self.context.get_mut_instruction(jump_ins)
        {
            *target = block2;
        }

        if let Some(alt) = if_expr.alternative.as_ref() {
            self.codegen_expression(env, alt)?;
        }

        //Connect with the exit block
        self.context.get_current_block_mut().left = Some(exit_block);

        //Exit block plumbing
        self.context.current_block = exit_block;
        self.context.get_current_block_mut().predecessor.push(block2);
        ssa_form::seal_block(&mut self.context, exit_block);

        // TODO: This return value is wrong! We should be returning a PHI of the then and else blocks
        Ok(Value::dummy())
    }
}
