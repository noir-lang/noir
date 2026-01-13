/// This module applies backend specific transformations to a [`Circuit`].
///
/// ## CSAT: transforms AssertZero opcodes into AssertZero opcodes having the required width.
///
/// For instance, if the width is 4, the AssertZero opcode x1 + x2 + x3 + x4 + x5 - y = 0 will be transformed using 2 intermediate variables (z1,z2):
/// x1 + x2 + x3 = z1
/// x4 + x5 = z2
/// z1 + z2 - y = 0
/// If x1,..x5 are inputs to the program, they are tagged as 'solvable', and would be used to compute the value of y.
/// If we generate the intermediate variable x4 + x5 - y = z3, we get an unsolvable circuit because this AssertZero opcode will have two unknown values: y and z3
/// So the CSAT transformation keeps track of which witnesses would be solved for each opcode in order to only generate solvable intermediate variables.
///
/// ## Eliminate intermediate variables
///
/// The 'eliminate intermediate variables' pass will remove any intermediate variables (for instance created by the previous transformation)
/// that are used in exactly two AssertZero opcodes.
/// This results in arithmetic opcodes having linear combinations of potentially large width.
/// For instance if the intermediate variable is z1 and is only used in y:
/// z1 = x1 + x2 + x3
/// y = z1 + x4
/// We remove it, undoing the work done during the CSAT transformation: y = x1 + x2 + x3 + x4
///
/// We do this because the backend is expected to handle linear combinations of 'unbounded width' in a more efficient way
/// than the 'CSAT transformation'.
/// However, it is worthwhile to compute intermediate variables if they are used in more than one other opcode.
///
/// ## redundant_range
///
/// The 'range optimization' pass, from the optimizers module, will remove any redundant range opcodes.
use std::collections::BTreeMap;

use acir::{
    AcirField,
    circuit::{
        Circuit, Opcode,
        brillig::{BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput, MemOp},
    },
    native_types::{Expression, Witness},
};
use indexmap::IndexMap;

mod csat;
mod merge_expressions;

use csat::CSatTransformer;
use merge_expressions::MergeExpressionsOptimizer;

use std::hash::BuildHasher;
use tracing::info;

use super::RangeOptimizer;

/// We use multiple passes to stabilize the output in many cases
const DEFAULT_MAX_TRANSFORMER_PASSES: usize = 3;

/// Applies backend specific optimizations to a [`Circuit`].
///
/// Accepts an injected `acir_opcode_positions` to allow transformations to be applied directly after optimizations.
///
/// Does multiple passes until the output stabilizes.
///
/// Pre-Conditions:
/// - General Optimizer must run before this pass,
///   when `max_transformer_passes_or_default.unwrap_or(DEFAULT_MAX_TRANSFORMER_PASSES)` is greater than 0
#[tracing::instrument(level = "trace", name = "transform_acir", skip(acir, acir_opcode_positions))]
pub(super) fn transform_internal<F: AcirField>(
    mut acir: Circuit<F>,
    mut acir_opcode_positions: Vec<usize>,
    brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
    max_transformer_passes_or_default: Option<usize>,
) -> (Circuit<F>, Vec<usize>, bool) {
    if acir.opcodes.len() == 1 && matches!(acir.opcodes[0], Opcode::BrilligCall { .. }) {
        info!("Program is fully unconstrained, skipping transformation pass");
        return (acir, acir_opcode_positions, true);
    }

    // Allow multiple passes until we have stable output.
    let mut prev_opcodes_hash = rustc_hash::FxBuildHasher.hash_one(&acir.opcodes);

    // Checking for stable output after MAX_TRANSFORMER_PASSES
    let mut opcodes_hash_stabilized = false;

    let max_transformer_passes =
        max_transformer_passes_or_default.unwrap_or(DEFAULT_MAX_TRANSFORMER_PASSES);

    // For most test programs it would be enough to loop here, but some of them
    // don't stabilize unless we also repeat the backend agnostic optimizations.
    for _ in 0..max_transformer_passes {
        info!("Number of opcodes {}", acir.opcodes.len());
        let (new_acir, new_acir_opcode_positions) =
            transform_internal_once(acir, acir_opcode_positions, brillig_side_effects);

        acir = new_acir;
        acir_opcode_positions = new_acir_opcode_positions;

        let new_opcodes_hash = rustc_hash::FxBuildHasher.hash_one(&acir.opcodes);

        if new_opcodes_hash == prev_opcodes_hash {
            opcodes_hash_stabilized = true;
            break;
        }
        prev_opcodes_hash = new_opcodes_hash;
    }

    // After the elimination of intermediate variables the `current_witness_index` is potentially higher than it needs to be,
    // which would cause gaps if we ran the optimization a second time, making it look like new variables were added.
    acir.current_witness_index = max_witness(&acir).witness_index();

    (acir, acir_opcode_positions, opcodes_hash_stabilized)
}

/// Accepts an injected `acir_opcode_positions` to allow transformations to be applied directly after optimizations.
///
/// If the width is unbounded, it does nothing.
/// If it is bounded, it first performs the 'CSAT transformation' in one pass, by creating intermediate variables when necessary.
/// Then it performs `eliminate_intermediate_variable()` which (re-)combine intermediate variables used only once.
/// It concludes with a round of `replace_redundant_ranges()` which removes range checks made redundant by the previous pass.
///
/// Pre-Conditions:
/// - General Optimizer must run before this pass
#[tracing::instrument(
    level = "trace",
    name = "transform_acir_once",
    skip(acir, acir_opcode_positions)
)]
fn transform_internal_once<F: AcirField>(
    mut acir: Circuit<F>,
    acir_opcode_positions: Vec<usize>,
    brillig_side_effects: &BTreeMap<BrilligFunctionId, bool>,
) -> (Circuit<F>, Vec<usize>) {
    // 1. CSAT transformation
    // Process each opcode in the circuit by marking the solvable witnesses and transforming the AssertZero opcodes
    // to the required width by creating intermediate variables.
    // Knowing if a witness is solvable avoids creating un-solvable intermediate variables.
    let mut transformer = CSatTransformer::new(4);
    for value in acir.circuit_arguments() {
        transformer.mark_solvable(value);
    }

    let mut new_acir_opcode_positions: Vec<usize> = Vec::with_capacity(acir_opcode_positions.len());
    // Optimize the assert-zero gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_opcodes = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    // maps a normalized expression to the intermediate variable which represents the expression, along with its 'norm'
    // the 'norm' is simply the value of the first non-zero coefficient in the expression, taken from the linear terms, or quadratic terms if there is none.
    let mut intermediate_variables: IndexMap<Expression<F>, (F, Witness)> = IndexMap::new();
    for (index, opcode) in acir.opcodes.into_iter().enumerate() {
        match opcode {
            Opcode::AssertZero(arith_expr) => {
                let len = intermediate_variables.len();

                let arith_expr = transformer.transform(
                    arith_expr,
                    &mut intermediate_variables,
                    &mut next_witness_index,
                );

                let mut new_opcodes = Vec::new();
                for (g, (norm, w)) in intermediate_variables.iter().skip(len) {
                    // de-normalize
                    let mut intermediate_opcode = g * *norm;
                    // constrain the intermediate opcode to the intermediate variable
                    intermediate_opcode.linear_combinations.push((-F::one(), *w));
                    intermediate_opcode.sort();
                    new_opcodes.push(intermediate_opcode);
                }
                new_opcodes.push(arith_expr);
                for opcode in new_opcodes {
                    new_acir_opcode_positions.push(acir_opcode_positions[index]);
                    transformed_opcodes.push(Opcode::AssertZero(opcode));
                }
            }
            Opcode::BlackBoxFuncCall(ref func) => {
                for witness in func.get_outputs_vec() {
                    transformer.mark_solvable(witness);
                }

                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::MemoryInit { .. } => {
                // `MemoryInit` does not write values to the `WitnessMap`
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::MemoryOp { ref op, .. } => {
                for (_, witness1, witness2) in &op.value.mul_terms {
                    transformer.mark_solvable(*witness1);
                    transformer.mark_solvable(*witness2);
                }
                for (_, witness) in &op.value.linear_combinations {
                    transformer.mark_solvable(*witness);
                }
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::BrilligCall { ref outputs, .. } => {
                for output in outputs {
                    match output {
                        BrilligOutputs::Simple(witness) => transformer.mark_solvable(*witness),
                        BrilligOutputs::Array(witnesses) => {
                            for witness in witnesses {
                                transformer.mark_solvable(*witness);
                            }
                        }
                    }
                }

                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
            Opcode::Call { ref outputs, .. } => {
                for witness in outputs {
                    transformer.mark_solvable(*witness);
                }

                // `Call` does not write values to the `WitnessMap`
                // A separate ACIR function should have its own respective `WitnessMap`
                new_acir_opcode_positions.push(acir_opcode_positions[index]);
                transformed_opcodes.push(opcode);
            }
        }
    }

    let current_witness_index = next_witness_index - 1;

    acir = Circuit {
        current_witness_index,
        opcodes: transformed_opcodes,
        // The transformer does not add new public inputs
        ..acir
    };

    // 2. Eliminate intermediate variables, when they are used in exactly two arithmetic opcodes.
    let mut merge_optimizer = MergeExpressionsOptimizer::new();

    let (opcodes, new_acir_opcode_positions) =
        merge_optimizer.eliminate_intermediate_variable(&acir, new_acir_opcode_positions);

    // n.b. if we do not update current_witness_index after the eliminate_intermediate_variable pass, the real index could be less.
    acir = Circuit {
        opcodes,
        // The optimizer does not add new public inputs
        ..acir
    };

    // 3. Remove redundant range constraints.
    // The `MergeOptimizer` can merge two witnesses which have range opcodes applied to them
    // so we run the `RangeOptimizer` afterwards to clear these up.
    let range_optimizer = RangeOptimizer::new(acir, brillig_side_effects);
    let (acir, new_acir_opcode_positions) =
        range_optimizer.replace_redundant_ranges(new_acir_opcode_positions);

    (acir, new_acir_opcode_positions)
}

/// Find the witness with the highest ID in the circuit.
fn max_witness<F: AcirField>(circuit: &Circuit<F>) -> Witness {
    let mut witnesses = WitnessFolder::new(Witness::default(), |state, witness| {
        *state = witness.max(*state);
    });
    witnesses.fold_circuit(circuit);
    witnesses.into_state()
}

/// Fold all witnesses in a circuit.
struct WitnessFolder<S, A> {
    state: S,
    accumulate: A,
}

impl<S, A> WitnessFolder<S, A>
where
    A: Fn(&mut S, Witness),
{
    /// Create the folder with some initial state and an accumulator function.
    fn new(init: S, accumulate: A) -> Self {
        Self { state: init, accumulate }
    }

    /// Take the accumulated state.
    fn into_state(self) -> S {
        self.state
    }

    /// Add all witnesses from the circuit.
    fn fold_circuit<F: AcirField>(&mut self, circuit: &Circuit<F>) {
        self.fold_many(circuit.private_parameters.iter());
        self.fold_many(circuit.public_parameters.0.iter());
        self.fold_many(circuit.return_values.0.iter());
        for opcode in &circuit.opcodes {
            self.fold_opcode(opcode);
        }
    }

    /// Fold a witness into the state.
    fn fold(&mut self, witness: Witness) {
        (self.accumulate)(&mut self.state, witness);
    }

    /// Fold many witnesses into the state.
    fn fold_many<'w, I: Iterator<Item = &'w Witness>>(&mut self, witnesses: I) {
        for witness in witnesses {
            self.fold(*witness);
        }
    }

    /// Add witnesses from the opcode.
    fn fold_opcode<F: AcirField>(&mut self, opcode: &Opcode<F>) {
        match opcode {
            Opcode::AssertZero(expr) => {
                self.fold_expr(expr);
            }
            Opcode::BlackBoxFuncCall(call) => self.fold_blackbox(call),
            Opcode::MemoryOp { block_id: _, op } => {
                let MemOp { operation, index, value } = op;
                self.fold_expr(operation);
                self.fold_expr(index);
                self.fold_expr(value);
            }
            Opcode::MemoryInit { block_id: _, init, block_type: _ } => {
                for witness in init {
                    self.fold(*witness);
                }
            }
            // We keep the display for a BrilligCall and circuit Call separate as they
            // are distinct in their functionality and we should maintain this separation for debugging.
            Opcode::BrilligCall { id: _, inputs, outputs, predicate } => {
                self.fold_expr(predicate);
                self.fold_brillig_inputs(inputs);
                self.fold_brillig_outputs(outputs);
            }
            Opcode::Call { id: _, inputs, outputs, predicate } => {
                self.fold_expr(predicate);
                self.fold_many(inputs.iter());
                self.fold_many(outputs.iter());
            }
        }
    }

    fn fold_expr<F: AcirField>(&mut self, expr: &Expression<F>) {
        for i in &expr.mul_terms {
            self.fold(i.1);
            self.fold(i.2);
        }
        for i in &expr.linear_combinations {
            self.fold(i.1);
        }
    }

    fn fold_brillig_inputs<F: AcirField>(&mut self, inputs: &[BrilligInputs<F>]) {
        for input in inputs {
            match input {
                BrilligInputs::Single(expr) => {
                    self.fold_expr(expr);
                }
                BrilligInputs::Array(exprs) => {
                    for expr in exprs {
                        self.fold_expr(expr);
                    }
                }
                BrilligInputs::MemoryArray(_) => {}
            }
        }
    }

    fn fold_brillig_outputs(&mut self, outputs: &[BrilligOutputs]) {
        for output in outputs {
            match output {
                BrilligOutputs::Simple(witness) => {
                    self.fold(*witness);
                }
                BrilligOutputs::Array(witnesses) => self.fold_many(witnesses.iter()),
            }
        }
    }

    fn fold_blackbox<F: AcirField>(&mut self, call: &BlackBoxFuncCall<F>) {
        match call {
            BlackBoxFuncCall::AES128Encrypt { inputs, iv, key, outputs } => {
                self.fold_inputs(inputs.as_slice());
                self.fold_inputs(iv.as_slice());
                self.fold_inputs(key.as_slice());
                self.fold_many(outputs.iter());
            }
            BlackBoxFuncCall::AND { lhs, rhs, output, .. } => {
                self.fold_input(lhs);
                self.fold_input(rhs);
                self.fold(*output);
            }
            BlackBoxFuncCall::XOR { lhs, rhs, output, .. } => {
                self.fold_input(lhs);
                self.fold_input(rhs);
                self.fold(*output);
            }
            BlackBoxFuncCall::RANGE { input, .. } => {
                self.fold_input(input);
            }
            BlackBoxFuncCall::Blake2s { inputs, outputs } => {
                self.fold_inputs(inputs.as_slice());
                self.fold_many(outputs.iter());
            }
            BlackBoxFuncCall::Blake3 { inputs, outputs } => {
                self.fold_inputs(inputs.as_slice());
                self.fold_many(outputs.iter());
            }
            BlackBoxFuncCall::EcdsaSecp256k1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                output,
                predicate,
            } => {
                self.fold_inputs(public_key_x.as_slice());
                self.fold_inputs(public_key_y.as_slice());
                self.fold_inputs(signature.as_slice());
                self.fold_inputs(hashed_message.as_slice());
                self.fold(*output);
                self.fold_input(predicate);
            }
            BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                output,
                predicate,
            } => {
                self.fold_inputs(public_key_x.as_slice());
                self.fold_inputs(public_key_y.as_slice());
                self.fold_inputs(signature.as_slice());
                self.fold_inputs(hashed_message.as_slice());
                self.fold(*output);
                self.fold_input(predicate);
            }
            BlackBoxFuncCall::MultiScalarMul { points, scalars, predicate, outputs } => {
                self.fold_inputs(points.as_slice());
                self.fold_inputs(scalars.as_slice());
                self.fold_input(predicate);
                let (x, y, i) = outputs;
                self.fold(*x);
                self.fold(*y);
                self.fold(*i);
            }
            BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, predicate, outputs } => {
                self.fold_inputs(input1.as_slice());
                self.fold_inputs(input2.as_slice());
                self.fold_input(predicate);
                let (x, y, i) = outputs;
                self.fold(*x);
                self.fold(*y);
                self.fold(*i);
            }
            BlackBoxFuncCall::Keccakf1600 { inputs, outputs } => {
                self.fold_inputs(inputs.as_slice());
                self.fold_many(outputs.iter());
            }
            BlackBoxFuncCall::RecursiveAggregation {
                verification_key,
                proof,
                public_inputs,
                key_hash,
                proof_type: _,
                predicate,
            } => {
                self.fold_inputs(verification_key.as_slice());
                self.fold_inputs(proof.as_slice());
                self.fold_inputs(public_inputs.as_slice());
                self.fold_input(key_hash);
                self.fold_input(predicate);
            }
            BlackBoxFuncCall::Poseidon2Permutation { inputs, outputs } => {
                self.fold_inputs(inputs.as_slice());
                self.fold_many(outputs.iter());
            }
            BlackBoxFuncCall::Sha256Compression { inputs, hash_values, outputs } => {
                self.fold_inputs(inputs.as_slice());
                self.fold_inputs(hash_values.as_slice());
                self.fold_many(outputs.iter());
            }
        }
    }

    fn fold_inputs<F: AcirField>(&mut self, inputs: &[FunctionInput<F>]) {
        for input in inputs {
            self.fold_input(input);
        }
    }

    fn fold_input<F: AcirField>(&mut self, input: &FunctionInput<F>) {
        if let FunctionInput::Witness(witness) = input {
            self.fold(*witness);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::transform_internal;
    use crate::compiler::CircuitSimulator;
    use acir::circuit::{Circuit, brillig::BrilligFunctionId};
    use std::collections::BTreeMap;

    #[test]
    fn test_max_transformer_passes() {
        let formatted_acir = r#"private parameters: [w0]
        public parameters: []
        return values: [w1, w2, w3, w4, w5, w6, w7, w8, w9, w10, w11, w12, w13, w14, w15, w16, w17, w18, w19, w20, w21, w22, w23, w24, w25, w26, w27, w28, w29, w30, w31]
        BRILLIG CALL func: 0, predicate: 1, inputs: [w0, 31, 256], outputs: [w32, w33, w34, w35, w36, w37, w38, w39, w40, w41, w42, w43, w44, w45, w46, w47, w48, w49, w50, w51, w52, w53, w54, w55, w56, w57, w58, w59, w60, w61, w62]
        BLACKBOX::RANGE input: w35, bits: 8
        BLACKBOX::RANGE input: w36, bits: 8
        BLACKBOX::RANGE input: w37, bits: 8
        BLACKBOX::RANGE input: w38, bits: 8
        BLACKBOX::RANGE input: w39, bits: 8
        BLACKBOX::RANGE input: w40, bits: 8
        BLACKBOX::RANGE input: w41, bits: 8
        BLACKBOX::RANGE input: w42, bits: 8
        BLACKBOX::RANGE input: w43, bits: 8
        BLACKBOX::RANGE input: w44, bits: 8
        BLACKBOX::RANGE input: w45, bits: 8
        BLACKBOX::RANGE input: w46, bits: 8
        BLACKBOX::RANGE input: w47, bits: 8
        BLACKBOX::RANGE input: w48, bits: 8
        BLACKBOX::RANGE input: w49, bits: 8
        BLACKBOX::RANGE input: w50, bits: 8
        BLACKBOX::RANGE input: w51, bits: 8
        BLACKBOX::RANGE input: w52, bits: 8
        BLACKBOX::RANGE input: w53, bits: 8
        BLACKBOX::RANGE input: w54, bits: 8
        BLACKBOX::RANGE input: w55, bits: 8
        BLACKBOX::RANGE input: w56, bits: 8
        BLACKBOX::RANGE input: w57, bits: 8
        BLACKBOX::RANGE input: w58, bits: 8
        BLACKBOX::RANGE input: w59, bits: 8
        BLACKBOX::RANGE input: w60, bits: 8
        BLACKBOX::RANGE input: w61, bits: 8
        BLACKBOX::RANGE input: w62, bits: 8
        ASSERT w32 = w0 - 256*w33 - 65536*w34 - 16777216*w35 - 4294967296*w36 - 1099511627776*w37 - 281474976710656*w38 - 72057594037927936*w39 - 18446744073709551616*w40 - 4722366482869645213696*w41 - 1208925819614629174706176*w42 - 309485009821345068724781056*w43 - 79228162514264337593543950336*w44 - 20282409603651670423947251286016*w45 - 5192296858534827628530496329220096*w46 - 1329227995784915872903807060280344576*w47 - 340282366920938463463374607431768211456*w48 - 87112285931760246646623899502532662132736*w49 - 22300745198530623141535718272648361505980416*w50 - 5708990770823839524233143877797980545530986496*w51 - 1461501637330902918203684832716283019655932542976*w52 - 374144419156711147060143317175368453031918731001856*w53 - 95780971304118053647396689196894323976171195136475136*w54 - 24519928653854221733733552434404946937899825954937634816*w55 - 6277101735386680763835789423207666416102355444464034512896*w56 - 1606938044258990275541962092341162602522202993782792835301376*w57 - 411376139330301510538742295639337626245683966408394965837152256*w58 - 105312291668557186697918027683670432318895095400549111254310977536*w59 - 26959946667150639794667015087019630673637144422540572481103610249216*w60 - 6901746346790563787434755862277025452451108972170386555162524223799296*w61 - 1766847064778384329583297500742918515827483896875618958121606201292619776*w62
        ASSERT w32 = 60
        ASSERT w33 = 33
        ASSERT w34 = 31
        ASSERT w0 = 16777216*w35 + 4294967296*w36 + 1099511627776*w37 + 281474976710656*w38 + 72057594037927936*w39 + 18446744073709551616*w40 + 4722366482869645213696*w41 + 1208925819614629174706176*w42 + 309485009821345068724781056*w43 + 79228162514264337593543950336*w44 + 20282409603651670423947251286016*w45 + 5192296858534827628530496329220096*w46 + 1329227995784915872903807060280344576*w47 + 340282366920938463463374607431768211456*w48 + 87112285931760246646623899502532662132736*w49 + 22300745198530623141535718272648361505980416*w50 + 5708990770823839524233143877797980545530986496*w51 + 1461501637330902918203684832716283019655932542976*w52 + 374144419156711147060143317175368453031918731001856*w53 + 95780971304118053647396689196894323976171195136475136*w54 + 24519928653854221733733552434404946937899825954937634816*w55 + 6277101735386680763835789423207666416102355444464034512896*w56 + 1606938044258990275541962092341162602522202993782792835301376*w57 + 411376139330301510538742295639337626245683966408394965837152256*w58 + 105312291668557186697918027683670432318895095400549111254310977536*w59 + 26959946667150639794667015087019630673637144422540572481103610249216*w60 + 6901746346790563787434755862277025452451108972170386555162524223799296*w61 + 1766847064778384329583297500742918515827483896875618958121606201292619776*w62 + 2040124
        ASSERT w62 = w1
        ASSERT w61 = w2
        ASSERT w60 = w3
        ASSERT w59 = w4
        ASSERT w58 = w5
        ASSERT w57 = w6
        ASSERT w56 = w7
        ASSERT w55 = w8
        ASSERT w54 = w9
        ASSERT w53 = w10
        ASSERT w52 = w11
        ASSERT w51 = w12
        ASSERT w50 = w13
        ASSERT w49 = w14
        ASSERT w48 = w15
        ASSERT w47 = w16
        ASSERT w46 = w17
        ASSERT w45 = w18
        ASSERT w44 = w19
        ASSERT w43 = w20
        ASSERT w42 = w21
        ASSERT w41 = w22
        ASSERT w40 = w23
        ASSERT w39 = w24
        ASSERT w38 = w25
        ASSERT w37 = w26
        ASSERT w36 = w27
        ASSERT w35 = w28
        ASSERT w29 = 31
        ASSERT w30 = 33
        ASSERT w31 = 60
        "#;

        let acir = Circuit::from_str(formatted_acir).unwrap();
        assert!(CircuitSimulator::check_circuit(&acir).is_none());

        let acir_opcode_positions = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 29, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43,
            44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
        ];
        let mut brillig_side_effects = BTreeMap::new();
        brillig_side_effects.insert(BrilligFunctionId(0), false);

        let (_, _, opcodes_hash_stabilized) =
            transform_internal(acir, acir_opcode_positions.clone(), &brillig_side_effects, None);
        assert!(!opcodes_hash_stabilized);
    }
}
