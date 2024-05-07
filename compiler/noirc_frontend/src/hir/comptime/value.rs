use std::{borrow::Cow, rc::Rc};

use acvm::FieldElement;
use im::Vector;
use iter_extended::{try_vecmap, vecmap};
use noirc_errors::Location;

use crate::{
    ast::{BlockExpression, Ident, IntegerBitSize, Signedness},
    hir_def::expr::{HirArrayLiteral, HirConstructorExpression, HirIdent, HirLambda, ImplKind},
    macros_api::{HirExpression, HirLiteral, NodeInterner},
    node_interner::{ExprId, FuncId},
    Shared, Type,
};
use rustc_hash::FxHashMap as HashMap;

use super::errors::{IResult, InterpreterError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Field(FieldElement),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    String(Rc<String>),
    Function(FuncId, Type),
    Closure(HirLambda, Vec<Value>, Type),
    Tuple(Vec<Value>),
    Struct(HashMap<Rc<String>, Value>, Type),
    Pointer(Shared<Value>),
    Array(Vector<Value>, Type),
    Slice(Vector<Value>, Type),
    Code(Rc<BlockExpression>),
}

impl Value {
    pub(crate) fn get_type(&self) -> Cow<Type> {
        Cow::Owned(match self {
            Value::Unit => Type::Unit,
            Value::Bool(_) => Type::Bool,
            Value::Field(_) => Type::FieldElement,
            Value::I8(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Eight),
            Value::I16(_) => Type::Integer(Signedness::Signed, IntegerBitSize::Sixteen),
            Value::I32(_) => Type::Integer(Signedness::Signed, IntegerBitSize::ThirtyTwo),
            Value::I64(_) => Type::Integer(Signedness::Signed, IntegerBitSize::SixtyFour),
            Value::U8(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Eight),
            Value::U16(_) => Type::Integer(Signedness::Unsigned, IntegerBitSize::Sixteen),
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
            Value::Array(_, typ) => return Cow::Borrowed(typ),
            Value::Slice(_, typ) => return Cow::Borrowed(typ),
            Value::Code(_) => Type::Code,
            Value::Pointer(element) => {
                let element = element.borrow().get_type().into_owned();
                Type::MutableReference(Box::new(element))
            }
        })
    }

    pub(crate) fn into_expression(
        self,
        interner: &mut NodeInterner,
        location: Location,
    ) -> IResult<ExprId> {
        let typ = self.get_type().into_owned();

        let expression = match self {
            Value::Unit => HirExpression::Literal(HirLiteral::Unit),
            Value::Bool(value) => HirExpression::Literal(HirLiteral::Bool(value)),
            Value::Field(value) => HirExpression::Literal(HirLiteral::Integer(value, false)),
            Value::I8(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::I16(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::I32(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::I64(value) => {
                let negative = value < 0;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            }
            Value::U8(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U16(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U32(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::U64(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            }
            Value::String(value) => HirExpression::Literal(HirLiteral::Str(unwrap_rc(value))),
            Value::Function(id, _typ) => {
                let id = interner.function_definition_id(id);
                let impl_kind = ImplKind::NotATraitMethod;
                HirExpression::Ident(HirIdent { location, id, impl_kind })
            }
            Value::Closure(_lambda, _env, _typ) => {
                // TODO: How should a closure's environment be inlined?
                let item = "Returning closures from a comptime fn";
                return Err(InterpreterError::Unimplemented { item, location });
            }
            Value::Tuple(fields) => {
                let fields = try_vecmap(fields, |field| field.into_expression(interner, location))?;
                HirExpression::Tuple(fields)
            }
            Value::Struct(fields, typ) => {
                let fields = try_vecmap(fields, |(name, field)| {
                    let field = field.into_expression(interner, location)?;
                    Ok((Ident::new(unwrap_rc(name), location.span), field))
                })?;

                let (r#type, struct_generics) = match typ.follow_bindings() {
                    Type::Struct(def, generics) => (def, generics),
                    _ => return Err(InterpreterError::NonStructInConstructor { typ, location }),
                };

                HirExpression::Constructor(HirConstructorExpression {
                    r#type,
                    struct_generics,
                    fields,
                })
            }
            Value::Array(elements, _) => {
                let elements =
                    try_vecmap(elements, |elements| elements.into_expression(interner, location))?;
                HirExpression::Literal(HirLiteral::Array(HirArrayLiteral::Standard(elements)))
            }
            Value::Slice(elements, _) => {
                let elements =
                    try_vecmap(elements, |elements| elements.into_expression(interner, location))?;
                HirExpression::Literal(HirLiteral::Slice(HirArrayLiteral::Standard(elements)))
            }
            Value::Code(block) => HirExpression::Unquote(unwrap_rc(block)),
            Value::Pointer(_) => {
                return Err(InterpreterError::CannotInlineMacro { value: self, location })
            }
        };

        let id = interner.push_expr(expression);
        interner.push_expr_location(id, location.span, location.file);
        interner.push_expr_type(id, typ);
        Ok(id)
    }
}

/// Unwraps an Rc value without cloning the inner value if the reference count is 1. Clones otherwise.
fn unwrap_rc<T: Clone>(rc: Rc<T>) -> T {
    Rc::try_unwrap(rc).unwrap_or_else(|rc| (*rc).clone())
}
