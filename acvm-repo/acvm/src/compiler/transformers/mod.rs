use acir::{
    AcirField,
    circuit::{
        Circuit, ExpressionWidth, Opcode,
        brillig::{BrilligInputs, BrilligOutputs},
        opcodes::{BlackBoxFuncCall, FunctionInput, MemOp},
    },
    native_types::{Expression, Witness},
};
use indexmap::IndexMap;

mod csat;

pub(crate) use csat::CSatTransformer;
pub use csat::MIN_EXPRESSION_WIDTH;
use tracing::info;

use super::{
    AcirTransformationMap,
    optimizers::{MergeExpressionsOptimizer, RangeOptimizer},
    transform_assert_messages,
};

/// We need multiple passes to stabilize the output.
/// The value was determined by running tests.
const MAX_TRANSFORMER_PASSES: usize = 3;

/// Applies backend specific optimizations to a [`Circuit`].
pub fn transform<F: AcirField>(
    acir: Circuit<F>,
    expression_width: ExpressionWidth,
) -> (Circuit<F>, AcirTransformationMap) {
    // Track original acir opcode positions throughout the transformation passes of the compilation
    // by applying the modifications done to the circuit opcodes and also to the opcode_positions (delete and insert)
    let acir_opcode_positions = acir.opcodes.iter().enumerate().map(|(i, _)| i).collect();

    let (mut acir, acir_opcode_positions) =
        transform_internal(acir, expression_width, acir_opcode_positions);

    let transformation_map = AcirTransformationMap::new(&acir_opcode_positions);

    acir.assert_messages = transform_assert_messages(acir.assert_messages, &transformation_map);

    (acir, transformation_map)
}

/// Applies backend specific optimizations to a [`Circuit`].
///
/// Accepts an injected `acir_opcode_positions` to allow transformations to be applied directly after optimizations.
///
/// Does multiple passes until the output stabilizes.
#[tracing::instrument(level = "trace", name = "transform_acir", skip(acir, acir_opcode_positions))]
pub(super) fn transform_internal<F: AcirField>(
    mut acir: Circuit<F>,
    expression_width: ExpressionWidth,
    mut acir_opcode_positions: Vec<usize>,
) -> (Circuit<F>, Vec<usize>) {
    if acir.opcodes.len() == 1 && matches!(acir.opcodes[0], Opcode::BrilligCall { .. }) {
        info!("Program is fully unconstrained, skipping transformation pass");
        return (acir, acir_opcode_positions);
    }

    // Allow multiple passes until we have stable output.
    let mut prev_opcodes_hash = fxhash::hash64(&acir.opcodes);

    // For most test programs it would be enough to loop here, but some of them
    // don't stabilize unless we also repeat the backend agnostic optimizations.
    for _ in 0..MAX_TRANSFORMER_PASSES {
        info!("Number of opcodes {}", acir.opcodes.len());
        let (new_acir, new_acir_opcode_positions) =
            transform_internal_once(acir, expression_width, acir_opcode_positions);

        acir = new_acir;
        acir_opcode_positions = new_acir_opcode_positions;

        let new_opcodes_hash = fxhash::hash64(&acir.opcodes);

        if new_opcodes_hash == prev_opcodes_hash {
            break;
        }
        prev_opcodes_hash = new_opcodes_hash;
    }
    // After the elimination of intermediate variables the `current_witness_index` is potentially higher than it needs to be,
    // which would cause gaps if we ran the optimization a second time, making it look like new variables were added.
    acir.current_witness_index = max_witness(&acir).witness_index();

    (acir, acir_opcode_positions)
}

/// Applies backend specific optimizations to a [`Circuit`].
///
/// Accepts an injected `acir_opcode_positions` to allow transformations to be applied directly after optimizations.
///
/// If the width is unbounded, it does nothing.
/// If it is bounded, it first performs the 'CSAT transformation' in one pass, by creating intermediate variables when necessary.
/// This results in arithmetic opcodes having the required width.
/// Then the 'eliminate intermediate variables' pass will remove any intermediate variables (for instance created by the previous transformation)
/// that are used in exactly two arithmetic opcodes.
/// This results in arithmetic opcodes having linear combinations of potentially large width.
/// Finally, the 'range optimization' pass will remove any redundant range opcodes.
/// The backend is expected to handle linear combinations of 'unbounded width' in a more efficient way
/// than the 'CSAT transformation'.
#[tracing::instrument(
    level = "trace",
    name = "transform_acir_once",
    skip(acir, acir_opcode_positions)
)]
fn transform_internal_once<F: AcirField>(
    mut acir: Circuit<F>,
    expression_width: ExpressionWidth,
    acir_opcode_positions: Vec<usize>,
) -> (Circuit<F>, Vec<usize>) {
    // If the expression width is unbounded, we don't need to do anything.
    let mut transformer = match &expression_width {
        ExpressionWidth::Unbounded => {
            return (acir, acir_opcode_positions);
        }
        ExpressionWidth::Bounded { width } => {
            let mut csat = CSatTransformer::new(*width);
            for value in acir.circuit_arguments() {
                csat.mark_solvable(value);
            }
            csat
        }
    };

    // 1. CSAT transformation
    // Process each opcode in the circuit by marking the solvable witnesses and transforming the AssertZero opcodes
    // to the required width by creating intermediate variables.
    // Knowing if a witness is solvable avoids creating un-solvable intermediate variables.

    let mut new_acir_opcode_positions: Vec<usize> = Vec::with_capacity(acir_opcode_positions.len());
    // Optimize the assert-zero gates by reducing them into the correct width and
    // creating intermediate variables when necessary
    let mut transformed_opcodes = Vec::new();

    let mut next_witness_index = acir.current_witness_index + 1;
    // maps a normalized expression to the intermediate variable which represents the expression, along with its 'norm'
    // the 'norm' is simply the value of the first non zero coefficient in the expression, taken from the linear terms, or quadratic terms if there is none.
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
                        BrilligOutputs::Simple(w) => transformer.mark_solvable(*w),
                        BrilligOutputs::Array(v) => {
                            for witness in v {
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
    let range_optimizer = RangeOptimizer::new(acir);
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
        for w in witnesses {
            self.fold(*w);
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
                for w in init {
                    self.fold(*w);
                }
            }
            // We keep the display for a BrilligCall and circuit Call separate as they
            // are distinct in their functionality and we should maintain this separation for debugging.
            Opcode::BrilligCall { id: _, inputs, outputs, predicate } => {
                if let Some(pred) = predicate {
                    self.fold_expr(pred);
                }
                self.fold_brillig_inputs(inputs);
                self.fold_brillig_outputs(outputs);
            }
            Opcode::Call { id: _, inputs, outputs, predicate } => {
                if let Some(pred) = predicate {
                    self.fold_expr(pred);
                }
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
                BrilligOutputs::Simple(w) => {
                    self.fold(*w);
                }
                BrilligOutputs::Array(ws) => self.fold_many(ws.iter()),
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
            } => {
                self.fold_inputs(public_key_x.as_slice());
                self.fold_inputs(public_key_y.as_slice());
                self.fold_inputs(signature.as_slice());
                self.fold_inputs(hashed_message.as_slice());
                self.fold(*output);
            }
            BlackBoxFuncCall::EcdsaSecp256r1 {
                public_key_x,
                public_key_y,
                signature,
                hashed_message,
                output,
            } => {
                self.fold_inputs(public_key_x.as_slice());
                self.fold_inputs(public_key_y.as_slice());
                self.fold_inputs(signature.as_slice());
                self.fold_inputs(hashed_message.as_slice());
                self.fold(*output);
            }
            BlackBoxFuncCall::MultiScalarMul { points, scalars, outputs } => {
                self.fold_inputs(points.as_slice());
                self.fold_inputs(scalars.as_slice());
                let (x, y, i) = outputs;
                self.fold(*x);
                self.fold(*y);
                self.fold(*i);
            }
            BlackBoxFuncCall::EmbeddedCurveAdd { input1, input2, outputs } => {
                self.fold_inputs(input1.as_slice());
                self.fold_inputs(input2.as_slice());
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
            } => {
                self.fold_inputs(verification_key.as_slice());
                self.fold_inputs(proof.as_slice());
                self.fold_inputs(public_inputs.as_slice());
                self.fold_input(key_hash);
            }
            BlackBoxFuncCall::BigIntAdd { .. }
            | BlackBoxFuncCall::BigIntSub { .. }
            | BlackBoxFuncCall::BigIntMul { .. }
            | BlackBoxFuncCall::BigIntDiv { .. } => {}
            BlackBoxFuncCall::BigIntFromLeBytes { inputs, modulus: _, output: _ } => {
                self.fold_inputs(inputs.as_slice());
            }
            BlackBoxFuncCall::BigIntToLeBytes { input: _, outputs } => {
                self.fold_many(outputs.iter());
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
