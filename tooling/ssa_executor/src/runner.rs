use acvm::{
    BlackBoxFunctionSolver, FieldElement,
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
};
use m31_blackbox_solver::M31BlackBoxSolver;
use nargo::errors::NargoError;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::ops::execute_program;
use thiserror::Error;

/// Errors that can occur during execution of the program
/// It can be NargoError or rust panic
#[derive(Debug, Error)]
pub enum SsaExecutionError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(NargoError<FieldElement>),
    #[error("Execution panicked: {0}")]
    ExecutionPanicked(String),
    #[error("SSA compilation failed: {0}")]
    SsaCompilationFailed(String),
    #[error("SSA parsing failed: {0}")]
    SsaParsingFailed(String),
}

/// Low level function to execute the given program with the given initial witness
/// It uses nargo execute_program to run the program
fn execute<B: BlackBoxFunctionSolver<FieldElement> + Default>(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<WitnessStack<FieldElement>, NargoError<FieldElement>> {
    execute_program(
        program,
        initial_witness.clone(),
        &B::default(),
        &mut DefaultForeignCallBuilder::default().build(),
    )
}

/// High level function to execute the given program with the given initial witness
/// It returns the result of the program execution
pub fn execute_single(
    program: &Program<FieldElement>,
    initial_witness: WitnessMap<FieldElement>,
) -> Result<WitnessMap<FieldElement>, SsaExecutionError> {
    let result =
        std::panic::catch_unwind(|| execute::<M31BlackBoxSolver>(program, initial_witness));

    match result {
        Ok(result) => match result {
            Ok(result) => Ok(result
                .peek()
                .expect("Should have at least one witness on the stack")
                .witness
                .clone()),
            Err(e) => Err(SsaExecutionError::ExecutionFailed(e)),
        },
        Err(e) => {
            if let Some(message) = e.downcast_ref::<&str>() {
                Err(SsaExecutionError::ExecutionPanicked(message.to_string()))
            } else {
                Err(SsaExecutionError::ExecutionPanicked("Unknown error".to_string()))
            }
        }
    }
}
