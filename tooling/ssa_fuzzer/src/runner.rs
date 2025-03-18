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
use nargo::ops::execute::execute_program;
use thiserror::Error;

/// Errors that can occur during execution of the program
/// It can be NargoError or rust panic
#[derive(Debug, Error)]
pub enum RunnerErrors {
    #[error("Nargo error: {0}")]
    NargoError(NargoError<FieldElement>),
    #[error("Execution panicked: {0}")]
    ExecutionPanicked(String),
}

/// Low level function to execute the given program with the given initial witness
/// It uses nargo execute_program to run the program
fn execute<B: BlackBoxFunctionSolver<FieldElement> + Default>(
    _foreign_call_resolver_url: Option<&str>,
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
) -> Result<FieldElement, RunnerErrors> {
    let result =
        std::panic::catch_unwind(|| execute::<Bn254BlackBoxSolver>(None, program, initial_witness))
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Execution panicked"));

    match result {
        Ok(result) => match result {
            Ok(result) => {
                let witness = result.peek().expect("Should have at least one witness on the stack");
                Ok(witness.witness[&return_witness])
            }
            Err(e) => Err(RunnerErrors::NargoError(e)),
        },
        Err(e) => {
            // execution panicked case
            Err(RunnerErrors::ExecutionPanicked(e.to_string()))
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
) -> (bool, FieldElement, FieldElement) {
    let acir_result = std::panic::catch_unwind(|| {
        execute_single(acir_program, initial_witness.clone(), return_witness_acir)
    })
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "ACIR execution panicked"));
    let brillig_result = std::panic::catch_unwind(|| {
        execute_single(brillig_program, initial_witness, return_witness_brillig)
    })
    .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Brillig execution panicked"));

    // we found bug in case of
    // 1) acir_result != brillig_result
    // 2) acir execution failed, brillig execution succeeded
    // 3) acir execution succeeded, brillig execution failed
    // it has depth 2, because nargo can panic or return NargoError
    match (acir_result, brillig_result) {
        (Ok(acir_result), Ok(brillig_result)) => {
            match (acir_result, brillig_result) {
                (Ok(acir_result), Ok(brillig_result)) => {
                    if acir_result != brillig_result {
                        panic!(
                            "ACIR and Brillig results do not match. ACIR result: {:?}, Brillig result: {:?}",
                            acir_result, brillig_result
                        );
                    }
                    (true, acir_result, brillig_result)
                }
                (Err(e), Ok(brillig_result)) => {
                    panic!(
                        "Failed to execute acir program: {:?}, but brillig program succeeded with value {:?}",
                        e, brillig_result
                    );
                }
                (Ok(acir_result), Err(e)) => {
                    panic!(
                        "Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}",
                        e, acir_result
                    );
                }
                (Err(_e), Err(_e2)) => {
                    // both failed, okay
                    (true, FieldElement::from(0_u32), FieldElement::from(0_u32))
                }
            }
        }
        (Ok(acir_result), Err(e)) => {
            log::debug!(
                "Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}",
                e,
                acir_result
            );
            panic!(
                "Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}",
                e, acir_result
            );
        }
        (Err(e), Ok(brillig_result)) => {
            log::debug!(
                "Failed to execute acir program: {:?}, brillig program succeeded with value {:?}",
                e,
                brillig_result
            );
            panic!(
                "Failed to execute acir program: {:?}, but brillig program succeeded with value {:?}",
                e, brillig_result
            );
        }
        (Err(e), Err(e2)) => {
            // both failed, constructed program unsolvable
            log::debug!("Failed to execute acir program: {:?}", e);
            log::debug!("Failed to execute brillig program: {:?}", e2);
            // we dont care about the result, we have similar behavior in both cases
            (true, FieldElement::from(0_u32), FieldElement::from(0_u32))
        }
    }
}
