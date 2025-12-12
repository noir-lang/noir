use acir::{
    AcirField,
    circuit::opcodes::MemOp,
    native_types::{Witness, WitnessMap},
};

use super::{ErrorLocation, OpcodeResolutionError};
use super::{arithmetic::ExpressionSolver, get_value, insert_value, witness_to_value};

type MemoryIndex = u32;

/// Maintains the state for solving [`MemoryInit`][`acir::circuit::Opcode::MemoryInit`] and [`MemoryOp`][`acir::circuit::Opcode::MemoryOp`] opcodes.
pub(crate) struct MemoryOpSolver<F> {
    /// Known values of the memory block, based on the index
    /// This vec starts as big as it needs to, when initialized,
    /// then evolves as we process the opcodes.
    pub(super) block_value: Vec<F>,
}

impl<F: AcirField> MemoryOpSolver<F> {
    /// Creates a new MemoryOpSolver with the values given in `init`.
    pub(crate) fn new(
        init: &[Witness],
        initial_witness: &WitnessMap<F>,
    ) -> Result<Self, OpcodeResolutionError<F>> {
        Ok(Self {
            block_value: init
                .iter()
                .map(|witness| witness_to_value(initial_witness, *witness).copied())
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub(crate) fn len(&self) -> u32 {
        u32::try_from(self.block_value.len()).expect("expected a length that fits into a u32")
    }

    /// Convert a field element into a memory index
    /// Only 32 bits values are valid memory indices
    pub(crate) fn index_from_field(
        &self,
        index: F,
    ) -> Result<MemoryIndex, OpcodeResolutionError<F>> {
        index.try_to_u32().ok_or_else({
            || OpcodeResolutionError::IndexOutOfBounds {
                opcode_location: ErrorLocation::Unresolved,
                index,
                array_size: self.len(),
            }
        })
    }

    /// Update the 'block_value' map with the provided index/value
    /// Returns an 'IndexOutOfBounds' error if the index is outside the block range.
    pub(crate) fn write_memory_index(
        &mut self,
        index: MemoryIndex,
        value: F,
    ) -> Result<(), OpcodeResolutionError<F>> {
        if index >= self.len() {
            return Err(OpcodeResolutionError::IndexOutOfBounds {
                opcode_location: ErrorLocation::Unresolved,
                index: F::from(u128::from(index)),
                array_size: self.len(),
            });
        }

        self.block_value[index as usize] = value;
        Ok(())
    }

    /// Returns the value stored in the 'block_value' map for the provided index
    /// Returns an 'IndexOutOfBounds' error if the index is not in the map.
    pub(crate) fn read_memory_index(
        &self,
        index: MemoryIndex,
    ) -> Result<F, OpcodeResolutionError<F>> {
        self.block_value.get(index as usize).copied().ok_or(
            OpcodeResolutionError::IndexOutOfBounds {
                opcode_location: ErrorLocation::Unresolved,
                index: F::from(u128::from(index)),
                array_size: self.len(),
            },
        )
    }

    /// Update the 'block_values' by processing the provided Memory opcode
    /// The opcode 'op' contains the index and value of the operation and the type
    /// of the operation.
    /// They are all stored as an [acir::native_types::Expression]
    /// The type of 'operation' is '0' for a read and '1' for a write. It must be a constant
    /// expression.
    /// Index is not required to be constant but it must reduce to a known value
    /// for processing the opcode. This is done by doing the (partial) evaluation of its expression,
    /// using the provided witness map.
    ///
    /// READ: read the block at index op.index and update op.value with the read value
    /// - 'op.value' must reduce to a witness (after the evaluation of its expression)
    /// - the value is updated in the provided witness map, for the 'op.value' witness
    ///
    /// WRITE: update the block at index 'op.index' with 'op.value'
    /// - 'op.value' must reduce to a known value
    ///
    /// If a requirement is not met, it returns an error.
    pub(crate) fn solve_memory_op(
        &mut self,
        op: &MemOp<F>,
        initial_witness: &mut WitnessMap<F>,
        pedantic_solving: bool,
    ) -> Result<(), OpcodeResolutionError<F>> {
        let operation = get_value(&op.operation, initial_witness)?;

        // Find the memory index associated with this memory operation.
        let index = get_value(&op.index, initial_witness)?;
        let memory_index = self.index_from_field(index)?;

        // Calculate the value associated with this memory operation.
        //
        // In read operations, this corresponds to the witness index at which the value from memory will be written.
        // In write operations, this corresponds to the expression which will be written to memory.
        let value = ExpressionSolver::evaluate(&op.value, initial_witness);

        // `operation == 0` implies a read operation. (`operation == 1` implies write operation).
        let is_read_operation = operation.is_zero();
        if pedantic_solving {
            // We expect that the 'operation' should resolve to either 0 or 1.
            if !is_read_operation && !operation.is_one() {
                let opcode_location = ErrorLocation::Unresolved;
                return Err(OpcodeResolutionError::MemoryOperationLargerThanOne {
                    opcode_location,
                    operation,
                });
            }
        }

        if is_read_operation {
            // `value_read = arr[memory_index]`
            //
            // This is the value that we want to read into; i.e. copy from the memory block
            // into this value.
            let value_read_witness = value.to_witness().expect(
                "Memory must be read into a specified witness index, encountered an Expression",
            );

            let value_in_array = self.read_memory_index(memory_index)?;
            insert_value(&value_read_witness, value_in_array, initial_witness)
        } else {
            // `arr[memory_index] = value_write`
            //
            // This is the value that we want to write into; i.e. copy from `value_write`
            // into the memory block.
            let value_write = value;

            let value_to_write = get_value(&value_write, initial_witness)?;
            self.write_memory_index(memory_index, value_to_write)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        AcirField, FieldElement,
        circuit::opcodes::MemOp,
        native_types::{Witness, WitnessMap},
    };

    use super::MemoryOpSolver;

    #[test]
    fn test_solver() {
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));

        let init = vec![Witness(1), Witness(2)];
        // Write the value '2' at index '1', and then read from index '1' into witness 4
        let trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::one().into(), Witness(4)),
        ];

        let mut block_solver = MemoryOpSolver::new(&init, &initial_witness).unwrap();

        for op in trace {
            let pedantic_solving = true;
            block_solver.solve_memory_op(&op, &mut initial_witness, pedantic_solving).unwrap();
        }

        assert_eq!(initial_witness[&Witness(4)], FieldElement::from(2u128));
    }

    #[test]
    fn test_index_out_of_bounds() {
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));

        let init = vec![Witness(1), Witness(2)];
        // Write at index '1', and then read at index '2' on an array of size 2.
        let invalid_trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::from(2u128).into(), Witness(4)),
        ];
        let mut block_solver = MemoryOpSolver::new(&init, &initial_witness).unwrap();
        let mut err = None;
        for op in invalid_trace {
            if err.is_none() {
                let pedantic_solving = true;
                err =
                    block_solver.solve_memory_op(&op, &mut initial_witness, pedantic_solving).err();
            }
        }

        assert!(matches!(
            err,
            Some(crate::pwg::OpcodeResolutionError::IndexOutOfBounds {
                opcode_location: _,
                index,
                array_size: 2
            }) if index == FieldElement::from(2u128)
        ));
    }
}
