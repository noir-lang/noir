#![cfg(test)]

use proptest::arbitrary::any;
use proptest::collection;
use proptest::prelude::*;
use proptest::result::maybe_ok;
use proptest::strategy;

use acvm::{AcirField, FieldElement};

use super::get_program_errors;
use crate::ast::{IntegerBitSize, Signedness};
use crate::hir::type_check::TypeCheckError;
use crate::hir_def::types::{BinaryTypeOperator, Kind, Type, TypeVariable, TypeVariableId};
use crate::monomorphization::errors::MonomorphizationError;
use crate::tests::get_monomorphization_error;

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

#[test]
fn arithmetic_generics_checked_cast_zeros() {
    let source = r#"
        struct W<let N: u1> {}
        
        fn foo<let N: u1>(_x: W<N>) -> W<(0 * N) / (N % N)> {
            W {}
        }
        
        fn bar<let N: u1>(_x: W<N>) -> u1 {
            N
        }
        
        fn main() -> pub u1 {
            let w_0: W<0> = W {};
            let w: W<_> = foo(w_0);
            bar(w)
        }
    "#;

    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 0);

    let monomorphization_error = get_monomorphization_error(source);
    assert!(monomorphization_error.is_some());

    // Expect a CheckedCast (0 % 0) failure
    let monomorphization_error = monomorphization_error.unwrap();
    if let MonomorphizationError::UnknownArrayLength { ref length, ref err, location: _ } =
        monomorphization_error
    {
        match length {
            Type::CheckedCast { from, to } => {
                assert!(matches!(*from.clone(), Type::InfixExpr { .. }));
                assert!(matches!(*to.clone(), Type::InfixExpr { .. }));
            }
            _ => panic!("unexpected length: {:?}", length),
        }
        assert!(matches!(
            err,
            TypeCheckError::FailingBinaryOp { op: BinaryTypeOperator::Modulo, lhs: 0, rhs: 0, .. }
        ));
    } else {
        panic!("unexpected error: {:?}", monomorphization_error);
    }
}

#[test]
fn arithmetic_generics_checked_cast_indirect_zeros() {
    let source = r#"
        struct W<let N: Field> {}
        
        fn foo<let N: Field>(_x: W<N>) -> W<(N - N) % (N - N)> {
            W {}
        }
        
        fn bar<let N: Field>(_x: W<N>) -> Field {
            N
        }
        
        fn main() {
            let w_0: W<0> = W {};
            let w = foo(w_0);
            let _ = bar(w);
        }
    "#;

    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 0);

    let monomorphization_error = get_monomorphization_error(source);
    assert!(monomorphization_error.is_some());

    // Expect a CheckedCast (0 % 0) failure
    let monomorphization_error = monomorphization_error.unwrap();
    if let MonomorphizationError::UnknownArrayLength { ref length, ref err, location: _ } =
        monomorphization_error
    {
        match length {
            Type::CheckedCast { from, to } => {
                assert!(matches!(*from.clone(), Type::InfixExpr { .. }));
                assert!(matches!(*to.clone(), Type::InfixExpr { .. }));
            }
            _ => panic!("unexpected length: {:?}", length),
        }
        match err {
            TypeCheckError::ModuloOnFields { lhs, rhs, .. } => {
                assert_eq!(lhs.clone(), FieldElement::zero());
                assert_eq!(rhs.clone(), FieldElement::zero());
            }
            _ => panic!("expected ModuloOnFields, but found: {:?}", err),
        }
    } else {
        panic!("unexpected error: {:?}", monomorphization_error);
    }
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

// NOTE: this is roughly the same method from acvm/tests/solver
prop_compose! {
    // Use both `u128` and hex proptest strategies
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
    ]
    .boxed()
}

prop_compose! {
    fn arbitrary_variable(typ: Type, num_variables: usize)
        (variable_index in any::<usize>())
    -> Type {
        assert!(num_variables != 0);
        let id = TypeVariableId(variable_index % num_variables);
        let kind = Kind::numeric(typ.clone());
        let var = TypeVariable::unbound(id, kind);
        Type::TypeVariable(var)
    }
}

fn first_n_variables(typ: Type, num_variables: usize) -> impl Iterator<Item = TypeVariable> {
    (0..num_variables).map(move |id| {
        let id = TypeVariableId(id);
        let kind = Kind::numeric(typ.clone());
        TypeVariable::unbound(id, kind)
    })
}

fn arbitrary_infix_expr(
    typ: Type,
    arbitrary_value: BoxedStrategy<FieldElement>,
    num_variables: usize,
) -> impl Strategy<Value = Type> {
    let leaf = prop_oneof![
        arbitrary_variable(typ.clone(), num_variables),
        arbitrary_value.prop_map(move |value| Type::Constant(value, Kind::numeric(typ.clone()))),
    ];

    leaf.prop_recursive(
        8,   // 8 levels deep maximum
        256, // Shoot for maximum size of 256 nodes
        10,  // We put up to 10 items per collection
        |inner| {
            (inner.clone(), any::<BinaryTypeOperator>(), inner)
                .prop_map(|(lhs, op, rhs)| Type::InfixExpr(Box::new(lhs), op, Box::new(rhs)))
        },
    )
}

prop_compose! {
    // (infix_expr, type, generator)
    fn arbitrary_infix_expr_type_gen(num_variables: usize)
        (type_and_gen in arbitrary_unsigned_type_with_generator())
        (infix_expr in arbitrary_infix_expr(type_and_gen.clone().0, type_and_gen.clone().1, num_variables), type_and_gen in Just(type_and_gen))
    -> (Type, Type, BoxedStrategy<FieldElement>) {
        let (typ, value_generator) = type_and_gen;
        (infix_expr, typ, value_generator)
    }
}

prop_compose! {
    // (Type::InfixExpr, numeric kind, bindings)
    fn arbitrary_infix_expr_with_bindings_sized(num_variables: usize)
        (infix_type_gen in arbitrary_infix_expr_type_gen(num_variables))
        (values in collection::vec(infix_type_gen.clone().2, num_variables), infix_type_gen in Just(infix_type_gen))
    -> (Type, Type, Vec<(TypeVariable, Type)>) {
        let (infix_expr, typ, _value_generator) = infix_type_gen;
        let bindings: Vec<_> = first_n_variables(typ.clone(), num_variables)
            .zip(values.iter().map(|value| {
                Type::Constant(*value, Kind::numeric(typ.clone()))
            }))
            .collect();
        (infix_expr, typ, bindings)
    }
}

prop_compose! {
    // the lint misfires on 'num_variables'
    #[allow(unused_variables)]
    fn arbitrary_infix_expr_with_bindings(max_num_variables: usize)
        (num_variables in any::<usize>().prop_map(move |num_variables| (num_variables % max_num_variables).clamp(1, max_num_variables)))
        (infix_type_bindings in arbitrary_infix_expr_with_bindings_sized(num_variables), num_variables in Just(num_variables))
    -> (Type, Type, Vec<(TypeVariable, Type)>) {
        infix_type_bindings
    }
}

#[test]
fn instantiate_after_canonicalize_smoke_test() {
    let field_element_kind = Kind::numeric(Type::FieldElement);
    let x_var = TypeVariable::unbound(TypeVariableId(0), field_element_kind.clone());
    let x_type = Type::TypeVariable(x_var.clone());
    let one = Type::Constant(FieldElement::one(), field_element_kind.clone());

    let lhs = Type::InfixExpr(
        Box::new(x_type.clone()),
        BinaryTypeOperator::Addition,
        Box::new(one.clone()),
    );
    let rhs =
        Type::InfixExpr(Box::new(one), BinaryTypeOperator::Addition, Box::new(x_type.clone()));

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

proptest! {
    #[test]
    // Expect cases that don't resolve to constants, e.g. see
    // `arithmetic_generics_checked_cast_indirect_zeros`
    #[should_panic(expected = "matches!(infix, Type :: Constant(..))")]
    fn instantiate_before_or_after_canonicalize(infix_type_bindings in arbitrary_infix_expr_with_bindings(10)) {
        let (infix, typ, bindings) = infix_type_bindings;

        // canonicalize
        let infix_canonicalized = infix.canonicalize();

        // bind vars
        for (var, binding) in bindings {
            var.bind(binding);
        }

        // attempt to canonicalize to a constant
        let infix = infix.canonicalize();
        let infix_canonicalized = infix_canonicalized.canonicalize();

        // ensure we've canonicalized to constants
        prop_assert!(matches!(infix, Type::Constant(..)));
        prop_assert!(matches!(infix_canonicalized, Type::Constant(..)));

        // ensure result kinds are the same as the original kind
        let kind = Kind::numeric(typ);
        prop_assert_eq!(infix.kind(), kind.clone());
        prop_assert_eq!(infix_canonicalized.kind(), kind);

        // ensure results are the same
        prop_assert_eq!(infix, infix_canonicalized);
    }

    #[test]
    fn instantiate_before_or_after_canonicalize_checked_cast(infix_type_bindings in arbitrary_infix_expr_with_bindings(10)) {
        let (infix, typ, bindings) = infix_type_bindings;

        // wrap in CheckedCast
        let infix = Type::CheckedCast {
            from: Box::new(infix.clone()),
            to: Box::new(infix)
        };

        // canonicalize
        let infix_canonicalized = infix.canonicalize();

        // bind vars
        for (var, binding) in bindings {
            var.bind(binding);
        }

        // attempt to canonicalize to a constant
        let infix = infix.canonicalize();
        let infix_canonicalized = infix_canonicalized.canonicalize();

        // ensure result kinds are the same as the original kind
        let kind = Kind::numeric(typ);
        prop_assert_eq!(infix.kind(), kind.clone());
        prop_assert_eq!(infix_canonicalized.kind(), kind.clone());

        // ensure the results are still wrapped in CheckedCast's
        match (&infix, &infix_canonicalized) {
            (Type::CheckedCast { from, to }, Type::CheckedCast { from: from_canonicalized, to: to_canonicalized }) => {
                // ensure from's are the same
                prop_assert_eq!(from, from_canonicalized);

                // ensure to's have the same kinds
                prop_assert_eq!(to.kind(), kind.clone());
                prop_assert_eq!(to_canonicalized.kind(), kind);
            }
            _ => {
                prop_assert!(false, "expected CheckedCast");
            }
        }
    }
}
