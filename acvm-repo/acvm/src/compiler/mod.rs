use std::collections::HashMap;

use acir::{
    circuit::{AssertionPayload, Circuit, ExpressionWidth, OpcodeLocation},
    AcirField,
};

// The various passes that we can use over ACIR
mod optimizers;
mod simulator;
mod transformers;

pub use optimizers::optimize;
use optimizers::optimize_internal;
pub use simulator::CircuitSimulator;
use transformers::transform_internal;
pub use transformers::{transform, MIN_EXPRESSION_WIDTH};

/// We need multiple passes to stabilize the output.
/// The value was determined by running tests.
const MAX_OPTIMIZER_PASSES: usize = 3;

/// This module moves and decomposes acir opcodes. The transformation map allows consumers of this module to map
/// metadata they had about the opcodes to the new opcode structure generated after the transformation.
#[derive(Debug)]
pub struct AcirTransformationMap {
    /// Maps the old acir indices to the new acir indices
    old_indices_to_new_indices: HashMap<usize, Vec<usize>>,
}

impl AcirTransformationMap {
    /// Builds a map from a vector of pointers to the old acir opcodes.
    /// The index of the vector is the new opcode index.
    /// The value of the vector is the old opcode index pointed.
    fn new(acir_opcode_positions: &[usize]) -> Self {
        let mut old_indices_to_new_indices = HashMap::with_capacity(acir_opcode_positions.len());
        for (new_index, old_index) in acir_opcode_positions.iter().copied().enumerate() {
            old_indices_to_new_indices.entry(old_index).or_insert_with(Vec::new).push(new_index);
        }
        AcirTransformationMap { old_indices_to_new_indices }
    }

    pub fn new_locations(
        &self,
        old_location: OpcodeLocation,
    ) -> impl Iterator<Item = OpcodeLocation> + '_ {
        let old_acir_index = match old_location {
            OpcodeLocation::Acir(index) => index,
            OpcodeLocation::Brillig { acir_index, .. } => acir_index,
        };

        self.old_indices_to_new_indices.get(&old_acir_index).into_iter().flat_map(
            move |new_indices| {
                new_indices.iter().map(move |new_index| match old_location {
                    OpcodeLocation::Acir(_) => OpcodeLocation::Acir(*new_index),
                    OpcodeLocation::Brillig { brillig_index, .. } => {
                        OpcodeLocation::Brillig { acir_index: *new_index, brillig_index }
                    }
                })
            },
        )
    }
}

fn transform_assert_messages<F: Clone>(
    assert_messages: Vec<(OpcodeLocation, AssertionPayload<F>)>,
    map: &AcirTransformationMap,
) -> Vec<(OpcodeLocation, AssertionPayload<F>)> {
    assert_messages
        .into_iter()
        .flat_map(|(location, message)| {
            let new_locations = map.new_locations(location);
            new_locations.into_iter().map(move |new_location| (new_location, message.clone()))
        })
        .collect()
}

/// Applies [`ProofSystemCompiler`][crate::ProofSystemCompiler] specific optimizations to a [`Circuit`].
///
/// Runs multiple passes until the output stabilizes.
pub fn compile<F: AcirField>(
    acir: Circuit<F>,
    expression_width: ExpressionWidth,
) -> (Circuit<F>, AcirTransformationMap) {
    if MAX_OPTIMIZER_PASSES == 0 {
        let acir_opcode_positions = (0..acir.opcodes.len()).collect::<Vec<_>>();
        let transformation_map = AcirTransformationMap::new(&acir_opcode_positions);
        return (acir, transformation_map);
    }
    let mut pass = 0;
    let mut prev_opcodes_hash = fxhash::hash64(&acir.opcodes);
    let mut prev_acir = acir;

    // For most test programs it would be enough to only loop `transform_internal`,
    // but some of them don't stabilize unless we also repeat the backend agnostic optimizations.
    let (mut acir, acir_opcode_positions) = loop {
        let (acir, acir_opcode_positions) = optimize_internal(prev_acir);

        // Stop if we have already done at least one transform and an extra optimization changed nothing.
        if pass > 0 && prev_opcodes_hash == fxhash::hash64(&acir.opcodes) {
            break (acir, acir_opcode_positions);
        }

        let (acir, acir_opcode_positions) =
            transform_internal(acir, expression_width, acir_opcode_positions);

        let opcodes_hash = fxhash::hash64(&acir.opcodes);

        // Stop if the output hasn't change in this loop or we went too long.
        if pass == MAX_OPTIMIZER_PASSES - 1 || prev_opcodes_hash == opcodes_hash {
            break (acir, acir_opcode_positions);
        }

        pass += 1;
        prev_acir = acir;
        prev_opcodes_hash = opcodes_hash;
    };

    let transformation_map = AcirTransformationMap::new(&acir_opcode_positions);
    acir.assert_messages = transform_assert_messages(acir.assert_messages, &transformation_map);

    (acir, transformation_map)
}
