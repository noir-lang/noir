use acir::AcirField;
use acvm_blackbox_solver::BlackBoxFunctionSolver;
use num_bigint::BigUint;

use crate::{
    BinaryFieldOp, BinaryIntOp, MemoryValue, NextOpcodePositionOrState, OpcodePosition, VM,
};
use std::collections::HashMap;

/// A state that represents a true comparison as part of a feature
const FUZZING_COMPARISON_TRUE_STATE: usize = usize::MAX - 1;
/// A state that represents a false comparison as part of a feature
const FUZZING_COMPARISON_FALSE_STATE: usize = usize::MAX;

/// The start of the range of the states that represent logarithm of the difference between the comparison arguments as part of a feature
const FUZZING_COMPARISON_LOG_RANGE_START_STATE: usize = 0;

/// A tuple of the current opcode position and the next opcode position or state
pub type Branch = (OpcodePosition, NextOpcodePositionOrState);

/// The index of a unique feature in the fuzzing trace
pub type UniqueFeatureIndex = usize;

/// A map for translating encountered branching logic to features for fuzzing
pub type BranchToFeatureMap = HashMap<Branch, UniqueFeatureIndex>;

/// Context structure for all information necessary to compute the fuzzing trace
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub(super) struct FuzzingTrace {
    /// Fuzzer tracing memory ddd
    trace: Vec<u32>,
    /// Branch to feature map for fuzzing
    /// Maps program counter + feature to index in the trace vector
    branch_to_feature_map: HashMap<(usize, usize), usize>,
}

impl FuzzingTrace {
    pub(super) fn new(branch_to_feature_map: HashMap<(usize, usize), usize>) -> Self {
        let len = branch_to_feature_map.len();
        Self { trace: vec![0; len], branch_to_feature_map }
    }

    fn record_branch(&mut self, pc: usize, destination: usize) {
        let index = self.branch_to_feature_map[&(pc, destination)];
        self.trace[index] += 1;
    }

    fn record_conditional_mov(&mut self, pc: usize, branch: bool) {
        let index = self.branch_to_feature_map[&(
            pc,
            if branch { FUZZING_COMPARISON_TRUE_STATE } else { FUZZING_COMPARISON_FALSE_STATE },
        )];
        self.trace[index] += 1;
    }

    fn record_binary_field_op_comparison<F: AcirField>(
        &mut self,
        pc: usize,
        op: &BinaryFieldOp,
        lhs: MemoryValue<F>,
        rhs: MemoryValue<F>,
        result: MemoryValue<F>,
    ) {
        match op {
            BinaryFieldOp::Add
            | BinaryFieldOp::Sub
            | BinaryFieldOp::Mul
            | BinaryFieldOp::Div
            | BinaryFieldOp::IntegerDiv => {}
            BinaryFieldOp::Equals | BinaryFieldOp::LessThan | BinaryFieldOp::LessThanEquals => {
                let a = match lhs {
                    MemoryValue::Field(a) => a,
                    _ => return,
                };
                let b = match rhs {
                    MemoryValue::Field(b) => b,
                    _ => return,
                };
                let c = match result {
                    MemoryValue::U1(value) => value,
                    _ => return,
                };

                let approach_index = self.branch_to_feature_map[&(
                    pc,
                    FUZZING_COMPARISON_LOG_RANGE_START_STATE
                        + BigUint::from_bytes_be(&(b - a).to_be_bytes()).bits() as usize,
                )];
                let condition_index = self.branch_to_feature_map[&(
                    pc,
                    if c { FUZZING_COMPARISON_TRUE_STATE } else { FUZZING_COMPARISON_FALSE_STATE },
                )];
                self.trace[approach_index] += 1;
                self.trace[condition_index] += 1;
            }
        }
    }

    fn record_binary_int_op_comparison<F: AcirField>(
        &mut self,
        pc: usize,
        op: &BinaryIntOp,
        lhs: MemoryValue<F>,
        rhs: MemoryValue<F>,
        result: MemoryValue<F>,
    ) {
        match op {
            BinaryIntOp::Add
            | BinaryIntOp::Sub
            | BinaryIntOp::Mul
            | BinaryIntOp::Div
            | BinaryIntOp::And
            | BinaryIntOp::Or
            | BinaryIntOp::Xor
            | BinaryIntOp::Shl
            | BinaryIntOp::Shr => {}
            BinaryIntOp::Equals | BinaryIntOp::LessThan | BinaryIntOp::LessThanEquals => {
                let lhs_val = lhs.to_u128().expect("lhs is not an integer");
                let rhs_val = rhs.to_u128().expect("rhs is not an integer");
                let c = match result {
                    MemoryValue::U1(value) => value,
                    _ => return,
                };

                let approach_index = self.branch_to_feature_map[&(
                    pc,
                    FUZZING_COMPARISON_LOG_RANGE_START_STATE
                        + rhs_val.abs_diff(lhs_val).checked_ilog2().map_or_else(|| 0, |x| x + 1)
                            as usize,
                )];
                let condition_index = self.branch_to_feature_map[&(
                    pc,
                    if c { FUZZING_COMPARISON_TRUE_STATE } else { FUZZING_COMPARISON_FALSE_STATE },
                )];
                self.trace[approach_index] += 1;
                self.trace[condition_index] += 1;
            }
        }
    }

    pub(super) fn get_trace(&self) -> Vec<u32> {
        self.trace.clone()
    }
}

impl<F: AcirField, B: BlackBoxFunctionSolver<F>> VM<'_, F, B> {
    /// Collect information about the comparison of two field values in the fuzzing trace
    pub(super) fn fuzzing_trace_binary_field_op_comparison(
        &mut self,
        op: &BinaryFieldOp,
        lhs: MemoryValue<F>,
        rhs: MemoryValue<F>,
        result: MemoryValue<F>,
    ) {
        if let Some(trace) = &mut self.fuzzing_trace {
            trace.record_binary_field_op_comparison(self.program_counter, op, lhs, rhs, result);
        }
    }

    /// Collect information about the comparison of two integer values in the fuzzing trace
    pub(super) fn fuzzing_trace_binary_int_op_comparison(
        &mut self,
        op: &BinaryIntOp,
        lhs: MemoryValue<F>,
        rhs: MemoryValue<F>,
        result: MemoryValue<F>,
    ) {
        if let Some(trace) = &mut self.fuzzing_trace {
            trace.record_binary_int_op_comparison(self.program_counter, op, lhs, rhs, result);
        }
    }

    /// Mark the execution of a particular branch in the fuzzing trace
    pub(super) fn fuzzing_trace_branching(&mut self, destination: NextOpcodePositionOrState) {
        if let Some(trace) = &mut self.fuzzing_trace {
            trace.record_branch(self.program_counter, destination);
        }
    }

    /// Mark the execution of a conditional move in the fuzzing trace
    pub(super) fn fuzzing_trace_conditional_mov(&mut self, branch: bool) {
        if let Some(trace) = &mut self.fuzzing_trace {
            trace.record_conditional_mov(self.program_counter, branch);
        }
    }
}
