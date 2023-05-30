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

    pub(crate) fn with_default(size: usize, default: AcirVar) -> Array {
        Self { elements: vec![Some(default); size] }
    }
}

#[derive(Debug, Default)]
/// Memory used to represent Arrays
pub(crate) struct Memory {
    arrays: Vec<Array>,
}

impl Memory {
    pub(crate) fn new() -> Self {
        Self { arrays: Vec::new() }
    }

    /// Allocates an array of size `size`.
    /// The elements in the array are not zero initialized
    ///
    /// TODO: Check if this method is needed, ie allocation without a default
    pub(crate) fn allocate(&mut self, size: usize) -> ArrayId {
        let array = Array::new(size);
        self.add_array(array)
    }
    pub(crate) fn allocate_with_default(
        &mut self,
        size: usize,
        default_element: AcirVar,
    ) -> ArrayId {
        let array = Array::with_default(size, default_element);
        self.add_array(array)
    }

    fn add_array(&mut self, array: Array) -> ArrayId {
        let id = self.arrays.len();
        self.arrays.push(array);
        ArrayId(id)
    }

    fn mut_array(&mut self, array_id: ArrayId) -> &mut Array {
        &mut self.arrays[array_id.0]
    }

    /// Sets an element at the array that `ArrayId` points to.
    /// The index must be constant in the Noir program.
    pub(crate) fn constant_set(
        &mut self,
        array_id: ArrayId,
        index: usize,
        element: AcirVar,
    ) -> Result<(), AcirGenError> {
        // Check if the index is larger than the array size
        let array = self.mut_array(array_id);
        let array_size = array.size();

        if index >= array_size {
            return Err(AcirGenError::IndexOutOfBounds { index, array_size });
        }

        array.elements[index] = Some(element);

        Ok(())
    }

    /// Gets an element at the array that `ArrayId` points to.
    /// The index must be constant in the Noir program.
    pub(crate) fn constant_get(
        &mut self,
        array_id: ArrayId,
        index: usize,
    ) -> Result<AcirVar, AcirGenError> {
        // Check if the index is larger than the array size
        let array = self.mut_array(array_id);
        let array_size = array.size();

        if index >= array_size {
            return Err(AcirGenError::IndexOutOfBounds { index, array_size });
        }

        let element = array.elements[index];

        match element {
            Some(element) => Ok(element),
            None => {
                // The element was never initialized
                Err(AcirGenError::UninitializedElementInArray { index, array_id })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArrayId(usize);

#[cfg(test)]
mod tests {
    use crate::ssa_refactor::acir_gen::acir_ir::{errors::AcirGenError, memory::Memory};

    #[test]
    fn smoke_api_get_uninitialized_element_out() {
        let mut memory = Memory::new();

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
        let mut memory = Memory::new();

        let array_size = 10;
        let array_id = memory.allocate(array_size);

        let element = memory.constant_get(array_id, array_size);
        // Should get an index out of bounds error
        assert_eq!(element, Err(AcirGenError::IndexOutOfBounds { index: array_size, array_size }));
    }
}
