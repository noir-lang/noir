use std::borrow::Cow;

use crate::ssa::{function_builder::data_bus::DataBus, ir::instruction::SimplifyResult};

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    call_stack::{CallStack, CallStackHelper, CallStackId},
    function::{FunctionId, RuntimeType},
    instruction::{
        Instruction, InstructionId, InstructionResultType, Intrinsic, TerminatorInstruction,
    },
    map::DenseMap,
    types::{NumericType, Type},
    value::{Value, ValueId},
};

use acvm::{acir::AcirField, FieldElement};
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use serde_with::DisplayFromStr;

/// The DataFlowGraph contains most of the actual data in a function including
/// its blocks, instructions, and values. This struct is largely responsible for
/// owning most data in a function and handing out Ids to this data that can be
/// shared without worrying about ownership.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct DataFlowGraph {
    /// Runtime of the [Function] that owns this [DataFlowGraph].
    /// This might change during the `runtime_separation` pass where
    /// ACIR functions are cloned as Brillig functions.
    runtime: RuntimeType,

    /// All of the instructions in a function
    instructions: DenseMap<Instruction>,

    /// Stores the results for a particular instruction.
    ///
    /// An instruction may return multiple values
    /// and for this, we will also use the cranelift strategy
    /// to fetch them via indices.
    ///
    /// Currently, we need to define them in a better way
    /// Call instructions require the func signature, but
    /// other instructions may need some more reading on my part
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    results: HashMap<InstructionId, smallvec::SmallVec<[ValueId; 1]>>,

    /// Storage for all of the values defined in this
    /// function.
    values: DenseMap<Value>,

    /// Each constant is unique, attempting to insert the same constant
    /// twice will return the same ValueId.
    #[serde(skip)]
    constants: HashMap<(FieldElement, NumericType), ValueId>,

    /// Contains each function that has been imported into the current function.
    /// A unique `ValueId` for each function's [`Value::Function`] is stored so any given FunctionId
    /// will always have the same ValueId within this function.
    #[serde(skip)]
    functions: HashMap<FunctionId, ValueId>,

    /// Contains each intrinsic that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given intrinsic is always
    /// represented by only 1 ValueId within this function.
    #[serde(skip)]
    intrinsics: HashMap<Intrinsic, ValueId>,

    /// Contains each foreign function that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given foreign function is always
    /// represented by only 1 ValueId within this function.
    #[serde(skip)]
    foreign_functions: HashMap<String, ValueId>,

    /// All blocks in a function
    blocks: DenseMap<BasicBlock>,

    /// Debugging information about which `ValueId`s have had their underlying `Value` substituted
    /// for that of another. In theory this information is purely used for printing the SSA,
    /// and has no material effect on the SSA itself, however in practice the IDs can get out of
    /// sync and may need this resolution before they can be compared.
    #[serde(skip)]
    replaced_value_ids: HashMap<ValueId, ValueId>,

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
    locations: HashMap<InstructionId, CallStackId>,

    pub(crate) call_stack_data: CallStackHelper,

    #[serde(skip)]
    pub(crate) data_bus: DataBus,
}

impl DataFlowGraph {
    /// Runtime type of the function.
    pub(crate) fn runtime(&self) -> RuntimeType {
        self.runtime
    }

    /// Set runtime type of the function.
    pub(crate) fn set_runtime(&mut self, runtime: RuntimeType) {
        self.runtime = runtime;
    }

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
        let parameters = self.blocks[block].parameters();

        let parameters = vecmap(parameters.iter().enumerate(), |(position, param)| {
            let typ = self.values[*param].get_type().into_owned();
            self.values.insert(Value::Param { block: new_block, position, typ })
        });

        self.blocks[new_block].set_parameters(parameters);
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

    /// Iterate over every Value in this DFG in no particular order, including unused Values
    pub(crate) fn values_iter(&self) -> impl ExactSizeIterator<Item = (ValueId, &Value)> {
        self.values.iter()
    }

    /// Returns the parameters of the given block
    pub(crate) fn block_parameters(&self, block: BasicBlockId) -> &[ValueId] {
        self.blocks[block].parameters()
    }

    /// Inserts a new instruction into the DFG.
    /// This does not add the instruction to the block.
    /// Returns the id of the new instruction and its results.
    ///
    /// Populates the instruction's results with the given ctrl_typevars if the instruction
    /// is a Load, Call, or Intrinsic. Otherwise the instruction's results will be known
    /// by the instruction itself and None can safely be passed for this parameter.
    pub(crate) fn make_instruction(
        &mut self,
        instruction_data: Instruction,
        ctrl_typevars: Option<Vec<Type>>,
    ) -> InstructionId {
        let id = self.instructions.insert(instruction_data);
        self.make_instruction_results(id, ctrl_typevars);
        id
    }

    /// Check if the function runtime would simply ignore this instruction.
    pub(crate) fn is_handled_by_runtime(&self, instruction: &Instruction) -> bool {
        !(self.runtime().is_acir() && instruction.is_brillig_only())
    }

    fn insert_instruction_without_simplification(
        &mut self,
        instruction_data: Instruction,
        block: BasicBlockId,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStackId,
    ) -> InstructionId {
        let id = self.make_instruction(instruction_data, ctrl_typevars);
        self.blocks[block].insert_instruction(id);
        self.locations.insert(id, call_stack);
        id
    }

    pub(crate) fn insert_instruction_and_results_without_simplification(
        &mut self,
        instruction_data: Instruction,
        block: BasicBlockId,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStackId,
    ) -> InsertInstructionResult {
        if !self.is_handled_by_runtime(&instruction_data) {
            return InsertInstructionResult::InstructionRemoved;
        }

        let id = self.insert_instruction_without_simplification(
            instruction_data,
            block,
            ctrl_typevars,
            call_stack,
        );

        InsertInstructionResult::Results(id, self.instruction_results(id))
    }

    /// Simplifies a new instruction and inserts it at the end of the given block and returns its results.
    /// If the instruction is not handled by the current runtime, `InstructionRemoved` is returned.
    pub(crate) fn insert_instruction_and_results(
        &mut self,
        instruction: Instruction,
        block: BasicBlockId,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStackId,
    ) -> InsertInstructionResult {
        if !self.is_handled_by_runtime(&instruction) {
            return InsertInstructionResult::InstructionRemoved;
        }

        match instruction.simplify(self, block, ctrl_typevars.clone(), call_stack) {
            SimplifyResult::SimplifiedTo(simplification) => {
                InsertInstructionResult::SimplifiedTo(simplification)
            }
            SimplifyResult::SimplifiedToMultiple(simplification) => {
                InsertInstructionResult::SimplifiedToMultiple(simplification)
            }
            SimplifyResult::Remove => InsertInstructionResult::InstructionRemoved,
            result @ (SimplifyResult::SimplifiedToInstruction(_)
            | SimplifyResult::SimplifiedToInstructionMultiple(_)
            | SimplifyResult::None) => {
                let mut instructions = result.instructions().unwrap_or(vec![instruction]);
                assert!(!instructions.is_empty(), "`SimplifyResult::SimplifiedToInstructionMultiple` must not return empty vector");

                if instructions.len() > 1 {
                    // There's currently no way to pass results from one instruction in `instructions` on to the next.
                    // We then restrict this to only support multiple instructions if they're all `Instruction::Constrain`
                    // as this instruction type does not have any results.
                    assert!(
                        instructions.iter().all(|instruction| matches!(instruction, Instruction::Constrain(..))),
                        "`SimplifyResult::SimplifiedToInstructionMultiple` only supports `Constrain` instructions"
                    );
                }

                // Pull off the last instruction as we want to return its results.
                let last_instruction = instructions.pop().expect("`instructions` can't be empty");
                for instruction in instructions {
                    self.insert_instruction_without_simplification(
                        instruction,
                        block,
                        ctrl_typevars.clone(),
                        call_stack,
                    );
                }
                self.insert_instruction_and_results_without_simplification(
                    last_instruction,
                    block,
                    ctrl_typevars,
                    call_stack,
                )
            }
        }
    }

    /// Set the value of value_to_replace to refer to the value referred to by new_value.
    ///
    /// This is the preferred method to call for optimizations simplifying
    /// values since other instructions referring to the same ValueId need
    /// not be modified to refer to a new ValueId.
    pub(crate) fn set_value_from_id(&mut self, value_to_replace: ValueId, new_value: ValueId) {
        if value_to_replace != new_value {
            self.replaced_value_ids.insert(value_to_replace, self.resolve(new_value));
            let new_value = self.values[new_value].clone();
            self.values[value_to_replace] = new_value;
        }
    }

    /// Set the type of value_id to the target_type.
    pub(crate) fn set_type_of_value(&mut self, value_id: ValueId, target_type: Type) {
        let value = &mut self.values[value_id];
        match value {
            Value::Instruction { typ, .. } | Value::Param { typ, .. } => {
                *typ = target_type;
            }
            Value::NumericConstant { typ, .. } => {
                *typ = target_type.unwrap_numeric();
            }
            _ => {
                unreachable!("ICE: Cannot set type of {:?}", value);
            }
        }
    }

    /// If `original_value_id`'s underlying `Value` has been substituted for that of another
    /// `ValueId`, this function will return the `ValueId` from which the substitution was taken.
    /// If `original_value_id`'s underlying `Value` has not been substituted, the same `ValueId`
    /// is returned.
    pub(crate) fn resolve(&self, original_value_id: ValueId) -> ValueId {
        match self.replaced_value_ids.get(&original_value_id) {
            Some(id) => self.resolve(*id),
            None => original_value_id,
        }
    }

    /// Creates a new constant value, or returns the Id to an existing one if
    /// one already exists.
    pub(crate) fn make_constant(&mut self, constant: FieldElement, typ: NumericType) -> ValueId {
        if let Some(id) = self.constants.get(&(constant, typ)) {
            return *id;
        }
        let id = self.values.insert(Value::NumericConstant { constant, typ });
        self.constants.insert((constant, typ), id);
        id
    }

    /// Gets or creates a ValueId for the given FunctionId.
    pub(crate) fn import_function(&mut self, function: FunctionId) -> ValueId {
        if let Some(existing) = self.functions.get(&function) {
            return *existing;
        }
        self.values.insert(Value::Function(function))
    }

    /// Gets or creates a ValueId for the given FunctionId.
    pub(crate) fn import_foreign_function(&mut self, function: &str) -> ValueId {
        if let Some(existing) = self.foreign_functions.get(function) {
            return *existing;
        }
        self.values.insert(Value::ForeignFunction(function.to_owned()))
    }

    /// Gets or creates a ValueId for the given Intrinsic.
    pub(crate) fn import_intrinsic(&mut self, intrinsic: Intrinsic) -> ValueId {
        if let Some(existing) = self.get_intrinsic(intrinsic) {
            return *existing;
        }
        let intrinsic_value_id = self.values.insert(Value::Intrinsic(intrinsic));
        self.intrinsics.insert(intrinsic, intrinsic_value_id);
        intrinsic_value_id
    }

    pub(crate) fn get_intrinsic(&self, intrinsic: Intrinsic) -> Option<&ValueId> {
        self.intrinsics.get(&intrinsic)
    }

    /// Attaches results to the instruction, clearing any previous results.
    ///
    /// This does not normally need to be called manually as it is called within
    /// make_instruction automatically.
    ///
    /// Returns the results of the instruction
    pub(crate) fn make_instruction_results(
        &mut self,
        instruction: InstructionId,
        ctrl_typevars: Option<Vec<Type>>,
    ) {
        let mut results = smallvec::SmallVec::new();
        let mut position = 0;
        self.for_each_instruction_result_type(instruction, ctrl_typevars, |this, typ| {
            let result = this.values.insert(Value::Instruction { typ, position, instruction });
            position += 1;
            results.push(result);
        });

        self.results.insert(instruction, results);
    }

    /// Return the result types of this instruction.
    ///
    /// In the case of Load, Call, and Intrinsic, the function's result
    /// type may be unknown. In this case, the given ctrl_typevars are returned instead.
    /// ctrl_typevars is taken in as an Option since it is common to omit them when getting
    /// the type of an instruction that does not require them. Compared to passing an empty Vec,
    /// Option has the benefit of panicking if it is accidentally used for a Call instruction,
    /// rather than silently returning the empty Vec and continuing.
    fn for_each_instruction_result_type(
        &mut self,
        instruction_id: InstructionId,
        ctrl_typevars: Option<Vec<Type>>,
        mut f: impl FnMut(&mut Self, Type),
    ) {
        let instruction = &self.instructions[instruction_id];
        match instruction.result_type() {
            InstructionResultType::Known(typ) => f(self, typ),
            InstructionResultType::Operand(value) => f(self, self.type_of_value(value)),
            InstructionResultType::None => (),
            InstructionResultType::Unknown => {
                for typ in ctrl_typevars.expect("Control typevars required but not given") {
                    f(self, typ);
                }
            }
        }
    }

    /// Returns the type of a given value
    pub(crate) fn type_of_value(&self, value: ValueId) -> Type {
        self.values[value].get_type().into_owned()
    }

    /// Returns the maximum possible number of bits that `value` can potentially be.
    ///
    /// Should `value` be a numeric constant then this function will return the exact number of bits required,
    /// otherwise it will return the minimum number of bits based on type information.
    pub(crate) fn get_value_max_num_bits(&self, value: ValueId) -> u32 {
        match self[value] {
            Value::Instruction { instruction, .. } => {
                let value_bit_size = self.type_of_value(value).bit_size();
                if let Instruction::Cast(original_value, _) = self[instruction] {
                    let original_bit_size = self.type_of_value(original_value).bit_size();
                    // We might have cast e.g. `u1` to `u8` to be able to do arithmetic,
                    // in which case we want to recover the original smaller bit size;
                    // OTOH if we cast down, then we don't need the higher original size.
                    value_bit_size.min(original_bit_size)
                } else {
                    value_bit_size
                }
            }

            Value::NumericConstant { constant, .. } => constant.num_bits(),
            _ => self.type_of_value(value).bit_size(),
        }
    }

    /// True if the type of this value is Type::Reference.
    /// Using this method over type_of_value avoids cloning the value's type.
    pub(crate) fn value_is_reference(&self, value: ValueId) -> bool {
        matches!(self.values[value].get_type().as_ref(), Type::Reference(_))
    }

    /// Replaces an instruction result with a fresh id.
    pub(crate) fn replace_result(
        &mut self,
        instruction_id: InstructionId,
        prev_value_id: ValueId,
    ) -> ValueId {
        let typ = self.type_of_value(prev_value_id);
        let results = self.results.get_mut(&instruction_id).unwrap();
        let res_position = results
            .iter()
            .position(|&id| id == prev_value_id)
            .expect("Result id not found while replacing");

        let value_id = self.values.insert(Value::Instruction {
            typ,
            position: res_position,
            instruction: instruction_id,
        });

        // Replace the value in list of results for this instruction
        results[res_position] = value_id;
        value_id
    }

    /// Returns all of result values which are attached to this instruction.
    pub(crate) fn instruction_results(&self, instruction_id: InstructionId) -> &[ValueId] {
        self.results.get(&instruction_id).expect("expected a list of Values").as_slice()
    }

    /// Remove an instruction by replacing it with a `Noop` instruction.
    /// Doing this avoids shifting over each instruction after this one in its block's instructions vector.
    #[allow(unused)]
    pub(crate) fn remove_instruction(&mut self, instruction: InstructionId) {
        self.instructions[instruction] = Instruction::Noop;
        self.results.insert(instruction, smallvec::SmallVec::new());
    }

    /// Add a parameter to the given block
    pub(crate) fn add_block_parameter(&mut self, block_id: BasicBlockId, typ: Type) -> ValueId {
        let block = &mut self.blocks[block_id];
        let position = block.parameters().len();
        let parameter = self.values.insert(Value::Param { block: block_id, position, typ });
        block.add_parameter(parameter);
        parameter
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
    ) -> Option<(FieldElement, NumericType)> {
        match &self.values[self.resolve(value)] {
            Value::NumericConstant { constant, typ } => Some((*constant, *typ)),
            _ => None,
        }
    }

    /// Returns the Value::Array associated with this ValueId if it refers to an array constant.
    /// Otherwise, this returns None.
    pub(crate) fn get_array_constant(&self, value: ValueId) -> Option<(im::Vector<ValueId>, Type)> {
        match &self.values[self.resolve(value)] {
            Value::Instruction { instruction, .. } => match &self.instructions[*instruction] {
                Instruction::MakeArray { elements, typ } => Some((elements.clone(), typ.clone())),
                _ => None,
            },
            // Arrays are shared, so cloning them is cheap
            _ => None,
        }
    }

    /// If this value is an array, return the length of the array as indicated by its type.
    /// Otherwise, return None.
    pub(crate) fn try_get_array_length(&self, value: ValueId) -> Option<u32> {
        match self.type_of_value(value) {
            Type::Array(_, length) => Some(length),
            _ => None,
        }
    }

    /// If this value points to an array of constant bytes, returns a string
    /// consisting of those bytes if they form a valid UTF-8 string.
    pub(crate) fn get_string(&self, value: ValueId) -> Option<String> {
        let (value_ids, _typ) = self.get_array_constant(value)?;

        let mut bytes = Vec::new();
        for value_id in value_ids {
            let field_value = self.get_numeric_constant(value_id)?;
            let u64_value = field_value.try_to_u64()?;
            if u64_value > 255 {
                return None;
            };
            let byte = u64_value as u8;
            bytes.push(byte);
        }
        String::from_utf8(bytes).ok()
    }

    /// A constant index less than the array length is safe
    pub(crate) fn is_safe_index(&self, index: ValueId, array: ValueId) -> bool {
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

    pub(crate) fn get_instruction_call_stack(&self, instruction: InstructionId) -> CallStack {
        let call_stack = self.get_instruction_call_stack_id(instruction);
        self.call_stack_data.get_call_stack(call_stack)
    }

    pub(crate) fn get_instruction_call_stack_id(&self, instruction: InstructionId) -> CallStackId {
        self.locations.get(&instruction).cloned().unwrap_or_default()
    }

    pub(crate) fn get_call_stack(&self, call_stack: CallStackId) -> CallStack {
        self.call_stack_data.get_call_stack(call_stack)
    }

    pub(crate) fn get_value_call_stack(&self, value: ValueId) -> CallStack {
        match &self.values[self.resolve(value)] {
            Value::Instruction { instruction, .. } => self.get_instruction_call_stack(*instruction),
            _ => CallStack::new(),
        }
    }

    pub(crate) fn get_value_call_stack_id(&self, value: ValueId) -> CallStackId {
        match &self.values[self.resolve(value)] {
            Value::Instruction { instruction, .. } => {
                self.get_instruction_call_stack_id(*instruction)
            }
            _ => CallStackId::root(),
        }
    }

    /// True if the given ValueId refers to a (recursively) constant value
    pub(crate) fn is_constant(&self, argument: ValueId) -> bool {
        match &self[self.resolve(argument)] {
            Value::Param { .. } => false,
            Value::Instruction { instruction, .. } => match &self[*instruction] {
                Instruction::MakeArray { elements, .. } => {
                    elements.iter().all(|element| self.is_constant(*element))
                }
                _ => false,
            },
            _ => true,
        }
    }

    /// True that the input is a non-zero `Value::NumericConstant`
    pub(crate) fn is_constant_true(&self, argument: ValueId) -> bool {
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

impl std::ops::Index<ValueId> for DataFlowGraph {
    type Output = Value;
    fn index(&self, id: ValueId) -> &Self::Output {
        &self.values[id]
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

// The result of calling DataFlowGraph::insert_instruction can
// be a list of results or a single ValueId if the instruction was simplified
// to an existing value.
#[derive(Debug)]
pub(crate) enum InsertInstructionResult<'dfg> {
    /// Results is the standard case containing the instruction id and the results of that instruction.
    Results(InstructionId, &'dfg [ValueId]),
    SimplifiedTo(ValueId),
    SimplifiedToMultiple(Vec<ValueId>),
    InstructionRemoved,
}

impl<'dfg> InsertInstructionResult<'dfg> {
    /// Retrieve the first (and expected to be the only) result.
    pub(crate) fn first(&self) -> ValueId {
        match self {
            InsertInstructionResult::SimplifiedTo(value) => *value,
            InsertInstructionResult::SimplifiedToMultiple(values) => values[0],
            InsertInstructionResult::Results(_, results) => {
                assert_eq!(results.len(), 1);
                results[0]
            }
            InsertInstructionResult::InstructionRemoved => {
                panic!("Instruction was removed, no results")
            }
        }
    }

    /// Return all the results contained in the internal results array.
    /// This is used for instructions returning multiple results like function calls.
    pub(crate) fn results(self) -> Cow<'dfg, [ValueId]> {
        match self {
            InsertInstructionResult::Results(_, results) => Cow::Borrowed(results),
            InsertInstructionResult::SimplifiedTo(result) => Cow::Owned(vec![result]),
            InsertInstructionResult::SimplifiedToMultiple(results) => Cow::Owned(results),
            InsertInstructionResult::InstructionRemoved => Cow::Owned(vec![]),
        }
    }

    /// Returns the amount of ValueIds contained
    pub(crate) fn len(&self) -> usize {
        match self {
            InsertInstructionResult::SimplifiedTo(_) => 1,
            InsertInstructionResult::SimplifiedToMultiple(results) => results.len(),
            InsertInstructionResult::Results(_, results) => results.len(),
            InsertInstructionResult::InstructionRemoved => 0,
        }
    }
}

impl<'dfg> std::ops::Index<usize> for InsertInstructionResult<'dfg> {
    type Output = ValueId;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            InsertInstructionResult::Results(_, results) => &results[index],
            InsertInstructionResult::SimplifiedTo(result) => {
                assert_eq!(index, 0);
                result
            }
            InsertInstructionResult::SimplifiedToMultiple(results) => &results[index],
            InsertInstructionResult::InstructionRemoved => {
                panic!("Cannot index into InsertInstructionResult::InstructionRemoved")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DataFlowGraph;
    use crate::ssa::ir::{instruction::Instruction, types::Type};

    #[test]
    fn make_instruction() {
        let mut dfg = DataFlowGraph::default();
        let ins = Instruction::Allocate;
        let ins_id = dfg.make_instruction(ins, Some(vec![Type::field()]));

        let results = dfg.instruction_results(ins_id);
        assert_eq!(results.len(), 1);
    }
}
