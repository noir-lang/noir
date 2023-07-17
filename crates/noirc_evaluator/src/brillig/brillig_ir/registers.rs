use acvm::acir::brillig::RegisterIndex;

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

    /// Ensures a register is allocated.
    pub(crate) fn ensure_register_is_allocated(&mut self, register: RegisterIndex) {
        let index = register.to_usize();
        if index < self.next_free_register_index {
            // If it could be allocated, check if it's in the deallocated list and remove it from there
            self.deallocated_registers.retain(|&r| r != register);
        } else {
            // If it couldn't yet be, expand the register space.
            self.next_free_register_index = index + 1;
        }
    }

    /// Lazily iterate over the used registers,
    /// counting to next_free_register_index while excluding deallocated and reserved registers.
    pub(crate) fn used_registers_iter(&self) -> impl Iterator<Item = RegisterIndex> + '_ {
        (ReservedRegisters::NUM_RESERVED_REGISTERS..self.next_free_register_index)
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
    /// TODO(AD): currently, register deallocation is only done with immediate values.
    /// TODO(AD): See https://github.com/noir-lang/noir/issues/1720
    pub(crate) fn deallocate_register(&mut self, register_index: RegisterIndex) {
        assert!(!self.deallocated_registers.contains(&register_index));
        self.deallocated_registers.push(register_index);
    }
}
