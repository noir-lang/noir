use acvm::{
    acir::{
        native_types::{WitnessStack, WitnessMap, Witness},
        circuit::Program,
    },
    FieldElement, BlackBoxFunctionSolver
};
use bn254_blackbox_solver::Bn254BlackBoxSolver;
use nargo::ops::execute::execute_program;
use nargo::foreign_calls::DefaultForeignCallBuilder;
use nargo::PrintOutput;
use nargo::errors::NargoError;
use log;
use env_logger;

fn execute<B: BlackBoxFunctionSolver<FieldElement> + Default>(
    _foreign_call_resolver_url: Option<&str>, 
    acir_program: &Program<FieldElement>, 
    brillig_program: &Program<FieldElement>, 
    initial_witness: WitnessMap<FieldElement>) -> (
        Result<WitnessStack<FieldElement>, NargoError<FieldElement>>,
        Result<WitnessStack<FieldElement>, NargoError<FieldElement>>
    ) 
{
    let acir_result = execute_program(
        acir_program,
        initial_witness.clone(),
        &B::default(),
        &mut DefaultForeignCallBuilder::default()
            .with_output(PrintOutput::None)
            .build()
    );

    let brillig_result = execute_program(
        brillig_program,
        initial_witness,
        &B::default(),
        &mut DefaultForeignCallBuilder::default()
            .with_output(PrintOutput::None)
            .build()
    );

    (acir_result, brillig_result)
}

pub fn run_and_compare(
    acir_program: &Program<FieldElement>, 
    brillig_program: &Program<FieldElement>, 
    initial_witness: WitnessMap<FieldElement>, 
    return_witness_acir: Witness,
    return_witness_brillig: Witness
) -> (bool, FieldElement, FieldElement) {
    let (acir_result, brillig_result) = execute::<Bn254BlackBoxSolver>(None, acir_program, brillig_program, initial_witness);
    let _ = env_logger::try_init();
    match (acir_result, brillig_result) {
        (Ok(acir_witness_stack), Ok(brillig_witness_stack)) => {
            let acir_map = &acir_witness_stack.peek().expect("Should have at least one witness on the stack").witness;
            let brillig_map = &brillig_witness_stack.peek().expect("Should have at least one witness on the stack").witness;
            let acir_result = acir_map[&return_witness_acir];
            let brillig_result = brillig_map[&return_witness_brillig];
            (acir_result == brillig_result, acir_result, brillig_result)
        }
        (Ok(acir_witness_stack), Err(e)) => {
            let acir_map = &acir_witness_stack.peek().expect("Should have at least one witness on the stack").witness;
            let acir_result = acir_map[&return_witness_acir];
            log::error!("Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}", e, acir_result);
            panic!("Failed to execute brillig program: {:?}, but acir program succeeded with value {:?}", e, acir_result);
        }
        (Err(e), Ok(brillig_witness_stack)) => {
            let brillig_map = &brillig_witness_stack.peek().expect("Should have at least one witness on the stack").witness;
            let brillig_result = brillig_map[&return_witness_brillig];
            log::error!("Failed to execute acir program: {:?}, brillig program succeeded with value {:?}", e, brillig_result);
            panic!("Failed to execute acir program: {:?}, but brillig program succeeded with value {:?}", e, brillig_result);
        }
        (Err(e), Err(e2)) => {
            // both failed, constructed program unsolvable
            log::error!("Failed to execute acir program: {:?}", e);
            log::error!("Failed to execute brillig program: {:?}", e2);
            return (true, FieldElement::from(0 as u32), FieldElement::from(0 as u32));
        }
    }
}
