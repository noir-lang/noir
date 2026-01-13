#![cfg(test)]

use core::panic;

use crate::hir::type_check::TypeCheckError;
use crate::hir_def::types::{BinaryTypeOperator, Type};
use crate::monomorphization::errors::MonomorphizationError;
use crate::signed_field::SignedField;
use crate::test_utils::get_monomorphized;
use crate::tests::{assert_no_errors, check_errors};

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
    assert_no_errors(source);
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

    fn main() { }
    "#;
    assert_no_errors(source);
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

    let monomorphization_error = get_monomorphized(source).unwrap_err();

    // Expect a CheckedCast (0 % 0) failure
    if let MonomorphizationError::UnknownArrayLength { ref length, ref err, location: _ } =
        monomorphization_error
    {
        match length {
            Type::CheckedCast { from, to } => {
                assert!(matches!(*from.clone(), Type::InfixExpr { .. }));
                assert!(matches!(*to.clone(), Type::InfixExpr { .. }));
            }
            _ => panic!("unexpected length: {length:?}"),
        }
        let TypeCheckError::FailingBinaryOp { op, lhs, rhs, .. } = err else {
            panic!("Expected FailingBinaryOp, but found: {err:?}");
        };
        assert_eq!(op, &BinaryTypeOperator::Modulo);
        assert_eq!(lhs, "0");
        assert_eq!(rhs, "0");
    } else {
        panic!("unexpected error: {monomorphization_error:?}");
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

    let monomorphization_error = get_monomorphized(source).unwrap_err();

    // Expect a CheckedCast (0 % 0) failure
    if let MonomorphizationError::UnknownArrayLength { ref length, ref err, location: _ } =
        monomorphization_error
    {
        match length {
            Type::CheckedCast { from, to } => {
                assert!(matches!(*from.clone(), Type::InfixExpr { .. }));
                assert!(matches!(*to.clone(), Type::InfixExpr { .. }));
            }
            _ => panic!("unexpected length: {length:?}"),
        }
        match err {
            TypeCheckError::ModuloOnFields { lhs, rhs, .. } => {
                assert_eq!(lhs.clone(), SignedField::zero());
                assert_eq!(rhs.clone(), SignedField::zero());
            }
            _ => panic!("expected ModuloOnFields, but found: {err:?}"),
        }
    } else {
        panic!("unexpected error: {monomorphization_error:?}");
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
    assert_no_errors(source);
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
    assert_no_errors(source);
}

#[test]
fn arithmetic_generics_rounding_pass() {
    let src = r#"
        fn main() {
            // 3/2*2 = 2
            round::<3, 2>([1, 2]);
        }

        fn round<let N: u32, let M: u32>(_x: [Field; N / M * M]) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn arithmetic_generics_rounding_fail() {
    let src = r#"
        fn main() {
            // Do not simplify N/M*M to just N
            // This should be 3/2*2 = 2, not 3
            round::<3, 2>([1, 2, 3]);
                          ^^^^^^^^^ Expected type [Field; 2], found type [Field; 3]
        }

        fn round<let N: u32, let M: u32>(_x: [Field; N / M * M]) {}
    "#;
    check_errors(src);
}

#[test]
fn arithmetic_generics_rounding_fail_on_struct() {
    let src = r#"
        struct W<let N: u32> {}

        fn foo<let N: u32, let M: u32>(_x: W<N>, _y: W<M>) -> W<N / M * M> {
            W {}
        }

        fn main() {
            let w_2: W<2> = W {};
            let w_3: W<3> = W {};
            // Do not simplify N/M*M to just N
            // This should be 3/2*2 = 2, not 3
            let _: W<3> = foo(w_3, w_2);
                          ^^^^^^^^^^^^^ Expected type W<3>, found type W<2>
        }
    "#;
    check_errors(src);
}

#[test]
fn allows_struct_with_generic_infix_type_as_main_input_1() {
    let src = r#"
        struct Foo<let N: u32> {
            x: [u64; N * 2],
        }

        fn main(_x: Foo<18>) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn allows_struct_with_generic_infix_type_as_main_input_2() {
    let src = r#"
        struct Foo<let N: u32> {
            x: [u64; N * 2],
        }

        fn main(_x: Foo<2 * 9>) {}
    "#;
    assert_no_errors(src);
}

#[test]
fn allows_struct_with_generic_infix_type_as_main_input_3() {
    let src = r#"
        struct Foo<let N: u32> {
            x: [u64; N * 2],
        }

        global N: u32 = 9;

        fn main(_x: Foo<N * 2>) {}
    "#;
    assert_no_errors(src);
}
