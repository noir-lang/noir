use crate::{
    circuit,
    proto::brillig::{BitSize, BlackBoxOp, HeapArray, HeapValueType, HeapVector, ValueOrArray},
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
                location: Self::encode(location),
            }),
            brillig::Opcode::JumpIf { condition, location } => Value::JumpIf(JumpIf {
                condition: Self::encode_some(condition),
                location: Self::encode(location),
            }),
            brillig::Opcode::Jump { location } => {
                Value::Jump(Jump { location: Self::encode(location) })
            }
            brillig::Opcode::CalldataCopy { destination_address, size_address, offset_address } => {
                Value::CalldataCopy(CalldataCopy {
                    destination_address: Self::encode_some(destination_address),
                    size_address: Self::encode_some(size_address),
                    offset_address: Self::encode_some(offset_address),
                })
            }
            brillig::Opcode::Call { location } => {
                Value::Call(Call { location: Self::encode(location) })
            }
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
        use crate::proto::brillig::memory_address::*;
        let value = match value {
            brillig::MemoryAddress::Direct(addr) => Value::Direct(Self::encode(addr)),
            brillig::MemoryAddress::Relative(addr) => Value::Relative(Self::encode(addr)),
        };
        MemoryAddress { value: Some(value) }
    }

    fn decode(value: &MemoryAddress) -> eyre::Result<brillig::MemoryAddress> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BinaryFieldOp, BinaryFieldOpKind> for ProtoSchema<F> {
    fn encode(value: &brillig::BinaryFieldOp) -> BinaryFieldOpKind {
        match value {
            brillig::BinaryFieldOp::Add => BinaryFieldOpKind::BfoAdd,
            brillig::BinaryFieldOp::Sub => BinaryFieldOpKind::BfoSub,
            brillig::BinaryFieldOp::Mul => BinaryFieldOpKind::BfoMul,
            brillig::BinaryFieldOp::Div => BinaryFieldOpKind::BfoDiv,
            brillig::BinaryFieldOp::IntegerDiv => BinaryFieldOpKind::BfoIntegerDiv,
            brillig::BinaryFieldOp::Equals => BinaryFieldOpKind::BfoEquals,
            brillig::BinaryFieldOp::LessThan => BinaryFieldOpKind::BfoLessThan,
            brillig::BinaryFieldOp::LessThanEquals => BinaryFieldOpKind::BfoLessThanEquals,
        }
    }

    fn decode(value: &BinaryFieldOpKind) -> eyre::Result<brillig::BinaryFieldOp> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BinaryIntOp, BinaryIntOpKind> for ProtoSchema<F> {
    fn encode(value: &brillig::BinaryIntOp) -> BinaryIntOpKind {
        match value {
            brillig::BinaryIntOp::Add => BinaryIntOpKind::BioAdd,
            brillig::BinaryIntOp::Sub => BinaryIntOpKind::BioSub,
            brillig::BinaryIntOp::Mul => BinaryIntOpKind::BioMul,
            brillig::BinaryIntOp::Div => BinaryIntOpKind::BioDiv,
            brillig::BinaryIntOp::Equals => BinaryIntOpKind::BioEquals,
            brillig::BinaryIntOp::LessThan => BinaryIntOpKind::BioLessThan,
            brillig::BinaryIntOp::LessThanEquals => BinaryIntOpKind::BioLessThanEquals,
            brillig::BinaryIntOp::And => BinaryIntOpKind::BioAnd,
            brillig::BinaryIntOp::Or => BinaryIntOpKind::BioOr,
            brillig::BinaryIntOp::Xor => BinaryIntOpKind::BioXor,
            brillig::BinaryIntOp::Shl => BinaryIntOpKind::BioShl,
            brillig::BinaryIntOp::Shr => BinaryIntOpKind::BioShr,
        }
    }

    fn decode(value: &BinaryIntOpKind) -> eyre::Result<brillig::BinaryIntOp> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::IntegerBitSize, IntegerBitSize> for ProtoSchema<F> {
    fn encode(value: &brillig::IntegerBitSize) -> IntegerBitSize {
        match value {
            brillig::IntegerBitSize::U1 => IntegerBitSize::IbsU1,
            brillig::IntegerBitSize::U8 => IntegerBitSize::IbsU8,
            brillig::IntegerBitSize::U16 => IntegerBitSize::IbsU16,
            brillig::IntegerBitSize::U32 => IntegerBitSize::IbsU32,
            brillig::IntegerBitSize::U64 => IntegerBitSize::IbsU64,
            brillig::IntegerBitSize::U128 => IntegerBitSize::IbsU128,
        }
    }

    fn decode(value: &IntegerBitSize) -> eyre::Result<brillig::IntegerBitSize> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::BitSize, BitSize> for ProtoSchema<F> {
    fn encode(value: &brillig::BitSize) -> BitSize {
        use crate::proto::brillig::bit_size::*;
        let value = match value {
            brillig::BitSize::Field => Value::Field(Field {}),
            brillig::BitSize::Integer(integer_bit_size) => {
                Value::Integer(Self::encode_enum(integer_bit_size))
            }
        };
        BitSize { value: Some(value) }
    }

    fn decode(value: &BitSize) -> eyre::Result<brillig::BitSize> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::ValueOrArray, ValueOrArray> for ProtoSchema<F> {
    fn encode(value: &brillig::ValueOrArray) -> ValueOrArray {
        use crate::proto::brillig::value_or_array::*;
        let value = match value {
            brillig::ValueOrArray::MemoryAddress(memory_address) => {
                Value::MemoryAddress(Self::encode(memory_address))
            }
            brillig::ValueOrArray::HeapArray(heap_array) => {
                Value::HeapArray(Self::encode(heap_array))
            }
            brillig::ValueOrArray::HeapVector(heap_vector) => {
                Value::HeapVector(Self::encode(heap_vector))
            }
        };
        ValueOrArray { value: Some(value) }
    }

    fn decode(value: &ValueOrArray) -> eyre::Result<brillig::ValueOrArray> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::HeapValueType, HeapValueType> for ProtoSchema<F> {
    fn encode(value: &brillig::HeapValueType) -> HeapValueType {
        use crate::proto::brillig::heap_value_type::*;
        let value = match value {
            brillig::HeapValueType::Simple(bit_size) => Value::Simple(Self::encode(bit_size)),
            brillig::HeapValueType::Array { value_types, size } => Value::Array(Array {
                value_types: Self::encode_vec(value_types),
                size: *size as u64,
            }),
            brillig::HeapValueType::Vector { value_types } => {
                Value::Vector(Vector { value_types: Self::encode_vec(value_types) })
            }
        };
        HeapValueType { value: Some(value) }
    }

    fn decode(value: &HeapValueType) -> eyre::Result<brillig::HeapValueType> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::HeapArray, HeapArray> for ProtoSchema<F> {
    fn encode(value: &brillig::HeapArray) -> HeapArray {
        HeapArray { pointer: Self::encode_some(&value.pointer), size: Self::encode(&value.size) }
    }

    fn decode(value: &HeapArray) -> eyre::Result<brillig::HeapArray> {
        todo!()
    }
}

impl<F> ProtoCodec<brillig::HeapVector, HeapVector> for ProtoSchema<F> {
    fn encode(value: &brillig::HeapVector) -> HeapVector {
        HeapVector {
            pointer: Self::encode_some(&value.pointer),
            size: Self::encode_some(&value.size),
        }
    }

    fn decode(value: &HeapVector) -> eyre::Result<brillig::HeapVector> {
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
