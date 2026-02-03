use std::collections::BTreeMap;

use noirc_abi::{Abi, AbiParameter, AbiReturnType, AbiType, AbiVisibility, Sign};
use noirc_frontend::{
    monomorphization::ast::{Program, Type},
    shared::Visibility,
};

/// Generate the [Abi] interface of a [Program].
pub fn program_abi(program: &Program) -> Abi {
    let main = program.main();

    let parameters = main
        .parameters
        .iter()
        .map(|(_id, _is_mutable, name, typ, vis)| AbiParameter {
            name: name.clone(),
            typ: to_abi_type(typ),
            visibility: to_abi_visibility(vis),
        })
        .collect();

    let return_type = match &main.return_type {
        Type::Unit => None,
        typ => Some(AbiReturnType {
            abi_type: to_abi_type(typ),
            visibility: to_abi_visibility(&program.return_visibility()),
        }),
    };

    Abi { parameters, return_type, error_types: BTreeMap::default() }
}

/// Check if a type is valid as an ABI parameter for the `main` function.
fn is_valid_in_abi(typ: &Type) -> bool {
    match typ {
        Type::Unit
        | Type::FmtString(_, _)
        | Type::Vector(_)
        | Type::Reference(_, _)
        | Type::Function(_, _, _, _) => false,

        Type::Field | Type::Bool | Type::String(_) | Type::Integer(_, _) => true,

        Type::Array(_, typ) => is_valid_in_abi(typ),
        Type::Tuple(items) => items.iter().all(is_valid_in_abi),
    }
}

/// Map an AST type to an ABI type.
///
/// Panics if it's called on a type which should not appear in the ABI.
fn to_abi_type(typ: &Type) -> AbiType {
    match typ {
        Type::Field => AbiType::Field,
        Type::Array(len, typ) => AbiType::Array { length: *len, typ: Box::new(to_abi_type(typ)) },
        Type::Integer(signedness, integer_bit_size) => AbiType::Integer {
            sign: if signedness.is_signed() { Sign::Signed } else { Sign::Unsigned },
            width: integer_bit_size.bit_size().into(),
        },
        Type::Bool => AbiType::Boolean,
        Type::String(len) => AbiType::String { length: *len },
        Type::Tuple(items) => AbiType::Tuple { fields: items.iter().map(to_abi_type).collect() },

        _ => {
            if !is_valid_in_abi(typ) {
                // We should not have generated such a parameter for the main function.
                panic!("Invalid type in ABI: {typ}");
            } else {
                unreachable!("Unexpected type in ABI: {typ}")
            }
        }
    }
}

fn to_abi_visibility(vis: &Visibility) -> AbiVisibility {
    match vis {
        Visibility::Public => AbiVisibility::Public,
        Visibility::Private => AbiVisibility::Public,
        Visibility::CallData(_) => AbiVisibility::DataBus,
        Visibility::ReturnData => AbiVisibility::DataBus,
    }
}
