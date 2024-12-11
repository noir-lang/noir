use std::collections::HashMap;

use acir::{
    circuit::opcodes::MemOp,
    native_types::{Expression, Witness, WitnessMap},
    AcirField,
};

use super::{
    arithmetic::ExpressionSolver, get_value, insert_value, is_predicate_false, witness_to_value,
};
use super::{ErrorLocation, OpcodeResolutionError};

type MemoryIndex = u32;

/// Maintains the state for solving [`MemoryInit`][`acir::circuit::Opcode::MemoryInit`] and [`MemoryOp`][`acir::circuit::Opcode::MemoryOp`] opcodes.
#[derive(Default)]
pub(crate) struct MemoryOpSolver<F> {
    pub(super) block_value: HashMap<MemoryIndex, F>,
    pub(super) block_len: u32,
}

impl<F: AcirField> MemoryOpSolver<F> {
    fn write_memory_index(
        &mut self,
        index: MemoryIndex,
        value: F,
    ) -> Result<(), OpcodeResolutionError<F>> {
        if index >= self.block_len {
            return Err(OpcodeResolutionError::IndexOutOfBounds {
                opcode_location: ErrorLocation::Unresolved,
                index,
                array_size: self.block_len,
            });
        }
        self.block_value.insert(index, value);
        Ok(())
    }

    fn read_memory_index(&self, index: MemoryIndex) -> Result<F, OpcodeResolutionError<F>> {
        self.block_value.get(&index).copied().ok_or(OpcodeResolutionError::IndexOutOfBounds {
            opcode_location: ErrorLocation::Unresolved,
            index,
            array_size: self.block_len,
        })
    }

    /// Set the block_value from a MemoryInit opcode
    pub(crate) fn init(
        &mut self,
        init: &[Witness],
        initial_witness: &WitnessMap<F>,
    ) -> Result<(), OpcodeResolutionError<F>> {
        self.block_len = init.len() as u32;
        for (memory_index, witness) in init.iter().enumerate() {
            self.write_memory_index(
                memory_index as MemoryIndex,
                *witness_to_value(initial_witness, *witness)?,
            )?;
        }
        Ok(())
    }

    pub(crate) fn solve_memory_op(
        &mut self,
        op: &MemOp<F>,
        initial_witness: &mut WitnessMap<F>,
        predicate: &Option<Expression<F>>,
        pedantic_solving: bool,
    ) -> Result<(), OpcodeResolutionError<F>> {
        let operation = get_value(&op.operation, initial_witness)?;

        // Find the memory index associated with this memory operation.
        let index = get_value(&op.index, initial_witness)?;
        let memory_index = index.try_to_u64().unwrap() as MemoryIndex;

        // Calculate the value associated with this memory operation.
        //
        // In read operations, this corresponds to the witness index at which the value from memory will be written.
        // In write operations, this corresponds to the expression which will be written to memory.
        let value = ExpressionSolver::evaluate(&op.value, initial_witness);

        // `operation == 0` implies a read operation. (`operation == 1` implies write operation).
        let is_read_operation = operation.is_zero();

        // Fetch whether or not the predicate is false (e.g. equal to zero)
        let opcode_location = ErrorLocation::Unresolved;
        let skip_operation =
            is_predicate_false(initial_witness, predicate, pedantic_solving, &opcode_location)?;

        if is_read_operation {
            // `value_read = arr[memory_index]`
            //
            // This is the value that we want to read into; i.e. copy from the memory block
            // into this value.
            let value_read_witness = value.to_witness().expect(
                "Memory must be read into a specified witness index, encountered an Expression",
            );

            // A zero predicate indicates that we should skip the read operation
            // and zero out the operation's output.
            let value_in_array =
                if skip_operation { F::zero() } else { self.read_memory_index(memory_index)? };
            insert_value(&value_read_witness, value_in_array, initial_witness)
        } else {
            // `arr[memory_index] = value_write`
            //
            // This is the value that we want to write into; i.e. copy from `value_write`
            // into the memory block.
            let value_write = value;

            // A zero predicate indicates that we should skip the write operation.
            if skip_operation {
                // We only want to write to already initialized memory.
                // Do nothing if the predicate is zero.
                Ok(())
            } else {
                let value_to_write = get_value(&value_write, initial_witness)?;
                self.write_memory_index(memory_index, value_to_write)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        circuit::opcodes::MemOp,
        native_types::{Expression, Witness, WitnessMap},
        AcirField, FieldElement,
    };

    use super::MemoryOpSolver;

    // use pedantic_solving for tests
    const PEDANTIC_SOLVING: bool = true;

    #[test]
    fn test_solver() {
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));

        let init = vec![Witness(1), Witness(2)];

        let trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::one().into(), Witness(4)),
        ];

        let mut block_solver = MemoryOpSolver::default();
        block_solver.init(&init, &initial_witness).unwrap();

        for op in trace {
            block_solver
                .solve_memory_op(&op, &mut initial_witness, &None, PEDANTIC_SOLVING)
                .unwrap();
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

        let invalid_trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::from(2u128).into(), Witness(4)),
        ];
        let mut block_solver = MemoryOpSolver::default();
        block_solver.init(&init, &initial_witness).unwrap();
        let mut err = None;
        for op in invalid_trace {
            if err.is_none() {
                err = block_solver
                    .solve_memory_op(&op, &mut initial_witness, &None, PEDANTIC_SOLVING)
                    .err();
            }
        }

        assert!(matches!(
            err,
            Some(crate::pwg::OpcodeResolutionError::IndexOutOfBounds {
                opcode_location: _,
                index: 2,
                array_size: 2
            })
        ));
    }

    #[test]
    fn test_predicate_on_read() {
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));

        let init = vec![Witness(1), Witness(2)];

        let invalid_trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(1u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::from(2u128).into(), Witness(4)),
        ];
        let mut block_solver = MemoryOpSolver::default();
        block_solver.init(&init, &initial_witness).unwrap();
        let mut err = None;
        for op in invalid_trace {
            if err.is_none() {
                err = block_solver
                    .solve_memory_op(
                        &op,
                        &mut initial_witness,
                        &Some(Expression::zero()),
                        PEDANTIC_SOLVING,
                    )
                    .err();
            }
        }

        // Should have no index out of bounds error where predicate is zero
        assert_eq!(err, None);
        // The result of a read under a zero predicate should be zero
        assert_eq!(initial_witness[&Witness(4)], FieldElement::from(0u128));
    }

    #[test]
    fn test_predicate_on_write() {
        let mut initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));

        let init = vec![Witness(1), Witness(2)];

        let invalid_trace = vec![
            MemOp::write_to_mem_index(FieldElement::from(2u128).into(), Witness(3).into()),
            MemOp::read_at_mem_index(FieldElement::from(0u128).into(), Witness(4)),
            MemOp::read_at_mem_index(FieldElement::from(1u128).into(), Witness(5)),
        ];
        let mut block_solver = MemoryOpSolver::default();
        block_solver.init(&init, &initial_witness).unwrap();
        let mut err = None;
        for op in invalid_trace {
            if err.is_none() {
                err = block_solver
                    .solve_memory_op(
                        &op,
                        &mut initial_witness,
                        &Some(Expression::zero()),
                        PEDANTIC_SOLVING,
                    )
                    .err();
            }
        }

        // Should have no index out of bounds error where predicate is zero
        assert_eq!(err, None);
        // The memory under a zero predicate should be zeroed out
        assert_eq!(initial_witness[&Witness(4)], FieldElement::from(0u128));
        assert_eq!(initial_witness[&Witness(5)], FieldElement::from(0u128));
    }
}
