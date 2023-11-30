use super::super::CompileError;
use acir::{
    circuit::{opcodes::BlackBoxFuncCall, Circuit, Opcode},
    native_types::Expression,
};

/// The initial transformer to act on a [`Circuit`]. This replaces any unsupported opcodes with
/// fallback implementations consisting of well supported opcodes.
pub(crate) struct FallbackTransformer;

impl FallbackTransformer {
    //ACIR pass which replace unsupported opcodes using arithmetic fallback
    pub(crate) fn transform(
        acir: Circuit,
        is_supported: impl Fn(&Opcode) -> bool,
        opcode_positions: Vec<usize>,
    ) -> Result<(Circuit, Vec<usize>), CompileError> {
        let mut acir_supported_opcodes = Vec::with_capacity(acir.opcodes.len());
        let mut new_opcode_positions = Vec::with_capacity(opcode_positions.len());
        let mut witness_idx = acir.current_witness_index + 1;

        for (idx, opcode) in acir.opcodes.into_iter().enumerate() {
            match &opcode {
                Opcode::Arithmetic(_) | Opcode::Directive(_) | Opcode::Brillig(_) => {
                    // directive, arithmetic expression or blocks are handled by acvm
                    new_opcode_positions.push(opcode_positions[idx]);
                    acir_supported_opcodes.push(opcode);
                    continue;
                }
                Opcode::MemoryInit { .. } | Opcode::MemoryOp { .. } => {
                    if !is_supported(&opcode) {
                        return Err(CompileError::UnsupportedMemoryOpcode(
                            opcode.unsupported_opcode(),
                        ));
                    }
                    new_opcode_positions.push(opcode_positions[idx]);
                    acir_supported_opcodes.push(opcode);
                }
                Opcode::BlackBoxFuncCall(bb_func_call) => {
                    // We know it is an black box function. Now check if it is
                    // supported by the backend. If it is supported, then we can simply
                    // collect the opcode
                    if is_supported(&opcode) {
                        new_opcode_positions.push(opcode_positions[idx]);
                        acir_supported_opcodes.push(opcode);
                        continue;
                    } else {
                        // If we get here then we know that this black box function is not supported
                        // so we need to replace it with a version of the opcode which only uses arithmetic
                        // expressions
                        let (updated_witness_index, opcodes_fallback) =
                            Self::opcode_fallback(bb_func_call, witness_idx)?;
                        witness_idx = updated_witness_index;
                        new_opcode_positions
                            .extend(vec![opcode_positions[idx]; opcodes_fallback.len()]);
                        acir_supported_opcodes.extend(opcodes_fallback);
                    }
                }
            }
        }

        Ok((
            Circuit {
                current_witness_index: witness_idx - 1,
                opcodes: acir_supported_opcodes,
                ..acir
            },
            new_opcode_positions,
        ))
    }

    fn opcode_fallback(
        gc: &BlackBoxFuncCall,
        current_witness_idx: u32,
    ) -> Result<(u32, Vec<Opcode>), CompileError> {
        let (updated_witness_index, opcodes_fallback) = match gc {
            BlackBoxFuncCall::AND { lhs, rhs, output } => {
                assert_eq!(
                    lhs.num_bits, rhs.num_bits,
                    "number of bits specified for each input must be the same"
                );
                stdlib::blackbox_fallbacks::and(
                    Expression::from(lhs.witness),
                    Expression::from(rhs.witness),
                    *output,
                    lhs.num_bits,
                    current_witness_idx,
                )
            }
            BlackBoxFuncCall::XOR { lhs, rhs, output } => {
                assert_eq!(
                    lhs.num_bits, rhs.num_bits,
                    "number of bits specified for each input must be the same"
                );
                stdlib::blackbox_fallbacks::xor(
                    Expression::from(lhs.witness),
                    Expression::from(rhs.witness),
                    *output,
                    lhs.num_bits,
                    current_witness_idx,
                )
            }
            BlackBoxFuncCall::RANGE { input } => {
                // Note there are no outputs because range produces no outputs
                stdlib::blackbox_fallbacks::range(
                    Expression::from(input.witness),
                    input.num_bits,
                    current_witness_idx,
                )
            }
            #[cfg(feature = "unstable-fallbacks")]
            BlackBoxFuncCall::SHA256 { inputs, outputs } => {
                let sha256_inputs =
                    inputs.iter().map(|input| (input.witness.into(), input.num_bits)).collect();
                stdlib::blackbox_fallbacks::sha256(
                    sha256_inputs,
                    outputs.to_vec(),
                    current_witness_idx,
                )
            }
            #[cfg(feature = "unstable-fallbacks")]
            BlackBoxFuncCall::Blake2s { inputs, outputs } => {
                let blake2s_inputs =
                    inputs.iter().map(|input| (input.witness.into(), input.num_bits)).collect();
                stdlib::blackbox_fallbacks::blake2s(
                    blake2s_inputs,
                    outputs.to_vec(),
                    current_witness_idx,
                )
            }
            #[cfg(feature = "unstable-fallbacks")]
            BlackBoxFuncCall::HashToField128Security { inputs, output } => {
                let hash_to_field_inputs =
                    inputs.iter().map(|input| (input.witness.into(), input.num_bits)).collect();
                stdlib::blackbox_fallbacks::hash_to_field(
                    hash_to_field_inputs,
                    *output,
                    current_witness_idx,
                )
            }
            #[cfg(feature = "unstable-fallbacks")]
            BlackBoxFuncCall::Keccak256 { inputs, outputs } => {
                let keccak_inputs =
                    inputs.iter().map(|input| (input.witness.into(), input.num_bits)).collect();
                stdlib::blackbox_fallbacks::keccak256(
                    keccak_inputs,
                    outputs.to_vec(),
                    current_witness_idx,
                )
            }
            _ => {
                return Err(CompileError::UnsupportedBlackBox(gc.get_black_box_func()));
            }
        };

        Ok((updated_witness_index, opcodes_fallback))
    }
}
