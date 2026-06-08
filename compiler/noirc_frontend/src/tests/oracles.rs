use crate::monomorphization::errors::MonomorphizationError;
use crate::test_utils::{get_monomorphized_with_stdlib, stdlib_src};
use crate::tests::{
    assert_no_errors, check_errors, check_errors_using_features, check_monomorphization_error,
};

#[test]
fn deny_oracle_attribute_on_non_unconstrained() {
    let src = r#"
        #[oracle(foo)]
        ^^^^^^^^^^^^^^ Usage of the `#[oracle]` function attribute is only valid on unconstrained functions
        pub fn foo(x: Field, y: Field) {
               ~~~ Oracle functions must have the `unconstrained` keyword applied
        }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_declaration_has_function_body() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call() {
                         ^^^^^^^^^^^ Functions marked with #[oracle] must have no body
                         ~~~~~~~~~~~ This function body will never be run so should be removed
        assert(true);
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_returns_multiple_vectors() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call() -> ([u32], [Field]) {}
                         ^^^^^^^^^^^ Oracle functions cannot return multiple vectors
    "#;
    check_errors(src);
}

#[test]
fn does_not_error_if_oracle_called_from_constrained_directly() {
    // Assuming that direct oracle calls will be automatically wrapped in a proxy function.
    let src = r#"
    fn main() {
        // safety:
        unsafe {
            oracle_call();
        }
    }

    #[oracle(oracle_call)]
    unconstrained fn oracle_call() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_oracle_called_from_constrained_via_local_var() {
    let src = r#"
    fn main() {
        let oracle: unconstrained fn() = oracle_call;

        // safety:
        unsafe {
            oracle();
        }
    }

    #[oracle(oracle_call)]
    unconstrained fn oracle_call() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_oracle_called_from_constrained_via_global_var() {
    let src = r#"
    global ORACLE: unconstrained fn() = oracle_call;

    fn main() {

        // safety:
        unsafe {
            ORACLE();
        }
    }

    #[oracle(oracle_call)]
    unconstrained fn oracle_call() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn does_not_error_if_oracle_called_from_constrained_via_other() {
    let src = r#"
      struct Foo {
          foo: unconstrained fn(),
      }

      fn main() {
          let foo_tuple = (foo, foo);
          let foo_array = [foo, foo];
          let foo_struct = Foo { foo };
          // safety:
          unsafe {
              (foo_struct.foo)();
              (foo_tuple.0)();
              foo_array[0]();
          }
      }

      #[oracle(foo)]
      unconstrained fn foo() {}
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_oracle_returns_reference() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call() -> &mut Field {}
                         ^^^^^^^^^^^ Oracle functions cannot return references
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_returns_reference_in_tuple() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call() -> (Field, &Field) {}
                         ^^^^^^^^^^^ Oracle functions cannot return references
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn errors_if_oracle_returns_reference_in_struct() {
    let src = r#"
    pub struct Foo {
        field: &Field,
    }

    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call() -> Foo {}
                         ^^^^^^^^^^^ Oracle functions cannot return references
    "#;
    //check_errors(src);
    check_errors_using_features(src, &[]);
}

#[test]
fn errors_if_oracle_has_mutable_reference_parameter() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call(x: &mut Field) {}
                         ^^^^^^^^^^^ Oracle functions cannot accept references as parameters
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_has_immutable_reference_parameter() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call(x: &Field) {}
                         ^^^^^^^^^^^ Oracle functions cannot accept references as parameters
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn errors_if_oracle_has_reference_parameter_in_tuple() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call(x: (Field, &Field)) {}
                         ^^^^^^^^^^^ Oracle functions cannot accept references as parameters
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn errors_if_oracle_has_reference_parameter_in_struct() {
    let src = r#"
    pub struct Foo {
        field: &Field,
    }

    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call(x: Foo) {}
                         ^^^^^^^^^^^ Oracle functions cannot accept references as parameters
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn errors_if_oracle_has_reference_parameter_behind_generics() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 10;
        pass_ref(&mut x);
        ^^^^^^^^ Reference `&mut Field` cannot be passed to an oracle function
    }

    #[oracle(pass_ref)]
    unconstrained fn pass_ref<T>(x: T) {}
    "#;
    check_monomorphization_error(src);
}

#[test]
fn errors_if_oracle_has_reference_parameter_nested_in_container_behind_generics() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 10;
        pass_ref((1, &mut x));
        ^^^^^^^^ Reference `(Field, &mut Field)` cannot be passed to an oracle function
    }

    #[oracle(pass_ref)]
    unconstrained fn pass_ref<T>(x: (Field, T)) {}
    "#;
    check_monomorphization_error(src);
}

// The `print` oracle is exempt from the reference-parameter rule: the compiler loads any
// reference in the printed value before the call, so the oracle is monomorphized with a
// reference-free type. References at the top level and nested in tuples are supported;
// references inside data types, arrays, vectors, etc. are not yet supported.

#[test]
fn prints_mutable_reference() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 1;
        println(&mut x);
    }
    "#;
    get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect("printing a mutable reference should compile");
}

#[test]
fn prints_reference_nested_in_tuple() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 1;
        println((2, &mut x));
    }
    "#;
    get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect("printing a reference inside a tuple should compile");
}

#[test]
fn prints_reference_to_struct() {
    let src = r#"
    struct Foo {
        a: Field,
        b: Field,
    }

    unconstrained fn main() {
        let mut foo = Foo { a: 1, b: 2 };
        println(&mut foo);
    }
    "#;
    get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect("printing a reference to a struct should compile");
}

#[test]
fn prints_reference_nested_in_array() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 1;
        let mut y = 2;
        println([&mut x, &mut y]);
    }
    "#;
    get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect("printing an array of references should compile");
}

#[test]
fn prints_reference_nested_in_struct() {
    let src = r#"
    struct Foo {
        a: &mut Field,
        b: Field,
    }

    unconstrained fn main() {
        let mut x = 1;
        println(Foo { a: &mut x, b: 2 });
    }
    "#;
    get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect("printing a struct with a reference field should compile");
}

#[test]
fn errors_when_printing_reference_nested_in_vector() {
    let src = r#"
    unconstrained fn main() {
        let mut x = 1;
        let mut y = 2;
        println(@[&mut x, &mut y]);
    }
    "#;
    let error = get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect_err("printing a vector of references is not supported");
    assert!(
        matches!(error, MonomorphizationError::ReferenceParameterToOracle { .. }),
        "unexpected error: {error:?}"
    );
}

#[test]
fn errors_when_printing_reference_nested_in_enum() {
    let src = r#"
    enum Foo {
        Bar(&mut Field),
    }

    unconstrained fn main() {
        let mut x = 1;
        println(Foo::Bar(&mut x));
    }
    "#;
    let error = get_monomorphized_with_stdlib(src, &[stdlib_src::PRINT])
        .expect_err("printing an enum with a reference field is not supported");
    assert!(
        matches!(error, MonomorphizationError::ReferenceParameterToOracle { .. }),
        "unexpected error: {error:?}"
    );
}

#[test]
fn errors_if_oracle_returns_vector_with_nested_array() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call() -> [[(u8, u8); 3]] {}
                         ^^^^^^^^^^^ Oracle functions cannot return vectors containing nested arrays
                         ~~~~~~~~~~~ Vectors with nested arrays are not yet supported for foreign call returns
    "#;
    check_errors(src);
}

#[test]
fn vector_with_nested_array_behind_generics_returned_from_oracle() {
    let src = r#"
    unconstrained fn main() {
        let _result: [[(u8, u8); 3]] = get_array();
                                       ^^^^^^^^^ Vector with nested array `[[(u8, u8); 3]]` cannot be returned from an oracle function
    }

    #[oracle(get_array)]
    unconstrained fn get_array<T>() -> [T] {}
    "#;
    check_monomorphization_error(src);
}

#[test]
fn errors_if_oracle_clashes_with_stdlib() {
    let src = r#"
    #[oracle(create_mock)]
    ^^^^^^^^^^^^^^^^^^^^^^ The name of an `#[oracle]` function clashes with one defined in the Noir standard library
    ~~~~~~~~~~~~~~~~~~~~~~ Naming an `#[oracle]` function the same as one in the Noir standard library could lead to unexpected behavior
    unconstrained fn create_mock_oracle() {}

    unconstrained fn main() {
        create_mock_oracle();
    }
    "#;
    check_errors(src);
}

#[test]
fn allows_pure_attribute_on_oracle() {
    let src = r#"
    #[pure]
    #[oracle(foo)]
    unconstrained fn foo(x: Field) -> Field {}

    unconstrained fn main(x: Field) {
        let _ = foo(x);
    }
    "#;
    assert_no_errors(src);
}

#[test]
fn errors_if_pure_attribute_on_non_oracle() {
    let src = r#"
    #[pure]
    ^^^^^^^ The `#[pure]` attribute is only valid on `unconstrained` functions marked `#[oracle(...)]`
    pub unconstrained fn helper(x: Field) -> Field {
                         ~~~~~~ `#[pure]` requires `unconstrained` and `#[oracle(...)]`
        x
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_pure_attribute_on_constrained_function() {
    // `#[pure]` requires `#[oracle(...)]`. The frontend reports the missing-oracle
    // error rather than a separate "constrained" error, which is enough to reject the case.
    let src = r#"
    #[pure]
    ^^^^^^^ The `#[pure]` attribute is only valid on `unconstrained` functions marked `#[oracle(...)]`
    pub fn helper(x: Field) -> Field {
           ~~~~~~ `#[pure]` requires `unconstrained` and `#[oracle(...)]`
        x
    }
    "#;
    check_errors(src);
}

#[test]
fn oracle_returning_recursive_struct() {
    let src = r#"
    pub struct Foo {
        bar: Bar,
    }

    pub struct Bar {
               ^^^ Dependency cycle found
               ~~~ 'Bar' recursively depends on itself: Bar -> Foo -> Bar
        foo: Foo,
    }

    #[oracle(foo)]
    pub unconstrained fn foo() -> Foo {}
    "#;
    check_errors(src);
}

#[test]
fn oracle_returning_multiple_vectors() {
    let src = r#"
    pub struct Foo {
        xs: [u32],
    }
    pub struct Bar {
        a: Foo,
        b: Foo,
    }

    #[oracle(bar)]
    unconstrained fn bar() -> Bar {}
                     ^^^ Oracle functions cannot return multiple vectors

    unconstrained fn main() {
        let _bar = bar();
    }
    "#;
    check_errors(src);
}
