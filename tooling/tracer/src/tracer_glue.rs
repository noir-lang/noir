use crate::{stack_frame::Variable, SourceLocation, StackFrame};

use acvm::acir::AcirField; // necessary, for `to_i128` to work
use acvm::FieldElement;
use noirc_printable_type::{PrintableType, PrintableValue};
use runtime_tracing::{Line, Tracer, ValueRecord};
use std::fmt::Write as _;
use std::path::PathBuf;

/// Registers a tracing step to the given `location` in the given `tracer`.
pub(crate) fn register_step(tracer: &mut Tracer, location: &SourceLocation) {
    let SourceLocation { filepath, line_number } = &location;
    let path = &PathBuf::from(filepath.to_string());
    let line = Line(*line_number as i64);
    tracer.register_step(path, line);
}

/// Registers all variables in the given frame for the last registered step. Each time a new step is
/// registered, all of its variables need to be registered too. If no variables are registered for a
/// step, the frontend will not carry over the variables registered for the previous step.
pub(crate) fn register_variables(tracer: &mut Tracer, frame: &StackFrame) {
    let mut variables = frame.variables.clone();
    variables.sort();
    for variable in &variables {
        register_variable(tracer, variable);
    }
}

/// Registers a variable for the last registered step.
///
/// See `register_variables`.
fn register_variable(tracer: &mut Tracer, variable: &Variable) {
    let value_record = register_value(tracer, &variable.value, &variable.typ);
    tracer.register_variable_with_full_value(&variable.name, value_record);
}

/// Registers a value of a given type. Registers the type, if it's the first time it occurs.
fn register_value(
    tracer: &mut Tracer,
    value: &PrintableValue<FieldElement>,
    typ: &PrintableType,
) -> ValueRecord {
    match typ {
        PrintableType::Field => {
            if let PrintableValue::Field(field_value) = value {
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Int, "Field");
                ValueRecord::Int { i: field_value.to_i128() as i64, type_id }
            } else {
                // Note(stanm): panic here, because this means the compiler frontend is broken, which
                // is not the responsibility of this module. Should not be reachable in integration
                // tests (but reachable in unit tests).
                //
                // The same applies for the other `panic!`s in this function.
                panic!("type-value mismatch: value: {:?} does not match type Field", value)
            }
        }
        PrintableType::UnsignedInteger { width } => {
            if let PrintableValue::Field(field_value) = value {
                let mut noir_type_name = String::new();
                if let Err(err) = write!(&mut noir_type_name, "u{width}") {
                    panic!("failed to generate Noir type name: {err}");
                }
                let type_id =
                    tracer.ensure_type_id(runtime_tracing::TypeKind::Int, &noir_type_name);
                ValueRecord::Int { i: field_value.to_i128() as i64, type_id }
            } else {
                panic!(
                    "type-value mismatch: value: {:?} does not match type UnsignedInteger",
                    value
                )
            }
        }
        _ => {
            // TODO(stanm): cover all types and remove `todo!`.
            todo!("not implemented yet: type that is not Field: {:?}", typ)
        }
    }
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
