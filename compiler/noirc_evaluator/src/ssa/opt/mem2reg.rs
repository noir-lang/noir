//! The goal of the mem2reg SSA optimization pass is to replace any `Load` instructions to known
//! addresses with the value stored at that address, if it is also known. This pass will also remove
//! any `Store` instructions within a block that are no longer needed because no more loads occur in
//! between the Store in question and the next Store.
//!
//! The pass works as follows:
//! - Each block in each function is iterated in forward-order.
//! - The starting value of each reference in the block is the unification of the same references
//!   at the end of each direct predecessor block to the current block.
//! - At each step, the value of each reference is either Known(ValueId) or Unknown.
//! - Two reference values unify to each other if they are exactly equal, or to Unknown otherwise.
//! - If a block has no predecessors, the starting value of each reference is Unknown.
//! - Throughout this pass, aliases of each reference are also tracked.
//!   - References typically have 1 alias - themselves.
//!   - A reference with multiple aliases means we will not be able to optimize out loads if the
//!     reference is stored to. Note that this means we can still optimize out loads if these
//!     aliased references are never stored to, or the store occurs after a load.
//!   - A reference with 0 aliases means we were unable to find which reference this reference
//!     refers to. If such a reference is stored to, we must conservatively invalidate every
//!     reference in the current block.
//!
//! From there, to figure out the value of each reference at the end of block, iterate each instruction:
//! - On `Instruction::Allocate`:
//!   - Register a new reference was made with itself as its only alias
//! - On `Instruction::Load { address }`:
//!   - If `address` is known to only have a single alias (including itself) and if the value of
//!     that alias is known, replace the value of the load with the known value.
//!   - Furthermore, if the result of the load is a reference, mark the result as an alias
//!     of the reference it dereferences to (if known).
//!     - If which reference it dereferences to is not known, this load result has no aliases.
//! - On `Instruction::Store { address, value }`:
//!   - If the address of the store is known:
//!     - If the address has exactly 1 alias:
//!       - Set the value of the address to `Known(value)`.
//!     - If the address has more than 1 alias:
//!       - Set the value of every possible alias to `Unknown`.
//!     - If the address has 0 aliases:
//!       - Conservatively mark every alias in the block to `Unknown`.
//!   - If the address of the store is not known:
//!     - Conservatively mark every alias in the block to `Unknown`.
//!   - Additionally, if there were no Loads to any alias of the address between this Store and
//!     the previous Store to the same address, the previous store can be removed.
//! - On `Instruction::Call { arguments }`:
//!   - If any argument of the call is a reference, set the value of each alias of that
//!     reference to `Unknown`
//!   - Any builtin functions that may return aliases if their input also contains a
//!     reference should be tracked. Examples: `slice_push_back`, `slice_insert`, `slice_remove`, etc.
//!
//! On a terminator instruction:
//! - If the terminator is a `Jmp`:
//!   - For each reference argument of the jmp, mark the corresponding block parameter it is passed
//!     to as an alias for the jmp argument.
//!
//! Finally, if this is the only block in the function, we can remove any Stores that were not
//! referenced by the terminator instruction.
//!
//! Repeating this algorithm for each block in the function in program order should result in
//! optimizing out most known loads. However, identifying all aliases correctly has been proven
//! undecidable in general (Landi, 1992). So this pass will not always optimize out all loads
//! that could theoretically be optimized out. This pass can be performed at any time in the
//! SSA optimization pipeline, although it will be more successful the simpler the program's CFG is.
//! This pass is currently performed several times to enable other passes - most notably being
//! performed before loop unrolling to try to allow for mutable variables used for loop indices.
//!
//! As stated above, the algorithm above can sometimes miss known references.
//! This most commonly occurs in the case of loops, where we may have allocations preceding a loop that are known,
//! but the loop body's blocks are predecessors to the loop header block, causing those known allocations to be marked unknown.
//! In certain cases we may be able to remove these allocations that precede a loop.
//! For example, if a reference is not stored to again in the loop we should be able to remove that store which precedes the loop.
//!
//! To handle cases such as the one laid out above, we maintain some extra state per function,
//! that we will analyze after the initial run through all of the blocks.
//! We refer to this as the "function cleanup" and it requires having already iterated through all blocks.
//!
//! The state contains the following:
//! - For each load address we store the number of loads from a given address,
//!   the last load instruction from a given address across all blocks, and the respective block id of that instruction.
//! - A mapping of each load result to its number of uses, the load instruction that produced the given result, and the respective block id of that instruction.
//! - A set of the references and their aliases passed as an argument to a call.
//! - Maps the references which have been aliased to the instructions that aliased that reference.
//! - As we go through each instruction, if a load result has been used we increment its usage counter.
//!   Upon removing an instruction, we decrement the load result counter.
//! After analyzing all of a function's blocks we can analyze the per function state:
//! - If we find that a load result's usage counter equals zero, we can remove that load.
//! - We can then remove a store if the following conditions are met:
//!   - All loads to a given address have been removed
//!   - None of the aliases of a reference are used in any of the following:
//!     - Block parameters, function parameters, call arguments, terminator arguments
//!   - The store address is not aliased.
//! - If a store is in a return block, we can have special handling that only checks if there is a load after
//!   that store in the return block. In the case of a return block, even if there are other loads
//!   in preceding blocks we can safely remove those stores.
//! - To further catch any stores to references which are never loaded, we can count the number of stores
//!   that were removed in the previous step. If there is only a single store leftover, we can safely map
//!   the value of this final store to any loads of that store.
mod alias_set;
mod block;

use std::collections::{BTreeMap, BTreeSet};

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
        value::ValueId,
    },
    ssa_gen::Ssa,
};

use self::alias_set::AliasSet;
use self::block::{Block, Expression};

impl Ssa {
    /// Attempts to remove any load instructions that recover values that are already available in
    /// scope, and attempts to remove stores that are subsequently redundant.
    #[tracing::instrument(level = "trace", skip(self))]
    pub(crate) fn mem2reg(mut self) -> Ssa {
        for function in self.functions.values_mut() {
            let mut context = PerFunctionContext::new(function);
            context.mem2reg();
            context.remove_instructions();
            context.update_data_bus();
        }

        self
    }
}

struct PerFunctionContext<'f> {
    cfg: ControlFlowGraph,
    post_order: PostOrder,

    blocks: BTreeMap<BasicBlockId, Block>,

    inserter: FunctionInserter<'f>,

    /// Load and Store instructions that should be removed at the end of the pass.
    ///
    /// We avoid removing individual instructions as we go since removing elements
    /// from the middle of Vecs many times will be slower than a single call to `retain`.
    instructions_to_remove: HashSet<InstructionId>,

    /// Track a value's last load across all blocks.
    /// If a value is not used in anymore loads we can remove the last store to that value.
    last_loads: HashMap<ValueId, PerFuncLastLoadContext>,

    /// Track whether a load result was used across all blocks.
    load_results: HashMap<ValueId, PerFuncLoadResultContext>,

    /// Track whether a reference was passed into another entry point
    /// This is needed to determine whether we can remove a store.
    calls_reference_input: HashSet<ValueId>,

    /// Track whether a reference has been aliased, and store the respective
    /// instruction that aliased that reference.
    /// If that store has been set for removal, we can also remove this instruction.
    aliased_references: HashMap<ValueId, HashSet<InstructionId>>,

    // The index of the last load instruction in a given block
    return_block_load_locations: HashMap<(ValueId, BasicBlockId), usize>,
}

#[derive(Debug, Clone)]
struct PerFuncLastLoadContext {
    /// Reference counter that keeps track of how many times we loaded from a given address
    num_loads: u32,
    /// Last load instruction from a given address
    load_instruction: InstructionId,
    /// Block of the last load instruction
    block_id: BasicBlockId,
}

impl PerFuncLastLoadContext {
    fn new(load_instruction: InstructionId, block_id: BasicBlockId, num_loads: u32) -> Self {
        Self { num_loads, load_instruction, block_id }
    }
}

#[derive(Debug, Clone)]
struct PerFuncLoadResultContext {
    /// Reference counter that keeps track of how many times a load was used in other instructions
    uses: u32,
    /// Load instruction that produced a given load result
    load_instruction: InstructionId,
    /// Block of the load instruction that produced a given result
    block_id: BasicBlockId,
}

impl PerFuncLoadResultContext {
    fn new(load_instruction: InstructionId, block_id: BasicBlockId) -> Self {
        Self { uses: 0, load_instruction, block_id }
    }
}

impl<'f> PerFunctionContext<'f> {
    fn new(function: &'f mut Function) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_function(function);

        PerFunctionContext {
            cfg,
            post_order,
            inserter: FunctionInserter::new(function),
            blocks: BTreeMap::new(),
            instructions_to_remove: HashSet::default(),
            last_loads: HashMap::default(),
            load_results: HashMap::default(),
            calls_reference_input: HashSet::default(),
            aliased_references: HashMap::default(),
            return_block_load_locations: HashMap::default(),
        }
    }

    /// Apply the mem2reg pass to the given function.
    ///
    /// This function is expected to be the same one that the internal cfg, post_order, and
    /// dom_tree were created from.
    fn mem2reg(&mut self) {
        // Iterate each block in reverse post order = forward order
        let mut block_order = PostOrder::with_function(self.inserter.function).into_vec();
        block_order.reverse();

        for block in block_order {
            let references = self.find_starting_references(block);
            self.analyze_block(block, references);
        }

        self.cleanup_function();
    }

    /// The value of each reference at the start of the given block is the unification
    /// of the value of the same reference at the end of its predecessor blocks.
    fn find_starting_references(&mut self, block: BasicBlockId) -> Block {
        let mut predecessors = self.cfg.predecessors(block);

        if let Some(first_predecessor) = predecessors.next() {
            let mut first = self.blocks.get(&first_predecessor).cloned().unwrap_or_default();
            first.last_stores.clear();

            // Note that we have to start folding with the first block as the accumulator.
            // If we started with an empty block, an empty block union'd with any other block
            // is always also empty so we'd never be able to track any references across blocks.
            predecessors.fold(first, |block, predecessor| {
                let predecessor = self.blocks.entry(predecessor).or_default();
                block.unify(predecessor)
            })
        } else {
            Block::default()
        }
    }

    /// Analyze a block with the given starting reference values.
    ///
    /// This will remove any known loads in the block and track the value of references
    /// as they are stored to. When this function is finished, the value of each reference
    /// at the end of this block will be remembered in `self.blocks`.
    fn analyze_block(&mut self, block: BasicBlockId, mut references: Block) {
        let instructions = self.inserter.function.dfg[block].take_instructions();

        for instruction in instructions {
            self.analyze_instruction(block, &mut references, instruction);
        }

        self.handle_terminator(block, &mut references);

        // If there's only 1 block in the function total, we can remove any remaining last stores
        // as well. We can't do this if there are multiple blocks since subsequent blocks may
        // reference these stores.
        if self.post_order.as_slice().len() == 1 {
            self.remove_stores_that_do_not_alias_parameters(&references);
        }

        self.blocks.insert(block, references);
    }

    /// Add all instructions in `last_stores` to `self.instructions_to_remove` which do not
    /// possibly alias any parameters of the given function.
    fn remove_stores_that_do_not_alias_parameters(&mut self, references: &Block) {
        let parameters = self.inserter.function.parameters().iter();
        let reference_parameters = parameters
            .filter(|param| self.inserter.function.dfg.value_is_reference(**param))
            .collect::<BTreeSet<_>>();

        // Must collect here as we are immutably borrowing `self` to fetch the reference parameters
        let mut values_to_reduce_counts = Vec::new();
        for (allocation, instruction) in &references.last_stores {
            if let Some(expression) = references.expressions.get(allocation) {
                if let Some(aliases) = references.aliases.get(expression) {
                    let allocation_aliases_parameter =
                        aliases.any(|alias| reference_parameters.contains(&alias));

                    // If `allocation_aliases_parameter` is known to be false
                    if allocation_aliases_parameter == Some(false) {
                        self.instructions_to_remove.insert(*instruction);
                        values_to_reduce_counts.push(*allocation);
                    }
                }
            }
        }

        for value in values_to_reduce_counts {
            self.reduce_load_result_count(value);
        }
    }

    fn increase_load_ref_counts(&mut self, value: ValueId) {
        if let Some(context) = self.load_results.get_mut(&value) {
            context.uses += 1;
        }
        let array_const = self.inserter.function.dfg.get_array_constant(value);
        if let Some((values, _)) = array_const {
            for array_value in values {
                self.increase_load_ref_counts(array_value);
            }
        }
    }

    fn analyze_instruction(
        &mut self,
        block_id: BasicBlockId,
        references: &mut Block,
        mut instruction: InstructionId,
    ) {
        // If the instruction was simplified and optimized out of the program we shouldn't analyze
        // it. Analyzing it could make tracking aliases less accurate if it is e.g. an ArrayGet
        // call that used to hold references but has since been optimized out to a known result.
        if let Some(new_id) = self.inserter.push_instruction(instruction, block_id) {
            instruction = new_id;
        } else {
            return;
        }

        let mut collect_values = Vec::new();
        // Track whether any load results were used in the instruction
        self.inserter.function.dfg[instruction].for_each_value(|value| {
            collect_values.push(value);
        });

        for value in collect_values {
            self.increase_load_ref_counts(value);
        }

        match &self.inserter.function.dfg[instruction] {
            Instruction::Load { address } => {
                let address = self.inserter.function.dfg.resolve(*address);

                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.remember_dereference(self.inserter.function, address, result);

                // If the load is known, replace it with the known value and remove the load
                if let Some(value) = references.get_known_value(address) {
                    self.inserter.map_value(result, value);
                    self.instructions_to_remove.insert(instruction);
                } else {
                    references.mark_value_used(address, self.inserter.function);

                    let expression =
                        references.expressions.entry(result).or_insert(Expression::Other(result));
                    // Make sure this load result is marked an alias to itself
                    if let Some(aliases) = references.aliases.get_mut(expression) {
                        // If we have an alias set, add to the set
                        aliases.insert(result);
                    } else {
                        // Otherwise, create a new alias set containing just the load result
                        references
                            .aliases
                            .insert(Expression::Other(result), AliasSet::known(result));
                    }
                    // Mark that we know a load result is equivalent to the address of a load.
                    references.set_known_value(result, address);

                    self.load_results
                        .insert(result, PerFuncLoadResultContext::new(instruction, block_id));

                    let num_loads =
                        self.last_loads.get(&address).map_or(1, |context| context.num_loads + 1);
                    let last_load = PerFuncLastLoadContext::new(instruction, block_id, num_loads);
                    self.last_loads.insert(address, last_load);

                    // If we are in a return block we want to save the last location of a load
                    let terminator = self.inserter.function.dfg[block_id].unwrap_terminator();
                    let is_return = matches!(terminator, TerminatorInstruction::Return { .. });
                    if is_return {
                        let instruction_index =
                            self.inserter.function.dfg[block_id].instructions().len();
                        self.return_block_load_locations
                            .insert((address, block_id), instruction_index);
                    }
                }
            }
            Instruction::Store { address, value } => {
                let address = self.inserter.function.dfg.resolve(*address);
                let value = self.inserter.function.dfg.resolve(*value);

                self.check_array_aliasing(references, value);

                // If there was another store to this address without any (unremoved) loads or
                // function calls in-between, we can remove the previous store.
                if let Some(last_store) = references.last_stores.get(&address) {
                    self.instructions_to_remove.insert(*last_store);
                    let Instruction::Store { address, value } =
                        self.inserter.function.dfg[*last_store]
                    else {
                        panic!("Should have a store instruction here");
                    };
                    self.reduce_load_result_count(address);
                    self.reduce_load_result_count(value);
                }

                let known_value = references.get_known_value(value);
                if let Some(known_value) = known_value {
                    let known_value_is_address = known_value == address;
                    if known_value_is_address {
                        self.instructions_to_remove.insert(instruction);
                        self.reduce_load_result_count(address);
                        self.reduce_load_result_count(value);
                    } else {
                        references.last_stores.insert(address, instruction);
                    }
                } else {
                    references.last_stores.insert(address, instruction);
                }

                if self.inserter.function.dfg.value_is_reference(value) {
                    if let Some(expression) = references.expressions.get(&value) {
                        if let Some(aliases) = references.aliases.get(expression) {
                            aliases.for_each(|alias| {
                                self.aliased_references
                                    .entry(alias)
                                    .or_default()
                                    .insert(instruction);
                            });
                        }
                    }
                }

                references.set_known_value(address, value);
            }
            Instruction::Allocate => {
                // Register the new reference
                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.expressions.insert(result, Expression::Other(result));
                references.aliases.insert(Expression::Other(result), AliasSet::known(result));
            }
            Instruction::ArrayGet { array, .. } => {
                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.mark_value_used(*array, self.inserter.function);

                if self.inserter.function.dfg.value_is_reference(result) {
                    let array = self.inserter.function.dfg.resolve(*array);
                    let expression = Expression::ArrayElement(Box::new(Expression::Other(array)));

                    if let Some(aliases) = references.aliases.get_mut(&expression) {
                        aliases.insert(result);
                    }
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                references.mark_value_used(*array, self.inserter.function);
                let element_type = self.inserter.function.dfg.type_of_value(*value);

                if Self::contains_references(&element_type) {
                    let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                    let array = self.inserter.function.dfg.resolve(*array);

                    let expression = Expression::ArrayElement(Box::new(Expression::Other(array)));

                    let mut aliases = if let Some(aliases) = references.aliases.get_mut(&expression)
                    {
                        aliases.clone()
                    } else if let Some((elements, _)) =
                        self.inserter.function.dfg.get_array_constant(array)
                    {
                        let aliases = references.collect_all_aliases(elements);
                        self.set_aliases(references, array, aliases.clone());
                        aliases
                    } else {
                        AliasSet::unknown()
                    };

                    aliases.unify(&references.get_aliases_for_value(*value));

                    references.expressions.insert(result, expression.clone());
                    references.aliases.insert(expression, aliases);
                }
            }
            Instruction::Call { arguments, .. } => {
                for arg in arguments {
                    if self.inserter.function.dfg.value_is_reference(*arg) {
                        if let Some(expression) = references.expressions.get(arg) {
                            if let Some(aliases) = references.aliases.get(expression) {
                                aliases.for_each(|alias| {
                                    self.calls_reference_input.insert(alias);
                                });
                            }
                        }
                    }
                }
                self.mark_all_unknown(arguments, references);
            }
            _ => (),
        }
    }

    fn check_array_aliasing(&self, references: &mut Block, array: ValueId) {
        if let Some((elements, typ)) = self.inserter.function.dfg.get_array_constant(array) {
            if Self::contains_references(&typ) {
                // TODO: Check if type directly holds references or holds arrays that hold references
                let expr = Expression::ArrayElement(Box::new(Expression::Other(array)));
                references.expressions.insert(array, expr.clone());
                let aliases = references.aliases.entry(expr).or_default();

                for element in elements {
                    aliases.insert(element);
                }
            }
        }
    }

    fn contains_references(typ: &Type) -> bool {
        match typ {
            Type::Numeric(_) => false,
            Type::Function => false,
            Type::Reference(_) => true,
            Type::Array(elements, _) | Type::Slice(elements) => {
                elements.iter().any(Self::contains_references)
            }
        }
    }

    fn set_aliases(&self, references: &mut Block, address: ValueId, new_aliases: AliasSet) {
        let expression =
            references.expressions.entry(address).or_insert(Expression::Other(address));
        let aliases = references.aliases.entry(expression.clone()).or_default();
        *aliases = new_aliases;
    }

    fn mark_all_unknown(&self, values: &[ValueId], references: &mut Block) {
        for value in values {
            if self.inserter.function.dfg.value_is_reference(*value) {
                let value = self.inserter.function.dfg.resolve(*value);
                references.set_unknown(value);
                references.mark_value_used(value, self.inserter.function);
            }
        }
    }

    /// Remove any instructions in `self.instructions_to_remove` from the current function.
    /// This is expected to contain any loads which were replaced and any stores which are
    /// no longer needed.
    fn remove_instructions(&mut self) {
        // The order we iterate blocks in is not important
        for block in self.post_order.as_slice() {
            self.inserter.function.dfg[*block]
                .instructions_mut()
                .retain(|instruction| !self.instructions_to_remove.contains(instruction));
        }
    }

    fn update_data_bus(&mut self) {
        self.inserter.map_data_bus_in_place();
    }

    fn handle_terminator(&mut self, block: BasicBlockId, references: &mut Block) {
        self.inserter.map_terminator_in_place(block);

        let terminator: &TerminatorInstruction =
            self.inserter.function.dfg[block].unwrap_terminator();

        let mut collect_values = Vec::new();
        terminator.for_each_value(|value| {
            collect_values.push(value);
        });

        let terminator = terminator.clone();
        for value in collect_values.iter() {
            self.increase_load_ref_counts(*value);
        }

        match &terminator {
            TerminatorInstruction::JmpIf { .. } => (), // Nothing to do
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                let destination_parameters = self.inserter.function.dfg[*destination].parameters();
                assert_eq!(destination_parameters.len(), arguments.len());

                // Add an alias for each reference parameter
                for (parameter, argument) in destination_parameters.iter().zip(arguments) {
                    if self.inserter.function.dfg.value_is_reference(*parameter) {
                        let argument = self.inserter.function.dfg.resolve(*argument);

                        if let Some(expression) = references.expressions.get(&argument) {
                            if let Some(aliases) = references.aliases.get_mut(expression) {
                                // The argument reference is possibly aliased by this block parameter
                                aliases.insert(*parameter);
                            }
                        }
                    }
                }
            }
            TerminatorInstruction::Return { return_values, .. } => {
                // Removing all `last_stores` for each returned reference is more important here
                // than setting them all to ReferenceValue::Unknown since no other block should
                // have a block with a Return terminator as a predecessor anyway.
                self.mark_all_unknown(return_values, references);
            }
        }
    }

    fn reduce_load_result_count(&mut self, value: ValueId) {
        if let Some(context) = self.load_results.get_mut(&value) {
            // TODO this was saturating https://github.com/noir-lang/noir/issues/6124
            context.uses = context.uses.wrapping_sub(1);
        }
    }

    fn recursively_add_values(&self, value: ValueId, set: &mut HashSet<ValueId>) {
        set.insert(value);
        if let Some((elements, _)) = self.inserter.function.dfg.get_array_constant(value) {
            for array_element in elements {
                self.recursively_add_values(array_element, set);
            }
        }
    }

    /// The mem2reg pass is sometimes unable to determine certain known values
    /// when iterating over a function's block in reverse post order.
    /// We collect state about any final loads and stores to a given address during the initial mem2reg pass.
    /// We can then utilize this state to clean up any loads and stores that may have been missed.
    fn cleanup_function(&mut self) {
        // Removing remaining unused loads during mem2reg can help expose removable stores that the initial
        // mem2reg pass deemed we could not remove due to the existence of those unused loads.
        let removed_loads = self.remove_unused_loads();
        let remaining_last_stores = self.remove_unloaded_last_stores(&removed_loads);
        let stores_were_removed =
            self.remove_remaining_last_stores(&removed_loads, &remaining_last_stores);

        // When removing some last loads with the last stores we will map the load result to the store value.
        // We need to then map all the instructions again as we do not know which instructions are reliant on the load result.
        if stores_were_removed {
            let mut block_order = PostOrder::with_function(self.inserter.function).into_vec();
            block_order.reverse();
            for block in block_order {
                let instructions = self.inserter.function.dfg[block].take_instructions();
                for instruction in instructions {
                    if !self.instructions_to_remove.contains(&instruction) {
                        self.inserter.push_instruction(instruction, block);
                    }
                }
                self.inserter.map_terminator_in_place(block);
            }
        }
    }

    /// Cleanup remaining loads across the entire function
    /// Remove any loads whose reference counter is zero.
    /// Returns a map of the removed load address to the number of load instructions removed for that address
    fn remove_unused_loads(&mut self) -> HashMap<ValueId, u32> {
        let mut removed_loads = HashMap::default();
        for (_, PerFuncLoadResultContext { uses, load_instruction, block_id, .. }) in
            self.load_results.iter()
        {
            let Instruction::Load { address } = self.inserter.function.dfg[*load_instruction]
            else {
                unreachable!("Should only have a load instruction here");
            };
            // If the load result's counter is equal to zero we can safely remove that load instruction.
            if *uses == 0 {
                self.return_block_load_locations.remove(&(address, *block_id));

                removed_loads.entry(address).and_modify(|counter| *counter += 1).or_insert(1);
                self.instructions_to_remove.insert(*load_instruction);
            }
        }
        removed_loads
    }

    fn recursively_check_address_in_terminator(
        &self,
        return_value: ValueId,
        store_address: ValueId,
        is_return_value: &mut bool,
    ) {
        *is_return_value = return_value == store_address || *is_return_value;
        let array_const = self.inserter.function.dfg.get_array_constant(return_value);
        if let Some((values, _)) = array_const {
            for array_value in values {
                self.recursively_check_address_in_terminator(
                    array_value,
                    store_address,
                    is_return_value,
                );
            }
        }
    }

    /// Cleanup remaining stores across the entire function.
    /// If we never load from an address within a function we can remove all stores to that address.
    /// This rule does not apply to reference parameters, which we must also check for before removing these stores.
    /// Returns a map of any remaining stores which may still have loads in use.
    fn remove_unloaded_last_stores(
        &mut self,
        removed_loads: &HashMap<ValueId, u32>,
    ) -> HashMap<ValueId, (InstructionId, u32)> {
        let mut all_terminator_values = HashSet::default();
        let mut per_func_block_params: HashSet<ValueId> = HashSet::default();
        for (block_id, _) in self.blocks.iter() {
            let block_params = self.inserter.function.dfg.block_parameters(*block_id);
            per_func_block_params.extend(block_params.iter());

            let terminator = self.inserter.function.dfg[*block_id].unwrap_terminator();
            terminator.for_each_value(|value| {
                self.recursively_add_values(value, &mut all_terminator_values);
            });
        }

        let mut remaining_last_stores: HashMap<ValueId, (InstructionId, u32)> = HashMap::default();
        for (block_id, block) in self.blocks.iter() {
            for (store_address, store_instruction) in block.last_stores.iter() {
                if self.instructions_to_remove.contains(store_instruction) {
                    continue;
                }

                let all_loads_removed = self.all_loads_removed_for_address(
                    store_address,
                    *store_instruction,
                    *block_id,
                    removed_loads,
                );

                let store_alias_used = self.is_store_alias_used(
                    store_address,
                    block,
                    &all_terminator_values,
                    &per_func_block_params,
                );

                if all_loads_removed && !store_alias_used {
                    self.instructions_to_remove.insert(*store_instruction);
                    if let Some((_, counter)) = remaining_last_stores.get_mut(store_address) {
                        // TODO this was saturating https://github.com/noir-lang/noir/issues/6124
                        *counter = counter.wrapping_sub(1);
                    }
                } else if let Some((_, counter)) = remaining_last_stores.get_mut(store_address) {
                    *counter += 1;
                } else {
                    remaining_last_stores.insert(*store_address, (*store_instruction, 1));
                }
            }
        }
        remaining_last_stores
    }

    fn all_loads_removed_for_address(
        &self,
        store_address: &ValueId,
        store_instruction: InstructionId,
        block_id: BasicBlockId,
        removed_loads: &HashMap<ValueId, u32>,
    ) -> bool {
        let terminator = self.inserter.function.dfg[block_id].unwrap_terminator();
        let is_return = matches!(terminator, TerminatorInstruction::Return { .. });
        // Determine whether any loads that reference this store address
        // have been removed while cleaning up unused loads.
        if is_return {
            // If we are in a return terminator, and the last loads of a reference
            // come before a store to that reference, we can safely remove that store.
            let store_after_load = if let Some(max_load_index) =
                self.return_block_load_locations.get(&(*store_address, block_id))
            {
                let store_index = self.inserter.function.dfg[block_id]
                    .instructions()
                    .iter()
                    .position(|id| *id == store_instruction)
                    .expect("Store instruction should exist in the return block");
                store_index > *max_load_index
            } else {
                // Otherwise there is no load in this block
                true
            };
            store_after_load
        } else if let (Some(context), Some(loads_removed_counter)) =
            (self.last_loads.get(store_address), removed_loads.get(store_address))
        {
            // `last_loads` contains the total number of loads for a given load address
            // If the number of removed loads for a given address is equal to the total number of loads for that address,
            // we know we can safely remove any stores to that load address.
            context.num_loads == *loads_removed_counter
        } else {
            self.last_loads.get(store_address).is_none()
        }
    }

    // Extra checks on where a reference can be used aside a load instruction.
    // Even if all loads to a reference have been removed we need to make sure that
    // an allocation did not come from an entry point or was passed to an entry point.
    fn is_store_alias_used(
        &self,
        store_address: &ValueId,
        block: &Block,
        all_terminator_values: &HashSet<ValueId>,
        per_func_block_params: &HashSet<ValueId>,
    ) -> bool {
        let func_params = self.inserter.function.parameters();
        let reference_parameters = func_params
            .iter()
            .filter(|param| self.inserter.function.dfg.value_is_reference(**param))
            .collect::<BTreeSet<_>>();

        let mut store_alias_used = false;
        if let Some(expression) = block.expressions.get(store_address) {
            if let Some(aliases) = block.aliases.get(expression) {
                let allocation_aliases_parameter =
                    aliases.any(|alias| reference_parameters.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    store_alias_used = true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| per_func_block_params.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    store_alias_used = true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| self.calls_reference_input.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    store_alias_used = true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| all_terminator_values.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    store_alias_used = true;
                }

                let allocation_aliases_parameter = aliases.any(|alias| {
                    if let Some(alias_instructions) = self.aliased_references.get(&alias) {
                        self.instructions_to_remove.is_disjoint(alias_instructions)
                    } else {
                        false
                    }
                });
                if allocation_aliases_parameter == Some(true) {
                    store_alias_used = true;
                }
            }
        }

        store_alias_used
    }

    /// Check if any remaining last stores are only used in a single load
    /// Returns true if any stores were removed.
    fn remove_remaining_last_stores(
        &mut self,
        removed_loads: &HashMap<ValueId, u32>,
        remaining_last_stores: &HashMap<ValueId, (InstructionId, u32)>,
    ) -> bool {
        let mut stores_were_removed = false;
        // Filter out any still in use load results and any load results that do not contain addresses from the remaining last stores
        self.load_results.retain(|_, PerFuncLoadResultContext { load_instruction, uses, .. }| {
            let Instruction::Load { address } = self.inserter.function.dfg[*load_instruction]
            else {
                unreachable!("Should only have a load instruction here");
            };
            remaining_last_stores.contains_key(&address) && *uses > 0
        });

        for (store_address, (store_instruction, store_counter)) in remaining_last_stores {
            let Instruction::Store { value, .. } = self.inserter.function.dfg[*store_instruction]
            else {
                unreachable!("Should only have a store instruction");
            };

            if let (Some(context), Some(loads_removed_counter)) =
                (self.last_loads.get(store_address), removed_loads.get(store_address))
            {
                assert!(
                    context.num_loads >= *loads_removed_counter,
                    "The number of loads removed should not be more than all loads"
                );
            }

            // We only want to remove last stores referencing a single address.
            if *store_counter != 0 {
                continue;
            }

            self.instructions_to_remove.insert(*store_instruction);

            // Map any remaining load results to the value from the removed store
            for (result, context) in self.load_results.iter() {
                let Instruction::Load { address } =
                    self.inserter.function.dfg[context.load_instruction]
                else {
                    unreachable!("Should only have a load instruction here");
                };
                if address != *store_address {
                    continue;
                }

                // Map the load result to its respective store value
                // We will have to map all instructions following this method
                // as we do not know what instructions depend upon this result
                self.inserter.map_value(*result, value);
                self.instructions_to_remove.insert(context.load_instruction);

                stores_were_removed = true;
            }
        }
        stores_were_removed
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use acvm::{acir::AcirField, FieldElement};
    use im::vector;

    use crate::ssa::{
        function_builder::FunctionBuilder,
        ir::{
            basic_block::BasicBlockId,
            dfg::DataFlowGraph,
            instruction::{BinaryOp, Instruction, Intrinsic, TerminatorInstruction},
            map::Id,
            types::Type,
        },
    };

    #[test]
    fn test_simple() {
        // fn func() {
        //   b0():
        //     v0 = allocate
        //     store [Field 1, Field 2] in v0
        //     v1 = load v0
        //     v2 = array_get v1, index 1
        //     return v2
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::Array(Arc::new(vec![Type::field()]), 2));
        let one = builder.field_constant(FieldElement::one());
        let two = builder.field_constant(FieldElement::one());

        let element_type = Arc::new(vec![Type::field()]);
        let array_type = Type::Array(element_type, 2);
        let array = builder.array_constant(vector![one, two], array_type.clone());

        builder.insert_store(v0, array);
        let v1 = builder.insert_load(v0, array_type);
        let v2 = builder.insert_array_get(v1, one, Type::field());
        builder.terminate_with_return(vec![v2]);

        let ssa = builder.finish().mem2reg().fold_constants();

        let func = ssa.main();
        let block_id = func.entry_block();

        assert_eq!(count_loads(block_id, &func.dfg), 0);
        assert_eq!(count_stores(block_id, &func.dfg), 0);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values, .. } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[two]);
    }

    #[test]
    fn test_simple_with_call() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     v1 = load v0
        //     call f0(v0)
        //     return v1
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::field());
        let one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, one);
        let v1 = builder.insert_load(v0, Type::field());
        let f0 = builder.import_intrinsic_id(Intrinsic::AssertConstant);
        builder.insert_call(f0, vec![v0], vec![]);
        builder.terminate_with_return(vec![v1]);

        let ssa = builder.finish().mem2reg();

        let func = ssa.main();
        let block_id = func.entry_block();

        assert_eq!(count_loads(block_id, &func.dfg), 0);
        assert_eq!(count_stores(block_id, &func.dfg), 1);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values, .. } => return_values.first().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(func.dfg[*ret_val_id], func.dfg[one]);
    }

    #[test]
    fn test_simple_with_return() {
        // fn func {
        //   b0():
        //     v0 = allocate
        //     store v0, Field 1
        //     return v0
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::field());
        let const_one = builder.field_constant(FieldElement::one());
        builder.insert_store(v0, const_one);
        builder.terminate_with_return(vec![v0]);

        let ssa = builder.finish().mem2reg();

        let func = ssa.main();
        let block_id = func.entry_block();

        // Store is needed by the return value, and can't be removed
        assert_eq!(count_stores(block_id, &func.dfg), 1);
        let instructions = func.dfg[block_id].instructions();
        assert_eq!(instructions.len(), 2);

        let ret_val_id = match func.dfg[block_id].terminator().unwrap() {
            TerminatorInstruction::Return { return_values, .. } => *return_values.first().unwrap(),
            _ => unreachable!(),
        };

        // Since the mem2reg pass simplifies as it goes, the id of the allocate instruction result
        // is most likely no longer v0. We have to retrieve the new id here.
        let allocate_id = func.dfg.instruction_results(instructions[0])[0];
        assert_eq!(ret_val_id, allocate_id);
    }

    fn count_stores(block: BasicBlockId, dfg: &DataFlowGraph) -> usize {
        dfg[block]
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(dfg[**instruction_id], Instruction::Store { .. }))
            .count()
    }

    fn count_loads(block: BasicBlockId, dfg: &DataFlowGraph) -> usize {
        dfg[block]
            .instructions()
            .iter()
            .filter(|instruction_id| matches!(dfg[**instruction_id], Instruction::Load { .. }))
            .count()
    }

    // Test that loads across multiple blocks are removed
    #[test]
    fn multiple_blocks() {
        // fn main {
        //   b0():
        //     v0 = allocate
        //     store Field 5 in v0
        //     v1 = load v0
        //     jmp b1(v1):
        //   b1(v2: Field):
        //     v3 = load v0
        //     store Field 6 in v0
        //     v4 = load v0
        //     return v2, v3, v4
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());

        let five = builder.field_constant(5u128);
        builder.insert_store(v0, five);

        let v1 = builder.insert_load(v0, Type::field());
        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![v1]);

        builder.switch_to_block(b1);
        let v2 = builder.add_block_parameter(b1, Type::field());
        let v3 = builder.insert_load(v0, Type::field());

        let six = builder.field_constant(6u128);
        builder.insert_store(v0, six);
        let v4 = builder.insert_load(v0, Type::field());

        builder.terminate_with_return(vec![v2, v3, v4]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 2);

        // Expected result:
        // acir fn main f0 {
        //   b0():
        //     v7 = allocate
        //     jmp b1(Field 5)
        //   b1(v3: Field):
        //     return v3, Field 5, Field 6
        // }
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // The loads should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);

        // All stores are removed as there are no loads to the values being stored anywhere in the function.
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_stores(b1, &main.dfg), 0);

        // The jmp to b1 should also be a constant 5 now
        match main.dfg[main.entry_block()].terminator() {
            Some(TerminatorInstruction::Jmp { arguments, .. }) => {
                assert_eq!(arguments.len(), 1);
                let argument =
                    main.dfg.get_numeric_constant(arguments[0]).expect("Expected constant value");
                assert_eq!(argument.to_u128(), 5);
            }
            _ => unreachable!(),
        };
    }

    // Test that a load in a predecessor block has been removed if the value
    // is later stored in a successor block
    #[test]
    fn load_aliases_in_predecessor_block() {
        // fn main {
        //     b0():
        //       v0 = allocate
        //       store Field 0 at v0
        //       v2 = allocate
        //       store v0 at v2
        //       v3 = load v2
        //       v4 = load v2
        //       jmp b1()
        //     b1():
        //       store Field 1 at v3
        //       store Field 2 at v4
        //       v7 = load v3
        //       v8 = eq v7, Field 2
        //       return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());

        let zero = builder.field_constant(0u128);
        builder.insert_store(v0, zero);

        let v2 = builder.insert_allocate(Type::Reference(Arc::new(Type::field())));
        builder.insert_store(v2, v0);

        let v3 = builder.insert_load(v2, Type::field());
        let v4 = builder.insert_load(v2, Type::field());
        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![]);

        builder.switch_to_block(b1);

        let one = builder.field_constant(1u128);
        builder.insert_store(v3, one);

        let two = builder.field_constant(2u128);
        builder.insert_store(v4, two);

        let v8 = builder.insert_load(v3, Type::field());
        let _ = builder.insert_binary(v8, BinaryOp::Eq, two);

        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();
        assert_eq!(ssa.main().reachable_blocks().len(), 2);

        // Expected result:
        // acir fn main f0 {
        //   b0():
        //     v9 = allocate
        //     v10 = allocate
        //     jmp b1()
        //   b1():
        //     return
        // }
        let ssa = ssa.mem2reg();
        println!("{}", ssa);
        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 2);

        // All loads should be removed
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);

        // All stores should be removed.
        // The first store in b1 is removed since there is another store to the same reference
        // in the same block, and the store is not needed before the later store.
        // The rest of the stores are also removed as no loads are done within any blocks
        // to the stored values.
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_stores(b1, &main.dfg), 0);

        let b1_instructions = main.dfg[b1].instructions();

        // We expect the last eq to be optimized out
        assert_eq!(b1_instructions.len(), 0);
    }

    #[test]
    fn remove_unused_loads_and_stores() {
        // acir(inline) fn main f0 {
        //     b0():
        //       v0 = allocate
        //       store Field 1 at v0
        //       v2 = allocate
        //       store Field 1 at v2
        //       v4 = allocate
        //       store u1 0 at v4
        //       v5 = allocate
        //       store u1 0 at v5
        //       v6 = allocate
        //       store u1 0 at v6
        //       jmp b1(u1 0)
        //     b1(v7: u32):
        //       v9 = eq v7, u32 0
        //       jmpif v9 then: b3, else: b2
        //     b3():
        //       v20 = load v0
        //       v21 = load v2
        //       v22 = load v4
        //       v23 = load v5
        //       v24 = load v6
        //       constrain v20 == Field 1
        //       v25 = eq v21, Field 1
        //       constrain v21 == Field 1
        //       v26 = eq v7, u32 0
        //       jmp b1(v26)
        //     b2():
        //       v10 = load v0
        //       v11 = load v2
        //       v12 = load v4
        //       v13 = load v5
        //       v14 = load v6
        //       store Field 1 at v0
        //       store Field 1 at v2
        //       store v12 at v4
        //       store v13 at v5
        //       store v14 at v6
        //       v15 = load v0
        //       v16 = load v2
        //       v17 = load v4
        //       v18 = load v5
        //       v19 = load v6
        //       constrain v15 == Field 1
        //       return v16
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());
        let one = builder.numeric_constant(1u128, Type::field());
        builder.insert_store(v0, one);

        let v2 = builder.insert_allocate(Type::field());
        builder.insert_store(v2, one);

        let zero_bool = builder.numeric_constant(0u128, Type::bool());
        let v4 = builder.insert_allocate(Type::bool());
        builder.insert_store(v4, zero_bool);

        let v6 = builder.insert_allocate(Type::bool());
        builder.insert_store(v6, zero_bool);

        let v8 = builder.insert_allocate(Type::bool());
        builder.insert_store(v8, zero_bool);

        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![zero_bool]);

        builder.switch_to_block(b1);

        let v7 = builder.add_block_parameter(b1, Type::unsigned(32));
        let zero_u32 = builder.numeric_constant(0u128, Type::unsigned(32));
        let is_zero = builder.insert_binary(v7, BinaryOp::Eq, zero_u32);

        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        builder.terminate_with_jmpif(is_zero, b3, b2);

        builder.switch_to_block(b2);

        let _ = builder.insert_load(v0, Type::field());
        let _ = builder.insert_load(v2, Type::field());
        let v12 = builder.insert_load(v4, Type::bool());
        let v13 = builder.insert_load(v6, Type::bool());
        let v14 = builder.insert_load(v8, Type::bool());

        builder.insert_store(v0, one);
        builder.insert_store(v2, one);
        builder.insert_store(v4, v12);
        builder.insert_store(v6, v13);
        builder.insert_store(v8, v14);

        let v15 = builder.insert_load(v0, Type::field());
        // Insert unused loads
        let v16 = builder.insert_load(v2, Type::field());
        let _ = builder.insert_load(v4, Type::bool());
        let _ = builder.insert_load(v6, Type::bool());
        let _ = builder.insert_load(v8, Type::bool());

        builder.insert_constrain(v15, one, None);
        builder.terminate_with_return(vec![v16]);

        builder.switch_to_block(b3);

        let v26 = builder.insert_load(v0, Type::field());
        // Insert unused loads
        let v27 = builder.insert_load(v2, Type::field());
        let _ = builder.insert_load(v4, Type::bool());
        let _ = builder.insert_load(v6, Type::bool());
        let _ = builder.insert_load(v8, Type::bool());

        builder.insert_constrain(v26, one, None);
        let _ = builder.insert_binary(v27, BinaryOp::Eq, one);
        builder.insert_constrain(v27, one, None);
        let one_u32 = builder.numeric_constant(0u128, Type::unsigned(32));
        let plus_one = builder.insert_binary(v7, BinaryOp::Eq, one_u32);
        builder.terminate_with_jmp(b1, vec![plus_one]);

        let ssa = builder.finish();

        // Expected result:
        // acir(inline) fn main f0 {
        //     b0():
        //       v27 = allocate
        //       v28 = allocate
        //       v29 = allocate
        //       v30 = allocate
        //       v31 = allocate
        //       jmp b1(u1 0)
        //     b1(v7: u32):
        //       v32 = eq v7, u32 0
        //       jmpif v32 then: b3, else: b2
        //     b3():
        //       v49 = eq v7, u32 0
        //       jmp b1(v49)
        //     b2():
        //       return Field 1
        //   }
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 4);

        // All loads should be removed
        assert_eq!(count_loads(b2, &main.dfg), 0);
        assert_eq!(count_loads(b3, &main.dfg), 0);

        // All stores should be removed
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_stores(b2, &main.dfg), 0);
        // Should only have one instruction in b3
        assert_eq!(main.dfg[b3].instructions().len(), 1);
    }

    #[test]
    fn keep_store_to_alias_in_loop_block() {
        // This test makes sure the instruction `store Field 2 at v5` in b2 remains after mem2reg.
        // Although the only instruction on v5 is a lone store without any loads,
        // v5 is an alias of the reference v0 which is stored in v2.
        // This test makes sure that we are not inadvertently removing stores to aliases across blocks.
        //
        // acir(inline) fn main f0 {
        //     b0():
        //       v0 = allocate
        //       store Field 0 at v0
        //       v2 = allocate
        //       store v0 at v2
        //       jmp b1(Field 0)
        //     b1(v3: Field):
        //       v4 = eq v3, Field 0
        //       jmpif v4 then: b2, else: b3
        //     b2():
        //       v5 = load v2
        //       store Field 2 at v5
        //       v8 = add v3, Field 1
        //       jmp b1(v8)
        //     b3():
        //       v9 = load v0
        //       v10 = eq v9, Field 2
        //       constrain v9 == Field 2
        //       v11 = load v2
        //       v12 = load v10
        //       v13 = eq v12, Field 2
        //       constrain v11 == Field 2
        //       return
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());
        let zero = builder.numeric_constant(0u128, Type::field());
        builder.insert_store(v0, zero);

        let v2 = builder.insert_allocate(Type::field());
        // Construct alias
        builder.insert_store(v2, v0);
        let v2_type = builder.current_function.dfg.type_of_value(v2);
        assert!(builder.current_function.dfg.value_is_reference(v2));

        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![zero]);

        // Loop header
        builder.switch_to_block(b1);
        let v3 = builder.add_block_parameter(b1, Type::field());
        let is_zero = builder.insert_binary(v3, BinaryOp::Eq, zero);

        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        builder.terminate_with_jmpif(is_zero, b2, b3);

        // Loop body
        builder.switch_to_block(b2);
        let v5 = builder.insert_load(v2, v2_type.clone());
        let two = builder.numeric_constant(2u128, Type::field());
        builder.insert_store(v5, two);
        let one = builder.numeric_constant(1u128, Type::field());
        let v3_plus_one = builder.insert_binary(v3, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v3_plus_one]);

        builder.switch_to_block(b3);
        let v9 = builder.insert_load(v0, Type::field());
        let _ = builder.insert_binary(v9, BinaryOp::Eq, two);

        builder.insert_constrain(v9, two, None);
        let v11 = builder.insert_load(v2, v2_type);
        let v12 = builder.insert_load(v11, Type::field());
        let _ = builder.insert_binary(v12, BinaryOp::Eq, two);

        builder.insert_constrain(v11, two, None);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();

        // We expect the same result as above.
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 4);

        // The store from the original SSA should remain
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 2);
        assert_eq!(count_stores(b2, &main.dfg), 1);

        assert_eq!(count_loads(b2, &main.dfg), 1);
        assert_eq!(count_loads(b3, &main.dfg), 3);
    }

    #[test]
    fn accurate_tracking_of_load_results() {
        // acir(inline) fn main f0 {
        //     b0():
        //       v0 = allocate
        //       store Field 5 at v0
        //       v2 = allocate
        //       store u32 10 at v2
        //       v4 = load v0
        //       v5 = load v2
        //       v6 = allocate
        //       store v4 at v6
        //       v7 = allocate
        //       store v5 at v7
        //       v8 = load v6
        //       v9 = load v7
        //       v10 = load v6
        //       v11 = load v7
        //       v12 = allocate
        //       store Field 0 at v12
        //       v15 = eq v11, u32 0
        //       jmpif v15 then: b1, else: b2
        //     b1():
        //       v16 = load v12
        //       v17 = add v16, v8
        //       store v17 at v12
        //       jmp b2()
        //     b2():
        //       v18 = load v12
        //       return [v18]
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());
        let five = builder.numeric_constant(5u128, Type::field());
        builder.insert_store(v0, five);

        let v2 = builder.insert_allocate(Type::unsigned(32));
        let ten = builder.numeric_constant(10u128, Type::unsigned(32));
        builder.insert_store(v2, ten);

        let v4 = builder.insert_load(v0, Type::field());
        let v5 = builder.insert_load(v2, Type::unsigned(32));
        let v4_type = builder.current_function.dfg.type_of_value(v4);
        let v5_type = builder.current_function.dfg.type_of_value(v5);

        let v6 = builder.insert_allocate(Type::field());
        builder.insert_store(v6, v4);
        let v7 = builder.insert_allocate(Type::unsigned(32));
        builder.insert_store(v7, v5);

        let v8 = builder.insert_load(v6, v4_type.clone());
        let _v9 = builder.insert_load(v7, v5_type.clone());

        let _v10 = builder.insert_load(v6, v4_type);
        let v11 = builder.insert_load(v7, v5_type);

        let v12 = builder.insert_allocate(Type::field());
        let zero = builder.numeric_constant(0u128, Type::field());
        builder.insert_store(v12, zero);

        let zero_u32 = builder.numeric_constant(0u128, Type::unsigned(32));
        let v15 = builder.insert_binary(v11, BinaryOp::Eq, zero_u32);

        let b1 = builder.insert_block();
        let b2 = builder.insert_block();
        builder.terminate_with_jmpif(v15, b1, b2);

        builder.switch_to_block(b1);

        let v16 = builder.insert_load(v12, Type::field());
        let v17 = builder.insert_binary(v16, BinaryOp::Add, v8);
        builder.insert_store(v12, v17);

        builder.terminate_with_jmp(b2, vec![]);

        builder.switch_to_block(b2);
        let v18 = builder.insert_load(v12, Type::field());

        // Include the load result as part of an array constant to check that we are accounting for arrays
        // when updating the reference counts of load results.
        //
        // If we were not accounting for arrays appropriately, the load of v18 would be removed.
        // If v18 is the last load of a reference and is inadvertently removed,
        // any stores to v12 will then be potentially removed as well and the program will be broken.
        let return_array =
            builder.array_constant(vector![v18], Type::Array(Arc::new(vec![Type::field()]), 1));
        builder.terminate_with_return(vec![return_array]);

        let ssa = builder.finish();

        // Expected result:
        // acir(inline) fn main f0 {
        //     b0():
        //       v20 = allocate
        //       v21 = allocate
        //       v24 = allocate
        //       v25 = allocate
        //       v30 = allocate
        //       store Field 0 at v30
        //       jmpif u1 0 then: b1, else: b2
        //     b1():
        //       store Field 5 at v30
        //       jmp b2()
        //     b2():
        //       v33 = load v30
        //       return [v33]
        //   }
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 3);

        // A single store from the entry block should remain.
        // If we are not appropriately handling unused stores across a function,
        // we would expect all five stores from the original SSA to remain.
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 1);
        // The store from the conditional block should remain,
        // as it is loaded from in a successor block and used in the return terminator.
        assert_eq!(count_stores(b1, &main.dfg), 1);

        assert_eq!(count_loads(main.entry_block(), &main.dfg), 0);
        assert_eq!(count_loads(b1, &main.dfg), 0);
        assert_eq!(count_loads(b2, &main.dfg), 1);
    }

    #[test]
    fn keep_unused_store_only_used_as_an_alias_across_blocks() {
        // acir(inline) fn main f0 {
        //     b0(v0: u32):
        //       v1 = allocate
        //       store u32 0 at v1
        //       v3 = allocate
        //       store v1 at v3
        //       v4 = allocate
        //       store v0 at v4
        //       v5 = allocate
        //       store v4 at v5
        //       jmp b1(u32 0)
        //     b1(v6: u32):
        //       v7 = eq v6, u32 0
        //       jmpif v7 then: b2, else: b3
        //     b2():
        //       v8 = load v5
        //       store v8 at u2 2
        //       v11 = add v6, u32 1
        //       jmp b1(v11)
        //     b3():
        //       v12 = load v4
        //       constrain v12 == u2 2
        //       v13 = load v5
        //       v14 = load v13
        //       constrain v14 == u2 2
        //       v15 = load v3
        //       v16 = load v15
        //       v18 = lt v16, u32 4
        //       constrain v18 == u32 1
        //       return
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.add_parameter(Type::unsigned(32));

        let v1 = builder.insert_allocate(Type::unsigned(32));
        let zero = builder.numeric_constant(0u128, Type::unsigned(32));
        builder.insert_store(v1, zero);

        let v1_type = builder.type_of_value(v1);
        let v3 = builder.insert_allocate(v1_type.clone());
        builder.insert_store(v3, v1);

        let v4 = builder.insert_allocate(Type::unsigned(32));
        builder.insert_store(v4, v0);

        let v5 = builder.insert_allocate(Type::Reference(Arc::new(Type::unsigned(32))));
        builder.insert_store(v5, v4);

        let b1 = builder.insert_block();
        builder.terminate_with_jmp(b1, vec![zero]);
        builder.switch_to_block(b1);

        let v6 = builder.add_block_parameter(b1, Type::unsigned(32));
        let is_zero = builder.insert_binary(v6, BinaryOp::Eq, zero);

        let b2 = builder.insert_block();
        let b3 = builder.insert_block();
        builder.terminate_with_jmpif(is_zero, b2, b3);

        builder.switch_to_block(b2);
        let v4_type = builder.type_of_value(v4);
        // let v0_type = builder.type_of_value(v4);
        let v8 = builder.insert_load(v5, v4_type);
        let two = builder.numeric_constant(2u128, Type::unsigned(2));
        builder.insert_store(v8, two);
        let one = builder.numeric_constant(1u128, Type::unsigned(32));
        let v11 = builder.insert_binary(v6, BinaryOp::Add, one);
        builder.terminate_with_jmp(b1, vec![v11]);

        builder.switch_to_block(b3);

        let v12 = builder.insert_load(v4, Type::unsigned(32));
        builder.insert_constrain(v12, two, None);

        let v3_type = builder.type_of_value(v3);
        let v13 = builder.insert_load(v5, v3_type);
        let v14 = builder.insert_load(v13, Type::unsigned(32));
        builder.insert_constrain(v14, two, None);

        let v15 = builder.insert_load(v3, v1_type);
        let v16 = builder.insert_load(v15, Type::unsigned(32));
        let four = builder.numeric_constant(4u128, Type::unsigned(32));
        let less_than_four = builder.insert_binary(v16, BinaryOp::Lt, four);
        builder.insert_constrain(less_than_four, one, None);

        builder.terminate_with_return(vec![]);
        let ssa = builder.finish();

        // We expect the same result as above.
        let ssa = ssa.mem2reg();
        let main = ssa.main();

        // We expect all the stores to remain.
        // The references in b0 are aliased and those are aliases may never be stored to again,
        // but they are loaded from and used in later instructions.
        // We need to make sure that the store of the address being aliased, is not removed from the program.
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 4);
        // The store inside of the loop should remain
        assert_eq!(count_stores(b2, &main.dfg), 1);

        // We expect the loads to remain the same
        assert_eq!(count_loads(b2, &main.dfg), 1);
        assert_eq!(count_loads(b3, &main.dfg), 5);
    }
}
