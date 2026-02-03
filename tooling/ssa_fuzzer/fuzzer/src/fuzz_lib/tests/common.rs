use crate::initial_witness::{FieldRepresentation, WitnessValue, WitnessValueNumeric};
use noir_ssa_fuzzer::typed_value::{NumericType, Type};
use noirc_evaluator::ssa::ir::function::RuntimeType;
use noirc_frontend::monomorphization::ast::InlineType as FrontendInlineType;

/// Creates default witness values for testing
/// Returns [Field(0), Field(1), Field(2), Field(3), Field(4)]
pub(crate) fn default_witness() -> Vec<WitnessValue> {
    vec![
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 0 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 1 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 2 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 3 })),
        WitnessValue::Numeric(WitnessValueNumeric::Field(FieldRepresentation { high: 0, low: 4 })),
    ]
}

pub(crate) fn default_input_types() -> Vec<Type> {
    vec![Type::Numeric(NumericType::Field); 5]
}

pub(crate) fn default_runtimes() -> Vec<RuntimeType> {
    vec![
        RuntimeType::Brillig(FrontendInlineType::default()),
        RuntimeType::Acir(FrontendInlineType::default()),
    ]
}
