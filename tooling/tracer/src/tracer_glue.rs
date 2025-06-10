use crate::{SourceLocation, StackFrame, stack_frame::Variable};

use acvm::FieldElement;
use acvm::acir::AcirField; // necessary, for `to_i128` to work
use noirc_printable_type::{PrintableType, PrintableValue};
use runtime_tracing::{EventLogKind, FullValueRecord, Line, Tracer, ValueRecord, TraceEventsFileFormat};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Stores the trace accumulated in `tracer` in the specified directory. The trace is stored as
/// multiple JSON files.
pub fn store_trace(tracer: Tracer, trace_dir: &str, trace_format: TraceEventsFileFormat) {
    let trace_path = Path::new(trace_dir).join(match trace_format {
        TraceEventsFileFormat::Json => "trace.json",
        TraceEventsFileFormat::Binary => "trace.bin",
    });
    let mut storing_errors = false;
    match tracer.store_trace_events(&trace_path, trace_format) {
        Ok(_) => {}
        Err(err) => {
            storing_errors = true;
            println!("Warning: tracer failed to store trace events: {err}")
        }
    }

    let trace_path = Path::new(trace_dir).join("trace_metadata.json");
    match tracer.store_trace_metadata(&trace_path) {
        Ok(_) => {}
        Err(err) => {
            storing_errors = true;
            println!("Warning: tracer failed to store trace metadata: {err}")
        }
    }

    let trace_path = Path::new(trace_dir).join("trace_paths.json");
    match tracer.store_trace_paths(&trace_path) {
        Ok(_) => {}
        Err(err) => {
            storing_errors = true;
            println!("Warning: tracer failed to store trace paths: {err}")
        }
    }
    if !storing_errors {
        println!("Saved trace to {}", trace_dir);
    }
}

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
    for variable in &frame.variables {
        if variable.name != "__debug_return_expr" {
            register_variable(tracer, variable);
        }
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
        PrintableType::SignedInteger { width } => {
            if let PrintableValue::Field(field_value) = value {
                let mut noir_type_name = String::new();
                if let Err(err) = write!(&mut noir_type_name, "i{width}") {
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
                panic!("type-value mismatch: value: {:?} does not match type Bool", value)
            }
        }
        PrintableType::Slice { typ } => {
            if let PrintableValue::Vec { array_elements, is_slice } = value {
                if !is_slice {
                    panic!("value of is_slice: {:?} does not match type Slice", value)
                }
                let element_values: Vec<ValueRecord> =
                    array_elements.iter().map(|e| register_value(tracer, e, typ)).collect();
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Slice, "&[..]");
                ValueRecord::Sequence { elements: element_values, type_id, is_slice: true }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Slice", value)
            }
        }
        PrintableType::Array { length, typ } => {
            if let PrintableValue::Vec { array_elements, is_slice } = value {
                if *is_slice {
                    panic!("value of is_slice: {:?} does not match type Array", value)
                }
                let element_values: Vec<ValueRecord> =
                    array_elements.iter().map(|e| register_value(tracer, e, typ)).collect();
                let type_id = tracer.ensure_type_id(
                    runtime_tracing::TypeKind::Seq,
                    &format!("Array<{length}, ..>"),
                ); // TODO: more precise?
                ValueRecord::Sequence { elements: element_values, type_id, is_slice: false }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Array", value)
            }
        }
        PrintableType::String { length: _ } => {
            if let PrintableValue::String(s) = value {
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::String, "String");
                ValueRecord::String { text: s.clone(), type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type String", value);
            }
        }
        PrintableType::Struct { name, fields } => {
            if let PrintableValue::Struct(struc) = value {
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Struct, name);
                let mut field_values = vec![];
                for (field_name, field_type) in fields {
                    let field_value = struc
                        .get(field_name)
                        .unwrap_or_else(|| panic!("field value missing: {field_name}"));
                    field_values.push(register_value(tracer, field_value, field_type));
                }
                ValueRecord::Struct { field_values, type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Struct", value);
            }
        }
        PrintableType::Unit => {
            let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Raw, "()");
            ValueRecord::Raw { r: "()".to_string(), type_id }
        }
        PrintableType::Tuple { types } => {
            if let PrintableValue::Vec { array_elements, is_slice } = value {
                if *is_slice {
                    panic!("value of is_slice: {:?} does not match type Tuple", value)
                }
                let element_values: Vec<ValueRecord> = array_elements
                    .iter()
                    .zip(types.iter())
                    .map(|e| register_value(tracer, e.0, e.1))
                    .collect();
                let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Tuple, "(..)");
                ValueRecord::Tuple { elements: element_values, type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Tuple", value)
            }
        }
        PrintableType::Reference { typ, mutable } => {
            let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::Ref, "&");
            let v = register_value(tracer, value, typ);
            ValueRecord::Reference { dereferenced: Box::new(v), address: 0, mutable: *mutable, type_id }
        }
        PrintableType::Function { arguments: _, return_type: _, env: _, unconstrained } => {
            let type_name = if *unconstrained { "unconstrained fn" } else { "fn" };
            let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::FunctionKind, type_name);
            ValueRecord::Raw { r: "fn".to_string(), type_id }
        }
        PrintableType::Enum { .. } => {
            // Enums are an unstable, experimental Noir feature.
            // Even when enabled with -Z enums, they don't seem to become visible in the debugger, so we can't
            // implement them, yet. Therefore, this code is unreachable in practice. Once debugger support for enums is
            // added, we need to implement this as well.
            todo!("Tracing support for enums is not yet implemented")
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
        let value_record = register_value(tracer, &variable.value, &variable.typ);
        result.push(tracer.arg(&variable.name, value_record));
    }
    result
}

/// Register a return statement in the given `tracer`.
///
/// The tracer seems to be keeping context of which function is returning and is not expecting that
/// to be specified.
pub(crate) fn register_return(tracer: &mut Tracer, return_value: &Option<Variable>) {
    if let Some(return_value) = return_value {
        let value_record = register_value(tracer, &return_value.value, &return_value.typ);
        tracer.register_return(value_record);
    } else {
        let type_id = tracer.ensure_type_id(runtime_tracing::TypeKind::None, "()");

        tracer.register_return(runtime_tracing::ValueRecord::None { type_id });
    }
}

pub(crate) fn register_print(tracer: &mut Tracer, s: &str) {
    tracer.register_special_event(EventLogKind::Write, s);
}

pub(crate) fn register_error(tracer: &mut Tracer, s: &str) {
    tracer.register_special_event(EventLogKind::Error, s);
}
