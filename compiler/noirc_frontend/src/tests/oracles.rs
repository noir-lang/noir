use crate::tests::{
    assert_no_errors, check_errors, check_errors_using_features, check_errors_with_stdlib,
    check_monomorphization_error,
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
fn deny_oracle_attribute_on_comptime() {
    let src = r#"
        #[oracle(foo)]
        ^^^^^^^^^^^^^^ Usage of the `#[oracle]` function attribute is not allowed on comptime functions
        pub unconstrained comptime fn foo(x: Field, y: Field) {}
                                      ~~~ Oracle functions cannot be marked `comptime`

        fn main() {
            // safety: test
            unsafe {
                foo(1, 2);
            }
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
                                     ^ Oracle functions cannot accept references as parameters
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_has_immutable_reference_parameter() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call(x: &Field) {}
                                     ^ Oracle functions cannot accept references as parameters
    "#;
    check_errors_using_features(src, &[]);
}

#[test]
fn errors_if_oracle_has_reference_parameter_in_tuple() {
    let src = r#"
    #[oracle(oracle_call)]
    pub unconstrained fn oracle_call(x: (Field, &Field)) {}
                                     ^ Oracle functions cannot accept references as parameters
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
                                     ^ Oracle functions cannot accept references as parameters
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
fn multiple_primary_attributes_fail() {
    let src = r#"
    #[oracle(oracleName)]
    #[builtin(builtinName)]
    ^^^^^^^^^^^^^^^^^^^^^^^ Multiple primary attributes found. Only one function attribute is allowed per function
    fn main(x: Field) -> pub Field {}
    "#;
    check_errors(src);
}

#[test]
fn primary_attribute_on_struct_fails() {
    let src = r#"
    #[oracle(some_oracle)]
    ^^^^^^^^^^^^^^^^^^^^^^ A function attribute cannot be placed on a struct or enum
    pub struct SomeStruct {
        x: Field,
        y: Field,
    }

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn fuzz_attribute_on_function_without_parameters_fails() {
    let src = r#"
    #[fuzz]
    ^^^^^^^ The `#[fuzz]` attribute may only be used on functions with parameters
    fn fuzz_no_arguments() {}

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn test_only_fail_with_attribute_on_function_without_parameters_fails() {
    let src = r#"
    #[test(only_fail_with = "error")]
    ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ The `#[test(only_fail_with = "..")]` attribute may only be used on functions with parameters
    fn test() {}

    fn main() {}
    "#;
    check_errors(src);
}

#[test]
fn oracle_returning_vector_of_nested_array_in_tuple() {
    let src = r#"
    #[oracle(byte_array_vec_in_tuple)]
    pub unconstrained fn byte_array_vec_in_tuple() -> (u32, [[u8; 2]], bool) {}
                         ^^^^^^^^^^^^^^^^^^^^^^^ Oracle functions cannot return vectors containing nested arrays
                         ~~~~~~~~~~~~~~~~~~~~~~~ Vectors with nested arrays are not yet supported for foreign call returns
    "#;
    check_errors(src);
}

#[test]
fn oracle_returning_vector_of_str() {
    let src = r#"
    #[oracle(str_vec)]
    pub unconstrained fn str_vec() -> [str<2>] {}
                         ^^^^^^^ Oracle functions cannot return vectors containing nested arrays
                         ~~~~~~~ Vectors with nested arrays are not yet supported for foreign call returns
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_returns_multiple_vectors_via_wrapper_tuple() {
    let src = r#"
    #[oracle(void_to_vectors)]
    unconstrained fn void_to_vectors_oracle() -> ([Field], [Field]) {}
                     ^^^^^^^^^^^^^^^^^^^^^^ Oracle functions cannot return multiple vectors

    unconstrained fn main() {
        let _ = void_to_vectors_oracle();
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_clashes_with_stdlib_print() {
    let src = r#"
    #[oracle(print)]
    ^^^^^^^^^^^^^^^^ The name of an `#[oracle]` function clashes with one defined in the Noir standard library
    ~~~~~~~~~~~~~~~~ Naming an `#[oracle]` function the same as one in the Noir standard library could lead to unexpected behavior
    unconstrained fn foo() {}

    unconstrained fn main() {
        foo()
    }
    "#;
    check_errors(src);
}

#[test]
fn cannot_call_std_verify_proof_with_type_in_unconstrained_context() {
    let stdlib = r#"
    pub fn verify_proof_with_type<let N: u32, let M: u32, let K: u32>(
        _verification_key: [Field; N],
        _proof: [Field; M],
        _public_inputs: [Field; K],
        _key_hash: Field,
        _proof_type: u32,
    ) {}
    "#;
    let src = r#"
    unconstrained fn main() {
        let verification_key: [Field; 114] = [0; 114];
        let proof: [Field; 94] = [0; 94];
        let public_inputs: [Field; 1] = [0];
        let key_hash: Field = 0;
        let proof_type: u32 = 0;

        crate::verify_proof_with_type(verification_key, proof, public_inputs, key_hash, proof_type);
        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ Cannot call `std::verify_proof_with_type` in unconstrained context
    }
    "#;
    check_errors_with_stdlib(src, [stdlib]);
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

#[test]
fn errors_if_oracle_defined_in_trait_impl() {
    // Dispatching to a trait-impl oracle method generically (e.g. `T::fetch(x)`) used to
    // reach an `unreachable!()` in monomorphization. Reject the definition up front instead.
    let src = r#"
    pub trait Fetcher {
        unconstrained fn fetch(x: Field) -> Field;
    }

    pub struct Remote {}

    impl Fetcher for Remote {
        #[oracle(fetch_value)]
        ^^^^^^^^^^^^^^^^^^^^^^ Usage of the `#[oracle]` function attribute is only valid on free functions
        unconstrained fn fetch(x: Field) -> Field {}
                         ~~~~~ Oracle functions cannot be defined within a trait or impl block
    }
    "#;
    check_errors(src);
}

#[test]
fn errors_if_oracle_defined_in_regular_impl() {
    let src = r#"
    pub struct Remote {}

    impl Remote {
        #[oracle(fetch_value)]
        ^^^^^^^^^^^^^^^^^^^^^^ Usage of the `#[oracle]` function attribute is only valid on free functions
        pub unconstrained fn fetch(x: Field) -> Field {}
                             ~~~~~ Oracle functions cannot be defined within a trait or impl block
    }
    "#;
    check_errors(src);
}
