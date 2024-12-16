use super::{
    call_stack::CallStackId,
    instruction::{InstructionId, TerminatorInstruction},
    map::Id,
    value::ValueId,
};
use serde::{Deserialize, Serialize};

/// A Basic block is a maximal collection of instructions
/// such that there are only jumps at the end of block
/// and one can only enter the block from the beginning.
///
/// This means that if one instruction is executed in a basic
/// block, then all instructions are executed. ie single-entry single-exit.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub(crate) struct BasicBlock {
    /// Parameters to the basic block.
    parameters: Vec<ValueId>,

    /// Instructions in the basic block.
    instructions: Vec<InstructionId>,

    /// The terminating instruction for the basic block.
    ///
    /// This will be a control flow instruction. This is only
    /// None if the block is still being constructed.
    terminator: Option<TerminatorInstruction>,
}

/// An identifier for a Basic Block.
pub(crate) type BasicBlockId = Id<BasicBlock>;

impl BasicBlock {
    /// Create a new BasicBlock with the given parameters.
    /// Parameters can also be added later via BasicBlock::add_parameter
    pub(crate) fn new() -> Self {
        Self { parameters: Vec::new(), instructions: Vec::new(), terminator: None }
    }

    /// Returns the parameters of this block
    pub(crate) fn parameters(&self) -> &[ValueId] {
        &self.parameters
    }

    /// Removes all the parameters of this block
    pub(crate) fn take_parameters(&mut self) -> Vec<ValueId> {
        std::mem::take(&mut self.parameters)
    }

    /// Adds a parameter to this BasicBlock.
    /// Expects that the ValueId given should refer to a Value::Param
    /// instance with its position equal to self.parameters.len().
    pub(crate) fn add_parameter(&mut self, parameter: ValueId) {
        self.parameters.push(parameter);
    }

    /// Replace this block's current parameters with that of the given Vec.
    /// This does not perform any checks that any previous parameters were unused.
    pub(crate) fn set_parameters(&mut self, parameters: Vec<ValueId>) {
        self.parameters = parameters;
    }

    /// Insert an instruction at the end of this block
    pub(crate) fn insert_instruction(&mut self, instruction: InstructionId) {
        self.instructions.push(instruction);
    }

    /// Retrieve a reference to all instructions in this block.
    pub(crate) fn instructions(&self) -> &[InstructionId] {
        &self.instructions
    }

    /// Retrieve a mutable reference to all instructions in this block.
    pub(crate) fn instructions_mut(&mut self) -> &mut Vec<InstructionId> {
        &mut self.instructions
    }

    /// Take the instructions in this block, replacing it with an empty Vec
    pub(crate) fn take_instructions(&mut self) -> Vec<InstructionId> {
        std::mem::take(&mut self.instructions)
    }

    /// Sets the terminator instruction of this block.
    ///
    /// A properly-constructed block will always terminate with a TerminatorInstruction -
    /// which either jumps to another block or returns from the current function. A block
    /// will only have no terminator if it is still under construction.
    pub(crate) fn set_terminator(&mut self, terminator: TerminatorInstruction) {
        self.terminator = Some(terminator);
    }

    /// Returns the terminator of this block.
    ///
    /// Once this block has finished construction, this is expected to always be Some.
    pub(crate) fn terminator(&self) -> Option<&TerminatorInstruction> {
        self.terminator.as_ref()
    }

    /// Returns the terminator of this block, panics if there is None.
    ///
    /// Once this block has finished construction, this is expected to always be Some.
    pub(crate) fn unwrap_terminator(&self) -> &TerminatorInstruction {
        self.terminator().expect("Expected block to have terminator instruction")
    }

    /// Returns a mutable reference to the terminator of this block.
    ///
    /// Once this block has finished construction, this is expected to always be Some.
    pub(crate) fn unwrap_terminator_mut(&mut self) -> &mut TerminatorInstruction {
        self.terminator.as_mut().expect("Expected block to have terminator instruction")
    }

    /// Take ownership of this block's terminator, replacing it with an empty return terminator
    /// so that no clone is needed.
    ///
    /// It is expected that this function is used as an optimization on blocks that are no longer
    /// reachable or will have their terminator overwritten afterwards. Using this on a reachable
    /// block without setting the terminator afterward will result in the empty return terminator
    /// being kept, which is likely unwanted.
    pub(crate) fn take_terminator(&mut self) -> TerminatorInstruction {
        let terminator = self.terminator.as_mut().expect("Expected block to have a terminator");
        std::mem::replace(
            terminator,
            TerminatorInstruction::Return {
                return_values: Vec::new(),
                call_stack: CallStackId::root(),
            },
        )
    }

    /// Return the jmp arguments, if any, of this block's TerminatorInstruction.
    ///
    /// If this block has no terminator, or a Return terminator this will be empty.
    pub(crate) fn terminator_arguments(&self) -> &[ValueId] {
        match &self.terminator {
            Some(TerminatorInstruction::Jmp { arguments, .. }) => arguments,
            _ => &[],
        }
    }

    /// Iterate over all the successors of the currently block, as determined by
    /// the blocks jumped to in the terminator instruction. If there is no terminator
    /// instruction yet, this will iterate 0 times.
    pub(crate) fn successors(
        &self,
    ) -> impl ExactSizeIterator<Item = BasicBlockId> + DoubleEndedIterator {
        match &self.terminator {
            Some(TerminatorInstruction::Jmp { destination, .. }) => vec![*destination].into_iter(),
            Some(TerminatorInstruction::JmpIf { then_destination, else_destination, .. }) => {
                vec![*then_destination, *else_destination].into_iter()
            }
            Some(TerminatorInstruction::Return { .. }) => vec![].into_iter(),
            None => vec![].into_iter(),
        }
    }
}
