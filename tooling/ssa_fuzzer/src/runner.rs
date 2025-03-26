use acvm::{
    BlackBoxFunctionSolver, FieldElement,
    acir::{
        circuit::Program,
        native_types::{Witness, WitnessMap, WitnessStack},
    },
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::PrintOutput;
use nargo::errors::NargoError;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::ops::execute_program;
use thiserror::Error;

/// Errors that can occur during execution of the program
/// It can be NargoError or rust panic
#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(NargoError<FieldElement>),
    #[error("Execution panicked: {0}")]
    ExecutionPanicked(String),
}


pub enum CompareResults {
    Agree(FieldElement),
    Disagree(FieldElement, FieldElement),
    BothFailed(String, String),
    AcirFailed(String, FieldElement),
    BrilligFailed(String, FieldElement),
}

/// Low level function to execute the given program with the given initial witness
/// It uses nargo execute_program to run the program
fn execute<B: BlackBoxFunctionSolver<FieldElement> + Default>(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<WitnessStack<FieldElement>, NargoError<FieldElement>> {
    let result = execute_program(
        program,
        initial_witness.clone(),
        &B::default(),
        &mut DefaultForeignCallBuilder::default().with_output(PrintOutput::None).build(),
    );

    result
}

/// High level function to execute the given program with the given initial witness
/// It returns the result of the program execution
pub fn execute_single(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    return_witness: Witness,
) -> Result<FieldElement, RunnerError> {
    let result = std::panic::catch_unwind(|| execute::<Bn254BlackBoxSolver>(program, initial_witness));

    match result {
        Ok(result) => {
            match result {
                Ok(result) => Ok(result.peek().expect("Should have at least one witness on the stack").witness[&return_witness]),
                Err(e) => Err(RunnerError::ExecutionFailed(e)),
            }
        },
        Err(e) => {
            if let Some(message) = e.downcast_ref::<&str>() {
                Err(RunnerError::ExecutionPanicked(message.to_string()))
            } else {
                Err(RunnerError::ExecutionPanicked("Unknown error".to_string()))
            }
        }
    }
    
}

/// High level function to execute the given ACIR and Brillig programs with the given initial witness
/// It returns a tuple with a boolean indicating if the programs succeeded,
/// and the results of the ACIR and Brillig programs
pub fn run_and_compare(
    acir_program: &Program<FieldElement>,
    brillig_program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
    return_witness_acir: Witness,
    return_witness_brillig: Witness,
) -> CompareResults {
    let acir_result = execute_single(acir_program, initial_witness.clone(), return_witness_acir);
    let brillig_result = execute_single(brillig_program, initial_witness, return_witness_brillig);

    // we found bug in case of
    // 1) acir_result != brillig_result
    // 2) acir execution failed, brillig execution succeeded
    // 3) acir execution succeeded, brillig execution failed
    // it has depth 2, because nargo can panic or return NargoError
    match (acir_result, brillig_result) {
        (Ok(acir_result), Ok(brillig_result)) => {
            if acir_result == brillig_result {
                CompareResults::Agree(acir_result)
            } else {
                CompareResults::Disagree(acir_result, brillig_result)
            }
        }
        (Err(acir_error), Ok(brillig_result)) => CompareResults::AcirFailed(acir_error.to_string(), brillig_result),
        (Ok(acir_result), Err(brillig_error)) => CompareResults::BrilligFailed(brillig_error.to_string(), acir_result),
        (Err(acir_error), Err(brillig_error)) => CompareResults::BothFailed(acir_error.to_string(), brillig_error.to_string()),
    }
}
