use crate::{
    circuit,
    proto::brillig::{BitSize, BlackBoxOp, HeapValueType, HeapVector, ValueOrArray},
};
use acir_field::AcirField;
use color_eyre::eyre;
use noir_protobuf::ProtoCodec;

use crate::proto::brillig::{
    brillig_opcode, BinaryFieldOpKind, BinaryIntOpKind, BrilligBytecode, BrilligOpcode,
    IntegerBitSize, MemoryAddress,
};

use super::ProtoSchema;

impl<F> ProtoCodec<circuit::brillig::BrilligBytecode<F>, BrilligBytecode> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::brillig::BrilligBytecode<F>) -> BrilligBytecode {
        BrilligBytecode { bytecode: Self::encode_vec(&value.bytecode) }
    }

    fn decode(value: &BrilligBytecode) -> eyre::Result<circuit::brillig::BrilligBytecode<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::Opcode<F>, BrilligOpcode> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &brillig::Opcode<F>) -> BrilligOpcode {
        use brillig_opcode::*;

        let value = match value {
            brillig::Opcode::BinaryFieldOp { destination, op, lhs, rhs } => {
                Value::BinaryFieldOp(BinaryFieldOp {
                    destination: Self::encode_some(destination),
                    op: Self::encode_enum(op),
                    lhs: Self::encode_some(lhs),
                    rhs: Self::encode_some(rhs),
                })
            }
            brillig::Opcode::BinaryIntOp { destination, op, bit_size, lhs, rhs } => {
                Value::BinaryIntOp(BinaryIntOp {
                    destination: Self::encode_some(destination),
                    op: Self::encode_enum(op),
                    bit_size: Self::encode_enum(bit_size),
                    lhs: Self::encode_some(lhs),
                    rhs: Self::encode_some(rhs),
                })
            }
            brillig::Opcode::Not { destination, source, bit_size } => Value::Not(Not {
                destination: Self::encode_some(destination),
                source: Self::encode_some(source),
                bit_size: Self::encode_enum(bit_size),
            }),
            brillig::Opcode::Cast { destination, source, bit_size } => Value::Cast(Cast {
                destination: Self::encode_some(destination),
                source: Self::encode_some(source),
                bit_size: Self::encode_some(bit_size),
            }),
            brillig::Opcode::JumpIfNot { condition, location } => Value::JumpIfNot(JumpIfNot {
                condition: Self::encode_some(condition),
                location: *location as u64,
            }),
            brillig::Opcode::JumpIf { condition, location } => Value::JumpIf(JumpIf {
                condition: Self::encode_some(condition),
                location: *location as u64,
            }),
            brillig::Opcode::Jump { location } => Value::Jump(Jump { location: *location as u64 }),
            brillig::Opcode::CalldataCopy { destination_address, size_address, offset_address } => {
                Value::CalldataCopy(CalldataCopy {
                    destination_address: Self::encode_some(destination_address),
                    size_address: Self::encode_some(size_address),
                    offset_address: Self::encode_some(offset_address),
                })
            }
            brillig::Opcode::Call { location } => Value::Call(Call { location: *location as u64 }),
            brillig::Opcode::Const { destination, bit_size, value } => Value::Const(Const {
                destination: Self::encode_some(destination),
                bit_size: Self::encode_some(bit_size),
                value: Self::encode_some(value),
            }),
            brillig::Opcode::IndirectConst { destination_pointer, bit_size, value } => {
                Value::IndirectConst(IndirectConst {
                    destination_pointer: Self::encode_some(destination_pointer),
                    bit_size: Self::encode_some(bit_size),
                    value: Self::encode_some(value),
                })
            }
            brillig::Opcode::Return => Value::Return(Return {}),
            brillig::Opcode::ForeignCall {
                function,
                destinations,
                destination_value_types,
                inputs,
                input_value_types,
            } => Value::ForeignCall(ForeignCall {
                function: function.to_string(),
                destinations: Self::encode_vec(destinations),
                destination_value_types: Self::encode_vec(destination_value_types),
                inputs: Self::encode_vec(inputs),
                input_value_types: Self::encode_vec(input_value_types),
            }),
            brillig::Opcode::Mov { destination, source } => Value::Mov(Mov {
                destination: Self::encode_some(destination),
                source: Self::encode_some(source),
            }),
            brillig::Opcode::ConditionalMov { destination, source_a, source_b, condition } => {
                Value::ConditionalMov(ConditionalMov {
                    destination: Self::encode_some(destination),
                    source_a: Self::encode_some(source_a),
                    source_b: Self::encode_some(source_b),
                    condition: Self::encode_some(condition),
                })
            }
            brillig::Opcode::Load { destination, source_pointer } => Value::Load(Load {
                destination: Self::encode_some(destination),
                source_pointer: Self::encode_some(source_pointer),
            }),
            brillig::Opcode::Store { destination_pointer, source } => Value::Store(Store {
                destination_pointer: Self::encode_some(destination_pointer),
                source: Self::encode_some(source),
            }),
            brillig::Opcode::BlackBox(black_box_op) => {
                Value::BlackBox(BlackBox { op: Self::encode_some(black_box_op) })
            }
            brillig::Opcode::Trap { revert_data } => {
                Value::Trap(Trap { revert_data: Self::encode_some(revert_data) })
            }
            brillig::Opcode::Stop { return_data } => {
                Value::Stop(Stop { return_data: Self::encode_some(return_data) })
            }
        };
        BrilligOpcode { value: Some(value) }
    }

    fn decode(value: &BrilligOpcode) -> eyre::Result<brillig::Opcode<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::MemoryAddress, MemoryAddress> for ProtoSchema<F> {
    fn encode(value: &brillig::MemoryAddress) -> MemoryAddress {
        todo!()
    }

    fn decode(value: &MemoryAddress) -> eyre::Result<brillig::MemoryAddress> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BinaryFieldOp, BinaryFieldOpKind> for ProtoSchema<F> {
    fn encode(value: &brillig::BinaryFieldOp) -> BinaryFieldOpKind {
        todo!()
    }

    fn decode(value: &BinaryFieldOpKind) -> eyre::Result<brillig::BinaryFieldOp> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BinaryIntOp, BinaryIntOpKind> for ProtoSchema<F> {
    fn encode(value: &brillig::BinaryIntOp) -> BinaryIntOpKind {
        todo!()
    }

    fn decode(value: &BinaryIntOpKind) -> eyre::Result<brillig::BinaryIntOp> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::IntegerBitSize, IntegerBitSize> for ProtoSchema<F> {
    fn encode(value: &brillig::IntegerBitSize) -> IntegerBitSize {
        todo!()
    }

    fn decode(value: &IntegerBitSize) -> eyre::Result<brillig::IntegerBitSize> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BitSize, BitSize> for ProtoSchema<F> {
    fn encode(value: &brillig::BitSize) -> BitSize {
        todo!()
    }

    fn decode(value: &BitSize) -> eyre::Result<brillig::BitSize> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::ValueOrArray, ValueOrArray> for ProtoSchema<F> {
    fn encode(value: &brillig::ValueOrArray) -> ValueOrArray {
        todo!()
    }

    fn decode(value: &ValueOrArray) -> eyre::Result<brillig::ValueOrArray> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::HeapValueType, HeapValueType> for ProtoSchema<F> {
    fn encode(value: &brillig::HeapValueType) -> HeapValueType {
        todo!()
    }

    fn decode(value: &HeapValueType) -> eyre::Result<brillig::HeapValueType> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BlackBoxOp, BlackBoxOp> for ProtoSchema<F> {
    fn encode(value: &brillig::BlackBoxOp) -> BlackBoxOp {
        todo!()
    }

    fn decode(value: &BlackBoxOp) -> eyre::Result<brillig::BlackBoxOp> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::HeapVector, HeapVector> for ProtoSchema<F> {
    fn encode(value: &brillig::HeapVector) -> HeapVector {
        todo!()
    }

    fn decode(value: &HeapVector) -> eyre::Result<brillig::HeapVector> {
        todo!()
    }
}
