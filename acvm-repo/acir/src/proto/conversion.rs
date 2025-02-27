use std::marker::PhantomData;

use color_eyre::eyre;
use noir_protobuf::ProtoCodec;

use crate::circuit;
use crate::proto::program::Program;

pub struct ProtoSchema<F> {
    field: PhantomData<F>,
}

impl<F> ProtoCodec<circuit::Program<F>, Program> for ProtoSchema<F> {
    fn encode(value: &circuit::Program<F>) -> Program {
        Program {
            functions: Self::encode_vec(&value.functions),
            unconstrained_functions: Self::encode_vec(&value.unconstrained_functions),
        }
    }

    fn decode(value: &Program) -> eyre::Result<circuit::Program<F>> {
        Ok(circuit::Program {
            functions: Self::decode_vec_msg(&value.functions, "functions")?,
            unconstrained_functions: Self::decode_vec_msg(
                &value.unconstrained_functions,
                "unconstrained_functions",
            )?,
        })
    }
}

mod brillig {
    use crate::circuit;
    use color_eyre::eyre;
    use noir_protobuf::ProtoCodec;

    use crate::proto::brillig::{
        brillig_opcode, BinaryFieldOpKind, BinaryIntOpKind, BrilligBytecode, BrilligOpcode,
        IntegerBitSize, MemoryAddress,
    };

    use super::ProtoSchema;

    impl<F> ProtoCodec<circuit::brillig::BrilligBytecode<F>, BrilligBytecode> for ProtoSchema<F> {
        fn encode(value: &circuit::brillig::BrilligBytecode<F>) -> BrilligBytecode {
            BrilligBytecode { bytecode: Self::encode_vec(&value.bytecode) }
        }

        fn decode(value: &BrilligBytecode) -> eyre::Result<circuit::brillig::BrilligBytecode<F>> {
            todo!()
        }
    }

    impl<F> ProtoCodec<brillig::Opcode<F>, BrilligOpcode> for ProtoSchema<F> {
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
                brillig::Opcode::Not { destination, source, bit_size } => todo!(),
                brillig::Opcode::Cast { destination, source, bit_size } => todo!(),
                brillig::Opcode::JumpIfNot { condition, location } => todo!(),
                brillig::Opcode::JumpIf { condition, location } => todo!(),
                brillig::Opcode::Jump { location } => todo!(),
                brillig::Opcode::CalldataCopy {
                    destination_address,
                    size_address,
                    offset_address,
                } => todo!(),
                brillig::Opcode::Call { location } => todo!(),
                brillig::Opcode::Const { destination, bit_size, value } => todo!(),
                brillig::Opcode::IndirectConst { destination_pointer, bit_size, value } => todo!(),
                brillig::Opcode::Return => todo!(),
                brillig::Opcode::ForeignCall {
                    function,
                    destinations,
                    destination_value_types,
                    inputs,
                    input_value_types,
                } => todo!(),
                brillig::Opcode::Mov { destination, source } => todo!(),
                brillig::Opcode::ConditionalMov { destination, source_a, source_b, condition } => {
                    todo!()
                }
                brillig::Opcode::Load { destination, source_pointer } => todo!(),
                brillig::Opcode::Store { destination_pointer, source } => todo!(),
                brillig::Opcode::BlackBox(black_box_op) => todo!(),
                brillig::Opcode::Trap { revert_data } => todo!(),
                brillig::Opcode::Stop { return_data } => todo!(),
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
}

mod acir {
    use crate::{circuit, proto::acir::circuit::Circuit};
    use color_eyre::eyre;
    use noir_protobuf::ProtoCodec;

    use super::ProtoSchema;

    impl<F> ProtoCodec<circuit::Circuit<F>, Circuit> for ProtoSchema<F> {
        fn encode(value: &circuit::Circuit<F>) -> Circuit {
            todo!()
        }

        fn decode(value: &Circuit) -> eyre::Result<circuit::Circuit<F>> {
            todo!()
        }
    }
}
