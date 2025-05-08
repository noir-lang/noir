use crate::block_context::BlockContext;
use crate::instruction::InstructionBlock;
use crate::options::{ContextOptions, SsaBlockOptions};
use acvm::FieldElement;
use acvm::acir::native_types::Witness;
use libfuzzer_sys::arbitrary;
use libfuzzer_sys::arbitrary::Arbitrary;
use noir_ssa_fuzzer::{
    builder::{FuzzerBuilder, FuzzerBuilderError, InstructionWithOneArg, InstructionWithTwoArgs},
    config::NUMBER_OF_VARIABLES_INITIAL,
    typed_value::{TypedValue, ValueType},
};
use noirc_driver::CompiledProgram;
use noirc_evaluator::ssa::ir::basic_block::BasicBlockId;
use std::{
    cmp::{max, min},
    collections::{HashMap, HashSet, VecDeque},
    hash::{Hash, Hasher},
    mem::discriminant,
};

/// Represents set of commands for the fuzzer
///
/// After executing all commands, terminates all blocks from current_block_queue with return
#[derive(Arbitrary, Debug, Clone, Hash)]
pub(crate) enum FuzzerCommand {
    /// Adds instructions to current_block_context from stored instruction_blocks
    InsertSimpleInstructionBlock { instruction_block_idx: usize },
    /// Merges two instruction blocks, stores result in instruction_blocks
    MergeInstructionBlocks { first_block_idx: usize, second_block_idx: usize },
    /// terminates current SSA block with jmp_if_else. Creates two new SSA blocks from chosen InstructionBlocks.
    /// Switches current_block_context to then_branch.
    /// Adds else_branch to the next_block_queue. If current SSA block is already terminated, skip.
    InsertJmpIfBlock { block_then_idx: usize, block_else_idx: usize },
    /// Terminates current SSA block with jmp. Creates new SSA block from chosen InstructionBlock.
    /// Switches current_block_context to jmp_destination.
    InsertJmpBlock { block_idx: usize },
    /// Adds current SSA block to the next_block_queue. Switches context to stored in next_block_queue.
    SwitchToNextBlock,
}

#[derive(Clone)]
pub(crate) struct StoredBlock {
    context: BlockContext,
    block_id: BasicBlockId,
}

/// Main context for the fuzzer containing both ACIR and Brillig builders and their state
/// It works with indices of variables Ids, because it cannot handle Ids logic for ACIR and Brillig
pub(crate) struct FuzzerContext {
    /// ACIR builder
    acir_builder: FuzzerBuilder,
    /// Brillig builder
    brillig_builder: FuzzerBuilder,
    /// Current ACIR and Brillig blocks
    current_block: StoredBlock,
    /// Stored ACIR and Brillig blocks that are not terminated
    not_terminated_blocks: VecDeque<StoredBlock>,
    /// Instruction blocks
    instruction_blocks: Vec<InstructionBlock>,
    /// Hashmap of stored variables in blocks
    stored_variables_for_block: HashMap<BasicBlockId, HashMap<ValueType, Vec<TypedValue>>>,
    /// Hashmap of stored blocks
    stored_blocks: HashMap<BasicBlockId, StoredBlock>,
    /// Whether the program is executed in constants
    is_constant: bool,
    context_options: ContextOptions,
}

impl FuzzerContext {
    /// Creates a new fuzzer context with the given types
    /// It creates a new variable for each type and stores it in the map
    pub(crate) fn new(
        types: Vec<ValueType>,
        instruction_blocks: Vec<InstructionBlock>,
        context_options: ContextOptions,
    ) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        let mut acir_ids = HashMap::new();
        for type_ in types {
            let acir_id = acir_builder.insert_variable(type_.to_ssa_type());
            let brillig_id = brillig_builder.insert_variable(type_.to_ssa_type());
            assert_eq!(acir_id, brillig_id);
            acir_ids.entry(type_).or_insert(Vec::new()).push(acir_id);
        }

        let main_block = acir_builder.get_current_block();
        let current_block = StoredBlock {
            context: BlockContext::new(
                acir_ids.clone(),
                VecDeque::new(),
                SsaBlockOptions::from(context_options.clone()),
            ),
            block_id: main_block,
        };

        Self {
            acir_builder,
            brillig_builder,
            current_block,
            not_terminated_blocks: VecDeque::new(),
            instruction_blocks,
            stored_variables_for_block: HashMap::new(),
            stored_blocks: HashMap::new(),
            is_constant: false,
            context_options,
        }
    }

    /// Creates a new fuzzer context with the given values and inserts them as constants
    pub(crate) fn new_constant_context(
        values: Vec<impl Into<FieldElement>>,
        types: Vec<ValueType>,
        instruction_blocks: Vec<InstructionBlock>,
        context_options: ContextOptions,
    ) -> Self {
        let mut acir_builder = FuzzerBuilder::new_acir();
        let mut brillig_builder = FuzzerBuilder::new_brillig();
        let mut acir_ids = HashMap::new();
        let mut brillig_ids = HashMap::new();

        for (value, type_) in values.into_iter().zip(&types) {
            let field_element = value.into();
            acir_ids
                .entry(*type_)
                .or_insert(Vec::new())
                .push(acir_builder.insert_constant(field_element, *type_));
            brillig_ids
                .entry(*type_)
                .or_insert(Vec::new())
                .push(brillig_builder.insert_constant(field_element, *type_));
            assert_eq!(brillig_ids, acir_ids);
        }

        let main_block = acir_builder.get_current_block();
        let current_block = StoredBlock {
            context: BlockContext::new(
                acir_ids.clone(),
                VecDeque::new(),
                SsaBlockOptions::from(context_options.clone()),
            ),
            block_id: main_block,
        };

        Self {
            acir_builder,
            brillig_builder,
            current_block,
            not_terminated_blocks: VecDeque::new(),
            instruction_blocks,
            stored_variables_for_block: HashMap::new(),
            stored_blocks: HashMap::new(),
            is_constant: true,
            context_options,
        }
    }

    fn switch_to_block(&mut self, block_id: BasicBlockId) {
        self.acir_builder.switch_to_block(block_id);
        self.brillig_builder.switch_to_block(block_id);
    }

    /// Stores variables of the current block
    fn store_variables(&mut self) {
        self.stored_variables_for_block
            .insert(self.current_block.block_id, self.current_block.context.stored_values.clone());
    }

    fn process_jmp_if_command(&mut self, block_then_idx: usize, block_else_idx: usize) {
        self.store_variables();

        // find instruction block to be inserted
        let block_then_instruction_block =
            self.instruction_blocks[block_then_idx % self.instruction_blocks.len()].clone();
        let block_else_instruction_block =
            self.instruction_blocks[block_else_idx % self.instruction_blocks.len()].clone();

        // creates new blocks
        let block_then_id = self.acir_builder.insert_block();
        assert_eq!(block_then_id, self.brillig_builder.insert_block());

        let block_else_id = self.acir_builder.insert_block();
        assert_eq!(block_else_id, self.brillig_builder.insert_block());

        // creates new contexts of created blocks
        let mut parent_blocks_history = self.current_block.context.parent_blocks_history.clone();
        parent_blocks_history.push_front(self.current_block.block_id);
        let mut block_then_context = BlockContext::new(
            self.current_block.context.stored_values.clone(),
            parent_blocks_history.clone(),
            SsaBlockOptions::from(self.context_options.clone()),
        );
        let mut block_else_context = BlockContext::new(
            self.current_block.context.stored_values.clone(),
            parent_blocks_history,
            SsaBlockOptions::from(self.context_options.clone()),
        );

        // inserts instructions into created blocks
        self.switch_to_block(block_then_id);
        block_then_context.insert_instructions(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            &block_then_instruction_block.instructions,
        );

        self.switch_to_block(block_else_id);
        block_else_context.insert_instructions(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            &block_else_instruction_block.instructions,
        );

        // terminates current block with jmp_if
        self.switch_to_block(self.current_block.block_id);
        self.current_block.context.finalize_block_with_jmp_if(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            block_then_id,
            block_else_id,
        );
        self.stored_blocks.insert(self.current_block.block_id, self.current_block.clone());

        // switch context to then block and push else block to the queue
        let then_stored_block =
            StoredBlock { context: block_then_context, block_id: block_then_id };
        let else_stored_block =
            StoredBlock { context: block_else_context, block_id: block_else_id };
        self.not_terminated_blocks.push_back(else_stored_block);
        self.switch_to_block(then_stored_block.block_id);
        self.current_block = then_stored_block.clone();
    }

    fn process_jmp_block(&mut self, block_idx: usize) {
        self.store_variables();

        // find instruction block to be inserted
        let block = self.instruction_blocks[block_idx % self.instruction_blocks.len()].clone();

        // creates new block
        let destination_block_id = self.acir_builder.insert_block();
        assert_eq!(destination_block_id, self.brillig_builder.insert_block());

        // creates new context for the new block
        let mut parent_blocks_history = self.current_block.context.parent_blocks_history.clone();
        parent_blocks_history.push_front(self.current_block.block_id);
        self.switch_to_block(destination_block_id);
        let mut destination_block_context = BlockContext::new(
            self.current_block.context.stored_values.clone(),
            parent_blocks_history,
            SsaBlockOptions::from(self.context_options.clone()),
        );

        // inserts instructions into the new block
        destination_block_context.insert_instructions(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            &block.instructions,
        );

        // switches to the current block and terminates it with jmp
        self.switch_to_block(self.current_block.block_id);
        self.current_block.context.finalize_block_with_jmp(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            destination_block_id,
        );
        self.stored_blocks.insert(self.current_block.block_id, self.current_block.clone());

        // switches to the new block and updates current block
        self.switch_to_block(destination_block_id);
        self.current_block =
            StoredBlock { context: destination_block_context, block_id: destination_block_id };
    }

    pub(crate) fn process_fuzzer_command(&mut self, command: &FuzzerCommand) {
        match command {
            FuzzerCommand::InsertSimpleInstructionBlock { instruction_block_idx } => {
                let instruction_block =
                    &self.instruction_blocks[instruction_block_idx % self.instruction_blocks.len()];
                self.current_block.context.insert_instructions(
                    &mut self.acir_builder,
                    &mut self.brillig_builder,
                    &instruction_block.instructions,
                );
            }
            FuzzerCommand::MergeInstructionBlocks { first_block_idx, second_block_idx } => {
                let first_idx = first_block_idx % self.instruction_blocks.len();
                let second_idx = second_block_idx % self.instruction_blocks.len();

                let combined_instructions = self.instruction_blocks[first_idx]
                    .instructions
                    .iter()
                    .chain(&self.instruction_blocks[second_idx].instructions)
                    .cloned()
                    .collect();

                self.instruction_blocks
                    .push(InstructionBlock { instructions: combined_instructions });
            }
            FuzzerCommand::InsertJmpIfBlock { block_then_idx, block_else_idx } => {
                self.process_jmp_if_command(*block_then_idx, *block_else_idx);
            }
            FuzzerCommand::InsertJmpBlock { block_idx } => {
                self.process_jmp_block(*block_idx);
            }
            FuzzerCommand::SwitchToNextBlock => {
                self.not_terminated_blocks.push_back(self.current_block.clone());
                self.current_block = self.not_terminated_blocks.pop_front().unwrap();
                self.switch_to_block(self.current_block.block_id);
            }
        }
    }

    /// Merges two blocks into one
    /// Create empty merged_block. Terminates first_block and second_block with jmp to merged_block
    /// Returns merged_block
    fn merge_blocks(
        &mut self,
        mut first_block: StoredBlock,
        mut second_block: StoredBlock,
    ) -> StoredBlock {
        let merged_block_id = self.acir_builder.insert_block();
        assert_eq!(merged_block_id, self.brillig_builder.insert_block());

        let mut parent_blocks_history = first_block.context.parent_blocks_history.clone();
        parent_blocks_history.push_front(first_block.block_id);
        parent_blocks_history.push_front(second_block.block_id);

        let closest_parent = self.find_closest_parent(&first_block, &second_block);
        let values_block_can_use = self.stored_variables_for_block.get(&closest_parent).unwrap();

        let merged_block_context = BlockContext::new(
            values_block_can_use.clone(),
            parent_blocks_history,
            SsaBlockOptions::from(self.context_options.clone()),
        );

        self.switch_to_block(first_block.block_id);
        first_block.context.finalize_block_with_jmp(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            merged_block_id,
        );
        self.stored_blocks.insert(first_block.block_id, first_block.clone());

        self.switch_to_block(second_block.block_id);
        second_block.context.finalize_block_with_jmp(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            merged_block_id,
        );
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
    /// between b5 and b6. They both have parents history [b4, b3, b2, b1] and closest parent is b1
    /// between b7 and b8. b7 has history [b5, b6, b4, b2, b1, b0], b8 has history [b3, b1, b0], closest parent is b1
    fn find_closest_parent(&mut self, lhs: &StoredBlock, rhs: &StoredBlock) -> BasicBlockId {
        for block in &lhs.context.parent_blocks_history {
            if rhs.context.parent_blocks_history.contains(block) {
                return *block;
            }
        }

        unreachable!("Blocks are not in the same CFG. How?");
    }

    fn ids_to_blocks(&self, ids: &Vec<BasicBlockId>) -> Vec<StoredBlock> {
        ids.into_iter().map(|id| self.stored_blocks[&id].clone()).collect()
    }

    /// Checks if blocks' children blocks have only one end or block has no children blocks
    fn end_of_block(&self, block_id: BasicBlockId) -> Option<BasicBlockId> {
        let block = match self.stored_blocks.get(&block_id) {
            Some(block) => block,
            None => unreachable!("Block not found in stored blocks. How?"),
        };

        if block.context.children_blocks.len() == 0 {
            return Some(block.block_id);
        }
        let mut blocks_stack = vec![block.block_id];
        let mut end_blocks = Vec::new();
        while !blocks_stack.is_empty() {
            let block_id = blocks_stack.pop().unwrap();
            let block = &self.stored_blocks[&block_id];
            let children_blocks = self.ids_to_blocks(&block.context.children_blocks);
            for child_block in children_blocks {
                if child_block.context.children_blocks.len() == 0 {
                    end_blocks.push(child_block.block_id);
                } else {
                    blocks_stack.push(child_block.block_id);
                }
            }
        }
        let set_of_end_blocks = end_blocks.into_iter().collect::<HashSet<_>>();
        if set_of_end_blocks.len() == 1 {
            return Some(set_of_end_blocks.into_iter().next().unwrap());
        }

        None
    }

    fn merge_one_block(&mut self, block_id: BasicBlockId) -> StoredBlock {
        let block = &self.stored_blocks[&block_id];
        let block_end = self.end_of_block(block_id);
        if block_end.is_some() {
            return self.stored_blocks[&block_end.unwrap()].clone();
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

    /// We can merge block if it has only one end
    ///
    fn try_merge(&mut self) -> StoredBlock {
        let main_block = self.stored_blocks[&BasicBlockId::new(0)].clone();

        self.merge_one_block(main_block.block_id)
    }

    /// Creates return block and terminates all blocks from current_block_queue with return
    pub(crate) fn finalize(&mut self, return_instruction_block_idx: usize) {
        // Every block must have 0, 1 or 2 successors(blocks that jumped to it)
        // so we need to merge all not terminated blocks into one
        // and then terminate merged block with return

        // save all blocks to stored_blocks
        self.not_terminated_blocks.push_back(self.current_block.clone());
        for block in self.not_terminated_blocks.iter() {
            self.stored_blocks.insert(block.block_id, block.clone());
        }
        let mut last_block = self.try_merge();

        let return_instruction_block = self.instruction_blocks
            [return_instruction_block_idx % self.instruction_blocks.len()]
        .clone();
        let return_block_id = self.acir_builder.insert_block();
        assert_eq!(return_block_id, self.brillig_builder.insert_block());

        self.switch_to_block(return_block_id);
        let mut return_block_context = BlockContext::new(
            last_block.context.stored_values.clone(),
            VecDeque::new(),
            SsaBlockOptions::from(self.context_options.clone()),
        );
        return_block_context.insert_instructions(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            &return_instruction_block.instructions,
        );

        return_block_context
            .finalize_block_with_return(&mut self.acir_builder, &mut self.brillig_builder);

        self.switch_to_block(last_block.block_id);
        last_block.context.finalize_block_with_jmp(
            &mut self.acir_builder,
            &mut self.brillig_builder,
            return_block_id,
        );
    }

    /// Returns witnesses for ACIR and Brillig
    /// If program does not have any instructions, it terminated with the last witness
    /// Resulting WitnessStack of programs contains only variables and return value
    /// If we inserted some instructions, WitnessStack contains return value, so we return the last one
    /// If we are checking constant folding, the witness stack will only contain the return value, so we return Witness(0)
    pub(crate) fn get_return_witnesses(&self) -> (Witness, Witness) {
        if self.is_constant {
            (Witness(0), Witness(0))
        } else {
            (Witness(NUMBER_OF_VARIABLES_INITIAL), Witness(NUMBER_OF_VARIABLES_INITIAL))
        }
    }

    /// Returns programs for ACIR and Brillig
    pub(crate) fn get_programs(
        self,
    ) -> (Result<CompiledProgram, FuzzerBuilderError>, Result<CompiledProgram, FuzzerBuilderError>)
    {
        (self.acir_builder.compile(), self.brillig_builder.compile())
    }
}
