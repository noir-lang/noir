use std::collections::HashMap;

use super::instructions::Instruction;
use crate::ssa_ref::{basic_blocks::BasicBlock, cfg::BasicBlockId};
use acvm::acir::BlackBoxFunc;
use noirc_errors::Location;

/// Similar to Crane-lift, functions own their
/// basics blocks which in turn own their instructions.
pub struct Function {
    /// Basic blocks associated to a particular function
    basic_blocks: HashMap<BasicBlockId, BasicBlock>,

    /// Maps instructions to source locations
    source_locations: HashMap<Instruction, Location>,

    /// The first basic block in the function
    entry_block: BasicBlockId,
}

/// FunctionId is a reference for a function
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);

/// There are two types of functions in the IR:
/// Normal - These are functions defined and implemented in Noir source code.
/// Builtin - These are functions that may be defined in Noir source code
/// but are not implemented in the source code.
///
/// Builtin categorizes functions which have been marked as `foreign`
/// and `builtin` in the Noir source code. ie functions which have been
/// built into the compiler.
/// TODO: This can be confusing since we have #[builtin(to_bits)]
#[derive(Debug, Copy, Clone)]
pub(crate) enum FunctionKind {
    Normal(FunctionId),
    Builtin(Opcode),
}

/// Opcode here refers to two types of functions:
///
/// BlackBox/LowLevel - These functions are defined in the target machine's
/// Instruction Set -- ACIR.
///
/// Compiler known - These are functions which have a function signature in the
/// Noir source code, however their implementation is in the compiler.
/// The reason for this is due to Noir not being expressiveness enough
/// to implement the particular functionality.
/// For example:
///     -  Noir does not a concept of IO so "printing" needs to
/// be a concept that the compiler needs to explicitly know about.
///     - Noir also does not have full non-determinism, so some functions
/// for now simply cannot be implemented in Noir.
#[derive(Clone, Debug, Hash, Copy, PartialEq, Eq)]
pub(crate) enum Opcode {
    LowLevel(BlackBoxFunc),
    // Compiler known
    //
    /// Converts a field element into bits
    /// TODO: why does ToBits need endian?
    ToBits(Endian),
    /// Converts a field element into an array of integers
    /// of a particular radix
    ToRadix(Endian),
    /// Sorts an array.
    Sort,
}

/// Endian types allowed. This is needed since
/// ToRadix and ToBits require one to know the Endian.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) enum Endian {
    Big,
    Little,
}
