use acvm::{
    FieldElement,
    acir::{
        circuit::Program,
        native_types::{Witness, WitnessMap},
    },
};
use noir_ssa_executor::runner::execute_single;

#[derive(Debug)]
pub enum CompareResults {
    Agree(FieldElement),
    Disagree(FieldElement, FieldElement),
    BothFailed(String, String),
    AcirFailed(String, FieldElement),
    BrilligFailed(String, FieldElement),
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
    assert_eq!(return_witnesses_acir.0.len(), 1, "Multiple return value witnesses encountered");
    assert_eq!(return_witnesses_brillig.0.len(), 1, "Multiple return value witnesses encountered");
    let return_witness_acir: &Witness = return_witnesses_acir.0.first().unwrap();
    let return_witness_brillig: &Witness = return_witnesses_brillig.0.first().unwrap();

    // we found bug in case of
    // 1) acir_result != brillig_result
    // 2) acir execution failed, brillig execution succeeded
    // 3) acir execution succeeded, brillig execution failed
    // it has depth 2, because nargo can panic or return NargoError
    match (acir_result, brillig_result) {
        (Ok(acir_result), Ok(brillig_result)) => {
            let acir_result = acir_result[return_witness_acir];
            let brillig_result = brillig_result[return_witness_brillig];
            if acir_result == brillig_result {
                CompareResults::Agree(acir_result)
            } else {
                CompareResults::Disagree(acir_result, brillig_result)
            }
        }
        (Err(acir_error), Ok(brillig_result)) => {
            let brillig_result = brillig_result[return_witness_brillig];
            CompareResults::AcirFailed(acir_error.to_string(), brillig_result)
        }
        (Ok(acir_result), Err(brillig_error)) => {
            let acir_result = acir_result[return_witness_acir];
            CompareResults::BrilligFailed(brillig_error.to_string(), acir_result)
        }
        (Err(acir_error), Err(brillig_error)) => {
            CompareResults::BothFailed(acir_error.to_string(), brillig_error.to_string())
        }
    }
}
