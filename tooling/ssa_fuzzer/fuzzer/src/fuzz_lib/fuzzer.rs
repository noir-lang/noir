//! This module implements a fuzzer for testing and comparing ACIR and Brillig SSA implementations.
//! It generates random sequences of arithmetic and logical operations and ensures both implementations
//! produce identical results. Also it runs the fuzzer with constant values and checks if the results are the same.
//!
//! Main fuzz steps:
//!    1. Generate random witness
//!    2. Generate random sequence of instructions
//!    3. Insert instructions into ACIR and Brillig builders
//!    4. Get programs, and compile them
//!    5. Run and compare
//!
//! A bug is detected in two cases:
//!    - If programs return different results
//!    - If one program fails to compile but the other executes successfully

use super::function_context::{FunctionData, WitnessValue};
use super::instruction::InstructionBlock;
use super::options::{FunctionContextOptions, FuzzerOptions};
use super::program_context::FuzzerProgramContext;
use super::{NUMBER_OF_PREDEFINED_VARIABLES, NUMBER_OF_VARIABLES_INITIAL};
use acvm::FieldElement;
use acvm::acir::native_types::{WitnessMap, WitnessStack};
use libfuzzer_sys::{arbitrary, arbitrary::Arbitrary};
use noir_ssa_executor::runner::execute_single;
use noir_ssa_fuzzer::runner::{CompareResults, run_and_compare};
use noir_ssa_fuzzer::typed_value::ValueType;
use noirc_driver::CompiledProgram;
use serde::{Deserialize, Serialize};

#[derive(Arbitrary, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FuzzerData {
    pub(crate) functions: Vec<FunctionData>,
    pub(crate) initial_witness:
        [WitnessValue; (NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES) as usize],
    pub(crate) instruction_blocks: Vec<InstructionBlock>,
}

impl Default for FuzzerData {
    fn default() -> Self {
        FuzzerData {
            functions: vec![FunctionData::default()],
            initial_witness: [WitnessValue::default();
                (NUMBER_OF_VARIABLES_INITIAL - NUMBER_OF_PREDEFINED_VARIABLES) as usize],
            instruction_blocks: vec![],
        }
    }
}

pub(crate) struct Fuzzer {
    pub(crate) context_non_constant: Option<FuzzerProgramContext>,
    pub(crate) context_non_constant_with_idempotent_morphing: Option<FuzzerProgramContext>,
    pub(crate) context_constant: Option<FuzzerProgramContext>,
}

pub(crate) struct FuzzerOutput {
    pub(crate) witness_stack: WitnessStack<FieldElement>,
    pub(crate) program: CompiledProgram,
}

impl FuzzerOutput {
    pub(crate) fn get_return_value(&self) -> FieldElement {
        let return_witness = self.program.program.functions[0].return_values.0.first().unwrap();
        self.witness_stack.peek().unwrap().witness[return_witness]
    }
}

impl Fuzzer {
    pub(crate) fn new(
        instruction_blocks: Vec<InstructionBlock>,
        values: Vec<FieldElement>,
        options: FuzzerOptions,
    ) -> Self {
        let context_constant = if options.constant_execution_enabled {
            Some(FuzzerProgramContext::new_constant_context(
                FunctionContextOptions {
                    idempotent_morphing_enabled: false,
                    ..FunctionContextOptions::from(&options)
                },
                instruction_blocks.clone(),
                values.clone(),
            ))
        } else {
            None
        };
        let context_non_constant = Some(FuzzerProgramContext::new(
            FunctionContextOptions {
                idempotent_morphing_enabled: false,
                ..FunctionContextOptions::from(&options)
            },
            instruction_blocks.clone(),
            values.clone(),
        ));
        let context_non_constant_with_idempotent_morphing =
            if options.constrain_idempotent_morphing_enabled {
                Some(FuzzerProgramContext::new(
                    FunctionContextOptions {
                        idempotent_morphing_enabled: true,
                        ..FunctionContextOptions::from(&options)
                    },
                    instruction_blocks.clone(),
                    values.clone(),
                ))
            } else {
                None
            };
        Self {
            context_non_constant,
            context_non_constant_with_idempotent_morphing,
            context_constant,
        }
    }

    pub(crate) fn process_function(&mut self, function_data: FunctionData, types: Vec<ValueType>) {
        if let Some(context) = &mut self.context_non_constant {
            context.process_function(function_data.clone(), types.clone());
        }
        if let Some(context) = &mut self.context_non_constant_with_idempotent_morphing {
            context.process_function(function_data.clone(), types.clone());
        }
        if let Some(context) = &mut self.context_constant {
            context.process_function(function_data, types);
        }
    }

    /// Finalizes the function for both contexts, executes and compares the results
    pub(crate) fn finalize_and_run(
        mut self,
        initial_witness: WitnessMap<FieldElement>,
    ) -> Option<FuzzerOutput> {
        let mut non_constant_context = self.context_non_constant.take().unwrap();
        non_constant_context.finalize_program();
        let non_constant_result =
            Self::execute_and_compare(non_constant_context, initial_witness.clone());

        if let Some(context) = self.context_constant.take() {
            let mut constant_context = context;
            constant_context.finalize_program();
            let constant_result =
                Self::execute_and_compare(constant_context, initial_witness.clone());
            if non_constant_result.is_some() {
                assert_eq!(
                    non_constant_result.as_ref().unwrap().get_return_value(),
                    constant_result?.get_return_value(),
                    "Non-constant and constant contexts should return the same result"
                );
            }
        }

        if let Some(context) = self.context_non_constant_with_idempotent_morphing.take() {
            let mut context_with_idempotent_morphing = context;
            context_with_idempotent_morphing.finalize_program();
            let result_with_constrains =
                Self::execute_and_compare(context_with_idempotent_morphing, initial_witness);
            if non_constant_result.is_some() {
                assert_eq!(
                    non_constant_result.as_ref().unwrap().get_return_value(),
                    result_with_constrains?.get_return_value(),
                    "Non-constant and idempotent morphing contexts should return the same result"
                );
            }
        }
        non_constant_result
    }

    fn execute_and_compare(
        context: FuzzerProgramContext,
        initial_witness: WitnessMap<FieldElement>,
    ) -> Option<FuzzerOutput> {
        let (acir_program, brillig_program) = context.get_programs();
        let (acir_program, brillig_program) = match (acir_program, brillig_program) {
            (Ok(acir), Ok(brillig)) => (acir, brillig),
            (Err(acir_error), Err(brillig_error)) => {
                log::debug!("ACIR compilation error: {acir_error:?}");
                log::debug!("Brillig compilation error: {brillig_error:?}");
                log::debug!("ACIR and Brillig compilation failed");
                return None;
            }
            (Ok(acir), Err(brillig_error)) => {
                let acir_result = execute_single(&acir.program, initial_witness);
                match acir_result {
                    Ok(acir_result) => {
                        let acir_return_witness =
                            acir.program.functions[0].return_values.0.first().unwrap();
                        panic!(
                            "ACIR compiled and successfully executed, \
                            but brillig compilation failed. Execution result of \
                            acir only {:?}. Brillig compilation failed with: {:?}",
                            acir_result.peek().unwrap().witness[acir_return_witness],
                            brillig_error
                        );
                    }
                    Err(acir_error) => {
                        log::debug!("ACIR execution error: {acir_error:?}");
                        log::debug!("Brillig compilation error: {brillig_error:?}");
                        return None;
                    }
                }
            }
            (Err(acir_error), Ok(brillig)) => {
                let brillig_result = execute_single(&brillig.program, initial_witness);
                match brillig_result {
                    Ok(brillig_result) => {
                        let brillig_return_witness =
                            brillig.program.functions[0].return_values.0.first().unwrap();
                        panic!(
                            "Brillig compiled and successfully executed, \
                            but ACIR compilation failed. Execution result of \
                            brillig only {:?}. ACIR compilation failed with: {:?}",
                            brillig_result.peek().unwrap().witness[brillig_return_witness],
                            acir_error
                        );
                    }
                    Err(brillig_error) => {
                        log::debug!("Brillig execution error: {brillig_error:?}");
                        log::debug!("ACIR compilation error: {acir_error:?}");
                        return None;
                    }
                }
            }
        };
        let comparison_result =
            run_and_compare(&acir_program.program, &brillig_program.program, initial_witness);
        log::debug!("Comparison result: {comparison_result:?}");
        match comparison_result {
            CompareResults::Agree(result) => {
                Some(FuzzerOutput { witness_stack: result, program: acir_program })
            }
            CompareResults::Disagree(acir_return_value, brillig_return_value) => {
                panic!(
                    "ACIR and Brillig programs returned different results: \
                    ACIR returned {acir_return_value:?}, Brillig returned {brillig_return_value:?}"
                );
            }
            CompareResults::AcirFailed(acir_error, brillig_return_value) => {
                panic!(
                    "ACIR execution failed with error: {acir_error:?}, Brillig returned {brillig_return_value:?}"
                );
            }
            CompareResults::BrilligFailed(brillig_error, acir_return_value) => {
                panic!(
                    "Brillig execution failed with error: {brillig_error:?}, ACIR returned {acir_return_value:?}"
                );
            }
            CompareResults::BothFailed(acir_error, brillig_error) => {
                log::debug!("ACIR execution error: {acir_error:?}");
                log::debug!("Brillig execution error: {brillig_error:?}");
                None
            }
        }
    }
}
