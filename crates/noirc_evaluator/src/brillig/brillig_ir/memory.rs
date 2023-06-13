use acvm::acir::brillig_vm::Value;
/// Simple memory allocator for brillig
/// For now it just tracks a free memory pointer
/// Will probably get smarter in the future
#[derive(Default)]
pub(crate) struct BrilligMemory {
    free_mem_pointer: usize,
}

/// A number representing a memory address
#[derive(Default, Clone, Copy)]
pub struct MemoryAddress(usize);

impl From<MemoryAddress> for Value {
    fn from(value: MemoryAddress) -> Self {
        Value::from(value.0)
    }
}

/// Allocation details metadata about an array allocation
pub(crate) struct Allocation {
    /// This is the starting address for an allocation
    /// and can also be seen as the array ptr
    pub(crate) start_address: MemoryAddress,
    /// This is the address of the last item allocated for
    /// this array.
    pub(crate) end_address: MemoryAddress,
}

impl BrilligMemory {
    /// Allocate an array of size `size`
    pub(crate) fn allocate(&mut self, size: usize) -> Allocation {
        let start = self.free_mem_pointer;
        self.free_mem_pointer += size;
        Allocation {
            start_address: MemoryAddress(start),
            end_address: MemoryAddress(self.free_mem_pointer - 1),
        }
    }
}
