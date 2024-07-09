use crate::{SourceLocation, StackFrame};

use runtime_tracing::{Line, Tracer};
use std::path::PathBuf;

/// Registers a tracing step to the given `location` in the given `tracer`.
pub(crate) fn register_step(tracer: &mut Tracer, location: &SourceLocation) {
    let SourceLocation { filepath, line_number } = &location;
    let path = &PathBuf::from(filepath.to_string());
    let line = Line(*line_number as i64);
    tracer.register_step(path, line);
}

/// Registers a call to the given `frame` at the given `location` in the given `tracer`.
///
/// A helper method, that makes it easier to interface with `Tracer`.
pub(crate) fn register_call(tracer: &mut Tracer, location: &SourceLocation, frame: &StackFrame) {
    let SourceLocation { filepath, line_number } = &location;
    let path = &PathBuf::from(filepath.to_string());
    let line = Line(*line_number as i64);
    let file_id = tracer.ensure_function_id(&frame.function_name, path, line);
    tracer.register_call(file_id, vec![]);
}

/// Register a return statement in the given `tracer`.
///
/// The tracer seems to be keeping context of which function is returning and is not expecting that
/// to be specified.
pub(crate) fn register_return(tracer: &mut Tracer) {
    let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::None, "()");
    tracer.register_return(runtime_tracing::ValueRecord::None { type_id });
}
