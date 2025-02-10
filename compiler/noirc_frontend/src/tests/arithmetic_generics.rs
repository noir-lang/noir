#![cfg(test)]

use acvm::{AcirField, FieldElement};

use super::get_program_errors;
use crate::hir::type_check::TypeCheckError;
use crate::hir_def::types::{BinaryTypeOperator, Type};
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
fn checked_casts_do_not_prevent_canonicalization() {
    // Regression test for https://github.com/noir-lang/noir/issues/6495
    let source = r#"
    pub trait Serialize<let N: u32> {
        fn serialize(self) -> [Field; N];
    }

    pub struct Counted<T> {
        pub inner: T,
    }

    pub fn append<T, let N: u32>(array1: [T; N]) -> [T; N + 1] {
        [array1[0]; N + 1]
    }

    impl<T, let N: u32> Serialize<N> for Counted<T>
    where
        T: Serialize<N - 1>,
    {
        fn serialize(self) -> [Field; N] {
            append(self.inner.serialize())
        }
    }
    "#;
    let errors = get_program_errors(source);
    println!("{:?}", errors);
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

#[test]
fn global_numeric_generic_larger_than_u32() {
    // Regression test for https://github.com/noir-lang/noir/issues/6125
    let source = r#"
    global A: Field = 4294967297;
    
    fn foo<let A: Field>() { }
    
    fn main() {
        let _ = foo::<A>();
    }
    "#;
    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 0);
}

#[test]
fn global_arithmetic_generic_larger_than_u32() {
    // Regression test for https://github.com/noir-lang/noir/issues/6126
    let source = r#"
    struct Foo<let F: Field> {}
    
    impl<let F: Field> Foo<F> {
        fn size(self) -> Field {
            let _ = self;
            F
        }
    }
    
    // 2^32 - 1
    global A: Field = 4294967295;
    
    // Avoiding overflow succeeds:
    // fn foo<let A: Field>() -> Foo<A> {
    fn foo<let A: Field>() -> Foo<A + A> {
        Foo {}
    }
    
    fn main() {
        let _ = foo::<A>().size();
    }
    "#;
    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 0);
}
