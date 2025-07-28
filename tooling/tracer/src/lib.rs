mod source_location;
use acvm::acir::circuit::{ErrorSelector, OpcodeLocation};
use acvm::pwg::{OpcodeResolutionError, RawAssertionPayload, ResolvedAssertionPayload};
use nargo::errors::ExecutionError;
use noirc_abi::AbiErrorType;
use source_location::SourceLocation;

mod stack_frame;
use stack_frame::{StackFrame, Variable};

mod debugger_glue;
use debugger_glue::{
    get_current_source_locations, get_source_locations_for_call_stack, get_stack_frames,
};

pub mod tracer_glue;
use tracer_glue::{
    register_call, register_error, register_print, register_return, register_step,
    register_variables,
};

pub mod tail_diff_vecs;
use tail_diff_vecs::tail_diff_vecs;

use acvm::acir::circuit::brillig::{BrilligBytecode, BrilligFunctionId};
use acvm::{AcirField, BlackBoxFunctionSolver, FieldElement};
use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use nargo::NargoError;
use noir_debugger::context::{DebugCommandResult, DebugContext};
use noir_debugger::foreign_calls::DefaultDebugForeignCallExecutor;
use noirc_artifacts::debug::DebugArtifact;
use runtime_tracing::{TraceWriter, TypeKind};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Write;
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

pub struct StringWriter {
    target: Rc<RefCell<String>>,
}

impl StringWriter {
    pub fn new(target: Rc<RefCell<String>>) -> Self {
        Self { target }
    }

    pub fn get_inner(&self) -> Rc<RefCell<String>> {
        Rc::clone(&self.target)
    }
}

impl Write for StringWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.target.borrow_mut().push_str(&s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
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
        circuit: &'a [Circuit<FieldElement>],
        debug_artifact: &'a DebugArtifact,
        initial_witness: WitnessMap<FieldElement>,
        unconstrained_functions: &'a [BrilligBytecode<FieldElement>],
    ) -> Self {
        let print_output = Rc::new(RefCell::new(String::new()));
        let writer: StringWriter = StringWriter::new(Rc::clone(&print_output));

        let foreign_call_executor = Box::new(DefaultDebugForeignCallExecutor::from_artifact(
            writer,
            None,
            debug_artifact,
            None,
            String::new(),
        ));
        let debug_context = DebugContext::new(
            blackbox_solver,
            circuit,
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
            print_output,
        }
    }

    fn are_src_locations_equal(
        src_location_1: &[SourceLocation],
        src_location_2: &[SourceLocation],
    ) -> bool {
        if src_location_1.len() != src_location_2.len() {
            false
        } else {
            for i in 0..src_location_1.len() {
                if src_location_1[i] != src_location_2[i] {
                    return false;
                }
            }
            true
        }
    }

    /// Steps debugging execution until the next source location, while simultaneously checking for return values after each opcode
    fn next_into_with_return_values_check(&mut self) -> DebugCommandResult {
        let start_location = self.debug_context.get_current_source_location();
        loop {
            let result = self.debug_context.step_into_opcode();
            if !matches!(result, DebugCommandResult::Ok) {
                return result;
            }

            // check for return values
            let stack_frames = get_stack_frames(&self.debug_context);
            if let Some(frame) = stack_frames.last() {
                Self::maybe_update_saved_return_value(frame, &mut self.saved_return_value);
            }

            let new_location = self.debug_context.get_current_source_location();
            if new_location.is_some() && new_location != start_location {
                return DebugCommandResult::Ok;
            }
        }
    }

    /// Steps the debugger until a new line is reached, or the debugger returns anything other than
    /// Ok.
    ///
    /// Propagates the debugger result.
    fn step_debugger(&mut self) -> DebugStepResult<NargoError<FieldElement>> {
        loop {
            match self.next_into_with_return_values_check() {
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

            if Self::are_src_locations_equal(&self.source_locations, &source_locations) {
                // Continue stepping until a new line in the same file is reached, or the current file
                // has changed.
                continue;
            }

            return DebugStepResult::Paused(source_locations);
        }
    }

    fn maybe_update_saved_return_value(
        frame: &StackFrame,
        saved_return_value: &mut Option<Variable>,
    ) {
        for variable in &frame.variables {
            if variable.name == "__debug_return_expr" {
                *saved_return_value = Some(variable.clone());
                break;
            }
        }
    }

    fn maybe_report_print_events(&self, tracer: &mut dyn TraceWriter) {
        let mut s = self.print_output.borrow_mut();
        if !(*s).is_empty() {
            register_print(tracer, (*s).as_str());
            *s = String::new();
        }
    }

    /// Propagates information about the current execution state to `tracer`.
    fn update_record(&mut self, tracer: &mut dyn TraceWriter, source_locations: &[SourceLocation]) {
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
                    register_step(tracer, call_site_location);
                    register_variables(tracer, frame);
                    Self::maybe_update_saved_return_value(frame, &mut self.saved_return_value);
                    self.maybe_report_print_events(tracer);
                }
            }
        }

        assert!(new_frames.len() <= 1, "more than one frame entered at the same step");
        if !new_frames.is_empty() {
            let location = self.source_locations.last().expect("no previous location before call");
            register_call(tracer, location, new_frames[0]);
        }

        let index = stack_frames.len() as isize - 1;
        if index >= 0 {
            let index = index as usize;
            let location = &source_locations[index];
            self.maybe_report_print_events(tracer);
            register_step(tracer, location);
            register_variables(tracer, &stack_frames[index]);
            Self::maybe_update_saved_return_value(
                &stack_frames[index],
                &mut self.saved_return_value,
            );
        }

        self.stack_frames = stack_frames;
    }
}

pub fn trace_circuit<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    circuit: &[Circuit<FieldElement>],
    debug_artifact: &DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
    error_types: &BTreeMap<ErrorSelector, AbiErrorType>,
    tracer: &mut dyn TraceWriter,
) -> Result<(), NargoError<FieldElement>> {
    let mut tracing_context = TracingContext::new(
        blackbox_solver,
        circuit,
        debug_artifact,
        initial_witness,
        unconstrained_functions,
    );

    if tracing_context.debug_context.get_current_debug_location().is_none() {
        println!("Warning: circuit contains no opcodes; generating no trace");
        return Ok(());
    }

    let _ = TraceWriter::ensure_type_id(tracer, TypeKind::None, "None");
    loop {
        let source_locations = match tracing_context.step_debugger() {
            DebugStepResult::Finished => break,
            DebugStepResult::Error(err) => match &err {
                NargoError::ExecutionError(ExecutionError::SolvingError(
                    OpcodeResolutionError::BrilligFunctionFailed {
                        function_id,
                        call_stack,
                        payload,
                    },
                    _,
                )) => {
                    handle_function_error(
                        function_id,
                        call_stack,
                        payload.as_ref(),
                        error_types,
                        &err,
                        &mut tracing_context,
                        tracer,
                    );
                    break;
                }
                NargoError::ExecutionError(ExecutionError::AssertionFailed(
                    payload,
                    call_stack,
                    Some(function_id),
                )) => {
                    let opcode_locations =
                        call_stack.iter().map(|loc| loc.opcode_location).collect::<Vec<_>>();
                    handle_function_error(
                        function_id,
                        &opcode_locations,
                        Some(payload),
                        error_types,
                        &err,
                        &mut tracing_context,
                        tracer,
                    );
                    break;
                }
                _ => {
                    println!("Error: {err}");
                    break;
                }
            },
            DebugStepResult::Paused(source_location) => source_location,
        };

        debug!("debugger stepped until line {:?}", source_locations.last().unwrap());

        tracing_context.update_record(tracer, &source_locations);

        // This update is intentionally explicit here, to show what drives the loop.
        tracing_context.source_locations = source_locations;
    }

    Ok(())
}

fn handle_function_error<F, B: BlackBoxFunctionSolver<FieldElement>>(
    function_id: &BrilligFunctionId,
    call_stack: &[OpcodeLocation],
    payload: Option<&ResolvedAssertionPayload<F>>,
    error_types: &BTreeMap<ErrorSelector, AbiErrorType>,
    err: &NargoError<F>,
    tracing_context: &mut TracingContext<B>,
    tracer: &mut dyn TraceWriter,
) where
    F: AcirField,
{
    let err_str =
        if let Some(ResolvedAssertionPayload::Raw(RawAssertionPayload { selector, data: _ })) =
            payload
        {
            if let Some(AbiErrorType::String { string }) = error_types.get(selector) {
                string.clone()
            } else {
                err.to_string()
            }
        } else {
            err.to_string()
        };

    let debug_locations = call_stack
        .iter()
        .map(|opcode_loc| noir_debugger::context::DebugLocation {
            circuit_id: 0,
            opcode_location: *opcode_loc,
            brillig_function_id: Some(*function_id),
        })
        .collect();

    let source_locations =
        get_source_locations_for_call_stack(&tracing_context.debug_context, debug_locations);

    tracing_context.update_record(tracer, &source_locations);
    register_error(tracer, &err_str);
}
