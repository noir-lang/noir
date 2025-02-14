use noirc_evaluator::ssa::ir::map::Id;
use noirc_evaluator::ssa::ir::value::ValueId;
use acvm::acir::native_types::Witness;

pub fn id_to_witness<T>(id: Id<T>) -> Witness {
    Witness(id.to_u32())
}

pub fn id_to_int<T>(id: Id<T>) -> u32 {
    id.to_u32()
}

pub fn u32_to_id(value: u32) -> ValueId {
    ValueId::new(value)
}