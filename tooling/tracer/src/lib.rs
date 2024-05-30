use noir_debugger::context::{DebugCommandResult, DebugContext};

use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use acvm::acir::circuit::brillig::BrilligBytecode;

use nargo::artifacts::debug::DebugArtifact;
use noir_debugger::foreign_calls::DefaultDebugForeignCallExecutor;

use nargo::NargoError;

use std::cell::RefCell;

pub struct TracingContext<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    context: DebugContext<'a, B>,
    last_result: DebugCommandResult,
}

impl<'a, B: BlackBoxFunctionSolver<FieldElement>> TracingContext<'a, B> {
    pub fn new(
        blackbox_solver: &'a B,
        circuit: &'a Circuit<FieldElement>,
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let foreign_call_executor =
            Box::new(DefaultDebugForeignCallExecutor::from_artifact(true, debug_artifact));
        let context = DebugContext::new(
            blackbox_solver,
            circuit,
            debug_artifact,
            initial_witness.clone(),
            foreign_call_executor,
            unconstrained_functions,
        );
        let last_result = if context.get_current_opcode_location().is_none() {
            // handle circuit with no opcodes
            DebugCommandResult::Done
        } else {
            DebugCommandResult::Ok
        };
        Self { context, last_result }
    }

    fn validate_in_progress(&self) -> bool {
        match self.last_result {
            DebugCommandResult::Ok | DebugCommandResult::BreakpointReached(..) => true,
            DebugCommandResult::Done => {
                println!("Execution finished");
                false
            }
            DebugCommandResult::Error(ref error) => {
                println!("ERROR: {}", error);
                false
            }
        }
    }

    fn handle_debug_command_result(&mut self, result: DebugCommandResult) {
        match &result {
            DebugCommandResult::BreakpointReached(location) => {
                println!("Stopped at breakpoint in opcode {}", location);
            }
            DebugCommandResult::Error(error) => {
                println!("ERROR: {}", error);
            }
            _ => (),
        }
        self.last_result = result;
    }

    fn next_into(&mut self) -> bool {
        if self.validate_in_progress() {
            let result = self.context.next_into();
            let has_more_steps = match result {
                DebugCommandResult::Done => false,
                DebugCommandResult::Error(_) => false,
                _ => true,
            };
            self.handle_debug_command_result(result);
            has_more_steps
        } else {
            false
        }
    }

    fn is_solved(&self) -> bool {
        self.context.is_solved()
    }

    fn finalize(self) -> WitnessMap<FieldElement> {
        self.context.finalize()
    }
}

pub fn trace_circuit<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    circuit: &Circuit<FieldElement>,
    debug_artifact: &DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
) -> Result<Option<WitnessMap<FieldElement>>, NargoError> {
    let context = RefCell::new(TracingContext::new(
        blackbox_solver,
        circuit,
        debug_artifact,
        initial_witness,
        unconstrained_functions,
    ));

    let mut steps = 0;
    while (&context).borrow_mut().next_into() {
        steps += 1;
    }
    println!("Total tracing steps: {steps}");

    if context.borrow().is_solved() {
        let solved_witness = context.into_inner().finalize();
        Ok(Some(solved_witness))
    } else {
        Ok(None)
    }
}
