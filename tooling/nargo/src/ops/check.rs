use acvm::compiler::CircuitSimulator;
use noirc_driver::{CompiledProgram, ErrorsAndWarnings};
use noirc_errors::CustomDiagnostic;

/// Run each function through a circuit simulator to check that they are solvable.
#[tracing::instrument(level = "trace", skip_all)]
pub fn check_program(compiled_program: &CompiledProgram) -> Result<(), ErrorsAndWarnings> {
    for circuit in &compiled_program.program.functions {
        let mut simulator = CircuitSimulator::default();
        if !simulator.check_circuit(circuit) {
            let diag = CustomDiagnostic::from_message(
                &format!("Circuit \"{}\" is not solvable", circuit.name),
                fm::FileId::dummy(),
            );
            return Err(vec![diag]);
        }
    }
    Ok(())
}
