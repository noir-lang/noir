// Re-usable methods that backends can use to implement their PWG

use std::collections::HashMap;

use acir::{
    brillig::ForeignCallResult,
    circuit::{
        brillig::BrilligBytecode, opcodes::BlockId, AssertionPayload, ErrorSelector,
        ExpressionOrMemory, Opcode, OpcodeLocation, RawAssertionPayload, ResolvedAssertionPayload,
        STRING_ERROR_SELECTOR,
    },
    native_types::{Expression, Witness, WitnessMap},
    AcirField, BlackBoxFunc,
};
use acvm_blackbox_solver::BlackBoxResolutionError;

use self::{
    arithmetic::ExpressionSolver, blackbox::bigint::AcvmBigIntSolver, directives::solve_directives,
    memory_op::MemoryOpSolver,
};
use crate::BlackBoxFunctionSolver;

use thiserror::Error;

// arithmetic
pub(crate) mod arithmetic;
// Brillig bytecode
pub(crate) mod brillig;
// Directives
pub(crate) mod directives;
// black box functions
pub(crate) mod blackbox;
mod memory_op;

pub use self::brillig::{BrilligSolver, BrilligSolverStatus};
pub use brillig::ForeignCallWaitInfo;

#[derive(Debug, Clone, PartialEq)]
pub enum ACVMStatus<F> {
    /// All opcodes have been solved.
    Solved,

    /// The ACVM is in the process of executing the circuit.
    InProgress,

    /// The ACVM has encountered an irrecoverable error while executing the circuit and can not progress.
    /// Most commonly this will be due to an unsatisfied constraint due to invalid inputs to the circuit.
    Failure(OpcodeResolutionError<F>),

    /// The ACVM has encountered a request for a Brillig [foreign call][acir::brillig_vm::Opcode::ForeignCall]
    /// to retrieve information from outside of the ACVM. The result of the foreign call must be passed back
    /// to the ACVM using [`ACVM::resolve_pending_foreign_call`].
    ///
    /// Once this is done, the ACVM can be restarted to solve the remaining opcodes.
    RequiresForeignCall(ForeignCallWaitInfo<F>),

    /// The ACVM has encountered a request for an ACIR [call][acir::circuit::Opcode]
    /// to execute a separate ACVM instance. The result of the ACIR call must be passd back to the ACVM.
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

pub enum StepResult<'a, F, B: BlackBoxFunctionSolver<F>> {
    Status(ACVMStatus<F>),
    IntoBrillig(BrilligSolver<'a, F, B>),
}

// This enum represents the different cases in which an
// opcode can be unsolvable.
// The most common being that one of its input has not been
// assigned a value.
//
// TODO: ExpressionHasTooManyUnknowns is specific for expression solver
// TODO: we could have a error enum for expression solver failure cases in that module
// TODO that can be converted into an OpcodeNotSolvable or OpcodeResolutionError enum
#[derive(Clone, PartialEq, Eq, Debug, Error)]
pub enum OpcodeNotSolvable<F> {
    #[error("missing assignment for witness index {0}")]
    MissingAssignment(u32),
    #[error("Attempted to load uninitialized memory block")]
    MissingMemoryBlock(u32),
    #[error("expression has too many unknowns {0}")]
    ExpressionHasTooManyUnknowns(Expression<F>),
}

/// Allows to point to a specific opcode as cause in errors.
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
    IndexOutOfBounds { opcode_location: ErrorLocation, index: u32, array_size: u32 },
    #[error("Failed to solve blackbox function: {0}, reason: {1}")]
    BlackBoxFunctionFailed(BlackBoxFunc, String),
    #[error("Failed to solve brillig function")]
    BrilligFunctionFailed {
        call_stack: Vec<OpcodeLocation>,
        payload: Option<ResolvedAssertionPayload<F>>,
    },
    #[error("Attempted to call `main` with a `Call` opcode")]
    AcirMainCallAttempted { opcode_location: ErrorLocation },
    #[error("{results_size:?} result values were provided for {outputs_size:?} call output witnesses, most likely due to bad ACIR codegen")]
    AcirCallOutputsMismatch { opcode_location: ErrorLocation, results_size: u32, outputs_size: u32 },
}

impl<F> From<BlackBoxResolutionError> for OpcodeResolutionError<F> {
    fn from(value: BlackBoxResolutionError) -> Self {
        match value {
            BlackBoxResolutionError::Failed(func, reason) => {
                OpcodeResolutionError::BlackBoxFunctionFailed(func, reason)
            }
        }
    }
}

pub struct ACVM<'a, F, B: BlackBoxFunctionSolver<F>> {
    status: ACVMStatus<F>,

    backend: &'a B,

    /// Stores the solver for memory operations acting on blocks of memory disambiguated by [block][`BlockId`].
    block_solvers: HashMap<BlockId, MemoryOpSolver<F>>,

    bigint_solver: AcvmBigIntSolver,

    /// A list of opcodes which are to be executed by the ACVM.
    opcodes: &'a [Opcode<F>],
    /// Index of the next opcode to be executed.
    instruction_pointer: usize,

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
            bigint_solver: AcvmBigIntSolver::default(),
            opcodes,
            instruction_pointer: 0,
            witness_map: initial_witness,
            brillig_solver: None,
            acir_call_counter: 0,
            acir_call_results: Vec::default(),
            unconstrained_functions,
            assertion_payloads,
        }
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

    /// Resolves a foreign call's [result][acir::brillig_vm::ForeignCallResult] using a result calculated outside of the ACVM.
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

    pub fn solve_opcode(&mut self) -> ACVMStatus<F> {
        let opcode = &self.opcodes[self.instruction_pointer];

        let resolution = match opcode {
            Opcode::AssertZero(expr) => ExpressionSolver::solve(&mut self.witness_map, expr),
            Opcode::BlackBoxFuncCall(bb_func) => blackbox::solve(
                self.backend,
                &mut self.witness_map,
                bb_func,
                &mut self.bigint_solver,
            ),
            Opcode::Directive(directive) => solve_directives(&mut self.witness_map, directive),
            Opcode::MemoryInit { block_id, init, .. } => {
                let solver = self.block_solvers.entry(*block_id).or_default();
                solver.init(init, &self.witness_map)
            }
            Opcode::MemoryOp { block_id, op, predicate } => {
                let solver = self.block_solvers.entry(*block_id).or_default();
                solver.solve_memory_op(op, &mut self.witness_map, predicate)
            }
            Opcode::BrilligCall { .. } => match self.solve_brillig_call_opcode() {
                Ok(Some(foreign_call)) => return self.wait_for_foreign_call(foreign_call),
                res => res.map(|_| ()),
            },
            Opcode::Call { .. } => match self.solve_call_opcode() {
                Ok(Some(input_values)) => return self.wait_for_acir_call(input_values),
                res => res.map(|_| ()),
            },
        };
        self.handle_opcode_resolution(resolution)
    }

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
                    // If we have an index out of bounds or an unsatisfied constraint, the opcode label will be unresolved
                    // because the solvers do not have knowledge of this information.
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
        let (_, found_assertion_payload) =
            self.assertion_payloads.iter().find(|(loc, _)| location == *loc)?;
        match found_assertion_payload {
            AssertionPayload::StaticString(string) => {
                Some(ResolvedAssertionPayload::String(string.clone()))
            }
            AssertionPayload::Dynamic(error_selector, expression) => {
                let mut fields = vec![];
                for expr in expression {
                    match expr {
                        ExpressionOrMemory::Expression(expr) => {
                            let value = get_value(expr, &self.witness_map).ok()?;
                            fields.push(value);
                        }
                        ExpressionOrMemory::Memory(block_id) => {
                            let memory_block = self.block_solvers.get(block_id)?;
                            fields.extend((0..memory_block.block_len).map(|memory_index| {
                                *memory_block
                                    .block_value
                                    .get(&memory_index)
                                    .expect("All memory is initialized on creation")
                            }));
                        }
                    }
                }
                let error_selector = ErrorSelector::new(*error_selector);

                Some(match error_selector {
                    STRING_ERROR_SELECTOR => {
                        // If the error selector is 0, it means the error is a string
                        let string = fields
                            .iter()
                            .map(|field| {
                                let as_u8: u8 = field
                                    .try_to_u64()
                                    .expect("String character doesn't fit in u64")
                                    .try_into()
                                    .expect("String character doesn't fit in u8");
                                as_u8 as char
                            })
                            .collect();
                        ResolvedAssertionPayload::String(string)
                    }
                    _ => {
                        // If the error selector is not 0, it means the error is a custom error
                        ResolvedAssertionPayload::Raw(RawAssertionPayload {
                            selector: error_selector,
                            data: fields,
                        })
                    }
                })
            }
        }
    }

    fn solve_brillig_call_opcode(
        &mut self,
    ) -> Result<Option<ForeignCallWaitInfo<F>>, OpcodeResolutionError<F>> {
        let Opcode::BrilligCall { id, inputs, outputs, predicate } =
            &self.opcodes[self.instruction_pointer]
        else {
            unreachable!("Not executing a BrilligCall opcode");
        };

        if is_predicate_false(&self.witness_map, predicate)? {
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
                &self.unconstrained_functions[*id as usize].bytecode,
                self.backend,
                self.instruction_pointer,
            )?,
        };

        let result = solver.solve().map_err(|err| self.map_brillig_error(err))?;

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
                // Write execution outputs
                solver.finalize(&mut self.witness_map, outputs)?;
                Ok(None)
            }
        }
    }

    fn map_brillig_error(&self, mut err: OpcodeResolutionError<F>) -> OpcodeResolutionError<F> {
        match &mut err {
            OpcodeResolutionError::BrilligFunctionFailed { call_stack, payload } => {
                // Some brillig errors have static strings as payloads, we can resolve them here
                let last_location =
                    call_stack.last().expect("Call stacks should have at least one item");
                let assertion_descriptor =
                    self.assertion_payloads.iter().find_map(|(loc, payload)| {
                        if loc == last_location {
                            Some(payload)
                        } else {
                            None
                        }
                    });

                if let Some(AssertionPayload::StaticString(string)) = assertion_descriptor {
                    *payload = Some(ResolvedAssertionPayload::String(string.clone()));
                }

                err
            }
            _ => err,
        }
    }

    pub fn step_into_brillig(&mut self) -> StepResult<'a, F, B> {
        let Opcode::BrilligCall { id, inputs, outputs, predicate } =
            &self.opcodes[self.instruction_pointer]
        else {
            return StepResult::Status(self.solve_opcode());
        };

        let witness = &mut self.witness_map;
        let should_skip = match is_predicate_false(witness, predicate) {
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
            &self.unconstrained_functions[*id as usize].bytecode,
            self.backend,
            self.instruction_pointer,
        );
        match solver {
            Ok(solver) => StepResult::IntoBrillig(solver),
            Err(..) => StepResult::Status(self.handle_opcode_resolution(solver.map(|_| ()))),
        }
    }

    pub fn finish_brillig_with_solver(&mut self, solver: BrilligSolver<'a, F, B>) -> ACVMStatus<F> {
        if !matches!(self.opcodes[self.instruction_pointer], Opcode::BrilligCall { .. }) {
            unreachable!("Not executing a Brillig/BrilligCall opcode");
        }
        self.brillig_solver = Some(solver);
        self.solve_opcode()
    }

    pub fn solve_call_opcode(
        &mut self,
    ) -> Result<Option<AcirCallWaitInfo<F>>, OpcodeResolutionError<F>> {
        let Opcode::Call { id, inputs, outputs, predicate } =
            &self.opcodes[self.instruction_pointer]
        else {
            unreachable!("Not executing a Call opcode");
        };
        if *id == 0 {
            return Err(OpcodeResolutionError::AcirMainCallAttempted {
                opcode_location: ErrorLocation::Resolved(OpcodeLocation::Acir(
                    self.instruction_pointer(),
                )),
            });
        }

        if is_predicate_false(&self.witness_map, predicate)? {
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
                opcode_location: ErrorLocation::Resolved(OpcodeLocation::Acir(
                    self.instruction_pointer(),
                )),
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

// TODO: There is an issue open to decide on whether we need to get values from Expressions
// TODO versus just getting values from Witness
pub fn get_value<F: AcirField>(
    expr: &Expression<F>,
    initial_witness: &WitnessMap<F>,
) -> Result<F, OpcodeResolutionError<F>> {
    let expr = ExpressionSolver::evaluate(expr, initial_witness);
    match expr.to_const() {
        Some(value) => Ok(value),
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
        if expr.mul_terms.is_empty() {
            None
        } else {
            Some(expr.mul_terms[0].1)
        }
    } else {
        Some(expr.linear_combinations[0].1)
    }
}

/// Returns `true` if the predicate is zero
/// A predicate is used to indicate whether we should skip a certain operation.
/// If we have a zero predicate it means the operation should be skipped.
pub(crate) fn is_predicate_false<F: AcirField>(
    witness: &WitnessMap<F>,
    predicate: &Option<Expression<F>>,
) -> Result<bool, OpcodeResolutionError<F>> {
    match predicate {
        Some(pred) => get_value(pred, witness).map(|pred_value| pred_value.is_zero()),
        // If the predicate is `None`, then we treat it as an unconditional `true`
        None => Ok(false),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AcirCallWaitInfo<F> {
    /// Index in the list of ACIR function's that should be called
    pub id: u32,
    /// Initial witness for the given circuit to be called
    pub initial_witness: WitnessMap<F>,
}
