use crate::{stack_frame::Variable, SourceLocation, StackFrame};

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
    for debug_location in call_stack {
        let locations = debug_context.get_source_location_for_debug_location(&debug_location);
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
    debug_context.get_variables().iter().map(convert_debugger_stack_frame).collect()
}

fn convert_debugger_stack_frame(
    debugger_stack_frame: &noirc_artifacts::debug::StackFrame<FieldElement>,
) -> StackFrame {
    let function_name = String::from(debugger_stack_frame.function_name);
    let mut variables: Vec<Variable> =
        debugger_stack_frame.variables.iter().map(Variable::from_tuple).collect();
    variables.sort();

    let mut function_param_indexes = Vec::new();
    for param_name in &debugger_stack_frame.function_params {
        // Note(stanm): `mut` in params is put in the name; remove it.
        let stripped_param_name = match param_name.strip_prefix("mut ") {
            Some(stripped_param_name) => stripped_param_name,
            None => param_name,
        };
        match variables.binary_search_by(|var| var.name.as_str().cmp(stripped_param_name)) {
            Err(_) => {
                // This panic causes a crash when tracing zk_dungeon:
                // TODO(BSN-2042): investigate why this happens
                //panic!("param_name {param_name} not found in variables {variables:?}");
                println!("!!!param_name {param_name} not found in variables {variables:?}");
            },
            Ok(index) => function_param_indexes.push(index),
        };
    }
    StackFrame { function_name, function_param_indexes, variables }
}

/// Converts a debugger `Location` into a tracer `SourceLocation`.
///
/// In case there is a problem getting the filepath or the line number from the debugger, a
/// `SourceLocation::create_unknown` is used to return an unknown location.
fn convert_debugger_location<B: BlackBoxFunctionSolver<FieldElement>>(
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
