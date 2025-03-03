use crate::{
    circuit,
    proto::brillig::{BitSize, BlackBoxOp, HeapArray, HeapValueType, HeapVector, ValueOrArray},
};
use acir_field::AcirField;
use color_eyre::eyre::{self, bail};
use noir_protobuf::{decode_oneof_map, ProtoCodec};

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
        Ok(circuit::brillig::BrilligBytecode {
            bytecode: Self::decode_vec_wrap(&value.bytecode, "bytecode")?,
        })
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
        use brillig_opcode::*;

        decode_oneof_map(&value.value, |value| match value {
            Value::BinaryFieldOp(binary_field_op) => Ok(brillig::Opcode::BinaryFieldOp {
                destination: Self::decode_some_wrap(&binary_field_op.destination, "destination")?,
                op: Self::decode_enum_wrap(binary_field_op.op, "op")?,
                lhs: Self::decode_some_wrap(&binary_field_op.lhs, "lhs")?,
                rhs: Self::decode_some_wrap(&binary_field_op.rhs, "rhs")?,
            }),
            Value::BinaryIntOp(binary_int_op) => Ok(brillig::Opcode::BinaryIntOp {
                destination: Self::decode_some_wrap(&binary_int_op.destination, "destination")?,
                op: Self::decode_enum_wrap(binary_int_op.op, "op")?,
                bit_size: Self::decode_enum_wrap(binary_int_op.bit_size, "bit_size")?,
                lhs: Self::decode_some_wrap(&binary_int_op.lhs, "lhs")?,
                rhs: Self::decode_some_wrap(&binary_int_op.rhs, "rhs")?,
            }),
            Value::Not(not) => Ok(brillig::Opcode::Not {
                destination: Self::decode_some_wrap(&not.destination, "destination")?,
                source: Self::decode_some_wrap(&not.source, "source")?,
                bit_size: Self::decode_enum_wrap(not.bit_size, "bit_size")?,
            }),
            Value::Cast(cast) => Ok(brillig::Opcode::Cast {
                destination: Self::decode_some_wrap(&cast.destination, "destination")?,
                source: Self::decode_some_wrap(&cast.source, "source")?,
                bit_size: Self::decode_some_wrap(&cast.bit_size, "bit_size")?,
            }),
            Value::JumpIfNot(jump_if_not) => Ok(brillig::Opcode::JumpIfNot {
                condition: Self::decode_some_wrap(&jump_if_not.condition, "condition")?,
                location: Self::decode_wrap(&jump_if_not.location, "location")?,
            }),
            Value::JumpIf(jump_if) => Ok(brillig::Opcode::JumpIf {
                condition: Self::decode_some_wrap(&jump_if.condition, "condition")?,
                location: Self::decode_wrap(&jump_if.location, "location")?,
            }),
            Value::Jump(jump) => Ok(brillig::Opcode::Jump {
                location: Self::decode_wrap(&jump.location, "location")?,
            }),
            Value::CalldataCopy(calldata_copy) => Ok(brillig::Opcode::CalldataCopy {
                destination_address: Self::decode_some_wrap(
                    &calldata_copy.destination_address,
                    "destination_address",
                )?,
                size_address: Self::decode_some_wrap(&calldata_copy.size_address, "size_address")?,
                offset_address: Self::decode_some_wrap(
                    &calldata_copy.offset_address,
                    "offset_address",
                )?,
            }),
            Value::Call(call) => Ok(brillig::Opcode::Call {
                location: Self::decode_wrap(&call.location, "location")?,
            }),
            Value::Const(constant) => Ok(brillig::Opcode::Const {
                destination: Self::decode_some_wrap(&constant.destination, "destination")?,
                bit_size: Self::decode_some_wrap(&constant.bit_size, "bit_size")?,
                value: Self::decode_some_wrap(&constant.value, "value")?,
            }),
            Value::IndirectConst(indirect_const) => Ok(brillig::Opcode::IndirectConst {
                destination_pointer: Self::decode_some_wrap(
                    &indirect_const.destination_pointer,
                    "destination_pointer",
                )?,
                bit_size: Self::decode_some_wrap(&indirect_const.bit_size, "bit_size")?,
                value: Self::decode_some_wrap(&indirect_const.value, "value")?,
            }),
            Value::Return(_) => Ok(brillig::Opcode::Return),
            Value::ForeignCall(foreign_call) => Ok(brillig::Opcode::ForeignCall {
                function: foreign_call.function.clone(),
                destinations: Self::decode_vec_wrap(&foreign_call.destinations, "destinations")?,
                destination_value_types: Self::decode_vec_wrap(
                    &foreign_call.destination_value_types,
                    "destination_value_types",
                )?,
                inputs: Self::decode_vec_wrap(&foreign_call.inputs, "inputs")?,
                input_value_types: Self::decode_vec_wrap(
                    &foreign_call.input_value_types,
                    "input_value_types",
                )?,
            }),
            Value::Mov(mov) => Ok(brillig::Opcode::Mov {
                destination: Self::decode_some_wrap(&mov.destination, "destination")?,
                source: Self::decode_some_wrap(&mov.source, "source")?,
            }),
            Value::ConditionalMov(conditional_mov) => Ok(brillig::Opcode::ConditionalMov {
                destination: Self::decode_some_wrap(&conditional_mov.destination, "destination")?,
                source_a: Self::decode_some_wrap(&conditional_mov.source_a, "source_a")?,
                source_b: Self::decode_some_wrap(&conditional_mov.source_b, "source_b")?,
                condition: Self::decode_some_wrap(&conditional_mov.condition, "condition")?,
            }),
            Value::Load(load) => Ok(brillig::Opcode::Load {
                destination: Self::decode_some_wrap(&load.destination, "destination")?,
                source_pointer: Self::decode_some_wrap(&load.source_pointer, "source_pointer")?,
            }),
            Value::Store(store) => Ok(brillig::Opcode::Store {
                destination_pointer: Self::decode_some_wrap(
                    &store.destination_pointer,
                    "destination_pointer",
                )?,
                source: Self::decode_some_wrap(&store.source, "source")?,
            }),
            Value::BlackBox(black_box) => {
                Ok(brillig::Opcode::BlackBox(Self::decode_some_wrap(&black_box.op, "black_box")?))
            }
            Value::Trap(trap) => Ok(brillig::Opcode::Trap {
                revert_data: Self::decode_some_wrap(&trap.revert_data, "revert_data")?,
            }),
            Value::Stop(stop) => Ok(brillig::Opcode::Stop {
                return_data: Self::decode_some_wrap(&stop.return_data, "return_data")?,
            }),
        })
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
        use crate::proto::brillig::memory_address::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Direct(v) => Self::decode_wrap(v, "direct").map(brillig::MemoryAddress::Direct),
            Value::Relative(v) => {
                Self::decode_wrap(v, "relative").map(brillig::MemoryAddress::Relative)
            }
        })
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
        match value {
            BinaryFieldOpKind::BfoUnspecified => bail!("unspecified BinaryFieldOp"),
            BinaryFieldOpKind::BfoAdd => Ok(brillig::BinaryFieldOp::Add),
            BinaryFieldOpKind::BfoSub => Ok(brillig::BinaryFieldOp::Sub),
            BinaryFieldOpKind::BfoMul => Ok(brillig::BinaryFieldOp::Mul),
            BinaryFieldOpKind::BfoDiv => Ok(brillig::BinaryFieldOp::Div),
            BinaryFieldOpKind::BfoIntegerDiv => Ok(brillig::BinaryFieldOp::IntegerDiv),
            BinaryFieldOpKind::BfoEquals => Ok(brillig::BinaryFieldOp::Equals),
            BinaryFieldOpKind::BfoLessThan => Ok(brillig::BinaryFieldOp::LessThan),
            BinaryFieldOpKind::BfoLessThanEquals => Ok(brillig::BinaryFieldOp::LessThanEquals),
        }
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
        match value {
            BinaryIntOpKind::BioUnspecified => bail!("unspecified BinaryIntOp"),
            BinaryIntOpKind::BioAdd => Ok(brillig::BinaryIntOp::Add),
            BinaryIntOpKind::BioSub => Ok(brillig::BinaryIntOp::Sub),
            BinaryIntOpKind::BioMul => Ok(brillig::BinaryIntOp::Mul),
            BinaryIntOpKind::BioDiv => Ok(brillig::BinaryIntOp::Div),
            BinaryIntOpKind::BioEquals => Ok(brillig::BinaryIntOp::Equals),
            BinaryIntOpKind::BioLessThan => Ok(brillig::BinaryIntOp::LessThan),
            BinaryIntOpKind::BioLessThanEquals => Ok(brillig::BinaryIntOp::LessThanEquals),
            BinaryIntOpKind::BioAnd => Ok(brillig::BinaryIntOp::And),
            BinaryIntOpKind::BioOr => Ok(brillig::BinaryIntOp::Or),
            BinaryIntOpKind::BioXor => Ok(brillig::BinaryIntOp::Xor),
            BinaryIntOpKind::BioShl => Ok(brillig::BinaryIntOp::Shl),
            BinaryIntOpKind::BioShr => Ok(brillig::BinaryIntOp::Shr),
        }
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
        match value {
            IntegerBitSize::IbsUnspecified => bail!("unspecified IntegerBitSize"),
            IntegerBitSize::IbsU1 => Ok(brillig::IntegerBitSize::U1),
            IntegerBitSize::IbsU8 => Ok(brillig::IntegerBitSize::U8),
            IntegerBitSize::IbsU16 => Ok(brillig::IntegerBitSize::U16),
            IntegerBitSize::IbsU32 => Ok(brillig::IntegerBitSize::U32),
            IntegerBitSize::IbsU64 => Ok(brillig::IntegerBitSize::U64),
            IntegerBitSize::IbsU128 => Ok(brillig::IntegerBitSize::U128),
        }
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
        use crate::proto::brillig::bit_size::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Field(_) => Ok(brillig::BitSize::Field),
            Value::Integer(size) => {
                Ok(brillig::BitSize::Integer(Self::decode_enum_wrap(*size, "size")?))
            }
        })
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
        use crate::proto::brillig::value_or_array::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::MemoryAddress(memory_address) => Ok(brillig::ValueOrArray::MemoryAddress(
                Self::decode_wrap(memory_address, "memory_address")?,
            )),
            Value::HeapArray(heap_array) => {
                Ok(brillig::ValueOrArray::HeapArray(Self::decode_wrap(heap_array, "heap_array")?))
            }
            Value::HeapVector(heap_vector) => Ok(brillig::ValueOrArray::HeapVector(
                Self::decode_wrap(heap_vector, "heap_vector")?,
            )),
        })
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
        use crate::proto::brillig::heap_value_type::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Simple(bit_size) => {
                Ok(brillig::HeapValueType::Simple(Self::decode_wrap(bit_size, "simple")?))
            }
            Value::Array(array) => Ok(brillig::HeapValueType::Array {
                value_types: Self::decode_vec_wrap(&array.value_types, "value_types")?,
                size: Self::decode_wrap(&array.size, "size")?,
            }),
            Value::Vector(vector) => Ok(brillig::HeapValueType::Vector {
                value_types: Self::decode_vec_wrap(&vector.value_types, "value_types")?,
            }),
        })
    }
}

impl<F> ProtoCodec<brillig::HeapArray, HeapArray> for ProtoSchema<F> {
    fn encode(value: &brillig::HeapArray) -> HeapArray {
        HeapArray { pointer: Self::encode_some(&value.pointer), size: Self::encode(&value.size) }
    }

    fn decode(value: &HeapArray) -> eyre::Result<brillig::HeapArray> {
        Ok(brillig::HeapArray {
            pointer: Self::decode_some_wrap(&value.pointer, "pointer")?,
            size: Self::decode_wrap(&value.size, "size")?,
        })
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
        Ok(brillig::HeapVector {
            pointer: Self::decode_some_wrap(&value.pointer, "pointer")?,
            size: Self::decode_some_wrap(&value.size, "size")?,
        })
    }
}

impl<F> ProtoCodec<brillig::BlackBoxOp, BlackBoxOp> for ProtoSchema<F> {
    fn encode(value: &brillig::BlackBoxOp) -> BlackBoxOp {
        use crate::proto::brillig::black_box_op::*;
        let value = match value {
            brillig::BlackBoxOp::AES128Encrypt { inputs, iv, key, outputs } => {
                Value::Aes128Encrypt(Aes128Encrypt {
                    inputs: Self::encode_some(inputs),
                    iv: Self::encode_some(iv),
                    key: Self::encode_some(key),
                    outputs: Self::encode_some(outputs),
                })
            }
            brillig::BlackBoxOp::Blake2s { message, output } => Value::Blake2s(Blake2s {
                message: Self::encode_some(message),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::Blake3 { message, output } => Value::Blake3(Blake3 {
                message: Self::encode_some(message),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::Keccakf1600 { input, output } => Value::KeccakF1600(Keccakf1600 {
                input: Self::encode_some(input),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::EcdsaSecp256k1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => Value::EcdsaSecp256k1(EcdsaSecp256k1 {
                hashed_msg: Self::encode_some(hashed_msg),
                public_key_x: Self::encode_some(public_key_x),
                public_key_y: Self::encode_some(public_key_y),
                signature: Self::encode_some(signature),
                result: Self::encode_some(result),
            }),
            brillig::BlackBoxOp::EcdsaSecp256r1 {
                hashed_msg,
                public_key_x,
                public_key_y,
                signature,
                result,
            } => Value::EcdsaSecp256r1(EcdsaSecp256r1 {
                hashed_msg: Self::encode_some(hashed_msg),
                public_key_x: Self::encode_some(public_key_x),
                public_key_y: Self::encode_some(public_key_y),
                signature: Self::encode_some(signature),
                result: Self::encode_some(result),
            }),
            brillig::BlackBoxOp::MultiScalarMul { points, scalars, outputs } => {
                Value::MultiScalarMul(MultiScalarMul {
                    points: Self::encode_some(points),
                    scalars: Self::encode_some(scalars),
                    outputs: Self::encode_some(outputs),
                })
            }
            brillig::BlackBoxOp::EmbeddedCurveAdd {
                input1_x,
                input1_y,
                input1_infinite,
                input2_x,
                input2_y,
                input2_infinite,
                result,
            } => Value::EmbeddedCurveAdd(EmbeddedCurveAdd {
                input1_x: Self::encode_some(input1_x),
                input1_y: Self::encode_some(input1_y),
                input1_infinite: Self::encode_some(input1_infinite),
                input2_x: Self::encode_some(input2_x),
                input2_y: Self::encode_some(input2_y),
                input2_infinite: Self::encode_some(input2_infinite),
                result: Self::encode_some(result),
            }),
            brillig::BlackBoxOp::BigIntAdd { lhs, rhs, output } => Value::BigIntAdd(BigIntAdd {
                lhs: Self::encode_some(lhs),
                rhs: Self::encode_some(rhs),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::BigIntSub { lhs, rhs, output } => Value::BigIntSub(BigIntSub {
                lhs: Self::encode_some(lhs),
                rhs: Self::encode_some(rhs),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::BigIntMul { lhs, rhs, output } => Value::BigIntMul(BigIntMul {
                lhs: Self::encode_some(lhs),
                rhs: Self::encode_some(rhs),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::BigIntDiv { lhs, rhs, output } => Value::BigIntDiv(BigIntDiv {
                lhs: Self::encode_some(lhs),
                rhs: Self::encode_some(rhs),
                output: Self::encode_some(output),
            }),
            brillig::BlackBoxOp::BigIntFromLeBytes { inputs, modulus, output } => {
                Value::BigIntFromLeBytes(BigIntFromLeBytes {
                    inputs: Self::encode_some(inputs),
                    modulus: Self::encode_some(modulus),
                    output: Self::encode_some(output),
                })
            }
            brillig::BlackBoxOp::BigIntToLeBytes { input, output } => {
                Value::BigIntToLeBytes(BigIntToLeBytes {
                    input: Self::encode_some(input),
                    output: Self::encode_some(output),
                })
            }
            brillig::BlackBoxOp::Poseidon2Permutation { message, output, len } => {
                Value::Poseidon2Permutation(Poseidon2Permutation {
                    message: Self::encode_some(message),
                    output: Self::encode_some(output),
                    len: Self::encode_some(len),
                })
            }
            brillig::BlackBoxOp::Sha256Compression { input, hash_values, output } => {
                Value::Sha256Compression(Sha256Compression {
                    input: Self::encode_some(input),
                    hash_values: Self::encode_some(hash_values),
                    output: Self::encode_some(output),
                })
            }
            brillig::BlackBoxOp::ToRadix {
                input,
                radix,
                output_pointer,
                num_limbs,
                output_bits,
            } => Value::ToRadix(ToRadix {
                input: Self::encode_some(input),
                radix: Self::encode_some(radix),
                output_pointer: Self::encode_some(output_pointer),
                num_limbs: Self::encode_some(num_limbs),
                output_bits: Self::encode_some(output_bits),
            }),
        };
        BlackBoxOp { value: Some(value) }
    }

    fn decode(value: &BlackBoxOp) -> eyre::Result<brillig::BlackBoxOp> {
        use crate::proto::brillig::black_box_op::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Aes128Encrypt(aes128_encrypt) => Ok(brillig::BlackBoxOp::AES128Encrypt {
                inputs: Self::decode_some_wrap(&aes128_encrypt.inputs, "inputs")?,
                iv: Self::decode_some_wrap(&aes128_encrypt.iv, "iv")?,
                key: Self::decode_some_wrap(&aes128_encrypt.key, "key")?,
                outputs: Self::decode_some_wrap(&aes128_encrypt.outputs, "outputs")?,
            }),
            Value::Blake2s(blake2s) => Ok(brillig::BlackBoxOp::Blake2s {
                message: Self::decode_some_wrap(&blake2s.message, "message")?,
                output: Self::decode_some_wrap(&blake2s.output, "output")?,
            }),
            Value::Blake3(blake3) => Ok(brillig::BlackBoxOp::Blake3 {
                message: Self::decode_some_wrap(&blake3.message, "message")?,
                output: Self::decode_some_wrap(&blake3.output, "output")?,
            }),
            Value::KeccakF1600(keccakf1600) => Ok(brillig::BlackBoxOp::Keccakf1600 {
                input: Self::decode_some_wrap(&keccakf1600.input, "input")?,
                output: Self::decode_some_wrap(&keccakf1600.output, "output")?,
            }),
            Value::EcdsaSecp256k1(ecdsa_secp256k1) => Ok(brillig::BlackBoxOp::EcdsaSecp256k1 {
                hashed_msg: Self::decode_some_wrap(&ecdsa_secp256k1.hashed_msg, "hashed_msg")?,
                public_key_x: Self::decode_some_wrap(
                    &ecdsa_secp256k1.public_key_x,
                    "public_key_x",
                )?,
                public_key_y: Self::decode_some_wrap(
                    &ecdsa_secp256k1.public_key_y,
                    "public_key_y",
                )?,
                signature: Self::decode_some_wrap(&ecdsa_secp256k1.signature, "signature")?,
                result: Self::decode_some_wrap(&ecdsa_secp256k1.result, "result")?,
            }),
            Value::EcdsaSecp256r1(ecdsa_secp256r1) => Ok(brillig::BlackBoxOp::EcdsaSecp256r1 {
                hashed_msg: Self::decode_some_wrap(&ecdsa_secp256r1.hashed_msg, "hashed_msg")?,
                public_key_x: Self::decode_some_wrap(
                    &ecdsa_secp256r1.public_key_x,
                    "public_key_x",
                )?,
                public_key_y: Self::decode_some_wrap(
                    &ecdsa_secp256r1.public_key_y,
                    "public_key_y",
                )?,
                signature: Self::decode_some_wrap(&ecdsa_secp256r1.signature, "signature")?,
                result: Self::decode_some_wrap(&ecdsa_secp256r1.result, "result")?,
            }),
            Value::MultiScalarMul(multi_scalar_mul) => Ok(brillig::BlackBoxOp::MultiScalarMul {
                points: Self::decode_some_wrap(&multi_scalar_mul.points, "points")?,
                scalars: Self::decode_some_wrap(&multi_scalar_mul.scalars, "scalars")?,
                outputs: Self::decode_some_wrap(&multi_scalar_mul.outputs, "outputs")?,
            }),
            Value::EmbeddedCurveAdd(embedded_curve_add) => {
                Ok(brillig::BlackBoxOp::EmbeddedCurveAdd {
                    input1_x: Self::decode_some_wrap(&embedded_curve_add.input1_x, "input1_x")?,
                    input1_y: Self::decode_some_wrap(&embedded_curve_add.input1_y, "input1_y")?,
                    input1_infinite: Self::decode_some_wrap(
                        &embedded_curve_add.input1_infinite,
                        "input1_infinite",
                    )?,
                    input2_x: Self::decode_some_wrap(&embedded_curve_add.input2_x, "input2_x")?,
                    input2_y: Self::decode_some_wrap(&embedded_curve_add.input2_y, "input2_y")?,
                    input2_infinite: Self::decode_some_wrap(
                        &embedded_curve_add.input2_infinite,
                        "input2_infinite",
                    )?,
                    result: Self::decode_some_wrap(&embedded_curve_add.result, "result")?,
                })
            }
            Value::BigIntAdd(big_int_add) => Ok(brillig::BlackBoxOp::BigIntAdd {
                lhs: Self::decode_some_wrap(&big_int_add.lhs, "lhs")?,
                rhs: Self::decode_some_wrap(&big_int_add.rhs, "rhs")?,
                output: Self::decode_some_wrap(&big_int_add.output, "output")?,
            }),
            Value::BigIntSub(big_int_sub) => Ok(brillig::BlackBoxOp::BigIntSub {
                lhs: Self::decode_some_wrap(&big_int_sub.lhs, "lhs")?,
                rhs: Self::decode_some_wrap(&big_int_sub.rhs, "rhs")?,
                output: Self::decode_some_wrap(&big_int_sub.output, "output")?,
            }),
            Value::BigIntMul(big_int_mul) => Ok(brillig::BlackBoxOp::BigIntMul {
                lhs: Self::decode_some_wrap(&big_int_mul.lhs, "lhs")?,
                rhs: Self::decode_some_wrap(&big_int_mul.rhs, "rhs")?,
                output: Self::decode_some_wrap(&big_int_mul.output, "output")?,
            }),
            Value::BigIntDiv(big_int_div) => Ok(brillig::BlackBoxOp::BigIntDiv {
                lhs: Self::decode_some_wrap(&big_int_div.lhs, "lhs")?,
                rhs: Self::decode_some_wrap(&big_int_div.rhs, "rhs")?,
                output: Self::decode_some_wrap(&big_int_div.output, "output")?,
            }),
            Value::BigIntFromLeBytes(big_int_from_le_bytes) => {
                Ok(brillig::BlackBoxOp::BigIntFromLeBytes {
                    inputs: Self::decode_some_wrap(&big_int_from_le_bytes.inputs, "inputs")?,
                    modulus: Self::decode_some_wrap(&big_int_from_le_bytes.modulus, "modulus")?,
                    output: Self::decode_some_wrap(&big_int_from_le_bytes.output, "output")?,
                })
            }
            Value::BigIntToLeBytes(big_int_to_le_bytes) => {
                Ok(brillig::BlackBoxOp::BigIntToLeBytes {
                    input: Self::decode_some_wrap(&big_int_to_le_bytes.input, "input")?,
                    output: Self::decode_some_wrap(&big_int_to_le_bytes.output, "output")?,
                })
            }
            Value::Poseidon2Permutation(poseidon2_permutation) => {
                Ok(brillig::BlackBoxOp::Poseidon2Permutation {
                    message: Self::decode_some_wrap(&poseidon2_permutation.message, "message")?,
                    output: Self::decode_some_wrap(&poseidon2_permutation.output, "output")?,
                    len: Self::decode_some_wrap(&poseidon2_permutation.len, "len")?,
                })
            }
            Value::Sha256Compression(sha256_compression) => {
                Ok(brillig::BlackBoxOp::Sha256Compression {
                    input: Self::decode_some_wrap(&sha256_compression.input, "input")?,
                    hash_values: Self::decode_some_wrap(
                        &sha256_compression.hash_values,
                        "hash_values",
                    )?,
                    output: Self::decode_some_wrap(&sha256_compression.output, "output")?,
                })
            }
            Value::ToRadix(to_radix) => Ok(brillig::BlackBoxOp::ToRadix {
                input: Self::decode_some_wrap(&to_radix.input, "input")?,
                radix: Self::decode_some_wrap(&to_radix.radix, "radix")?,
                output_pointer: Self::decode_some_wrap(&to_radix.output_pointer, "output_pointer")?,
                num_limbs: Self::decode_some_wrap(&to_radix.num_limbs, "num_limbs")?,
                output_bits: Self::decode_some_wrap(&to_radix.output_bits, "output_bits")?,
            }),
        })
    }
}
