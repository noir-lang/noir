use super::context::SsaContext;
use super::function::FuncIndex;
use super::mem::ArrayId;
use super::node::{Binary, BinaryOp, NodeId, ObjectType, Operation, Variable};
use super::{block, node, ssa_form};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;

use super::super::errors::RuntimeError;

use crate::errors;
use crate::ssa::block::BlockType;
use crate::ssa::function;
use acvm::FieldElement;
use iter_extended::vecmap;
use noirc_frontend::monomorphisation::ast::*;
use noirc_frontend::{BinaryOpKind, UnaryOp};
use num_bigint::BigUint;
use num_traits::Zero;

pub struct IRGenerator {
    pub context: SsaContext,
    pub function_context: Option<FuncIndex>,

    /// The current value of a variable. Used for flattening structs
    /// into multiple variables/values
    variable_values: HashMap<DefinitionId, Value>,

    pub program: Program,
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

    pub fn is_dummy(&self) -> bool {
        match self {
            Value::Single(id) => *id == NodeId::dummy(),
            _ => false,
        }
    }

    pub fn to_node_ids(&self) -> Vec<NodeId> {
        match self {
            Value::Single(id) => vec![*id],
            Value::Tuple(v) => v.iter().flat_map(|i| i.to_node_ids()).collect(),
        }
    }

    pub fn zip<F>(&self, v2: &Value, f: &mut F) -> Value
    where
        F: FnMut(NodeId, NodeId) -> NodeId,
    {
        if self.is_dummy() || v2.is_dummy() {
            return Value::dummy();
        }

        match (self, v2) {
            (Value::Single(id1), Value::Single(id2)) => Value::Single(f(*id1, *id2)),
            (Value::Tuple(tup1), Value::Tuple(tup2)) => {
                Value::Tuple(vecmap(tup1.iter().zip(tup2), |(v1, v2)| v1.zip(v2, f)))
            }
            _ => unreachable!(),
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

    //Reconstruct a value whose type is provided in argument, from a bunch of NodeIds
    fn reshape(value_type: &Type, iter: &mut core::slice::Iter<NodeId>) -> Value {
        match value_type {
            Type::Tuple(tup) => {
                let values = vecmap(tup, |v| Self::reshape(v, iter));
                Value::Tuple(values)
            }
            _ => Value::Single(*iter.next().unwrap()),
        }
    }

    fn from_slice(value_type: &Type, slice: &[NodeId]) -> Value {
        let mut iter = slice.iter();
        let result = Value::reshape(value_type, &mut iter);
        assert!(iter.next().is_none());
        result
    }
}

impl IRGenerator {
    pub fn new(program: Program) -> IRGenerator {
        IRGenerator {
            context: SsaContext::new(),
            variable_values: HashMap::new(),
            function_context: None,
            program,
        }
    }

    pub fn codegen_main(&mut self) -> Result<(), RuntimeError> {
        let main_body = self.program.take_main_body();
        self.codegen_expression(&main_body)?;
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

    pub fn get_object_type_from_abi(&self, el_type: &noirc_abi::AbiType) -> ObjectType {
        match el_type {
            noirc_abi::AbiType::Field => ObjectType::NativeField,
            noirc_abi::AbiType::Integer { sign, width, .. } => match sign {
                noirc_abi::Sign::Unsigned => ObjectType::Unsigned(*width),
                noirc_abi::Sign::Signed => ObjectType::Signed(*width),
            },
            noirc_abi::AbiType::Array { .. } => {
                unreachable!("array of arrays are not supported for now")
            }
            noirc_abi::AbiType::Struct { .. } => {
                unreachable!("array of structs are not supported for now")
            }
            noirc_abi::AbiType::String { .. } => {
                unreachable!("array of strings are not supported for now")
            }
        }
    }

    pub fn abi_array(
        &mut self,
        name: &str,
        ident_def: Option<DefinitionId>,
        el_type: &noirc_abi::AbiType,
        len: u64,
        witness: Vec<acvm::acir::native_types::Witness>,
    ) -> NodeId {
        let element_type = self.get_object_type_from_abi(el_type);
        let (v_id, array_idx) = self.new_array(name, element_type, len as u32, ident_def);
        self.context.mem[array_idx].values = vecmap(witness, |w| w.into());
        self.context.get_current_block_mut().update_variable(v_id, v_id);
        v_id
    }

    pub fn abi_struct(
        &mut self,
        struct_name: &str,
        ident_def: Option<DefinitionId>,
        fields: &BTreeMap<String, noirc_abi::AbiType>,
        witnesses: BTreeMap<String, Vec<acvm::acir::native_types::Witness>>,
    ) -> Value {
        let values = vecmap(fields, |(name, field_typ)| {
            let new_name = format!("{struct_name}.{name}");
            match field_typ {
                noirc_abi::AbiType::Array { length, typ } => {
                    let v_id =
                        self.abi_array(&new_name, None, typ, *length, witnesses[&new_name].clone());
                    Value::Single(v_id)
                }
                noirc_abi::AbiType::Struct { fields, .. } => {
                    let new_name = format!("{struct_name}.{name}");
                    self.abi_struct(&new_name, None, fields, witnesses.clone())
                }
                _ => {
                    let obj_type = self.get_object_type_from_abi(field_typ);
                    let v_id = self.create_new_variable(
                        new_name.clone(),
                        None,
                        obj_type,
                        Some(witnesses[&new_name][0]),
                    );
                    Value::Single(v_id)
                }
            }
        });
        self.insert_new_struct(ident_def, values)
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
                let op = Operation::Binary(node::Binary { operator, lhs, rhs, predicate: None });
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
    ) -> Result<(NodeId, NodeId), RuntimeError> {
        let value = self.lvalue_to_value(array);
        let lhs = value.unwrap_id();
        let index = self.codegen_expression(index)?.unwrap_id();
        Ok((lhs, index))
    }

    fn lvalue_to_value(&self, lvalue: &LValue) -> &Value {
        match lvalue {
            LValue::Ident(ident) => self.find_variable(ident.id).unwrap(),
            LValue::Index { array, .. } => {
                self.find_variable(Self::lvalue_ident_def(array.as_ref())).unwrap()
            }
            LValue::MemberAccess { object, field_index, .. } => {
                let ident_def = Self::lvalue_ident_def(object.as_ref());
                let val = self.find_variable(ident_def).unwrap();
                val.get_field_member(*field_index)
            }
        }
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
                    let name = format!("{base_name}.{i}");
                    self.create_new_value(field, &name, None)
                });
                self.insert_new_struct(def, values)
            }
            Type::Array(len, _) => {
                //TODO support array of structs
                let obj_type = node::ObjectType::from(typ);
                let len = *len;
                let (v_id, _) = self.new_array(base_name, obj_type, len.try_into().unwrap(), def);
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
    ) -> (NodeId, ArrayId) {
        let (id, array_id) = self.context.new_array(name, element_type, len, def_id);
        if let Some(def) = def_id {
            self.variable_values.insert(def, super::code_gen::Value::Single(id));
        }
        (id, array_id)
    }

    // Add a constraint to constrain two expression together
    fn codegen_constrain(
        &mut self,
        expr: &Expression,
        location: noirc_errors::Location,
    ) -> Result<Value, RuntimeError> {
        let cond = self.codegen_expression(expr)?.unwrap_id();
        let operation = Operation::Constrain(cond, Some(location));
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
                        let name = format!("{basename}.{i}");
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
            self.context.new_array(&variable_name, el_type, len, definition_id).0
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
    ) -> Result<Value, RuntimeError> {
        let ident_def = Self::lvalue_ident_def(lvalue);
        let rhs = self.codegen_expression(expression)?;

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
                let (lhs_id, array_idx) = self.codegen_indexed_value(array.as_ref(), index)?;
                let rhs_id = rhs.unwrap_id();
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
    fn codegen_let(&mut self, let_expr: &Let) -> Result<Value, RuntimeError> {
        let rhs = self.codegen_expression(&let_expr.expression)?;
        self.bind_id(let_expr.id, rhs, &let_expr.name)?;
        Ok(Value::dummy())
    }

    pub(crate) fn codegen_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Literal(Literal::Integer(x, typ)) => {
                Ok(Value::Single(self.context.get_or_create_const(*x, typ.into())))
            }
            Expression::Literal(Literal::Array(arr_lit)) => {
                let element_type = ObjectType::from(&arr_lit.element_type);

                let (new_var, array_id) =
                    self.context.new_array("", element_type, arr_lit.contents.len() as u32, None);

                let elements = self.codegen_expression_list(&arr_lit.contents);
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
            Expression::Literal(Literal::Str(string)) => {
                let string_as_integers = string
                    .bytes()
                    .into_iter()
                    .map(|byte| {
                        let f = FieldElement::from_be_bytes_reduce(&[byte]);
                        Expression::Literal(Literal::Integer(
                            f,
                            Type::Integer(noirc_frontend::Signedness::Unsigned, 8),
                        ))
                    })
                    .collect::<Vec<_>>();

                let string_arr_literal = ArrayLiteral {
                    contents: string_as_integers,
                    element_type: Type::Integer(noirc_frontend::Signedness::Unsigned, 8),
                };

                let new_value = self
                    .codegen_expression(&Expression::Literal(Literal::Array(string_arr_literal)))?;
                Ok(new_value)
            }
            Expression::Ident(ident) => {
                Ok(self.codegen_identifier(ident))
                //n.b this creates a new variable if it does not exist, may be we should delegate this to explicit statements (let) - TODO
            }
            Expression::Binary(binary) => {
                // Note: we disallows structs/tuples in infix expressions.
                // The type checker currently disallows this as well but not if they come from generic type
                // We could allow some in the future, e.g. struct == struct
                let lhs = self.codegen_expression(&binary.lhs)?.to_node_ids();
                let rhs = self.codegen_expression(&binary.rhs)?.to_node_ids();
                if lhs.len() != 1 || rhs.len() != 1 {
                    return Err(errors::RuntimeErrorKind::UnsupportedOp {
                        op: binary.operator.to_string(),
                        first_type: "struct/tuple".to_string(),
                        second_type: "struct/tuple".to_string(),
                    }
                    .into());
                }
                Ok(Value::Single(self.codegen_infix_expression(lhs[0], rhs[0], binary.operator)?))
            }
            Expression::Cast(cast_expr) => {
                let lhs = self.codegen_expression(&cast_expr.lhs)?.unwrap_id();
                let rtype = ObjectType::from(&cast_expr.r#type);

                Ok(Value::Single(self.context.new_instruction(Operation::Cast(lhs), rtype)?))
            }
            Expression::Index(indexed_expr) => {
                // Evaluate the 'array' expression
                let expr_node = self.codegen_expression(&indexed_expr.collection)?.unwrap_id();
                let array = match self.context.get_object_type(expr_node) {
                    ObjectType::Pointer(array_id) => &self.context.mem[array_id],
                    other => unreachable!("Expected Pointer type, found {:?}", other),
                };
                let array_id = array.id;
                let e_type = array.element_type;
                // Evaluate the index expression
                let index_as_obj = self.codegen_expression(&indexed_expr.index)?.unwrap_id();
                let load = Operation::Load { array_id, index: index_as_obj };
                Ok(Value::Single(self.context.new_instruction(load, e_type)?))
            }
            Expression::Call(call_expr) => {
                if self.context.get_ssafunc(call_expr.func_id).is_none() {
                    let index = self.context.get_function_index();
                    self.create_function(call_expr.func_id, index)?;
                }

                let callee = self.context.get_ssafunc(call_expr.func_id).unwrap().idx;
                //generate a call instruction to the function cfg
                if let Some(caller) = self.function_context {
                    function::update_call_graph(&mut self.context.call_graph, caller, callee);
                }

                let results = self.call(call_expr)?;

                let function = &self.program[call_expr.func_id];
                Ok(Value::from_slice(&function.return_type, &results))
            }
            Expression::CallLowLevel(call) => Ok(Value::Single(self.codegen_lowlevel(call)?)),
            Expression::CallBuiltin(call) => {
                let call =
                    CallLowLevel { opcode: call.opcode.clone(), arguments: call.arguments.clone() };
                Ok(Value::Single(self.codegen_lowlevel(&call)?))
            }
            Expression::For(for_expr) => self.codegen_for(for_expr),
            Expression::Tuple(fields) => self.codegen_tuple(fields),
            Expression::If(if_expr) => self.handle_if_expr(if_expr),
            Expression::Unary(prefix) => {
                let rhs = self.codegen_expression(&prefix.rhs)?.unwrap_id();
                self.codegen_prefix_expression(rhs, prefix.operator).map(Value::Single)
            }
            Expression::Literal(l) => Ok(Value::Single(self.codegen_literal(l))),
            Expression::Block(block) => self.codegen_block(block),
            Expression::ExtractTupleField(expr, field) => {
                let tuple = self.codegen_expression(expr.as_ref())?;
                Ok(tuple.into_field_member(*field))
            }
            Expression::Let(let_expr) => self.codegen_let(let_expr),
            Expression::Constrain(expr, location) => {
                self.codegen_constrain(expr.as_ref(), *location)
            }
            Expression::Assign(assign) => {
                self.codegen_assign(&assign.lvalue, assign.expression.as_ref())
            }
            Expression::Semi(expr) => {
                self.codegen_expression(expr.as_ref())?;
                Ok(Value::dummy())
            }
        }
    }

    fn codegen_lowlevel(&mut self, call: &CallLowLevel) -> Result<NodeId, RuntimeError> {
        match super::builtin::Opcode::lookup(&call.opcode) {
            Some(func) => self.call_low_level(func, call),
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
    fn codegen_tuple(&mut self, fields: &[Expression]) -> Result<Value, RuntimeError> {
        let fields = fields
            .iter()
            .map(|field| self.codegen_expression(field))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Value::Tuple(fields))
    }

    pub fn codegen_expression_list(&mut self, exprs: &[Expression]) -> Vec<NodeId> {
        let mut result = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let value = self.codegen_expression(expr);
            result.extend(value.unwrap().to_node_ids());
        }
        result
    }

    fn codegen_for(&mut self, for_expr: &For) -> Result<Value, RuntimeError> {
        //we add the 'i = start' instruction (in the block before the join)
        let start_idx = self.codegen_expression(&for_expr.start_range).unwrap().unwrap_id();
        let end_idx = self.codegen_expression(&for_expr.end_range).unwrap().unwrap_id();

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

        self.codegen_expression(for_expr.block.as_ref())?;

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

        //exit block
        self.context.current_block = exit_id;
        let exit_first = self.context.get_current_block().get_first_instruction();
        block::link_with_target(&mut self.context, join_idx, Some(exit_id), Some(body_id));

        //seal join
        ssa_form::seal_block(&mut self.context, join_idx, join_idx);

        Ok(Value::Single(exit_first)) //TODO what should we return???
    }

    //Parse a block of AST statements into ssa form
    pub fn codegen_block(&mut self, block: &[Expression]) -> Result<Value, RuntimeError> {
        let mut last_value = Value::dummy();
        for expr in block {
            last_value = self.codegen_expression(expr)?;
        }
        Ok(last_value)
    }

    fn handle_if_expr(&mut self, if_expr: &If) -> Result<Value, RuntimeError> {
        //jump instruction
        let mut entry_block = self.context.current_block;
        if self.context[entry_block].kind != BlockType::Normal {
            entry_block =
                block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);
        }

        let condition = self.codegen_expression(if_expr.condition.as_ref())?.unwrap_id();

        if let Some(cond) = node::NodeEval::from_id(&self.context, condition).into_const_value() {
            if cond.is_zero() {
                if let Some(alt) = &if_expr.alternative {
                    return self.codegen_expression(alt);
                } else {
                    return Ok(Value::dummy());
                }
            } else {
                return self.codegen_expression(if_expr.consequence.as_ref());
            }
        }

        let jump_op = Operation::Jeq(condition, block::BlockId::dummy());
        let jump_ins = self.context.new_instruction(jump_op, ObjectType::NotAnObject).unwrap();

        //Then block
        block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);

        let v1 = self.codegen_expression(if_expr.consequence.as_ref())?;

        //Exit block
        let exit_block =
            block::new_unsealed_block(&mut self.context, block::BlockType::IfJoin, true);
        self.context[exit_block].dominator = Some(entry_block);

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

        let mut v2 = Value::dummy();
        if let Some(alt) = if_expr.alternative.as_ref() {
            v2 = self.codegen_expression(alt)?;
        }

        //Connect with the exit block
        self.context.get_current_block_mut().left = Some(exit_block);

        //Exit block plumbing
        self.context.current_block = exit_block;
        self.context.get_current_block_mut().predecessor.push(block2);
        ssa_form::seal_block(&mut self.context, exit_block, entry_block);

        // return value:
        let mut counter = 0;
        let mut phi = |a, b| self.context.new_phi(a, b, &mut counter);
        Ok(v1.zip(&v2, &mut phi))
    }
}
