use crate::ssa_refactor::ir::map::{DenseMap, Id};

use super::{acir_variable::AcirVar, errors::AcirGenError};

#[derive(Debug, Default)]
pub(crate) struct Array {
    /// Elements in the array.
    /// None represents the element not being set yet
    ///
    /// The size of this vector cannot be changed once set.
    /// This follows the behavior of an Array as opposed to a Vector.
    elements: Vec<Option<AcirVar>>,
}

impl Array {
    pub(crate) fn size(&self) -> usize {
        self.elements.len()
    }

    pub(crate) fn new(size: usize) -> Array {
        Self { elements: vec![None; size] }
    }
}

#[derive(Debug, Default)]
/// Memory used to represent Arrays
pub(crate) struct Memory {
    arrays: DenseMap<Array>,
}

impl Memory {
    /// Allocates an array of size `size`.
    /// The elements in the array are not zero initialized
    pub(crate) fn allocate(&mut self, size: usize) -> ArrayId {
        let array = Array::new(size);
        self.arrays.insert(array)
    }

    /// Sets an element at the array that `ArrayId` points to.
    /// The index must be constant in the Noir program.
    pub(crate) fn constant_set(
        &mut self,
        array_id: ArrayId,
        index: usize,
        element: AcirVar,
    ) -> Result<(), AcirGenError> {
        let array = &mut self.arrays[array_id];
        Self::check_bounds(index, array.size())?;

        array.elements[index] = Some(element);

        Ok(())
    }

    /// Gets an element at the array that `ArrayId` points to.
    /// The index must be constant in the Noir program.
    pub(crate) fn constant_get(
        &self,
        array_id: ArrayId,
        index: usize,
    ) -> Result<AcirVar, AcirGenError> {
        let array = &self.arrays[array_id];
        Self::check_bounds(index, array.size())?;

        array.elements[index].ok_or(AcirGenError::UninitializedElementInArray { index, array_id })
    }

    /// Gets all elements at the array that `ArrayId` points to.
    ///
    /// This returns an error if any of the array's elements have not been initialized.
    pub(crate) fn constant_get_all(&self, array_id: ArrayId) -> Result<Vec<AcirVar>, AcirGenError> {
        let array = &self.arrays[array_id];
        let mut elements = Vec::new();
        for index in 0..array.size() {
            elements.push(self.constant_get(array_id, index)?);
        }
        Ok(elements)
    }

    /// Check if the index is larger than the array size
    fn check_bounds(index: usize, array_size: usize) -> Result<(), AcirGenError> {
        if index < array_size {
            Ok(())
        } else {
            Err(AcirGenError::IndexOutOfBounds { index, array_size })
        }
    }
}

/// Pointer to an allocated `Array`
pub(crate) type ArrayId = Id<Array>;

#[cfg(test)]
mod tests {
    use crate::ssa_refactor::acir_gen::acir_ir::{errors::AcirGenError, memory::Memory};

    #[test]
    fn smoke_api_get_uninitialized_element_out() {
        let mut memory = Memory::default();

        let array_size = 10;
        let index = 0;

        let array_id = memory.allocate(array_size);

        let element = memory.constant_get(array_id, index);
        // Should get an error because we are trying to get an element which has not been initialized
        // yet.
        assert_eq!(element, Err(AcirGenError::UninitializedElementInArray { index, array_id }));
    }
    #[test]
    fn smoke_api_out_of_bounds() {
        let mut memory = Memory::default();

        let array_size = 10;
        let array_id = memory.allocate(array_size);

        let element = memory.constant_get(array_id, array_size);
        // Should get an index out of bounds error
        assert_eq!(element, Err(AcirGenError::IndexOutOfBounds { index: array_size, array_size }));
    }
}
