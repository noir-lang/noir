//! This file is for pretty-printing the SSA IR in a human-readable form for debugging.
use std::fmt::{Formatter, Result};

use iter_extended::vecmap;

use super::{
    basic_block::BasicBlockId,
    function::Function,
    instruction::{Instruction, InstructionId, TerminatorInstruction},
    value::ValueId,
};

pub(crate) fn display_function(function: &Function, f: &mut Formatter) -> Result {
    writeln!(f, "fn {} {{", function.name)?;
    display_block_with_successors(function, function.entry_block, f)?;
    write!(f, "}}")
}

pub(crate) fn display_block_with_successors(
    function: &Function,
    block_id: BasicBlockId,
    f: &mut Formatter,
) -> Result {
    display_block(function, block_id, f)?;

    for successor in function.dfg[block_id].successors() {
        display_block(function, successor, f)?;
    }
    Ok(())
}

pub(crate) fn display_block(
    function: &Function,
    block_id: BasicBlockId,
    f: &mut Formatter,
) -> Result {
    let block = &function.dfg[block_id];

    writeln!(f, "{}({}):", block_id, value_list(block.parameters()))?;

    for instruction in block.instructions() {
        display_instruction(function, *instruction, f)?;
    }

    display_terminator(block.terminator(), f)
}

fn value_list(values: &[ValueId]) -> String {
    vecmap(values, ToString::to_string).join(", ")
}

pub(crate) fn display_terminator(
    terminator: Option<&TerminatorInstruction>,
    f: &mut Formatter,
) -> Result {
    match terminator {
        Some(TerminatorInstruction::Jmp { destination, arguments }) => {
            writeln!(f, "    jmp {}({})", destination, value_list(arguments))
        }
        Some(TerminatorInstruction::JmpIf {
            condition,
            arguments,
            then_destination,
            else_destination,
        }) => {
            let args = value_list(arguments);
            writeln!(
                f,
                "    jmpif {}({}) then: {}, else: {}",
                condition, args, then_destination, else_destination
            )
        }
        Some(TerminatorInstruction::Return { return_values }) => {
            writeln!(f, "    return {}", value_list(return_values))
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
        write!(f, "{} = ", value_list(results))?;
    }

    match &function.dfg[instruction] {
        Instruction::Binary(binary) => {
            writeln!(f, "{} {}, {}", binary.operator, binary.lhs, binary.rhs)
        }
        Instruction::Cast(value, typ) => writeln!(f, "cast {value} as {typ}"),
        Instruction::Not(value) => writeln!(f, "not {value}"),
        Instruction::Truncate { value, bit_size, max_bit_size } => {
            writeln!(f, "truncate {value} to {bit_size} bits, max_bit_size: {max_bit_size}")
        }
        Instruction::Constrain(value) => {
            writeln!(f, "constrain {value}")
        }
        Instruction::Call { func, arguments } => {
            writeln!(f, "call {func}({})", value_list(arguments))
        }
        Instruction::Intrinsic { func, arguments } => {
            writeln!(f, "intrinsic {func}({})", value_list(arguments))
        }
        Instruction::Allocate { size } => writeln!(f, "alloc {size} fields"),
        Instruction::Load { address } => writeln!(f, "load {address}"),
        Instruction::Store { address, value } => writeln!(f, "store {value} at {address}"),
    }
}
