use noir_debugger::context::{DebugCommandResult, DebugContext};

use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use acvm::acir::circuit::brillig::BrilligBytecode;

use noir_debugger::foreign_calls::DefaultDebugForeignCallExecutor;
use noirc_artifacts::debug::DebugArtifact;
use noirc_artifacts::trace::TraceArtifact;

use nargo::NargoError;

pub struct TracingContext<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    debug_context: DebugContext<'a, B>,
    trace_artifact: TraceArtifact, // The result of tracing, built incrementally.
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
        let debug_context = DebugContext::new(
            blackbox_solver,
            circuit,
            debug_artifact,
            initial_witness.clone(),
            foreign_call_executor,
            unconstrained_functions,
        );
        let last_result = if debug_context.get_current_opcode_location().is_none() {
            // handle circuit with no opcodes
            DebugCommandResult::Done
        } else {
            DebugCommandResult::Ok
        };

        let trace_artifact = TraceArtifact::new();

        Self { debug_context, trace_artifact, last_result }
    }

    /// Performs one debugger step. This is equivalent to running `nargo debug` and using the `next`
    /// command.
    ///
    /// Returns whether the debugger has more steps it can perform.
    fn step_debugger(&mut self) -> &DebugCommandResult {
        if let DebugCommandResult::Ok = self.last_result {
            let result = self.debug_context.next_into();
            self.last_result = result;
        }
        &self.last_result
    }
}

pub fn trace_circuit<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    circuit: &Circuit<FieldElement>,
    debug_artifact: &DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
) -> Result<TraceArtifact, NargoError<FieldElement>> {
    let mut debug_context = TracingContext::new(
        blackbox_solver,
        circuit,
        debug_artifact,
        initial_witness,
        unconstrained_functions,
    );

    let mut steps = 0;
    loop {
        match debug_context.step_debugger() {
            DebugCommandResult::Done => break,
            DebugCommandResult::Ok => steps += 1,
            DebugCommandResult::Error(err) => {
                println!("Error: {err}");
                break;
            }
            DebugCommandResult::BreakpointReached(loc) => {
                println!("Error: Breakpoint unexpected in tracer; loc={loc}");
                break;
            }
        }
    }
    println!("Total tracing steps: {steps}");

    Ok(debug_context.trace_artifact)
}
