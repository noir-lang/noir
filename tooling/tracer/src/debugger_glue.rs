use crate::{SourceLocation, StackFrame};

use acvm::{BlackBoxFunctionSolver, FieldElement};
use nargo::errors::Location;
use noir_debugger::context::DebugContext;

/// Extracts the current stack of source locations from the debugger, given that the relevant
/// debugging information is present. In the context of this method, a source location is a path
/// to a source file and a line in that file. The most recently called function is last in the
/// returned vector/stack.
///
/// If there is no debugging information, an empty vector will be returned.
///
/// If some of the debugging information is missing (no line or filename for a certain frame of
/// the stack), an "unknown location" will be created for that frame. See
/// `SourceLocation::create_unknown`.
pub(crate) fn get_current_source_locations<B: BlackBoxFunctionSolver<FieldElement>>(
    debug_context: &DebugContext<B>,
) -> Vec<SourceLocation> {
    let call_stack = debug_context.get_call_stack();

    let mut result: Vec<SourceLocation> = vec![];
    for opcode_location in call_stack {
        let locations = debug_context.get_source_location_for_debug_location(&opcode_location);
        for location in locations {
            let source_location = convert_debugger_location(debug_context, location);
            result.push(source_location);
        }
    }

    result
}

/// Converts the debugger stack frames into a vector of stack frames that own their data.
pub(crate) fn get_stack_frames<B: BlackBoxFunctionSolver<FieldElement>>(
    debug_context: &DebugContext<B>,
) -> Vec<StackFrame> {
    debug_context
        .get_variables()
        .iter()
        .map(|f| StackFrame { function_name: String::from(f.function_name) })
        .collect()
}

/// Converts a debugger `Location` into a tracer `SourceLocation`.
///
/// In case there is a problem getting the filepath or the line number from the debugger, a
/// `SourceLocation::create_unknown` is used to return an unknown location.
pub fn convert_debugger_location<B: BlackBoxFunctionSolver<FieldElement>>(
    debug_context: &DebugContext<B>,
    location: Location,
) -> SourceLocation {
    let filepath = match debug_context.get_filepath_for_location(location) {
        Ok(filepath) => filepath,
        Err(error) => {
            println!("Warning: could not get filepath for source location: {error}");
            return SourceLocation::create_unknown();
        }
    };

    let line_number = match debug_context.get_line_for_location(location) {
        Ok(line) => line as isize + 1,
        Err(error) => {
            println!("Warning: could not get line for source location: {error}");
            return SourceLocation::create_unknown();
        }
    };
    SourceLocation { filepath, line_number }
}
