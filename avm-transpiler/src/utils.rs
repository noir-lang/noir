use log::debug;

use acvm::acir::circuit::brillig::Brillig;
use acvm::acir::circuit::Opcode;

use crate::instructions::AvmInstruction;

/// Extract the Brillig program from its ACIR wrapper instruction.
/// An Noir unconstrained function compiles to one ACIR instruction
/// wrapping a Brillig program. This function just extracts that Brillig
/// assuming the 0th ACIR opcode is the wrapper.
pub fn extract_brillig_from_acir(opcodes: &Vec<Opcode>) -> &Brillig {
    if opcodes.len() != 1 {
        panic!("An AVM program should be contained entirely in only a single ACIR opcode flagged as 'Brillig'");
    }
    let opcode = &opcodes[0];
    match opcode {
        Opcode::Brillig(brillig) => brillig,
        _ => panic!("Tried to extract a Brillig program from its ACIR wrapper opcode, but the opcode doesn't contain Brillig!"),
    }
}

/// Print inputs, outputs, and instructions in a Brillig program
pub fn dbg_print_brillig_program(brillig: &Brillig) {
    debug!("Printing Brillig program...");
    debug!("\tInputs: {:?}", brillig.inputs);
    for (i, instruction) in brillig.bytecode.iter().enumerate() {
        debug!("\tPC:{0} {1:?}", i, instruction);
    }
    debug!("\tOutputs: {:?}", brillig.outputs);
}

/// Print each instruction in an AVM program
pub fn dbg_print_avm_program(avm_program: &[AvmInstruction]) {
    debug!("Printing AVM program...");
    for (i, instruction) in avm_program.iter().enumerate() {
        debug!("\tPC:{0}: {1}", i, &instruction.to_string());
    }
}
