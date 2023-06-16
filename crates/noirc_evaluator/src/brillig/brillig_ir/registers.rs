use acvm::acir::brillig_vm::RegisterIndex;

use super::ReservedRegisters;

/// Every brillig stack frame/call context has its own view of register space.
/// This is maintained by copying these registers to the stack during calls and reading them back.
///
/// Each has a stack base pointer from which all stack allocations can be offset.
pub(crate) struct BrilligRegistersContext {
    /// A free-list of registers that have been deallocated and can be used again.
    /// TODO(AD): currently, register deallocation is only done with immediate values.
    /// TODO(AD): See https://github.com/noir-lang/noir/issues/1720
    deallocated_registers: Vec<RegisterIndex>,
    /// A usize indicating the next un-used register.
    next_free_register_index: usize,
}

impl BrilligRegistersContext {
    /// Initial register allocation
    pub(crate) fn new() -> BrilligRegistersContext {
        BrilligRegistersContext {
            deallocated_registers: Vec::new(),
            next_free_register_index: ReservedRegisters::len(),
        }
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
    /// TODO(AD): currently, register deallocation is only done with immediate values.
    /// TODO(AD): See https://github.com/noir-lang/noir/issues/1720
    pub(crate) fn deallocate_register(&mut self, register_index: RegisterIndex) {
        assert!(!self.deallocated_registers.contains(&register_index));
        self.deallocated_registers.push(register_index);
    }
}
