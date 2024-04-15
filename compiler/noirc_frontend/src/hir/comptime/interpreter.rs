use std::{borrow::Cow, rc::Rc};

use acvm::FieldElement;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    hir_def::{
        expr::{
            HirArrayLiteral, HirBlockExpression, HirCallExpression, HirCastExpression,
            HirConstructorExpression, HirIfExpression, HirIndexExpression, HirInfixExpression,
            HirLambda, HirMemberAccess, HirMethodCallExpression, HirPrefixExpression,
        },
        stmt::HirPattern,
    },
    macros_api::{HirExpression, HirLiteral, HirStatement, NodeInterner},
    node_interner::{DefinitionId, ExprId, FuncId, StmtId},
    BinaryOpKind, BlockExpression, FunctionKind, IntegerBitSize, Shared, Signedness, Type,
};

struct Interpreter<'interner> {
    /// To expand macros the Interpreter may mutate hir nodes within the NodeInterner
    interner: &'interner mut NodeInterner,

    /// Each value currently in scope in the interpreter.
    /// Each element of the Vec represents a scope with every scope together making
    /// up all currently visible definitions.
    scopes: Vec<FxHashMap<DefinitionId, Value>>,

    /// True if we've expanded any macros into any functions and will need
    /// to redo name resolution & type checking for that function.
    changed_functions: FxHashSet<FuncId>,

    /// True if we've expanded any macros into global scope and will need
    /// to redo name resolution & type checking for everything.
    changed_globally: bool,
}

#[derive(Debug, Clone)]
enum Value {
    Unit,
    Bool(bool),
    Field(FieldElement),
    I8(i8),
    I32(i32),
    I64(i64),
    U8(u8),
    U32(u32),
    U64(u64),
    String(Rc<String>),
    Function(FuncId, Type),
    Closure(HirLambda, Vec<Value>, Type),
    Tuple(Vec<Value>),
    Struct(FxHashMap<Rc<String>, Value>, Type),
    Pointer(Shared<Value>),
    Array(Vector<Value>, Type),
    Slice(Vector<Value>, Type),
    Code(Rc<BlockExpression>),
}

/// The possible errors that can halt the interpreter.
///
/// TODO: All error variants should have Locations
enum InterpreterError {
    ArgumentCountMismatch { expected: usize, actual: usize, call_location: Location },
    TypeMismatch { expected: Type, value: Value },
    NoValueForId(DefinitionId),
    IntegerOutOfRangeForType(FieldElement, Type),
    UnableToEvaluateTypeToInteger(Type),
    ErrorNodeEncountered { location: Location },
    NonFunctionCalled { value: Value, location: Location },
    NonBoolUsedInIf { value: Value, location: Location },
    NoMethodFound { object: Value, typ: Type, location: Location },
}

type IResult<T> = std::result::Result<T, InterpreterError>;

impl<'a> Interpreter<'a> {
    fn call_function(
        &mut self,
        function: FuncId,
        arguments: Vec<Value>,
        call_location: Location,
    ) -> IResult<Value> {
        let modifiers = self.interner.function_modifiers(&function);
        assert!(modifiers.is_comptime, "Tried to evaluate non-comptime function!");

        self.push_scope();

        let meta = self.interner.function_meta(&function);

        if meta.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: meta.parameters.len(),
                actual: arguments.len(),
                call_location,
            });
        }

        for ((parameter, typ, _), argument) in meta.parameters.0.iter().zip(arguments) {
            self.define_pattern(parameter, typ, argument);
        }

        match meta.kind {
            FunctionKind::Normal => (),
            other => todo!("Evaluation for {:?} is unimplemented", meta.kind),
        }

        let function_body = self.interner.function(&function).as_expr();
        let result = self.evaluate(function_body)?;

        self.pop_scope();
        Ok(result)
    }

    fn call_closure(
        &mut self,
        closure: HirLambda,
        environment: Vec<Value>,
        arguments: Vec<Value>,
        call_location: Location,
    ) -> IResult<Value> {
        self.push_scope();

        if closure.parameters.len() != arguments.len() {
            return Err(InterpreterError::ArgumentCountMismatch {
                expected: closure.parameters.len(),
                actual: arguments.len(),
                call_location,
            });
        }

        for ((parameter, typ), argument) in closure.parameters.iter().zip(arguments) {
            self.define_pattern(parameter, typ, argument);
        }

        let result = self.evaluate(closure.body)?;

        self.pop_scope();
        Ok(result)
    }

    fn push_scope(&mut self) {
        self.scopes.push(FxHashMap::default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&mut self) -> &mut FxHashMap<DefinitionId, Value> {
        self.scopes.last_mut().unwrap()
    }

    fn define_pattern(&self, pattern: &HirPattern, typ: &Type, argument: Value) -> IResult<()> {
        match pattern {
            HirPattern::Identifier(identifier) => self.define(identifier.id, typ, argument),
            HirPattern::Mutable(pattern, _) => self.define_pattern(pattern, typ, argument),
            HirPattern::Tuple(pattern_fields, _) => {
                self.type_check(typ, &argument)?;

                if let (Value::Tuple(fields), Type::Tuple(type_fields)) = (argument, typ) {
                    // The type check already ensures fields.len() == type_fields.len()
                    if fields.len() == pattern_fields.len() {
                        for ((pattern, typ), argument) in
                            pattern_fields.iter().zip(type_fields).zip(fields)
                        {
                            self.define_pattern(pattern, typ, argument)?;
                        }
                        return Ok(());
                    }
                }

                Err(InterpreterError::TypeMismatch { expected: typ.clone(), value: argument })
            }
            HirPattern::Struct(struct_type, pattern_fields, _) => {
                self.type_check(typ, &argument)?;
                self.type_check(struct_type, &argument)?;

                if let (Value::Struct(fields, _), Type::Struct(struct_def, generics)) =
                    (argument, typ)
                {
                    let struct_def = struct_def.borrow();

                    // The type check already ensures fields.len() == type_fields.len()
                    if fields.len() == pattern_fields.len() {
                        for (field_name, field_pattern) in pattern_fields {
                            let field = fields.get(&field_name.0.contents).unwrap_or_else(|| {
                                panic!("Expected Struct value {argument:?} to have a field named '{field_name}'")
                            });

                            let field_type = struct_def.get_field(&field_name.0.contents, generics).unwrap_or_else(|| {
                                panic!("Expected struct type {typ} to have a field named '{field_name}'")
                            }).0;

                            self.define_pattern(field_pattern, &field_type, field.clone())?;
                        }
                        return Ok(());
                    }
                }

                Err(InterpreterError::TypeMismatch { expected: typ.clone(), value: argument })
            }
        }
    }

    fn define(&self, id: DefinitionId, typ: &Type, argument: Value) -> IResult<()> {
        self.type_check(typ, &argument)?;
        self.current_scope().insert(id, argument);
        Ok(())
    }

    fn lookup(&self, id: DefinitionId) -> IResult<Value> {
        self.current_scope().get(&id).cloned().ok_or_else(|| InterpreterError::NoValueForId(id))
    }

    /// Do a quick, shallow type check to catch some obviously wrong cases.
    /// The interpreter generally relies on expressions to already be well-typed
    /// but this helps catch bugs. It is also worth noting expression types may not
    /// correlate 1-1 with non-comptime code. For example, comptime code also allows
    /// pointers and unsized data types like strings and (unbounded) vectors.
    fn type_check(&self, typ: &Type, value: &Value) -> IResult<()> {
        let typ = typ.follow_bindings();
        use crate::IntegerBitSize::*;
        use crate::Signedness::*;

        match (value, &typ) {
            (Value::Unit, Type::Unit) => (),
            (Value::Bool(_), Type::Bool) => (),
            (Value::Field(_), Type::FieldElement) => (),
            (Value::I8(_), Type::Integer(Signed, Eight)) => (),
            (Value::I32(_), Type::Integer(Signed, ThirtyTwo)) => (),
            (Value::I64(_), Type::Integer(Signed, SixtyFour)) => (),
            (Value::U8(_), Type::Integer(Unsigned, Eight)) => (),
            (Value::U32(_), Type::Integer(Unsigned, ThirtyTwo)) => (),
            (Value::U64(_), Type::Integer(Unsigned, SixtyFour)) => (),
            (Value::String(_), Type::String(_)) => (),
            (Value::Function(..), Type::Function(..)) => (),
            (Value::Tuple(fields1), Type::Tuple(fields2)) if fields1.len() == fields2.len() => (),
            (Value::Struct(..), _) => (),
            (Value::Array(..), Type::Array(..)) => (),
            (Value::Slice(..), Type::Slice(_)) => (),
            (Value::Pointer(_), _) => (),
            (Value::Code(_), Type::Code) => (),
            _ => {
                return Err(InterpreterError::TypeMismatch { expected: typ, value: value.clone() })
            }
        }

        Ok(())
    }

    /// Evaluate an expression and return the result
    fn evaluate(&mut self, id: ExprId) -> IResult<Value> {
        match self.interner.expression(&id) {
            HirExpression::Ident(ident) => self.lookup(ident.id),
            HirExpression::Literal(literal) => self.evaluate_literal(literal, id),
            HirExpression::Block(block) => self.evaluate_block(block),
            HirExpression::Prefix(prefix) => self.evaluate_prefix(prefix),
            HirExpression::Infix(infix) => self.evaluate_infix(infix),
            HirExpression::Index(index) => self.evaluate_index(index),
            HirExpression::Constructor(constructor) => self.evaluate_constructor(constructor, id),
            HirExpression::MemberAccess(access) => self.evaluate_access(access),
            HirExpression::Call(call) => self.evaluate_call(call, id),
            HirExpression::MethodCall(call) => self.evaluate_method_call(call, id),
            HirExpression::Cast(cast) => self.evaluate_cast(cast),
            HirExpression::If(if_) => self.evaluate_if(if_, id),
            HirExpression::Tuple(tuple) => self.evaluate_tuple(tuple),
            HirExpression::Lambda(lambda) => self.evaluate_lambda(lambda, id),
            HirExpression::Quote(block) => Ok(Value::Code(Rc::new(block))),
            HirExpression::Error => {
                let location = self.interner.expr_location(&id);
                Err(InterpreterError::ErrorNodeEncountered { location })
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
        if let Type::Integer(sign, bit_size) = &typ {
            match (sign, bit_size) {
                (Signedness::Unsigned, IntegerBitSize::One) => {
                    panic!("u1 is not supported by the interpreter")
                }
                (Signedness::Unsigned, IntegerBitSize::Eight) => {
                    let value: u8 = value
                        .try_to_u64()
                        .and_then(|value| value.try_into().ok())
                        .ok_or_else(|| InterpreterError::IntegerOutOfRangeForType(value, typ))?;
                    let value = if is_negative { 0u8.wrapping_sub(value) } else { value };
                    Ok(Value::U8(value))
                }
                (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                    let value: u32 = value
                        .try_to_u64()
                        .and_then(|value| value.try_into().ok())
                        .ok_or_else(|| InterpreterError::IntegerOutOfRangeForType(value, typ))?;
                    let value = if is_negative { 0u32.wrapping_sub(value) } else { value };
                    Ok(Value::U32(value))
                }
                (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                    let value: u64 = value
                        .try_to_u64()
                        .and_then(|value| value.try_into().ok())
                        .ok_or_else(|| InterpreterError::IntegerOutOfRangeForType(value, typ))?;
                    let value = if is_negative { 0u64.wrapping_sub(value) } else { value };
                    Ok(Value::U64(value))
                }
                (Signedness::Signed, IntegerBitSize::One) => {
                    panic!("i1 is not supported by the interpreter")
                }
                (Signedness::Signed, IntegerBitSize::Eight) => {
                    let value: i8 = value
                        .try_to_u64()
                        .and_then(|value| value.try_into().ok())
                        .ok_or_else(|| InterpreterError::IntegerOutOfRangeForType(value, typ))?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I8(value))
                }
                (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                    let value: i32 = value
                        .try_to_u64()
                        .and_then(|value| value.try_into().ok())
                        .ok_or_else(|| InterpreterError::IntegerOutOfRangeForType(value, typ))?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I32(value))
                }
                (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                    let value: i64 = value
                        .try_to_u64()
                        .and_then(|value| value.try_into().ok())
                        .ok_or_else(|| InterpreterError::IntegerOutOfRangeForType(value, typ))?;
                    let value = if is_negative { -value } else { value };
                    Ok(Value::I64(value))
                }
            }
        } else {
            unreachable!("Non-integer integer literal of type {typ}")
        }
    }

    fn evaluate_block(&mut self, mut block: HirBlockExpression) -> IResult<Value> {
        let last_statement = block.statements.pop();

        for statement in block.statements {
            self.evaluate_statement(statement);
        }

        if let Some(statement) = last_statement {
            self.evaluate_statement(statement)
        } else {
            Ok(Value::Unit)
        }
    }

    fn evaluate_array(&self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
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
                    Err(InterpreterError::UnableToEvaluateTypeToInteger(length))
                }
            }
        }
    }

    fn evaluate_slice(&self, array: HirArrayLiteral, id: ExprId) -> IResult<Value> {
        self.evaluate_array(array, id).map(|value| match value {
            Value::Array(array, typ) => Value::Slice(array, typ),
            other => unreachable!("Non-array value returned from evaluate array: {other:?}"),
        })
    }

    fn evaluate_prefix(&mut self, prefix: HirPrefixExpression) -> IResult<Value> {
        let rhs = self.evaluate(prefix.rhs)?;
        match prefix.operator {
            crate::UnaryOp::Minus => match rhs {
                Value::Field(value) => Ok(Value::Field(FieldElement::zero() - value)),
                Value::I8(value) => Ok(Value::I8(-value)),
                Value::I32(value) => Ok(Value::I32(-value)),
                Value::I64(value) => Ok(Value::I64(-value)),
                Value::U8(value) => Ok(Value::U8(0 - value)),
                Value::U32(value) => Ok(Value::U32(0 - value)),
                Value::U64(value) => Ok(Value::U64(0 - value)),
                other => panic!("Invalid value for unary minus operation: {other:?}"),
            },
            crate::UnaryOp::Not => match rhs {
                Value::Bool(value) => Ok(Value::Bool(!value)),
                Value::I8(value) => Ok(Value::I8(!value)),
                Value::I32(value) => Ok(Value::I32(!value)),
                Value::I64(value) => Ok(Value::I64(!value)),
                Value::U8(value) => Ok(Value::U8(!value)),
                Value::U32(value) => Ok(Value::U32(!value)),
                Value::U64(value) => Ok(Value::U64(!value)),
                other => panic!("Invalid value for unary not operation: {other:?}"),
            },
            crate::UnaryOp::MutableReference => Ok(Value::Pointer(Shared::new(rhs))),
            crate::UnaryOp::Dereference { implicitly_added: _ } => match rhs {
                Value::Pointer(element) => Ok(element.borrow().clone()),
                other => panic!("Cannot dereference {other:?}"),
            },
        }
    }

    fn evaluate_infix(&mut self, infix: HirInfixExpression) -> IResult<Value> {
        let lhs = self.evaluate(infix.lhs)?;
        let rhs = self.evaluate(infix.rhs)?;

        // TODO: Need to account for operator overloading
        match infix.operator.kind {
            BinaryOpKind::Add => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs + rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs + rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs + rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs + rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs + rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs + rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs + rhs)),
                (lhs, rhs) => panic!("Operator (+) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Subtract => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs - rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs - rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs - rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs - rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs - rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs - rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs - rhs)),
                (lhs, rhs) => panic!("Operator (-) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Multiply => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs * rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs * rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs * rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs * rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs * rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs * rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs * rhs)),
                (lhs, rhs) => panic!("Operator (*) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Divide => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Field(lhs / rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs / rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs / rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs / rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs / rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs / rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs / rhs)),
                (lhs, rhs) => panic!("Operator (/) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Equal => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs == rhs)),
                (lhs, rhs) => panic!("Operator (==) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::NotEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs != rhs)),
                (lhs, rhs) => panic!("Operator (!=) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Less => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs < rhs)),
                (lhs, rhs) => panic!("Operator (<) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::LessEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs <= rhs)),
                (lhs, rhs) => panic!("Operator (<=) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Greater => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs > rhs)),
                (lhs, rhs) => panic!("Operator (>) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::GreaterEqual => match (lhs, rhs) {
                (Value::Field(lhs), Value::Field(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::Bool(lhs >= rhs)),
                (lhs, rhs) => panic!("Operator (>=) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::And => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs & rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs & rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs & rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs & rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs & rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs & rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs & rhs)),
                (lhs, rhs) => panic!("Operator (&) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Or => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs | rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs | rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs | rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs | rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs | rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs | rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs | rhs)),
                (lhs, rhs) => panic!("Operator (|) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Xor => match (lhs, rhs) {
                (Value::Bool(lhs), Value::Bool(rhs)) => Ok(Value::Bool(lhs ^ rhs)),
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs ^ rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs ^ rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs ^ rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs ^ rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs ^ rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs ^ rhs)),
                (lhs, rhs) => panic!("Operator (^) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::ShiftRight => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs >> rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs >> rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs >> rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs >> rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs >> rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs >> rhs)),
                (lhs, rhs) => panic!("Operator (>>) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::ShiftLeft => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs << rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs << rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs << rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs << rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs << rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs << rhs)),
                (lhs, rhs) => panic!("Operator (<<) invalid for values {lhs:?} and {rhs:?}"),
            },
            BinaryOpKind::Modulo => match (lhs, rhs) {
                (Value::I8(lhs), Value::I8(rhs)) => Ok(Value::I8(lhs % rhs)),
                (Value::I32(lhs), Value::I32(rhs)) => Ok(Value::I32(lhs % rhs)),
                (Value::I64(lhs), Value::I64(rhs)) => Ok(Value::I64(lhs % rhs)),
                (Value::U8(lhs), Value::U8(rhs)) => Ok(Value::U8(lhs % rhs)),
                (Value::U32(lhs), Value::U32(rhs)) => Ok(Value::U32(lhs % rhs)),
                (Value::U64(lhs), Value::U64(rhs)) => Ok(Value::U64(lhs % rhs)),
                (lhs, rhs) => panic!("Operator (%) invalid for values {lhs:?} and {rhs:?}"),
            },
        }
    }

    fn evaluate_index(&self, index: HirIndexExpression) -> IResult<Value> {
        let collection = match self.evaluate(index.collection)? {
            Value::Array(array, _) => array,
            Value::Slice(array, _) => array,
            other => panic!("Cannot index into {other:?}"),
        };

        let index = match self.evaluate(index.index)? {
            Value::Field(value) => {
                value.try_to_u64().expect("index could not fit into u64") as usize
            }
            Value::I8(value) => value as usize,
            Value::I32(value) => value as usize,
            Value::I64(value) => value as usize,
            Value::U8(value) => value as usize,
            Value::U32(value) => value as usize,
            Value::U64(value) => value as usize,
            other => panic!("Cannot use {other:?} as an index"),
        };

        Ok(collection[index].clone())
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

    fn evaluate_access(&mut self, access: HirMemberAccess) -> IResult<Value> {
        let fields = match self.evaluate(access.lhs)? {
            Value::Struct(fields, _) => fields,
            other => panic!("Cannot access fields of a non-struct value: {other:?}"),
        };

        Ok(fields
            .get(&access.rhs.0.contents)
            .unwrap_or_else(|| panic!("Expected struct to have field {}", access.rhs))
            .clone())
    }

    fn evaluate_call(&mut self, call: HirCallExpression, id: ExprId) -> IResult<Value> {
        let function = self.evaluate(call.func)?;
        let arguments = try_vecmap(call.arguments, |arg| self.evaluate(arg))?;
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
        let arguments = try_vecmap(call.arguments, |arg| self.evaluate(arg))?;
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
            Err(InterpreterError::NoMethodFound { object, typ, location })
        }
    }

    fn evaluate_cast(&mut self, cast: HirCastExpression) -> IResult<Value> {
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

        let (lhs, lhs_is_negative) = match self.evaluate(cast.lhs)? {
            Value::Field(value) => (value, false),
            Value::U8(value) => ((value as u128).into(), false),
            Value::U32(value) => ((value as u128).into(), false),
            Value::U64(value) => ((value as u128).into(), false),
            Value::I8(value) => signed_int_to_field!(value),
            Value::I32(value) => signed_int_to_field!(value),
            Value::I64(value) => signed_int_to_field!(value),
            Value::Bool(value) => {
                (if value { FieldElement::one() } else { FieldElement::zero() }, false)
            }
            other => unreachable!("Cannot cast from non-numeric value '{other:?}'"),
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
                    panic!("u1 is not supported by the interpreter")
                }
                (Signedness::Unsigned, IntegerBitSize::Eight) => cast_to_int!(lhs, to_u128, u8, U8),
                (Signedness::Unsigned, IntegerBitSize::ThirtyTwo) => {
                    cast_to_int!(lhs, to_u128, u32, U32)
                }
                (Signedness::Unsigned, IntegerBitSize::SixtyFour) => {
                    cast_to_int!(lhs, to_u128, u64, U64)
                }
                (Signedness::Signed, IntegerBitSize::One) => {
                    panic!("i1 is not supported by the interpreter")
                }
                (Signedness::Signed, IntegerBitSize::Eight) => cast_to_int!(lhs, to_i128, i8, I8),
                (Signedness::Signed, IntegerBitSize::ThirtyTwo) => {
                    cast_to_int!(lhs, to_i128, i32, I32)
                }
                (Signedness::Signed, IntegerBitSize::SixtyFour) => {
                    cast_to_int!(lhs, to_i128, i64, I64)
                }
            },
            Type::Bool => Ok(Value::Bool(!lhs.is_zero() || lhs_is_negative)),
            other => unreachable!("Cannot cast to non-numeric type '{other}'"),
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

        if condition {
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
        }
    }

    fn evaluate_tuple(&mut self, tuple: Vec<ExprId>) -> IResult<Value> {
        let fields = try_vecmap(tuple, |field| self.evaluate(field))?;
        Ok(Value::Tuple(fields))
    }

    fn evaluate_lambda(&mut self, lambda: HirLambda, id: ExprId) -> IResult<Value> {
        let environment = try_vecmap(&lambda.captures, |capture| self.lookup(capture.ident.id))?;

        let typ = self.interner.id_type(id);
        Ok(Value::Closure(lambda, environment, typ))
    }

    fn evaluate_statement(&mut self, statement: StmtId) -> IResult<Value> {
        match self.interner.statement(&statement) {
            HirStatement::Let(_) => todo!(),
            HirStatement::Constrain(_) => todo!(),
            HirStatement::Assign(_) => todo!(),
            HirStatement::For(_) => todo!(),
            HirStatement::Break => todo!(),
            HirStatement::Continue => todo!(),
            HirStatement::Expression(_) => todo!(),
            HirStatement::Semi(_) => todo!(),
            HirStatement::Error => todo!(),
        }
    }
}

impl Value {
    fn get_type(&self) -> Cow<Type> {
        Cow::Owned(match self {
            Value::Unit => Type::Unit,
            Value::Bool(_) => Type::Bool,
            Value::Field(_) => Type::FieldElement,
            Value::I8(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Eight),
            Value::I32(_) => Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo),
            Value::I64(_) => Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour),
            Value::U8(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
            Value::U32(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::ThirtyTwo),
            Value::U64(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::SixtyFour),
            Value::String(value) => {
                let length = Type::Constant(value.len() as u64);
                Type::String(Box::new(length))
            }
            Value::Function(_, typ) => return Cow::Borrowed(typ),
            Value::Closure(_, _, typ) => return Cow::Borrowed(typ),
            Value::Tuple(fields) => {
                Type::Tuple(vecmap(fields, |field| field.get_type().into_owned()))
            }
            Value::Struct(_, typ) => return Cow::Borrowed(typ),
            Value::Array(array, typ) => return Cow::Borrowed(typ),
            Value::Slice(slice, typ) => return Cow::Borrowed(typ),
            Value::Code(_) => Type::Code,
            Value::Pointer(_) => panic!("No type equivalent for pointers yet!"),
        })
    }
}
