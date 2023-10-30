use acvm::acir::circuit::{Opcode, OpcodeLocation};
use acvm::pwg::{
    ACVMStatus, BrilligSolver, BrilligSolverStatus, ForeignCallWaitInfo, StepResult, ACVM,
};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::artifacts::debug::DebugArtifact;
use nargo::errors::{ExecutionError, Location};
use nargo::ops::ForeignCallExecutor;
use nargo::NargoError;

use std::collections::{hash_set::Iter, HashSet};

#[derive(Debug)]
pub(super) enum DebugCommandResult {
    Done,
    Ok,
    BreakpointReached(OpcodeLocation),
    Error(NargoError),
}

pub(super) struct DebugContext<'a, B: BlackBoxFunctionSolver> {
    acvm: ACVM<'a, B>,
    brillig_solver: Option<BrilligSolver<'a, B>>,
    foreign_call_executor: ForeignCallExecutor,
    debug_artifact: &'a DebugArtifact,
    show_output: bool,
    breakpoints: HashSet<OpcodeLocation>,
}

impl<'a, B: BlackBoxFunctionSolver> DebugContext<'a, B> {
    pub(super) fn new(
        blackbox_solver: &'a B,
        circuit: &'a Circuit,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap,
    ) -> Self {
        Self {
            acvm: ACVM::new(blackbox_solver, &circuit.opcodes, initial_witness),
            brillig_solver: None,
            foreign_call_executor: ForeignCallExecutor::default(),
            debug_artifact,
            show_output: true,
            breakpoints: HashSet::new(),
        }
    }

    pub(super) fn get_opcodes(&self) -> &[Opcode] {
        self.acvm.opcodes()
    }

    pub(super) fn get_current_opcode_location(&self) -> Option<OpcodeLocation> {
        let ip = self.acvm.instruction_pointer();
        if ip >= self.get_opcodes().len() {
            None
        } else if let Some(ref solver) = self.brillig_solver {
            Some(OpcodeLocation::Brillig {
                acir_index: ip,
                brillig_index: solver.program_counter(),
            })
        } else {
            Some(OpcodeLocation::Acir(ip))
        }
    }

    pub(super) fn get_current_source_location(&self) -> Option<Vec<Location>> {
        self.get_current_opcode_location()
            .as_ref()
            .and_then(|location| self.debug_artifact.debug_symbols[0].opcode_location(location))
    }

    fn step_brillig_opcode(&mut self) -> DebugCommandResult {
        let Some(mut solver) = self.brillig_solver.take() else {
            unreachable!("Missing Brillig solver");
        };
        match solver.step() {
            Ok(BrilligSolverStatus::InProgress) => {
                self.brillig_solver = Some(solver);
                if self.breakpoint_reached() {
                    DebugCommandResult::BreakpointReached(
                        self.get_current_opcode_location()
                            .expect("Breakpoint reached but we have no location"),
                    )
                } else {
                    DebugCommandResult::Ok
                }
            }
            Ok(BrilligSolverStatus::Finished) => {
                let status = self.acvm.finish_brillig_with_solver(solver);
                self.handle_acvm_status(status)
            }
            Ok(BrilligSolverStatus::ForeignCallWait(foreign_call)) => {
                self.brillig_solver = Some(solver);
                self.handle_foreign_call(foreign_call)
            }
            Err(err) => DebugCommandResult::Error(NargoError::ExecutionError(
                ExecutionError::SolvingError(err),
            )),
        }
    }

    fn handle_foreign_call(&mut self, foreign_call: ForeignCallWaitInfo) -> DebugCommandResult {
        let foreign_call_result =
            self.foreign_call_executor.execute(&foreign_call, self.show_output);
        match foreign_call_result {
            Ok(foreign_call_result) => {
                self.acvm.resolve_pending_foreign_call(foreign_call_result);
                // TODO: should we retry executing the opcode somehow in this case?
                DebugCommandResult::Ok
            }
            Err(error) => DebugCommandResult::Error(error),
        }
    }

    fn handle_acvm_status(&mut self, status: ACVMStatus) -> DebugCommandResult {
        if let ACVMStatus::RequiresForeignCall(foreign_call) = status {
            return self.handle_foreign_call(foreign_call);
        }

        match status {
            ACVMStatus::Solved => DebugCommandResult::Done,
            ACVMStatus::InProgress => {
                if self.breakpoint_reached() {
                    DebugCommandResult::BreakpointReached(
                        self.get_current_opcode_location()
                            .expect("Breakpoint reached but we have no location"),
                    )
                } else {
                    DebugCommandResult::Ok
                }
            }
            ACVMStatus::Failure(error) => DebugCommandResult::Error(NargoError::ExecutionError(
                ExecutionError::SolvingError(error),
            )),
            ACVMStatus::RequiresForeignCall(_) => {
                unreachable!("Unexpected pending foreign call resolution");
            }
        }
    }

    pub(super) fn step_into_opcode(&mut self) -> DebugCommandResult {
        if matches!(self.brillig_solver, Some(_)) {
            return self.step_brillig_opcode();
        }

        match self.acvm.step_into_brillig_opcode() {
            StepResult::IntoBrillig(solver) => {
                self.brillig_solver = Some(solver);
                self.step_brillig_opcode()
            }
            StepResult::Status(status) => self.handle_acvm_status(status),
        }
    }

    pub(super) fn step_acir_opcode(&mut self) -> DebugCommandResult {
        let status = if let Some(solver) = self.brillig_solver.take() {
            self.acvm.finish_brillig_with_solver(solver)
        } else {
            self.acvm.solve_opcode()
        };
        self.handle_acvm_status(status)
    }

    pub(super) fn next(&mut self) -> DebugCommandResult {
        let start_location = self.get_current_source_location();
        loop {
            let result = self.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
            let new_location = self.get_current_source_location();
            if new_location.is_some() && new_location != start_location {
                return DebugCommandResult::Ok;
            }
        }
    }

    pub(super) fn cont(&mut self) -> DebugCommandResult {
        loop {
            let result = self.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
        }
    }

    fn breakpoint_reached(&self) -> bool {
        if let Some(location) = self.get_current_opcode_location() {
            self.breakpoints.contains(&location)
        } else {
            false
        }
    }

    pub(super) fn is_valid_location(&self, location: &OpcodeLocation) -> bool {
        let opcodes = self.get_opcodes();
        match *location {
            OpcodeLocation::Acir(acir_index) => acir_index < opcodes.len(),
            OpcodeLocation::Brillig { acir_index, brillig_index } => {
                acir_index < opcodes.len()
                    && matches!(opcodes[acir_index], Opcode::Brillig(..))
                    && {
                        if let Opcode::Brillig(ref brillig) = opcodes[acir_index] {
                            brillig_index < brillig.bytecode.len()
                        } else {
                            false
                        }
                    }
            }
        }
    }

    pub(super) fn is_breakpoint_set(&self, location: &OpcodeLocation) -> bool {
        self.breakpoints.contains(location)
    }

    pub(super) fn add_breakpoint(&mut self, location: OpcodeLocation) {
        _ = self.breakpoints.insert(location);
    }

    pub(super) fn delete_breakpoint(&mut self, location: &OpcodeLocation) {
        _ = self.breakpoints.remove(location);
    }

    pub(super) fn iterate_breakpoints(&self) -> Iter<'_, OpcodeLocation> {
        self.breakpoints.iter()
    }

    pub(super) fn is_solved(&self) -> bool {
        matches!(self.acvm.get_status(), ACVMStatus::Solved)
    }

    pub fn finalize(self) -> WitnessMap {
        self.acvm.finalize()
    }
}
