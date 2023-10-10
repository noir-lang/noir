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

#[allow(deprecated)]
pub(crate) enum VMType {
    Brillig(VM<'static, BarretenbergSolver>),
    #[allow(dead_code)]
    Acvm(VM<'static, BarretenbergSolver>),
}

impl VMType {
    pub(crate) fn program_counter(&self) -> usize {
        match self {
            VMType::Brillig(vm) => vm.program_counter(),
            _ => unimplemented!(),
        }
    }
    pub(crate) fn process_opcode(&mut self) -> acvm::brillig_vm::VMStatus {
        match self {
            VMType::Brillig(vm) => vm.process_opcode(),
            _ => unimplemented!(),
        }
    }
    pub(crate) fn get_memory(&self) -> &Vec<acvm::acir::brillig::Value> {
        match self {
            VMType::Brillig(vm) => vm.get_memory(),
            _ => unimplemented!(),
        }
    }
    pub(crate) fn get_registers(&self) -> &Vec<acvm::acir::brillig::Value> {
        match self {
            VMType::Brillig(vm) => {
                let registers = vm.get_registers();
                &registers.inner
            }
            _ => unimplemented!(),
        }
    }
}
