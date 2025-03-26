use crate::base_context::FuzzerContext;
use acvm::FieldElement;
use noir_ssa_fuzzer::runner::{execute_single, run_and_compare, CompareResults};
use acvm::acir::native_types::{WitnessMap, Witness};
use noirc_evaluator::ssa::ir::types::Type;

pub(crate) struct Fuzzer {
    pub(crate) context_non_constant: FuzzerContext,
    pub(crate) context_constant: FuzzerContext,
}

impl Fuzzer {
    pub(crate) fn new(type_: Type, values: Vec<impl Into<FieldElement>>) -> Self {
        Self {
            context_non_constant: FuzzerContext::new(type_.clone()),
            context_constant: FuzzerContext::new_constant(values, type_),
        }
    }

    /// Finalizes the function for both contexts, executes and compares the results
    pub(crate) fn run(mut self, initial_witness: WitnessMap<FieldElement>) {
        self.context_non_constant.finalize_function();
        self.context_constant.finalize_function();

        let (acir_return_witness, brillig_return_witness) = self.context_non_constant.get_return_witnesses();
        Self::execute_and_compare(self.context_non_constant, initial_witness.clone(), acir_return_witness, brillig_return_witness);
        
        let (acir_return_witness, brillig_return_witness) = self.context_constant.get_return_witnesses();
        Self::execute_and_compare(self.context_constant, initial_witness, acir_return_witness, brillig_return_witness);
    }

    fn execute_and_compare(context: FuzzerContext, initial_witness: WitnessMap<FieldElement>, acir_return_witness: Witness, brillig_return_witness: Witness) {
        let (acir_program, brillig_program) = context.get_programs();
        let (acir_program, brillig_program) = match (acir_program, brillig_program) {
            (Ok(acir), Ok(brillig)) => (acir, brillig),
            (Err(_), Err(_)) => {
                return;
            }
            (Ok(acir), Err(e)) => {
                let acir_result = execute_single(&acir.program, initial_witness, acir_return_witness);
                match acir_result {
                    Ok(acir_result) => {
                        panic!(
                            "ACIR compiled and successfully executed, 
                            but brillig compilation failed. Execution result of 
                            acir only {:?}. Brillig compilation failed with: {:?}",
                            acir_result, e
                        );
                    }
                    Err(_) => {
                        return;
                    }
                }
            }
            (Err(e), Ok(brillig)) => {
                let brillig_result = execute_single(&brillig.program, initial_witness, brillig_return_witness);
                match brillig_result {
                    Ok(brillig_result) => {
                        panic!("Brillig compiled and successfully executed, but ACIR compilation failed. Execution result of brillig only {:?}. ACIR compilation failed with: {:?}", brillig_result, e);
                    }
                    Err(_) => {
                        return;
                    }
                }
            }
        };
        let comparison_result = run_and_compare(&acir_program.program, &brillig_program.program, initial_witness, acir_return_witness, brillig_return_witness);
        match comparison_result {
            CompareResults::Agree(..) | CompareResults::BothFailed(..) => {
                return;
            }
            CompareResults::Disagree(acir_return_value, brillig_return_value) => {
                panic!("ACIR and Brillig programs returned different results: ACIR returned {:?}, Brillig returned {:?}", acir_return_value, brillig_return_value);
            }
            CompareResults::AcirFailed(acir_error, brillig_return_value) => {
                panic!("ACIR execution failed with error: {:?}, Brillig returned {:?}", acir_error, brillig_return_value);
            }
            CompareResults::BrilligFailed(brillig_error, acir_return_value) => {
                panic!("Brillig execution failed with error: {:?}, ACIR returned {:?}", brillig_error, acir_return_value);
            }
        }
    }
}
