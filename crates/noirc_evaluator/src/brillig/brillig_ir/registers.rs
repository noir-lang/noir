use acvm::acir::brillig_vm::RegisterIndex;

use super::NUM_RESERVED_REGISTERS;

/// Every brillig stack frame/call context has its own view of register space.
/// This is maintained by copying these registers to the stack during calls and reading them back.
///
/// Each has a stack base pointer from which all stack allocations can be offset.
pub(crate) struct BrilligRegistersContext {
    /// A free-list of registers that have been deallocated and can be used again.
    /// TODO(AD): currently, register deallocation is only done with immediate values.
    /// TODO(AD): In the future, we should do liveness analysis to deallocate any registers once unused.
    /// TODO(AD): Have we done a similar analysis elsewhere/will need one?
    deallocated_registers: Vec<RegisterIndex>,
    /// A usize indicating the next un-used register.
    next_free_register_index: usize,
}

impl BrilligRegistersContext {
    /// Initial register allocation
    pub(crate) fn new() -> BrilligRegistersContext {
        BrilligRegistersContext {
            deallocated_registers: Vec::new(),
            next_free_register_index: NUM_RESERVED_REGISTERS,
        }
    }
    /// Lazily iterate over the used registers,
    /// counting to next_free_register_index while excluding deallocated and reserved registers.
    pub(crate) fn used_registers_iter(&self) -> impl Iterator<Item = RegisterIndex> + '_ {
        (NUM_RESERVED_REGISTERS..self.next_free_register_index)
            .map(RegisterIndex::from)
            .filter(|&index| !self.deallocated_registers.contains(&index))
    }

    /// Creates a new register.
    pub(crate) fn allocate_register(&mut self) -> RegisterIndex {
        // If we have a register in our free list of deallocated registers,
        // consume it first. This prioritizes reuse.
        if let Some(register) = self.deallocated_registers.pop() {
            return register;
        }
        // Otherwise, move to our latest register.
        let register = RegisterIndex::from(self.next_free_register_index);
        self.next_free_register_index += 1;
        register
    }

    /// Push a register to the deallocation list, ready for reuse.
    /// TODO(AD): Currently only used for constants. Later, do lifecycle analysis.
    pub(crate) fn deallocate_register(&mut self, register_index: RegisterIndex) {
        assert!(!self.deallocated_registers.contains(&register_index));
        self.deallocated_registers.push(register_index);
    }
}
