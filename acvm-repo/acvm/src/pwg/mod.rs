// Re-usable methods that backends can use to implement their PWG

//! This module contains methods to implement the partial witness generation (PWG) of an ACIR program.
//! The goal of ACIR execution is to compute the values of all the ACIR witnesses, or an error if it could not compute them all.
//! A proving system will then be able to use the ACIR circuit and the values of the ACIR witnesses to generate a proof of this execution.
//! The ACIR opcodes are not modified by the execution.
//! Witness generation means getting valid values for the witnesses used by the ACIR opcodes of the program.
//! They are called *partial* witness because a proving system may create additional witnesses on its own for
//! generating the proof (and a corresponding low-level circuit). The PWG generates values for all the witnesses
//! of the ACIR program, or returns an error if it cannot do it.
//!
//! Implementation details & examples:
//! It starts by instantiating an ACVM (ACIR Virtual Machine), which executes the given ACIR opcodes in the `solve()` function.
//!
//! Parameters: When instantiating the ACVM, it needs to be provided with:
//!  - a `backend` implementing the `BlackBoxFunctionSolver` trait. Different implementation can be used depending on the EC used by the underlying proving system.
//!  - `opcodes`: the ACIR opcodes of the program to solve.
//!  - `initial_witness`: a mapping of initial witness values representing the inputs of the program. The ACVM will update this map as it solves the opcodes.
//!  - `unconstrained_functions`: the Brillig bytecode of the unconstrained functions used by the program.
//!  - `assertion_payloads`: additional information used to provide feedback to the user when an assertion fails.
//!
//! Returns: [`ACVMStatus`]
//!
//! Each opcode is solved independently. In general we require its inputs to be already known, i.e previously solved,
//! and the output is simply computed from the inputs, and then the output becomes 'known' for the subsequent opcodes.
//!
//! See [`acir::circuit::Opcode`] for more details.
//!
//! Example:
// Compiled ACIR for main (non-transformed):
// func 0
// private parameters: [w0, w1, w2, w3, w4]
// public parameters: []
// return values: [w9]
// BLACKBOX::RANGE input: w0, bits: 32
// BLACKBOX::RANGE input: w1, bits: 32
// BLACKBOX::RANGE input: w2, bits: 32
// BLACKBOX::RANGE input: w3, bits: 32
// BLACKBOX::RANGE input: w4, bits: 32
// ASSERT w0 - w1 - w6 = 0
// BRILLIG CALL func: 0, inputs: [w6], outputs: [w7]
// ASSERT w6*w7 + w8 - 1 = 0
// ASSERT w6*w8 = 0
// ASSERT w1*w8 = 0
// ASSERT w0 - w2 - w9 = 0
//!
//! This ACIR program defines the 'main' function and indicates it is 'non-transformed'.
//! Indeed, some ACIR pass can transform the ACIR program in order to apply optimizations,
//! or to make it compatible with a specific proving system.
//! However, ACIR execution is expected to work on any ACIR program (transformed or not).
//! Then we see the parameters of the program as public and private inputs.
//! The `initial_witness` needs to contain values for these parameters before execution, else
//! the execution will fail.
//! The first ACIR opcodes are RANGE opcodes which ensure the inputs have the expected range (as specified in the Noir source code).
//! Solving this black-box simply means to validate that the values (from `initial_witness`) are indeed 32 bits for w0, w1, w2, w3, w4
//! If `initial_witness` does not have values for w0, w1, w2, w3, w4, or if the values are over 32 bits, the execution will fail.
//! The next opcode is an AssertZero opcode: ASSERT w0 - w1 - w6 = 0, which indicates that `w0 - w1 - w6` should be equal to 0.
//! Since we know the values of `w0, w1` from `initial_witness`, we can compute `w6 = w0 + w1` so that the AssertZero is satisfied.
//! Solving AssertZero means computing the unknown witness and adding the result to `initial_witness`, which now contains the value for `w6`.
//! The next opcode is a Brillig Call where input is `w6` and output is `w7`. From the function id of the opcode, the solver will retrieve the
//! corresponding Brillig bytecode and instantiate a Brillig VM with the value of the input. This value was just computed before.
//! Executing the Brillig VM on this input will give us the output which is the value for `w7`, that we add to `initial_witness`.
//! The next opcode is again an AssertZero: `w6 * w7 + w8 - 1 = 0`, which computes the value of `w8`.
//! The two next opcodes are AssertZero without any unknown witnesses: `w6 * w8 = 0` and `w1 * w8 = 0`
//! Solving such opcodes means that we compute `w6 * w8 ` and `w1 * w8` using the known values, and check that they evaluate to 0.
//! If not, we would return an error.
//! Finally, the last AssertZero computes `w9` which is the last witness. All of the witnesses have now been computed; execution is complete.

use std::collections::HashMap;

use acir::{
    AcirField, BlackBoxFunc,
    brillig::ForeignCallResult,
    circuit::{
        AssertionPayload, ErrorSelector, ExpressionOrMemory, Opcode, OpcodeLocation,
        brillig::{BrilligBytecode, BrilligFunctionId, BrilligInputs, BrilligOutputs},
        opcodes::{AcirFunctionId, BlockId, FunctionInput, InvalidInputBitSize},
    },
    native_types::{Expression, Witness, WitnessMap},
};
use acvm_blackbox_solver::BlackBoxResolutionError;
use brillig_vm::fuzzing::BranchToFeatureMap;

use self::{arithmetic::ExpressionSolver, memory_op::MemoryOpSolver};
use crate::BlackBoxFunctionSolver;

use thiserror::Error;

// arithmetic
pub(crate) mod arithmetic;
// Brillig bytecode
pub(crate) mod brillig;
// black box functions
pub(crate) mod blackbox;
mod memory_op;

pub use self::brillig::{BrilligSolver, BrilligSolverStatus};
pub use brillig::ForeignCallWaitInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum ACVMStatus<F> {
    /// All witnesses have been computed and all opcodes have been successfully resolved. Execution is complete.
    Solved,

    /// The ACVM is processing the circuit, i.e solving the opcodes. This status is used to resume execution after it has been paused.
    InProgress,

    /// The ACVM has encountered an irrecoverable error while executing the circuit and can not progress.
    /// Most commonly this will be due to an unsatisfied constraint due to invalid inputs to the circuit.
    Failure(OpcodeResolutionError<F>),

    /// The ACVM has encountered a request for a Brillig [foreign call][brillig_vm::brillig::Opcode::ForeignCall]
    /// to retrieve information from outside of the ACVM. The result of the foreign call must be passed back
    /// to the ACVM using [`ACVM::resolve_pending_foreign_call`].
    ///
    /// Once this is done, the ACVM can be restarted to solve the remaining opcodes.
    RequiresForeignCall(ForeignCallWaitInfo<F>),

    /// The ACVM has encountered a request for an ACIR [call][acir::circuit::Opcode]
    /// to execute a separate ACVM instance. The result of the ACIR call must be passed back to the ACVM.
    ///
    /// Once this is done, the ACVM can be restarted to solve the remaining opcodes.
    RequiresAcirCall(AcirCallWaitInfo<F>),
}

impl<F> std::fmt::Display for ACVMStatus<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ACVMStatus::Solved => write!(f, "Solved"),
            ACVMStatus::InProgress => write!(f, "In progress"),
            ACVMStatus::Failure(_) => write!(f, "Execution failure"),
            ACVMStatus::RequiresForeignCall(_) => write!(f, "Waiting on foreign call"),
            ACVMStatus::RequiresAcirCall(_) => write!(f, "Waiting on acir call"),
        }
    }
}

#[expect(clippy::large_enum_variant)]
pub enum StepResult<'a, F, B: BlackBoxFunctionSolver<F>> {
    Status(ACVMStatus<F>),
    IntoBrillig(BrilligSolver<'a, F, B>),
}

// This enum represents the different cases in which an
// opcode can be unsolvable.
// The most common being that one of its input has not been
// assigned a value.
//
// TODO(https://github.com/noir-lang/noir/issues/10052): ExpressionHasTooManyUnknowns is specific for expression solver
// TODO(https://github.com/noir-lang/noir/issues/10052): we could have a error enum for expression solver failure cases in that module
// TODO(https://github.com/noir-lang/noir/issues/10052): that can be converted into an OpcodeNotSolvable or OpcodeResolutionError enum
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeNotSolvable<F> {
    #[error("missing assignment for witness index {0}")]
    MissingAssignment(u32),
    #[error("Attempted to load uninitialized memory block")]
    MissingMemoryBlock(u32),
    #[error("expression has too many unknowns {0}")]
    ExpressionHasTooManyUnknowns(Expression<F>),
}

/// Used by errors to point to a specific opcode as that error's cause
///
/// Some errors don't have a specific opcode associated with them, or are created without one and added later.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum ErrorLocation {
    #[default]
    Unresolved,
    Resolved(OpcodeLocation),
}

impl std::fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorLocation::Unresolved => write!(f, "unresolved"),
            ErrorLocation::Resolved(location) => {
                write!(f, "{location}")
            }
        }
    }
}

/// A dynamic assertion payload whose data has been resolved.
/// This is instantiated upon hitting an assertion failure.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct RawAssertionPayload<F> {
    /// Selector to the respective ABI type the data in this payload represents
    pub selector: ErrorSelector,
    /// Resolved data that represents some ABI type.
    /// To be decoded in the final step of error resolution.
    pub data: Vec<F>,
}

/// Enumeration of possible resolved assertion payloads.
/// This is instantiated upon hitting an assertion failure,
/// and can either be static strings or dynamic payloads.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ResolvedAssertionPayload<F> {
    String(String),
    Raw(RawAssertionPayload<F>),
}

#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeResolutionError<F> {
    #[error("Cannot solve opcode: {0}")]
    OpcodeNotSolvable(#[from] OpcodeNotSolvable<F>),
    #[error("Cannot satisfy constraint")]
    UnsatisfiedConstrain {
        opcode_location: ErrorLocation,
        payload: Option<ResolvedAssertionPayload<F>>,
    },
    #[error("Index out of bounds, array has size {array_size:?}, but index was {index:?}")]
    IndexOutOfBounds { opcode_location: ErrorLocation, index: F, array_size: u32 },
    #[error("Cannot solve opcode: {invalid_input_bit_size}")]
    InvalidInputBitSize {
        opcode_location: ErrorLocation,
        invalid_input_bit_size: InvalidInputBitSize,
    },
    #[error("Failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
    #[error("Failed to solve brillig function")]
    BrilligFunctionFailed {
        function_id: BrilligFunctionId,
        call_stack: Vec<OpcodeLocation>,
        payload: Option<ResolvedAssertionPayload<F>>,
    },
    #[error("Attempted to call `main` with a `Call` opcode")]
    AcirMainCallAttempted { opcode_location: ErrorLocation },
    #[error(
        "{results_size:?} result values were provided for {outputs_size:?} call output witnesses, most likely due to bad ACIR codegen"
    )]
    AcirCallOutputsMismatch { opcode_location: ErrorLocation, results_size: u32, outputs_size: u32 },
    #[error("(--pedantic): Predicates are expected to be 0 or 1, but found: {pred_value}")]
    PredicateLargerThanOne { opcode_location: ErrorLocation, pred_value: F },
    #[error("(--pedantic): Memory operations are expected to be 0 or 1, but found: {operation}")]
    MemoryOperationLargerThanOne { opcode_location: ErrorLocation, operation: F },
}

impl<F> From<BlackBoxResolutionError> for OpcodeResolutionError<F> {
    fn from(value: BlackBoxResolutionError) -> Self {
        match value {
            BlackBoxResolutionError::Failed(func, reason) => {
                OpcodeResolutionError::BlackBoxFunctionFailed(func, reason)
            }
            BlackBoxResolutionError::AssertFailed(error) => {
                OpcodeResolutionError::UnsatisfiedConstrain {
                    opcode_location: ErrorLocation::Unresolved,
                    payload: Some(ResolvedAssertionPayload::String(error)),
                }
            }
        }
    }
}

impl<F> From<InvalidInputBitSize> for OpcodeResolutionError<F> {
    fn from(invalid_input_bit_size: InvalidInputBitSize) -> Self {
        Self::InvalidInputBitSize {
            opcode_location: ErrorLocation::Unresolved,
            invalid_input_bit_size,
        }
    }
}

pub type ProfilingSamples = Vec<ProfilingSample>;

#[derive(Default)]
pub struct ProfilingSample {
    pub call_stack: Vec<OpcodeLocation>,
    pub brillig_function_id: Option<BrilligFunctionId>,
}

pub struct ACVM<'a, F: AcirField, B: BlackBoxFunctionSolver<F>> {
    status: ACVMStatus<F>,

    backend: &'a B,

    /// Stores the solver for memory operations acting on blocks of memory disambiguated by [block][`BlockId`].
    block_solvers: HashMap<BlockId, MemoryOpSolver<F>>,

    /// A list of opcodes which are to be executed by the ACVM.
    opcodes: &'a [Opcode<F>],
    /// Index of the next opcode to be executed.
    instruction_pointer: usize,

    /// A mapping of witnesses to their solved values
    /// The map is updated as the ACVM executes.
    witness_map: WitnessMap<F>,

    brillig_solver: Option<BrilligSolver<'a, F, B>>,

    /// A counter maintained throughout an ACVM process that determines
    /// whether the caller has resolved the results of an ACIR [call][Opcode::Call].
    acir_call_counter: usize,
    /// Represents the outputs of all ACIR calls during an ACVM process
    /// List is appended onto by the caller upon reaching a [ACVMStatus::RequiresAcirCall]
    acir_call_results: Vec<Vec<F>>,

    // Each unconstrained function referenced in the program
    unconstrained_functions: &'a [BrilligBytecode<F>],

    assertion_payloads: &'a [(OpcodeLocation, AssertionPayload<F>)],

    profiling_active: bool,

    profiling_samples: ProfilingSamples,

    // Whether we need to trace brillig execution for fuzzing
    brillig_fuzzing_active: bool,

    // Brillig branch to feature map
    brillig_branch_to_feature_map: Option<&'a BranchToFeatureMap>,

    brillig_fuzzing_trace: Option<Vec<u32>>,
}

impl<'a, F: AcirField, B: BlackBoxFunctionSolver<F>> ACVM<'a, F, B> {
    pub fn new(
        backend: &'a B,
        opcodes: &'a [Opcode<F>],
        initial_witness: WitnessMap<F>,
        unconstrained_functions: &'a [BrilligBytecode<F>],
        assertion_payloads: &'a [(OpcodeLocation, AssertionPayload<F>)],
    ) -> Self {
        let status = if opcodes.is_empty() { ACVMStatus::Solved } else { ACVMStatus::InProgress };
        ACVM {
            status,
            backend,
            block_solvers: HashMap::default(),
            opcodes,
            instruction_pointer: 0,
            witness_map: initial_witness,
            brillig_solver: None,
            acir_call_counter: 0,
            acir_call_results: Vec::default(),
            unconstrained_functions,
            assertion_payloads,
            profiling_active: false,
            profiling_samples: Vec::new(),
            brillig_fuzzing_active: false,
            brillig_branch_to_feature_map: None,
            brillig_fuzzing_trace: None,
        }
    }

    /// Enable profiling
    pub fn with_profiler(&mut self, profiling_active: bool) {
        self.profiling_active = profiling_active;
    }

    /// Enable brillig fuzzing
    pub fn with_brillig_fuzzing(
        &mut self,
        brillig_branch_to_feature_map: Option<&'a BranchToFeatureMap>,
    ) {
        self.brillig_fuzzing_active = brillig_branch_to_feature_map.is_some();
        self.brillig_branch_to_feature_map = brillig_branch_to_feature_map;
    }

    pub fn get_brillig_fuzzing_trace(&self) -> Option<Vec<u32>> {
        self.brillig_fuzzing_trace.clone()
    }

    /// Returns a reference to the current state of the ACVM's [`WitnessMap`].
    ///
    /// Once execution has completed, the witness map can be extracted using [`ACVM::finalize`]
    pub fn witness_map(&self) -> &WitnessMap<F> {
        &self.witness_map
    }

    pub fn overwrite_witness(&mut self, witness: Witness, value: F) -> Option<F> {
        self.witness_map.insert(witness, value)
    }

    /// Returns a slice containing the opcodes of the circuit being executed.
    pub fn opcodes(&self) -> &[Opcode<F>] {
        self.opcodes
    }

    /// Returns the index of the current opcode to be executed.
    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    pub fn take_profiling_samples(&mut self) -> ProfilingSamples {
        std::mem::take(&mut self.profiling_samples)
    }

    /// Finalize the ACVM execution, returning the resulting [`WitnessMap`].
    pub fn finalize(self) -> WitnessMap<F> {
        if self.status != ACVMStatus::Solved {
            panic!("ACVM execution is not complete: ({})", self.status);
        }
        self.witness_map
    }

    /// Updates the current status of the VM.
    /// Returns the given status.
    fn status(&mut self, status: ACVMStatus<F>) -> ACVMStatus<F> {
        self.status = status.clone();
        status
    }

    pub fn get_status(&self) -> &ACVMStatus<F> {
        &self.status
    }

    /// Sets the VM status to [ACVMStatus::Failure] using the provided `error`.
    /// Returns the new status.
    fn fail(&mut self, error: OpcodeResolutionError<F>) -> ACVMStatus<F> {
        self.status(ACVMStatus::Failure(error))
    }

    /// Sets the status of the VM to `RequiresForeignCall`.
    /// Indicating that the VM is now waiting for a foreign call to be resolved.
    fn wait_for_foreign_call(&mut self, foreign_call: ForeignCallWaitInfo<F>) -> ACVMStatus<F> {
        self.status(ACVMStatus::RequiresForeignCall(foreign_call))
    }

    /// Return a reference to the arguments for the next pending foreign call, if one exists.
    pub fn get_pending_foreign_call(&self) -> Option<&ForeignCallWaitInfo<F>> {
        if let ACVMStatus::RequiresForeignCall(foreign_call) = &self.status {
            Some(foreign_call)
        } else {
            None
        }
    }

    /// Resolves a foreign call's [result][brillig_vm::brillig::ForeignCallResult] using a result calculated outside of the ACVM.
    ///
    /// The ACVM can then be restarted to solve the remaining Brillig VM process as well as the remaining ACIR opcodes.
    pub fn resolve_pending_foreign_call(&mut self, foreign_call_result: ForeignCallResult<F>) {
        if !matches!(self.status, ACVMStatus::RequiresForeignCall(_)) {
            panic!("ACVM is not expecting a foreign call response as no call was made");
        }

        let brillig_solver = self.brillig_solver.as_mut().expect("No active Brillig solver");
        brillig_solver.resolve_pending_foreign_call(foreign_call_result);

        // Now that the foreign call has been resolved then we can resume execution.
        self.status(ACVMStatus::InProgress);
    }

    /// Sets the status of the VM to `RequiresAcirCall`
    /// Indicating that the VM is now waiting for an ACIR call to be resolved
    fn wait_for_acir_call(&mut self, acir_call: AcirCallWaitInfo<F>) -> ACVMStatus<F> {
        self.status(ACVMStatus::RequiresAcirCall(acir_call))
    }

    /// Resolves an ACIR call's result (simply a list of fields) using a result calculated by a separate ACVM instance.
    ///
    /// The current ACVM instance can then be restarted to solve the remaining ACIR opcodes.
    pub fn resolve_pending_acir_call(&mut self, call_result: Vec<F>) {
        if !matches!(self.status, ACVMStatus::RequiresAcirCall(_)) {
            panic!("ACVM is not expecting an ACIR call response as no call was made");
        }

        if self.acir_call_counter < self.acir_call_results.len() {
            panic!("No unresolved ACIR calls");
        }
        self.acir_call_results.push(call_result);

        // Now that the ACIR call has been resolved then we can resume execution.
        self.status(ACVMStatus::InProgress);
    }

    /// Executes the ACVM's circuit until execution halts.
    ///
    /// Execution can halt due to three reasons:
    /// 1. All opcodes have been executed successfully.
    /// 2. The circuit has been found to be unsatisfiable.
    /// 2. A Brillig [foreign call][`ForeignCallWaitInfo`] has been encountered and must be resolved.
    pub fn solve(&mut self) -> ACVMStatus<F> {
        while self.status == ACVMStatus::InProgress {
            self.solve_opcode();
        }
        self.status.clone()
    }

    fn current_opcode(&self) -> &'a Opcode<F> {
        &self.opcodes[self.instruction_pointer]
    }

    /// Executes a single opcode using the dedicated solver.
    ///
    /// Foreign or ACIR Calls are deferred to the caller, which will
    /// either instantiate a new ACVM to execute the called ACIR function
    /// or a custom implementation to execute the foreign call.
    /// Then it will resume execution of the current ACVM with the results of the call.
    pub fn solve_opcode(&mut self) -> ACVMStatus<F> {
        let resolution = match self.current_opcode() {
            Opcode::AssertZero(expr) => ExpressionSolver::solve(&mut self.witness_map, expr),
            Opcode::BlackBoxFuncCall(bb_func) => {
                blackbox::solve(self.backend, &mut self.witness_map, bb_func)
            }
            Opcode::MemoryInit { block_id, init, .. } => {
                MemoryOpSolver::new(init, &self.witness_map).map(|solver| {
                    let existing_block_id = self.block_solvers.insert(*block_id, solver);
                    assert!(existing_block_id.is_none(), "Memory block already initialized");
                })
            }
            Opcode::MemoryOp { block_id, op } => {
                let solver = self
                    .block_solvers
                    .get_mut(block_id)
                    .expect("Memory block should have been initialized before use");
                solver.solve_memory_op(op, &mut self.witness_map, self.backend.pedantic_solving())
            }
            Opcode::BrilligCall { id, inputs, outputs, predicate } => {
                match self.solve_brillig_call_opcode(id, inputs, outputs, predicate) {
                    Ok(Some(foreign_call)) => return self.wait_for_foreign_call(foreign_call),
                    res => res.map(|_| ()),
                }
            }
            Opcode::Call { id, inputs, outputs, predicate } => {
                match self.solve_call_opcode(id, inputs, outputs, predicate) {
                    Ok(Some(input_values)) => return self.wait_for_acir_call(input_values),
                    res => res.map(|_| ()),
                }
            }
        };
        self.handle_opcode_resolution(resolution)
    }

    /// Returns the status of the ACVM
    /// If the status is an error, it converts the error into [OpcodeResolutionError]
    fn handle_opcode_resolution(
        &mut self,
        resolution: Result<(), OpcodeResolutionError<F>>,
    ) -> ACVMStatus<F> {
        match resolution {
            Ok(()) => {
                self.instruction_pointer += 1;
                if self.instruction_pointer == self.opcodes.len() {
                    self.status(ACVMStatus::Solved)
                } else {
                    self.status(ACVMStatus::InProgress)
                }
            }
            Err(mut error) => {
                match &mut error {
                    // If we have an index out of bounds, unsatisfied constraint, or an invalid input bit size,
                    // the opcode label will be unresolved because the solvers do not have knowledge of this information.
                    // We resolve, by setting this to the corresponding opcode that we just attempted to solve.
                    OpcodeResolutionError::IndexOutOfBounds {
                        opcode_location: opcode_index,
                        ..
                    } => {
                        *opcode_index = ErrorLocation::Resolved(OpcodeLocation::Acir(
                            self.instruction_pointer(),
                        ));
                    }
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: opcode_index,
                        payload: assertion_payload,
                    } => {
                        let location = OpcodeLocation::Acir(self.instruction_pointer());
                        *opcode_index = ErrorLocation::Resolved(location);
                        *assertion_payload = self.extract_assertion_payload(location);
                    }
                    OpcodeResolutionError::InvalidInputBitSize {
                        opcode_location: opcode_index,
                        ..
                    } => {
                        let location = OpcodeLocation::Acir(self.instruction_pointer());
                        *opcode_index = ErrorLocation::Resolved(location);
                    }
                    // All other errors are thrown normally.
                    _ => (),
                };
                self.fail(error)
            }
        }
    }

    fn extract_assertion_payload(
        &self,
        location: OpcodeLocation,
    ) -> Option<ResolvedAssertionPayload<F>> {
        let (_, assertion_descriptor) =
            self.assertion_payloads.iter().find(|(loc, _)| location == *loc)?;
        let mut fields = Vec::new();
        for expr in assertion_descriptor.payload.iter() {
            match expr {
                ExpressionOrMemory::Expression(expr) => {
                    let value = get_value(expr, &self.witness_map).ok()?;
                    fields.push(value);
                }
                ExpressionOrMemory::Memory(block_id) => {
                    let memory_block = self.block_solvers.get(block_id)?;
                    fields.extend(&memory_block.block_value);
                }
            }
        }
        let error_selector = ErrorSelector::new(assertion_descriptor.error_selector);

        Some(ResolvedAssertionPayload::Raw(RawAssertionPayload {
            selector: error_selector,
            data: fields,
        }))
    }

    /// Solves a Brillig Call opcode, which represents a call to an unconstrained function.
    /// It first handles the predicate and returns zero values if the predicate is false.
    /// Then it executes (or resumes execution) the Brillig function using a Brillig VM.
    fn solve_brillig_call_opcode(
        &mut self,
        id: &BrilligFunctionId,
        inputs: &'a [BrilligInputs<F>],
        outputs: &[BrilligOutputs],
        predicate: &Option<Expression<F>>,
    ) -> Result<Option<ForeignCallWaitInfo<F>>, OpcodeResolutionError<F>> {
        let opcode_location =
            ErrorLocation::Resolved(OpcodeLocation::Acir(self.instruction_pointer()));
        if is_predicate_false(
            &self.witness_map,
            predicate,
            self.backend.pedantic_solving(),
            &opcode_location,
        )? {
            return BrilligSolver::<F, B>::zero_out_brillig_outputs(&mut self.witness_map, outputs)
                .map(|_| None);
        }

        // If we're resuming execution after resolving a foreign call then
        // there will be a cached `BrilligSolver` to avoid recomputation.
        let mut solver: BrilligSolver<'_, F, B> = match self.brillig_solver.take() {
            Some(solver) => solver,
            None => BrilligSolver::new_call(
                &self.witness_map,
                &self.block_solvers,
                inputs,
                &self.unconstrained_functions[id.as_usize()].bytecode,
                self.backend,
                self.instruction_pointer,
                *id,
                self.profiling_active,
                self.brillig_branch_to_feature_map,
            )?,
        };

        // If we're fuzzing, we need to get the fuzzing trace on an error
        let result = solver.solve().inspect_err(|_| {
            if self.brillig_fuzzing_active {
                self.brillig_fuzzing_trace = Some(solver.get_fuzzing_trace());
            };
        })?;

        match result {
            BrilligSolverStatus::ForeignCallWait(foreign_call) => {
                // Cache the current state of the solver
                self.brillig_solver = Some(solver);
                Ok(Some(foreign_call))
            }
            BrilligSolverStatus::InProgress => {
                unreachable!("Brillig solver still in progress")
            }
            BrilligSolverStatus::Finished => {
                if self.brillig_fuzzing_active {
                    self.brillig_fuzzing_trace = Some(solver.get_fuzzing_trace());
                }
                // Write execution outputs
                if self.profiling_active {
                    let profiling_info =
                        solver.finalize_with_profiling(&mut self.witness_map, outputs)?;
                    profiling_info.into_iter().for_each(|sample| {
                        let mapped =
                            sample.call_stack.into_iter().map(|loc| OpcodeLocation::Brillig {
                                acir_index: self.instruction_pointer,
                                brillig_index: loc,
                            });
                        self.profiling_samples.push(ProfilingSample {
                            call_stack: std::iter::once(OpcodeLocation::Acir(
                                self.instruction_pointer,
                            ))
                            .chain(mapped)
                            .collect(),
                            brillig_function_id: Some(*id),
                        });
                    });
                } else {
                    solver.finalize(&mut self.witness_map, outputs)?;
                }

                Ok(None)
            }
        }
    }

    // This function is used by the debugger
    pub fn step_into_brillig(&mut self) -> StepResult<'a, F, B> {
        let Opcode::BrilligCall { id, inputs, outputs, predicate } = self.current_opcode() else {
            return StepResult::Status(self.solve_opcode());
        };

        let opcode_location =
            ErrorLocation::Resolved(OpcodeLocation::Acir(self.instruction_pointer()));
        let witness = &mut self.witness_map;
        let should_skip = match is_predicate_false(
            witness,
            predicate,
            self.backend.pedantic_solving(),
            &opcode_location,
        ) {
            Ok(result) => result,
            Err(err) => return StepResult::Status(self.handle_opcode_resolution(Err(err))),
        };
        if should_skip {
            let resolution = BrilligSolver::<F, B>::zero_out_brillig_outputs(witness, outputs);
            return StepResult::Status(self.handle_opcode_resolution(resolution));
        }

        let solver = BrilligSolver::new_call(
            witness,
            &self.block_solvers,
            inputs,
            &self.unconstrained_functions[id.as_usize()].bytecode,
            self.backend,
            self.instruction_pointer,
            *id,
            self.profiling_active,
            self.brillig_branch_to_feature_map,
        );
        match solver {
            Ok(solver) => StepResult::IntoBrillig(solver),
            Err(..) => StepResult::Status(self.handle_opcode_resolution(solver.map(|_| ()))),
        }
    }

    // This function is used by the debugger
    pub fn finish_brillig_with_solver(&mut self, solver: BrilligSolver<'a, F, B>) -> ACVMStatus<F> {
        if !matches!(self.current_opcode(), Opcode::BrilligCall { .. }) {
            unreachable!("Not executing a Brillig/BrilligCall opcode");
        }
        self.brillig_solver = Some(solver);
        self.solve_opcode()
    }

    /// Defer execution of the ACIR call opcode to the caller, or finalize the execution.
    /// 1. It first handles the predicate and return zero values if the predicate is false.
    /// 2. If the results of the execution are not available, it issues a 'AcirCallWaitInfo'
    ///    to notify the caller that it (the caller) needs to execute the ACIR function.
    /// 3. If the results are available, it updates the witness map and indicates that the opcode is solved.
    pub fn solve_call_opcode(
        &mut self,
        id: &AcirFunctionId,
        inputs: &[Witness],
        outputs: &[Witness],
        predicate: &Option<Expression<F>>,
    ) -> Result<Option<AcirCallWaitInfo<F>>, OpcodeResolutionError<F>> {
        let opcode_location =
            ErrorLocation::Resolved(OpcodeLocation::Acir(self.instruction_pointer()));
        if *id == AcirFunctionId(0) {
            return Err(OpcodeResolutionError::AcirMainCallAttempted { opcode_location });
        }

        if is_predicate_false(
            &self.witness_map,
            predicate,
            self.backend.pedantic_solving(),
            &opcode_location,
        )? {
            // Zero out the outputs if we have a false predicate
            for output in outputs {
                insert_value(output, F::zero(), &mut self.witness_map)?;
            }
            return Ok(None);
        }

        if self.acir_call_counter >= self.acir_call_results.len() {
            let mut initial_witness = WitnessMap::default();
            for (i, input_witness) in inputs.iter().enumerate() {
                let input_value = *witness_to_value(&self.witness_map, *input_witness)?;
                initial_witness.insert(Witness(i as u32), input_value);
            }
            return Ok(Some(AcirCallWaitInfo { id: *id, initial_witness }));
        }

        let result_values = &self.acir_call_results[self.acir_call_counter];
        if outputs.len() != result_values.len() {
            return Err(OpcodeResolutionError::AcirCallOutputsMismatch {
                opcode_location,
                results_size: result_values.len() as u32,
                outputs_size: outputs.len() as u32,
            });
        }

        for (output_witness, result_value) in outputs.iter().zip(result_values) {
            insert_value(output_witness, *result_value, &mut self.witness_map)?;
        }

        self.acir_call_counter += 1;
        Ok(None)
    }
}

// Returns the concrete value for a particular witness
// If the witness has no assignment, then
// an error is returned
pub fn witness_to_value<F>(
    initial_witness: &WitnessMap<F>,
    witness: Witness,
) -> Result<&F, OpcodeResolutionError<F>> {
    match initial_witness.get(&witness) {
        Some(value) => Ok(value),
        None => Err(OpcodeNotSolvable::MissingAssignment(witness.0).into()),
    }
}

pub fn input_to_value<F: AcirField>(
    initial_witness: &WitnessMap<F>,
    input: FunctionInput<F>,
) -> Result<F, OpcodeResolutionError<F>> {
    match input {
        FunctionInput::Witness(witness) => {
            let initial_value = *witness_to_value(initial_witness, witness)?;
            Ok(initial_value)
        }
        FunctionInput::Constant(value) => Ok(value),
    }
}

pub fn check_bit_size<F: AcirField>(
    value: F,
    num_bits: u32,
) -> Result<(), OpcodeResolutionError<F>> {
    if value.num_bits() <= num_bits {
        Ok(())
    } else {
        let value_num_bits = value.num_bits();
        let value = value.to_string();
        Err(OpcodeResolutionError::InvalidInputBitSize {
            opcode_location: ErrorLocation::Unresolved,
            invalid_input_bit_size: InvalidInputBitSize {
                value,
                value_num_bits,
                max_bits: num_bits,
            },
        })
    }
}

/// Returns the concrete value for a particular expression
/// If the value cannot be computed, it returns an 'OpcodeNotSolvable' error.
pub fn get_value<F: AcirField>(
    expr: &Expression<F>,
    initial_witness: &WitnessMap<F>,
) -> Result<F, OpcodeResolutionError<F>> {
    let expr = ExpressionSolver::evaluate(expr, initial_witness);
    match expr.to_const() {
        Some(value) => Ok(*value),
        None => Err(OpcodeResolutionError::OpcodeNotSolvable(
            OpcodeNotSolvable::MissingAssignment(any_witness_from_expression(&expr).unwrap().0),
        )),
    }
}

/// Inserts `value` into the initial witness map under the index `witness`.
///
/// Returns an error if there was already a value in the map
/// which does not match the value that one is about to insert
pub fn insert_value<F: AcirField>(
    witness: &Witness,
    value_to_insert: F,
    initial_witness: &mut WitnessMap<F>,
) -> Result<(), OpcodeResolutionError<F>> {
    let optional_old_value = initial_witness.insert(*witness, value_to_insert);

    let old_value = match optional_old_value {
        Some(old_value) => old_value,
        None => return Ok(()),
    };

    if old_value != value_to_insert {
        return Err(OpcodeResolutionError::UnsatisfiedConstrain {
            opcode_location: ErrorLocation::Unresolved,
            payload: None,
        });
    }

    Ok(())
}

// Returns one witness belonging to an expression, in no relevant order
// Returns None if the expression is const
// The function is used during partial witness generation to report unsolved witness
fn any_witness_from_expression<F>(expr: &Expression<F>) -> Option<Witness> {
    if expr.linear_combinations.is_empty() {
        if expr.mul_terms.is_empty() { None } else { Some(expr.mul_terms[0].1) }
    } else {
        Some(expr.linear_combinations[0].1)
    }
}

/// Returns `Ok(true)` if the predicate is zero
/// A predicate is used to indicate whether we should skip a certain operation.
/// If we have a zero predicate it means the operation should be skipped.
///
/// Returns `Ok(false)` when the `predicate` is `None`.
pub(crate) fn is_predicate_false<F: AcirField>(
    witness: &WitnessMap<F>,
    predicate: &Option<Expression<F>>,
    pedantic_solving: bool,
    opcode_location: &ErrorLocation,
) -> Result<bool, OpcodeResolutionError<F>> {
    match predicate {
        Some(pred) => {
            let pred_value = get_value(pred, witness)?;
            let predicate_is_false = pred_value.is_zero();
            if pedantic_solving {
                // We expect that the predicate should resolve to either 0 or 1.
                if !predicate_is_false && !pred_value.is_one() {
                    let opcode_location = *opcode_location;
                    return Err(OpcodeResolutionError::PredicateLargerThanOne {
                        opcode_location,
                        pred_value,
                    });
                }
            }
            Ok(predicate_is_false)
        }
        // If the predicate is `None`, then we treat it as an unconditional `true`
        None => Ok(false),
    }
}

/// Encapsulates a request from the ACVM that encounters an [ACIR call opcode][brillig_vm::brillig::Opcode::Call]
/// where the result of the circuit execution has not yet been provided.
///
/// The caller must resolve this opcode externally based upon the information in the request.
#[derive(Debug, Clone, PartialEq)]
pub struct AcirCallWaitInfo<F> {
    /// Index in the list of ACIR function's that should be called
    pub id: AcirFunctionId,
    /// Initial witness for the given circuit to be called
    pub initial_witness: WitnessMap<F>,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use acir::{
        FieldElement,
        native_types::{Witness, WitnessMap},
        parse_opcodes,
    };

    use crate::pwg::{ACVM, ACVMStatus, OpcodeResolutionError};

    #[test]
    fn solve_simple_circuit() {
        let initial_witness = WitnessMap::from(BTreeMap::from_iter([
            (Witness(1), FieldElement::from(1u128)),
            (Witness(2), FieldElement::from(1u128)),
            (Witness(3), FieldElement::from(2u128)),
        ]));
        let backend = acvm_blackbox_solver::StubbedBlackBoxSolver(false);

        let src = "
        BLACKBOX::RANGE input: w1, bits: 32
        BLACKBOX::RANGE input: w2, bits: 32
        BLACKBOX::RANGE input: w3, bits: 32
        ASSERT w4 = 2*w1 - w2
        ASSERT w5 = -w2*w4 + 1
        ";
        let opcodes = parse_opcodes(src).unwrap();

        let mut acvm = ACVM::new(&backend, &opcodes, initial_witness, &[], &[]);
        assert_eq!(acvm.solve(), ACVMStatus::Solved);
        assert_eq!(acvm.witness_map()[&Witness(5)], FieldElement::from(0u128));
    }

    #[test]
    fn errors_when_calling_function_zero() {
        let initial_witness =
            WitnessMap::from(BTreeMap::from_iter([(Witness(1), FieldElement::from(1u128))]));
        let backend = acvm_blackbox_solver::StubbedBlackBoxSolver(false);

        let src = "
        CALL func: 0, inputs: [w1], outputs: [w2]
        ";
        let opcodes = parse_opcodes(src).unwrap();

        let mut acvm = ACVM::new(&backend, &opcodes, initial_witness, &[], &[]);
        assert!(matches!(
            acvm.solve(),
            ACVMStatus::Failure(OpcodeResolutionError::AcirMainCallAttempted { .. })
        ));
    }
}
