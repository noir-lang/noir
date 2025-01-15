use acir::circuit::{brillig::BrilligInputs, opcodes::BlockId, Circuit, Opcode};
use std::collections::HashSet;

/// `UnusedMemoryOptimizer` will remove initializations of memory blocks which are unused.
pub(crate) struct UnusedMemoryOptimizer<F> {
    unused_memory_initializations: HashSet<BlockId>,
    circuit: Circuit<F>,
}

impl<F> UnusedMemoryOptimizer<F> {
    /// Creates a new `UnusedMemoryOptimizer ` by collecting unused memory init
    /// opcodes from `Circuit`.
    pub(crate) fn new(circuit: Circuit<F>) -> Self {
        let unused_memory_initializations = Self::collect_unused_memory_initializations(&circuit);
        Self { circuit, unused_memory_initializations }
    }

    /// Creates a set of ids for memory blocks for which no [`Opcode::MemoryOp`]s exist.
    ///
    /// These memory blocks can be safely removed.
    fn collect_unused_memory_initializations(circuit: &Circuit<F>) -> HashSet<BlockId> {
        let mut unused_memory_initialization = HashSet::new();

        for opcode in &circuit.opcodes {
            match opcode {
                Opcode::MemoryInit { block_id, .. } => {
                    unused_memory_initialization.insert(*block_id);
                }
                Opcode::MemoryOp { block_id, .. } => {
                    unused_memory_initialization.remove(block_id);
                }
                Opcode::BrilligCall { inputs, .. } => {
                    for input in inputs {
                        if let BrilligInputs::MemoryArray(block) = input {
                            unused_memory_initialization.remove(block);
                        }
                    }
                }
                _ => (),
            }
        }
        unused_memory_initialization
    }

    /// Returns a `Circuit` where [`Opcode::MemoryInit`]s for unused memory blocks are dropped.
    pub(crate) fn remove_unused_memory_initializations(
        self,
        order_list: Vec<usize>,
    ) -> (Circuit<F>, Vec<usize>) {
        let mut new_order_list = Vec::with_capacity(order_list.len());
        let mut optimized_opcodes = Vec::with_capacity(self.circuit.opcodes.len());
        for (idx, opcode) in self.circuit.opcodes.into_iter().enumerate() {
            match opcode {
                Opcode::MemoryInit { block_id, block_type, .. }
                    if !block_type.is_databus()
                        && self.unused_memory_initializations.contains(&block_id) =>
                {
                    // Drop opcode
                }
                _ => {
                    new_order_list.push(order_list[idx]);
                    optimized_opcodes.push(opcode);
                }
            }
        }

        (Circuit { opcodes: optimized_opcodes, ..self.circuit }, new_order_list)
    }
}
