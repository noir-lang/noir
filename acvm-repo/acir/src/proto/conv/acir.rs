use crate::{
    circuit,
    proto::acir::circuit::{AssertMessage, Circuit, ExpressionWidth, Opcode},
};
use color_eyre::eyre;
use noir_protobuf::ProtoCodec;

use super::ProtoSchema;

impl<F> ProtoCodec<circuit::Circuit<F>, Circuit> for ProtoSchema<F> {
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

impl<F> ProtoCodec<circuit::Opcode<F>, Opcode> for ProtoSchema<F> {
    fn encode(value: &circuit::Opcode<F>) -> Opcode {
        todo!()
    }

    fn decode(value: &Opcode) -> eyre::Result<circuit::Opcode<F>> {
        todo!()
    }
}

impl<F> ProtoCodec<circuit::ExpressionWidth, ExpressionWidth> for ProtoSchema<F> {
    fn encode(value: &circuit::ExpressionWidth) -> ExpressionWidth {
        todo!()
    }

    fn decode(value: &ExpressionWidth) -> eyre::Result<circuit::ExpressionWidth> {
        todo!()
    }
}

impl<F> ProtoCodec<(circuit::OpcodeLocation, circuit::AssertionPayload<F>), AssertMessage>
    for ProtoSchema<F>
{
    fn encode(value: &(circuit::OpcodeLocation, circuit::AssertionPayload<F>)) -> AssertMessage {
        todo!()
    }

    fn decode(
        value: &AssertMessage,
    ) -> eyre::Result<(circuit::OpcodeLocation, circuit::AssertionPayload<F>)> {
        todo!()
    }
}
