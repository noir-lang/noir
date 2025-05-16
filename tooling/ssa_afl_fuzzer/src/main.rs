#[macro_use]
extern crate afl;
extern crate noirc_evaluator;

use std::str::FromStr;

use noirc_evaluator::ssa::interpreter::value::Value;
use noirc_evaluator::ssa::ssa_gen::Ssa;

fn main() {
    fuzz!(|data: &[u8]| {
        if let Ok(src) = std::str::from_utf8(data) {
            if let Ok(ssa) = Ssa::from_str(src) {
                // re-parsing is easier than cloning
                let mut other_ssa = Ssa::from_str(src).expect("expected repeated SSA parsing to succeed");

                let src_header_byte = if let Some(src_with_header) = src.strip_prefix("// ") {
                    if let Some(src_header_char) = src_with_header.chars().next() {
                        u32::from(src_header_char)
                    } else {
                        // enable no passes (empty file)
                        0
                    }
                } else {
                    // enable all passes
                    1 + 2 + 4
                };

                // if first bit is set, 'remove_unreachable_functions'
                if src_header_byte % 2 == 1 {
                    other_ssa = other_ssa.remove_unreachable_functions();
                }
                // if second bit is set, 'inline_functions_with_at_most_one_instruction'
                if (src_header_byte / 2) % 2 == 1 {
                    other_ssa = other_ssa.inline_functions_with_at_most_one_instruction();
                }
                // if third bit is set, 'defunctionalize'
                if (src_header_byte / 4) % 2 == 1 {
                    other_ssa = other_ssa.defunctionalize();
                }

                let main_fn = ssa.main();
                let parameter_ids = main_fn.parameters();
                let parameter_types = main_fn.signature().params;
                assert_eq!(parameter_ids.len(), parameter_types.len());

                let parameters: Vec<_> = parameter_types.iter().zip(parameter_ids).map(|(param_typ, param_id)| { 
                    Value::uninitialized(param_typ, *param_id)
                }).collect();
                let other_parameters: Vec<_> = parameter_types.iter().zip(parameter_ids).map(|(param_typ, param_id)| { 
                    Value::uninitialized(param_typ, *param_id)
                }).collect();

                let result = ssa.interpret(parameters.clone());
                let other_result = other_ssa.interpret(other_parameters);

                // ensure both pass with the same result or both fail with the same error variant
                match (result, other_result) {
                    (Ok(result_ok), Ok(other_result_ok)) => {
                        assert_eq!(result_ok, other_result_ok);
                    }
                    // check that the errors have the same discriminant (i.e. same enum variant)
                    (Err(err), Err(other_err)) => {
                        let disciminant = std::mem::discriminant(&err);
                        let other_disciminant = std::mem::discriminant(&other_err);
                        if disciminant != other_disciminant {
                            dbg!(&err, &other_err);
                            panic!("interpreter produced different errors")
                        }
                    }
                    (result, other_result) => {
                        panic!("results did not match when applying 'remove_unreachable_functions'!\n{:?}\n-----------\n{:?}", result, other_result)
                    }
                }
            }
        }
    });
}
