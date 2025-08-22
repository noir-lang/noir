use acvm::{
    FieldElement,
    acir::{
        circuit::Program,
        native_types::{Witness, WitnessMap, WitnessStack},
    },
};
use noir_ssa_executor::runner::execute_single;

#[derive(Debug)]
pub enum CompareResults {
    Agree(WitnessStack<FieldElement>),
    Disagree(WitnessStack<FieldElement>, WitnessStack<FieldElement>),
    BothFailed(String, String),
    AcirFailed(String, WitnessStack<FieldElement>),
    BrilligFailed(String, WitnessStack<FieldElement>),
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
    assert!(return_witnesses_acir.0.len() <= 1, "Multiple return value witnesses encountered");
    assert!(return_witnesses_brillig.0.len() <= 1, "Multiple return value witnesses encountered");
    let return_witness_acir: Option<&Witness> = return_witnesses_acir.0.first();
    let return_witness_brillig: Option<&Witness> = return_witnesses_brillig.0.first();

    // we found bug in case of
    // 1) acir_result != brillig_result
    // 2) acir execution failed, brillig execution succeeded
    // 3) acir execution succeeded, brillig execution failed
    match (acir_result, brillig_result) {
        (Ok(acir_witness), Ok(brillig_witness)) => {
            // we assume that if execution for both modes succeeds both programs returned something
            let acir_witness_map = acir_witness.peek().unwrap().witness.clone();
            let brillig_witness_map = brillig_witness.peek().unwrap().witness.clone();
            let acir_result = acir_witness_map[return_witness_acir.unwrap()];
            let brillig_result = brillig_witness_map[return_witness_brillig.unwrap()];
            if acir_result == brillig_result {
                CompareResults::Agree(acir_witness)
            } else {
                CompareResults::Disagree(acir_witness, brillig_witness)
            }
        }
        (Err(acir_error), Ok(brillig_witness)) => match return_witness_brillig {
            Some(_) => CompareResults::AcirFailed(acir_error.to_string(), brillig_witness),
            None => CompareResults::BothFailed(
                acir_error.to_string(),
                "Brillig program does not return anything".into(),
            ),
        },
        (Ok(acir_witness), Err(brillig_error)) => match return_witness_acir {
            Some(_) => CompareResults::BrilligFailed(brillig_error.to_string(), acir_witness),
            None => CompareResults::BothFailed(
                "ACIR program does not return anything".into(),
                brillig_error.to_string(),
            ),
        },
        (Err(acir_error), Err(brillig_error)) => {
            CompareResults::BothFailed(acir_error.to_string(), brillig_error.to_string())
        }
    }
}
