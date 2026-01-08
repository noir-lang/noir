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
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeSet,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use acvm::acir::brillig::{HeapArray, HeapVector, MemoryAddress, lengths::SemiFlattenedLength};
use iter_extended::vecmap;
use smallvec::{SmallVec, smallvec};

use crate::brillig::brillig_ir::{
    BRILLIG_MEMORY_ADDRESSING_BIT_SIZE,
    brillig_variable::{BrilligArray, BrilligVector},
};

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

    /// The maximum size of an individual stack frame.
    pub(crate) fn max_stack_frame_size(&self) -> usize {
        self.max_stack_frame_size
    }

    /// The overall maximum stack size is the maximum number of frames times the maximum size of an individual stack frame.
    pub(crate) fn max_stack_size(&self) -> usize {
        self.max_stack_size
    }

    pub(crate) fn max_scratch_space(&self) -> usize {
        self.max_scratch_space
    }

    /// Start of the entry point region:
    /// `{reserved} {scratch space} {globals} [call data]`
    ///
    /// Returns the point where `call data` can start.
    pub(crate) fn entry_point_start(&self, globals_size: usize) -> usize {
        ScratchSpace::end_with_layout(self) + globals_size
    }

    /// Start of the return data within the entry point region:
    /// `{reserved} {scratch space} {globals} {call data} [return data]`
    pub(crate) fn return_data_start(&self, globals_size: usize, calldata_size: usize) -> usize {
        self.entry_point_start(globals_size) + calldata_size
    }
}

// These constants represent expert chosen defaults that are appropriate for the majority of programs
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
    fn deallocate_register(&mut self, register: MemoryAddress);
    /// Ensures a register is allocated, allocating it if necessary.
    fn ensure_register_is_allocated(&mut self, register: MemoryAddress);
    /// Creates a new register context from a set of registers allocated previously.
    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self;
    /// Finds the first register which is followed only by free registers.
    ///
    /// Always returns a [MemoryAddress::Relative] address.
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
        Self { storage: DeallocationListAllocator::new(Self::start()), layout }
    }

    /// Check if a `Relative` address is within the bounds of the stack.
    ///
    /// Panics if the address is `Direct`.
    fn is_within_bounds(&self, register: MemoryAddress) -> bool {
        let offset = register.unwrap_relative();
        offset >= self.start() && offset < self.end()
    }

    /// Static start address.
    ///
    /// The addressable space starts at offset 1; at offset 0 is the previous stack pointer.
    pub(super) fn start() -> usize {
        1
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

    fn deallocate_register(&mut self, register: MemoryAddress) {
        self.storage.deallocate_register(register.unwrap_relative());
    }

    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self {
        let empty = Stack::new(layout);
        for register in &preallocated_registers {
            assert!(empty.is_within_bounds(*register), "Register out of stack bounds: {register}");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                empty.start(),
                vecmap(preallocated_registers, |r| r.unwrap_relative()),
            ),
            layout,
        }
    }

    fn empty_registers_start(&self) -> MemoryAddress {
        MemoryAddress::relative(self.storage.empty_registers_start())
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

    /// Check if a `Direct` address is within the bounds of the scratch space.
    ///
    /// Panics if the address is `Relative`.
    fn is_within_bounds(&self, register: MemoryAddress) -> bool {
        let index = register.unwrap_direct();
        index >= self.start() && index < self.end()
    }

    /// Return the end of `{reserved}` where `{scratch space}` can start.
    pub(super) fn start() -> usize {
        ReservedRegisters::len()
    }

    /// Return the end of `{reserved} {scratch space}`
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

    fn deallocate_register(&mut self, register: MemoryAddress) {
        self.storage.deallocate_register(register.unwrap_direct());
    }

    fn from_preallocated_registers(
        preallocated_registers: Vec<MemoryAddress>,
        layout: LayoutConfig,
    ) -> Self {
        let empty = Self::new(layout);
        for register in &preallocated_registers {
            assert!(empty.is_within_bounds(*register), "Register out of scratch space bounds");
        }

        Self {
            storage: DeallocationListAllocator::from_preallocated_registers(
                empty.start(),
                vecmap(preallocated_registers, |r| r.unwrap_direct()),
            ),
            layout,
        }
    }

    fn empty_registers_start(&self) -> MemoryAddress {
        MemoryAddress::direct(self.storage.empty_registers_start())
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

    /// Expand the global space to fit a new register if necessary.
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

    /// Computes the first valid memory address for global space.
    ///
    /// The `{global space}` starts after the `{scratch space}`.
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

    fn deallocate_register(&mut self, register: MemoryAddress) {
        self.storage.deallocate_register(register.unwrap_direct());
    }

    fn ensure_register_is_allocated(&mut self, register: MemoryAddress) {
        self.update_max_address(register);
        self.storage.ensure_register_is_allocated(register.unwrap_direct());
    }

    fn from_preallocated_registers(
        _preallocated_registers: Vec<MemoryAddress>,
        _layout: LayoutConfig,
    ) -> Self {
        unimplemented!("`GlobalSpace` does not implement `from_preallocated_registers")
    }

    fn empty_registers_start(&self) -> MemoryAddress {
        MemoryAddress::direct(self.storage.empty_registers_start())
    }

    fn layout(&self) -> LayoutConfig {
        self.layout
    }
}

/// Register allocator that tracks a contiguous range of register slots using:
/// * the highest ever allocated register slot, and
/// * the set of deallocated registers.
///
/// When asked to allocate a new register, it returns a deallocated one,
/// or expands the high water mark of registers.
#[derive(Default)]
struct DeallocationListAllocator {
    /// A free-list of registers that have been deallocated and can be used again.
    deallocated_registers: BTreeSet<usize>,
    /// A usize indicating the next unused register.
    next_free_register_index: usize,
    /// Original start index.
    start_register_index: usize,
}

impl DeallocationListAllocator {
    /// Create a new allocator, starting a specific slot (could be direct or relative).
    fn new(start: usize) -> Self {
        Self {
            deallocated_registers: BTreeSet::new(),
            next_free_register_index: start,
            start_register_index: start,
        }
    }

    /// Ensure that a register index is allocated by:
    /// * expanding the range, if the index is beyond the current maximum, or
    /// * removing it from the deallocated registers.
    ///
    /// If the register is already allocated (ie. it fits and isn't deallocated) then nothing happens.
    fn ensure_register_is_allocated(&mut self, index: usize) {
        if index < self.next_free_register_index {
            // If it could be allocated, check if it's in the deallocated list and remove it from there
            self.deallocated_registers.remove(&index);
        } else {
            // If it couldn't yet be, expand the register space.
            self.next_free_register_index = index + 1;
        }
    }

    /// Returns the first deallocated register, or expands the range to fit a new one.
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

    /// Adds the register to the deallocation list.
    ///
    /// Panics if:
    /// * The register is already deallocated.
    /// * The register is not within the tracked range.
    fn deallocate_register(&mut self, register_index: usize) {
        assert!(
            !self.deallocated_registers.contains(&register_index),
            "register already deallocated"
        );
        assert!(register_index < self.next_free_register_index, "deallocate after next free");
        assert!(register_index >= self.start_register_index, "deallocate before start");
        self.deallocated_registers.insert(register_index);
    }

    /// Create an allocator that ranges from `start` to just after the highest
    /// preallocated register, with everything not on that list considered deallocated
    /// and ready to be used.
    fn from_preallocated_registers(start: usize, preallocated_registers: Vec<usize>) -> Self {
        let next_free_register_index = preallocated_registers.iter().fold(
            start,
            |free_register_index, &preallocated_register| {
                free_register_index.max(preallocated_register + 1)
            },
        );

        let mut deallocated_registers = (start..next_free_register_index).collect::<BTreeSet<_>>();
        for pr in preallocated_registers {
            deallocated_registers.remove(&pr);
        }

        Self { deallocated_registers, next_free_register_index, start_register_index: start }
    }

    /// Find the first register after which there are only free registers.
    fn empty_registers_start(&self) -> usize {
        let mut first_free = self.next_free_register_index;
        while first_free > self.start_register_index {
            if !self.deallocated_registers.contains(&(first_free - 1)) {
                break;
            }
            first_free -= 1;
        }
        first_free
    }
}

impl<F, Registers: RegisterAllocator> BrilligContext<F, Registers> {
    /// Read-only reference to the registers.
    pub(crate) fn registers(&self) -> Ref<Registers> {
        self.registers.borrow()
    }

    /// Read-write reference to the registers.
    pub(crate) fn registers_mut(&self) -> RefMut<Registers> {
        self.registers.borrow_mut()
    }

    /// Manually deallocates a register which is no longer in use.
    ///
    /// This could be one of the pre-allocated ones which doesn't have automation.
    pub(crate) fn deallocate_register(&mut self, register: MemoryAddress) {
        self.registers_mut().deallocate_register(register);
    }

    /// Allocates an unused register.
    pub(crate) fn allocate_register(&mut self) -> Allocated<MemoryAddress, Registers> {
        let addr = self.registers_mut().allocate_register();
        Allocated::new_addr(addr, self.registers.clone())
    }

    /// Resets the registers to a new list of allocated ones.
    pub(crate) fn set_allocated_registers(&mut self, allocated_registers: Vec<MemoryAddress>) {
        self.registers = Rc::new(RefCell::new(Registers::from_preallocated_registers(
            allocated_registers,
            self.layout(),
        )));
    }

    /// Allocate a [SingleAddrVariable].
    pub(crate) fn allocate_single_addr(
        &mut self,
        bit_size: u32,
    ) -> Allocated<SingleAddrVariable, Registers> {
        self.allocate_register().map(|a| SingleAddrVariable::new(a, bit_size))
    }

    /// Allocate a [SingleAddrVariable] with the size of a Brillig memory address.
    pub(crate) fn allocate_single_addr_usize(
        &mut self,
    ) -> Allocated<SingleAddrVariable, Registers> {
        self.allocate_register().map(SingleAddrVariable::new_usize)
    }

    /// Allocate a [SingleAddrVariable] with a size of 1 bit.
    pub(crate) fn allocate_single_addr_bool(&mut self) -> Allocated<SingleAddrVariable, Registers> {
        self.allocate_single_addr(1)
    }

    /// Allocate a [SingleAddrVariable] with a size of `BRILLIG_MEMORY_ADDRESSING_BIT_SIZE` bit.
    #[allow(unused)]
    pub(crate) fn allocate_single_addr_mem(&mut self) -> Allocated<SingleAddrVariable, Registers> {
        self.allocate_single_addr(BRILLIG_MEMORY_ADDRESSING_BIT_SIZE)
    }

    /// Allocate a pointer for [BrilligVector].
    ///
    /// This does not include allocating memory for the data on the heap or shaping the meta-data.
    /// That is done by [BrilligContext::codegen_initialize_vector].
    pub(crate) fn allocate_brillig_vector(&mut self) -> Allocated<BrilligVector, Registers> {
        self.allocate_register().map(|a| BrilligVector { pointer: a })
    }

    /// Allocate a pointer for [BrilligArray].
    ///
    /// This does not include allocating memory for the data on the heap or shaping the meta-data.
    /// That is done by [BrilligContext::codegen_initialize_array].
    pub(crate) fn allocate_brillig_array(
        &mut self,
        size: usize,
    ) -> Allocated<BrilligArray, Registers> {
        self.allocate_register().map(|a| BrilligArray { pointer: a, size })
    }

    /// Allocate a [HeapVector].
    pub(crate) fn allocate_heap_vector(&mut self) -> Allocated<HeapVector, Registers> {
        let pointer = self.allocate_register();
        let size = self.allocate_register();
        pointer.map2(size, |pointer, size| HeapVector { pointer, size })
    }

    /// Allocate a [HeapArray].
    pub(crate) fn allocate_heap_array(&mut self, size: usize) -> Allocated<HeapArray, Registers> {
        let size = SemiFlattenedLength(size);
        self.allocate_register().map(|pointer| HeapArray { pointer, size })
    }

    /// Create a number of consecutive [MemoryAddress::Direct] addresses at the start of the [ScratchSpace].
    pub(crate) fn make_scratch_registers<const N: usize>(&self) -> [MemoryAddress; N] {
        let scratch_start = ScratchSpace::start();
        std::array::from_fn(|i| MemoryAddress::direct(scratch_start + i))
    }
}

impl<F> BrilligContext<F, ScratchSpace> {
    /// Allocate a number of consecutive scratch registers and replace the current allocator with
    /// one that has the new registers pre-allocated.
    pub(crate) fn allocate_scratch_registers<const N: usize>(&mut self) -> [MemoryAddress; N] {
        let registers = self.make_scratch_registers();
        self.set_allocated_registers(registers.iter().copied().collect());
        registers
    }
}

/// Wrapper for a memory address which automatically deallocates itself
/// when it goes out of scope.
pub(crate) struct Allocated<T, R: RegisterAllocator> {
    /// The inner structure is optional, so that we can map it to a different value
    /// and let the original wrapper be dropped without any effect.
    inner: Option<AllocatedInner<T, R>>,
}

/// Address list with enough slots for `HeapVector`.
type AddressList = SmallVec<[MemoryAddress; 2]>;

struct AllocatedInner<A, R> {
    /// The value that we actually wanted to use, holding the allocated `MemoryAddress`.
    value: A,
    /// Addresses allocated.
    addresses: AddressList,
    /// Reference to the registers, for deallocation.
    /// Optional so that pure values don't have to clone the registers.
    registers: Option<Rc<RefCell<R>>>,
}

impl<A, R: RegisterAllocator> Drop for Allocated<A, R> {
    fn drop(&mut self) {
        let Some(mut inner) = self.inner.take() else {
            return;
        };
        let Some(registers) = inner.registers.take() else {
            return;
        };
        let addresses = std::mem::take(&mut inner.addresses);
        let mut registers = registers.borrow_mut();
        for addr in addresses {
            registers.deallocate_register(addr);
        }
    }
}

/// Construct a new allocated address.
impl<R: RegisterAllocator> Allocated<MemoryAddress, R> {
    /// Wrap single allocated memory address.
    fn new_addr(addr: MemoryAddress, registers: Rc<RefCell<R>>) -> Self {
        Self::new(addr, smallvec![addr], registers)
    }
}

impl<A, R: RegisterAllocator> Allocated<A, R> {
    /// Wrap a value in [Allocated] without deallocating any address at the end.
    pub(crate) fn pure(value: A) -> Self {
        Self::from_inner(AllocatedInner { value, addresses: smallvec![], registers: None })
    }

    /// Create an [Allocated] value that deallocates its associated addresses at the end.
    pub(crate) fn new(value: A, addresses: AddressList, registers: Rc<RefCell<R>>) -> Self {
        Self::from_inner(AllocatedInner { value, addresses, registers: Some(registers) })
    }

    fn from_inner(inner: AllocatedInner<A, R>) -> Self {
        Self { inner: Some(inner) }
    }

    /// Take the inner, leaving `self` ready to be dropped without any effect.
    fn into_inner(mut self) -> AllocatedInner<A, R> {
        self.inner.take().expect("allocated value already taken")
    }

    fn inner(&self) -> &AllocatedInner<A, R> {
        self.inner.as_ref().expect("allocated value already taken")
    }

    fn inner_mut(&mut self) -> &mut AllocatedInner<A, R> {
        self.inner.as_mut().expect("allocated value already taken")
    }

    /// Map the `value` to something else.
    ///
    /// The deallocation of addresses is deferred to when the new value goes out of scope.
    pub(crate) fn map<B>(self, f: impl FnOnce(A) -> B) -> Allocated<B, R> {
        let inner = self.into_inner();
        Allocated::from_inner(AllocatedInner {
            value: f(inner.value),
            addresses: inner.addresses,
            registers: inner.registers,
        })
    }

    /// Map two values into one.
    ///
    /// The addresses of the two values are merged, with the implicit assumption that the
    /// underlying registers are the same.
    pub(crate) fn map2<B>(self, other: Self, f: impl FnOnce(A, A) -> B) -> Allocated<B, R> {
        let inner = self.into_inner();
        let other = other.into_inner();
        Allocated::from_inner(AllocatedInner {
            value: f(inner.value, other.value),
            addresses: Self::merge_addresses(inner.addresses, other.addresses),
            registers: inner.registers.or(other.registers),
        })
    }

    /// Map the `value` to something else that involves allocation.
    ///
    /// The resulting value keeps both addresses alive.
    pub(crate) fn and_then<B>(self, f: impl FnOnce(A) -> Allocated<B, R>) -> Allocated<B, R> {
        let inner = self.into_inner();
        let other = f(inner.value).into_inner();
        Allocated::from_inner(AllocatedInner {
            value: other.value,
            addresses: Self::merge_addresses(inner.addresses, other.addresses),
            registers: inner.registers.or(other.registers),
        })
    }

    fn merge_addresses(mut a: AddressList, mut b: AddressList) -> AddressList {
        a.append(&mut b);
        a.sort();
        a.dedup();
        a
    }

    /// Manually deallocate a register when it's no longer needed.
    pub(crate) fn deallocate(self) {
        drop(self);
    }

    /// Detach the allocated value from the registers, promising to take care of ownership in a different way.
    ///
    /// This is used when we allocate an address in a register, then create a new register where we consider the
    /// value pre-allocated, and finally want to deallocate the address in the new register, not the old one,
    /// so that we can reuse the memory for something else.
    pub(crate) fn detach(self) -> A {
        self.into_inner().value
    }
}

impl<A, R: RegisterAllocator> Deref for Allocated<A, R> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.inner().value
    }
}

impl<A, R: RegisterAllocator> DerefMut for Allocated<A, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner_mut().value
    }
}

#[cfg(test)]
mod tests {
    use crate::brillig::brillig_ir::{
        LayoutConfig,
        registers::{DeallocationListAllocator, RegisterAllocator, Stack},
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

    #[test]
    #[should_panic]
    fn deallocation_list_deallocate_before_start() {
        let mut allocator = DeallocationListAllocator::new(10);
        allocator.deallocate_register(5);
    }

    #[test]
    #[should_panic]
    fn deallocation_list_deallocate_after_end() {
        let mut allocator = DeallocationListAllocator::new(10);
        allocator.deallocate_register(15);
    }

    #[test]
    #[should_panic]
    fn deallocation_list_deallocate_twice() {
        let mut allocator = DeallocationListAllocator::from_preallocated_registers(0, vec![10]);
        allocator.deallocate_register(5);
    }
}
