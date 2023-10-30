use acvm::acir::circuit::{Opcode, OpcodeLocation};
use acvm::pwg::{
    ACVMStatus, BrilligSolver, BrilligSolverStatus, ForeignCallWaitInfo, StepResult, ACVM,
};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::errors::ExecutionError;
use nargo::ops::ForeignCallExecutor;
use nargo::NargoError;

#[derive(Debug)]
pub(super) enum DebugCommandResult {
    Done,
    Ok,
    Error(NargoError),
}

pub(super) struct DebugContext<'a, B: BlackBoxFunctionSolver> {
    acvm: ACVM<'a, B>,
    brillig_solver: Option<BrilligSolver<'a, B>>,
    foreign_call_executor: ForeignCallExecutor,
    show_output: bool,
}

impl<'a, B: BlackBoxFunctionSolver> DebugContext<'a, B> {
    pub(super) fn new(
        blackbox_solver: &'a B,
        circuit: &'a Circuit,
        initial_witness: WitnessMap,
    ) -> Self {
        Self {
            acvm: ACVM::new(blackbox_solver, &circuit.opcodes, initial_witness),
            brillig_solver: None,
            foreign_call_executor: ForeignCallExecutor::default(),
            show_output: true,
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

    fn step_brillig_opcode(&mut self) -> DebugCommandResult {
        let Some(mut solver) = self.brillig_solver.take() else {
            unreachable!("Missing Brillig solver");
        };
        match solver.step() {
            Ok(status) => match status {
                BrilligSolverStatus::InProgress => {
                    self.brillig_solver = Some(solver);
                    DebugCommandResult::Ok
                }
                BrilligSolverStatus::Finished => {
                    let status = self.acvm.finish_brillig_with_solver(solver);
                    self.handle_acvm_status(status)
                }
                BrilligSolverStatus::ForeignCallWait(foreign_call) => {
                    self.brillig_solver = Some(solver);
                    self.handle_foreign_call(foreign_call)
                }
            },
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
            self.handle_foreign_call(foreign_call)
        } else {
            match status {
                ACVMStatus::Solved => DebugCommandResult::Done,
                ACVMStatus::InProgress => DebugCommandResult::Ok,
                ACVMStatus::Failure(error) => DebugCommandResult::Error(
                    NargoError::ExecutionError(ExecutionError::SolvingError(error)),
                ),
                ACVMStatus::RequiresForeignCall(_) => {
                    unreachable!("Unexpected pending foreign call resolution");
                }
            }
        }
    }

    pub(super) fn step_into_opcode(&mut self) -> DebugCommandResult {
        if self.brillig_solver.is_some() {
            self.step_brillig_opcode()
        } else {
            match self.acvm.step_into_brillig_opcode() {
                StepResult::IntoBrillig(solver) => {
                    self.brillig_solver = Some(solver);
                    self.step_brillig_opcode()
                }
                StepResult::Status(status) => self.handle_acvm_status(status),
            }
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

    pub(super) fn cont(&mut self) -> DebugCommandResult {
        loop {
            let result = self.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }
        }
    }

    pub(super) fn is_solved(&self) -> bool {
        matches!(self.acvm.get_status(), ACVMStatus::Solved)
    }

    pub fn finalize(self) -> WitnessMap {
        self.acvm.finalize()
    }
}
