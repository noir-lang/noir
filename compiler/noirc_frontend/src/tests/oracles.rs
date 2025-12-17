use crate::tests::{assert_no_errors, check_errors};

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
                         ^^^^^^^^^^^ Oracle functions cannot return multiple lists
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
