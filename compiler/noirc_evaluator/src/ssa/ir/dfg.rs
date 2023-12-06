use std::borrow::Cow;

use crate::ssa::ir::instruction::SimplifyResult;

use super::{
    basic_block::{BasicBlock, BasicBlockId},
    function::FunctionId,
    instruction::{
        Instruction, InstructionId, InstructionResultType, Intrinsic, TerminatorInstruction,
    },
    map::DenseMap,
    types::Type,
    value::{Value, ValueId},
};

use acvm::FieldElement;
use fxhash::FxHashMap as HashMap;
use iter_extended::vecmap;
use noirc_errors::Location;
use serde::de::value;

/// The DataFlowGraph contains most of the actual data in a function including
/// its blocks, instructions, and values. This struct is largely responsible for
/// owning most data in a function and handing out Ids to this data that can be
/// shared without worrying about ownership.
#[derive(Debug, Default)]
pub(crate) struct DataFlowGraph {
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
    results: HashMap<InstructionId, Vec<ValueId>>,

    /// Storage for all of the values defined in this
    /// function.
    values: DenseMap<Value>,

    /// Each constant is unique, attempting to insert the same constant
    /// twice will return the same ValueId.
    constants: HashMap<(FieldElement, Type), ValueId>,

    /// Contains each function that has been imported into the current function.
    /// Each function's Value::Function is uniqued here so any given FunctionId
    /// will always have the same ValueId within this function.
    functions: HashMap<FunctionId, ValueId>,

    /// Contains each intrinsic that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given intrinsic is always
    /// represented by only 1 ValueId within this function.
    intrinsics: HashMap<Intrinsic, ValueId>,

    /// Contains each foreign function that has been imported into the current function.
    /// This map is used to ensure that the ValueId for any given foreign functôn is always
    /// represented by only 1 ValueId within this function.
    foreign_functions: HashMap<String, ValueId>,

    /// All blocks in a function
    blocks: DenseMap<BasicBlock>,

    /// Debugging information about which `ValueId`s have had their underlying `Value` substituted
    /// for that of another. This information is purely used for printing the SSA, and has no
    /// material effect on the SSA itself.
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
    locations: HashMap<InstructionId, CallStack>,

    slice_sizes: HashMap<ValueId, (usize, Vec<ValueId>)>,
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
        let parameters = self.blocks[block].parameters();

        let parameters = vecmap(parameters.iter().enumerate(), |(position, param)| {
            let typ = self.values[*param].get_type().clone();
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

    /// Inserts a new instruction at the end of the given block and returns its results
    pub(crate) fn insert_instruction_and_results(
        &mut self,
        instruction: Instruction,
        block: BasicBlockId,
        ctrl_typevars: Option<Vec<Type>>,
        call_stack: CallStack,
    ) -> InsertInstructionResult {
        use InsertInstructionResult::*;
        let results = match instruction.simplify(self, block, ctrl_typevars.clone()) {
            SimplifyResult::SimplifiedTo(simplification) => SimplifiedTo(simplification),
            SimplifyResult::SimplifiedToMultiple(simplification) => {
                SimplifiedToMultiple(simplification)
            }
            SimplifyResult::Remove => InstructionRemoved,
            result @ (SimplifyResult::SimplifiedToInstruction(_) | SimplifyResult::None) => {
                let instruction = result.instruction().unwrap_or(instruction);
                let id = self.make_instruction(instruction, ctrl_typevars);
                self.blocks[block].insert_instruction(id);
                self.locations.insert(id, call_stack);
                // let results = self.instruction_results(id).to_vec();
                // let x = &self.instructions[id].clone();
                // let mut slice_sizes = HashMap::default();
                // self.collect_slice_information(x, &mut slice_sizes, &results);
                InsertInstructionResult::Results(id, self.instruction_results(id))
            }
        };

        // let results_vec = results.results();
        
        results
    }

    /// Insert a value into the dfg's storage and return an id to reference it.
    /// Until the value is used in an instruction it is unreachable.
    pub(crate) fn make_value(&mut self, value: Value) -> ValueId {
        self.values.insert(value)
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
            Value::Instruction { typ, .. }
            | Value::Param { typ, .. }
            | Value::NumericConstant { typ, .. } => {
                *typ = target_type;
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
    pub(crate) fn make_constant(&mut self, constant: FieldElement, typ: Type) -> ValueId {
        if let Some(id) = self.constants.get(&(constant, typ.clone())) {
            return *id;
        }
        let id = self.values.insert(Value::NumericConstant { constant, typ: typ.clone() });
        self.constants.insert((constant, typ), id);
        id
    }

    /// Create a new constant array value from the given elements
    pub(crate) fn make_array(&mut self, array: im::Vector<ValueId>, typ: Type) -> ValueId {
        assert!(matches!(typ, Type::Array(..) | Type::Slice(_)));
        self.make_value(Value::Array { array, typ })
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

    pub(crate) fn get_intrinsic(&mut self, intrinsic: Intrinsic) -> Option<&ValueId> {
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
        instruction_id: InstructionId,
        ctrl_typevars: Option<Vec<Type>>,
    ) {
        self.results.insert(instruction_id, Default::default());

        // Get all of the types that this instruction produces
        // and append them as results.
        for typ in self.instruction_result_types(instruction_id, ctrl_typevars) {
            self.append_result(instruction_id, typ);
        }
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
        self.values[value].get_type().clone()
    }

    /// True if the type of this value is Type::Reference.
    /// Using this method over type_of_value avoids cloning the value's type.
    pub(crate) fn value_is_reference(&self, value: ValueId) -> bool {
        matches!(self.values[value].get_type(), Type::Reference(_))
    }

    /// Appends a result type to the instruction.
    pub(crate) fn append_result(&mut self, instruction_id: InstructionId, typ: Type) -> ValueId {
        let results = self.results.get_mut(&instruction_id).unwrap();
        let expected_res_position = results.len();

        let value_id = self.values.insert(Value::Instruction {
            typ,
            position: expected_res_position,
            instruction: instruction_id,
        });

        // Add value to the list of results for this instruction
        results.push(value_id);
        value_id
    }

    /// Returns the number of instructions
    /// inserted into functions.
    pub(crate) fn num_instructions(&self) -> usize {
        self.instructions.len()
    }

    /// Returns all of result values which are attached to this instruction.
    pub(crate) fn instruction_results(&self, instruction_id: InstructionId) -> &[ValueId] {
        self.results.get(&instruction_id).expect("expected a list of Values").as_slice()
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
    ) -> Option<(FieldElement, Type)> {
        match &self.values[self.resolve(value)] {
            Value::NumericConstant { constant, typ } => Some((*constant, typ.clone())),
            _ => None,
        }
    }

    /// Returns the Value::Array associated with this ValueId if it refers to an array constant.
    /// Otherwise, this returns None.
    pub(crate) fn get_array_constant(&self, value: ValueId) -> Option<(im::Vector<ValueId>, Type)> {
        match &self.values[self.resolve(value)] {
            // Arrays are shared, so cloning them is cheap
            Value::Array { array, typ } => Some((array.clone(), typ.clone())),
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
        match &self.values[self.resolve(value)] {
            Value::Instruction { instruction, .. } => self.get_call_stack(*instruction),
            _ => im::Vector::new(),
        }
    }

    /// True if the given ValueId refers to a constant value
    pub(crate) fn is_constant(&self, argument: ValueId) -> bool {
        !matches!(&self[self.resolve(argument)], Value::Instruction { .. } | Value::Param { .. })
    }

    pub(crate) fn get_slice_size(&self, value: ValueId) -> Option<&(usize, Vec<ValueId>)> {
        self.slice_sizes.get(&value)
    }

    pub(crate) fn get_slice_sizes(&self) -> HashMap<ValueId, (usize, Vec<ValueId>)> {
        self.slice_sizes.clone()
    }

    /// Determine how the slice sizes map needs to be updated according to the provided instruction.
    fn collect_slice_information(
        &mut self,
        instruction: &Instruction,
        slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
        results: &[ValueId],
    ) {
            // let (instruction, _) = &self.map_instruction(instruction_id);
            // let old_results = self.instruction_results(instruction_id);
            // dbg!(old_results.clone());
            // dbg!(&self.inserter.function.dfg.resolve(results[0]));
            // let resolved_results = results.into_iter().map(|res| &self.inserter.resolve(*res)).collect::<Vec<_>>();
            match instruction {
                Instruction::ArrayGet { array, .. } => {
                    let array_typ = self.type_of_value(*array);
                    let array_value = &self[*array];
                    // If we have an SSA value containing nested slices we should mark it
                    // as a slice that potentially requires to be filled with dummy data.
                    if matches!(array_value, Value::Array { .. }) && array_typ.contains_slice_element()
                    {
                        // Initial insertion into the slice sizes map
                        // Any other insertions should only occur if the value is already
                        // a part of the map.
                        self.compute_slice_sizes(*array, slice_sizes);
                    }
    
                    let res_typ = self.type_of_value(results[0]);
                    if res_typ.contains_slice_element() {
                        // dbg!(array);
                        if let Some(inner_sizes) = self.slice_sizes.get_mut(array) {
                            // dbg!(array);
                            // if array.to_usize() == 851 {
                            //     dbg!("got 851");
                            // }
                            // Include the result in the parent array potential children
                            // If the result has internal slices and is called in an array set
                            // we could potentially have a new larger slice which we need to account for
                            inner_sizes.1.push(results[0]);
    
                            let inner_sizes_iter = inner_sizes.1.clone();
                            for slice_value in inner_sizes_iter {
                                let inner_slice = slice_sizes.get(&slice_value).unwrap_or_else(|| {
                                    panic!("ICE: should have inner slice set for {slice_value}")
                                });
                                self.slice_sizes.insert(results[0], inner_slice.clone());

                                // self.slice_sizes.insert(results[0], inner_slice.clone());
                            }
                        }
                    }
                }
                Instruction::ArraySet { array, value, .. } => {
                    let array_typ = self.type_of_value(*array);
                    let array_value = &self[*array];
                    // If we have an SSA value containing nested slices we should mark it
                    // as a slice that potentially requires to be filled with dummy data.
                    if matches!(array_value, Value::Array { .. }) && array_typ.contains_slice_element()
                    {
                        // Initial insertion into the slice sizes map
                        // Any other insertions should only occur if the value is already
                        // a part of the map.
                        self.compute_slice_sizes(*array, slice_sizes);
                    }
    
                    let value_typ = self.type_of_value(*value);
                    if value_typ.contains_slice_element() {
                        self.compute_slice_sizes(*value, slice_sizes);
                        let result = &self.resolve(results[0]);
                        dbg!(result.clone());
                        let inner_sizes = self.slice_sizes.get_mut(array).unwrap_or_else(|| {
                            panic!("ICE: Expected slice sizes for ArraySet array {array} value {value} with results {results:?}")
                        });
                        inner_sizes.1.push(*value);
                    }
    
                    if let Some(inner_sizes) = slice_sizes.get_mut(array) {
                        let inner_sizes = inner_sizes.clone();
                        // if results[0].to_usize() == 866 {
                        //     dbg!("GOT 866");
                        // }
                        // if results[0].to_usize() == 868 {
                        //     dbg!("GOT 868");
                        // }
                        // dbg!(results[0]);
                        self.slice_sizes.insert(results[0], inner_sizes);
                    }
                }
                Instruction::Call { func, arguments } => {
                    let func = &self[*func];
                    if let Value::Intrinsic(intrinsic) = func {
                        let (argument_index, result_index) = match intrinsic {
                            Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SlicePopBack
                            | Intrinsic::SliceInsert
                            | Intrinsic::SliceRemove => (1, 1),
                            // `pop_front` returns the the popped element the slice and then the respective slice
                            // This means in the case of a slice with structs the result index of the popped slice
                            // will change depending on the number of elements in the struct.
                            // For example, a slice with four elements will look as such in SSA:
                            // v3, v4, v5, v6, v7, v8 = call slice_pop_front(v1, v2)
                            // where v7 is the slice length and v8 is the popped slice itself.
                            Intrinsic::SlicePopFront => (1, results.len() - 1),
                            _ => return,
                        };
                        let slice_contents = arguments[argument_index];
                        match intrinsic {
                            Intrinsic::SlicePushBack
                            | Intrinsic::SlicePushFront
                            | Intrinsic::SliceInsert => {
                                for arg in &arguments[(argument_index + 1)..] {
                                    let element_typ = self.type_of_value(*arg);
                                    if element_typ.contains_slice_element() {
                                        self.compute_slice_sizes(*arg, slice_sizes);
                                    }
                                }
                                if let Some(inner_sizes) = self.slice_sizes.get_mut(&slice_contents) {
                                    inner_sizes.0 += 1;
    
                                    let inner_sizes = inner_sizes.clone();
                                    self.slice_sizes.insert(results[result_index], inner_sizes);
                                }
                            }
                            Intrinsic::SlicePopBack
                            | Intrinsic::SliceRemove
                            | Intrinsic::SlicePopFront => {
                                // We do not decrement the size on intrinsics that could remove values from a slice.
                                // This is because we could potentially go back to the smaller slice and not fill in dummies.
                                // This pass should be tracking the potential max that a slice ***could be***
                                if let Some(inner_sizes) = self.slice_sizes.get(&slice_contents) {
                                    let inner_sizes = inner_sizes.clone();
                                    self.slice_sizes.insert(results[result_index], inner_sizes);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Instruction::Store { address, value } => {
                    let value_typ = self.type_of_value(*value);
                    if value_typ.contains_slice_element() {
                        dbg!(value);
                        dbg!(address);
                        // dbg!(self.inserter.function.dfg.resolve(*address));
                        self.compute_slice_sizes(*value, slice_sizes);
    
                        if let Some(inner_sizes) = self.slice_sizes.get(value) {
                            let inner_sizes = inner_sizes.clone();
                            self.slice_sizes.insert(*address, inner_sizes);
                        }
                    }
                }
                Instruction::Load { address } => {
                    let load_typ = self.type_of_value(*address);
                    if load_typ.contains_slice_element() {
                        // dbg!("got here");
                        if let Some(inner_sizes) = self.slice_sizes.get(address) {
                            let inner_sizes = inner_sizes.clone();
                            self.slice_sizes.insert(results[0], inner_sizes);
                        }
                    }
                }
                _ => {}
            }
        }
    
        // This methods computes a map representing a nested slice.
        // The method also automatically computes the given max slice size
        // at each depth of the recursive type.
        // For example if we had a next slice
        fn compute_slice_sizes(
            &mut self,
            array_id: ValueId,
            slice_sizes: &mut HashMap<ValueId, (usize, Vec<ValueId>)>,
        ) {
            dbg!(array_id);
            if let Value::Array { array, typ } = &self[array_id].clone() {
                if array_id.to_usize() == 14 {
                    dbg!(array.clone());
                    dbg!(typ.clone());
                }
                if let Type::Slice(_) = typ {
                    let element_size = typ.element_size();
                    let len = array.len() / element_size;
                    let mut slice_value = (len, vec![]);
                    for value in array {
                        let typ = self.type_of_value(*value);
                        if matches!(typ, Type::Slice(_)) && matches!(&self[*value], Value::Array { .. }) {
                            slice_value.1.push(*value);
                            dbg!(value);
                            self.compute_slice_sizes(*value, slice_sizes);
                        }
                    }
                    dbg!(self.slice_sizes.clone());

                    // Mark the correct max size based upon an array values internal structure
                    let mut max_size = 0;
                    for inner_value in slice_value.1.iter() {
                        let inner_slice =
                            self.slice_sizes.get(inner_value).unwrap_or_else(|| {
                                panic!("ICE: should have inner slice set for {inner_value}")
                            });
                        if inner_slice.0 > max_size {
                            max_size = inner_slice.0;
                        }
                    }
                    for inner_value in slice_value.1.iter() {
                        let inner_slice =
                            self.slice_sizes.get_mut(inner_value).expect("ICE: should have inner slice set");
                        if inner_slice.0 < max_size {
                            inner_slice.0 = max_size;
                        }
                    }
                    // slice_sizes.insert(array_id, slice_value);
                    dbg!(array_id);
                    self.slice_sizes.insert(array_id, slice_value);
                }
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
            InsertInstructionResult::Results(_, results) => results[0],
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
