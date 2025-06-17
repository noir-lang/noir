#![forbid(unsafe_code)]
#![warn(unreachable_pub)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]
#![allow(clippy::result_large_err)]

pub mod compiler;
pub mod runner;

use crate::compiler::compile_from_ssa;
use crate::runner::{SsaExecutionError, execute_single};
use acvm::FieldElement;
use acvm::acir::native_types::WitnessMap;
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::ssa_gen::Ssa;

pub fn execute_ssa(
    ssa: String,
    initial_witness: WitnessMap<FieldElement>,
    compile_options: CompileOptions,
) -> Result<WitnessMap<FieldElement>, SsaExecutionError> {
    let ssa = Ssa::from_str(&ssa);
    match ssa {
        Ok(ssa) => {
            let compiled_program = compile_from_ssa(ssa, &compile_options);
            match compiled_program {
                Ok(compiled_program) => execute_single(&compiled_program.program, initial_witness),
                Err(e) => Err(SsaExecutionError::SsaCompilationFailed(format!(
                    "SSA compilation failed: {:?}",
                    e
                ))),
            }
        }
        Err(e) => Err(SsaExecutionError::SsaParsingFailed(format!("SSA parsing failed: {:?}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::execute_ssa;
    use acvm::FieldElement;
    use acvm::acir::native_types::{Witness, WitnessMap};
    use noirc_driver::CompileOptions;

    #[test]
    fn test_ssa_execution_add() {
        let ssa = "
        acir(inline) predicate_pure fn main f0 {
            b0(v0: u8, v1: u8):
                v2 = add v0, v1
                return v2
            }";
        let mut witness_map = WitnessMap::new();
        witness_map.insert(Witness(0), FieldElement::from(1_u32));
        witness_map.insert(Witness(1), FieldElement::from(2_u32));
        let result = execute_ssa(ssa.to_string(), witness_map, CompileOptions::default());
        // 1 + 2 == 3
        assert_eq!(result.unwrap()[&Witness(2)], FieldElement::from(3_u32));
    }

    #[test]
    fn test_ssa_execution_mul() {
        let ssa = "
        acir(inline) predicate_pure fn main f0 {
            b0(v0: u8, v1: u8):
                v2 = mul v0, v1
                return v2
            }";
        let mut witness_map = WitnessMap::new();
        witness_map.insert(Witness(0), FieldElement::from(20_u32));
        witness_map.insert(Witness(1), FieldElement::from(10_u32));
        let result = execute_ssa(ssa.to_string(), witness_map, CompileOptions::default());
        // 20 * 10 == 200
        assert_eq!(result.unwrap()[&Witness(2)], FieldElement::from(200_u32));
    }

    #[test]
    fn test_invalid_ssa_execution() {
        let ssa = "
        acir(inline) predicate_pure fn main f0 {
            b0(v0: u8, v1: u8):
                v2 = mul v0, v1
                return v2
            }";
        let result = execute_ssa(ssa.to_string(), WitnessMap::new(), CompileOptions::default());
        assert!(result.is_err());
    }
}
