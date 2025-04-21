
    // TODO(https://github.com/noir-lang/noir/issues/7372): unexpected panic
    // The application panicked (crashed).
    // Message:  attempt to subtract with overflow
    // Location: compiler/noirc_evaluator/src/acir/generated_acir.rs:660

    pub unconstrained fn foo(input: Field) -> Field {
        (input == 0) as Field
    }
    
    pub fn bar(input: Field) {
        // Safety: test
        let output = unsafe { foo(input) };
    
        assert(output != 0, "");
    }
    
    mod test {
        use crate::bar;
    
        // this panics:
        #[test]
        fn test_local() {
            bar(0);
        }
    
        global x_global: Field = 0;
    
        // this panics:
        #[test]
        fn test_global() {
            bar(x_global);
        }
    
    }
    
    // it works when calling or inlining `bar` in `main` (with `input = 0`)
    fn main(input: Field) {
        bar(input)
    }
    