//! Memory layout and register allocation for Brillig programs.
//!
//! Brillig execution splits its memory/register space into fixed and dynamic regions. The layout
//! is strictly enforced by the register allocators defined in this module.
//!
//! The regions are as follows:
//! 1. Reserved registers: Used internally by the VM for special purposes (e.g., stack pointer,
//!    free memory pointer, etc.). These registers occupy the lowest memory addresses.
//! 2. Scratch space: Temporary memory used for procedure arguments and return values, allowing
//!    function calls without saving/restoring the stack. Scratch space uses direct addressing.
//! 3. Globals: Read-only memory initialized at the beginning of the program. The size is
//!    determined during global variable compilation and may vary per program. Starts immediately
//!    after the scratch space.
//! 4. Entry point: Memory region containing program input arguments and reserved space for return values.
//!    Starts immediately after the globals and ends before the stack.
//! 5. Stack: Dynamic stack frames used for function-local variables. Each function call
//!    creates its own view of the stack. Stack uses relative addressing for stack frames.
//! 6. Heap: Dynamic memory allocated after the stack for arrays, vectors, and other data
//!    structures. Starts immediately after the stack region.
//!
//! This module contains:
//! - [LayoutConfig]: Centralized configuration of maximum sizes for stack frames, total stack size,
//!   and scratch space. All register allocators query this configuration to determine their memory bounds.
//!   This config is meant to be immutable and provides the following benefits:
//!   - Clear separation between memory layout policy and actual code generation.
//!   - Unit tests to vary memory layouts and ensure bytecode remains consistent.
//! - [RegisterAllocator]: Trait implemented by all memory region allocators. Each allocator is expected
//!   to enforce its own bounds checks and allocation/deallocation logic.
//! - [Stack], [ScratchSpace], and [GlobalSpace]: Register allocator implementations for each memory region.
use std::collections::BTreeSet;

use acvm::acir::brillig::{HeapArray, HeapVector, MemoryAddress};
use iter_extended::vecmap;

use super::{BrilligContext, ReservedRegisters, brillig_variable::SingleAddrVariable};

/// Defines the memory layout for Brillig programs.
///
/// Brillig execution splits its register space into fixed regions
/// (reserved registers, scratch space, globals, calldata) and dynamic regions
/// (stack, heap). This configuration structure centralizes the sizing rules for
/// fixed regions and the stack.
#[derive(Clone, Copy, Debug)]
pub struct LayoutConfig {
    max_stack_frame_size: usize,
    max_stack_size: usize,
    max_scratch_space: usize,
}

impl LayoutConfig {
    pub(crate) fn new(
        max_stack_frame_size: usize,
        num_stack_frames: usize,
        max_scratch_space: usize,
    ) -> Self {
        let max_stack_size = num_stack_frames * max_stack_frame_size;
        Self { max_stack_frame_size, max_stack_size, max_scratch_space }
    }

    pub(crate) fn max_stack_frame_size(&self) -> usize {
        self.max_stack_frame_size
    }

    pub(crate) fn max_stack_size(&self) -> usize {
        self.max_stack_size
    }

    pub(crate) fn max_scratch_space(&self) -> usize {
        self.max_scratch_space
    }

    /// Start of the entry point region:
    /// {reserved} {scratch space} {globals} [call data]
    pub(crate) fn entry_point_start(&self, globals_size: usize) -> usize {
        ScratchSpace::end_with_layout(self) + globals_size
    }

    /// Start of the return data within the entry point region:
    /// {reserved} {scratch space} {globals} {call data} [return data]
    pub(crate) fn return_data_start(&self, globals_size: usize, calldata_size: usize) -> usize {
        self.entry_point_start(globals_size) + calldata_size
    }
}

/// These constants represent expert chosen defaults that are appropriate for the majority of programs
pub(crate) const NUM_STACK_FRAMES: usize = 16;
pub(crate) const MAX_STACK_FRAME_SIZE: usize = 2048;
pub(crate) const MAX_SCRATCH_SPACE: usize = 64;

impl Default for LayoutConfig {
    fn default() -> Self {
        Self::new(MAX_STACK_FRAME_SIZE, NUM_STACK_FRAMES, MAX_SCRATCH_SPACE)
    }
}

pub(crate) trait RegisterAllocator {
    /// First valid memory address
    fn start(&self) -> usize;
    /// Last valid memory address
    fn end(&self) -> usize;
    /// Allocates a new register.
    fn allocate_register(&mut self) -> MemoryAddress;
    /// Push a register to the deallocation list, ready for reuse.
    fn deallocate_register(&mut self, register_index: MemoryAddress);
    /// Ensures a register is allocated, allocating it if necessary
    fn ensure_register_is_allocated(&mut self, register: MemoryAddress);
    /// Creates a new register context from a set of registers allocated previously.
    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self;
    /// Finds the first register that is available based upon the deallocation list
    fn empty_registers_start(&self) -> MemoryAddress;
    /// Return the memory layout used by this allocator.
    fn layout(&self) -> LayoutConfig;
}

/// Every brillig stack frame/call context has its own view of register space.
/// This is maintained by copying these registers to the stack during calls and reading them back.
pub(crate) struct Stack {
    storage: DeallocationListAllocator,
    layout: LayoutConfig,
}

impl Stack {
    pub(crate) fn new(layout: LayoutConfig) -> Self {
        let start = Self::start();
        Self { storage: DeallocationListAllocator::new(start), layout }
    }

    fn is_within_bounds(&self, register: MemoryAddress) -> bool {
        let offset = register.unwrap_relative();
        offset >= self.start() && offset < self.end()
    }

    /// Static start method for constructors
    pub(super) fn start() -> usize {
        1 // Previous stack pointer is the first stack item
    }
}

impl RegisterAllocator for Stack {
    fn start(&self) -> usize {
        Self::start()
    }

    fn end(&self) -> usize {
        self.layout.max_stack_frame_size()
    }

    fn ensure_register_is_allocated(&mut self, register: MemoryAddress) {
        assert!(self.is_within_bounds(register), "Register out of stack bounds");
        self.storage.ensure_register_is_allocated(register.unwrap_relative());
    }

    fn allocate_register(&mut self) -> MemoryAddress {
        let allocated = MemoryAddress::relative(self.storage.allocate_register());
        assert!(self.is_within_bounds(allocated), "Stack frame too deep");
        allocated
    }

    fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.storage.deallocate_register(register_index.unwrap_relative());
    }

    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self {
        let mock = Stack::new(layout);
        for register in &preallocated_registers {
            assert!(mock.is_within_bounds(*register), "Register out of stack bounds");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                mock.start(),
                vecmap(preallocated_registers, |r| r.unwrap_relative()),
            ),
            layout,
        }
    }

    fn empty_registers_start(&self) -> MemoryAddress {
        MemoryAddress::relative(self.storage.empty_registers_start(self.start()))
    }

    fn layout(&self) -> LayoutConfig {
        self.layout
    }
}

/// Procedure arguments and returns are passed through scratch space.
/// This avoids having to dump and restore the stack to call procedures.
/// The scratch space is a much smaller set of memory cells.
pub(crate) struct ScratchSpace {
    storage: DeallocationListAllocator,
    layout: LayoutConfig,
}

impl ScratchSpace {
    pub(crate) fn new(layout: LayoutConfig) -> Self {
        Self { storage: DeallocationListAllocator::new(Self::start()), layout }
    }

    fn is_within_bounds(&self, register: MemoryAddress) -> bool {
        let index = register.unwrap_direct();
        index >= self.start() && index < self.end()
    }

    pub(super) fn start() -> usize {
        ReservedRegisters::len()
    }

    pub(super) fn end_with_layout(layout: &LayoutConfig) -> usize {
        ReservedRegisters::len() + layout.max_scratch_space()
    }
}

impl RegisterAllocator for ScratchSpace {
    fn start(&self) -> usize {
        Self::start()
    }

    fn end(&self) -> usize {
        Self::end_with_layout(&self.layout)
    }

    fn ensure_register_is_allocated(&mut self, register: MemoryAddress) {
        assert!(self.is_within_bounds(register), "Register out of scratch space bounds");
        self.storage.ensure_register_is_allocated(register.unwrap_direct());
    }

    fn allocate_register(&mut self) -> MemoryAddress {
        let allocated = MemoryAddress::direct(self.storage.allocate_register());
        assert!(self.is_within_bounds(allocated), "Scratch space too deep");
        allocated
    }

    fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.storage.deallocate_register(register_index.unwrap_direct());
    }

    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self {
        let mock = Self::new(layout);
        for register in &preallocated_registers {
            assert!(mock.is_within_bounds(*register), "Register out of scratch space bounds");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                mock.start(),
                vecmap(preallocated_registers, |r| r.unwrap_direct()),
            ),
            layout,
        }
    }

    fn empty_registers_start(&self) -> MemoryAddress {
        MemoryAddress::direct(self.storage.empty_registers_start(self.start()))
    }

    fn layout(&self) -> LayoutConfig {
        self.layout
    }
}

/// Globals have a separate memory space
/// This memory space is initialized once at the beginning of a program
/// and is read-only.
#[derive(Default)]
pub(crate) struct GlobalSpace {
    storage: DeallocationListAllocator,
    max_memory_address: usize,
    layout: LayoutConfig,
}

impl GlobalSpace {
    pub(crate) fn new(layout: LayoutConfig) -> Self {
        let start = Self::start_with_layout(&layout);
        Self { storage: DeallocationListAllocator::new(start), max_memory_address: start, layout }
    }

    fn is_within_bounds(&self, register: MemoryAddress) -> bool {
        let index = register.unwrap_direct();
        index >= self.start()
    }

    fn update_max_address(&mut self, register: MemoryAddress) {
        let index = register.unwrap_direct();
        assert!(index >= self.start(), "Global space malformed");
        if index > self.max_memory_address {
            self.max_memory_address = index;
        }
    }

    pub(super) fn max_memory_address(&self) -> usize {
        self.max_memory_address
    }

    /// Computes the first valid memory address for global space
    pub(crate) fn start_with_layout(layout: &LayoutConfig) -> usize {
        ScratchSpace::end_with_layout(layout)
    }
}

impl RegisterAllocator for GlobalSpace {
    fn start(&self) -> usize {
        Self::start_with_layout(&self.layout)
    }

    fn end(&self) -> usize {
        unreachable!("The global space is set by the program");
    }

    fn allocate_register(&mut self) -> MemoryAddress {
        let allocated = MemoryAddress::direct(self.storage.allocate_register());
        self.update_max_address(allocated);
        allocated
    }

    fn deallocate_register(&mut self, register_index: MemoryAddress) {
        self.storage.deallocate_register(register_index.unwrap_direct());
    }

    fn ensure_register_is_allocated(&mut self, register: MemoryAddress) {
        self.update_max_address(register);
        self.storage.ensure_register_is_allocated(register.unwrap_direct());
    }

    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self {
        let mock = Self::new(layout);
        for register in &preallocated_registers {
            assert!(mock.is_within_bounds(*register), "Register out of global space bounds");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                mock.start(),
                vecmap(preallocated_registers, |r| r.unwrap_direct()),
            ),
            max_memory_address: mock.start(),
            layout,
        }
    }

    fn empty_registers_start(&self) -> MemoryAddress {
        MemoryAddress::direct(self.storage.empty_registers_start(self.start()))
    }

    fn layout(&self) -> LayoutConfig {
        self.layout
    }
}

#[derive(Default)]
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
        let layout = self.registers.layout();
        self.registers = Registers::from_preallocated_registers(allocated_registers, layout);
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
    use crate::brillig::brillig_ir::{
        LayoutConfig,
        registers::{RegisterAllocator, Stack},
    };

    #[test]
    fn stack_should_prioritize_returning_low_registers() {
        let mut stack = Stack::new(LayoutConfig::default());
        let one = stack.allocate_register();
        let _two = stack.allocate_register();
        let three = stack.allocate_register();

        stack.deallocate_register(three);
        stack.deallocate_register(one);

        let one_again = stack.allocate_register();
        assert_eq!(one, one_again);
    }
}
