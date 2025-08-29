use crate::{
    circuit::{
        self,
        brillig::BrilligFunctionId,
        opcodes::{self, AcirFunctionId, BlockId},
    },
    proto::acir::circuit::{
        AssertMessage, AssertionPayload, BlackBoxFuncCall, BlockType, BrilligInputs,
        BrilligOutputs, Circuit, ExpressionOrMemory, ExpressionWidth, FunctionInput, MemOp, Opcode,
        OpcodeLocation,
    },
};
use acir_field::AcirField;
use color_eyre::eyre::{self};
use noir_protobuf::{ProtoCodec, decode_oneof_map};

use super::ProtoSchema;

impl<F: AcirField> ProtoCodec<circuit::Circuit<F>, Circuit> for ProtoSchema<F> {
    fn encode(value: &circuit::Circuit<F>) -> Circuit {
        Circuit {
            function_name: value.function_name.clone(),
            current_witness_index: value.current_witness_index,
            opcodes: Self::encode_vec(&value.opcodes),
            private_parameters: Self::encode_vec(value.private_parameters.iter()),
            public_parameters: Self::encode_vec(value.public_parameters.0.iter()),
            return_values: Self::encode_vec(value.return_values.0.iter()),
            assert_messages: Self::encode_vec(&value.assert_messages),
        }
    }

    fn decode(value: &Circuit) -> eyre::Result<circuit::Circuit<F>> {
        Ok(circuit::Circuit {
            function_name: value.function_name.clone(),
            current_witness_index: value.current_witness_index,
            opcodes: Self::decode_vec_wrap(&value.opcodes, "opcodes")?,
            private_parameters: Self::decode_vec_wrap(
                &value.private_parameters,
                "private_parameters",
            )?
            .into_iter()
            .collect(),
            public_parameters: circuit::PublicInputs(
                Self::decode_vec_wrap(&value.public_parameters, "public_parameters")?
                    .into_iter()
                    .collect(),
            ),
            return_values: circuit::PublicInputs(
                Self::decode_vec_wrap(&value.return_values, "return_values")?.into_iter().collect(),
            ),
            assert_messages: Self::decode_vec_wrap(&value.assert_messages, "assert_messages")?,
        })
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
        use crate::proto::acir::circuit::expression_width::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Unbounded(_) => Ok(circuit::ExpressionWidth::Unbounded),
            Value::Bounded(v) => Ok(circuit::ExpressionWidth::Bounded {
                width: Self::decode_wrap(&v.width, "width")?,
            }),
        })
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
        let location = Self::decode_some_wrap(&value.location, "location")?;
        let payload = Self::decode_some_wrap(&value.payload, "payload")?;
        Ok((location, payload))
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
        use crate::proto::acir::circuit::opcode_location::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Acir(location) => {
                Ok(circuit::OpcodeLocation::Acir(Self::decode_wrap(location, "location")?))
            }
            Value::Brillig(location) => Ok(circuit::OpcodeLocation::Brillig {
                acir_index: Self::decode_wrap(&location.acir_index, "acir_index")?,
                brillig_index: Self::decode_wrap(&location.brillig_index, "brillig_index")?,
            }),
        })
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
        Ok(circuit::AssertionPayload {
            error_selector: value.error_selector,
            payload: Self::decode_vec_wrap(&value.payload, "payload")?,
        })
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
        use crate::proto::acir::circuit::expression_or_memory::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Expression(expression) => Ok(circuit::ExpressionOrMemory::Expression(
                Self::decode_wrap(expression, "expression")?,
            )),
            Value::Memory(id) => Ok(circuit::ExpressionOrMemory::Memory(BlockId(*id))),
        })
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
            circuit::Opcode::MemoryOp { block_id, op } => Value::MemoryOp(
                #[allow(deprecated)]
                MemoryOp { block_id: block_id.0, op: Self::encode_some(op), predicate: None },
            ),
            circuit::Opcode::MemoryInit { block_id, init, block_type } => {
                Value::MemoryInit(MemoryInit {
                    block_id: block_id.0,
                    init: Self::encode_vec(init),
                    block_type: Self::encode_some(block_type),
                })
            }
            circuit::Opcode::BrilligCall { id, inputs, outputs, predicate } => {
                Value::BrilligCall(BrilligCall {
                    id: id.0,
                    inputs: Self::encode_vec(inputs),
                    outputs: Self::encode_vec(outputs),
                    predicate: predicate.as_ref().map(Self::encode),
                })
            }
            circuit::Opcode::Call { id, inputs, outputs, predicate } => Value::Call(Call {
                id: id.0,
                inputs: Self::encode_vec(inputs),
                outputs: Self::encode_vec(outputs),
                predicate: predicate.as_ref().map(Self::encode),
            }),
        };
        Opcode { value: Some(value) }
    }

    fn decode(value: &Opcode) -> eyre::Result<circuit::Opcode<F>> {
        use crate::proto::acir::circuit::opcode::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::AssertZero(expression) => {
                Ok(circuit::Opcode::AssertZero(Self::decode_wrap(expression, "assert_zero")?))
            }
            Value::BlackboxFuncCall(black_box_func_call) => Ok(circuit::Opcode::BlackBoxFuncCall(
                Self::decode_wrap(black_box_func_call, "blackbox_func_call")?,
            )),
            Value::MemoryOp(memory_op) => Ok(circuit::Opcode::MemoryOp {
                block_id: BlockId(memory_op.block_id),
                op: Self::decode_some_wrap(&memory_op.op, "op")?,
            }),
            Value::MemoryInit(memory_init) => Ok(circuit::Opcode::MemoryInit {
                block_id: BlockId(memory_init.block_id),
                init: Self::decode_vec_wrap(&memory_init.init, "init")?,
                block_type: Self::decode_some_wrap(&memory_init.block_type, "block_type")?,
            }),
            Value::BrilligCall(brillig_call) => Ok(circuit::Opcode::BrilligCall {
                id: BrilligFunctionId(brillig_call.id),
                inputs: Self::decode_vec_wrap(&brillig_call.inputs, "inputs")?,
                outputs: Self::decode_vec_wrap(&brillig_call.outputs, "outputs")?,
                predicate: Self::decode_opt_wrap(&brillig_call.predicate, "predicate")?,
            }),
            Value::Call(call) => Ok(circuit::Opcode::Call {
                id: AcirFunctionId(call.id),
                inputs: Self::decode_vec_wrap(&call.inputs, "inputs")?,
                outputs: Self::decode_vec_wrap(&call.outputs, "outputs")?,
                predicate: Self::decode_opt_wrap(&call.predicate, "predicate")?,
            }),
        })
    }
}

impl<F> ProtoCodec<opcodes::MemOp<F>, MemOp> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &opcodes::MemOp<F>) -> MemOp {
        MemOp {
            operation: Self::encode_some(&value.operation),
            index: Self::encode_some(&value.index),
            value: Self::encode_some(&value.value),
        }
    }

    fn decode(value: &MemOp) -> eyre::Result<opcodes::MemOp<F>> {
        Ok(opcodes::MemOp {
            operation: Self::decode_some_wrap(&value.operation, "operation")?,
            index: Self::decode_some_wrap(&value.index, "index")?,
            value: Self::decode_some_wrap(&value.value, "value")?,
        })
    }
}

impl<F> ProtoCodec<opcodes::BlackBoxFuncCall<F>, BlackBoxFuncCall> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &opcodes::BlackBoxFuncCall<F>) -> BlackBoxFuncCall {
        use crate::proto::acir::circuit::black_box_func_call::*;
        let value = match value {
            opcodes::BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs } => {
                Value::Aes128Encrypt(Aes128Encrypt {
                    inputs: Self::encode_vec(inputs),
                    iv: Self::encode_vec(iv.as_ref()),
                    key: Self::encode_vec(key.as_ref()),
                    outputs: Self::encode_vec(outputs),
                })
            }
            opcodes::BlackBoxFuncCall::AND { lhs, rhs, num_bits, output } => Value::And(And {
                lhs: Self::encode_some(lhs),
                rhs: Self::encode_some(rhs),
                num_bits: *num_bits,
                output: Self::encode_some(output),
            }),
            opcodes::BlackBoxFuncCall::XOR { lhs, rhs, num_bits, output } => Value::Xor(Xor {
                lhs: Self::encode_some(lhs),
                rhs: Self::encode_some(rhs),
                num_bits: *num_bits,
                output: Self::encode_some(output),
            }),
            opcodes::BlackBoxFuncCall::RANGE { input, num_bits } => {
                Value::Range(Range { input: Self::encode_some(input), num_bits: *num_bits })
            }
            opcodes::BlackBoxFuncCall::Blake2s { inputs, outputs } => Value::Blake2s(Blake2s {
                inputs: Self::encode_vec(inputs),
                outputs: Self::encode_vec(outputs.as_ref()),
            }),
            opcodes::BlackBoxFuncCall::Blake3 { inputs, outputs } => Value::Blake3(Blake3 {
                inputs: Self::encode_vec(inputs),
                outputs: Self::encode_vec(outputs.as_ref()),
            }),
            opcodes::BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                output,
                predicate,
            } => Value::EcdsaSecp256k1(EcdsaSecp256k1 {
                public_key_x: Self::encode_vec(public_key_x.as_ref()),
                public_key_y: Self::encode_vec(public_key_y.as_ref()),
                signature: Self::encode_vec(signature.as_ref()),
                hashed_message: Self::encode_vec(hashed_message.as_ref()),
                output: Self::encode_some(output),
                predicate: Self::encode_some(predicate),
            }),
            opcodes::BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                output,
                predicate,
            } => Value::EcdsaSecp256r1(EcdsaSecp256r1 {
                public_key_x: Self::encode_vec(public_key_x.as_ref()),
                public_key_y: Self::encode_vec(public_key_y.as_ref()),
                signature: Self::encode_vec(signature.as_ref()),
                hashed_message: Self::encode_vec(hashed_message.as_ref()),
                output: Self::encode_some(output),
                predicate: Self::encode_some(predicate),
            }),
            opcodes::BlackBoxFuncCall::MultiScalarMul { points, scalars, predicate, outputs } => {
                let (w1, w2, w3) = outputs;
                Value::MultiScalarMul(MultiScalarMul {
                    points: Self::encode_vec(points),
                    scalars: Self::encode_vec(scalars),
                    predicate: Self::encode_some(predicate),
                    outputs: Self::encode_vec([w1, w2, w3]),
                })
            }
            opcodes::BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, predicate, outputs } => {
                let (w1, w2, w3) = outputs;
                Value::EmbeddedCurveAdd(EmbeddedCurveAdd {
                    input1: Self::encode_vec(input1.as_ref()),
                    input2: Self::encode_vec(input2.as_ref()),
                    predicate: Self::encode_some(predicate),
                    outputs: Self::encode_vec([w1, w2, w3]),
                })
            }
            opcodes::BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
                Value::KeccakF1600(Keccakf1600 {
                    inputs: Self::encode_vec(inputs.as_ref()),
                    outputs: Self::encode_vec(outputs.as_ref()),
                })
            }
            opcodes::BlackBoxFuncCall::RecursiveAggregation {
                verification_key,
                proof,
                public_inputs,
                key_hash,
                proof_type,
                predicate,
            } => Value::RecursiveAggregation(RecursiveAggregation {
                verification_key: Self::encode_vec(verification_key),
                proof: Self::encode_vec(proof),
                public_inputs: Self::encode_vec(public_inputs),
                key_hash: Self::encode_some(key_hash),
                proof_type: *proof_type,
                predicate: Self::encode_some(predicate),
            }),
            opcodes::BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs } => {
                Value::Poseidon2Permutation(Poseidon2Permutation {
                    inputs: Self::encode_vec(inputs),
                    outputs: Self::encode_vec(outputs),
                })
            }
            opcodes::BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
                Value::Sha256Compression(Sha256Compression {
                    inputs: Self::encode_vec(inputs.as_ref()),
                    hash_values: Self::encode_vec(hash_values.as_ref()),
                    outputs: Self::encode_vec(outputs.as_ref()),
                })
            }
        };
        BlackBoxFuncCall { value: Some(value) }
    }

    fn decode(value: &BlackBoxFuncCall) -> eyre::Result<opcodes::BlackBoxFuncCall<F>> {
        use crate::proto::acir::circuit::black_box_func_call::*;
        decode_oneof_map(
            &value.value,
            |value| -> Result<opcodes::BlackBoxFuncCall<F>, eyre::Error> {
                match value {
                    Value::Aes128Encrypt(v) => Ok(opcodes::BlackBoxFuncCall::AES128Encrypt {
                        inputs: Self::decode_vec_wrap(&v.inputs, "inputs")?,
                        iv: Self::decode_box_arr_wrap(&v.iv, "iv")?,
                        key: Self::decode_box_arr_wrap(&v.key, "key")?,
                        outputs: Self::decode_vec_wrap(&v.outputs, "witness")?,
                    }),
                    Value::And(v) => Ok(opcodes::BlackBoxFuncCall::AND {
                        lhs: Self::decode_some_wrap(&v.lhs, "lhs")?,
                        rhs: Self::decode_some_wrap(&v.rhs, "rhs")?,
                        num_bits: v.num_bits,
                        output: Self::decode_some_wrap(&v.output, "output")?,
                    }),
                    Value::Xor(v) => Ok(opcodes::BlackBoxFuncCall::XOR {
                        lhs: Self::decode_some_wrap(&v.lhs, "lhs")?,
                        rhs: Self::decode_some_wrap(&v.rhs, "rhs")?,
                        num_bits: v.num_bits,
                        output: Self::decode_some_wrap(&v.output, "output")?,
                    }),
                    Value::Range(v) => Ok(opcodes::BlackBoxFuncCall::RANGE {
                        input: Self::decode_some_wrap(&v.input, "input")?,
                        num_bits: v.num_bits,
                    }),
                    Value::Blake2s(v) => Ok(opcodes::BlackBoxFuncCall::Blake2s {
                        inputs: Self::decode_vec_wrap(&v.inputs, "inputs")?,
                        outputs: Self::decode_box_arr_wrap(&v.outputs, "outputs")?,
                    }),
                    Value::Blake3(v) => Ok(opcodes::BlackBoxFuncCall::Blake3 {
                        inputs: Self::decode_vec_wrap(&v.inputs, "inputs")?,
                        outputs: Self::decode_box_arr_wrap(&v.outputs, "outputs")?,
                    }),
                    Value::EcdsaSecp256k1(v) => Ok(opcodes::BlackBoxFuncCall::EcdsaSecp256k1 {
                        public_key_x: Self::decode_box_arr_wrap(&v.public_key_x, "public_key_x")?,
                        public_key_y: Self::decode_box_arr_wrap(&v.public_key_y, "public_key_y")?,
                        signature: Self::decode_box_arr_wrap(&v.signature, "signature")?,
                        hashed_message: Self::decode_box_arr_wrap(
                            &v.hashed_message,
                            "hashed_message",
                        )?,
                        output: Self::decode_some_wrap(&v.output, "output")?,
                        predicate: Self::decode_some_wrap(&v.predicate, "predicate")?,
                    }),
                    Value::EcdsaSecp256r1(v) => Ok(opcodes::BlackBoxFuncCall::EcdsaSecp256r1 {
                        public_key_x: Self::decode_box_arr_wrap(&v.public_key_x, "public_key_x")?,
                        public_key_y: Self::decode_box_arr_wrap(&v.public_key_y, "public_key_y")?,
                        signature: Self::decode_box_arr_wrap(&v.signature, "signature")?,
                        hashed_message: Self::decode_box_arr_wrap(
                            &v.hashed_message,
                            "hashed_message",
                        )?,
                        output: Self::decode_some_wrap(&v.output, "output")?,
                        predicate: Self::decode_some_wrap(&v.predicate, "predicate")?,
                    }),
                    Value::MultiScalarMul(v) => Ok(opcodes::BlackBoxFuncCall::MultiScalarMul {
                        points: Self::decode_vec_wrap(&v.points, "points")?,
                        scalars: Self::decode_vec_wrap(&v.scalars, "scalars")?,
                        predicate: Self::decode_some_wrap(&v.predicate, "predicate")?,
                        outputs: Self::decode_arr_wrap(&v.outputs, "outputs")
                            .map(|[w1, w2, w3]| (w1, w2, w3))?,
                    }),
                    Value::EmbeddedCurveAdd(v) => Ok(opcodes::BlackBoxFuncCall::EmbeddedCurveAdd {
                        input1: Self::decode_box_arr_wrap(&v.input1, "input1")?,
                        input2: Self::decode_box_arr_wrap(&v.input2, "input2")?,
                        predicate: Self::decode_some_wrap(&v.predicate, "predicate")?,
                        outputs: Self::decode_arr_wrap(&v.outputs, "outputs")
                            .map(|[w1, w2, w3]| (w1, w2, w3))?,
                    }),
                    Value::KeccakF1600(v) => Ok(opcodes::BlackBoxFuncCall::Keccakf1600 {
                        inputs: Self::decode_box_arr_wrap(&v.inputs, "inputs")?,
                        outputs: Self::decode_box_arr_wrap(&v.outputs, "outputs")?,
                    }),
                    Value::RecursiveAggregation(v) => {
                        Ok(opcodes::BlackBoxFuncCall::RecursiveAggregation {
                            verification_key: Self::decode_vec_wrap(
                                &v.verification_key,
                                "verification_key",
                            )?,
                            proof: Self::decode_vec_wrap(&v.proof, "proof")?,
                            public_inputs: Self::decode_vec_wrap(
                                &v.public_inputs,
                                "public_inputs",
                            )?,
                            key_hash: Self::decode_some_wrap(&v.key_hash, "key_hash")?,
                            proof_type: v.proof_type,
                            predicate: Self::decode_some_wrap(&v.predicate, "predicate")?,
                        })
                    }
                    Value::Poseidon2Permutation(v) => {
                        Ok(opcodes::BlackBoxFuncCall::Poseidon2Permutation {
                            inputs: Self::decode_vec_wrap(&v.inputs, "inputs")?,
                            outputs: Self::decode_vec_wrap(&v.outputs, "outputs")?,
                        })
                    }
                    Value::Sha256Compression(v) => {
                        Ok(opcodes::BlackBoxFuncCall::Sha256Compression {
                            inputs: Self::decode_box_arr_wrap(&v.inputs, "inputs")?,
                            hash_values: Self::decode_box_arr_wrap(&v.hash_values, "hash_values")?,
                            outputs: Self::decode_box_arr_wrap(&v.outputs, "outputs")?,
                        })
                    }
                }
            },
        )
    }
}

impl<F> ProtoCodec<opcodes::FunctionInput<F>, FunctionInput> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &opcodes::FunctionInput<F>) -> FunctionInput {
        use crate::proto::acir::circuit::function_input::*;
        let value = match value {
            opcodes::FunctionInput::Constant(field) => Value::Constant(Self::encode(field)),
            opcodes::FunctionInput::Witness(witness) => Value::Witness(Self::encode(witness)),
        };
        FunctionInput { value: Some(value) }
    }

    fn decode(value: &FunctionInput) -> eyre::Result<opcodes::FunctionInput<F>> {
        use crate::proto::acir::circuit::function_input::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Constant(field) => {
                Ok(opcodes::FunctionInput::Constant(Self::decode_wrap(field, "constant")?))
            }
            Value::Witness(witness) => {
                Ok(opcodes::FunctionInput::Witness(Self::decode_wrap(witness, "witness")?))
            }
        })
    }
}

impl<F> ProtoCodec<opcodes::BlockType, BlockType> for ProtoSchema<F> {
    fn encode(value: &opcodes::BlockType) -> BlockType {
        use crate::proto::acir::circuit::block_type::*;
        let value = match value {
            opcodes::BlockType::Memory => Value::Memory(Memory {}),
            opcodes::BlockType::CallData(value) => Value::CallData(CallData { value: *value }),
            opcodes::BlockType::ReturnData => Value::ReturnData(ReturnData {}),
        };
        BlockType { value: Some(value) }
    }

    fn decode(value: &BlockType) -> eyre::Result<opcodes::BlockType> {
        use crate::proto::acir::circuit::block_type::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Memory(_) => Ok(opcodes::BlockType::Memory),
            Value::CallData(v) => Ok(opcodes::BlockType::CallData(v.value)),
            Value::ReturnData(_) => Ok(opcodes::BlockType::ReturnData),
        })
    }
}

impl<F> ProtoCodec<circuit::brillig::BrilligInputs<F>, BrilligInputs> for ProtoSchema<F>
where
    F: AcirField,
{
    fn encode(value: &circuit::brillig::BrilligInputs<F>) -> BrilligInputs {
        use crate::proto::acir::circuit::brillig_inputs::*;
        let value = match value {
            circuit::brillig::BrilligInputs::Single(expression) => {
                Value::Single(Self::encode(expression))
            }
            circuit::brillig::BrilligInputs::Array(expressions) => {
                Value::Array(Array { values: Self::encode_vec(expressions) })
            }
            circuit::brillig::BrilligInputs::MemoryArray(block_id) => {
                Value::MemoryArray(block_id.0)
            }
        };
        BrilligInputs { value: Some(value) }
    }

    fn decode(value: &BrilligInputs) -> eyre::Result<circuit::brillig::BrilligInputs<F>> {
        use crate::proto::acir::circuit::brillig_inputs::*;
        decode_oneof_map(&value.value, |value| match value {
            Value::Single(expression) => Ok(circuit::brillig::BrilligInputs::Single(
                Self::decode_wrap(expression, "single")?,
            )),
            Value::Array(array) => Ok(circuit::brillig::BrilligInputs::Array(
                Self::decode_vec_wrap(&array.values, "array")?,
            )),
            Value::MemoryArray(id) => {
                Ok(circuit::brillig::BrilligInputs::MemoryArray(BlockId(*id)))
            }
        })
    }
}

impl<F> ProtoCodec<circuit::brillig::BrilligOutputs, BrilligOutputs> for ProtoSchema<F> {
    fn encode(value: &circuit::brillig::BrilligOutputs) -> BrilligOutputs {
        use crate::proto::acir::circuit::brillig_outputs::*;
        let value = match value {
            circuit::brillig::BrilligOutputs::Simple(witness) => {
                Value::Simple(Self::encode(witness))
            }
            circuit::brillig::BrilligOutputs::Array(witnesses) => {
                Value::Array(Array { values: Self::encode_vec(witnesses) })
            }
        };
        BrilligOutputs { value: Some(value) }
    }

    fn decode(value: &BrilligOutputs) -> eyre::Result<circuit::brillig::BrilligOutputs> {
        use crate::proto::acir::circuit::brillig_outputs::*;

        decode_oneof_map(&value.value, |value| match value {
            Value::Simple(witness) => {
                Ok(circuit::brillig::BrilligOutputs::Simple(Self::decode_wrap(witness, "simple")?))
            }
            Value::Array(array) => Ok(circuit::brillig::BrilligOutputs::Array(
                Self::decode_vec_wrap(&array.values, "array")?,
            )),
        })
    }
}
