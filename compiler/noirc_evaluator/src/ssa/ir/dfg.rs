use crate::ssa::{function_builder::data_bus::DataBus, ir::instruction::SimplifyResult};

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::{
        insert_result::InsertInstructionResult, Instruction, InstructionId, InstructionResultType,
        TerminatorInstruction,
    },
    map::{DenseMap, ForeignFunctions},
    types::{NumericType, Type},
    value::{ForeignFunctionId, Value},
};

use acvm::{acir::AcirField, FieldElement};
use fxhash::FxHashMap as HashMap;
use noirc_errors::Location;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

/// The DataFlowGraph contains most of the actual data in a function including
/// its blocks, instructions, and values. This struct is largely responsible for
/// owning most data in a function and handing out Ids to this data that can be
/// shared without worrying about ownership.
#[serde_as]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub(crate) struct DataFlowGraph {
    /// All of the instructions in a function
    instructions: DenseMap<Instruction>,

    /// Contains each foreign function that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given foreign function is always
    /// represented by only 1 ValueId within this function.
    #[serde(skip)]
    foreign_functions: ForeignFunctions,

    /// All blocks in a function
    blocks: DenseMap<BasicBlock>,

    /// Debugging information about which Values are substituted for another.
    #[serde(skip)]
    replaced_values: HashMap<Value, Value>,

    /// Source location of each instruction for debugging and issuing errors.
    ///
    /// The `CallStack` here corresponds to the entire callstack of locations. Initially this
    /// only contains the actual location of the instruction. During inlining, a new location
    /// will be pushed to each instruction for the location of the function call of the function
    /// the instruction was originally located in. Once inlining is complete, the locations Vec
    /// here should contain the entire callstack for each instruction.
    ///
    /// Instructions inserted by internal SSA passes that don't correspond to user code
    /// may not have a corresponding location.
    #[serde(skip)]
    locations: HashMap<InstructionId, CallStack>,

    #[serde(skip)]
    pub(crate) data_bus: DataBus,
}

pub(crate) type CallStack = super::list::List<Location>;

impl DataFlowGraph {
    /// Creates a new basic block with no parameters.
    /// After being created, the block is unreachable in the current function
    /// until another block is made to jump to it.
    pub(crate) fn make_block(&mut self) -> BasicBlockId {
        self.blocks.insert(BasicBlock::new())
    }

    /// Create a new block with the same parameter count and parameter
    /// types from the given block.
    /// This is a somewhat niche operation used in loop unrolling but is included
    /// here as doing it outside the DataFlowGraph would require cloning the parameters.
    pub(crate) fn make_block_with_parameters_from_block(
        &mut self,
        block: BasicBlockId,
    ) -> BasicBlockId {
        let new_block = self.make_block();
        let parameter_types = self.blocks[block].parameter_types().to_vec();
        self.blocks[new_block].set_parameters(parameter_types);
        new_block
    }

    /// Get an iterator over references to each basic block within the dfg, paired with the basic
    /// block's id.
    ///
    /// The pairs are order by id, which is not guaranteed to be meaningful.
    pub(crate) fn basic_blocks_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = (BasicBlockId, &BasicBlock)> {
        self.blocks.iter()
    }

    /// Iterate over the parameters of a block
    pub(crate) fn block_parameters(
        &self,
        block: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = Value> {
        (0..self[block].parameter_types().len())
            .map(move |position| Value::Param { block, position })
    }

    /// Inserts a new instruction into the DFG.
    /// This does not add the instruction to the block.
    /// Returns the id of the new instruction.
    pub(crate) fn make_instruction(&mut self, instruction_data: Instruction) -> InstructionId {
        self.instructions.insert(instruction_data)
    }

    /// Inserts a new instruction at the end of the given block and returns its results
    pub(crate) fn insert_instruction_and_results(
        &mut self,
        instruction: Instruction,
        block: BasicBlockId,
        call_stack: CallStack,
    ) -> InsertInstructionResult {
        match instruction.simplify(self, block, &call_stack) {
            SimplifyResult::SimplifiedTo(simplification) => {
                InsertInstructionResult::SimplifiedTo(simplification)
            }
            SimplifyResult::SimplifiedToMultiple(simplification) => {
                InsertInstructionResult::SimplifiedToMultiple(simplification)
            }
            SimplifyResult::Remove => InsertInstructionResult::InstructionRemoved,
            SimplifyResult::SimplifiedToInstructionMultiple(instructions) => {
                if instructions.len() > 1 {
                    // There's currently no way to pass results from one instruction in `instructions` on to the next.
                    // We then restrict this to only support multiple instructions if they're all `Instruction::Constrain`
                    // as this instruction type does not have any results.
                    assert!(
                        instructions.iter().all(|instruction| matches!(instruction, Instruction::Constrain(..))),
                        "`SimplifyResult::SimplifiedToInstructionMultiple` only supports `Constrain` instructions"
                    );
                }

                let mut last_id = None;
                let mut last_count = 0;

                for instruction in instructions {
                    last_count = instruction.result_count();
                    let id = self.make_instruction(instruction);
                    self.blocks[block].insert_instruction(id);
                    self.locations.insert(id, call_stack.clone());
                    last_id = Some(id);
                }

                let id = last_id.expect("There should be at least 1 simplified instruction");
                InsertInstructionResult::Results { id, result_count: last_count }
            }
            result @ (SimplifyResult::SimplifiedToInstruction(_) | SimplifyResult::None) => {
                let instruction = result.instruction().unwrap_or(instruction);
                let result_count = instruction.result_count();

                let id = self.make_instruction(instruction);
                self.blocks[block].insert_instruction(id);
                self.locations.insert(id, call_stack.clone());

                InsertInstructionResult::Results { id, result_count }
            }
        }
    }

    /// Set the value of value_to_replace to refer to the value referred to by new_value.
    ///
    /// This is the preferred method to call for optimizations simplifying
    /// values since other instructions referring to the same ValueId need
    /// not be modified to refer to a new ValueId.
    pub(crate) fn replace_value(&mut self, value_to_replace: Value, new_value: Value) {
        if value_to_replace != new_value {
            self.replaced_values.insert(value_to_replace, self.resolve(new_value));
        }
    }

    /// Set the type of value_id to the target_type.
    pub(crate) fn set_type_of_value(&mut self, value: Value, target_type: Type) {
        match value {
            Value::Instruction { instruction, position } => {
                match &mut self.instructions[instruction] {
                    Instruction::Call { result_types, .. } => {
                        result_types[position] = target_type;
                    }
                    Instruction::Load { result_type, .. }
                    | Instruction::ArrayGet { result_type, .. } => {
                        *result_type = target_type;
                    }

                    Instruction::Cast(_, result_type) => {
                        *result_type = target_type.unwrap_numeric();
                    }

                    instruction @ (Instruction::Binary(_)
                    | Instruction::Not(_)
                    | Instruction::Truncate { .. }
                    | Instruction::Allocate { .. }
                    | Instruction::Store { .. }
                    | Instruction::ArraySet { .. }
                    | Instruction::IfElse { .. }
                    | Instruction::MakeArray { .. }) => {
                        panic!("Can't set the type of {instruction:?}")
                    }

                    Instruction::EnableSideEffectsIf { .. }
                    | Instruction::Constrain(..)
                    | Instruction::RangeCheck { .. }
                    | Instruction::IncrementRc { .. }
                    | Instruction::DecrementRc { .. } => {
                        unreachable!("These instructions have no results")
                    }
                }
            }
            Value::Param { block, position } => {
                self.blocks[block].parameter_types_mut()[position] = target_type;
            }
            value => unreachable!("ICE: Cannot set type of {:?}", value),
        }
    }

    /// If `original_value_id`'s underlying `Value` has been substituted for that of another
    /// `ValueId`, this function will return the `ValueId` from which the substitution was taken.
    /// If `original_value_id`'s underlying `Value` has not been substituted, the same `ValueId`
    /// is returned.
    pub(crate) fn resolve(&self, original_value_id: Value) -> Value {
        match self.replaced_values.get(&original_value_id) {
            Some(id) => self.resolve(*id),
            None => original_value_id,
        }
    }

    /// Gets or creates a ValueId for the given FunctionId.
    pub(crate) fn import_foreign_function(&mut self, function: &str) -> Value {
        Value::ForeignFunction(self.foreign_functions.get_or_insert(function))
    }

    /// Returns the type of a given value
    pub(crate) fn type_of_value(&self, value: Value) -> Type {
        match value {
            Value::Instruction { instruction, position } => {
                match self[instruction].result_type() {
                    // How expensive is this recursive call? Maybe we should store types
                    InstructionResultType::Operand(value) => self.type_of_value(value),
                    InstructionResultType::Known(typ) => typ,
                    InstructionResultType::None => unreachable!("Instruction has no results"),
                    InstructionResultType::Multiple(types) => types[position].clone(),
                }
            }
            Value::Param { block, position } => self[block].type_of_parameter(position).clone(),
            Value::NumericConstant { typ, .. } => Type::Numeric(typ),
            Value::Function(_) => Type::Function,
            Value::Intrinsic(_) => Type::Function,
            Value::ForeignFunction(_) => Type::Function,
        }
    }

    /// Returns the maximum possible number of bits that `value` can potentially be.
    ///
    /// Should `value` be a numeric constant then this function will return the exact number of bits required,
    /// otherwise it will return the minimum number of bits based on type information.
    pub(crate) fn get_value_max_num_bits(&self, value: Value) -> u32 {
        match value {
            Value::Instruction { instruction, .. } => {
                if let Instruction::Cast(original_value, _) = self[instruction] {
                    self.type_of_value(original_value).bit_size()
                } else {
                    self.type_of_value(value).bit_size()
                }
            }

            Value::NumericConstant { constant, .. } => constant.num_bits(),
            _ => self.type_of_value(value).bit_size(),
        }
    }

    /// True if the type of this value is Type::Reference.
    /// Using this method over type_of_value avoids cloning the value's type.
    pub(crate) fn value_is_reference(&self, value: Value) -> bool {
        matches!(self.type_of_value(value), Type::Reference(_))
    }

    /// Returns all of result values which are attached to this instruction.
    pub(crate) fn instruction_results(
        &self,
        instruction: InstructionId,
    ) -> impl ExactSizeIterator<Item = Value> {
        let result_count = self[instruction].result_count();
        (0..result_count).map(move |position| Value::Instruction { instruction, position })
    }

    /// Add a parameter to the given block
    pub(crate) fn add_block_parameter(&mut self, block_id: BasicBlockId, typ: Type) -> Value {
        let block = &mut self.blocks[block_id];
        let position = block.parameter_types().len();
        block.add_parameter(typ);
        Value::Param { block: block_id, position }
    }

    /// Returns the field element represented by this value if it is a numeric constant.
    /// Returns None if the given value is not a numeric constant.
    pub(crate) fn get_numeric_constant(&self, value: Value) -> Option<FieldElement> {
        self.get_numeric_constant_with_type(value).map(|(value, _typ)| value)
    }

    /// Returns the field element and type represented by this value if it is a numeric constant.
    /// Returns None if the given value is not a numeric constant.
    pub(crate) fn get_numeric_constant_with_type(
        &self,
        value: Value,
    ) -> Option<(FieldElement, NumericType)> {
        match self.resolve(value) {
            Value::NumericConstant { constant, typ } => Some((constant, typ)),
            _ => None,
        }
    }

    /// Returns the Value::Array associated with this ValueId if it refers to an array constant.
    /// Otherwise, this returns None.
    pub(crate) fn get_array_constant(&self, value: Value) -> Option<(im::Vector<Value>, Type)> {
        match self.resolve(value) {
            Value::Instruction { instruction, .. } => match &self.instructions[instruction] {
                Instruction::MakeArray { elements, typ } => Some((elements.clone(), typ.clone())),
                _ => None,
            },
            // Arrays are shared, so cloning them is cheap
            _ => None,
        }
    }

    /// If this value is an array, return the length of the array as indicated by its type.
    /// Otherwise, return None.
    pub(crate) fn try_get_array_length(&self, value: Value) -> Option<u32> {
        match self.type_of_value(value) {
            Type::Array(_, length) => Some(length),
            _ => None,
        }
    }

    /// A constant index less than the array length is safe
    pub(crate) fn is_safe_index(&self, index: Value, array: Value) -> bool {
        #[allow(clippy::match_like_matches_macro)]
        match (self.type_of_value(array), self.get_numeric_constant(index)) {
            (Type::Array(_, len), Some(index)) if index.to_u128() < (len as u128) => true,
            _ => false,
        }
    }

    /// Sets the terminator instruction for the given basic block
    pub(crate) fn set_block_terminator(
        &mut self,
        block: BasicBlockId,
        terminator: TerminatorInstruction,
    ) {
        self.blocks[block].set_terminator(terminator);
    }

    /// Moves the entirety of the given block's contents into the destination block.
    /// The source block afterward will be left in a valid but emptied state. The
    /// destination block will also have its terminator overwritten with that of the
    /// source block.
    pub(crate) fn inline_block(&mut self, source: BasicBlockId, destination: BasicBlockId) {
        let source = &mut self.blocks[source];
        let mut instructions = source.take_instructions();
        let terminator = source.take_terminator();

        let destination = &mut self.blocks[destination];
        destination.instructions_mut().append(&mut instructions);
        destination.set_terminator(terminator);
    }

    pub(crate) fn get_call_stack(&self, instruction: InstructionId) -> CallStack {
        self.locations.get(&instruction).cloned().unwrap_or_default()
    }

    pub(crate) fn add_location(&mut self, instruction: InstructionId, location: Location) {
        self.locations.entry(instruction).or_default().push_back(location);
    }

    pub(crate) fn get_value_call_stack(&self, value: Value) -> CallStack {
        match self.resolve(value) {
            Value::Instruction { instruction, .. } => self.get_call_stack(instruction),
            _ => CallStack::new(),
        }
    }

    /// True if the given ValueId refers to a (recursively) constant value
    pub(crate) fn is_constant(&self, argument: Value) -> bool {
        match self.resolve(argument) {
            Value::Param { .. } => false,
            Value::Instruction { instruction, .. } => match &self[instruction] {
                Instruction::MakeArray { elements, .. } => {
                    elements.iter().all(|element| self.is_constant(*element))
                }
                _ => false,
            },
            _ => true,
        }
    }

    /// True that the input is a non-zero `Value::NumericConstant`
    pub(crate) fn is_constant_true(&self, argument: Value) -> bool {
        if let Some(constant) = self.get_numeric_constant(argument) {
            !constant.is_zero()
        } else {
            false
        }
    }
}

impl std::ops::Index<InstructionId> for DataFlowGraph {
    type Output = Instruction;
    fn index(&self, id: InstructionId) -> &Self::Output {
        &self.instructions[id]
    }
}

impl std::ops::IndexMut<InstructionId> for DataFlowGraph {
    fn index_mut(&mut self, id: InstructionId) -> &mut Self::Output {
        &mut self.instructions[id]
    }
}

impl std::ops::Index<BasicBlockId> for DataFlowGraph {
    type Output = BasicBlock;
    fn index(&self, id: BasicBlockId) -> &Self::Output {
        &self.blocks[id]
    }
}

impl std::ops::IndexMut<BasicBlockId> for DataFlowGraph {
    /// Get a mutable reference to a function's basic block for the given id.
    fn index_mut(&mut self, id: BasicBlockId) -> &mut Self::Output {
        &mut self.blocks[id]
    }
}

impl std::ops::Index<ForeignFunctionId> for DataFlowGraph {
    type Output = String;

    fn index(&self, id: ForeignFunctionId) -> &Self::Output {
        &self.foreign_functions[id]
    }
}

#[cfg(test)]
mod tests {
    use super::DataFlowGraph;
    use crate::ssa::ir::{instruction::Instruction, types::Type};

    #[test]
    fn make_instruction() {
        let mut dfg = DataFlowGraph::default();
        let ins = Instruction::Allocate { element_type: Type::field() };
        let ins_id = dfg.make_instruction(ins);

        let results = dfg.instruction_results(ins_id);
        assert_eq!(results.len(), 1);
    }
}
