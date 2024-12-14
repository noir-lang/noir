use std::collections::BTreeSet;

use acvm::acir::brillig::{HeapArray, HeapVector, MemoryAddress};
use iter_extended::vecmap;

use crate::brillig::brillig_ir::entry_point::MAX_STACK_SIZE;

use super::{
    brillig_variable::SingleAddrVariable,
    entry_point::{MAX_SCRATCH_SPACE, MAX_STACK_FRAME_SIZE},
    BrilligContext, ReservedRegisters,
};

pub(crate) trait RegisterAllocator {
    /// First valid memory address
    fn start() -> usize;
    /// Last valid memory address
    fn end() -> usize;
    /// Allocates a new register.
    fn allocate_register(&mut self) -> MemoryAddress;
    /// Push a register to the deallocation list, ready for reuse.
    fn deallocate_register(&mut self, register_index: MemoryAddress);
    /// Ensures a register is allocated, allocating it if necessary
    fn ensure_register_is_allocated(&mut self, register: MemoryAddress);
    /// Creates a new register context from a set of registers allocated previously.
    fn from_preallocated_registers(preallocated_registers: Vec<MemoryAddress>) -> Self;
}

/// Every brillig stack frame/call context has its own view of register space.
/// This is maintained by copying these registers to the stack during calls and reading them back.
pub(crate) struct Stack {
    storage: DeallocationListAllocator,
}

impl Stack {
    pub(crate) fn new() -> Self {
        Self { storage: DeallocationListAllocator::new(Self::start()) }
    }

    fn is_within_bounds(register: MemoryAddress) -> bool {
        let offset = register.unwrap_relative();
        offset >= Self::start() && offset < Self::end()
    }

    pub(crate) fn empty_stack_start(&self) -> MemoryAddress {
        MemoryAddress::relative(self.storage.empty_registers_start(Self::start()))
    }
}

impl RegisterAllocator for Stack {
    fn start() -> usize {
        1 // Previous stack pointer is the first stack item
    }

    fn end() -> usize {
        MAX_STACK_FRAME_SIZE
    }

    fn ensure_register_is_allocated(&mut self, register: MemoryAddress) {
        assert!(Self::is_within_bounds(register), "Register out of stack bounds");
        self.storage.ensure_register_is_allocated(register.unwrap_relative());
    }

    fn allocate_register(&mut self) -> MemoryAddress {
        let allocated = MemoryAddress::relative(self.storage.allocate_register());
        assert!(Self::is_within_bounds(allocated), "Stack frame too deep");
        allocated
    }

    fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.storage.deallocate_register(register_index.unwrap_relative());
    }

    fn from_preallocated_registers(preallocated_registers: Vec<MemoryAddress>) -> Self {
        for register in &preallocated_registers {
            assert!(Self::is_within_bounds(*register), "Register out of stack bounds");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                Self::start(),
                vecmap(preallocated_registers, |r| r.unwrap_relative()),
            ),
        }
    }
}

/// Procedure arguments and returns are passed through scratch space.
/// This avoids having to dump and restore the stack to call procedures.
/// The scratch space is a much smaller set of memory cells.
pub(crate) struct ScratchSpace {
    storage: DeallocationListAllocator,
}

impl ScratchSpace {
    pub(crate) fn new() -> Self {
        Self { storage: DeallocationListAllocator::new(Self::start()) }
    }

    fn is_within_bounds(register: MemoryAddress) -> bool {
        let index = register.unwrap_direct();
        index >= Self::start() && index < Self::end()
    }
}

impl RegisterAllocator for ScratchSpace {
    fn start() -> usize {
        ReservedRegisters::len() + MAX_STACK_SIZE
    }

    fn end() -> usize {
        ReservedRegisters::len() + MAX_STACK_SIZE + MAX_SCRATCH_SPACE
    }

    fn ensure_register_is_allocated(&mut self, register: MemoryAddress) {
        assert!(Self::is_within_bounds(register), "Register out of scratch space bounds");
        self.storage.ensure_register_is_allocated(register.unwrap_direct());
    }

    fn allocate_register(&mut self) -> MemoryAddress {
        let allocated = MemoryAddress::direct(self.storage.allocate_register());
        assert!(Self::is_within_bounds(allocated), "Scratch space too deep");
        allocated
    }

    fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.storage.deallocate_register(register_index.unwrap_direct());
    }

    fn from_preallocated_registers(preallocated_registers: Vec<MemoryAddress>) -> Self {
        for register in &preallocated_registers {
            assert!(Self::is_within_bounds(*register), "Register out of scratch space bounds");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                Self::start(),
                vecmap(preallocated_registers, |r| r.unwrap_direct()),
            ),
        }
    }
}

struct DeallocationListAllocator {
    /// A free-list of registers that have been deallocated and can be used again.
    deallocated_registers: BTreeSet<usize>,
    /// A usize indicating the next un-used register.
    next_free_register_index: usize,
}

impl DeallocationListAllocator {
    fn new(start: usize) -> Self {
        Self { deallocated_registers: BTreeSet::new(), next_free_register_index: start }
    }

    fn ensure_register_is_allocated(&mut self, index: usize) {
        if index < self.next_free_register_index {
            // If it could be allocated, check if it's in the deallocated list and remove it from there
            self.deallocated_registers.retain(|&r| r != index);
        } else {
            // If it couldn't yet be, expand the register space.
            self.next_free_register_index = index + 1;
        }
    }

    fn allocate_register(&mut self) -> usize {
        // If we have a register in our free list of deallocated registers,
        // consume it first. This prioritizes reuse.
        if let Some(register) = self.deallocated_registers.pop_first() {
            return register;
        }
        // Otherwise, move to our latest register.
        let register = self.next_free_register_index;
        self.next_free_register_index += 1;
        register
    }

    fn deallocate_register(&mut self, register_index: usize) {
        assert!(!self.deallocated_registers.contains(&register_index));
        self.deallocated_registers.insert(register_index);
    }

    fn from_preallocated_registers(start: usize, preallocated_registers: Vec<usize>) -> Self {
        let next_free_register_index = preallocated_registers.iter().fold(
            start,
            |free_register_index, &preallocated_register| {
                if preallocated_register < free_register_index {
                    free_register_index
                } else {
                    preallocated_register + 1
                }
            },
        );
        let mut deallocated_registers = BTreeSet::new();
        for i in start..next_free_register_index {
            if !preallocated_registers.contains(&i) {
                deallocated_registers.insert(i);
            }
        }

        Self { deallocated_registers, next_free_register_index }
    }

    fn empty_registers_start(&self, start: usize) -> usize {
        let mut first_free = self.next_free_register_index;
        while first_free > start {
            if !self.deallocated_registers.contains(&(first_free - 1)) {
                break;
            }
            first_free -= 1;
        }
        first_free
    }
}

impl<F, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Allocates an unused register.
    pub(crate) fn allocate_register(&mut self) -> MemoryAddress {
        self.registers.allocate_register()
    }

    pub(crate) fn set_allocated_registers(&mut self, allocated_registers: Vec<MemoryAddress>) {
        self.registers = Registers::from_preallocated_registers(allocated_registers);
    }

    /// Push a register to the deallocation list, ready for reuse.
    pub(crate) fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.registers.deallocate_register(register_index);
    }

    /// Deallocates the address where the single address variable is stored
    pub(crate) fn deallocate_single_addr(&mut self, var: SingleAddrVariable) {
        self.deallocate_register(var.address);
    }

    pub(crate) fn deallocate_heap_array(&mut self, arr: HeapArray) {
        self.deallocate_register(arr.pointer);
    }

    pub(crate) fn deallocate_heap_vector(&mut self, vec: HeapVector) {
        self.deallocate_register(vec.pointer);
        self.deallocate_register(vec.size);
    }
}

#[cfg(test)]
mod tests {
    use crate::brillig::brillig_ir::registers::{RegisterAllocator, Stack};

    #[test]
    fn stack_should_prioritize_returning_low_registers() {
        let mut stack = Stack::new();
        let one = stack.allocate_register();
        let _two = stack.allocate_register();
        let three = stack.allocate_register();

        stack.deallocate_register(three);
        stack.deallocate_register(one);

        let one_again = stack.allocate_register();
        assert_eq!(one, one_again);
    }
}
