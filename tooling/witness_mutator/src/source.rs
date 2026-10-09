//! Turning an opcode index into a place in the Noir source.
//!
//! A finding is only actionable if the reader can see which line of the program produced the call,
//! so the report resolves the call site through the artifact's debug symbols.

use acir::{
    FieldElement,
    circuit::{AcirOpcodeLocation, Program},
};
use noirc_artifacts::program::CompiledProgram;

/// `path:line` of the innermost source location the opcode came from.
pub fn location_of(
    program: &CompiledProgram,
    function_index: usize,
    opcode_index: usize,
) -> Option<String> {
    let debug = program.debug.get(function_index)?;
    let call_stack_id = *debug.acir_locations.get(&AcirOpcodeLocation::new(opcode_index))?;

    let location = *debug.location_tree.get_call_stack(call_stack_id).last()?;
    let file = program.file_map.get(&location.file)?;
    let line = file.source[..location.span.start() as usize].lines().count().max(1);

    Some(format!("{}:{}", file.path.display(), line))
}

/// How many opcodes an ACIR function has, for walking call sites by index.
pub fn opcode_count(program: &Program<FieldElement>, function_index: usize) -> usize {
    program.functions.get(function_index).map_or(0, |function| function.opcodes.len())
}
