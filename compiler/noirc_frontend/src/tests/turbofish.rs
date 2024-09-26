use crate::hir::{def_collector::dc_crate::CompilationError, type_check::TypeCheckError};

use super::{assert_no_errors, get_program_errors};

#[test]
fn turbofish_numeric_generic_nested_call() {
    // Check for turbofish numeric generics used with function calls
    let src = r#"
    fn foo<let N: u32>() -> [u8; N] {
        [0; N]
    }

    fn bar<let N: u32>() -> [u8; N] {
        foo::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors(src);

    // Check for turbofish numeric generics used with method calls
    let src = r#"
    struct Foo<T> {
        a: T
    }

    impl<T> Foo<T> {
        fn static_method<let N: u32>() -> [u8; N] {
            [0; N]
        }

        fn impl_method<let N: u32>(self) -> [T; N] {
            [self.a; N]
        }
    }

    fn bar<let N: u32>() -> [u8; N] {
        let _ = Foo::static_method::<N>();
        let x: Foo<u8> = Foo { a: 0 };
        x.impl_method::<N>()
    }

    global M: u32 = 3;

    fn main() {
        let _ = bar::<M>();
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_constructor_generics_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let _ = Foo::<i32, i64> { x: 1 };
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::GenericCountMismatch { .. }),
    ));
}

#[test]
fn turbofish_in_constructor() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let x: Field = 0;
        let _ = Foo::<i32> { x: x };
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::TypeMismatch {
        expected_typ, expr_typ, ..
    }) = &errors[0].0
    else {
        panic!("Expected a type mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(expected_typ, "i32");
    assert_eq!(expr_typ, "Field");
}

#[test]
fn turbofish_in_middle_of_variable_unsupported_yet() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    impl <T> Foo<T> {
        fn new(x: T) -> Self {
            Foo { x }
        }
    }

    fn main() {
        let _ = Foo::<i32>::new(1);
    }
    "#;
    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::UnsupportedTurbofishUsage { .. }),
    ));
}

#[test]
fn turbofish_in_struct_pattern() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<Field> { x } = Foo { x: value };
        let _ = x;
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn turbofish_in_struct_pattern_errors_if_type_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value: Field = 0;
        let Foo::<i32> { x } = Foo { x: value };
        let _ = x;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::TypeMismatchWithSource { .. }) = &errors[0].0
    else {
        panic!("Expected a type mismatch error, got {:?}", errors[0].0);
    };
}

#[test]
fn turbofish_in_struct_pattern_generic_count_mismatch() {
    let src = r#"
    struct Foo<T> {
        x: T
    }

    fn main() {
        let value = 0;
        let Foo::<i32, i64> { x } = Foo { x: value };
        let _ = x;
    }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);

    let CompilationError::TypeError(TypeCheckError::GenericCountMismatch {
        item,
        expected,
        found,
        ..
    }) = &errors[0].0
    else {
        panic!("Expected a generic count mismatch error, got {:?}", errors[0].0);
    };

    assert_eq!(item, "struct Foo");
    assert_eq!(*expected, 1);
    assert_eq!(*found, 2);
}
