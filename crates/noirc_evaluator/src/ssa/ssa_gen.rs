use crate::{
    errors::{RuntimeError, RuntimeErrorKind},
    ssa::{
        block::BlockType,
        context::SsaContext,
        function::FuncIndex,
        mem::ArrayId,
        node::{Binary, BinaryOp, NodeId, ObjectType, Operation, Variable},
        value::Value,
        {block, builtin, node, ssa_form},
    },
};
use acvm::{acir::native_types::Witness, FieldElement};
use iter_extended::vecmap;
use noirc_errors::Location;
use noirc_frontend::{
    monomorphization::ast::{
        ArrayLiteral, Definition, Expression, For, Ident, If, LValue, Let, Literal, LocalId,
        Program, Type,
    },
    BinaryOpKind, UnaryOp,
};
use num_bigint::BigUint;
use num_traits::Zero;
use std::collections::{BTreeMap, HashMap};

pub(crate) struct IrGenerator {
    pub(crate) context: SsaContext,
    pub(crate) function_context: Option<FuncIndex>,

    /// The current value of a variable. Used for flattening structs
    /// into multiple variables/values
    variable_values: HashMap<Definition, Value>,

    pub(crate) program: Program,
}

impl IrGenerator {
    pub(crate) fn new(program: Program) -> IrGenerator {
        IrGenerator {
            context: SsaContext::default(),
            variable_values: HashMap::new(),
            function_context: None,
            program,
        }
    }

    pub(crate) fn ssa_gen_main(&mut self) -> Result<(), RuntimeError> {
        let main_body = self.program.take_main_body();
        let value = self.ssa_gen_expression(&main_body)?;
        let node_ids = value.to_node_ids();

        if self.program.main().return_type != Type::Unit {
            self.context.new_instruction(Operation::Return(node_ids), ObjectType::NotAnObject)?;
        }
        Ok(())
    }

    pub(crate) fn find_variable(&self, variable_def: &Definition) -> Option<&Value> {
        self.variable_values.get(variable_def)
    }

    /// Returns the ssa value of a variable
    /// This method constructs the ssa value of a variable, while parsing the AST, using value numbering
    /// This is why it requires a mutable SsaContext
    pub(crate) fn get_current_value(&mut self, value: &Value) -> Value {
        match value {
            Value::Node(id) => Value::Node(ssa_form::get_current_value(&mut self.context, *id)),
            Value::Tuple(fields) => {
                Value::Tuple(vecmap(fields, |value| self.get_current_value(value)))
            }
        }
    }

    pub(crate) fn get_object_type_from_abi(&self, el_type: &noirc_abi::AbiType) -> ObjectType {
        match el_type {
            noirc_abi::AbiType::Field => ObjectType::native_field(),
            noirc_abi::AbiType::Integer { sign, width, .. } => match sign {
                noirc_abi::Sign::Unsigned => ObjectType::unsigned_integer(*width),
                noirc_abi::Sign::Signed => ObjectType::signed_integer(*width),
            },
            noirc_abi::AbiType::Boolean => ObjectType::boolean(),
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

    pub(crate) fn abi_array(
        &mut self,
        name: &str,
        ident_def: Option<Definition>,
        el_type: &noirc_abi::AbiType,
        len: u64,
        witness: &[Witness],
    ) -> NodeId {
        let element_type = self.get_object_type_from_abi(el_type);
        let (v_id, array_idx) = self.new_array(name, element_type, len as u32, ident_def);
        let values = vecmap(witness.iter().enumerate(), |(i, w)| {
            let mut var = Variable::new(
                element_type,
                format!("{name}_{i}"),
                None,
                self.context.current_block,
            );
            var.witness = Some(*w);
            self.context.add_variable(var, None)
        });
        let mut stack_frame = crate::ssa::inline::StackFrame::new(self.context.current_block);
        self.context.init_array_from_values(array_idx, values, &mut stack_frame);
        let block = self.context.get_current_block_mut();
        block.instructions.extend_from_slice(&stack_frame.stack);
        block.update_variable(v_id, v_id);
        v_id
    }

    pub(crate) fn abi_struct(
        &mut self,
        struct_name: &str,
        ident_def: Option<Definition>,
        fields: &[(String, noirc_abi::AbiType)],
        witnesses: &BTreeMap<String, Vec<Witness>>,
    ) -> Value {
        let values = vecmap(fields, |(name, field_typ)| {
            let new_name = format!("{struct_name}.{name}");
            match field_typ {
                noirc_abi::AbiType::Array { length, typ } => {
                    let v_id = self.abi_array(&new_name, None, typ, *length, &witnesses[&new_name]);
                    Value::Node(v_id)
                }
                noirc_abi::AbiType::Struct { fields, .. } => {
                    let new_name = format!("{struct_name}.{name}");
                    self.abi_struct(&new_name, None, fields, witnesses)
                }
                noirc_abi::AbiType::String { length } => {
                    let typ =
                        noirc_abi::AbiType::Integer { sign: noirc_abi::Sign::Unsigned, width: 8 };
                    let v_id =
                        self.abi_array(&new_name, None, &typ, *length, &witnesses[&new_name]);
                    Value::Node(v_id)
                }
                _ => {
                    let obj_type = self.get_object_type_from_abi(field_typ);
                    let v_id = self.create_new_variable(
                        new_name.clone(),
                        None,
                        obj_type,
                        Some(witnesses[&new_name][0]),
                    );
                    Value::Node(v_id)
                }
            }
        });
        self.insert_new_struct(ident_def, values)
    }

    fn ssa_gen_identifier(&mut self, ident: &Ident) -> Result<Value, RuntimeError> {
        // Check if we have already code-gen'd the definition of this variable
        if let Some(value) = self.variable_values.get(&ident.definition) {
            Ok(self.get_current_value(&value.clone()))
        } else {
            // If we haven't, it must be a global value, like a function or builtin
            match &ident.definition {
                Definition::Local(id) => unreachable!(
                    "Local variable encountered before its definition was compiled: {:?}",
                    id
                ),
                Definition::Function(id) => {
                    let id = *id;
                    if !self.context.function_already_compiled(id) {
                        let index = self.context.get_function_index();
                        self.create_function(id, index)?;
                    }

                    let expect_msg = "Expected called function to already be ssa_gen'd";
                    let function_node_id = self.context.get_function_node_id(id).expect(expect_msg);
                    Ok(Value::Node(function_node_id))
                }
                Definition::Builtin(opcode) | Definition::LowLevel(opcode) => {
                    let opcode = builtin::Opcode::lookup(opcode).unwrap_or_else(|| {
                        unreachable!("Unknown builtin/low level opcode '{}'", opcode)
                    });
                    let function_node_id = self.context.get_or_create_opcode_node_id(opcode);
                    Ok(Value::Node(function_node_id))
                }
                Definition::Oracle(_) => unimplemented!("oracles not supported by deprecated SSA"),
            }
        }
    }

    fn ssa_gen_prefix_expression(
        &mut self,
        rhs: NodeId,
        op: UnaryOp,
    ) -> Result<NodeId, RuntimeError> {
        let rhs_type = self.context.object_type(rhs);
        match op {
            UnaryOp::Minus => {
                let lhs = self.context.zero_with_type(rhs_type);
                let operator = BinaryOp::Sub { max_rhs_value: BigUint::zero() };
                let op = Operation::Binary(node::Binary { operator, lhs, rhs, predicate: None });
                self.context.new_instruction(op, rhs_type)
            }
            UnaryOp::Not => self.context.new_instruction(Operation::Not(rhs), rhs_type),
            UnaryOp::MutableReference | UnaryOp::Dereference => {
                unimplemented!("Mutable references are unimplemented in the old ssa backend")
            }
        }
    }

    fn ssa_gen_infix_expression(
        &mut self,
        lhs: NodeId,
        rhs: NodeId,
        op: BinaryOpKind,
        location: Location,
    ) -> Result<NodeId, RuntimeError> {
        let lhs_type = self.context.object_type(lhs);
        // Get the opcode from the infix operator
        let opcode = Operation::Binary(Binary::from_ast(op, lhs_type, lhs, rhs, location));
        let op_type = self.context.get_result_type(&opcode, lhs_type);
        self.context.new_instruction(opcode, op_type)
    }

    fn ssa_gen_indexed_value(
        &mut self,
        array: &LValue,
        index: &Expression,
        location: Location,
    ) -> Result<(NodeId, NodeId, Location), RuntimeError> {
        let value = self.lvalue_to_value(array);
        let lhs = value.unwrap_id();
        let index = self.ssa_gen_expression(index)?.unwrap_id();
        Ok((lhs, index, location))
    }

    fn lvalue_to_value(&self, lvalue: &LValue) -> &Value {
        match lvalue {
            LValue::Ident(ident) => self.find_variable(&ident.definition).unwrap(),
            LValue::Index { array, .. } => {
                self.find_variable(Self::lvalue_ident_def(array.as_ref())).unwrap()
            }
            LValue::MemberAccess { object, field_index, .. } => {
                let ident_def = Self::lvalue_ident_def(object.as_ref());
                let val = self.find_variable(ident_def).unwrap();
                val.get_field_member(*field_index)
            }
            LValue::Dereference { .. } => {
                unreachable!("Mutable references are unsupported in the old ssa backend")
            }
        }
    }

    fn lvalue_ident_def(lvalue: &LValue) -> &Definition {
        match lvalue {
            LValue::Ident(ident) => &ident.definition,
            LValue::Index { array, .. } => Self::lvalue_ident_def(array.as_ref()),
            LValue::MemberAccess { object, .. } => Self::lvalue_ident_def(object.as_ref()),
            LValue::Dereference { reference, .. } => Self::lvalue_ident_def(reference.as_ref()),
        }
    }

    pub(crate) fn create_new_variable(
        &mut self,
        var_name: String,
        def: Option<Definition>,
        obj_type: node::ObjectType,
        witness: Option<acvm::acir::native_types::Witness>,
    ) -> NodeId {
        let new_var = node::Variable {
            id: NodeId::dummy(),
            obj_type,
            name: var_name,
            root: None,
            def: def.clone(),
            witness,
            parent_block: self.context.current_block,
        };
        let v_id = self.context.add_variable(new_var, None);
        let v_value = Value::Node(v_id);
        if let Some(def) = def {
            self.variable_values.insert(def, v_value);
        }
        v_id
    }

    //Helper function for create_new_value()
    fn insert_new_struct(&mut self, def: Option<Definition>, values: Vec<Value>) -> Value {
        let result = Value::Tuple(values);
        if let Some(def_id) = def {
            self.variable_values.insert(def_id, result.clone());
        }
        result
    }

    pub(crate) fn create_new_value(
        &mut self,
        typ: &Type,
        base_name: &str,
        def: Option<Definition>,
    ) -> Value {
        match typ {
            Type::Tuple(fields) => {
                let values = vecmap(fields.iter().enumerate(), |(i, field)| {
                    let name = format!("{base_name}.{i}");
                    self.create_new_value(field, &name, None)
                });
                self.insert_new_struct(def, values)
            }
            Type::Array(len, elem) => {
                //TODO support array of structs
                let obj_type = self.context.convert_type(elem);
                let len = *len;
                let (v_id, _) = self.new_array(base_name, obj_type, len.try_into().unwrap(), def);
                Value::Node(v_id)
            }
            Type::String(len) => {
                // Strings are a packed array of utf-8 encoded bytes
                let obj_type = ObjectType::unsigned_integer(8);
                let len = *len;
                let (v_id, _) = self.new_array(base_name, obj_type, len.try_into().unwrap(), def);
                Value::Node(v_id)
            }
            _ => {
                let obj_type = self.context.convert_type(typ);
                let v_id = self.create_new_variable(base_name.to_string(), def, obj_type, None);
                self.context.get_current_block_mut().update_variable(v_id, v_id);
                Value::Node(v_id)
            }
        }
    }

    pub(crate) fn new_array(
        &mut self,
        name: &str,
        element_type: ObjectType,
        len: u32,
        def: Option<Definition>,
    ) -> (NodeId, ArrayId) {
        let (id, array_id) = self.context.new_array(name, element_type, len, def.clone());
        if let Some(def) = def {
            self.variable_values.insert(def, super::ssa_gen::Value::Node(id));
        }
        (id, array_id)
    }

    // Add a constraint to constrain two expression together
    fn ssa_gen_constrain(
        &mut self,
        expr: &Expression,
        location: noirc_errors::Location,
    ) -> Result<Value, RuntimeError> {
        let cond = self.ssa_gen_expression(expr)?.unwrap_id();
        let operation = Operation::Constrain(cond, Some(location));
        self.context.new_instruction(operation, ObjectType::NotAnObject)?;
        Ok(Value::dummy())
    }

    /// Bind the given Definition to the given Value. This will flatten the Value as needed,
    /// expanding each field of the value to a new variable.
    fn bind_id(&mut self, id: LocalId, value: Value, name: &str) -> Result<(), RuntimeError> {
        let definition = Definition::Local(id);
        match value {
            Value::Node(node_id) => {
                let object_type = self.context.object_type(node_id);
                let value = self.bind_variable(
                    name.to_owned(),
                    Some(definition.clone()),
                    object_type,
                    node_id,
                )?;
                self.variable_values.insert(definition, value);
            }
            value @ Value::Tuple(_) => {
                let value = self.bind_fresh_pattern(name, value)?;
                self.variable_values.insert(definition, value);
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
            Value::Node(node_id) => {
                let object_type = self.context.object_type(node_id);
                self.bind_variable(basename.to_owned(), None, object_type, node_id)
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
        definition_id: Option<Definition>,
        obj_type: node::ObjectType,
        value_id: NodeId,
    ) -> Result<Value, RuntimeError> {
        let id = if let node::ObjectType::ArrayPointer(a) = obj_type {
            let len = self.context.mem[a].len;
            let el_type = self.context.mem[a].element_type;
            self.context.new_array(&variable_name, el_type, len, definition_id).0
        } else {
            let new_var =
                Variable::new(obj_type, variable_name, definition_id, self.context.current_block);
            self.context.add_variable(new_var, None)
        };
        //Assign rhs to lhs
        Ok(Value::Node(self.context.handle_assign(id, None, value_id, None)?))
    }

    //same as update_variable but using the var index instead of var
    pub(crate) fn update_variable_id(
        &mut self,
        var_id: NodeId,
        new_var: NodeId,
        new_value: NodeId,
    ) {
        self.context.update_variable_id(var_id, new_var, new_value);
    }

    fn ssa_gen_assign(
        &mut self,
        lvalue: &LValue,
        expression: &Expression,
    ) -> Result<Value, RuntimeError> {
        let ident_def = Self::lvalue_ident_def(lvalue);
        let rhs = self.ssa_gen_expression(expression)?;

        match lvalue {
            LValue::Ident(_) => {
                let lhs = self.find_variable(ident_def).unwrap();
                // We may be able to avoid cloning here if we change find_variable
                // and assign_pattern to use only fields of self instead of `self` itself.
                let lhs = lhs.clone();
                let result = self.assign_pattern(&lhs, rhs)?;
                self.variable_values.insert(ident_def.clone(), result);
            }
            LValue::Index { array, index, location, .. } => {
                let (lhs_id, array_idx, loc) =
                    self.ssa_gen_indexed_value(array.as_ref(), index, *location)?;
                let rhs_id = rhs.unwrap_id();
                self.context.handle_assign(lhs_id, Some(array_idx), rhs_id, Some(loc))?;
            }
            LValue::MemberAccess { object: _, field_index } => {
                // TODO: This is incorrect for nested structs
                let val = self.find_variable(ident_def).unwrap();
                let value = val.get_field_member(*field_index).clone();
                self.assign_pattern(&value, rhs)?;
            }
            LValue::Dereference { .. } => {
                unreachable!("Mutable references are unsupported in the old ssa backend")
            }
        }
        Ok(Value::dummy())
    }

    /// Similar to bind_pattern but recursively creates Assignment instructions for
    /// each value rather than defining new variables.
    fn assign_pattern(&mut self, lhs: &Value, rhs: Value) -> Result<Value, RuntimeError> {
        match (lhs, rhs) {
            (Value::Node(lhs_id), Value::Node(rhs_id)) => {
                Ok(Value::Node(self.context.handle_assign(*lhs_id, None, rhs_id, None)?))
            }
            (Value::Tuple(lhs_fields), Value::Tuple(rhs_fields)) => {
                assert_eq!(lhs_fields.len(), rhs_fields.len());
                let fields = lhs_fields.iter().zip(rhs_fields).map(|(lhs_field, rhs_field)| {
                    self.assign_pattern(lhs_field, rhs_field)
                }).collect::<Result<Vec<_>, _>>()?;

                Ok(Value::Tuple(fields))
            }
            (Value::Node(_), Value::Tuple(_)) => unreachable!("variables with tuple/struct types should already be decomposed into multiple variables"),
            (Value::Tuple(_), Value::Node(_)) => unreachable!("Uncaught type error, tried to assign a single value to a tuple/struct type"),
        }
    }

    // Let statements are used to declare higher level objects
    fn ssa_gen_let(&mut self, let_expr: &Let) -> Result<Value, RuntimeError> {
        let rhs = self.ssa_gen_expression(&let_expr.expression)?;
        self.bind_id(let_expr.id, rhs, &let_expr.name)?;
        Ok(Value::dummy())
    }

    pub(crate) fn ssa_gen_expression(&mut self, expr: &Expression) -> Result<Value, RuntimeError> {
        match expr {
            Expression::Ident(ident) => self.ssa_gen_identifier(ident),
            Expression::Binary(binary) => {
                // Note: we disallow structs/tuples in infix expressions.
                // The type checker currently disallows this as well but not if they come from a generic type
                // We could allow some in the future, e.g. struct == struct
                let lhs = self.ssa_gen_expression(&binary.lhs)?.to_node_ids();
                let rhs = self.ssa_gen_expression(&binary.rhs)?.to_node_ids();
                if lhs.len() != 1 || rhs.len() != 1 {
                    return Err(RuntimeError {
                        location: Some(binary.location),
                        kind: RuntimeErrorKind::UnsupportedOp {
                            op: binary.operator.to_string(),
                            first_type: "struct/tuple".to_string(),
                            second_type: "struct/tuple".to_string(),
                        },
                    });
                }
                Ok(Value::Node(self.ssa_gen_infix_expression(
                    lhs[0],
                    rhs[0],
                    binary.operator,
                    binary.location,
                )?))
            }
            Expression::Cast(cast_expr) => {
                let lhs = self.ssa_gen_expression(&cast_expr.lhs)?.unwrap_id();
                let object_type = self.context.convert_type(&cast_expr.r#type);

                Ok(Value::Node(self.context.new_instruction(Operation::Cast(lhs), object_type)?))
            }
            Expression::Index(indexed_expr) => {
                // Evaluate the 'array' expression
                let expr_node = self.ssa_gen_expression(&indexed_expr.collection)?.unwrap_id();
                let array = match self.context.object_type(expr_node) {
                    ObjectType::ArrayPointer(array_id) => &self.context.mem[array_id],
                    other => unreachable!("Expected Pointer type, found {:?}", other),
                };
                let array_id = array.id;
                let e_type = array.element_type;
                // Evaluate the index expression
                let index_as_obj = self.ssa_gen_expression(&indexed_expr.index)?.unwrap_id();
                let load = Operation::Load {
                    array_id,
                    index: index_as_obj,
                    location: Some(indexed_expr.location),
                };
                Ok(Value::Node(self.context.new_instruction(load, e_type)?))
            }
            Expression::Call(call_expr) => {
                let results = self.call(call_expr)?;
                Ok(Value::from_slice(&call_expr.return_type, &results))
            }
            Expression::For(for_expr) => self.ssa_gen_for(for_expr),
            Expression::Tuple(fields) => self.ssa_gen_tuple(fields),
            Expression::If(if_expr) => self.handle_if_expr(if_expr),
            Expression::Unary(prefix) => {
                let rhs = self.ssa_gen_expression(&prefix.rhs)?.unwrap_id();
                self.ssa_gen_prefix_expression(rhs, prefix.operator).map(Value::Node)
            }
            Expression::Literal(l) => self.ssa_gen_literal(l),
            Expression::Block(block) => self.ssa_gen_block(block),
            Expression::ExtractTupleField(expr, field) => {
                let tuple = self.ssa_gen_expression(expr.as_ref())?;
                Ok(tuple.into_field_member(*field))
            }
            Expression::Let(let_expr) => self.ssa_gen_let(let_expr),
            Expression::Constrain(expr, location) => {
                self.ssa_gen_constrain(expr.as_ref(), *location)
            }
            Expression::Assign(assign) => {
                self.ssa_gen_assign(&assign.lvalue, assign.expression.as_ref())
            }
            Expression::Semi(expr) => {
                self.ssa_gen_expression(expr.as_ref())?;
                Ok(Value::dummy())
            }
        }
    }

    fn ssa_gen_literal(&mut self, l: &Literal) -> Result<Value, RuntimeError> {
        match l {
            Literal::Integer(x, typ) => {
                let typ = self.context.convert_type(typ);
                Ok(Value::Node(self.context.get_or_create_const(*x, typ)))
            }
            Literal::Array(arr_lit) => {
                let element_type = self.context.convert_type(&arr_lit.element_type);

                let (new_var, array_id) =
                    self.context.new_array("", element_type, arr_lit.contents.len() as u32, None);

                let elements = self.ssa_gen_expression_list(&arr_lit.contents);
                for (pos, object) in elements.into_iter().enumerate() {
                    let lhs_adr = self.context.get_or_create_const(
                        FieldElement::from((pos as u32) as u128),
                        ObjectType::native_field(),
                    );
                    let store = Operation::Store {
                        array_id,
                        index: lhs_adr,
                        value: object,
                        predicate: None,
                        location: None,
                    };
                    self.context.new_instruction(store, element_type)?;
                }
                Ok(Value::Node(new_var))
            }
            Literal::Str(string) => {
                let string_as_integers = vecmap(string.bytes(), |byte| {
                    let f = FieldElement::from_be_bytes_reduce(&[byte]);
                    Expression::Literal(Literal::Integer(
                        f,
                        Type::Integer(noirc_frontend::Signedness::Unsigned, 8),
                    ))
                });

                let string_arr_literal = ArrayLiteral {
                    contents: string_as_integers,
                    element_type: Type::Integer(noirc_frontend::Signedness::Unsigned, 8),
                };

                let new_value = self
                    .ssa_gen_expression(&Expression::Literal(Literal::Array(string_arr_literal)))?;
                Ok(new_value)
            }
            Literal::Bool(b) => {
                if *b {
                    Ok(Value::Node(self.context.one()))
                } else {
                    Ok(Value::Node(self.context.zero()))
                }
            }
        }
    }

    /// A tuple is much the same as a constructor, we just give it fields with numbered names
    fn ssa_gen_tuple(&mut self, fields: &[Expression]) -> Result<Value, RuntimeError> {
        let fields = fields
            .iter()
            .map(|field| self.ssa_gen_expression(field))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Value::Tuple(fields))
    }

    pub(super) fn ssa_gen_expression_list(&mut self, exprs: &[Expression]) -> Vec<NodeId> {
        let mut result = Vec::with_capacity(exprs.len());
        for expr in exprs {
            let value = self.ssa_gen_expression(expr);
            result.extend(value.unwrap().to_node_ids());
        }
        result
    }

    fn ssa_gen_for(&mut self, for_expr: &For) -> Result<Value, RuntimeError> {
        //we add the 'i = start' instruction (in the block before the join)
        let start_idx = self.ssa_gen_expression(&for_expr.start_range).unwrap().unwrap_id();
        let end_idx = self.ssa_gen_expression(&for_expr.end_range).unwrap().unwrap_id();

        //We support only const range for now
        let iter_def = Definition::Local(for_expr.index_variable);
        let iter_type = self.context.convert_type(&for_expr.index_type);
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

        let not_equal = Operation::binary(BinaryOp::Ne, phi, end_idx);
        let cond = self.context.new_instruction(not_equal, ObjectType::boolean())?;

        let to_fix = self.context.new_instruction(Operation::Nop, ObjectType::NotAnObject)?;

        //Body
        let body_id = block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);
        self.context.try_get_mut_instruction(to_fix).unwrap().operation =
            Operation::Jeq(cond, body_id);

        let body_block1 = &mut self.context[body_id];
        body_block1.update_variable(iter_id, phi); //TODO try with just a get_current_value(iter)

        self.ssa_gen_expression(for_expr.block.as_ref())?;

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

        Ok(Value::Node(exit_first))
    }

    //Parse a block of AST statements into ssa form
    fn ssa_gen_block(&mut self, block: &[Expression]) -> Result<Value, RuntimeError> {
        let mut last_value = Value::Node(self.context.zero());
        for expr in block {
            last_value = self.ssa_gen_expression(expr)?;
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

        let condition = self.ssa_gen_expression(if_expr.condition.as_ref())?.unwrap_id();

        if let Some(cond) = node::NodeEval::from_id(&self.context, condition).into_const_value() {
            if cond.is_zero() {
                if let Some(alt) = &if_expr.alternative {
                    return self.ssa_gen_expression(alt);
                } else {
                    return Ok(Value::dummy());
                }
            } else {
                return self.ssa_gen_expression(if_expr.consequence.as_ref());
            }
        }

        let jump_op = Operation::Jeq(condition, block::BlockId::dummy());
        let jump_ins = self.context.new_instruction(jump_op, ObjectType::NotAnObject).unwrap();

        //Then block
        block::new_sealed_block(&mut self.context, block::BlockType::Normal, true);

        let v1 = self.ssa_gen_expression(if_expr.consequence.as_ref())?;

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
            self.context.instruction_mut(jump_ins)
        {
            *target = block2;
        }

        let mut v2 = Value::dummy();
        if let Some(alt) = if_expr.alternative.as_ref() {
            v2 = self.ssa_gen_expression(alt)?;
        }

        //Connect with the exit block
        self.context.get_current_block_mut().left = Some(exit_block);

        //Exit block plumbing
        let block2 = self.context.current_block;
        self.context.current_block = exit_block;
        self.context.get_current_block_mut().predecessor.push(block2);
        ssa_form::seal_block(&mut self.context, exit_block, entry_block);

        // return value:
        let mut counter = 0;
        let mut phi = |a, b| self.context.new_phi(a, b, &mut counter);
        Ok(v1.zip(&v2, &mut phi))
    }
}
