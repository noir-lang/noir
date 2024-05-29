use std::{collections::hash_map::Entry, rc::Rc};

use acvm::{acir::AcirField, FieldElement};
use im::Vector;
use iter_extended::try_vecmap;
use noirc_errors::Location;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ast::{BinaryOpKind, FunctionKind, IntegerBitSize, Signedness};
use crate::{
    hir_def::{
        expr::{
            HirArrayLiteral, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstructorExpression, HirIdent, HirIfExpression, HirIndexExpression,
            HirInfixExpression, HirLambda, HirMemberAccess, HirMethodCallExpression,
            HirPrefixExpression,
        },
        stmt::{
            HirAssignStatement, HirConstrainStatement, HirForStatement, HirLValue, HirLetStatement,
            HirPattern,
        },
    },
    macros_api::{HirExpression, HirLiteral, HirStatement, NodeInterner},
    node_interner::{DefinitionId, DefinitionKind, ExprId, FuncId, StmtId},
    Shared, Type, TypeBinding, TypeBindings, TypeVariableKind,
};

use super::errors::{IResult, InterpreterError};
use super::value::Value;

#[allow(unused)]
pub struct Interpreter<'interner> {
    /// To expand macros the Interpreter may mutate hir nodes within the NodeInterner
    pub(super) interner: &'interner mut NodeInterner,

    /// Each value currently in scope in the interpreter.
    /// Each element of the Vec represents a scope with every scope together making
    /// up all currently visible definitions.
    scopes: Vec<HashMap<DefinitionId, Value>>,

    /// True if we've expanded any macros into any functions and will need
    /// to redo name resolution & type checking for that function.
    changed_functions: HashSet<FuncId>,

    /// True if we've expanded any macros into global scope and will need
    /// to redo name resolution & type checking for everything.
    changed_globally: bool,

    in_loop: bool,

    /// True if we're currently in a compile-time context.
    /// If this is false code is skipped over instead of executed.
    in_comptime_context: bool,
}

#[allow(unused)]
impl<'a> Interpreter<'a> {
    pub(crate) fn new(interner: &'a mut NodeInterner) -> Self {
        Self {
            interner,
            scopes: vec![HashMap::default()],
            changed_functions: HashSet::default(),
            changed_globally: false,
            in_loop: false,
            in_comptime_context: false,
        }
    }

    pub(crate) fn call_function(
        &mut self,
        function: FuncId,
        arguments: Vec<(Value, Location)>,
        location: Location,
    ) -> IResult<Value> {
        let previous_state = self.enter_function();

        let meta = self.interner.function_meta(&function);
        if meta.kind != FunctionKind::Normal {
            let item = "Evaluation for builtin functions";
            return Err(InterpreterError::Unimplemented { item, location });
        }

        if meta.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: meta.parameters.len(),
                actual: arguments.len(),
                location,
            });
        }

        let parameters = meta.parameters.0.clone();
        for ((parameter, typ, _), (argument, arg_location)) in parameters.iter().zip(arguments) {
            self.define_pattern(parameter, typ, argument, arg_location)?;
        }

        let function_body = self.interner.function(&function).as_expr();
        let result = self.evaluate(function_body)?;

        self.exit_function(previous_state);
        Ok(result)
    }

    fn call_closure(
        &mut self,
        closure: HirLambda,
        // TODO: How to define environment here?
        _environment: Vec<Value>,
        arguments: Vec<(Value, Location)>,
        call_location: Location,
    ) -> IResult<Value> {
        let previous_state = self.enter_function();

        if closure.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: closure.parameters.len(),
                actual: arguments.len(),
                location: call_location,
            });
        }

        let parameters = closure.parameters.iter().zip(arguments);
        for ((parameter, typ), (argument, arg_location)) in parameters {
            self.define_pattern(parameter, typ, argument, arg_location)?;
        }

        let result = self.evaluate(closure.body)?;

        self.exit_function(previous_state);
        Ok(result)
    }

    /// Enters a function, pushing a new scope and resetting any required state.
    /// Returns the previous values of the internal state, to be reset when
    /// `exit_function` is called.
    pub(super) fn enter_function(&mut self) -> (bool, Vec<HashMap<DefinitionId, Value>>) {
        // Drain every scope except the global scope
        let mut scope = Vec::new();
        if self.scopes.len() > 1 {
            scope = self.scopes.drain(1..).collect();
            self.push_scope();
        }
        (std::mem::take(&mut self.in_loop), scope)
    }

    pub(super) fn exit_function(&mut self, mut state: (bool, Vec<HashMap<DefinitionId, Value>>)) {
        self.in_loop = state.0;

        // Keep only the global scope
        self.scopes.truncate(1);
        self.scopes.append(&mut state.1);
    }

    pub(super) fn push_scope(&mut self) {
        self.scopes.push(HashMap::default());
    }

    pub(super) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<DefinitionId, Value> {
        // the global scope is always at index zero, so this is always Some
        self.scopes.last_mut().unwrap()
    }

    pub(super) fn define_pattern(
        &mut self,
        pattern: &HirPattern,
        typ: &Type,
        argument: Value,
        location: Location,
    ) -> IResult<()> {
        match pattern {
            HirPattern::Identifier(identifier) => {
                self.define(identifier.id, typ, argument, location)
            }
            HirPattern::Mutable(pattern, _) => {
                self.define_pattern(pattern, typ, argument, location)
            }
            HirPattern::Tuple(pattern_fields, _) => match (argument, typ) {
                (Value::Tuple(fields), Type::Tuple(type_fields))
                    if fields.len() == pattern_fields.len() =>
                {
                    for ((pattern, typ), argument) in
                        pattern_fields.iter().zip(type_fields).zip(fields)
                    {
                        self.define_pattern(pattern, typ, argument, location)?;
                    }
                    Ok(())
                }
                (value, _) => {
                    Err(InterpreterError::TypeMismatch { expected: typ.clone(), value, location })
                }
            },
            HirPattern::Struct(struct_type, pattern_fields, _) => {
                self.type_check(typ, &argument, location)?;
                self.type_check(struct_type, &argument, location)?;

                match argument {
                    Value::Struct(fields, struct_type) if fields.len() == pattern_fields.len() => {
                        for (field_name, field_pattern) in pattern_fields {
                            let field = fields.get(&field_name.0.contents).ok_or_else(|| {
                                InterpreterError::ExpectedStructToHaveField {
                                    value: Value::Struct(fields.clone(), struct_type.clone()),
                                    field_name: field_name.0.contents.clone(),
                                    location,
                                }
                            })?;

                            let field_type = field.get_type().into_owned();
                            self.define_pattern(
                                field_pattern,
                                &field_type,
                                field.clone(),
                                location,
                            )?;
                        }
                        Ok(())
                    }
                    value => Err(InterpreterError::TypeMismatch {
                        expected: typ.clone(),
                        value,
                        location,
                    }),
                }
            }
        }
    }

    /// Define a new variable in the current scope
    fn define(
        &mut self,
        id: DefinitionId,
        typ: &Type,
        argument: Value,
        location: Location,
    ) -> IResult<()> {
        self.type_check(typ, &argument, location)?;
        self.current_scope_mut().insert(id, argument);
        Ok(())
    }

    /// Mutate an existing variable, potentially from a prior scope.
    /// Also type checks the value being assigned
    fn checked_mutate(
        &mut self,
        id: DefinitionId,
        typ: &Type,
        argument: Value,
        location: Location,
    ) -> IResult<()> {
        self.type_check(typ, &argument, location)?;
        self.mutate(id, argument, location)
    }

    /// Mutate an existing variable, potentially from a prior scope
    fn mutate(&mut self, id: DefinitionId, argument: Value, location: Location) -> IResult<()> {
        for scope in self.scopes.iter_mut().rev() {
            if let Entry::Occupied(mut entry) = scope.entry(id) {
                entry.insert(argument);
                return Ok(());
            }
        }
        let name = self.interner.definition(id).name.clone();
        Err(InterpreterError::NonComptimeVarReferenced { name, location })
    }

    pub(super) fn lookup(&self, ident: &HirIdent) -> IResult<Value> {
        self.lookup_id(ident.id, ident.location)
    }

    fn lookup_id(&self, id: DefinitionId, location: Location) -> IResult<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(&id) {
                return Ok(value.clone());
            }
        }

        // Justification for `NonComptimeVarReferenced`:
        // If we have an id to lookup at all that means name resolution successfully
        // found another variable in scope for this name. If the name is in scope
        // but unknown by the interpreter it must be because it was not a comptime variable.
        let name = self.interner.definition(id).name.clone();
        Err(InterpreterError::NonComptimeVarReferenced { name, location })
    }

    fn type_check(&self, typ: &Type, value: &Value, location: Location) -> IResult<()> {
        let typ = typ.follow_bindings();
        let value_type = value.get_type();

        typ.try_unify(&value_type, &mut TypeBindings::new()).map_err(|_| {
            InterpreterError::TypeMismatch { expected: typ, value: value.clone(), location }
        })
    }

    /// Evaluate an expression and return the result
    pub(super) fn evaluate(&mut self, id: ExprId) -> IResult<Value> {
        match self.interner.expression(&id) {
            HirExpression::Ident(ident, _) => self.evaluate_ident(ident, id),
            HirExpression::Literal(literal) => self.evaluate_literal(literal, id),
            HirExpression::Block(block) => self.evaluate_block(block),
            HirExpression::Prefix(prefix) => self.evaluate_prefix(prefix, id),
            HirExpression::Infix(infix) => self.evaluate_infix(infix, id),
            HirExpression::Index(index) => self.evaluate_index(index, id),
            HirExpression::Constructor(constructor) => self.evaluate_constructor(constructor, id),
            HirExpression::MemberAccess(access) => self.evaluate_access(access, id),
            HirExpression::Call(call) => self.evaluate_call(call, id),
            HirExpression::MethodCall(call) => self.evaluate_method_call(call, id),
            HirExpression::Cast(cast) => self.evaluate_cast(cast, id),
            HirExpression::If(if_) => self.evaluate_if(if_, id),
            HirExpression::Tuple(tuple) => self.evaluate_tuple(tuple),
            HirExpression::Lambda(lambda) => self.evaluate_lambda(lambda, id),
            HirExpression::Quote(block) => Ok(Value::Code(Rc::new(block))),
            HirExpression::Comptime(block) => self.evaluate_block(block),
            HirExpression::Unquote(block) => {
                // An Unquote expression being found is indicative of a macro being
                // expanded within another comptime fn which we don't currently support.
                let location = self.interner.expr_location(&id);
                Err(InterpreterError::UnquoteFoundDuringEvaluation { location })
            }
            HirExpression::Error => {
                let location = self.interner.expr_location(&id);
                Err(InterpreterError::ErrorNodeEncountered { location })
            }
        }
    }

    pub(super) fn evaluate_ident(&mut self, ident: HirIdent, id: ExprId) -> IResult<Value> {
        let definition = self.interner.definition(ident.id);

        match &definition.kind {
            DefinitionKind::Function(function_id) => {
                let typ = self.interner.id_type(id);
                Ok(Value::Function(*function_id, typ))
            }
            DefinitionKind::Local(_) => self.lookup(&ident),
            DefinitionKind::Global(global_id) => {
                // Don't need to check let_.comptime, we can evaluate non-comptime globals too.
                // Avoid resetting the value if it is already known
                if let Ok(value) = self.lookup(&ident) {
                    Ok(value)
                } else {
                    let let_ = self.interner.get_global_let_statement(*global_id).unwrap();
                    self.evaluate_let(let_)?;
                    self.lookup(&ident)
                }
            }
            DefinitionKind::GenericType(type_variable) => {
                let value = match &*type_variable.borrow() {
                    TypeBinding::Unbound(_) => None,
                    TypeBinding::Bound(binding) => binding.evaluate_to_u64(),
                };

                if let Some(value) = value {
                    let typ = self.interner.id_type(id);
                    self.evaluate_integer((value as u128).into(), false, id)
                } else {
                    let location = self.interner.expr_location(&id);
                    let typ = Type::TypeVariable(type_variable.clone(), TypeVariableKind::Normal);
                    Err(InterpreterError::NonIntegerArrayLength { typ, location })
                }
            }
        }
    }

    fn evaluate_literal(&mut self, literal: HirLiteral, id: ExprId) -> IResult<Value> {
        match literal {
            HirLiteral::Unit => Ok(Value::Unit),
            HirLiteral::Bool(value) => Ok(Value::Bool(value)),
            HirLiteral::Integer(value, is_negative) => {
                self.evaluate_integer(value, is_negative, id)
            }
            HirLiteral::Str(string) => Ok(Value::String(Rc::new(string))),
            HirLiteral::FmtStr(_, _) => todo!("Evaluate format strings"),
            HirLiteral::Array(array) => self.evaluate_array(array, id),
            HirLiteral::Slice(array) => self.evaluate_slice(array, id),
        }
    }

    fn evaluate_integer(
        &self,
        value: FieldElement,
        is_negative: bool,
        id: ExprId,
    ) -> IResult<Value> {
        let typ = self.interner.id_type(id).follow_bindings();
        let location = self.interner.expr_location(&id);

        if let Type::FieldElement = &typ {
            Ok(Value::Field(value))
        } else if let Type::Integer(sign, bit_size) = &typ {
            match (sign, bit_size) {
                (Signedness::Unsigned, IntegerBitSize::One) => {
                    return Err(InterpreterError::TypeUnsupported { typ, location });
                }
                (Signedness::Unsigned, IntegerBitSize::Eight) => {
                    let value: u8 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { 0u8.wrapping_sub(value) } else { value };
                    Ok(Value::U8(value))
                }
                (Signedness::Unsigned, IntegerBitSize::Sixteen) => {
                    let value: u16 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { 0u16.wrapping_sub(value) } else { value };
                    Ok(Value::U16(value))
                }
                (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                    let value: u32 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { 0u32.wrapping_sub(value) } else { value };
                    Ok(Value::U32(value))
                }
                (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                    let value: u64 =
                        value.try_to_u64().ok_or(InterpreterError::IntegerOutOfRangeForType {
                            value,
                            typ,
                            location,
                        })?;
                    let value = if is_negative { 0u64.wrapping_sub(value) } else { value };
                    Ok(Value::U64(value))
                }
                (Signedness::Signed, IntegerBitSize::One) => {
                    return Err(InterpreterError::TypeUnsupported { typ, location });
                }
                (Signedness::Signed, IntegerBitSize::Eight) => {
                    let value: i8 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I8(value))
                }
                (Signedness::Signed, IntegerBitSize::Sixteen) => {
                    let value: i16 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I16(value))
                }
                (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                    let value: i32 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I32(value))
                }
                (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                    let value: i64 =
                        value.try_to_u64().and_then(|value| value.try_into().ok()).ok_or(
                            InterpreterError::IntegerOutOfRangeForType { value, typ, location },
                        )?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I64(value))
                }
            }
        } else {
            Err(InterpreterError::NonIntegerIntegerLiteral { typ, location })
        }
    }

    pub(super) fn evaluate_block(&mut self, mut block: HirBlockExpression) -> IResult<Value> {
        let last_statement = block.statements.pop();
        self.push_scope();

        for statement in block.statements {
            self.evaluate_statement(statement)?;
        }

        let result = if let Some(statement) = last_statement {
            self.evaluate_statement(statement)
        } else {
            Ok(Value::Unit)
        };

        self.pop_scope();
        result
    }

    fn evaluate_array(&mut self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
        let typ = self.interner.id_type(id);

        match array {
            HirArrayLiteral::Standard(elements) => {
                let elements = elements
                    .into_iter()
                    .map(|id| self.evaluate(id))
                    .collect::<IResult<Vector<_>>>()?;

                Ok(Value::Array(elements, typ))
            }
            HirArrayLiteral::Repeated { repeated_element, length } => {
                let element = self.evaluate(repeated_element)?;

                if let Some(length) = length.evaluate_to_u64() {
                    let elements = (0..length).map(|_| element.clone()).collect();
                    Ok(Value::Array(elements, typ))
                } else {
                    let location = self.interner.expr_location(&id);
                    Err(InterpreterError::NonIntegerArrayLength { typ: length, location })
                }
            }
        }
    }

    fn evaluate_slice(&mut self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
        self.evaluate_array(array, id).map(|value| match value {
            Value::Array(array, typ) => Value::Slice(array, typ),
            other => unreachable!("Non-array value returned from evaluate array: {other:?}"),
        })
    }

    fn evaluate_prefix(&mut self, prefix: HirPrefixExpression, id: ExprId) -> IResult<Value> {
        let rhs = self.evaluate(prefix.rhs)?;
        match prefix.operator {
            crate::ast::UnaryOp::Minus => match rhs {
                Value::Field(value) => Ok(Value::Field(FieldElement::zero() - value)),
                Value::I8(value) => Ok(Value::I8(-value)),
                Value::I16(value) => Ok(Value::I16(-value)),
                Value::I32(value) => Ok(Value::I32(-value)),
                Value::I64(value) => Ok(Value::I64(-value)),
                Value::U8(value) => Ok(Value::U8(0 - value)),
                Value::U16(value) => Ok(Value::U16(0 - value)),
                Value::U32(value) => Ok(Value::U32(0 - value)),
                Value::U64(value) => Ok(Value::U64(0 - value)),
                value => {
                    let location = self.interner.expr_location(&id);
                    let operator = "minus";
                    Err(InterpreterError::InvalidValueForUnary { value, location, operator })
                }
            },
            crate::ast::UnaryOp::Not => match rhs {
                Value::Bool(value) => Ok(Value::Bool(!value)),
                Value::I8(value) => Ok(Value::I8(!value)),
                Value::I16(value) => Ok(Value::I16(!value)),
                Value::I32(value) => Ok(Value::I32(!value)),
                Value::I64(value) => Ok(Value::I64(!value)),
                Value::U8(value) => Ok(Value::U8(!value)),
                Value::U16(value) => Ok(Value::U16(!value)),
                Value::U32(value) => Ok(Value::U32(!value)),
                Value::U64(value) => Ok(Value::U64(!value)),
                value => {
                    let location = self.interner.expr_location(&id);
                    Err(InterpreterError::InvalidValueForUnary { value, location, operator: "not" })
                }
            },
            crate::ast::UnaryOp::MutableReference => Ok(Value::Pointer(Shared::new(rhs))),
            crate::ast::UnaryOp::Dereference { implicitly_added: _ } => match rhs {
                Value::Pointer(element) => Ok(element.borrow().clone()),
                value => {
                    let location = self.interner.expr_location(&id);
                    Err(InterpreterError::NonPointerDereferenced { value, location })
                }
            },
        }
    }

    fn evaluate_infix(&mut self, infix: HirInfixExpression, id: ExprId) -> IResult<Value> {
        let lhs = self.evaluate(infix.lhs)?;
        let rhs = self.evaluate(infix.rhs)?;

        // TODO: Need to account for operator overloading
        assert!(
            self.interner.get_selected_impl_for_expression(id).is_none(),
            "Operator overloading is unimplemented in the interpreter"
        );

        use InterpreterError::InvalidValuesForBinary;
        match infix.operator.kind {
            BinaryOpKind::Add => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs + rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs + rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs + rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs + rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs + rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs + rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs + rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs + rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs + rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "+" })
                }
            },
            BinaryOpKind::Subtract => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs - rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs - rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs - rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs - rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs - rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs - rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs - rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs - rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs - rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "-" })
                }
            },
            BinaryOpKind::Multiply => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs * rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs * rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs * rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs * rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs * rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs * rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs * rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs * rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs * rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "*" })
                }
            },
            BinaryOpKind::Divide => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs / rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs / rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs / rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs / rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs / rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs / rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs / rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs / rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs / rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "/" })
                }
            },
            BinaryOpKind::Equal => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "==" })
                }
            },
            BinaryOpKind::NotEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "!=" })
                }
            },
            BinaryOpKind::Less => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "<" })
                }
            },
            BinaryOpKind::LessEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "<=" })
                }
            },
            BinaryOpKind::Greater => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: ">" })
                }
            },
            BinaryOpKind::GreaterEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: ">=" })
                }
            },
            BinaryOpKind::And => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs & rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs & rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs & rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs & rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs & rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs & rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs & rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs & rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs & rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "&" })
                }
            },
            BinaryOpKind::Or => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs | rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs | rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs | rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs | rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs | rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs | rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs | rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs | rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs | rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "|" })
                }
            },
            BinaryOpKind::Xor => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs ^ rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs ^ rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs ^ rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs ^ rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs ^ rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs ^ rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs ^ rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs ^ rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs ^ rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "^" })
                }
            },
            BinaryOpKind::ShiftRight => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs >> rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs >> rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs >> rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs >> rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs >> rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs >> rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs >> rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs >> rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: ">>" })
                }
            },
            BinaryOpKind::ShiftLeft => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs << rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs << rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs << rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs << rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs << rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs << rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs << rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs << rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "<<" })
                }
            },
            BinaryOpKind::Modulo => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs % rhs)),
                (Value::I16(lhs), Value::I16(rhs)) => Ok(Value::I16(lhs % rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs % rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs % rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs % rhs)),
                (Value::U16(lhs), Value::U16(rhs)) => Ok(Value::U16(lhs % rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs % rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs % rhs)),
                (lhs, rhs) => {
                    let location = self.interner.expr_location(&id);
                    Err(InvalidValuesForBinary { lhs, rhs, location, operator: "%" })
                }
            },
        }
    }

    fn evaluate_index(&mut self, index: HirIndexExpression, id: ExprId) -> IResult<Value> {
        let array = self.evaluate(index.collection)?;
        let index = self.evaluate(index.index)?;

        let location = self.interner.expr_location(&id);
        let (array, index) = self.bounds_check(array, index, location)?;

        Ok(array[index].clone())
    }

    /// Bounds check the given array and index pair.
    /// This will also ensure the given arguments are in fact an array and integer.
    fn bounds_check(
        &self,
        array: Value,
        index: Value,
        location: Location,
    ) -> IResult<(Vector<Value>, usize)> {
        let collection = match array {
            Value::Array(array, _) => array,
            Value::Slice(array, _) => array,
            value => {
                return Err(InterpreterError::NonArrayIndexed { value, location });
            }
        };

        let index = match index {
            Value::Field(value) => {
                value.try_to_u64().expect("index could not fit into u64") as usize
            }
            Value::I8(value) => value as usize,
            Value::I16(value) => value as usize,
            Value::I32(value) => value as usize,
            Value::I64(value) => value as usize,
            Value::U8(value) => value as usize,
            Value::U16(value) => value as usize,
            Value::U32(value) => value as usize,
            Value::U64(value) => value as usize,
            value => {
                return Err(InterpreterError::NonIntegerUsedAsIndex { value, location });
            }
        };

        if index >= collection.len() {
            use InterpreterError::IndexOutOfBounds;
            return Err(IndexOutOfBounds { index, location, length: collection.len() });
        }

        Ok((collection, index))
    }

    fn evaluate_constructor(
        &mut self,
        constructor: HirConstructorExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let fields = constructor
            .fields
            .into_iter()
            .map(|(name, expr)| {
                let field_value = self.evaluate(expr)?;
                Ok((Rc::new(name.0.contents), field_value))
            })
            .collect::<Result<_, _>>()?;

        let typ = self.interner.id_type(id);
        Ok(Value::Struct(fields, typ))
    }

    fn evaluate_access(&mut self, access: HirMemberAccess, id: ExprId) -> IResult<Value> {
        let (fields, struct_type) = match self.evaluate(access.lhs)? {
            Value::Struct(fields, typ) => (fields, typ),
            value => {
                let location = self.interner.expr_location(&id);
                return Err(InterpreterError::NonTupleOrStructInMemberAccess { value, location });
            }
        };

        fields.get(&access.rhs.0.contents).cloned().ok_or_else(|| {
            let location = self.interner.expr_location(&id);
            let value = Value::Struct(fields, struct_type);
            let field_name = access.rhs.0.contents;
            InterpreterError::ExpectedStructToHaveField { value, field_name, location }
        })
    }

    fn evaluate_call(&mut self, call: HirCallExpression, id: ExprId) -> IResult<Value> {
        let function = self.evaluate(call.func)?;
        let arguments = try_vecmap(call.arguments, |arg| {
            Ok((self.evaluate(arg)?, self.interner.expr_location(&arg)))
        })?;
        let location = self.interner.expr_location(&id);

        match function {
            Value::Function(function_id, _) => self.call_function(function_id, arguments, location),
            Value::Closure(closure, env, _) => self.call_closure(closure, env, arguments, location),
            value => Err(InterpreterError::NonFunctionCalled { value, location }),
        }
    }

    fn evaluate_method_call(
        &mut self,
        call: HirMethodCallExpression,
        id: ExprId,
    ) -> IResult<Value> {
        let object = self.evaluate(call.object)?;
        let arguments = try_vecmap(call.arguments, |arg| {
            Ok((self.evaluate(arg)?, self.interner.expr_location(&arg)))
        })?;
        let location = self.interner.expr_location(&id);

        let typ = object.get_type().follow_bindings();
        let method_name = &call.method.0.contents;

        // TODO: Traits
        let method = match &typ {
            Type::Struct(struct_def, _) => {
                self.interner.lookup_method(&typ, struct_def.borrow().id, method_name, false)
            }
            _ => self.interner.lookup_primitive_method(&typ, method_name),
        };

        if let Some(method) = method {
            self.call_function(method, arguments, location)
        } else {
            Err(InterpreterError::NoMethodFound { name: method_name.clone(), typ, location })
        }
    }

    fn evaluate_cast(&mut self, cast: HirCastExpression, id: ExprId) -> IResult<Value> {
        macro_rules! signed_int_to_field {
            ($x:expr) => {{
                // Need to convert the signed integer to an i128 before
                // we negate it to preserve the MIN value.
                let mut value = $x as i128;
                let is_negative = value < 0;
                if is_negative {
                    value = -value;
                }
                ((value as u128).into(), is_negative)
            }};
        }

        let (mut lhs, lhs_is_negative) = match self.evaluate(cast.lhs)? {
            Value::Field(value) => (value, false),
            Value::U8(value) => ((value as u128).into(), false),
            Value::U16(value) => ((value as u128).into(), false),
            Value::U32(value) => ((value as u128).into(), false),
            Value::U64(value) => ((value as u128).into(), false),
            Value::I8(value) => signed_int_to_field!(value),
            Value::I16(value) => signed_int_to_field!(value),
            Value::I32(value) => signed_int_to_field!(value),
            Value::I64(value) => signed_int_to_field!(value),
            Value::Bool(value) => {
                (if value { FieldElement::one() } else { FieldElement::zero() }, false)
            }
            value => {
                let location = self.interner.expr_location(&id);
                return Err(InterpreterError::NonNumericCasted { value, location });
            }
        };

        macro_rules! cast_to_int {
            ($x:expr, $method:ident, $typ:ty, $f:ident) => {{
                let mut value = $x.$method() as $typ;
                if lhs_is_negative {
                    value = 0 - value;
                }
                Ok(Value::$f(value))
            }};
        }

        // Now actually cast the lhs, bit casting and wrapping as necessary
        match cast.r#type.follow_bindings() {
            Type::FieldElement => {
                if lhs_is_negative {
                    lhs = FieldElement::zero() - lhs;
                }
                Ok(Value::Field(lhs))
            }
            Type::Integer(sign, bit_size) => match (sign, bit_size) {
                (Signedness::Unsigned, IntegerBitSize::One) => {
                    let location = self.interner.expr_location(&id);
                    Err(InterpreterError::TypeUnsupported { typ: cast.r#type, location })
                }
                (Signedness::Unsigned, IntegerBitSize::Eight) => cast_to_int!(lhs, to_u128, u8, U8),
                (Signedness::Unsigned, IntegerBitSize::Sixteen) => {
                    cast_to_int!(lhs, to_u128, u16, U16)
                }
                (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                    cast_to_int!(lhs, to_u128, u32, U32)
                }
                (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                    cast_to_int!(lhs, to_u128, u64, U64)
                }
                (Signedness::Signed, IntegerBitSize::One) => {
                    let location = self.interner.expr_location(&id);
                    Err(InterpreterError::TypeUnsupported { typ: cast.r#type, location })
                }
                (Signedness::Signed, IntegerBitSize::Eight) => cast_to_int!(lhs, to_i128, i8, I8),
                (Signedness::Signed, IntegerBitSize::Sixteen) => {
                    cast_to_int!(lhs, to_i128, i16, I16)
                }
                (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                    cast_to_int!(lhs, to_i128, i32, I32)
                }
                (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                    cast_to_int!(lhs, to_i128, i64, I64)
                }
            },
            Type::Bool => Ok(Value::Bool(!lhs.is_zero() || lhs_is_negative)),
            typ => {
                let location = self.interner.expr_location(&id);
                Err(InterpreterError::CastToNonNumericType { typ, location })
            }
        }
    }

    fn evaluate_if(&mut self, if_: HirIfExpression, id: ExprId) -> IResult<Value> {
        let condition = match self.evaluate(if_.condition)? {
            Value::Bool(value) => value,
            value => {
                let location = self.interner.expr_location(&id);
                return Err(InterpreterError::NonBoolUsedInIf { value, location });
            }
        };

        self.push_scope();

        let result = if condition {
            if if_.alternative.is_some() {
                self.evaluate(if_.consequence)
            } else {
                self.evaluate(if_.consequence)?;
                Ok(Value::Unit)
            }
        } else {
            match if_.alternative {
                Some(alternative) => self.evaluate(alternative),
                None => Ok(Value::Unit),
            }
        };

        self.pop_scope();
        result
    }

    fn evaluate_tuple(&mut self, tuple: Vec<ExprId>) -> IResult<Value> {
        let fields = try_vecmap(tuple, |field| self.evaluate(field))?;
        Ok(Value::Tuple(fields))
    }

    fn evaluate_lambda(&mut self, lambda: HirLambda, id: ExprId) -> IResult<Value> {
        let location = self.interner.expr_location(&id);
        let environment =
            try_vecmap(&lambda.captures, |capture| self.lookup_id(capture.ident.id, location))?;

        let typ = self.interner.id_type(id);
        Ok(Value::Closure(lambda, environment, typ))
    }

    fn evaluate_statement(&mut self, statement: StmtId) -> IResult<Value> {
        match self.interner.statement(&statement) {
            HirStatement::Let(let_) => self.evaluate_let(let_),
            HirStatement::Constrain(constrain) => self.evaluate_constrain(constrain),
            HirStatement::Assign(assign) => self.evaluate_assign(assign),
            HirStatement::For(for_) => self.evaluate_for(for_),
            HirStatement::Break => self.evaluate_break(statement),
            HirStatement::Continue => self.evaluate_continue(statement),
            HirStatement::Expression(expression) => self.evaluate(expression),
            HirStatement::Comptime(statement) => self.evaluate_comptime(statement),
            HirStatement::Semi(expression) => {
                self.evaluate(expression)?;
                Ok(Value::Unit)
            }
            HirStatement::Error => {
                let location = self.interner.id_location(statement);
                Err(InterpreterError::ErrorNodeEncountered { location })
            }
        }
    }

    pub(super) fn evaluate_let(&mut self, let_: HirLetStatement) -> IResult<Value> {
        let rhs = self.evaluate(let_.expression)?;
        let location = self.interner.expr_location(&let_.expression);
        self.define_pattern(&let_.pattern, &let_.r#type, rhs, location)?;
        Ok(Value::Unit)
    }

    fn evaluate_constrain(&mut self, constrain: HirConstrainStatement) -> IResult<Value> {
        match self.evaluate(constrain.0)? {
            Value::Bool(true) => Ok(Value::Unit),
            Value::Bool(false) => {
                let location = self.interner.expr_location(&constrain.0);
                let message = constrain.2.and_then(|expr| self.evaluate(expr).ok());
                Err(InterpreterError::FailingConstraint { location, message })
            }
            value => {
                let location = self.interner.expr_location(&constrain.0);
                Err(InterpreterError::NonBoolUsedInConstrain { value, location })
            }
        }
    }

    fn evaluate_assign(&mut self, assign: HirAssignStatement) -> IResult<Value> {
        let rhs = self.evaluate(assign.expression)?;
        self.store_lvalue(assign.lvalue, rhs)?;
        Ok(Value::Unit)
    }

    fn store_lvalue(&mut self, lvalue: HirLValue, rhs: Value) -> IResult<()> {
        match lvalue {
            HirLValue::Ident(ident, typ) => {
                self.checked_mutate(ident.id, &typ, rhs, ident.location)
            }
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                match self.evaluate_lvalue(&lvalue)? {
                    Value::Pointer(value) => {
                        *value.borrow_mut() = rhs;
                        Ok(())
                    }
                    value => Err(InterpreterError::NonPointerDereferenced { value, location }),
                }
            }
            HirLValue::MemberAccess { object, field_name, field_index, typ: _, location } => {
                let index = field_index.expect("The field index should be set after type checking");
                match self.evaluate_lvalue(&object)? {
                    Value::Tuple(mut fields) => {
                        fields[index] = rhs;
                        self.store_lvalue(*object, Value::Tuple(fields))
                    }
                    Value::Struct(mut fields, typ) => {
                        fields.insert(Rc::new(field_name.0.contents), rhs);
                        self.store_lvalue(*object, Value::Struct(fields, typ))
                    }
                    value => {
                        Err(InterpreterError::NonTupleOrStructInMemberAccess { value, location })
                    }
                }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array_value = self.evaluate_lvalue(&array)?;
                let index = self.evaluate(index)?;

                let constructor = match &array_value {
                    Value::Array(..) => Value::Array,
                    _ => Value::Slice,
                };

                let typ = array_value.get_type().into_owned();
                let (elements, index) = self.bounds_check(array_value, index, location)?;

                let new_array = constructor(elements.update(index, rhs), typ);
                self.store_lvalue(*array, new_array)
            }
        }
    }

    fn evaluate_lvalue(&mut self, lvalue: &HirLValue) -> IResult<Value> {
        match lvalue {
            HirLValue::Ident(ident, _) => self.lookup(ident),
            HirLValue::Dereference { lvalue, element_type: _, location } => {
                match self.evaluate_lvalue(lvalue)? {
                    Value::Pointer(value) => Ok(value.borrow().clone()),
                    value => {
                        Err(InterpreterError::NonPointerDereferenced { value, location: *location })
                    }
                }
            }
            HirLValue::MemberAccess { object, field_name, field_index, typ: _, location } => {
                let index = field_index.expect("The field index should be set after type checking");

                match self.evaluate_lvalue(object)? {
                    Value::Tuple(mut values) => Ok(values.swap_remove(index)),
                    Value::Struct(fields, _) => Ok(fields[&field_name.0.contents].clone()),
                    value => Err(InterpreterError::NonTupleOrStructInMemberAccess {
                        value,
                        location: *location,
                    }),
                }
            }
            HirLValue::Index { array, index, typ: _, location } => {
                let array = self.evaluate_lvalue(array)?;
                let index = self.evaluate(*index)?;
                let (elements, index) = self.bounds_check(array, index, *location)?;
                Ok(elements[index].clone())
            }
        }
    }

    fn evaluate_for(&mut self, for_: HirForStatement) -> IResult<Value> {
        // i128 can store all values from i8 - u64
        let get_index = |this: &mut Self, expr| -> IResult<(_, fn(_) -> _)> {
            match this.evaluate(expr)? {
                Value::I8(value) => Ok((value as i128, |i| Value::I8(i as i8))),
                Value::I16(value) => Ok((value as i128, |i| Value::I16(i as i16))),
                Value::I32(value) => Ok((value as i128, |i| Value::I32(i as i32))),
                Value::I64(value) => Ok((value as i128, |i| Value::I64(i as i64))),
                Value::U8(value) => Ok((value as i128, |i| Value::U8(i as u8))),
                Value::U16(value) => Ok((value as i128, |i| Value::U16(i as u16))),
                Value::U32(value) => Ok((value as i128, |i| Value::U32(i as u32))),
                Value::U64(value) => Ok((value as i128, |i| Value::U64(i as u64))),
                value => {
                    let location = this.interner.expr_location(&expr);
                    Err(InterpreterError::NonIntegerUsedInLoop { value, location })
                }
            }
        };

        let (start, make_value) = get_index(self, for_.start_range)?;
        let (end, _) = get_index(self, for_.end_range)?;
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);

        for i in start..end {
            self.push_scope();
            self.current_scope_mut().insert(for_.identifier.id, make_value(i));

            match self.evaluate(for_.block) {
                Ok(_) => (),
                Err(InterpreterError::Break) => break,
                Err(InterpreterError::Continue) => continue,
                Err(other) => return Err(other),
            }
            self.pop_scope();
        }

        self.in_loop = was_in_loop;
        Ok(Value::Unit)
    }

    fn evaluate_break(&mut self, id: StmtId) -> IResult<Value> {
        if self.in_loop {
            Err(InterpreterError::Break)
        } else {
            let location = self.interner.statement_location(id);
            Err(InterpreterError::BreakNotInLoop { location })
        }
    }

    fn evaluate_continue(&mut self, id: StmtId) -> IResult<Value> {
        if self.in_loop {
            Err(InterpreterError::Continue)
        } else {
            let location = self.interner.statement_location(id);
            Err(InterpreterError::ContinueNotInLoop { location })
        }
    }

    pub(super) fn evaluate_comptime(&mut self, statement: StmtId) -> IResult<Value> {
        let was_in_comptime = std::mem::replace(&mut self.in_comptime_context, true);
        let result = self.evaluate_statement(statement);
        self.in_comptime_context = was_in_comptime;
        result
    }
}
