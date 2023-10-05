use acvm::brillig_vm::{Registers, VM};
#[allow(deprecated)]
use barretenberg_blackbox_solver::BarretenbergSolver;
use noirc_evaluator::brillig::brillig_ir::artifact::GeneratedBrillig;

/// Create virtual machine to debug program.
#[allow(deprecated)]
pub(crate) fn new(
    program: GeneratedBrillig,
    solver: &BarretenbergSolver,
) -> VM<BarretenbergSolver> {
    VM::new(Registers { inner: vec![] }, vec![], program.byte_code, vec![], solver)
}
