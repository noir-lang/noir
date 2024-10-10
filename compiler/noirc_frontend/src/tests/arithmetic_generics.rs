#![cfg(test)]

use proptest::arbitrary::any;
use proptest::prelude::*;
use proptest::result::maybe_ok;
use proptest::strategy;

use acvm::{AcirField, FieldElement};

use crate::ast::{IntegerBitSize, Signedness};
use crate::hir_def::types::{BinaryTypeOperator, Kind, Type, TypeVariable, TypeVariableId};
use super::get_program_errors;

// DONE:
// - example test with constant types/expressions/values
// - generate arbitrary unsigned type
// - generate arbitrary value of unsigned type

// TODO:
// - generate arbitrary InfixExpr
//     + lhs/rhs: Constant, NamedGeneric, InfixExpr
// - fetch deduplicated variables from InfixExpr
//     + calculate number of variables in InfixExpr
// - instantiate InfixExpr from list of values


#[test]
fn arithmetic_generics_canonicalization_deduplication_regression() {
    let source = r#"
        struct ArrData<let N: u32> {
            a: [Field; N],
            b: [Field; N + N - 1],
        }

        fn main() {
            let _f: ArrData<5> = ArrData {
                a: [0; 5],
                b: [0; 9],
            };
        }
    "#;
    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 0);
}

prop_compose! {
    // maximum_size must be non-zero
    fn arbitrary_u128_field_element(maximum_size: u128)
        (u128_value in any::<u128>())
        -> FieldElement
    {
        assert!(maximum_size != 0);
        FieldElement::from(u128_value % maximum_size)
    }
}

// TODO import from acvm (requires adding a 'test' feature to export)
// use acvm::tests::solver::field_element as arbitrary_field_element;
prop_compose! {
    // Use both `u128` and hex proptest strategies
    // fn field_element()
    fn arbitrary_field_element()
        (u128_or_hex in maybe_ok(any::<u128>(), "[0-9a-f]{64}"))
        -> FieldElement
    {
        match u128_or_hex {
            Ok(number) => FieldElement::from(number),
            Err(hex) => FieldElement::from_hex(&hex).expect("should accept any 32 byte hex string"),
        }
    }
}

// Generate (arbitrary_unsigned_type, generator for that type)
fn arbitrary_unsigned_type_with_generator() -> BoxedStrategy<(Type, BoxedStrategy<FieldElement>)> {
    prop_oneof![
        strategy::Just((Type::FieldElement, arbitrary_field_element().boxed())),
        any::<IntegerBitSize>().prop_map(|bit_size| {
            let typ = Type::Integer(Signedness::Unsigned, bit_size);
            let maximum_size = typ.integral_maximum_size().unwrap().to_u128();
            (typ, arbitrary_u128_field_element(maximum_size).boxed())
        }),
        strategy::Just((Type::Bool, arbitrary_u128_field_element(1).boxed())),
    ].boxed()
}

#[test]
fn instantiate_before_or_after_canonicalize_eq() {
    // TODO generate these
    let field_element_kind = Kind::Numeric(Box::new(Type::FieldElement));
    let x_var = TypeVariable::unbound(TypeVariableId(0), field_element_kind.clone());
    let x_type = Type::TypeVariable(x_var.clone());
    let one = Type::Constant(FieldElement::one(), field_element_kind.clone());

    let lhs = Type::InfixExpr(Box::new(x_type.clone()), BinaryTypeOperator::Addition, Box::new(one.clone()));
    let rhs = Type::InfixExpr(Box::new(one), BinaryTypeOperator::Addition, Box::new(x_type.clone()));

    // canonicalize
    let lhs = lhs.canonicalize();
    let rhs = rhs.canonicalize();

    // bind vars
    let two = Type::Constant(FieldElement::one() + FieldElement::one(), field_element_kind.clone());
    x_var.bind(two);

    // canonicalize (expect constant)
    let lhs = lhs.canonicalize();
    let rhs = rhs.canonicalize();

    // ensure we've canonicalized to constants
    assert!(matches!(lhs, Type::Constant(..)));
    assert!(matches!(rhs, Type::Constant(..)));

    // ensure result kinds are the same as the original kind
    assert_eq!(lhs.kind(), field_element_kind);
    assert_eq!(rhs.kind(), field_element_kind);

    // ensure results are the same
    assert_eq!(lhs, rhs);
}

