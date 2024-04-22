use std::{borrow::Cow, rc::Rc};

use acvm::FieldElement;
use im::Vector;
use iter_extended::vecmap;

use crate::{
    hir_def::expr::HirLambda, node_interner::{FuncId, ExprId}, BlockExpression, IntegerBitSize, Shared,
    Signedness, Type, macros_api::{NodeInterner, HirExpression, HirLiteral},
};
use rustc_hash::FxHashMap as HashMap;

#[allow(unused)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
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
            Value::Array(_, typ) => return Cow::Borrowed(typ),
            Value::Slice(_, typ) => return Cow::Borrowed(typ),
            Value::Code(_) => Type::Code,
            Value::Pointer(element) => {
                let element = element.borrow().get_type().into_owned();
                Type::MutableReference(Box::new(element))
            }
        })
    }

    pub(crate) fn into_expression(self, interner: &mut NodeInterner) -> ExprId {
        let expression = match self {
            Value::Unit => todo!(),
            Value::Bool(value) => {
                HirExpression::Literal(HirLiteral::Bool(value))
            },
            Value::Field(value) => {
                HirExpression::Literal(HirLiteral::Integer(value, false))
            },
            Value::I8(value) => {
                let negative = value < 0 ;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            },
            Value::I32(value) => {
                let negative = value < 0 ;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            },
            Value::I64(value) => {
                let negative = value < 0 ;
                let value = value.abs();
                let value = (value as u128).into();
                HirExpression::Literal(HirLiteral::Integer(value, negative))
            },
            Value::U8(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            },
            Value::U32(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            },
            Value::U64(value) => {
                HirExpression::Literal(HirLiteral::Integer((value as u128).into(), false))
            },
            Value::String(value) => {
                HirExpression::Literal(HirLiteral::Str(value.as_ref().clone()))
            },
            Value::Function(_, _) => todo!(),
            Value::Closure(_, _, _) => todo!(),
            Value::Tuple(_) => todo!(),
            Value::Struct(_, _) => todo!(),
            Value::Pointer(_) => todo!(),
            Value::Array(_, _) => todo!(),
            Value::Slice(_, _) => todo!(),
            Value::Code(_) => todo!(),
        };
        interner.push_expr(expression)
    }
}
