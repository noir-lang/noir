use acvm::acir::brillig::MemoryAddress;

use super::BrilligContext;

impl BrilligContext {
    /// This function moves values from a set of registers to another set of registers.
    /// It first moves all sources to new allocated registers to avoid overwriting.
    pub(crate) fn codegen_mov_registers_to_registers(
        &mut self,
        sources: Vec<MemoryAddress>,
        destinations: Vec<MemoryAddress>,
    ) {
        let new_sources: Vec<_> = sources
            .iter()
            .map(|source| {
                let new_source = self.allocate_register();
                self.mov_instruction(new_source, *source);
                new_source
            })
            .collect();
        for (new_source, destination) in new_sources.iter().zip(destinations.iter()) {
            self.mov_instruction(*destination, *new_source);
            self.deallocate_register(*new_source);
        }
    }
}
