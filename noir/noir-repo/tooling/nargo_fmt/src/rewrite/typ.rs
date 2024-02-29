use noirc_frontend::{UnresolvedType, UnresolvedTypeData};

use crate::{
    utils::span_is_empty,
    visitor::{FmtVisitor, Shape},
};

pub(crate) fn rewrite(visitor: &FmtVisitor, _shape: Shape, typ: UnresolvedType) -> String {
    match typ.typ {
        UnresolvedTypeData::Array(length, element) => {
            let typ = rewrite(visitor, _shape, *element);
            if let Some(length) = length {
                let length = visitor.slice(length.span());
                format!("[{typ}; {length}]")
            } else {
                format!("[{typ}]")
            }
        }
        UnresolvedTypeData::Parenthesized(typ) => {
            let typ = rewrite(visitor, _shape, *typ);
            format!("({typ})")
        }
        UnresolvedTypeData::MutableReference(typ) => {
            let typ = rewrite(visitor, _shape, *typ);
            format!("&mut {typ}")
        }
        UnresolvedTypeData::Tuple(mut types) => {
            if types.len() == 1 {
                let typ = types.pop().unwrap();
                let typ = rewrite(visitor, _shape, typ);

                format!("({typ},)")
            } else {
                let types: Vec<_> =
                    types.into_iter().map(|typ| rewrite(visitor, _shape, typ)).collect();
                let types = types.join(", ");
                format!("({types})")
            }
        }
        UnresolvedTypeData::Function(args, return_type, env) => {
            let env = if span_is_empty(env.span.unwrap()) {
                "".into()
            } else {
                let ty = rewrite(visitor, _shape, *env);
                format!("[{ty}]")
            };

            let args = args
                .into_iter()
                .map(|arg| rewrite(visitor, _shape, arg))
                .collect::<Vec<_>>()
                .join(", ");

            let return_type = rewrite(visitor, _shape, *return_type);

            format!("fn{env}({args}) -> {return_type}")
        }
        UnresolvedTypeData::Unspecified => todo!(),
        UnresolvedTypeData::FieldElement
        | UnresolvedTypeData::Integer(_, _)
        | UnresolvedTypeData::Bool
        | UnresolvedTypeData::Named(_, _, _)
        | UnresolvedTypeData::Unit
        | UnresolvedTypeData::Expression(_)
        | UnresolvedTypeData::String(_)
        | UnresolvedTypeData::FormatString(_, _)
        | UnresolvedTypeData::TraitAsType(_, _) => visitor.slice(typ.span.unwrap()).into(),
        UnresolvedTypeData::Error => unreachable!(),
    }
}
