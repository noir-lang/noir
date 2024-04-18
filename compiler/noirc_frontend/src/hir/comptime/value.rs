use std::{borrow::Cow, rc::Rc};

use acvm::FieldElement;
use im::Vector;
use iter_extended::vecmap;

use crate::{
    hir_def::expr::HirLambda, node_interner::FuncId, BlockExpression, IntegerBitSize, Shared,
    Signedness, Type,
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
}
