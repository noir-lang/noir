use log::debug;

use acvm::acir::brillig::Opcode as BrilligOpcode;
use acvm::acir::circuit::{Opcode, Program};

use crate::instructions::AvmInstruction;

/// Extract the Brillig program from its `Program` wrapper.
/// Noir entry point unconstrained functions are compiled to their own list contained
/// as part of a full program. Function calls are then accessed through a function
/// pointer opcode in ACIR that fetches those unconstrained functions from the main list.
/// This function just extracts Brillig bytecode, with the assumption that the
/// 0th unconstrained function in the full `Program` structure.
pub fn extract_brillig_from_acir_program(program: &Program) -> &[BrilligOpcode] {
    assert_eq!(
        program.functions.len(),
        1,
        "An AVM program should have only a single ACIR function with a 'BrilligCall'"
    );
    let main_function = &program.functions[0];
    let opcodes = &main_function.opcodes;
    assert_eq!(
        opcodes.len(),
        1,
        "An AVM program should only have a single `BrilligCall`"
    );
    match opcodes[0] {
        Opcode::BrilligCall { id, .. } => assert_eq!(id, 0, "The ID of the `BrilligCall` must be 0 as we have a single `Brillig` function"),
        _ => panic!("Tried to extract a Brillig program from its ACIR wrapper opcode, but the opcode doesn't contain Brillig!"),
    }
    assert_eq!(
        program.unconstrained_functions.len(),
        1,
        "An AVM program should be contained entirely in only a single `Brillig` function"
    );
    &program.unconstrained_functions[0].bytecode
}

/// Print inputs, outputs, and instructions in a Brillig program
pub fn dbg_print_brillig_program(brillig_bytecode: &[BrilligOpcode]) {
    debug!("Printing Brillig program...");
    for (i, instruction) in brillig_bytecode.iter().enumerate() {
        debug!("\tPC:{0} {1:?}", i, instruction);
    }
}

/// Print each instruction in an AVM program
pub fn dbg_print_avm_program(avm_program: &[AvmInstruction]) {
    debug!("Printing AVM program...");
    for (i, instruction) in avm_program.iter().enumerate() {
        debug!("\tPC:{0}: {1}", i, &instruction.to_string());
    }
}
