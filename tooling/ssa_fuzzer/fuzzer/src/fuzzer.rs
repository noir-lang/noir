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

use crate::base_context::FuzzerContext;
use acvm::FieldElement;
use acvm::acir::native_types::{Witness, WitnessMap};
use noir_ssa_executor::runner::execute_single;
use noir_ssa_fuzzer::runner::{CompareResults, run_and_compare};
use noir_ssa_fuzzer::typed_value::ValueType;

pub(crate) struct Fuzzer {
    pub(crate) context_non_constant: FuzzerContext,
    pub(crate) context_constant: FuzzerContext,
}

impl Fuzzer {
    pub(crate) fn new(
        types: Vec<ValueType>,
        initial_witness_vector: Vec<impl Into<FieldElement>>,
    ) -> Self {
        Self {
            context_non_constant: FuzzerContext::new(types.clone()),
            context_constant: FuzzerContext::new_constant_context(initial_witness_vector, types),
        }
    }

    /// Finalizes the function for both contexts, executes and compares the results
    pub(crate) fn run(
        mut self,
        initial_witness: WitnessMap<FieldElement>,
        constant_checking_enabled: bool,
    ) {
        self.context_non_constant.finalize_function();
        self.context_constant.finalize_function();

        let (acir_return_witness, brillig_return_witness) =
            self.context_non_constant.get_return_witnesses();
        let non_constant_result = Self::execute_and_compare(
            self.context_non_constant,
            initial_witness.clone(),
            acir_return_witness,
            brillig_return_witness,
        );
        log::debug!("Non-constant result: {:?}", non_constant_result);

        let (acir_return_witness, brillig_return_witness) =
            self.context_constant.get_return_witnesses();

        if !constant_checking_enabled {
            return;
        }

        let constant_result = Self::execute_and_compare(
            self.context_constant,
            WitnessMap::new(),
            acir_return_witness,
            brillig_return_witness,
        );
        log::debug!("Constant result: {:?}", constant_result);

        if non_constant_result != constant_result {
            match (non_constant_result, constant_result) {
                (Some(non_constant_result), Some(constant_result)) => {
                    // #7947
                    if constant_result == FieldElement::from(0_u32) {
                        return;
                    }
                    panic!(
                        "Constant and non-constant results are different for the same program: {:?} != {:?}",
                        non_constant_result, constant_result
                    );
                }
                (Some(non_constant_result), None) => {
                    panic!(
                        "Non-constant result is {:?}, but constant result is None",
                        non_constant_result
                    );
                }
                (None, Some(constant_result)) => {
                    // #7947, non-constant failed due to overflow, but constant succeeded
                    println!(
                        "Constant result is {:?}, but non-constant program failed to execute",
                        constant_result
                    );
                }
                // both are None
                _ => {}
            }
        }
    }

    fn execute_and_compare(
        context: FuzzerContext,
        initial_witness: WitnessMap<FieldElement>,
        acir_return_witness: Witness,
        brillig_return_witness: Witness,
    ) -> Option<FieldElement> {
        let (acir_program, brillig_program) = context.get_programs();
        let (acir_program, brillig_program) = match (acir_program, brillig_program) {
            (Ok(acir), Ok(brillig)) => (acir, brillig),
            (Err(acir_error), Err(brillig_error)) => {
                log::debug!("ACIR compilation error: {:?}", acir_error);
                log::debug!("Brillig compilation error: {:?}", brillig_error);
                return None;
            }
            (Ok(acir), Err(brillig_error)) => {
                let acir_result = execute_single(&acir.program, initial_witness);
                match acir_result {
                    Ok(acir_result) => {
                        panic!(
                            "ACIR compiled and successfully executed, 
                            but brillig compilation failed. Execution result of 
                            acir only {:?}. Brillig compilation failed with: {:?}",
                            acir_result[&acir_return_witness], brillig_error
                        );
                    }
                    Err(acir_error) => {
                        log::debug!("ACIR execution error: {:?}", acir_error);
                        log::debug!("Brillig compilation error: {:?}", brillig_error);
                        return None;
                    }
                }
            }
            (Err(acir_error), Ok(brillig)) => {
                let brillig_result = execute_single(&brillig.program, initial_witness);
                match brillig_result {
                    Ok(brillig_result) => {
                        panic!(
                            "Brillig compiled and successfully executed, but ACIR compilation failed. Execution result of brillig only {:?}. ACIR compilation failed with: {:?}",
                            brillig_result[&brillig_return_witness], acir_error
                        );
                    }
                    Err(brillig_error) => {
                        log::debug!("Brillig execution error: {:?}", brillig_error);
                        log::debug!("ACIR compilation error: {:?}", acir_error);
                        return None;
                    }
                }
            }
        };
        let comparison_result = run_and_compare(
            &acir_program.program,
            &brillig_program.program,
            initial_witness,
            acir_return_witness,
            brillig_return_witness,
        );
        log::debug!("Comparison result: {:?}", comparison_result);
        log::debug!("ACIR program: {:?}", acir_program);
        log::debug!("Brillig program: {:?}", brillig_program);
        match comparison_result {
            CompareResults::Agree(result) => Some(result),
            CompareResults::Disagree(acir_return_value, brillig_return_value) => {
                panic!(
                    "ACIR and Brillig programs returned different results: ACIR returned {:?}, Brillig returned {:?}",
                    acir_return_value, brillig_return_value
                );
            }
            CompareResults::AcirFailed(acir_error, brillig_return_value) => {
                panic!(
                    "ACIR execution failed with error: {:?}, Brillig returned {:?}",
                    acir_error, brillig_return_value
                );
            }
            CompareResults::BrilligFailed(brillig_error, acir_return_value) => {
                panic!(
                    "Brillig execution failed with error: {:?}, ACIR returned {:?}",
                    brillig_error, acir_return_value
                );
            }
            CompareResults::BothFailed(acir_error, brillig_error) => {
                log::debug!("ACIR execution error: {:?}", acir_error);
                log::debug!("Brillig execution error: {:?}", brillig_error);
                None
            }
        }
    }
}
