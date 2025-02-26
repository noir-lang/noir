use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::value::ValueId;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use acvm::acir::native_types::Witness;

pub fn id_to_witness<T>(id: Id<T>) -> Witness {
    Witness(id.to_u32())
}

pub fn id_to_int<T>(id: Id<T>) -> u32 {
    id.to_u32()
}

pub fn u32_to_id_value(value: u32) -> ValueId {
    ValueId::new(value)
}

pub fn u32_to_id_basic_block(value: u32) -> BasicBlockId {
    BasicBlockId::new(value)
}