use acvm::{
    BlackBoxFunctionSolver, FieldElement,
    acir::{
        circuit::Program,
        native_types::{WitnessMap, WitnessStack},
    },
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
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
) -> Result<WitnessStack<FieldElement>, SsaExecutionError> {
    // Save the current panic hook
    let previous_hook = std::panic::take_hook();
    let panic_message = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
    let hook_message = panic_message.clone();

    // Set a custom panic hook to capture panic messages
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            format!("Panic: {s}")
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            format!("Panic: {s}")
        } else {
            format!("Unknown panic: {panic_info:?}")
        };

        if let Some(location) = panic_info.location() {
            let loc_info = format!(" at {}:{}", location.file(), location.line());
            *hook_message.lock().unwrap() = message + &loc_info;
        } else {
            *hook_message.lock().unwrap() = message;
        }
    }));

    let result =
        std::panic::catch_unwind(|| execute::<Bn254BlackBoxSolver>(program, initial_witness));

    // Restore the previous panic hook
    std::panic::set_hook(previous_hook);

    match result {
        Ok(result) => match result {
            Ok(result) => Ok(result),
            Err(e) => Err(SsaExecutionError::ExecutionFailed(e)),
        },
        Err(_) => {
            let error_msg = panic_message.lock().unwrap().clone();
            if error_msg.is_empty() {
                Err(SsaExecutionError::ExecutionPanicked("Unknown error".to_string()))
            } else {
                Err(SsaExecutionError::ExecutionPanicked(error_msg))
            }
        }
    }
}
