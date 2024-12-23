use acvm::compiler::CircuitSimulator;
use noirc_driver::{CompiledProgram, ErrorsAndWarnings};
use noirc_errors::{CustomDiagnostic, FileDiagnostic};

/// Run each function through a circuit simulator to check that they are solvable.
#[tracing::instrument(level = "trace", skip_all)]
pub fn check_program(compiled_program: &CompiledProgram) -> Result<(), ErrorsAndWarnings> {
    for (i, circuit) in compiled_program.program.functions.iter().enumerate() {
        let mut simulator = CircuitSimulator::default();
        if !simulator.check_circuit(circuit) {
            let diag = FileDiagnostic {
                file_id: fm::FileId::dummy(),
                diagnostic: CustomDiagnostic::from_message(&format!(
                    "Circuit \"{}\" is not solvable",
                    compiled_program.names[i]
                )),
            };
            return Err(vec![diag]);
        }
    }
    Ok(())
}
