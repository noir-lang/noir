use acvm::{
    FieldElement,
    acir::{
        circuit::Program,
        native_types::{Witness, WitnessMap, WitnessStack},
    },
};
use noir_ssa_executor::{runner::SsaExecutionError, runner::execute_single};
use std::collections::BTreeSet;

#[derive(Debug)]
pub enum CompareResults {
    Agree(WitnessStack<FieldElement>, WitnessStack<FieldElement>),
    Disagree(WitnessStack<FieldElement>, WitnessStack<FieldElement>),
    BothFailed(String, String),
    AcirFailed(String, WitnessStack<FieldElement>),
    BrilligFailed(String, WitnessStack<FieldElement>),
}

pub fn execute(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<WitnessStack<FieldElement>, SsaExecutionError> {
    execute_single(program, initial_witness)
}

/// High level function to execute the given ACIR and Brillig programs with the given initial witness
/// It returns a tuple with a boolean indicating if the programs succeeded,
/// and the results of the ACIR and Brillig programs
pub fn run_and_compare(
    acir_program: &Program<FieldElement>,
    brillig_program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
) -> CompareResults {
    let acir_result = execute_single(acir_program, initial_witness.clone());
    let brillig_result = execute_single(brillig_program, initial_witness);

    let return_witnesses_acir = &acir_program.functions[0].return_values;
    let return_witnesses_brillig = &brillig_program.functions[0].return_values;
    let return_witness_acir: BTreeSet<Witness> = return_witnesses_acir.0.clone();
    let return_witness_brillig: BTreeSet<Witness> = return_witnesses_brillig.0.clone();

    // we found bug in case of
    // 1) acir_result != brillig_result
    // 2) acir execution failed, brillig execution succeeded
    // 3) acir execution succeeded, brillig execution failed
    match (acir_result, brillig_result) {
        (Ok(acir_witness), Ok(brillig_witness)) => {
            // we assume that if execution for both modes succeeds both programs returned something
            let acir_witness_map = acir_witness.peek().unwrap().witness.clone();
            let brillig_witness_map = brillig_witness.peek().unwrap().witness.clone();
            let acir_results = return_witness_acir.iter().map(|w| acir_witness_map[w]);
            let brillig_results = return_witness_brillig.iter().map(|w| brillig_witness_map[w]);
            let results_equal = acir_results.eq(brillig_results);
            if results_equal {
                CompareResults::Agree(acir_witness, brillig_witness)
            } else {
                CompareResults::Disagree(acir_witness, brillig_witness)
            }
        }
        (Err(acir_error), Ok(brillig_witness)) => {
            CompareResults::AcirFailed(acir_error.to_string(), brillig_witness)
        }
        (Ok(acir_witness), Err(brillig_error)) => {
            CompareResults::BrilligFailed(brillig_error.to_string(), acir_witness)
        }
        (Err(acir_error), Err(brillig_error)) => {
            CompareResults::BothFailed(acir_error.to_string(), brillig_error.to_string())
        }
    }
}
