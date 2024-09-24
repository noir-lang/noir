use crate::hir::def_collector::dc_crate::CompilationError;

use super::get_program_errors;

#[test]
fn comptime_type_in_runtime_code() {
    let source = "pub fn foo(_f: FunctionDefinition) {}";
    let errors = get_program_errors(source);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::ResolverError(ResolverError::ComptimeTypeInRuntimeCode { .. })
    ));
}

#[test]
fn macro_result_type_mismatch() {
    let src = r#"
        fn main() {
            comptime {
                let x = unquote!(quote { "test" });
                let _: Field = x;
            }
        }

        comptime fn unquote(q: Quoted) -> Quoted {
            q
        }
    "#;

    let errors = get_program_errors(src);
    assert_eq!(errors.len(), 1);
    assert!(matches!(
        errors[0].0,
        CompilationError::TypeError(TypeCheckError::TypeMismatch { .. })
    ));
}

#[test]
fn unquoted_integer_as_integer_token() {
    let src = r#"
    trait Serialize<let N: u32> {
        fn serialize() {}
    }

    #[attr]
    pub fn foobar() {}

    comptime fn attr(_f: FunctionDefinition) -> Quoted {
        let serialized_len = 1;
        // We are testing that when we unquote $serialized_len, it's unquoted
        // as the token `1` and not as something else that later won't be parsed correctly
        // in the context of a generic argument.
        quote {
            impl Serialize<$serialized_len> for Field {
                fn serialize() { }
            }
        }
    }

    fn main() {}
    "#;

    assert_no_errors(src);
}
