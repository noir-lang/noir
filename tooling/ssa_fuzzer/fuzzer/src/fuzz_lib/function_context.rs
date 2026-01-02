use super::{
    NUMBER_OF_VARIABLES_INITIAL,
    block_context::BlockContext,
    instruction::{FunctionInfo, InstructionBlock},
    options::{FunctionContextOptions, SsaBlockOptions},
};
use acvm::FieldElement;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::{
    builder::FuzzerBuilder,
    typed_value::{NumericType, Type, TypedValue},
};
use noirc_evaluator::ssa::ir::{basic_block::BasicBlockId, function::Function, map::Id};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    hash::Hash,
};
use strum_macros::EnumCount;

const NUMBER_OF_BLOCKS_INSERTING_IN_JMP: usize = 1;
const NUMBER_OF_BLOCKS_INSERTING_IN_JMP_IF: usize = 2;
const NUMBER_OF_BLOCKS_INSERTING_IN_LOOP: usize = 4;

pub(crate) type ValueWithType = (FieldElement, NumericType);

/// Represents the data describing a function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FunctionData {
    pub(crate) commands: Vec<FuzzerFunctionCommand>,
    /// Input types of the function
    ///
    /// Overwritten for main function by the types of the initial witness
    pub(crate) input_types: Vec<Type>,
    pub(crate) return_instruction_block_idx: usize,
    pub(crate) return_type: Type,
}

impl Default for FunctionData {
    fn default() -> Self {
        FunctionData {
            commands: vec![],
            input_types: vec![Type::Numeric(NumericType::Field)],
            return_instruction_block_idx: 0,
            return_type: Type::Numeric(NumericType::Field),
        }
    }
}

/// Represents set of commands for the fuzzer
///
/// After executing all commands, terminates all blocks from current_block_queue with return
#[derive(Arbitrary, Debug, Clone, Hash, Serialize, Deserialize, EnumCount)]
pub(crate) enum FuzzerFunctionCommand {
    /// Adds instructions to current_block_context from stored instruction_blocks
    InsertSimpleInstructionBlock { instruction_block_idx: usize },
    /// terminates current SSA block with jmp_if_else. Creates two new SSA blocks from chosen InstructionBlocks.
    /// If in loop, finalizes then and else branches with jump to the loop iter block. Switches context to the loop end block.
    /// Otherwise, switches current_block_context to then_branch.
    /// Adds else_branch to the next_block_queue. If current SSA block is already terminated, skip.
    InsertJmpIfBlock { block_then_idx: usize, block_else_idx: usize },
    /// Terminates current SSA block with jmp.
    /// If in loop, finalizes the loop and switches context to the loop end block.
    ///
    /// Otherwise, creates new SSA block from chosen InstructionBlock.
    /// Switches current_block_context to jmp_destination.
    InsertJmpBlock { block_idx: usize },
    /// Adds current SSA block to the next_block_queue. Switches context to stored in next_block_queue.
    SwitchToNextBlock,

    /// Adds loop to the program.
    /// Switches context to the loop body block.
    InsertCycle { block_body_idx: usize, start_iter: u8, end_iter: u8 },

    /// Inserts call to another function
    InsertFunctionCall { function_idx: usize, args: [usize; NUMBER_OF_VARIABLES_INITIAL as usize] },
}

/// Default command is InsertSimpleInstructionBlock with instruction_block_idx = 0
///
/// Only used for mutations
impl Default for FuzzerFunctionCommand {
    fn default() -> Self {
        Self::InsertSimpleInstructionBlock { instruction_block_idx: 0 }
    }
}

struct CycleInfo {
    block_iter_id: BasicBlockId,
    block_end_id: BasicBlockId,
}

#[derive(Clone)]
pub(crate) struct StoredBlock {
    context: BlockContext,
    block_id: BasicBlockId,
}

/// Main context for the fuzzer containing both ACIR and Brillig builders and their state
pub(crate) struct FuzzerFunctionContext<'a> {
    /// Builder
    builder: &'a mut FuzzerBuilder,
    /// Current blocks
    current_block: StoredBlock,
    /// Stored blocks that are not terminated
    not_terminated_blocks: VecDeque<StoredBlock>,
    /// Instruction blocks
    instruction_blocks: &'a Vec<InstructionBlock>,
    /// Hashmap of stored variables in blocks
    stored_variables_for_block: HashMap<BasicBlockId, HashMap<Type, Vec<TypedValue>>>,
    /// Hashmap of stored blocks
    stored_blocks: HashMap<BasicBlockId, StoredBlock>,
    /// Options of the program context
    function_context_options: FunctionContextOptions,
    /// Number of instructions inserted in the program
    inserted_instructions_count: usize,
    /// Number of SSA blocks inserted in the program
    inserted_ssa_blocks_count: usize,

    /// Stored cycles info, to handle loops in Jmp, JmpIf and finalization
    cycle_bodies_to_iters_ids: HashMap<BasicBlockId, CycleInfo>,
    /// Number of iterations of loops in the program
    parent_iterations_count: usize,

    return_type: Type,
    defined_functions: BTreeMap<Id<Function>, FunctionInfo>,
}

impl<'a> FuzzerFunctionContext<'a> {
    /// Creates a new fuzzer context with the given types
    /// It creates a new variable for each type and stores it in the map
    pub(crate) fn new(
        types: Vec<Type>,
        instruction_blocks: &'a Vec<InstructionBlock>,
        context_options: FunctionContextOptions,
        return_type: Type,
        defined_functions: BTreeMap<Id<Function>, FunctionInfo>,
        builder: &'a mut FuzzerBuilder,
    ) -> Self {
        let mut ids = HashMap::new();
        for type_ in types {
            let id = builder.insert_variable(type_.clone().into());
            ids.entry(type_.clone()).or_insert(Vec::new()).push(id);
        }

        let main_block = builder.get_current_block();
        let current_block = StoredBlock {
            context: BlockContext::new(
                ids.clone(),
                VecDeque::new(),
                SsaBlockOptions::from(context_options.clone()),
            ),
            block_id: main_block,
        };

        Self {
            builder,
            current_block,
            not_terminated_blocks: VecDeque::new(),
            instruction_blocks,
            stored_variables_for_block: HashMap::new(),
            stored_blocks: HashMap::new(),
            function_context_options: context_options,
            inserted_instructions_count: 0,
            inserted_ssa_blocks_count: 0,
            cycle_bodies_to_iters_ids: HashMap::new(),
            parent_iterations_count: 1,
            defined_functions,
            return_type,
        }
    }

    /// Creates a new fuzzer context with the given values and inserts them as constants
    ///
    /// Used for fuzzing constant folding SSA pass.
    pub(crate) fn new_constant_context(
        values_types: Vec<ValueWithType>,
        instruction_blocks: &'a Vec<InstructionBlock>,
        context_options: FunctionContextOptions,
        return_type: Type,
        defined_functions: BTreeMap<Id<Function>, FunctionInfo>,
        builder: &'a mut FuzzerBuilder,
    ) -> Self {
        let mut ids = HashMap::new();

        for (value, type_) in values_types.into_iter() {
            let field_element = value;
            ids.entry(Type::Numeric(type_))
                .or_insert(Vec::new())
                .push(builder.insert_constant(field_element, type_));
        }

        let main_block = builder.get_current_block();
        let current_block = StoredBlock {
            context: BlockContext::new(
                ids.clone(),
                VecDeque::new(),
                SsaBlockOptions::from(context_options.clone()),
            ),
            block_id: main_block,
        };

        Self {
            builder,
            current_block,
            not_terminated_blocks: VecDeque::new(),
            instruction_blocks,
            stored_variables_for_block: HashMap::new(),
            stored_blocks: HashMap::new(),
            function_context_options: context_options,
            inserted_instructions_count: 0,
            inserted_ssa_blocks_count: 0,
            cycle_bodies_to_iters_ids: HashMap::new(),
            parent_iterations_count: 1,
            defined_functions,
            return_type,
        }
    }
    /// Inserts a new SSA block into builder and returns its id
    fn insert_ssa_block(&mut self) -> BasicBlockId {
        self.builder.insert_block()
    }

    /// Inserts a new constant into builder and returns its value
    fn insert_constant(
        &mut self,
        value: impl Into<FieldElement> + Clone,
        type_: NumericType,
    ) -> TypedValue {
        self.builder.insert_constant(value.clone(), type_)
    }

    /// Inserts a new jmp instruction into builder
    fn insert_jmp_instruction(&mut self, block_id: BasicBlockId, params: Vec<TypedValue>) {
        self.builder.insert_jmp_instruction(block_id, params.clone());
    }

    /// Switches to the block
    ///
    /// This function is used to switch to the block in builder context.
    fn switch_to_block(&mut self, block_id: BasicBlockId) {
        self.builder.switch_to_block(block_id);
    }

    /// Stores variables of the current block
    ///
    /// SSA block can use variables from predecessor that is not in branch.
    /// Look [Self::find_closest_parent] for more details.
    fn store_variables(&mut self) {
        self.stored_variables_for_block.insert(
            self.current_block.block_id,
            self.current_block.context.stored_variables.clone(),
        );
    }

    fn process_jmp_if_command(&mut self, block_then_idx: usize, block_else_idx: usize) {
        let block_then_instruction_block =
            self.instruction_blocks[block_then_idx % self.instruction_blocks.len()].clone();
        let block_else_instruction_block =
            self.instruction_blocks[block_else_idx % self.instruction_blocks.len()].clone();

        self.store_variables();

        if (block_then_instruction_block.instructions.len()
            + block_else_instruction_block.instructions.len())
            * self.parent_iterations_count
            + self.inserted_instructions_count
            > self.function_context_options.max_instructions_num
        {
            return;
        }
        if self.inserted_ssa_blocks_count + NUMBER_OF_BLOCKS_INSERTING_IN_JMP_IF
            > self.function_context_options.max_ssa_blocks_num
        {
            return;
        }
        self.inserted_instructions_count += block_then_instruction_block.instructions.len();
        self.inserted_instructions_count += block_else_instruction_block.instructions.len();
        self.inserted_ssa_blocks_count += NUMBER_OF_BLOCKS_INSERTING_IN_JMP_IF;

        // creates new blocks
        let block_then_id = self.insert_ssa_block();
        let block_else_id = self.insert_ssa_block();

        // creates new contexts of created blocks
        let mut parent_blocks_history = self.current_block.context.parent_blocks_history.clone();
        parent_blocks_history.push_front(self.current_block.block_id);
        let mut block_then_context = BlockContext {
            children_blocks: vec![],
            parent_blocks_history: parent_blocks_history.clone(),
            ..self.current_block.context.clone()
        };
        let mut block_else_context = BlockContext {
            children_blocks: vec![],
            parent_blocks_history,
            ..self.current_block.context.clone()
        };

        // inserts instructions into created blocks
        self.switch_to_block(block_then_id);
        block_then_context
            .insert_instructions(self.builder, &block_then_instruction_block.instructions);

        self.switch_to_block(block_else_id);
        block_else_context
            .insert_instructions(self.builder, &block_else_instruction_block.instructions);

        // terminates current block with jmp_if
        self.switch_to_block(self.current_block.block_id);
        self.current_block.context.finalize_block_with_jmp_if(
            self.builder,
            block_then_id,
            block_else_id,
        );
        self.stored_blocks.insert(self.current_block.block_id, self.current_block.clone());

        // switch context to then block and push else block to the queue
        let then_stored_block =
            StoredBlock { context: block_then_context, block_id: block_then_id };
        let else_stored_block =
            StoredBlock { context: block_else_context, block_id: block_else_id };

        // if current context is cycle body we define then and else branch as new bodies
        if self.cycle_bodies_to_iters_ids.contains_key(&self.current_block.block_id) {
            let CycleInfo { block_iter_id, block_end_id } =
                self.cycle_bodies_to_iters_ids[&self.current_block.block_id];
            // block cannot have more than two predecessors
            // so we create a join block that terminates with a jmp to iter block
            // and then terminate then and else blocks with jmp join block in Self::finalize_cycles
            let block_join_id = self.insert_ssa_block();
            self.switch_to_block(block_join_id);
            self.insert_jmp_instruction(block_iter_id, vec![]);
            self.cycle_bodies_to_iters_ids
                .insert(block_then_id, CycleInfo { block_iter_id: block_join_id, block_end_id });
            self.cycle_bodies_to_iters_ids
                .insert(block_else_id, CycleInfo { block_iter_id: block_join_id, block_end_id });
            self.cycle_bodies_to_iters_ids.remove(&self.current_block.block_id);
        } else {
            self.not_terminated_blocks.push_back(else_stored_block);
        }
        self.switch_to_block(then_stored_block.block_id);
        self.current_block = then_stored_block.clone();
    }

    fn process_jmp_block(&mut self, block_idx: usize) {
        // If the current block is a loop body
        if self.cycle_bodies_to_iters_ids.contains_key(&self.current_block.block_id) {
            let CycleInfo { block_iter_id, block_end_id } =
                self.cycle_bodies_to_iters_ids[&self.current_block.block_id];
            // finalize loop body with jmp to the loop iter block
            self.switch_to_block(self.current_block.block_id);
            self.insert_jmp_instruction(block_iter_id, vec![]);

            self.cycle_bodies_to_iters_ids.remove(&self.current_block.block_id);

            // switch context to the loop end block
            let current_block = self.stored_blocks[&block_end_id].context.clone();
            self.current_block = StoredBlock { context: current_block, block_id: block_end_id };
            self.switch_to_block(self.current_block.block_id);
            return;
        }
        self.store_variables();

        // find instruction block to be inserted
        let block = self.instruction_blocks[block_idx % self.instruction_blocks.len()].clone();
        if (block.instructions.len() + self.inserted_instructions_count)
            * self.parent_iterations_count
            > self.function_context_options.max_instructions_num
        {
            return;
        }
        if self.inserted_ssa_blocks_count + NUMBER_OF_BLOCKS_INSERTING_IN_JMP
            > self.function_context_options.max_ssa_blocks_num
        {
            return;
        }
        self.inserted_instructions_count += block.instructions.len();
        self.inserted_ssa_blocks_count += NUMBER_OF_BLOCKS_INSERTING_IN_JMP;

        // creates new block
        let destination_block_id = self.insert_ssa_block();

        // creates new context for the new block
        let mut parent_blocks_history = self.current_block.context.parent_blocks_history.clone();
        parent_blocks_history.push_front(self.current_block.block_id);
        self.switch_to_block(destination_block_id);
        let mut destination_block_context = BlockContext {
            children_blocks: vec![],
            parent_blocks_history,
            ..self.current_block.context.clone()
        };

        // inserts instructions into the new block
        destination_block_context.insert_instructions(self.builder, &block.instructions);

        // switches to the current block and terminates it with jmp
        self.switch_to_block(self.current_block.block_id);
        self.current_block.context.finalize_block_with_jmp(
            self.builder,
            destination_block_id,
            vec![],
        );
        self.stored_blocks.insert(self.current_block.block_id, self.current_block.clone());

        // switches to the new block and updates current block
        self.switch_to_block(destination_block_id);
        self.current_block =
            StoredBlock { context: destination_block_context, block_id: destination_block_id };
    }

    /// Adds a loop to the program. Switches context to the loop body block.
    ///
    /// Loops in Noir on SSA level work as follows:
    /// 1) Create constant for start iteration
    /// 2) Jump to the "block_if" (block that checks if the loop should continue)
    /// 3) In "block_if" create constant for end iteration
    /// 4) Finalize "block_if" with jmp_if iter < end_iter then "block_body" else "block_end"
    /// 5) In "block_body" do everything you want
    /// 6) "body_block" must be finalized with jmp to "block_iter"
    /// 7) "block_iter" increment the iterator and jump to "block_if"
    ///
    /// For example following Noir program:
    /// ```noir
    /// fn main(x: Field) -> pub Field {
    ///   let mut y = x;
    ///   for i in 0..10 {
    ///     y *= x;
    ///   }
    ///   y
    /// }
    /// ```
    /// Compiles into SSA (nargo compile --show-ssa --force-brillig):
    /// ```text
    /// fn main f0 {
    ///   b0(v0: Field):
    ///     v2 = allocate -> &mut Field
    ///     store v0 at v2
    ///     jmp b1(u32 0) <--------------------------------- create iter (0) and jump to the "if_block"
    ///   b1(v1: u32): <------------------------------------ "if_block"
    ///     v5 = lt v1, u32 10 <---------------------------- compare iter with end_iter (10)
    ///     jmpif v5 then: b3, else: b2 <------------------- if iter < end_iter, jump to the "body_block", otherwise jump to the "end_block"
    ///   b2(): <------------------------------------------- "end_block"
    ///     v6 = load v2 -> Field
    ///     return v6
    ///   b3(): <------------------------------------------- "body_block"
    ///     v7 = load v2 -> Field
    ///     v8 = mul v7, v0
    ///     store v8 at v2
    ///     // part below can be in other block
    ///     v10 = unchecked_add v1, u32 1 <------------------ increment iter
    ///     jmp b1(v10) <------------------------------------ jump to the "if_block"
    /// }
    /// ```
    fn process_cycle_command(&mut self, block_body_idx: usize, start_iter: usize, end_iter: usize) {
        let block_body =
            self.instruction_blocks[block_body_idx % self.instruction_blocks.len()].clone();

        if end_iter >= start_iter {
            let parent_iters_count = self.parent_iterations_count * (end_iter - start_iter + 1); // nested loops count of iters
            // check if the number of iterations is not too big
            if parent_iters_count > self.function_context_options.max_iterations_num {
                return;
            }
            if self.inserted_ssa_blocks_count + NUMBER_OF_BLOCKS_INSERTING_IN_LOOP
                > self.function_context_options.max_ssa_blocks_num
            {
                return;
            }
            self.inserted_instructions_count +=
                block_body.instructions.len() * (end_iter - start_iter + 1);
            self.inserted_ssa_blocks_count += NUMBER_OF_BLOCKS_INSERTING_IN_LOOP;
            self.parent_iterations_count = parent_iters_count;
        }

        let block_body_id = self.insert_ssa_block();

        // if we are in loop, we use iter_block of this loop as the end_block for the new loop
        let block_end_id =
            if self.cycle_bodies_to_iters_ids.contains_key(&self.current_block.block_id) {
                self.cycle_bodies_to_iters_ids[&self.current_block.block_id].block_iter_id
            } else {
                self.insert_ssa_block()
            };
        // create constant for start
        let start_id = self.insert_constant(start_iter, NumericType::U32);
        // create constant for end
        let end_id = self.insert_constant(end_iter, NumericType::U32);
        // create constant for 1 (to increment iter)
        let one_id = self.insert_constant(1_u32, NumericType::U32);

        // create if block
        let block_if_id = self.insert_ssa_block();
        self.switch_to_block(block_if_id);
        // create iter
        let real_iter_id =
            self.builder.add_block_parameter(block_if_id, Type::Numeric(NumericType::U32));
        // condition = iter < end
        let condition =
            self.builder.insert_lt_instruction(real_iter_id.clone(), end_id.clone()).value_id;
        // jmpif condition then: block_body, else: block_end
        self.builder.insert_jmpif_instruction(condition, block_body_id, block_end_id);

        // create iter block
        let block_iter_id = self.insert_ssa_block();
        self.switch_to_block(block_iter_id);
        // j = iter + 1
        let iterator_plus_one =
            self.builder.insert_add_instruction_checked(real_iter_id.clone(), one_id.clone());
        // jump to the "if_block" with j = iter + 1
        self.insert_jmp_instruction(block_if_id, vec![iterator_plus_one.clone()]);

        // switch to the context block and finalizes it with jmp to the "if_block" with iter = start
        self.switch_to_block(self.current_block.block_id);
        self.insert_jmp_instruction(block_if_id, vec![start_id.clone()]);

        // fill body block with instructions
        let mut block_body_context =
            BlockContext { children_blocks: vec![], ..self.current_block.context.clone() };
        self.switch_to_block(block_body_id);
        block_body_context.insert_instructions(self.builder, &block_body.instructions);

        let end_context =
            if self.cycle_bodies_to_iters_ids.contains_key(&self.current_block.block_id) {
                self.stored_blocks
                    [&self.cycle_bodies_to_iters_ids[&self.current_block.block_id].block_end_id]
                    .context
                    .clone()
            } else {
                self.current_block.context.clone()
            };
        // end block does not share variables with body block, so we copy them from the current block
        let block_end_context = BlockContext {
            children_blocks: vec![],
            parent_blocks_history: block_body_context.parent_blocks_history.clone(),
            ..end_context
        };

        let end_block_stored = StoredBlock { context: block_end_context, block_id: block_end_id };
        // connect end block with the current block
        // stores end_block and current_block
        // we skip other blocks, because loops has other logic of finalization
        self.current_block.context.children_blocks.push(end_block_stored.block_id);
        self.stored_blocks.insert(self.current_block.block_id, self.current_block.clone());
        self.stored_blocks.insert(end_block_stored.block_id, end_block_stored.clone());

        // switch context to the loop body block and store loop info
        self.current_block = StoredBlock { context: block_body_context, block_id: block_body_id };
        self.cycle_bodies_to_iters_ids
            .insert(block_body_id, CycleInfo { block_iter_id, block_end_id });
    }

    pub(crate) fn process_fuzzer_command(&mut self, command: &FuzzerFunctionCommand) {
        match command {
            FuzzerFunctionCommand::InsertSimpleInstructionBlock { instruction_block_idx } => {
                let instruction_block =
                    &self.instruction_blocks[instruction_block_idx % self.instruction_blocks.len()];
                if (self.inserted_instructions_count + instruction_block.instructions.len())
                    * self.parent_iterations_count
                    > self.function_context_options.max_instructions_num
                {
                    return;
                }
                self.current_block
                    .context
                    .insert_instructions(self.builder, &instruction_block.instructions);
                self.inserted_instructions_count += instruction_block.instructions.len();
            }
            FuzzerFunctionCommand::InsertJmpIfBlock { block_then_idx, block_else_idx } => {
                if !self.function_context_options.fuzzer_command_options.jmp_if_enabled {
                    return;
                }
                self.process_jmp_if_command(*block_then_idx, *block_else_idx);
            }
            FuzzerFunctionCommand::InsertJmpBlock { block_idx } => {
                if !self.function_context_options.fuzzer_command_options.jmp_block_enabled {
                    return;
                }
                self.process_jmp_block(*block_idx);
            }
            FuzzerFunctionCommand::SwitchToNextBlock => {
                if !self
                    .function_context_options
                    .fuzzer_command_options
                    .switch_to_next_block_enabled
                {
                    return;
                }
                self.not_terminated_blocks.push_back(self.current_block.clone());
                self.current_block = self.not_terminated_blocks.pop_front().unwrap();
                self.switch_to_block(self.current_block.block_id);
            }
            FuzzerFunctionCommand::InsertCycle { block_body_idx, start_iter, end_iter } => {
                if !self.function_context_options.fuzzer_command_options.loops_enabled {
                    return;
                }
                self.process_cycle_command(
                    *block_body_idx,
                    *start_iter as usize,
                    *end_iter as usize,
                );
            }
            FuzzerFunctionCommand::InsertFunctionCall { function_idx, args } => {
                let num_of_defined_functions = self.defined_functions.keys().len();
                if num_of_defined_functions == 0 {
                    return;
                }
                let function_id = *self
                    .defined_functions
                    .keys()
                    .nth(function_idx % num_of_defined_functions)
                    .unwrap();
                let function_info = self.defined_functions[&function_id].clone();
                if function_info.max_unrolled_size == 0 {
                    unreachable!("Encountered a function with no unrolled size");
                }
                let unrolled_size = function_info.max_unrolled_size * self.parent_iterations_count;
                if self.inserted_instructions_count + unrolled_size
                    > self.function_context_options.max_instructions_num
                {
                    return;
                }
                self.inserted_instructions_count += unrolled_size;

                self.current_block.context.process_function_call(
                    self.builder,
                    function_id,
                    function_info,
                    args,
                );
            }
        }
    }

    /// Merges two blocks into one
    /// Creates empty merged_block. Terminates first_block and second_block with jmp to merged_block
    /// Returns merged_block
    fn merge_blocks(
        &mut self,
        mut first_block: StoredBlock,
        mut second_block: StoredBlock,
    ) -> StoredBlock {
        let merged_block_id = self.insert_ssa_block();
        log::debug!("merging blocks {:?} and {:?}", first_block.block_id, second_block.block_id);

        let mut parent_blocks_history = first_block.context.parent_blocks_history.clone();
        parent_blocks_history.push_front(first_block.block_id);
        parent_blocks_history.push_front(second_block.block_id);

        let closest_parent = self.find_closest_parent(&first_block, &second_block);
        let closest_parent_block = self.stored_blocks[&closest_parent].clone();

        let merged_block_context = BlockContext {
            children_blocks: vec![],
            parent_blocks_history,
            ..closest_parent_block.context.clone()
        };
        self.switch_to_block(first_block.block_id);
        first_block.context.finalize_block_with_jmp(self.builder, merged_block_id, vec![]);
        self.stored_blocks.insert(first_block.block_id, first_block.clone());

        self.switch_to_block(second_block.block_id);
        second_block.context.finalize_block_with_jmp(self.builder, merged_block_id, vec![]);
        self.stored_blocks.insert(second_block.block_id, second_block.clone());

        let merged_block = StoredBlock { context: merged_block_context, block_id: merged_block_id };
        self.stored_blocks.insert(merged_block.block_id, merged_block.clone());
        merged_block
    }

    /// Finds closest parent for lhs and rhs blocks
    ///
    ///    b0
    ///    ↓
    ///    b1
    ///   ↙   ↘
    /// b2    b3
    /// ↓      |
    /// b4     |
    /// ↙  ↘   |
    ///b5  b6  |
    /// ↘  ↙   ↓
    ///  b7    b8
    ///   ↘   ↙
    ///    b9
    /// closest parent for b5 and b6 is b4
    /// closest parent for b7 and b8 is b1
    ///
    /// SSA block can use variables from predecessor that is not in branch. e.g. b7 can use variables from b4.
    /// This function is used to determine which block's variables can be inherited by merged block.
    fn find_closest_parent(&mut self, lhs: &StoredBlock, rhs: &StoredBlock) -> BasicBlockId {
        for block in &lhs.context.parent_blocks_history {
            if rhs.context.parent_blocks_history.contains(block) {
                return *block;
            }
        }

        unreachable!("Blocks are not in the same CFG.");
    }

    fn ids_to_blocks(&self, ids: &[BasicBlockId]) -> Vec<StoredBlock> {
        ids.iter().map(|id| self.stored_blocks[id].clone()).collect()
    }

    /// Returns end of the block if it has only one end or block has no children blocks
    /// e.g.
    ///     b0
    ///    ↙  ↘
    ///   b1   b2
    ///    ↘  ↙
    ///     b3
    /// b0 has only one end b3, so we return b3
    ///
    /// Returns None if block has more than one end or block has no children blocks
    /// e.g.
    ///     b0
    ///    ↙  ↘
    ///   b1   b2
    /// b0 has 2 children blocks b1 and b2, so it has 2 ends, so we return None
    ///
    /// This function is used to find end of the block for merging
    /// If block has no end, it means it has branches in the sub CFG, so we need to merge children blocks first
    fn end_of_block(&self, block_id: BasicBlockId) -> Option<BasicBlockId> {
        let block = match self.stored_blocks.get(&block_id) {
            Some(block) => block,
            None => unreachable!("Block not found in stored blocks."),
        };

        if block.context.children_blocks.is_empty() {
            return Some(block.block_id);
        }
        let mut blocks_stack = vec![block.block_id];
        let mut end_blocks = Vec::new();
        while let Some(block_id) = blocks_stack.pop() {
            let block = &self.stored_blocks[&block_id];
            let children_blocks = self.ids_to_blocks(&block.context.children_blocks);
            for child_block in children_blocks {
                if child_block.context.children_blocks.is_empty() {
                    end_blocks.push(child_block.block_id);
                } else {
                    blocks_stack.push(child_block.block_id);
                }
            }
        }
        let set_of_end_blocks = end_blocks.into_iter().collect::<HashSet<_>>();
        if set_of_end_blocks.len() == 1 {
            return set_of_end_blocks.into_iter().next();
        }

        None
    }

    /// Merges block and return ending block
    ///
    /// There are several restrictions for CFG
    /// 1) We can only have one return block;
    /// 2) Every block should have not more than two predecessors;
    /// 3) Every block must be terminated with return/jmp/jmp_if;
    /// 4) Blocks from different branches should not be merged, e.g.
    ///    ```text
    ///          b0
    ///         ↙  ↘
    ///        b1   b2
    ///       ↙  ↘    |
    ///      b3   b4  |
    ///             ↘ ↙
    ///              b5
    ///   ```  
    ///   is incorrect, because b2 and b4 are from different branches, so we cannot merge them.
    ///   
    /// so to merge blocks we need to merge every branch separately
    fn merge_one_block(&mut self, block_id: BasicBlockId) -> StoredBlock {
        let block = &self.stored_blocks[&block_id];
        log::debug!("merging block {block_id:?}");
        let block_end = self.end_of_block(block_id);
        if let Some(block_end) = block_end {
            return self.stored_blocks[&block_end].clone();
        }
        if block.context.children_blocks.len() == 1 {
            let child_block = self.stored_blocks[&block.context.children_blocks[0]].block_id;
            self.merge_one_block(child_block)
        } else if block.context.children_blocks.len() == 2 {
            let child_block_1 = self.stored_blocks[&block.context.children_blocks[0]].block_id;
            let child_block_2 = self.stored_blocks[&block.context.children_blocks[1]].block_id;
            let end_of_block_1 = self.merge_one_block(child_block_1);
            let end_of_block_2 = self.merge_one_block(child_block_2);
            self.merge_blocks(end_of_block_1, end_of_block_2)
        } else {
            unreachable!("Block {:?} has more than 2 children blocks.", block_id);
        }
    }

    /// Merges first block and returns ending block
    fn merge_main_block(&mut self) -> StoredBlock {
        let main_block = self.stored_blocks[&BasicBlockId::new(0)].clone();

        self.merge_one_block(main_block.block_id)
    }

    /// Finalizes loops in the program
    /// Terminates every loop with jmp to the loop iter block
    fn finalize_cycles(&mut self) {
        let cycle_info: Vec<_> = self.cycle_bodies_to_iters_ids.keys().cloned().collect();
        for body_id in cycle_info {
            let iter_id = self.cycle_bodies_to_iters_ids[&body_id].block_iter_id;
            log::debug!("body_id: {body_id:?}, iter_id: {iter_id:?}");
            self.switch_to_block(body_id);
            self.insert_jmp_instruction(iter_id, vec![]);
        }
    }

    /// Creates return block and terminates all blocks from current_block_queue with return
    pub(crate) fn finalize(&mut self, return_instruction_block_idx: usize) {
        // save all not-terminated blocks to stored_blocks
        self.finalize_cycles();
        self.not_terminated_blocks.push_back(self.current_block.clone());
        for block in self.not_terminated_blocks.iter() {
            self.stored_blocks.insert(block.block_id, block.clone());
        }

        // create empty return block
        let return_instruction_block = self.instruction_blocks
            [return_instruction_block_idx % self.instruction_blocks.len()]
        .clone();
        let return_block_id = self.insert_ssa_block();

        // finalize last block with jmp to return block
        let mut last_block = self.merge_main_block();
        self.switch_to_block(last_block.block_id);
        last_block.context.finalize_block_with_jmp(self.builder, return_block_id, vec![]);

        // add instructions to the return block
        self.switch_to_block(return_block_id);
        let mut return_block_context = BlockContext {
            children_blocks: vec![],
            parent_blocks_history: VecDeque::new(),
            ..last_block.context.clone()
        };
        return_block_context
            .insert_instructions(self.builder, &return_instruction_block.instructions);

        return_block_context.finalize_block_with_return(self.builder, self.return_type.clone());
    }
}
