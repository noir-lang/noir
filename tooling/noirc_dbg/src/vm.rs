use acvm::acir::brillig::Opcode;
use acvm::brillig_vm::brillig::Value;
use acvm::brillig_vm::{Registers, VM};
#[allow(deprecated)]
use barretenberg_blackbox_solver::BarretenbergSolver;

/// Create virtual machine to debug program.
#[allow(deprecated)]
pub(crate) fn brillig_new<'a>(
    program: &'a [Opcode],
    registers: Registers,
    memory: Vec<Value>,
    solver: &'a BarretenbergSolver,
) -> VM<'a, BarretenbergSolver> {
    VM::new(registers, memory, program.to_vec(), vec![], solver)
}

/// A wrapper over virtual machines. Should provide single interface for brillig_vm and acvm.
#[allow(deprecated)]
pub(crate) enum VMType {
    /// Keep brillig vm.
    Brillig(VM<'static, BarretenbergSolver>),
    /// Keep acvm.
    #[allow(dead_code)]
    Acvm(VM<'static, BarretenbergSolver>),
}

impl std::fmt::Debug for VMType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VMType::Brillig(_) => f.write_str("Brillig").unwrap(),
            VMType::Acvm(_) => f.write_str("Acvm").unwrap(),
        }
        Ok(())
    }
}

impl VMType {
    /// Get machine program counter.
    pub(crate) fn program_counter(&self) -> usize {
        match self {
            VMType::Brillig(vm) => vm.program_counter(),
            _ => unimplemented!(),
        }
    }

    /// Make single step.
    pub(crate) fn process_opcode(&mut self) -> acvm::brillig_vm::VMStatus {
        match self {
            VMType::Brillig(vm) => vm.process_opcode(),
            _ => unimplemented!(),
        }
    }

    /// Read vm memory.
    pub(crate) fn get_memory(&self) -> &Vec<acvm::acir::brillig::Value> {
        match self {
            VMType::Brillig(vm) => vm.get_memory(),
            _ => unimplemented!(),
        }
    }

    /// Get vm registers/variables.
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
