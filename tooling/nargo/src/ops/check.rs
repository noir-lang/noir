use acvm::compiler::CircuitSimulator;
use noirc_driver::{CompiledProgram, ErrorsAndWarnings};
use noirc_errors::{CustomDiagnostic, FileDiagnostic};

pub fn check_program(compiled_program: &CompiledProgram) -> Result<(), ErrorsAndWarnings> {
    // Check if the program is solvable
    for circuit in &compiled_program.program.functions {
        let mut simulator = CircuitSimulator::default();
        if !simulator.check_circuit(circuit) {
            let diag = FileDiagnostic {
                file_id: fm::FileId::dummy(),
                diagnostic: CustomDiagnostic::from_message("ACVM simulation failed"),
            };
            return Err(vec![diag]);
        }
    }
    Ok(())
}
