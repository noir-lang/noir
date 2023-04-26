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

pub(crate) fn display_function(function: &Function, f: &mut Formatter) -> Result {
    writeln!(f, "fn {} {} {{", function.name(), function.id())?;
    display_block_with_successors(function, function.entry_block(), &mut HashSet::new(), f)?;
    write!(f, "}}")
}

/// Displays a block followed by all of its successors recursively.
/// This uses a HashSet to keep track of the visited blocks. Otherwise,
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

pub(crate) fn display_block(
    function: &Function,
    block_id: BasicBlockId,
    f: &mut Formatter,
) -> Result {
    let block = &function.dfg[block_id];

    writeln!(f, "  {}({}):", block_id, value_list(function, block.parameters()))?;

    for instruction in block.instructions() {
        display_instruction(function, *instruction, f)?;
    }

    display_terminator(function, block.terminator(), f)
}

/// Specialize displaying value ids so that if they refer to constants we
/// print the constant directly
fn value(function: &Function, id: ValueId) -> String {
    match function.dfg.get_numeric_constant_with_type(id) {
        Some((value, typ)) => format!("{} {}", value, typ),
        None => id.to_string(),
    }
}

fn value_list(function: &Function, values: &[ValueId]) -> String {
    vecmap(values, |id| value(function, *id)).join(", ")
}

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
                condition, then_destination, else_destination
            )
        }
        Some(TerminatorInstruction::Return { return_values }) => {
            writeln!(f, "    return {}", value_list(function, return_values))
        }
        None => writeln!(f, "    (no terminator instruction)"),
    }
}

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
            writeln!(f, "call {func}({})", value_list(function, arguments))
        }
        Instruction::Intrinsic { func, arguments } => {
            writeln!(f, "intrinsic {func}({})", value_list(function, arguments))
        }
        Instruction::Allocate { size } => writeln!(f, "alloc {size} fields"),
        Instruction::Load { address } => writeln!(f, "load {}", show(*address)),
        Instruction::Store { address, value } => {
            writeln!(f, "store {} at {}", show(*address), show(*value))
        }
    }
}
