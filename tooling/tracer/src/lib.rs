use noir_debugger::context::{DebugCommandResult, DebugContext};

use acvm::{acir::circuit::Circuit, acir::native_types::WitnessMap};
use acvm::{BlackBoxFunctionSolver, FieldElement};

use acvm::acir::circuit::brillig::BrilligBytecode;

use noir_debugger::foreign_calls::DefaultDebugForeignCallExecutor;
use noirc_artifacts::debug::DebugArtifact;

use fm::PathString;
use std::path::PathBuf;

use runtime_tracing::{Line, Tracer};

use nargo::NargoError;

/// A location in the source code: filename and line number (1-indexed).
#[derive(PartialEq)]
struct SourceLocation {
    filepath: PathString,
    line_number: isize,
}

impl SourceLocation {
    /// Creates a source location that represents an unknown place in the source code.
    fn create_unknown() -> SourceLocation {
        SourceLocation { filepath: PathString::from_path(PathBuf::from("?")), line_number: -1 }
    }
}

/// The result from step_debugger: the debugger either paused at a new location, reached the end of
/// execution, or hit some kind of an error. Takes the error type as a parameter.
enum DebugStepResult<Error> {
    /// The debugger reached a new location and the execution is paused at it.
    Paused(SourceLocation),
    /// The debuger reached the end of the program and finished execution.
    Finished,
    /// The debugger reached an error and cannot continue.
    Error(Error),
}

pub struct TracingContext<'a, B: BlackBoxFunctionSolver<FieldElement>> {
    debug_context: DebugContext<'a, B>,
    /// The source location at the current moment of tracing.
    source_location: SourceLocation,
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

        Self { debug_context, source_location: SourceLocation::create_unknown() }
    }

    /// Extracts the current source location from the debugger, given that the relevant debugging
    /// information is present. In the context of this method, a source location is a path to a
    /// source file and a line in that file. The most recently called function is last in the
    /// returned vector/stack.
    ///
    /// Otherwise, returns a &str containing a warning message about the missing data.
    fn get_current_source_location(&self) -> Result<SourceLocation, String> {
        let call_stack = self.debug_context.get_call_stack();
        let opcode_location = match call_stack.last() {
            Some(location) => location,
            None => {
                return Err(String::from("Warning: no call stack"));
            }
        };

        let locations = self.debug_context.get_source_location_for_opcode_location(opcode_location);
        let source_location = match locations.last() {
            Some(location) => location,
            None => {
                return Err(String::from("Warning: no source location mapped to opcode"));
            }
        };

        let filepath = match self.debug_context.get_filepath_for_location(*source_location) {
            Ok(filepath) => filepath,
            Err(error) => {
                return Err(format!(
                    "Warning: could not get filepath for source location: {error}"
                ));
            }
        };

        let line_number = match self.debug_context.get_line_for_location(*source_location) {
            Ok(line) => line as isize + 1,
            Err(error) => {
                return Err(format!("Warning: could not get line for source location: {error}"));
            }
        };

        Ok(SourceLocation { filepath, line_number })
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

            let source_location = match self.get_current_source_location() {
                Ok(pair) => pair,
                Err(warning) => {
                    println!("{warning}");
                    continue;
                }
            };

            if self.source_location == source_location {
                // Continue stepping until a new line in the same file is reached, or the current file
                // has changed.
                // TODO(coda-bug/r916): a function call could result in an extra step
                continue;
            }

            return DebugStepResult::Paused(source_location);
        }
    }

    /// Propagates information about the current execution state to `tracer`.
    fn update_record(&mut self, tracer: &mut Tracer, source_location: &SourceLocation) {
        tracer.register_step(
            &PathBuf::from(source_location.filepath.to_string()),
            Line(source_location.line_number as i64),
        );
    }
}

pub fn trace_circuit<B: BlackBoxFunctionSolver<FieldElement>>(
    blackbox_solver: &B,
    circuit: &Circuit<FieldElement>,
    debug_artifact: &DebugArtifact,
    initial_witness: WitnessMap<FieldElement>,
    unconstrained_functions: &[BrilligBytecode<FieldElement>],
    tracer: &mut Tracer,
) -> Result<(), NargoError<FieldElement>> {
    let mut tracing_context = TracingContext::new(
        blackbox_solver,
        circuit,
        debug_artifact,
        initial_witness,
        unconstrained_functions,
    );

    if tracing_context.debug_context.get_current_opcode_location().is_none() {
        println!("Warning: circuit contains no opcodes; generating no trace");
        return Ok(());
    }

    tracer.start(&PathBuf::from(""), Line(-1));
    loop {
        let source_location = match tracing_context.step_debugger() {
            DebugStepResult::Finished => break,
            DebugStepResult::Error(err) => {
                println!("Error: {err}");
                break;
            }
            DebugStepResult::Paused(source_location) => source_location,
        };

        tracing_context.update_record(tracer, &source_location);

        // This update is intentionally explicit here, to show what drives the loop.
        tracing_context.source_location = source_location;
    }

    Ok(())
}
