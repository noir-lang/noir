use acvm::acir::brillig::Opcode;
use acvm::brillig_vm::{Registers, VM};
#[allow(deprecated)]
use barretenberg_blackbox_solver::BarretenbergSolver;

/// Create virtual machine to debug program.
#[allow(deprecated)]
pub(crate) fn new<'a>(
    program: &'a [Opcode],
    registers: Registers,
    solver: &'a BarretenbergSolver,
) -> VM<'a, BarretenbergSolver> {
    VM::new(
        /* registers */ Registers { inner: vec![] },
        registers.inner,
        program.to_vec(),
        vec![],
        solver,
    )
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
