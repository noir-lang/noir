#[macro_use]
extern crate afl;
extern crate noirc_evaluator;

use std::str::FromStr;

use noirc_evaluator::ssa::ssa_gen::Ssa;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(src) = std::str::from_utf8(data) {
            if let Ok(ssa) = Ssa::from_str(src) {
                // re-parsing is easier than cloning
                let other_ssa = Ssa::from_str(src).expect("expected repeated SSA parsing to succeed");
                let other_ssa = other_ssa.remove_unreachable_functions();

                let result = ssa.interpret(Vec::new());
                let other_result = other_ssa.interpret(Vec::new());

                // ensure both pass with the same result or both fail with the same error variant
                match (result, other_result) {
                    (Ok(result_ok), Ok(other_result_ok)) => {
                        assert_eq!(result_ok, other_result_ok);
                    }
                    // check that the errors have the same discriminant (i.e. same enum variant)
                    (Err(err), Err(other_err)) => {
                        let disciminant = std::mem::discriminant(&err);
                        let other_disciminant = std::mem::discriminant(&other_err);
                        assert_eq!(disciminant, other_disciminant);
                    }
                    (result, other_result) => {
                        panic!("results did not match when applying 'remove_unreachable_functions'!\n{:?}\n-----------\n{:?}", result, other_result)
                    }
                }
            }
        }
    });
}
