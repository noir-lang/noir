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
use super::options::{FuzzerMode, FuzzerOptions};
use super::program_context::{FuzzerProgramContext, program_context_by_mode};
use super::{NUMBER_OF_PREDEFINED_VARIABLES, NUMBER_OF_VARIABLES_INITIAL};
use acvm::FieldElement;
use acvm::acir::native_types::{WitnessMap, WitnessStack};
use libfuzzer_sys::{arbitrary, arbitrary::Arbitrary};
use noir_ssa_executor::runner::execute_single;
use noir_ssa_fuzzer::runner::{CompareResults, run_and_compare};
use noir_ssa_fuzzer::typed_value::ValueType;
use noirc_driver::CompiledProgram;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
    pub(crate) contexts: Vec<FuzzerProgramContext>,
}

#[derive(Clone, Debug)]
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
        let mut contexts = vec![];
        for mode in &options.modes {
            contexts.push(program_context_by_mode(
                mode.clone(),
                instruction_blocks.clone(),
                values.clone(),
                options.clone(),
            ));
        }
        Self { contexts }
    }

    pub(crate) fn process_function(&mut self, function_data: FunctionData, types: Vec<ValueType>) {
        for context in &mut self.contexts {
            context.process_function(function_data.clone(), types.clone());
        }
    }

    /// Finalizes the function for contexts, executes and compares the results
    pub(crate) fn finalize_and_run(
        self,
        initial_witness: WitnessMap<FieldElement>,
    ) -> Option<FuzzerOutput> {
        let mut execution_results: HashMap<FuzzerMode, Option<FuzzerOutput>> = HashMap::new();
        for mut context in self.contexts {
            context.finalize_program();
            execution_results.insert(
                context.get_mode(),
                Self::execute_and_compare(context, initial_witness.clone()),
            );
        }
        let results_set = execution_results
            .values()
            .map(|result| -> Option<FieldElement> { result.as_ref().map(|r| r.get_return_value()) })
            .collect::<HashSet<_>>();

        if results_set.len() != 1 {
            let mut panic_string = String::new();
            for (mode, result) in execution_results {
                if let Some(result) = result {
                    panic_string
                        .push_str(&format!("Mode {mode:?}: {:?}\n", result.get_return_value()));
                } else {
                    panic_string.push_str(&format!("Mode {mode:?} failed\n"));
                }
            }
            panic!("Fuzzer modes returned different results:\n{panic_string}");
        }
        execution_results.values().next().unwrap().clone()
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
