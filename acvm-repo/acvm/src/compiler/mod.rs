//! The `compiler` module contains several passes to transform an ACIR program.
//! Roughly, the passes are separated into the `optimizers` which try to reduce the number of opcodes
//! and the `transformers` which adapt the opcodes to the proving backend.
//!
//! # Optimizers
//! - GeneralOptimizer: simple pass which simplifies AssertZero opcodes when possible (e.g remove terms with null coefficient)
//! - UnusedMemoryOptimizer: simple pass which removes MemoryInit opcodes when they are not used (e.g no corresponding MemoryOp opcode)
//! - RangeOptimizer: forward pass to collect range check information, and backward pass to remove the ones that are redundant.
//!
//! # Transformers
//! - CSAT: create intermediate variables so that AssertZero opcodes have the correct Circuit's `ExpressionWidth`.
//!
//! ACIR generation is performed by calling the `Ssa::into_acir` method, providing any necessary brillig bytecode.
//! The compiled program will be returned as an `Artifacts` type.
//!

use std::collections::{BTreeMap, HashMap};

use acir::{
    AcirField,
    circuit::{
        AcirOpcodeLocation, AssertionPayload, Circuit, OpcodeLocation, brillig::BrilligFunctionId,
    },
};

// The various passes that we can use over ACIR
pub use optimizers::optimize;
mod optimizers;
mod simulator;

pub use simulator::CircuitSimulator;

/// This module can move and decompose acir opcodes into multiple opcodes. The transformation map allows consumers of this module to map
/// metadata they had about the opcodes to the new opcode structure generated after the transformation.
/// ACIR opcodes are stored inside a vector of opcodes. A transformation pass will generate a new vector of opcodes,
/// but each opcode is the result of the transformation of an opcode in the original vector.
/// So we simply keep track of the relation:  index of the original opcode -> index of the new opcode in the new vector
/// However we need a vector of new indexes for the map values in the case the old opcode is decomposed into multiple opcodes.
#[derive(Debug)]
pub struct AcirTransformationMap {
    /// Maps the old acir indices to the new acir indices
    old_indices_to_new_indices: HashMap<usize, Vec<usize>>,
}

impl AcirTransformationMap {
    /// Builds a map from a vector of pointers to the old acir opcodes.
    /// The index in the vector is the new opcode index.
    /// The value of the vector is where the old opcode index was pointed.
    /// E.g: If acir_opcode_positions = 0,1,2,4,5,5,6
    /// that means that old indices 0,1,2,4,5,5,6 are mapped to the new indexes: 0,1,2,3,4,5,6
    /// This gives the following map:
    /// 0 -> 0
    /// 1 -> 1
    /// 2 -> 2
    /// 4 -> 3
    /// 5 -> [4, 5]
    /// 6 -> 6
    fn new(acir_opcode_positions: &[usize]) -> Self {
        let mut old_indices_to_new_indices = HashMap::with_capacity(acir_opcode_positions.len());
        for (new_index, old_index) in acir_opcode_positions.iter().copied().enumerate() {
            old_indices_to_new_indices.entry(old_index).or_insert_with(Vec::new).push(new_index);
        }
        AcirTransformationMap { old_indices_to_new_indices }
    }

    /// Returns the new opcode location(s) corresponding to the old opcode.
    /// An OpcodeLocation contains the index of the opcode in the vector of opcodes
    /// This function returns the new OpcodeLocation by 'updating' the index within the given OpcodeLocation
    /// using the AcirTransformationMap. In fact, it does not update the given OpcodeLocation 'in-memory' but rather
    /// returns a new one, and even a vector of OpcodeLocation's in case there are multiple new indexes corresponding
    /// to the old opcode index.
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

    /// This function is similar to `new_locations()`, but only deals with
    /// the AcirOpcodeLocation variant
    pub fn new_acir_locations(
        &self,
        old_location: AcirOpcodeLocation,
    ) -> impl Iterator<Item = AcirOpcodeLocation> + '_ {
        let old_acir_index = old_location.index();

        self.old_indices_to_new_indices.get(&old_acir_index).into_iter().flat_map(
            move |new_indices| {
                new_indices.iter().map(move |new_index| AcirOpcodeLocation::new(*new_index))
            },
        )
    }
}

/// Update the assert messages to point to the new opcode locations.
fn transform_assert_messages<F: Clone>(
    assert_messages: Vec<(OpcodeLocation, AssertionPayload<F>)>,
    map: &AcirTransformationMap,
) -> Vec<(OpcodeLocation, AssertionPayload<F>)> {
    assert_messages
        .into_iter()
        .flat_map(|(location, message)| {
            let new_locations = map.new_locations(location);
            new_locations.map(move |new_location| (new_location, message.clone()))
        })
        .collect()
}

/// Applies backend specific optimizations to a [`Circuit`].
///
/// optimize_internal:
/// - General optimizer: canonicalize AssertZero opcodes.
/// - Unused Memory: remove unused MemoryInit opcodes.
/// - Redundant Ranges: remove RANGE opcodes that are redundant.
///
/// transform_internal: run multiple times (up to 4) until the output stabilizes.
/// - CSAT: limit AssertZero opcodes to the Circuit's width.
/// - Eliminate intermediate variables: Combine AssertZero opcodes used only once.
/// - Redundant Ranges: some RANGEs may be redundant as a side effect of the previous pass.
pub fn compile<F: AcirField>(
    acir: Circuit<F>,
    brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
) -> (Circuit<F>, AcirTransformationMap) {
    optimize(acir, brillig_side_effects)
}

#[macro_export]
macro_rules! assert_circuit_snapshot {
    ($acir:expr, $($arg:tt)*) => {
        #[allow(unused_mut)]
        let acir_string = $acir.to_string();
        insta::assert_snapshot!(acir_string, $($arg)*)
    };
}
