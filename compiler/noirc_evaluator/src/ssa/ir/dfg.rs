use crate::ssa::ir::instruction::SimplifyResult;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    instruction::{Instruction, InstructionId, InstructionResultType, TerminatorInstruction},
    map::{DenseMap, TwoWayMap},
    types::Type,
    value::{ArrayOrSlice, ForeignFunctionName, NumericConstant, ValueId, NumericConstantId},
};

use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;
use noirc_errors::Location;

/// The DataFlowGraph contains most of the actual data in a function including
/// its blocks, instructions, and values. This struct is largely responsible for
/// owning most data in a function and handing out Ids to this data that can be
/// shared without worrying about ownership.
#[derive(Debug, Default)]
pub(crate) struct DataFlowGraph {
    /// All of the instructions in a function
    instructions: DenseMap<Instruction>,

    /// Each constant is unique, attempting to insert the same constant
    /// twice will return the same ValueId.
    numeric_constants: TwoWayMap<NumericConstant>,

    /// Contains each foreign function that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given foreign functôn is always
    /// represented by only 1 ValueId within this function.
    foreign_functions: TwoWayMap<ForeignFunctionName>,

    /// Each array that may be used by this IR
    arrays: DenseMap<ArrayOrSlice>,

    /// All blocks in a function
    blocks: DenseMap<BasicBlock>,

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
    locations: HashMap<InstructionId, CallStack>,
}

pub(crate) type CallStack = im::Vector<Location>;

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
        let parameter_types = self.blocks[block].get_parameter_types();
        self.blocks[new_block].set_parameter_types(parameter_types.to_vec());
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

    /// Returns the parameters of the given block
    pub(crate) fn block_parameters(
        &self,
        block: BasicBlockId,
    ) -> impl ExactSizeIterator<Item = ValueId> {
        let parameter_count = self.blocks[block].parameter_count();
        (0..parameter_count).map(move |position| ValueId::Param { block, position })
    }

    /// Inserts a new instruction at the end of the given block and returns its results
    pub(crate) fn insert_instruction_and_results(
        &mut self,
        instruction: Instruction,
        block: BasicBlockId,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStack,
    ) -> InsertInstructionResult {
        use InsertInstructionResult::*;
        match instruction.simplify(self, block, ctrl_typevars.clone()) {
            SimplifyResult::SimplifiedTo(simplification) => SimplifiedTo(simplification),
            SimplifyResult::SimplifiedToMultiple(simplification) => {
                SimplifiedToMultiple(simplification)
            }
            SimplifyResult::Remove => InstructionRemoved,
            result @ (SimplifyResult::SimplifiedToInstruction(_) | SimplifyResult::None) => {
                let instruction = result.instruction().unwrap_or(instruction);
                let result_count = instruction.result_count();
                let id = self.instructions.insert(instruction);
                self.blocks[block].insert_instruction(id);
                self.locations.insert(id, call_stack);
                InsertInstructionResult::Results(id, result_count)
            }
        }
    }

    /// Creates a new constant value, or returns the Id to an existing one if
    /// one already exists.
    pub(crate) fn make_constant(&mut self, value: FieldElement, typ: Type) -> ValueId {
        let constant = NumericConstant { value, typ };
        ValueId::NumericConstant(self.numeric_constants.insert(constant))
    }

    /// Create a new constant array value from the given elements
    pub(crate) fn make_array(&mut self, elements: im::Vector<ValueId>, typ: Type) -> ValueId {
        assert!(matches!(typ, Type::Array(..) | Type::Slice(_)));
        ValueId::Array(self.arrays.insert(ArrayOrSlice { elements, typ }))
    }

    /// Gets or creates a ValueId for the given FunctionId.
    pub(crate) fn import_foreign_function(&mut self, function: String) -> ValueId {
        ValueId::ForeignFunction(self.foreign_functions.insert(function))
    }

    /// Return the result types of this instruction.
    ///
    /// In the case of Load, Call, and Intrinsic, the function's result
    /// type may be unknown. In this case, the given ctrl_typevars are returned instead.
    /// ctrl_typevars is taken in as an Option since it is common to omit them when getting
    /// the type of an instruction that does not require them. Compared to passing an empty Vec,
    /// Option has the benefit of panicking if it is accidentally used for a Call instruction,
    /// rather than silently returning the empty Vec and continuing.
    fn instruction_result_types(
        &self,
        instruction_id: InstructionId,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> Vec<Type> {
        let instruction = &self.instructions[instruction_id];
        match instruction.result_type() {
            InstructionResultType::Known(typ) => vec![typ],
            InstructionResultType::Operand(value) => vec![self.type_of_value(value)],
            InstructionResultType::None => vec![],
            InstructionResultType::Unknown => {
                ctrl_typevars.expect("Control typevars required but not given")
            }
        }
    }

    /// Returns the type of a given value
    pub(crate) fn type_of_value(&self, value: ValueId) -> Type {
        match value {
            ValueId::InstructionResult { instruction, position } => todo!(),
            ValueId::Param { block, position } => {
                self.blocks[block].get_parameter_types()[position as usize].clone()
            }
            ValueId::NumericConstant(constant_id) => {
                self.numeric_constants[constant_id].typ.clone()
            }
            ValueId::Array(array_id) => self.arrays[array_id].typ.clone(),
            ValueId::Function(_) => {
                todo!("type_of_value ValueId::Function")
            }
            ValueId::Intrinsic(_) => {
                todo!("type_of_value ValueId::Intrinsic")
            }
            ValueId::ForeignFunction(_) => {
                todo!("type_of_value ValueId::ForeignFunction")
            }
        }
    }

    /// True if the type of this value is Type::Reference.
    /// Using this method over type_of_value avoids cloning the value's type.
    pub(crate) fn value_is_reference(&self, value: ValueId) -> bool {
        matches!(self.type_of_value(value), Type::Reference)
    }

    /// Returns all of result values which are attached to this instruction.
    pub(crate) fn instruction_results(
        &self,
        instruction: InstructionId,
    ) -> impl ExactSizeIterator<Item = ValueId> {
        let result_count = self[instruction].result_count();
        ValueId::instruction_result_range(instruction, result_count)
    }

    /// Add a parameter to the given block
    pub(crate) fn add_block_parameter(&mut self, block: BasicBlockId, typ: Type) -> ValueId {
        let position = self.blocks[block].parameter_count();
        self.blocks[block].add_parameter(typ);
        ValueId::Param { block, position: position as u32 }
    }

    /// Returns the field element represented by this value if it is a numeric constant.
    /// Returns None if the given value is not a numeric constant.
    pub(crate) fn get_numeric_constant(&self, value: ValueId) -> Option<FieldElement> {
        self.get_numeric_constant_with_type(value).map(|(value, _typ)| value)
    }

    /// Returns the field element and type represented by this value if it is a numeric constant.
    /// Returns None if the given value is not a numeric constant.
    pub(crate) fn get_numeric_constant_with_type(
        &self,
        value: ValueId,
    ) -> Option<(FieldElement, Type)> {
        match value {
            ValueId::NumericConstant(constant_id) => {
                let constant = self.numeric_constants[constant_id];
                Some((constant.value, constant.typ))
            }
            _ => None,
        }
    }

    /// Returns the Value::Array associated with this ValueId if it refers to an array constant.
    /// Otherwise, this returns None.
    pub(crate) fn get_array_constant(&self, value: ValueId) -> Option<(im::Vector<ValueId>, Type)> {
        match value {
            ValueId::Array(array_id) => {
                let array = &self.arrays[array_id];
                // Arrays are shared, so cloning them is cheap
                Some((array.elements.clone(), array.typ.clone()))
            }
            _ => None,
        }
    }

    /// If this value is an array, return the length of the array as indicated by its type.
    /// Otherwise, return None.
    pub(crate) fn try_get_array_length(&self, value: ValueId) -> Option<usize> {
        match self.type_of_value(value) {
            Type::Array(_, length) => Some(length),
            _ => None,
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

    pub(crate) fn get_value_call_stack(&self, value: ValueId) -> CallStack {
        match value {
            ValueId::InstructionResult { instruction, .. } => self.get_call_stack(instruction),
            _ => im::Vector::new(),
        }
    }

    /// True if the given ValueId refers to a constant value
    pub(crate) fn is_constant(&self, argument: ValueId) -> bool {
        !matches!(argument, ValueId::InstructionResult { .. } | ValueId::Param { .. })
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

impl std::ops::Index<NumericConstantId> for DataFlowGraph {
    type Output = NumericConstant;
    fn index(&self, id: NumericConstantId) -> &Self::Output {
        &self.numeric_constants[id]
    }
}

// The result of calling DataFlowGraph::insert_instruction can
// be a list of results or a single ValueId if the instruction was simplified
// to an existing value.
pub(crate) enum InsertInstructionResult {
    /// Results is the standard case containing the instruction id and the number of results of that
    /// instruction. Each result can be constructed either manually via
    /// `ValueId::InstructionResult { instruction, result_index }`, or iterated over via
    /// `ValueId::instruction_result_range(instruction, result_count)`
    Results(InstructionId, u32),
    SimplifiedTo(ValueId),
    SimplifiedToMultiple(Vec<ValueId>),
    InstructionRemoved,
}

impl InsertInstructionResult {
    /// Retrieve the first (and expected to be the only) result.
    pub(crate) fn first(&self) -> ValueId {
        match self {
            InsertInstructionResult::SimplifiedTo(value) => *value,
            InsertInstructionResult::SimplifiedToMultiple(values) => values[0],
            InsertInstructionResult::Results(instruction, result_count) => {
                assert!(*result_count >= 1);
                ValueId::InstructionResult { instruction: *instruction, position: 0 }
            }
            InsertInstructionResult::InstructionRemoved => {
                panic!("Instruction was removed, no results")
            }
        }
    }

    /// Return all the results contained in the internal results array.
    /// This is used for instructions returning multiple results like function calls.
    pub(crate) fn results(self) -> Vec<ValueId> {
        match self {
            InsertInstructionResult::Results(instruction, result_count) => {
                // TODO: Can we avoid collecting into a Vec here?
                ValueId::instruction_result_range(instruction, result_count).collect()
            }
            InsertInstructionResult::SimplifiedTo(result) => vec![result],
            InsertInstructionResult::SimplifiedToMultiple(results) => results,
            InsertInstructionResult::InstructionRemoved => vec![],
        }
    }

    /// Returns the amount of ValueIds contained
    pub(crate) fn len(&self) -> usize {
        match self {
            InsertInstructionResult::SimplifiedTo(_) => 1,
            InsertInstructionResult::SimplifiedToMultiple(results) => results.len(),
            InsertInstructionResult::Results(_, result_count) => *result_count as usize,
            InsertInstructionResult::InstructionRemoved => 0,
        }
    }
}
