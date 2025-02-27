use crate::{
    circuit,
    proto::acir::circuit::{
        AssertMessage, AssertionPayload, BlackBoxFuncCall, Circuit, ExpressionOrMemory,
        ExpressionWidth, MemOp, Opcode, OpcodeLocation,
    },
};
use acir_field::AcirField;
use color_eyre::eyre;
use noir_protobuf::ProtoCodec;

use super::ProtoSchema;

impl<F> ProtoCodec<circuit::Circuit<F>, Circuit> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::Circuit<F>) -> Circuit {
        Circuit {
            current_witness_index: value.current_witness_index,
            opcodes: Self::encode_vec(&value.opcodes),
            expression_width: Self::encode_some(&value.expression_width),
            private_parameters: Self::encode_vec(value.private_parameters.iter()),
            public_parameters: Self::encode_vec(value.public_parameters.0.iter()),
            return_values: Self::encode_vec(value.return_values.0.iter()),
            assert_messages: Self::encode_vec(&value.assert_messages),
        }
    }

    fn decode(value: &Circuit) -> eyre::Result<circuit::Circuit<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::ExpressionWidth, ExpressionWidth> for ProtoSchema<F> {
    fn encode(value: &circuit::ExpressionWidth) -> ExpressionWidth {
        use crate::proto::acir::circuit::expression_width::*;
        let value = match value {
            circuit::ExpressionWidth::Unbounded => Value::Unbounded(Unbounded {}),
            circuit::ExpressionWidth::Bounded { width } => {
                Value::Bounded(Bounded { width: Self::encode(width) })
            }
        };
        ExpressionWidth { value: Some(value) }
    }

    fn decode(value: &ExpressionWidth) -> eyre::Result<circuit::ExpressionWidth> {
        todo!()
    }
}

impl<F> ProtoCodec<(circuit::OpcodeLocation, circuit::AssertionPayload<F>), AssertMessage>
    for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &(circuit::OpcodeLocation, circuit::AssertionPayload<F>)) -> AssertMessage {
        AssertMessage {
            location: Self::encode_some(&value.0),
            payload: Self::encode_some(&value.1),
        }
    }

    fn decode(
        value: &AssertMessage,
    ) -> eyre::Result<(circuit::OpcodeLocation, circuit::AssertionPayload<F>)> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::OpcodeLocation, OpcodeLocation> for ProtoSchema<F> {
    fn encode(value: &circuit::OpcodeLocation) -> OpcodeLocation {
        use crate::proto::acir::circuit::opcode_location::*;
        let value = match value {
            circuit::OpcodeLocation::Acir(size) => Value::Acir(Self::encode(size)),
            circuit::OpcodeLocation::Brillig { acir_index, brillig_index } => {
                Value::Brillig(BrilligLocation {
                    acir_index: Self::encode(acir_index),
                    brillig_index: Self::encode(brillig_index),
                })
            }
        };
        OpcodeLocation { value: Some(value) }
    }

    fn decode(value: &OpcodeLocation) -> eyre::Result<circuit::OpcodeLocation> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::AssertionPayload<F>, AssertionPayload> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::AssertionPayload<F>) -> AssertionPayload {
        AssertionPayload {
            error_selector: value.error_selector,
            payload: Self::encode_vec(&value.payload),
        }
    }

    fn decode(value: &AssertionPayload) -> eyre::Result<circuit::AssertionPayload<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::ExpressionOrMemory<F>, ExpressionOrMemory> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::ExpressionOrMemory<F>) -> ExpressionOrMemory {
        use crate::proto::acir::circuit::expression_or_memory::*;
        let value = match value {
            circuit::ExpressionOrMemory::Expression(expression) => {
                Value::Expression(Self::encode(expression))
            }
            circuit::ExpressionOrMemory::Memory(block_id) => Value::Memory(block_id.0),
        };
        ExpressionOrMemory { value: Some(value) }
    }

    fn decode(value: &ExpressionOrMemory) -> eyre::Result<circuit::ExpressionOrMemory<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::Opcode<F>, Opcode> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::Opcode<F>) -> Opcode {
        use crate::proto::acir::circuit::opcode::*;
        let value = match value {
            circuit::Opcode::AssertZero(expression) => Value::AssertZero(Self::encode(expression)),
            circuit::Opcode::BlackBoxFuncCall(black_box_func_call) => {
                Value::BlackboxFuncCall(Self::encode(black_box_func_call))
            }
            circuit::Opcode::MemoryOp { block_id, op, predicate } => Value::MemoryOp(MemoryOp {
                block_id: block_id.0,
                op: Self::encode_some(op),
                predicate: predicate.as_ref().map(Self::encode),
            }),
            circuit::Opcode::MemoryInit { block_id, init, block_type } => todo!(),
            circuit::Opcode::BrilligCall { id, inputs, outputs, predicate } => todo!(),
            circuit::Opcode::Call { id, inputs, outputs, predicate } => todo!(),
        };
        Opcode { value: Some(value) }
    }

    fn decode(value: &Opcode) -> eyre::Result<circuit::Opcode<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::opcodes::MemOp<F>, MemOp> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::opcodes::MemOp<F>) -> MemOp {
        MemOp {
            operation: Self::encode_some(&value.operation),
            index: Self::encode_some(&value.index),
            value: Self::encode_some(&value.value),
        }
    }

    fn decode(value: &MemOp) -> eyre::Result<circuit::opcodes::MemOp<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::opcodes::BlackBoxFuncCall<F>, BlackBoxFuncCall> for ProtoSchema<F> {
    fn encode(value: &circuit::opcodes::BlackBoxFuncCall<F>) -> BlackBoxFuncCall {
        todo!()
    }

    fn decode(value: &BlackBoxFuncCall) -> eyre::Result<circuit::opcodes::BlackBoxFuncCall<F>> {
        todo!()
    }
}
