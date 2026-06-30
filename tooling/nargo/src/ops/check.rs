use acvm::{
    acir::circuit::AcirOpcodeLocation,
    compiler::{CircuitSimulator, SimulationFailure},
};
use noirc_artifacts::program::CompiledProgram;
use noirc_driver::ErrorsAndWarnings;
use noirc_errors::CustomDiagnostic;

/// Run each function through a circuit simulator to check that they are solvable.
#[tracing::instrument(level = "trace", skip_all)]
pub fn check_program(compiled_program: &CompiledProgram) -> Result<(), ErrorsAndWarnings> {
    for (i, circuit) in compiled_program.program.functions.iter().enumerate() {
        let Some(failure) = CircuitSimulator::check_circuit(circuit) else {
            continue;
        };
        let diag = match failure {
            SimulationFailure::UnsolvableOpcode(opcode) => {
                if let Some(call_stack) =
                    compiled_program.debug[i].acir_locations.get(&AcirOpcodeLocation::new(opcode))
                {
                    let call_stack =
                        compiled_program.debug[i].location_tree.get_call_stack(*call_stack);
                    CustomDiagnostic::from_message(
                        &format!("Circuit \"{}\" is not solvable", circuit.function_name),
                        call_stack.last_or_dummy().file,
                    )
                    .with_call_stack(call_stack)
                } else {
                    CustomDiagnostic::from_message(
                        &format!(
                            "Circuit \"{}\" is not solvable for opcode \"{}\"",
                            circuit.function_name, opcode
                        ),
                        fm::FileId::dummy(),
                    )
                }
            }
            SimulationFailure::UnsolvableOutput(witness) => CustomDiagnostic::from_message(
                &format!(
                    "Circuit \"{}\" has a return value \"{}\" that is never computed",
                    circuit.function_name, witness
                ),
                fm::FileId::dummy(),
            ),
        };
        return Err(vec![diag]);
    }
    Ok(())
}
