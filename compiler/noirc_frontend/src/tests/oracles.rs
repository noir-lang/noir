use crate::tests::check_errors;

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
                         ^^^^^^^^^^^ Oracle functions cannot return multiple slices
    "#;
    check_errors(src);
}
