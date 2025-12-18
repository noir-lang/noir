//! This file is for pretty-printing the SSA IR in a human-readable form for debugging.
use std::fmt::{Display, Formatter, Result};

use acvm::{FieldElement, acir::AcirField};
use fm::codespan_files;
use im::Vector;
use iter_extended::vecmap;

use crate::ssa::{
    Ssa,
    function_builder::data_bus::DataBus,
    ir::{
        instruction::ArrayOffset,
        types::{NumericType, Type},
    },
};

use super::{
    basic_block::BasicBlockId,
    dfg::DataFlowGraph,
    function::Function,
    instruction::{ConstrainError, Instruction, InstructionId, TerminatorInstruction},
    value::{Value, ValueId},
};

pub struct Printer<'local> {
    ssa: &'local Ssa,
    fm: Option<&'local fm::FileManager>,
}

impl Ssa {
    pub fn print_without_locations(&self) -> Printer {
        Printer { ssa: self, fm: None }
    }

    pub fn print_with<'local>(
        &'local self,
        files: Option<&'local fm::FileManager>,
    ) -> Printer<'local> {
        Printer { ssa: self, fm: files }
    }
}

impl Display for Printer<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let globals = (*self.ssa.functions[&self.ssa.main_id].dfg.globals).clone();
        let globals_dfg = DataFlowGraph::from(globals);

        for (id, global_value) in globals_dfg.values_iter() {
            match global_value {
                Value::NumericConstant { constant, typ } => {
                    writeln!(f, "g{} = {typ} {}", id.to_u32(), number(*constant, typ))?;
                }
                Value::Instruction { instruction, .. } => {
                    display_instruction(&globals_dfg, *instruction, true, self.fm, f)?;
                }
                Value::Global(_) => {
                    panic!("Value::Global should only be in the function dfg");
                }
                Value::Function(id) => {
                    writeln!(f, "{id}")?;
                }
                _ => panic!("Expected only numeric constant or instruction"),
            };
        }

        if globals_dfg.values_iter().next().is_some() {
            writeln!(f)?;
        }

        for function in self.ssa.functions.values() {
            display_function(function, self.fm, f)?;
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        display_function(self, None, f)
    }
}

/// Helper function for Function's Display impl to pretty-print the function with the given formatter.
fn display_function(
    function: &Function,
    files: Option<&fm::FileManager>,
    f: &mut Formatter,
) -> Result {
    if let Some(purity) = function.dfg.purity_of(function.id()) {
        writeln!(f, "{} {purity} fn {} {} {{", function.runtime(), function.name(), function.id())?;
    } else {
        writeln!(f, "{} fn {} {} {{", function.runtime(), function.name(), function.id())?;
    }

    display_databus(&function.dfg.data_bus, &function.dfg, f)?;

    for block_id in function.reachable_blocks() {
        display_block(&function.dfg, block_id, files, f)?;
    }
    write!(f, "}}")
}

fn display_databus(data_bus: &DataBus, dfg: &DataFlowGraph, f: &mut Formatter) -> Result {
    for call_data in &data_bus.call_data {
        write!(
            f,
            "  call_data({}): array: {}, indices: [",
            call_data.call_data_id,
            value(dfg, call_data.array_id),
        )?;
        for (i, (value_id, index)) in call_data.index_map.iter().enumerate() {
            if i != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", value(dfg, *value_id), index)?;
        }
        writeln!(f, "]")?;
    }
    if let Some(return_data) = data_bus.return_data {
        writeln!(f, "  return_data: {}", value(dfg, return_data))?;
    }
    Ok(())
}

/// Display a single block. This will not display the block's successors.
fn display_block(
    dfg: &DataFlowGraph,
    block_id: BasicBlockId,
    fm: Option<&fm::FileManager>,
    f: &mut Formatter,
) -> Result {
    let block = &dfg[block_id];

    writeln!(f, "  {}({}):", block_id, value_vector_with_types(dfg, block.parameters()))?;

    for instruction in block.instructions() {
        display_instruction(dfg, *instruction, false, fm, f)?;
    }

    display_terminator(dfg, block.terminator(), f)
}

/// Specialize displaying value ids so that if they refer to a numeric
/// constant or a function we print those directly.
fn value(dfg: &DataFlowGraph, id: ValueId) -> String {
    match &dfg[id] {
        Value::NumericConstant { constant, typ } => {
            format!("{typ} {}", number(*constant, typ))
        }
        Value::Function(id) => id.to_string(),
        Value::Intrinsic(intrinsic) => intrinsic.to_string(),
        Value::ForeignFunction(function) => function.clone(),
        Value::Param { .. } | Value::Instruction { .. } => {
            if dfg.is_global(id) {
                format!("g{}", id.to_u32())
            } else {
                id.to_string()
            }
        }
        Value::Global(_) => {
            format!("g{}", id.to_u32())
        }
    }
}

/// Formats the given number assuming it has the given type.
/// Unsigned types and field element types will be formatter as-is,
/// while signed types will be formatted as positive or negative numbers
/// depending on where the number falls in the range given by the type's bit size.
fn number(number: FieldElement, typ: &NumericType) -> String {
    if let NumericType::Signed { bit_size } = typ {
        number.to_string_as_signed_integer(*bit_size)
    } else {
        number.to_string()
    }
}

/// Display each value along with its type. E.g. `v0: Field, v1: u64, v2: u1`
fn value_vector_with_types(dfg: &DataFlowGraph, values: &[ValueId]) -> String {
    vecmap(values, |id| {
        let value = value(dfg, *id);
        let typ = dfg.type_of_value(*id);
        format!("{value}: {typ}")
    })
    .join(", ")
}

/// Display each value separated by a comma
fn value_vector(dfg: &DataFlowGraph, values: &[ValueId]) -> String {
    vecmap(values, |id| value(dfg, *id)).join(", ")
}

/// Display a terminator instruction
fn display_terminator(
    dfg: &DataFlowGraph,
    terminator: Option<&TerminatorInstruction>,
    f: &mut Formatter,
) -> Result {
    match terminator {
        Some(TerminatorInstruction::Jmp { destination, arguments, call_stack: _ }) => {
            writeln!(f, "    jmp {}({})", destination, value_vector(dfg, arguments))
        }
        Some(TerminatorInstruction::JmpIf {
            condition,
            then_destination,
            else_destination,
            call_stack: _,
        }) => {
            writeln!(
                f,
                "    jmpif {} then: {}, else: {}",
                value(dfg, *condition),
                then_destination,
                else_destination
            )
        }
        Some(TerminatorInstruction::Return { return_values, .. }) => {
            if return_values.is_empty() {
                writeln!(f, "    return")
            } else {
                writeln!(f, "    return {}", value_vector(dfg, return_values))
            }
        }
        Some(TerminatorInstruction::Unreachable { .. }) => {
            writeln!(f, "    unreachable")
        }
        None => writeln!(f, "    (no terminator instruction)"),
    }
}

/// Display an arbitrary instruction
fn display_instruction(
    dfg: &DataFlowGraph,
    instruction: InstructionId,
    in_global_space: bool,
    fm: Option<&fm::FileManager>,
    f: &mut Formatter,
) -> Result {
    match display_instruction_buffer(dfg, instruction, in_global_space, fm) {
        Ok(string) => write!(f, "{string}"),
        Err(_) => Err(std::fmt::Error),
    }
}

fn display_instruction_buffer(
    dfg: &DataFlowGraph,
    instruction: InstructionId,
    in_global_space: bool,
    fm: Option<&fm::FileManager>,
) -> std::result::Result<String, ()> {
    // Need to write to a Vec<u8> and later convert that to a String so we can
    // count how large it is to add padding for the location comment later.
    let mut buffer: Vec<u8> = Vec::new();
    use std::io::Write;

    if !in_global_space {
        // instructions are always indented within a function
        write!(buffer, "    ").map_err(|_| ())?;
    }

    let results = dfg.instruction_results(instruction);
    if !results.is_empty() {
        let mut value_vector = value_vector(dfg, results);
        if in_global_space {
            value_vector = value_vector.replace('v', "g");
        }
        write!(buffer, "{value_vector} = ").map_err(|_| ())?;
    }

    display_instruction_inner(dfg, &dfg[instruction], results, in_global_space, &mut buffer)
        .map_err(|_| ())?;

    if let Some(fm) = fm {
        write_location_information(dfg, instruction, fm, &mut buffer).map_err(|_| ())?;
    }
    writeln!(buffer).map_err(|_| ())?;
    String::from_utf8(buffer).map_err(|_| ())
}

fn write_location_information(
    dfg: &DataFlowGraph,
    instruction: InstructionId,
    fm: &fm::FileManager,
    buffer: &mut Vec<u8>,
) -> std::io::Result<()> {
    use codespan_files::Files;
    use std::io::Write;
    let call_stack = dfg.get_instruction_call_stack(instruction);

    if let Some(location) = call_stack.last() {
        if let Ok(name) = fm.as_file_map().get_name(location.file) {
            let files = fm.as_file_map();
            let start_index = location.span.start() as usize;

            // Add some padding before the comment
            let arbitrary_padding_size = 50;
            if buffer.len() < arbitrary_padding_size {
                buffer.resize(arbitrary_padding_size, b' ');
            }

            write!(buffer, "\t// {name}")?;

            let Ok(line_index) = files.line_index(location.file, start_index) else {
                return Ok(());
            };

            // Offset index by 1 to get the line number
            write!(buffer, ":{}", line_index + 1)?;

            let Ok(column_number) = files.column_number(location.file, line_index, start_index)
            else {
                return Ok(());
            };
            write!(buffer, ":{column_number}")?;
        }
    }
    Ok(())
}

fn display_instruction_inner(
    dfg: &DataFlowGraph,
    instruction: &Instruction,
    results: &[ValueId],
    in_global_space: bool,
    f: &mut Vec<u8>,
) -> std::io::Result<()> {
    use std::io::Write;
    let show = |id| value(dfg, id);

    match instruction {
        Instruction::Binary(binary) => {
            write!(f, "{}", display_binary(binary, dfg))
        }
        Instruction::Cast(lhs, typ) => write!(f, "cast {} as {typ}", show(*lhs)),
        Instruction::Not(rhs) => write!(f, "not {}", show(*rhs)),
        Instruction::Truncate { value, bit_size, max_bit_size } => {
            let value = show(*value);
            write!(f, "truncate {value} to {bit_size} bits, max_bit_size: {max_bit_size}",)
        }
        Instruction::Constrain(lhs, rhs, error) => {
            write!(f, "constrain {} == {}", show(*lhs), show(*rhs))?;
            if let Some(error) = error { display_constrain_error(dfg, error, f) } else { Ok(()) }
        }
        Instruction::ConstrainNotEqual(lhs, rhs, error) => {
            write!(f, "constrain {} != {}", show(*lhs), show(*rhs))?;
            if let Some(error) = error { display_constrain_error(dfg, error, f) } else { Ok(()) }
        }
        Instruction::Call { func, arguments } => {
            let arguments = value_vector(dfg, arguments);
            write!(f, "call {}({}){}", show(*func), arguments, result_types(dfg, results))
        }
        Instruction::Allocate => {
            write!(f, "allocate{}", result_types(dfg, results))
        }
        Instruction::Load { address } => {
            write!(f, "load {}{}", show(*address), result_types(dfg, results))
        }
        Instruction::Store { address, value } => {
            write!(f, "store {} at {}", show(*value), show(*address))
        }
        Instruction::EnableSideEffectsIf { condition } => {
            write!(f, "enable_side_effects {}", show(*condition))
        }
        Instruction::ArrayGet { array, index } => {
            let offset = dfg.array_offset(*array, *index);
            write!(
                f,
                "array_get {}, index {}{}{}",
                show(*array),
                show(*index),
                display_array_offset(&offset),
                result_types(dfg, results)
            )
        }
        Instruction::ArraySet { array, index, value, mutable } => {
            let offset = dfg.array_offset(*array, *index);
            let array = show(*array);
            let index = show(*index);
            let value = show(*value);
            let mutable = if *mutable { " mut" } else { "" };
            write!(
                f,
                "array_set{} {}, index {}{}, value {}",
                mutable,
                array,
                index,
                display_array_offset(&offset),
                value
            )
        }
        Instruction::IncrementRc { value } => {
            write!(f, "inc_rc {}", show(*value))
        }
        Instruction::DecrementRc { value } => {
            write!(f, "dec_rc {}", show(*value))
        }
        Instruction::RangeCheck { value, max_bit_size, assert_message } => {
            let message =
                assert_message.as_ref().map(|message| format!(", {message:?}")).unwrap_or_default();
            write!(f, "range_check {} to {} bits{}", show(*value), *max_bit_size, message)
        }
        Instruction::IfElse { then_condition, then_value, else_condition, else_value } => {
            let then_condition = show(*then_condition);
            let then_value = show(*then_value);
            let else_condition = show(*else_condition);
            let else_value = show(*else_value);
            write!(
                f,
                "if {then_condition} then {then_value} else (if {else_condition}) {else_value}"
            )
        }
        Instruction::MakeArray { elements, typ } => {
            // If the array is a byte array, we check if all the bytes are printable ascii characters
            // and, if so, we print the array as a string literal (easier to understand).
            // It could happen that the byte array is a random byte sequence that happens to be printable
            // (it didn't come from a string literal) but this still reduces the noise in the output
            // and actually represents the same value.
            let (element_types, is_vector) = match typ {
                Type::Array(types, _) => (types, false),
                Type::Vector(types) => (types, true),
                _ => panic!("Expected array or vector type for MakeArray"),
            };
            if element_types.len() == 1
                && element_types[0] == Type::Numeric(NumericType::Unsigned { bit_size: 8 })
            {
                if let Some(string) = try_byte_array_to_string(elements, dfg) {
                    if is_vector {
                        return write!(f, "make_array &b{string:?}");
                    } else {
                        return write!(f, "make_array b{string:?}");
                    }
                }
            }

            write!(f, "make_array [")?;

            for (i, element) in elements.iter().enumerate() {
                if i != 0 {
                    write!(f, ", ")?;
                }
                let mut value = show(*element);
                if in_global_space {
                    value = value.replace('v', "g");
                }
                write!(f, "{value}")?;
            }

            write!(f, "] : {typ}")
        }
        Instruction::Noop => write!(f, "nop"),
    }
}

fn display_array_offset(offset: &ArrayOffset) -> String {
    match offset {
        ArrayOffset::None => String::new(),
        ArrayOffset::Array | ArrayOffset::Vector => format!(" minus {}", offset.to_u32()),
    }
}

pub(crate) fn display_binary(binary: &super::instruction::Binary, dfg: &DataFlowGraph) -> String {
    format!("{} {}, {}", binary.operator, value(dfg, binary.lhs), value(dfg, binary.rhs))
}

fn try_byte_array_to_string(elements: &Vector<ValueId>, dfg: &DataFlowGraph) -> Option<String> {
    let mut string = String::new();
    for element in elements {
        let element = dfg.get_numeric_constant(*element)?;
        let element = element.try_to_u32()?;
        if element > 0xFF {
            return None;
        }
        let byte: u8 = element as u8;
        if is_printable_byte(byte) {
            string.push(byte as char);
        } else {
            return None;
        }
    }
    Some(string)
}

pub fn is_printable_byte(byte: u8) -> bool {
    const FORM_FEED: u8 = 12; // This is the ASCII code for '\f', which isn't a valid escape sequence in strings
    byte != FORM_FEED
        && (byte.is_ascii_alphanumeric()
            || byte.is_ascii_punctuation()
            || byte.is_ascii_whitespace())
}

fn result_types(dfg: &DataFlowGraph, results: &[ValueId]) -> String {
    let types = vecmap(results, |result| dfg.type_of_value(*result).to_string());
    if types.is_empty() {
        String::new()
    } else if types.len() == 1 {
        format!(" -> {}", types[0])
    } else {
        format!(" -> ({})", types.join(", "))
    }
}

/// Tries to extract a constant string from an error payload.
pub(crate) fn try_to_extract_string_from_error_payload(
    is_string_type: bool,
    values: &[ValueId],
    dfg: &DataFlowGraph,
) -> Option<String> {
    if is_string_type && values.len() == 1 { dfg.get_string(values[0]) } else { None }
}

fn display_constrain_error(
    dfg: &DataFlowGraph,
    error: &ConstrainError,
    f: &mut Vec<u8>,
) -> std::io::Result<()> {
    use std::io::Write;
    match error {
        ConstrainError::StaticString(assert_message_string) => {
            write!(f, ", {assert_message_string:?}")
        }
        ConstrainError::Dynamic(_, is_string, values) => {
            if let Some(constant_string) =
                try_to_extract_string_from_error_payload(*is_string, values, dfg)
            {
                write!(f, ", {constant_string:?}")
            } else {
                write!(f, ", data {}", value_vector(dfg, values))
            }
        }
    }
}
