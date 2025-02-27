use color_eyre::eyre::{self, Context};
use iter_extended::{try_vecmap, vecmap};
use noir_protobuf::{from_proto, to_proto, ProtoCodec};

use crate::proto::program::Program;

impl<F> ProtoCodec for crate::circuit::Program<F> {
    type Repr = Program;

    fn encode(&self) -> Self::Repr {
        Program {
            functions: vecmap(&self.functions, to_proto),
            unconstrained_functions: vecmap(&self.unconstrained_functions, to_proto),
        }
    }

    fn decode(value: &Self::Repr) -> eyre::Result<Self> {
        Ok(Self {
            functions: try_vecmap(&value.functions, from_proto).wrap_err("functions")?,
            unconstrained_functions: try_vecmap(&value.unconstrained_functions, from_proto)
                .wrap_err("unconstrained_functions")?,
        })
    }
}

mod brillig {
    use color_eyre::eyre;
    use iter_extended::vecmap;
    use noir_protobuf::{
        from_proto, to_proto, to_proto_repr, to_proto_repr_f, ProtoCodec, ProtoRepr, ProtoReprF,
    };

    use crate::proto::brillig::{
        brillig_opcode, BinaryFieldOpKind, BinaryIntOpKind, BrilligBytecode, BrilligOpcode,
        IntegerBitSize, MemoryAddress,
    };

    impl<F> ProtoCodec for crate::circuit::brillig::BrilligBytecode<F> {
        type Repr = BrilligBytecode;

        fn encode(&self) -> Self::Repr {
            BrilligBytecode { bytecode: vecmap(&self.bytecode, to_proto_repr_f) }
        }

        fn decode(value: &Self::Repr) -> eyre::Result<Self> {
            todo!()
        }
    }

    impl<F> ProtoReprF<F> for BrilligOpcode {
        type Type = brillig::Opcode<F>;

        fn encode(value: &Self::Type) -> Self {
            use brillig_opcode::*;

            let value = match value {
                brillig::Opcode::BinaryFieldOp { destination, op, lhs, rhs } => {
                    Value::BinaryFieldOp(BinaryFieldOp {
                        destination: Some(to_proto_repr(destination)),
                        op: to_proto_repr::<BinaryFieldOpKind>(op).into(),
                        lhs: Some(to_proto_repr(lhs)),
                        rhs: Some(to_proto_repr(rhs)),
                    })
                }
                brillig::Opcode::BinaryIntOp { destination, op, bit_size, lhs, rhs } => {
                    Value::BinaryIntOp(BinaryIntOp {
                        destination: Some(to_proto_repr(destination)),
                        op: to_proto_repr::<BinaryIntOpKind>(op).into(),
                        bit_size: to_proto_repr::<IntegerBitSize>(bit_size).into(),
                        lhs: Some(to_proto_repr(lhs)),
                        rhs: Some(to_proto_repr(rhs)),
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

        fn decode(&self) -> eyre::Result<Self::Type> {
            todo!()
        }
    }

    impl ProtoRepr for MemoryAddress {
        type Type = brillig::MemoryAddress;

        fn encode(value: &Self::Type) -> Self {
            todo!()
        }

        fn decode(&self) -> eyre::Result<Self::Type> {
            todo!()
        }
    }

    impl ProtoRepr for BinaryFieldOpKind {
        type Type = brillig::BinaryFieldOp;

        fn encode(value: &Self::Type) -> Self {
            todo!()
        }

        fn decode(&self) -> eyre::Result<Self::Type> {
            todo!()
        }
    }

    impl ProtoRepr for BinaryIntOpKind {
        type Type = brillig::BinaryIntOp;

        fn encode(value: &Self::Type) -> Self {
            todo!()
        }

        fn decode(&self) -> eyre::Result<Self::Type> {
            todo!()
        }
    }

    impl ProtoRepr for IntegerBitSize {
        type Type = brillig::IntegerBitSize;

        fn encode(value: &Self::Type) -> Self {
            todo!()
        }

        fn decode(&self) -> eyre::Result<Self::Type> {
            todo!()
        }
    }
}

mod acir {
    use crate::proto::acir::circuit::Circuit;
    use color_eyre::eyre;
    use noir_protobuf::{from_proto, to_proto, to_proto_repr, ProtoCodec};

    impl<F> ProtoCodec for crate::circuit::Circuit<F> {
        type Repr = Circuit;

        fn encode(&self) -> Self::Repr {
            todo!()
        }

        fn decode(value: &Self::Repr) -> eyre::Result<Self> {
            todo!()
        }
    }
}
