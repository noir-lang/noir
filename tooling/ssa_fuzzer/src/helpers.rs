#![allow(dead_code)]
use acvm::acir::native_types::Witness;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::value::ValueId;

/// Converts an Id to a Witness, to take result from WitnessMap later
pub fn id_to_witness<T>(id: Id<T>) -> Witness {
    Witness(id.to_u32())
}

/// Converts an Id to an integer, to store it for FuzzerContext
pub fn id_to_int<T>(id: Id<T>) -> u32 {
    id.to_u32()
}

/// Converts an integer to an Id of Value, to call instructions with it
pub fn u32_to_id_value(value: u32) -> ValueId {
    ValueId::new(value)
}

/// Converts an integer to an Id of BasicBlock, to call instructions with it
pub fn u32_to_id_basic_block(value: u32) -> BasicBlockId {
    BasicBlockId::new(value)
}
