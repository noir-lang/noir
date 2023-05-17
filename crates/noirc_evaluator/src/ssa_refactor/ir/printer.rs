//! This file is for pretty-printing the SSA IR in a human-readable form for debugging.
use std::{
    collections::HashSet,
    fmt::{Formatter, Result},
};

use iter_extended::vecmap;

use super::{
    basic_block::BasicBlockId,
    function::Function,
    instruction::{Instruction, InstructionId, TerminatorInstruction},
    value::ValueId,
};

/// Helper function for Function's Display impl to pretty-print the function with the given formatter.
pub(crate) fn display_function(function: &Function, f: &mut Formatter) -> Result {
    writeln!(f, "fn {} {} {{", function.name(), function.id())?;
    display_block_with_successors(function, function.entry_block(), &mut HashSet::new(), f)?;
    write!(f, "}}")
}

/// Displays a block followed by all of its successors recursively.
/// This uses a HashSet to keep track of the visited blocks. Otherwise
/// there would be infinite recursion for any loops in the IR.
pub(crate) fn display_block_with_successors(
    function: &Function,
    block_id: BasicBlockId,
    visited: &mut HashSet<BasicBlockId>,
    f: &mut Formatter,
) -> Result {
    display_block(function, block_id, f)?;
    visited.insert(block_id);

    for successor in function.dfg[block_id].successors() {
        if !visited.contains(&successor) {
            display_block_with_successors(function, successor, visited, f)?;
        }
    }
    Ok(())
}

/// Display a single block. This will not display the block's successors.
pub(crate) fn display_block(
    function: &Function,
    block_id: BasicBlockId,
    f: &mut Formatter,
) -> Result {
    let block = &function.dfg[block_id];

    writeln!(f, "  {}({}):", block_id, value_list_with_types(function, block.parameters()))?;

    for instruction in block.instructions() {
        display_instruction(function, *instruction, f)?;
    }

    display_terminator(function, block.terminator(), f)
}

/// Specialize displaying value ids so that if they refer to a numeric
/// constant or a function we print those directly.
fn value(function: &Function, id: ValueId) -> String {
    use super::value::Value;
    match &function.dfg[id] {
        Value::NumericConstant { constant, typ } => {
            let value = function.dfg[*constant].value();
            format!("{typ} {value}")
        }
        Value::Function(id) => id.to_string(),
        Value::Intrinsic(intrinsic) => intrinsic.to_string(),
        _ => id.to_string(),
    }
}

/// Display each value along with its type. E.g. `v0: Field, v1: u64, v2: u1`
fn value_list_with_types(function: &Function, values: &[ValueId]) -> String {
    vecmap(values, |id| {
        let value = value(function, *id);
        let typ = function.dfg.type_of_value(*id);
        format!("{value}: {typ}")
    })
    .join(", ")
}

/// Display each value separated by a comma
fn value_list(function: &Function, values: &[ValueId]) -> String {
    vecmap(values, |id| value(function, *id)).join(", ")
}

/// Display a terminator instruction
pub(crate) fn display_terminator(
    function: &Function,
    terminator: Option<&TerminatorInstruction>,
    f: &mut Formatter,
) -> Result {
    match terminator {
        Some(TerminatorInstruction::Jmp { destination, arguments }) => {
            writeln!(f, "    jmp {}({})", destination, value_list(function, arguments))
        }
        Some(TerminatorInstruction::JmpIf { condition, then_destination, else_destination }) => {
            writeln!(
                f,
                "    jmpif {} then: {}, else: {}",
                value(function, *condition),
                then_destination,
                else_destination
            )
        }
        Some(TerminatorInstruction::Return { return_values }) => {
            writeln!(f, "    return {}", value_list(function, return_values))
        }
        None => writeln!(f, "    (no terminator instruction)"),
    }
}

/// Display an arbitrary instruction
pub(crate) fn display_instruction(
    function: &Function,
    instruction: InstructionId,
    f: &mut Formatter,
) -> Result {
    // instructions are always indented within a function
    write!(f, "    ")?;

    let results = function.dfg.instruction_results(instruction);
    if !results.is_empty() {
        write!(f, "{} = ", value_list(function, results))?;
    }

    let show = |id| value(function, id);

    match &function.dfg[instruction] {
        Instruction::Binary(binary) => {
            writeln!(f, "{} {}, {}", binary.operator, show(binary.lhs), show(binary.rhs))
        }
        Instruction::Cast(lhs, typ) => writeln!(f, "cast {} as {typ}", show(*lhs)),
        Instruction::Not(rhs) => writeln!(f, "not {}", show(*rhs)),
        Instruction::Truncate { value, bit_size, max_bit_size } => {
            let value = show(*value);
            writeln!(f, "truncate {value} to {bit_size} bits, max_bit_size: {max_bit_size}",)
        }
        Instruction::Constrain(value) => {
            writeln!(f, "constrain {}", show(*value))
        }
        Instruction::Call { func, arguments } => {
            writeln!(f, "call {}({})", show(*func), value_list(function, arguments))
        }
        Instruction::Allocate { size } => writeln!(f, "alloc {size} fields"),
        Instruction::Load { address } => writeln!(f, "load {}", show(*address)),
        Instruction::Store { address, value } => {
            writeln!(f, "store {} at {}", show(*value), show(*address))
        }
    }
}
