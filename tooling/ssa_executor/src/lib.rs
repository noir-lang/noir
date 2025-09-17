#![forbid(unsafe_code)]
#![cfg_attr(not(test), warn(unused_crate_dependencies, unused_extern_crates))]
#![allow(clippy::result_large_err)]

pub mod compiler;
pub mod runner;

use crate::compiler::compile_from_ssa;
use crate::runner::{SsaExecutionError, execute_single};
use acvm::FieldElement;
use acvm::acir::native_types::{WitnessMap, WitnessStack};
use noirc_driver::CompileOptions;
use noirc_evaluator::ssa::ssa_gen::{Ssa, validate_ssa};

pub fn execute_ssa(
    ssa: String,
    initial_witness: WitnessMap<FieldElement>,
    compile_options: CompileOptions,
) -> Result<WitnessStack<FieldElement>, SsaExecutionError> {
    let ssa = Ssa::from_str(&ssa);
    match ssa {
        Ok(ssa) => {
            validate_ssa(&ssa);

            let compiled_program = compile_from_ssa(ssa, &compile_options);
            match compiled_program {
                Ok(compiled_program) => execute_single(&compiled_program.program, initial_witness),
                Err(e) => Err(SsaExecutionError::SsaCompilationFailed(format!(
                    "SSA compilation failed: {e:?}"
                ))),
            }
        }
        Err(e) => Err(SsaExecutionError::SsaParsingFailed(format!("SSA parsing failed: {e:?}"))),
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
        assert_eq!(result.unwrap().peek().unwrap().witness[&Witness(2)], FieldElement::from(3_u32));
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
        assert_eq!(
            result.unwrap().peek().unwrap().witness[&Witness(2)],
            FieldElement::from(200_u32)
        );
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

    #[test]
    fn bound_constraint_with_offset_bug() {
        let ssa_without_runtime = "
            (inline) fn main f0 {
              b0(v0: i32, v1: u1, v2: u1, v3: u1, v4: u1, v5: u1, v6: u1):
                jmpif v6 then: b1, else: b2
              b1():
                v7 = cast v0 as u128
                jmp b3()
              b2():
                jmp b9()
              b3():
                v8 = div v7, v7
                jmp b4()
              b4():
                v9 = not v8
                jmp b5()
              b5():
                v10 = add v8, v9
                jmp b6()
              b6():
                v12 = div v9, v9
                jmp b7()
              b7():
                v13 = div v12, v10
                jmp b9()
              b8():
                v14 = cast v1 as Field
                return v14
              b9():
                jmp b8()
            }
        ";
        let acir_ssa = "acir".to_string() + ssa_without_runtime;
        let brillig_ssa = "brillig".to_string() + ssa_without_runtime;
        let mut witness_map = WitnessMap::new();
        witness_map.insert(Witness(0), FieldElement::from(1188688178_u32));
        for i in 1..6 {
            witness_map.insert(Witness(i), FieldElement::from(1_u32));
        }
        witness_map.insert(Witness(6), FieldElement::from(0_u32));
        let acir_result =
            execute_ssa(acir_ssa.to_string(), witness_map.clone(), CompileOptions::default());
        let brillig_result =
            execute_ssa(brillig_ssa.to_string(), witness_map, CompileOptions::default());
        match (acir_result, brillig_result) {
            (Err(acir), Ok(_brillig)) => panic!("Acir failed with: {acir}, brillig succeeded"),
            (Ok(_acir), Err(brillig)) => panic!("Acir succeeded, brillig failed: {brillig}"),
            _ => {}
        }
    }

    #[test]
    fn execute_brillig_stdlib_call_with_multiple_acir_calls() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u32, v1: u32, v2: u32):
            v5 = div v0, v1
            constrain v5 == v2
            v6 = call f1(v0, v1) -> u32
            v7 = call f1(v0, v1) -> u32
            v8 = call f2(v0, v1) -> u32
            v9 = div v1, v2
            constrain v9 == u32 1
            return
        }
        brillig(inline) fn foo f1 {
          b0(v0: u32, v1: u32):
            v2 = eq v0, v1
            constrain v2 == u1 0
            return v0
        }
        acir(fold) fn foo f2 {
          b0(v0: u32, v1: u32):
            v2 = eq v0, v1
            constrain v2 == u1 0
            return v0
        }
        ";
        let mut witness_map = WitnessMap::new();
        witness_map.insert(Witness(0), FieldElement::from(9_u32));
        witness_map.insert(Witness(1), FieldElement::from(3_u32));
        witness_map.insert(Witness(2), FieldElement::from(3_u32));
        let result = execute_ssa(src.to_string(), witness_map, CompileOptions::default());
        assert!(result.is_ok());
    }
}
