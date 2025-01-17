mod source_location;
use noirc_evaluator::debug_trace::DebugTraceList;
use source_location::SourceLocation;

mod stack_frame;
use stack_frame::{StackFrame, Variable};

mod debugger_glue;
use debugger_glue::{get_current_source_locations, get_stack_frames};

pub mod tracer_glue;
use tracer_glue::{
    register_call, register_print, register_return, register_step, register_variables,
};

pub mod tail_diff_vecs;
use tail_diff_vecs::tail_diff_vecs;

use acvm::acir::circuit::brillig::BrilligBytecode;
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};
use nargo::NargoError;
use noir_debugger::context::{DebugCommandResult, DebugContext};
use noir_debugger::foreign_calls::DefaultDebugForeignCallExecutor;
use noirc_artifacts::debug::DebugArtifact;
use runtime_tracing::{Line, Tracer};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::debug;

/// The result from step_debugger: the debugger either paused at a new location, reached the end of
/// execution, or hit some kind of an error. Takes the error type as a parameter.
enum DebugStepResult<Error> {
    /// The debugger reached a new location and the execution is paused at it. The wrapped value is
    /// a vector, because if the next source line is a function call, one debugger step includes
    /// it, together with the first line of the called function. This is just how `nargo debug`
    /// works and a fact of life we choose not to change.
    Paused(Vec<SourceLocation>),
    /// The debuger reached the end of the program and finished execution.
    Finished,
    /// The debugger reached an error and cannot continue.
    Error(Error),
}

pub struct TracingContext<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    debug_context: DebugContext<'a, B>,
    /// The source location at the current moment of tracing.
    source_locations: Vec<SourceLocation>,
    /// The stack trace at the current moment; last call is last in the vector.
    stack_frames: Vec<StackFrame>,
    saved_return_value: Option<Variable>,
    print_output: Rc<RefCell<String>>,
}

impl<'a, B: BlackBoxFunctionSolver<FieldElement>> TracingContext<'a, B> {
    pub fn new(
        blackbox_solver: &'a B,
        circuits: &'a [Circuit<FieldElement>],
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let print_output = Rc::new(RefCell::new(String::new()));
        let print_output_clone_1 = Rc::clone(&print_output);
        let print_output_clone_2 = Rc::clone(&print_output);
        let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
            nargo::PrintOutput::PrintCallback(Box::new(move |s| {
                *Rc::clone(&print_output_clone_1).borrow_mut() = s
            })),
            debug_artifact,
        ));
        let debug_context = DebugContext::new(
            blackbox_solver,
            circuits,
            debug_artifact,
            initial_witness.clone(),
            foreign_call_executor,
            unconstrained_functions,
        );

        Self {
            debug_context,
            source_locations: vec![],
            stack_frames: vec![],
            saved_return_value: None,
            print_output: print_output_clone_2,
        }
    }

    /// Steps the debugger until a new line is reached, or the debugger returns anything other than
    /// Ok.
    ///
    /// Propagates the debugger result.
    fn step_debugger(&mut self) -> DebugStepResult<NargoError<FieldElement>> {
        loop {
            match self.debug_context.next_into() {
                DebugCommandResult::Done => return DebugStepResult::Finished,
                DebugCommandResult::Error(error) => return DebugStepResult::Error(error),
                DebugCommandResult::BreakpointReached(loc) => {
                    // Note: this is panic! instead of an error, because it is more serious and
                    // indicates an internal inconsistency, rather than a recoverable error.
                    panic!("Error: Breakpoint unexpected in tracer; loc={loc}")
                }
                DebugCommandResult::Ok => (),
            }

            let source_locations = get_current_source_locations(&self.debug_context);
            if source_locations.is_empty() {
                println!("Warning: no call stack");
                continue;
            };

            if self.source_locations.len() == source_locations.len()
                && self.source_locations.last().unwrap() == source_locations.last().unwrap()
            {
                // Continue stepping until a new line in the same file is reached, or the current file
                // has changed.
                // TODO(coda-bug/r916): a function call could result in an extra step
                continue;
            }

            return DebugStepResult::Paused(source_locations);
        }
    }

    fn maybe_update_saved_return_value(frame: &StackFrame, saved_return_value: &mut Option<Variable>) {
        for variable in &frame.variables {
            if variable.name == "__debug_return_expr" {
                *saved_return_value = Some(variable.clone());
                break;
            }
        }
    }

    fn maybe_report_print_events(&self, tracer: &mut Tracer) {
        let mut s = self.print_output.borrow_mut();
        if !(*s).is_empty() {
            register_print(tracer, (*s).as_str());
            *s = String::new();
        }
    }

    /// Propagates information about the current execution state to `tracer`.
    fn update_record(
        &mut self,
        tracer: &mut Tracer,
        source_locations: &Vec<SourceLocation>,
        debug_trace_list: &mut Option<DebugTraceList>,
    ) {
        let stack_frames = get_stack_frames(&self.debug_context);
        let (first_nomatch, dropped_frames, new_frames) =
            tail_diff_vecs(&self.stack_frames, &stack_frames);

        for _ in dropped_frames {
            register_return(tracer, &self.saved_return_value);
            self.saved_return_value = None;
            if self.source_locations.len() > 1 {
                // This branch is for returns not from main.
                assert!(first_nomatch > 0, "no matching frames after return");
                let pre_last_index = self.source_locations.len() - 2;
                let call_site_location = &self.source_locations[pre_last_index];
                let current_location = source_locations.last().unwrap();
                if current_location != call_site_location {
                    let frame = &stack_frames[first_nomatch - 1];
                    register_step(tracer, call_site_location, debug_trace_list);
                    register_variables(tracer, frame);
                    Self::maybe_update_saved_return_value(frame, &mut self.saved_return_value);
                    self.maybe_report_print_events(tracer);
                }
            }
        }

        assert!(new_frames.len() <= 1, "more than one frame entered at the same step");
        if !new_frames.is_empty() {
            let location = self.source_locations.last().expect("no previous location before call");
            register_call(tracer, &location, new_frames[0]);
        }

        let index = stack_frames.len() as isize - 1;
        if index >= 0 {
            let index = index as usize;
            let location = &source_locations[index];
            register_step(tracer, location, debug_trace_list);
            register_variables(tracer, &stack_frames[index]);
            Self::maybe_update_saved_return_value(&stack_frames[index], &mut self.saved_return_value);
            self.maybe_report_print_events(tracer);
        }

        self.stack_frames = stack_frames;
    }
}

pub fn trace_circuit<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    circuits: &[Circuit<FieldElement>],
    debug_artifact: &DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
    mut debug_trace_list: Option<DebugTraceList>,
    tracer: &mut Tracer,
) -> Result<(), NargoError<FieldElement>> {
    let mut tracing_context = TracingContext::new(
        blackbox_solver,
        circuits,
        debug_artifact,
        initial_witness,
        unconstrained_functions,
    );

    if tracing_context.debug_context.get_current_debug_location().is_none() {
        println!("Warning: circuit contains no opcodes; generating no trace");
        return Ok(());
    }

    let SourceLocation { filepath, line_number } = SourceLocation::create_unknown();
    tracer.start(&PathBuf::from(filepath.to_string()), Line(line_number as i64));
    loop {
        let source_locations = match tracing_context.step_debugger() {
            DebugStepResult::Finished => break,
            DebugStepResult::Error(err) => {
                println!("Error: {err}");
                break;
            }
            DebugStepResult::Paused(source_location) => source_location,
        };

        debug!("debugger stepped until line {:?}", source_locations.last().unwrap());

        tracing_context.update_record(tracer, &source_locations, &mut debug_trace_list);

        // This update is intentionally explicit here, to show what drives the loop.
        tracing_context.source_locations = source_locations;
    }

    Ok(())
}
