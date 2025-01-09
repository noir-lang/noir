use crate::{stack_frame::Variable, SourceLocation, StackFrame};

use acvm::acir::AcirField; // necessary, for `to_i128` to work
use acvm::FieldElement;
use noirc_evaluator::debug_trace::{DebugTraceList, SourcePoint};
use noirc_printable_type::{PrintableType, PrintableValue};
use runtime_tracing::{FullValueRecord, Line, Tracer, ValueRecord};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::warn;

/// Stores the trace accumulated in `tracer` in the specified directory. The trace is stored as
/// multiple JSON files.
pub fn store_trace(tracer: Tracer, trace_dir: &str) {
    let trace_path = Path::new(trace_dir).join("trace.json");
    match tracer.store_trace_events(&trace_path) {
        Ok(_) => println!("Saved trace to {:?}", trace_path),
        Err(err) => println!("Warning: tracer failed to store trace events: {err}"),
    }

    let trace_path = Path::new(trace_dir).join("trace_metadata.json");
    match tracer.store_trace_metadata(&trace_path) {
        Ok(_) => println!("Saved trace to {:?}", trace_path),
        Err(err) => println!("Warning: tracer failed to store trace metadata: {err}"),
    }

    let trace_path = Path::new(trace_dir).join("trace_paths.json");
    match tracer.store_trace_paths(&trace_path) {
        Ok(_) => println!("Saved trace to {:?}", trace_path),
        Err(err) => println!("Warning: tracer failed to store trace metadata: {err}"),
    }
}

/// Registers a tracing step to the given `location` in the given `tracer`.
pub(crate) fn register_step(
    tracer: &mut Tracer,
    location: &SourceLocation,
    debug_trace_list: &mut Option<DebugTraceList>,
) {
    let SourceLocation { filepath, line_number } = &location;
    let path = &PathBuf::from(filepath.to_string());
    let line = Line(*line_number as i64);
    tracer.register_step(path, line);
    if let Some(dtl) = debug_trace_list {
        if let Some(deq) = dtl.source_map.get_mut(&SourcePoint {
            file: PathBuf::from_str(&filepath.to_string())
                .unwrap()
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            line_number: *line_number as usize,
        }) {
            if let Some(range) = deq.pop_front() {
                tracer
                    .register_asm(&dtl.list[range.start..=range.end.unwrap_or(dtl.list.len() - 1)]);
            }
        }
    }
}

/// Registers all variables in the given frame for the last registered step. Each time a new step is
/// registered, all of its variables need to be registered too. If no variables are registered for a
/// step, the frontend will not carry over the variables registered for the previous step.
pub(crate) fn register_variables(tracer: &mut Tracer, frame: &StackFrame) {
    for variable in &frame.variables {
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
        PrintableType::Boolean => {
            if let PrintableValue::Field(field_value) = value {
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Bool, "Bool");
                ValueRecord::Bool { b: field_value.to_i128() as i64 == 1, type_id }
            } else {
                panic!(
                    "type-value mismatch: value: {:?} does not match type Bool",
                    value
                )
            }
        }
        PrintableType::Array { length, typ } => {
            if let PrintableValue::Vec { array_elements, is_slice } = value {
                // println!("array elements {array_elements:?} is_slice {is_slice}");
                let element_values: Vec<ValueRecord> = array_elements.iter()
                    .map(|e| register_value(tracer, e, &*typ))
                    .collect();
                let type_name_base = if !is_slice { "Array" } else { "Slice" };
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Seq, &format!("{type_name_base}<{length}, ..>")); // TODO: more precise?
                ValueRecord::Sequence {
                    elements: element_values,
                    type_id
                }
            } else {
                panic!(
                    "type-value mismatch: value: {:?} does not match type Bool",
                    value
                )
            }
        }
        _ => {
            // TODO(stanm): cover all types and remove `warn!`.
            warn!("not implemented yet: type that is not Field: {:?}", typ);
            let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::None, "()");
            ValueRecord::None { type_id }
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
    let args = convert_params_to_args_vec(tracer, frame);
    tracer.register_call(file_id, args);
}

/// Extracts the relevant information from the given `frame` to construct a vector of `ArgRecord`
/// that the `Tracer` interface expects when registering function calls.
fn convert_params_to_args_vec(tracer: &mut Tracer, frame: &StackFrame) -> Vec<FullValueRecord> {
    let mut result = Vec::new();
    for param_index in &frame.function_param_indexes {
        let variable = &frame.variables[*param_index];
        // TODO(stanm): maybe don't duplicate values?
        let value_record = register_value(tracer, &variable.value, &variable.typ);
        result.push(tracer.arg(&variable.name, value_record));
    }
    result
}

/// Register a return statement in the given `tracer`.
///
/// The tracer seems to be keeping context of which function is returning and is not expecting that
/// to be specified.
pub(crate) fn register_return(tracer: &mut Tracer) {
    let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::None, "()");
    tracer.register_return(runtime_tracing::ValueRecord::None { type_id });
}
