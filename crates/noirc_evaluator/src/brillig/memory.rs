/// Simple memory allocator for brillig
/// For now it just tracks a free memory pointer
/// Will probably get smarter in the future
#[derive(Default)]
pub(crate) struct BrilligMemory {
    free_mem_pointer: usize,
}

impl BrilligMemory {
    pub(crate) fn allocate(&mut self, size: usize) -> usize {
        let start = self.free_mem_pointer;
        self.free_mem_pointer += size;
        start
    }

    pub(crate) fn pointer(&self) -> usize {
        self.free_mem_pointer
    }
}
