use acir::{
    circuit::{
        brillig::BrilligOutputs, directives::Directive, opcodes::UnsupportedMemoryOpcode, Circuit,
        Opcode, OpcodeLocation,
    },
    native_types::{Expression, Witness},
    BlackBoxFunc, FieldElement,
};
use indexmap::IndexMap;
use thiserror::Error;

use crate::Language;

// The various passes that we can use over ACIR
mod optimizers;
mod transformers;

use optimizers::{GeneralOptimizer, RangeOptimizer};
use transformers::{CSatTransformer, FallbackTransformer, R1CSTransformer};

#[derive(PartialEq, Eq, Debug, Error)]
pub enum CompileError {
    #[error("The blackbox function {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedBlackBox(BlackBoxFunc),
    #[error("The opcode {0} is not supported by the backend and acvm does not have a fallback implementation")]
    UnsupportedMemoryOpcode(UnsupportedMemoryOpcode),
}

/// This module moves and decomposes acir opcodes. The transformation map allows consumers of this module to map
/// metadata they had about the opcodes to the new opcode structure generated after the transformation.
#[derive(Debug)]
pub struct AcirTransformationMap {
    /// This is a vector of pointers to the old acir opcodes. The index of the vector is the new opcode index.
    /// The value of the vector is the old opcode index pointed.
    acir_opcode_positions: Vec<usize>,
}

impl AcirTransformationMap {
    pub fn new_locations(
        &self,
        old_location: OpcodeLocation,
    ) -> impl Iterator<Item = OpcodeLocation> + '_ {
        let old_acir_index = match old_location {
            OpcodeLocation::Acir(index) => index,
            OpcodeLocation::Brillig { acir_index, .. } => acir_index,
        };

        self.acir_opcode_positions
            .iter()
            .enumerate()
            .filter(move |(_, &old_index)| old_index == old_acir_index)
            .map(move |(new_index, _)| match old_location {
                OpcodeLocation::Acir(_) => OpcodeLocation::Acir(new_index),
                OpcodeLocation::Brillig { brillig_index, .. } => {
                    OpcodeLocation::Brillig { acir_index: new_index, brillig_index }
                }
            })
    }
}

fn transform_assert_messages(
    assert_messages: Vec<(OpcodeLocation, String)>,
    map: &AcirTransformationMap,
) -> Vec<(OpcodeLocation, String)> {
    assert_messages
        .into_iter()
        .flat_map(|(location, message)| {
            let new_locations = map.new_locations(location);
            new_locations.into_iter().map(move |new_location| (new_location, message.clone()))
        })
        .collect()
}

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] specific optimizations to a [`Circuit`].
pub fn compile(
    acir: Circuit,
    np_language: Language,
    is_opcode_supported: impl Fn(&Opcode) -> bool,
) -> Result<(Circuit, AcirTransformationMap), CompileError> {
    // Instantiate the optimizer.
    // Currently the optimizer and reducer are one in the same
    // for CSAT

    // Track original acir opcode positions throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_positions (delete and insert)
    let acir_opcode_positions = acir.opcodes.iter().enumerate().map(|(i, _)| i).collect();

    // Fallback transformer pass
    let (acir, acir_opcode_positions) =
        FallbackTransformer::transform(acir, is_opcode_supported, acir_opcode_positions)?;

    // General optimizer pass
    let mut opcodes: Vec<Opcode> = Vec::new();
    for opcode in acir.opcodes {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                opcodes.push(Opcode::Arithmetic(GeneralOptimizer::optimize(arith_expr)))
            }
            other_opcode => opcodes.push(other_opcode),
        };
    }
    let acir = Circuit { opcodes, ..acir };

    // Range optimization pass
    let range_optimizer = RangeOptimizer::new(acir);
    let (mut acir, acir_opcode_positions) =
        range_optimizer.replace_redundant_ranges(acir_opcode_positions);

    let mut transformer = match &np_language {
        crate::Language::R1CS => {
            let transformation_map = AcirTransformationMap { acir_opcode_positions };
            acir.assert_messages =
                transform_assert_messages(acir.assert_messages, &transformation_map);
            let transformer = R1CSTransformer::new(acir);
            return Ok((transformer.transform(), transformation_map));
        }
        crate::Language::PLONKCSat { width } => {
            let mut csat = CSatTransformer::new(*width);
            for value in acir.circuit_arguments() {
                csat.mark_solvable(value);
            }
            csat
        }
    };

    // TODO: the code below is only for CSAT transformer
    // TODO it may be possible to refactor it in a way that we do not need to return early from the r1cs
    // TODO or at the very least, we could put all of it inside of CSatOptimizer pass

    let mut new_acir_opcode_positions: Vec<usize> = Vec::with_capacity(acir_opcode_positions.len());
    // Optimize the arithmetic gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_opcodes = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    // maps a normalized expression to the intermediate variable which represents the expression, along with its 'norm'
    // the 'norm' is simply the value of the first non zero coefficient in the expression, taken from the linear terms, or quadratic terms if there is none.
    let mut intermediate_variables: IndexMap<Expression, (FieldElement, Witness)> = IndexMap::new();
    for (index, opcode) in acir.opcodes.iter().enumerate() {
        match opcode {
            Opcode::Arithmetic(arith_expr) => {
                let len = intermediate_variables.len();

                let arith_expr = transformer.transform(
                    arith_expr.clone(),
                    &mut intermediate_variables,
                    &mut next_witness_index,
                );

                // Update next_witness counter
                next_witness_index += (intermediate_variables.len() - len) as u32;
                let mut new_opcodes = Vec::new();
                for (g, (norm, w)) in intermediate_variables.iter().skip(len) {
                    // de-normalize
                    let mut intermediate_opcode = g * *norm;
                    // constrain the intermediate opcode to the intermediate variable
                    intermediate_opcode.linear_combinations.push((-FieldElement::one(), *w));
                    intermediate_opcode.sort();
                    new_opcodes.push(intermediate_opcode);
                }
                new_opcodes.push(arith_expr);
                for opcode in new_opcodes {
                    new_acir_opcode_positions.push(acir_opcode_positions[index]);
                    transformed_opcodes.push(Opcode::Arithmetic(opcode));
                }
            }
            Opcode::BlackBoxFuncCall(func) => {
                match func {
                    acir::circuit::opcodes::BlackBoxFuncCall::AND { output, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::XOR { output, .. } => {
                        transformer.mark_solvable(*output)
                    }
                    acir::circuit::opcodes::BlackBoxFuncCall::RANGE { .. } => (),
                    acir::circuit::opcodes::BlackBoxFuncCall::SHA256 { outputs, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Keccak256 { outputs, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Keccak256VariableLength {
                        outputs,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::RecursiveAggregation {
                        output_aggregation_object: outputs,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Blake2s { outputs, .. } => {
                        for witness in outputs {
                            transformer.mark_solvable(*witness);
                        }
                    }
                    acir::circuit::opcodes::BlackBoxFuncCall::FixedBaseScalarMul {
                        outputs,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::Pedersen { outputs, .. } => {
                        transformer.mark_solvable(outputs.0);
                        transformer.mark_solvable(outputs.1)
                    }
                    acir::circuit::opcodes::BlackBoxFuncCall::HashToField128Security {
                        output,
                        ..
                    }
                    | acir::circuit::opcodes::BlackBoxFuncCall::EcdsaSecp256k1 { output, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::EcdsaSecp256r1 { output, .. }
                    | acir::circuit::opcodes::BlackBoxFuncCall::SchnorrVerify { output, .. } => {
                        transformer.mark_solvable(*output)
                    }
                }

                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode.clone());
            }
            Opcode::Directive(directive) => {
                match directive {
                    Directive::Quotient(quotient_directive) => {
                        transformer.mark_solvable(quotient_directive.q);
                        transformer.mark_solvable(quotient_directive.r);
                    }
                    Directive::ToLeRadix { b, .. } => {
                        for witness in b {
                            transformer.mark_solvable(*witness);
                        }
                    }
                    Directive::PermutationSort { bits, .. } => {
                        for witness in bits {
                            transformer.mark_solvable(*witness);
                        }
                    }
                }
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode.clone());
            }
            Opcode::MemoryInit { .. } => {
                // `MemoryInit` does not write values to the `WitnessMap`
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode.clone());
            }
            Opcode::MemoryOp { op, .. } => {
                for (_, witness1, witness2) in &op.value.mul_terms {
                    transformer.mark_solvable(*witness1);
                    transformer.mark_solvable(*witness2);
                }
                for (_, witness) in &op.value.linear_combinations {
                    transformer.mark_solvable(*witness);
                }
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode.clone());
            }
            Opcode::Brillig(brillig) => {
                for output in &brillig.outputs {
                    match output {
                        BrilligOutputs::Simple(w) => transformer.mark_solvable(*w),
                        BrilligOutputs::Array(v) => {
                            for witness in v {
                                transformer.mark_solvable(*witness);
                            }
                        }
                    }
                }
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode.clone());
            }
        }
    }

    let current_witness_index = next_witness_index - 1;

    let transformation_map =
        AcirTransformationMap { acir_opcode_positions: new_acir_opcode_positions };

    let acir = Circuit {
        current_witness_index,
        opcodes: transformed_opcodes,
        // The optimizer does not add new public inputs
        private_parameters: acir.private_parameters,
        public_parameters: acir.public_parameters,
        return_values: acir.return_values,
        assert_messages: transform_assert_messages(acir.assert_messages, &transformation_map),
    };

    Ok((acir, transformation_map))
}
