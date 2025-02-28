use acvm::{
    acir::{
        native_types::{WitnessStack, WitnessMap, Witness},
        circuit::Program,
    },
    FieldElement, BlackBoxFunctionSolver
};
use std::time::Instant;
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::ops::execute::execute_program;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::PrintOutput;
use nargo::errors::NargoError;
use log;
use env_logger;

fn execute<B: BlackBoxFunctionSolver<FieldElement> + Default>(
    _foreign_call_resolver_url: Option<&str>, 
    program: &Program<FieldElement>, 
    initial_witness: WitnessMap<FieldElement>) -> Result<WitnessStack<FieldElement>, NargoError<FieldElement>> 
{
    let result = execute_program(
        program,
        initial_witness.clone(),
        &B::default(),
        &mut DefaultForeignCallBuilder::default()
            .with_output(PrintOutput::None)
            .build()
    );

    result
}

pub fn execute_single(
    program: &Program<FieldElement>, 
    initial_witness: WitnessMap<FieldElement>,
    return_witness: Witness
) -> Result<FieldElement, NargoError<FieldElement>> {
    let result = std::panic::catch_unwind(|| {
        execute::<Bn254BlackBoxSolver>(None, program, initial_witness)
    }).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Execution panicked"));

    let result = result.unwrap().unwrap(); // lol. 
    let witness = result.peek().expect("Should have at least one witness on the stack");
    Ok(witness.witness[&return_witness])
}

pub fn run_and_compare(
    acir_program: &Program<FieldElement>, 
    brillig_program: &Program<FieldElement>, 
    initial_witness: WitnessMap<FieldElement>, 
    return_witness_acir: Witness,
    return_witness_brillig: Witness
) -> (bool, FieldElement, FieldElement) {
    let acir_start = Instant::now();
    let acir_result = std::panic::catch_unwind(|| {
        execute_single(acir_program, initial_witness.clone(), return_witness_acir)
    }).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "ACIR execution panicked"));
    let acir_duration = acir_start.elapsed();
    //println!("ACIR execution took: {:?}", acir_duration);
    let brillig_start = Instant::now();
    let brillig_result = std::panic::catch_unwind(|| {
        execute_single(brillig_program, initial_witness, return_witness_brillig)
    }).map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "Brillig execution panicked"));
    let brillig_duration = brillig_start.elapsed();
    //println!("Brillig execution took: {:?}", brillig_duration);
    let _ = env_logger::try_init();
    match (acir_result, brillig_result) {
        (Ok(acir_result), Ok(brillig_result)) => {
            let acir_result = acir_result.unwrap();
            let brillig_result = brillig_result.unwrap();
            if acir_result != brillig_result {
                panic!("ACIR and Brillig results do not match. ACIR result: {:?}, Brillig result: {:?}", acir_result, brillig_result);
            }
            (true, acir_result, brillig_result)
        }
        (Ok(acir_result), Err(e)) => {
            log::debug!("Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}", e, acir_result);
            panic!("Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}", e, acir_result);
        }
        (Err(e), Ok(brillig_result)) => {
            log::debug!("Failed to execute acir program: {:?}, brillig program succeeded with value {:?}", e, brillig_result);
            panic!("Failed to execute acir program: {:?}, but brillig program succeeded with value {:?}", e, brillig_result);
        }
        (Err(e), Err(e2)) => {
            // both failed, constructed program unsolvable
            log::debug!("Failed to execute acir program: {:?}", e);
            log::debug!("Failed to execute brillig program: {:?}", e2);
            return (true, FieldElement::from(0_u32), FieldElement::from(0_u32));
        }
    }
}
