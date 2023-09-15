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
    pub(crate) fn new() -> Self {
        Self {
            deallocated_registers: Vec::new(),
            next_free_register_index: ReservedRegisters::len(),
        }
    }

    pub(crate) fn from_preallocated_registers(preallocated_registers: Vec<RegisterIndex>) -> Self {
        let next_free_register_index = preallocated_registers.iter().fold(
            ReservedRegisters::len(),
            |free_register_index, preallocated_register| {
                if preallocated_register.to_usize() < free_register_index {
                    free_register_index
                } else {
                    preallocated_register.to_usize() + 1
                }
            },
        );
        let mut deallocated_registers = Vec::new();
        for i in ReservedRegisters::len()..next_free_register_index {
            if !preallocated_registers.contains(&RegisterIndex::from(i)) {
                deallocated_registers.push(RegisterIndex::from(i));
            }
        }

        Self { deallocated_registers, next_free_register_index }
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
