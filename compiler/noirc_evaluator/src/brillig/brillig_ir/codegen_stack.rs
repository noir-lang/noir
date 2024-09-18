mod mov_registers_solver;

use acvm::{acir::brillig::MemoryAddress, AcirField};
use mov_registers_solver::{is_loop, MoveRegistersSolver};

use super::{debug_show::DebugToString, registers::RegisterAllocator, BrilligContext};

impl<F: AcirField + DebugToString, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// This function moves values from a set of registers to another set of registers.
    /// It first moves all sources to new allocated registers to avoid overwriting.
    pub(crate) fn codegen_mov_registers_to_registers(
        &mut self,
        sources: Vec<MemoryAddress>,
        destinations: Vec<MemoryAddress>,
    ) {
        let chains =
            MoveRegistersSolver::sources_destinations_to_move_chains(sources, destinations);
        for mut chain in chains {
            assert!(!chain.is_empty(), "Empty chain found");

            // If the chain is a loop, we need a temporary register to break the loop
            if is_loop(&chain) {
                let temp_register = self.allocate_register();
                // Backup the first destination
                self.mov_instruction(temp_register, chain[0].1);
                // Do all operations but the last one
                let last = chain.pop().unwrap();
                for (source, destination) in chain {
                    self.mov_instruction(destination, source);
                }
                // Move the backup to the last destination
                self.mov_instruction(last.1, temp_register);
                self.deallocate_register(temp_register);
            } else {
                for (source, destination) in chain {
                    self.mov_instruction(destination, source);
                }
            }
        }
    }
}
