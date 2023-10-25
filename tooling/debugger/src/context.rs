use acvm::acir::circuit::{Opcode, OpcodeLocation};
use acvm::pwg::{
    ACVMStatus, BrilligSolver, BrilligSolverStatus, ErrorLocation, OpcodeResolutionError,
    StepResult, ACVM,
};
use acvm::BlackBoxFunctionSolver;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};

use nargo::errors::ExecutionError;
use nargo::NargoError;

use nargo::ops::ForeignCallExecutor;

pub(super) enum DebugCommandResult {
    Done,
    Ok,
}

pub(super) struct DebugContext<'a, B: BlackBoxFunctionSolver> {
    acvm: ACVM<'a, B>,
    brillig_solver: Option<BrilligSolver<'a, B>>,
    foreign_call_executor: ForeignCallExecutor,
    circuit: &'a Circuit,
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
            circuit,
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

    fn step_brillig_opcode(&mut self) -> Result<DebugCommandResult, NargoError> {
        let Some(mut solver) = self.brillig_solver.take() else {
            unreachable!("Missing Brillig solver");
        };
        match solver.step() {
            Ok(status) => match status {
                BrilligSolverStatus::InProgress => {
                    self.brillig_solver = Some(solver);
                    Ok(DebugCommandResult::Ok)
                }
                BrilligSolverStatus::Finished => {
                    let status = self.acvm.finish_brillig_with_solver(solver);
                    self.handle_acvm_status(status)
                }
                BrilligSolverStatus::ForeignCallWait(foreign_call) => {
                    let foreign_call_result =
                        self.foreign_call_executor.execute(&foreign_call, self.show_output)?;
                    solver.resolve_pending_foreign_call(foreign_call_result);
                    self.brillig_solver = Some(solver);
                    Ok(DebugCommandResult::Ok)
                }
            },
            Err(err) => self.handle_acvm_status(ACVMStatus::Failure(err)),
        }
    }

    pub(super) fn step_into_opcode(&mut self) -> Result<DebugCommandResult, NargoError> {
        if matches!(self.brillig_solver, Some(_)) {
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

    pub(super) fn step_acir_opcode(&mut self) -> Result<DebugCommandResult, NargoError> {
        let status = if let Some(solver) = self.brillig_solver.take() {
            self.acvm.finish_brillig_with_solver(solver)
        } else {
            self.acvm.solve_opcode()
        };
        self.handle_acvm_status(status)
    }

    fn handle_acvm_status(&mut self, status: ACVMStatus) -> Result<DebugCommandResult, NargoError> {
        match status {
            ACVMStatus::Solved => Ok(DebugCommandResult::Done),
            ACVMStatus::InProgress => Ok(DebugCommandResult::Ok),
            ACVMStatus::Failure(error) => {
                let call_stack = match &error {
                    OpcodeResolutionError::UnsatisfiedConstrain {
                        opcode_location: ErrorLocation::Resolved(opcode_location),
                    } => Some(vec![*opcode_location]),
                    OpcodeResolutionError::BrilligFunctionFailed { call_stack, .. } => {
                        Some(call_stack.clone())
                    }
                    _ => None,
                };

                Err(NargoError::ExecutionError(match call_stack {
                    Some(call_stack) => {
                        if let Some(assert_message) = self.circuit.get_assert_message(
                            *call_stack.last().expect("Call stacks should not be empty"),
                        ) {
                            ExecutionError::AssertionFailed(assert_message.to_owned(), call_stack)
                        } else {
                            ExecutionError::SolvingError(error)
                        }
                    }
                    None => ExecutionError::SolvingError(error),
                }))
            }
            ACVMStatus::RequiresForeignCall(foreign_call) => {
                let foreign_call_result =
                    self.foreign_call_executor.execute(&foreign_call, self.show_output)?;
                self.acvm.resolve_pending_foreign_call(foreign_call_result);
                Ok(DebugCommandResult::Ok)
            }
        }
    }

    pub(super) fn cont(&mut self) -> Result<DebugCommandResult, NargoError> {
        loop {
            match self.step_acir_opcode()? {
                DebugCommandResult::Done => break,
                DebugCommandResult::Ok => {}
            }
        }
        Ok(DebugCommandResult::Done)
    }

    pub(super) fn is_finished(&self) -> bool {
        !matches!(
            self.acvm.get_status(),
            ACVMStatus::InProgress | ACVMStatus::RequiresForeignCall { .. }
        )
    }

    pub(super) fn is_solved(&self) -> bool {
        matches!(self.acvm.get_status(), ACVMStatus::Solved)
    }

    pub fn finalize(self) -> WitnessMap {
        self.acvm.finalize()
    }
}
