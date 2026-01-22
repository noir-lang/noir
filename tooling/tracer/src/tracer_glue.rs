use crate::{SourceLocation, StackFrame, stack_frame::Variable};

use acvm::FieldElement;
use acvm::acir::AcirField; // necessary, for `to_i128` to work
use codetracer_trace_types::{EventLogKind, FullValueRecord, Line, TypeKind, ValueRecord};
use codetracer_trace_writer::{TraceEventsFileFormat, trace_writer::TraceWriter};
use noirc_printable_type::{PrintableType, PrintableValue};
use std::path::{Path, PathBuf};

pub fn begin_trace(
    tracer: &mut dyn TraceWriter,
    trace_dir: &str,
    trace_format: TraceEventsFileFormat,
) {
    let trace_path = Path::new(trace_dir).join(match trace_format {
        TraceEventsFileFormat::Json => "trace.json",
        TraceEventsFileFormat::BinaryV0 | TraceEventsFileFormat::Binary => "trace.bin",
    });
    match TraceWriter::begin_writing_trace_events(tracer, &trace_path) {
        Ok(_) => {}
        Err(err) => {
            panic!("Error: trace writer failed to begin writing trace events: {err}")
        }
    }

    let trace_path = Path::new(trace_dir).join("trace_metadata.json");
    match TraceWriter::begin_writing_trace_metadata(tracer, &trace_path) {
        Ok(_) => {}
        Err(err) => {
            panic!("Error: trace writer failed to begin writing trace metadata: {err}")
        }
    }

    let trace_path = Path::new(trace_dir).join("trace_paths.json");
    match TraceWriter::begin_writing_trace_paths(tracer, &trace_path) {
        Ok(_) => {}
        Err(err) => {
            panic!("Error: trace writer failed to begin writing trace paths: {err}")
        }
    }
}

/// Stores the trace accumulated in `tracer` in the specified directory. The trace is stored as
/// multiple JSON files.
pub fn finish_trace(tracer: &mut dyn TraceWriter, trace_dir: &str) {
    let mut storing_errors = false;
    match TraceWriter::finish_writing_trace_events(tracer) {
        Ok(_) => {}
        Err(err) => {
            storing_errors = true;
            println!("Warning: trace writer failed to finish writing trace events: {err}")
        }
    }

    match TraceWriter::finish_writing_trace_metadata(tracer) {
        Ok(_) => {}
        Err(err) => {
            storing_errors = true;
            println!("Warning: trace writer failed to finish writing trace metadata: {err}")
        }
    }

    match TraceWriter::finish_writing_trace_paths(tracer) {
        Ok(_) => {}
        Err(err) => {
            storing_errors = true;
            println!("Warning: trace writer failed to finish writing trace paths: {err}")
        }
    }
    if !storing_errors {
        println!("Saved trace to {}", trace_dir);
    }
}

/// Registers a tracing step to the given `location` in the given `tracer`.
pub(crate) fn register_step(tracer: &mut dyn TraceWriter, location: &SourceLocation) {
    let SourceLocation { filepath, line_number } = &location;
    let path = &PathBuf::from(filepath.to_string());
    let line = Line(*line_number as i64);
    TraceWriter::register_step(tracer, path, line);
}

/// Registers all variables in the given frame for the last registered step. Each time a new step is
/// registered, all of its variables need to be registered too. If no variables are registered for a
/// step, the frontend will not carry over the variables registered for the previous step.
pub(crate) fn register_variables(tracer: &mut dyn TraceWriter, frame: &StackFrame) {
    for variable in &frame.variables {
        if variable.name != "__debug_return_expr" {
            register_variable(tracer, variable);
        }
    }
}

/// Registers a variable for the last registered step.
///
/// See `register_variables`.
fn register_variable(tracer: &mut dyn TraceWriter, variable: &Variable) {
    let value_record = register_value(tracer, &variable.value, &variable.typ);
    TraceWriter::register_variable_with_full_value(tracer, &variable.name, value_record);
}

/// Registers a value of a given type. Registers the type, if it's the first time it occurs.
fn register_value(
    tracer: &mut dyn TraceWriter,
    value: &PrintableValue<FieldElement>,
    typ: &PrintableType,
) -> ValueRecord {
    if matches!(value, PrintableValue::Other) {
        let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
        let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
        return ValueRecord::None { type_id };
    }

    match typ {
        PrintableType::Field => {
            if let PrintableValue::Field(field_value) = value {
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
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
        PrintableType::UnsignedInteger { .. } => {
            if let PrintableValue::Field(field_value) = value {
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::Int { i: field_value.to_i128() as i64, type_id }
            } else {
                panic!(
                    "type-value mismatch: value: {:?} does not match type UnsignedInteger",
                    value
                )
            }
        }
        PrintableType::SignedInteger { .. } => {
            if let PrintableValue::Field(field_value) = value {
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::Int { i: field_value.to_i128() as i64, type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type SignedInteger", value)
            }
        }
        PrintableType::Boolean => {
            if let PrintableValue::Field(field_value) = value {
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::Bool { b: field_value.to_i128() as i64 == 1, type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Bool", value)
            }
        }
        PrintableType::Vector { typ: element_type } => {
            if let PrintableValue::Vec { array_elements, is_vector } = value {
                if !is_vector {
                    panic!("value of is_slice: {:?} does not match type Slice", value)
                }
                let element_values: Vec<ValueRecord> = array_elements
                    .iter()
                    .map(|e| register_value(tracer, e, element_type))
                    .collect();
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::Sequence { elements: element_values, type_id, is_slice: true }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Slice", value)
            }
        }
        PrintableType::Array { typ: element_type, .. } => {
            if let PrintableValue::Vec { array_elements, is_vector } = value {
                if *is_vector {
                    panic!("value of is_slice: {:?} does not match type Array", value)
                }
                let element_values: Vec<ValueRecord> = array_elements
                    .iter()
                    .map(|e| register_value(tracer, e, element_type))
                    .collect();
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::Sequence { elements: element_values, type_id, is_slice: false }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Array", value)
            }
        }
        PrintableType::String { .. } => {
            if let PrintableValue::String(s) = value {
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::String { text: s.clone(), type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type String", value);
            }
        }
        PrintableType::Struct { fields, .. } => {
            if let PrintableValue::Struct(struc) = value {
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
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
            let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
            let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
            ValueRecord::Raw { r: "()".to_string(), type_id }
        }
        PrintableType::Tuple { types } => {
            if let PrintableValue::Vec { array_elements, is_vector } = value {
                if *is_vector {
                    panic!("value of is_slice: {:?} does not match type Tuple", value)
                }
                let element_values: Vec<ValueRecord> = array_elements
                    .iter()
                    .zip(types.iter())
                    .map(|(v, t)| register_value(tracer, v, t))
                    .collect();
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::Tuple { elements: element_values, type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type Tuple", value)
            }
        }
        PrintableType::Reference { typ: dereferenced_type, mutable } => {
            let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
            let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
            let v = register_value(tracer, value, dereferenced_type);
            ValueRecord::Reference {
                dereferenced: Box::new(v),
                address: 0,
                mutable: *mutable,
                type_id,
            }
        }
        PrintableType::Function { .. } => {
            let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
            let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
            ValueRecord::Raw { r: "fn".to_string(), type_id }
        }
        PrintableType::Enum { .. } => {
            // Enums are an unstable, experimental Noir feature.
            // Even when enabled with -Z enums, they don't seem to become visible in the debugger, so we can't
            // implement them, yet. Therefore, this code is unreachable in practice. Once debugger support for enums is
            // added, we need to implement this as well.
            todo!("Tracing support for enums is not yet implemented")
        }
        PrintableType::FmtString { typ: element_type, .. } => {
            // TODO: Proper handling for FmtString type
            if let PrintableValue::FmtString(msg, printable_values) = value {
                printable_values.iter().for_each(|printable_value| {
                    register_value(tracer, printable_value, element_type);
                });
                let (type_kind, type_name) = printable_type_to_kind_and_name(typ);
                let type_id = TraceWriter::ensure_type_id(tracer, type_kind, &type_name);
                ValueRecord::String { text: msg.clone(), type_id }
            } else {
                panic!("type-value mismatch: value: {:?} does not match type FmtString", value)
            }
        }
    }
}

/// Registers a call to the given `frame` at the given `location` in the given `tracer`.
///
/// A helper method, that makes it easier to interface with `Tracer`.
pub(crate) fn register_call(
    tracer: &mut dyn TraceWriter,
    location: &SourceLocation,
    frame: &StackFrame,
) {
    let SourceLocation { filepath, line_number } = &location;
    let path = &PathBuf::from(filepath.to_string());
    let line = Line(*line_number as i64);
    let file_id = TraceWriter::ensure_function_id(tracer, &frame.function_name, path, line);
    let args = convert_params_to_args_vec(tracer, frame);
    TraceWriter::register_call(tracer, file_id, args);
}

/// Extracts the relevant information from the given `frame` to construct a vector of `ArgRecord`
/// that the `Tracer` interface expects when registering function calls.
fn convert_params_to_args_vec(
    tracer: &mut dyn TraceWriter,
    frame: &StackFrame,
) -> Vec<FullValueRecord> {
    let mut result = Vec::new();
    for param_index in &frame.function_param_indexes {
        let variable = &frame.variables[*param_index];
        let value_record = register_value(tracer, &variable.value, &variable.typ);
        result.push(TraceWriter::arg(tracer, &variable.name, value_record));
    }
    result
}

/// Register a return statement in the given `tracer`.
///
/// The tracer seems to be keeping context of which function is returning and is not expecting that
/// to be specified.
pub(crate) fn register_return(tracer: &mut dyn TraceWriter, return_value: &Option<Variable>) {
    if let Some(return_value) = return_value {
        let value_record = register_value(tracer, &return_value.value, &return_value.typ);
        TraceWriter::register_return(tracer, value_record);
    } else {
        let type_id = TraceWriter::ensure_type_id(tracer, TypeKind::None, "()");

        TraceWriter::register_return(tracer, ValueRecord::None { type_id });
    }
}

pub(crate) fn register_print(tracer: &mut dyn TraceWriter, s: &str) {
    TraceWriter::register_special_event(tracer, EventLogKind::Write, s);
}

pub(crate) fn register_error(tracer: &mut dyn TraceWriter, s: &str) {
    TraceWriter::register_special_event(tracer, EventLogKind::Error, s);
}

fn printable_type_to_kind_and_name(
    printable_type: &PrintableType,
) -> (TypeKind, String) {
    match printable_type {
        PrintableType::Field => (TypeKind::Int, "Field".to_string()),
        PrintableType::UnsignedInteger { width } => {
            (TypeKind::Int, format!("u{width}"))
        }
        PrintableType::SignedInteger { width } => {
            (TypeKind::Int, format!("i{width}"))
        }
        PrintableType::Boolean => (TypeKind::Bool, "Bool".to_string()),
        PrintableType::Vector { .. } => (TypeKind::Slice, "&[..]".to_string()),
        PrintableType::Array { length, .. } => {
            (TypeKind::Seq, format!("Array<{length}, ..>"))
        }
        PrintableType::String { .. } => (TypeKind::String, "String".to_string()),
        PrintableType::Struct { name, .. } => (TypeKind::Struct, name.clone()),
        PrintableType::Unit => (TypeKind::Raw, "()".to_string()),
        PrintableType::Tuple { .. } => (TypeKind::Tuple, "(..)".to_string()),
        PrintableType::Reference { .. } => (TypeKind::Ref, "&".to_string()),
        PrintableType::Function { unconstrained, .. } => {
            let type_name = if *unconstrained { "unconstrained fn" } else { "fn" };
            (TypeKind::FunctionKind, type_name.to_string())
        }
        PrintableType::FmtString { .. } => {
            // FmtString is ultimately traced as a regular String
            (TypeKind::String, "String".to_string())
        }
        PrintableType::Enum { .. } => {
            // As in the original code, tracing for enums is not yet implemented.
            todo!("Tracing support for enums is not yet implemented")
        }
    }
}
