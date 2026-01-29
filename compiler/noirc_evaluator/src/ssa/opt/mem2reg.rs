//! The goal of the mem2reg SSA optimization pass is to replace any `Load` instructions to known
//! addresses with the value stored at that address, if it is also known. This pass will also remove
//! any `Store` instructions within a block that are no longer needed because no more loads occur in
//! between the Store in question and the next Store.
//!
//! ## How the pass works:
//! - Each block in each function is iterated in forward-order.
//! - The starting value of each reference in the block is the unification of the same references
//!   at the end of each direct predecessor block to the current block.
//! - At each step, the value of each reference is either known or unknown, tracked with a map.
//! - Two reference values unify to each other if they are exactly equal.
//! - If a block has no predecessors, the starting value of each reference is unknown (not present in the map).
//! - Throughout this pass, aliases of each reference are also tracked.
//!   - References typically have 1 alias - themselves.
//!   - A reference with multiple aliases means we will not be able to optimize out loads if the
//!     reference is stored to. Note that this means we can still optimize out loads if these
//!     aliased references are never stored to, or the store occurs after a load.
//!   - A reference with 0 aliases means we were unable to find which reference this reference
//!     refers to. If such a reference is stored to, we must conservatively invalidate every
//!     reference in the current block.
//! - We also track the last load instruction to each address per block.
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
//!   - We also track the last instance of a load instruction to each address in a block.
//!     If we see that the last load instruction was from the same address as the current load instruction,
//!     we move to replace the result of the current load with the result of the previous load.
//!
//!     This removal requires a couple conditions:
//!       - No store occurs to that address before the next load,
//!       - The address is not used as an argument to a call
//!
//!     This optimization helps us remove repeated loads for which there are not known values.
//! - On `Instruction::Store { address, value }`:
//!   - If the address of the store is known:
//!     - If the address has exactly 1 alias:
//!       - Set the value of the address to the known `value`.
//!     - If the address has more than 1 alias:
//!       - Clear out the known value of of every possible alias.
//!     - If the address has 0 aliases:
//!       - Conservatively mark every alias in the block as unknown.
//!   - If the address of the store is not known:
//!     - Conservatively mark every alias in the block as unknown.
//!   - Additionally, if there were no Loads to any alias of the address between this Store and
//!     the previous Store to the same address, the previous store can be removed.
//!   - Remove the instance of the last load instruction to the address and its aliases
//! - On `Instruction::Call { arguments }`:
//!   - If any argument of the call is a reference, remove the known value of each alias of that
//!     reference
//!   - Any builtin functions that may return aliases if their input also contains a
//!     reference should be tracked. Examples: `vector_push_back`, `vector_insert`, `vector_remove`, etc.
//!   - Remove the instance of the last load instruction for any reference arguments and their aliases
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
mod alias_set;
mod block;

use std::collections::{BTreeMap, BTreeSet};

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use vec_collections::VecSet;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        dfg::InsertInstructionResult,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
        value::{Value, ValueId},
    },
    opt::unrolling::Loops,
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
            function.mem2reg();
        }
        self
    }
}

impl Function {
    pub(crate) fn mem2reg(&mut self) {
        // Analyze loops to find potential loop carried aliases
        let loop_aliases = Self::analyze_loop_aliases(self);
        // Perform mem2reg optimization with loop carried alias information
        // Non-lop alias information will be analyzed as part of mem2reg
        let mut context = PerFunctionContext::new(self, loop_aliases);
        context.mem2reg();
        context.remove_instructions();
        context.update_data_bus();
    }

    /// Analyzes all loops in the function to find references with potential loop carried aliases.
    ///
    /// A loop carried alias occurs when a reference is stored into another reference
    /// within a loop. This creates an aliasing relationship that persists across loop iterations,
    /// making it unsafe to remove certain stores to these references.
    ///
    /// Returns a set of values that may have loop carried aliases.
    fn analyze_loop_aliases(function: &Function) -> HashSet<ValueId> {
        let loops = Loops::find_all(function);
        let mut aliases: HashSet<ValueId> = HashSet::default();

        // For each loop, find all `store ref_value at ref_address` patterns
        for loop_info in &loops.yet_to_unroll {
            for block_id in &loop_info.blocks {
                let block = &function.dfg[*block_id];

                for instruction_id in block.instructions() {
                    if let Instruction::Store { address, value } = &function.dfg[*instruction_id] {
                        // Check if both the address and value are references
                        // This indicates we're storing a reference into another reference
                        if function.dfg.value_is_reference(*address)
                            && function.dfg.value_is_reference(*value)
                        {
                            // Mark both the address and value as potentially aliased
                            aliases.insert(*address);
                            aliases.insert(*value);
                        }
                    }
                }
            }
        }

        aliases
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

    /// All instructions analyzed so far in the function.
    /// Anything new can be reinserted,  while things that appear repeatedly have to be cloned.
    instructions_analyzed: HashSet<InstructionId>,

    /// Track a value's last load across all blocks.
    /// If a value is not used in anymore loads we can remove the last store to that value.
    last_loads: HashSet<ValueId>,

    /// Track whether a reference was passed into another instruction (e.g. Call)
    /// This is needed to determine whether we can remove a store.
    instruction_input_references: HashSet<ValueId>,

    /// Track whether a reference has been aliased, and store the respective
    /// instruction that aliased that reference.
    /// If that store has been set for removal, we can also remove this instruction.
    aliased_references: HashMap<ValueId, HashSet<InstructionId>>,

    /// Loop carried aliases: references that may have aliases due to stores within loops.
    /// Contains values of references that should not have their stores removed.
    loop_aliases: HashSet<ValueId>,
}

impl<'f> PerFunctionContext<'f> {
    fn new(function: &'f mut Function, loop_aliases: HashSet<ValueId>) -> Self {
        let cfg = ControlFlowGraph::with_function(function);
        let post_order = PostOrder::with_cfg(&cfg);
        PerFunctionContext {
            cfg,
            post_order,
            inserter: FunctionInserter::new(function),
            blocks: BTreeMap::new(),
            instructions_to_remove: HashSet::default(),
            instructions_analyzed: HashSet::default(),
            last_loads: HashSet::default(),
            aliased_references: HashMap::default(),
            instruction_input_references: HashSet::default(),
            loop_aliases,
        }
    }

    /// Check if an address has loop carried aliases.
    ///
    /// This is important for preventing incorrect store removal. If `store v3 at v2`
    /// occurs in a loop, then v2 may point to v3 in future iterations. A subsequent
    /// `store X at v3` should not have its previous store removed, as that store may
    /// be read through v2 (which now aliases v3) in a future loop iteration.
    ///
    /// We only check the address, not the value being stored. The decision of
    /// whether to remove a previous store to `address` depends solely on whether
    /// `address` might be accessed through an alias, not on whether the value being
    /// stored has aliases.
    fn has_loop_carried_aliases(&self, address: ValueId) -> bool {
        self.loop_aliases.contains(&address)
    }

    /// Apply the mem2reg pass to the given function.
    ///
    /// This function is expected to be the same one that the internal cfg, post_order, and
    /// dom_tree were created from.
    fn mem2reg(&mut self) {
        // Iterate each block in reverse post order = forward order
        let block_order = self.post_order.clone().into_vec_reverse();

        for block in block_order {
            let references = self.find_starting_references(block);
            self.analyze_block(block, references);
        }

        let mut all_terminator_values = HashSet::default();
        let mut per_func_block_params: HashSet<ValueId> = HashSet::default();
        for (block_id, references) in self.blocks.iter_mut() {
            let block_params = self.inserter.function.dfg.block_parameters(*block_id);
            per_func_block_params.extend(block_params.iter());
            let terminator = self.inserter.function.dfg[*block_id].unwrap_terminator();
            terminator.for_each_value(|value| {
                all_terminator_values.insert(value);
                // Also insert all the aliases of this value as being used in the terminator,
                // so that for example if the value is an array and contains a reference,
                // then that reference gets to keep its last store.
                let typ = self.inserter.function.dfg.type_of_value(value);
                if typ.contains_reference() {
                    all_terminator_values.extend(references.get_aliases_for_value(value).iter());
                }
            });
        }

        // Add all the aliases of values used in the terminators.

        // If we never load from an address within a function we can remove all stores to that address.
        // This rule does not apply to reference parameters, which we must also check for before removing these stores.
        for (_, block) in self.blocks.iter() {
            for (store_address, store_instruction) in block.last_stores.iter() {
                let store_alias_used = self.is_store_alias_used(
                    store_address,
                    block,
                    &all_terminator_values,
                    &per_func_block_params,
                );

                let is_dereference = block
                    .expressions
                    .get(store_address)
                    .is_some_and(|expression| matches!(expression, Expression::Dereference(_)));

                if !self.last_loads.contains(store_address) && !store_alias_used && !is_dereference
                {
                    self.instructions_to_remove.insert(*store_instruction);
                }
            }
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
        let reference_parameters = self.reference_parameters();

        let aliases = block.get_aliases_for_value(*store_address);
        if aliases.is_unknown() {
            return true;
        }
        // Check whether the store address has an alias that crosses an entry point boundary (e.g. a Call or Return)
        for alias in aliases.iter() {
            if reference_parameters.contains(&alias) {
                return true;
            }

            if per_func_block_params.contains(&alias) {
                return true;
            }

            // Is any alias of this address an input to some function call, or a return value?
            if self.instruction_input_references.contains(&alias) {
                return true;
            }

            // Is any alias of this address used in a block terminator?
            if all_terminator_values.contains(&alias) {
                return true;
            }

            // Check whether there are any aliases whose instructions are not all marked for removal.
            // If there is any alias marked to survive, we should not remove its last store.
            if let Some(alias_instructions) = self.aliased_references.get(&alias) {
                if !alias_instructions.is_subset(&self.instructions_to_remove) {
                    return true;
                }
            }
        }

        false
    }

    /// Collect the input parameters of the function which are of reference type.
    /// All references are mutable, so these inputs are shared with the function caller
    /// and thus stores should not be eliminated, even if the blocks in this function
    /// don't use them anywhere.
    fn reference_parameters(&self) -> BTreeSet<ValueId> {
        let parameters = self.inserter.function.parameters().iter();
        parameters
            .filter(|param| self.inserter.function.dfg.value_is_reference(**param))
            .copied()
            .collect()
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

        // If this is the entry block, take all the block parameters and assume they may
        // be aliased to each other
        if block == self.inserter.function.entry_block() {
            self.add_aliases_for_reference_parameters(block, &mut references);
        }

        for instruction in instructions {
            self.analyze_instruction(block, &mut references, instruction);
        }

        self.handle_terminator(block, &mut references);

        // If there's only 1 block in the function total, we can remove any remaining last stores
        // as well. We can't do this if there are multiple blocks since subsequent blocks may
        // reference these stores.
        if self.post_order.as_slice().len() == 1 {
            self.remove_stores_that_do_not_alias_parameters_or_returns(&references);
        }

        self.blocks.insert(block, references);
    }

    /// Go through each parameter and register that all reference parameters of the same type are
    /// possibly aliased to each other. If there are parameters with nested references (arrays of
    /// references or references containing other references) we give up and assume all parameter
    /// references are `AliasSet::unknown()`.
    fn add_aliases_for_reference_parameters(&self, block: BasicBlockId, references: &mut Block) {
        let dfg = &self.inserter.function.dfg;
        let params = dfg.block_parameters(block);

        let mut aliases: HashMap<Type, AliasSet> = HashMap::default();

        for param in params {
            match dfg.type_of_value(*param) {
                // If the type indirectly contains a reference we have to assume all references
                // are unknown since we don't have any ValueIds to use.
                Type::Reference(element) if element.contains_reference() => {
                    self.mark_all_unknown(params, references);
                    return;
                }
                Type::Reference(element) => {
                    let empty_aliases = AliasSet::known_empty();
                    let alias_set =
                        aliases.entry(element.as_ref().clone()).or_insert(empty_aliases);
                    alias_set.insert(*param);
                }
                typ if typ.contains_reference() => {
                    self.mark_all_unknown(params, references);
                    return;
                }
                _ => continue,
            }
        }

        for aliases in aliases.into_values() {
            let first = aliases.first();
            let first = first.expect("All parameters alias at least themselves or we early return");

            let expression = Expression::Other(first);
            let previous = references.aliases.insert(expression, aliases.clone());
            assert!(previous.is_none());

            for alias in aliases.iter() {
                let previous = references.expressions.insert(alias, expression);
                assert!(previous.is_none());
            }
        }
    }

    /// Add all instructions in `last_stores` to `self.instructions_to_remove` which do not
    /// possibly alias any parameters of the given function.
    fn remove_stores_that_do_not_alias_parameters_or_returns(&mut self, references: &Block) {
        let reference_parameters = self.reference_parameters();

        for (allocation, instruction) in &references.last_stores {
            let aliases = references.get_aliases_for_value(*allocation);

            let allocation_aliases_parameter =
                aliases.any(|alias| reference_parameters.contains(&alias));
            // If `allocation_aliases_parameter` is known to be false
            if allocation_aliases_parameter != Some(false) {
                continue;
            }

            let allocation_aliases_input_reference =
                aliases.any(|alias| self.instruction_input_references.contains(&alias));
            if allocation_aliases_input_reference != Some(false) {
                continue;
            }

            self.instructions_to_remove.insert(*instruction);
        }
    }

    fn analyze_instruction(
        &mut self,
        block_id: BasicBlockId,
        references: &mut Block,
        instruction_id: InstructionId,
    ) {
        // If the instruction was simplified and optimized out of the program we shouldn't analyze it.
        // Analyzing it could make tracking aliases less accurate if it is e.g. an ArrayGet
        // call that used to hold references but has since been optimized out to a known result.
        // However, if we don't analyze it, then it may be a MakeArray replacing an ArraySet containing references,
        // and we need to mark those references as used to keep their stores alive.
        let (instruction, loc) = self.inserter.map_instruction(instruction_id);

        // We track which instructions can be removed by ID; if we allowed the same ID to appear multiple times
        // in a block then we could not tell them apart. When we see something the first time we can reuse it.
        let allow_reinsert = self.instructions_analyzed.insert(instruction_id);

        match self.inserter.push_instruction_value(
            instruction,
            instruction_id,
            block_id,
            loc,
            allow_reinsert,
        ) {
            InsertInstructionResult::Results(id, _) => {
                self.analyze_possibly_simplified_instruction(references, id, false);
            }
            InsertInstructionResult::SimplifiedTo(value) => {
                // Globals cannot contain references thus we do not need to analyze insertion which simplified to them.
                if self.inserter.function.dfg.is_global(value) {
                    return;
                }
                let value = &self.inserter.function.dfg[value];
                if let Value::Instruction { instruction, .. } = value {
                    self.analyze_possibly_simplified_instruction(references, *instruction, true);
                }
            }
            InsertInstructionResult::SimplifiedToMultiple(values) => {
                for value in values {
                    // Globals cannot contain references thus we do not need to analyze insertion which simplified to them.
                    if self.inserter.function.dfg.is_global(value) {
                        continue;
                    }
                    let value = &self.inserter.function.dfg[value];
                    if let Value::Instruction { instruction, .. } = value {
                        self.analyze_possibly_simplified_instruction(
                            references,
                            *instruction,
                            true,
                        );
                    }
                }
            }
            InsertInstructionResult::InstructionRemoved => (),
        }
    }

    fn analyze_possibly_simplified_instruction(
        &mut self,
        references: &mut Block,
        instruction: InstructionId,
        simplified: bool,
    ) {
        let ins = &self.inserter.function.dfg[instruction];

        // Some instructions, when simplified, cause problems if processed again.
        // We do need it for MakeArray in some cases, and at the moment that is not problematic.
        if simplified && !matches!(ins, Instruction::MakeArray { .. }) {
            return;
        }

        match ins {
            Instruction::Load { address } => {
                let address = *address;

                let [result] = self.inserter.function.dfg.instruction_result(instruction);
                references.remember_dereference(self.inserter.function, address, result);

                // If the load is known, replace it with the known value and remove the load.
                if let Some(value) = references.get_known_value(address) {
                    let [result] = self.inserter.function.dfg.instruction_result(instruction);
                    self.inserter.map_value(result, value);
                    self.instructions_to_remove.insert(instruction);
                }
                // Check whether the block has a repeat load from the same address (w/ no calls or stores in between the loads).
                // If we do have a repeat load, we can remove the current load and map its result to the previous load's result.
                else if let Some(last_load) = references.last_loads.get(&address) {
                    let Instruction::Load { address: previous_address } =
                        &self.inserter.function.dfg[*last_load]
                    else {
                        panic!("Expected a Load instruction here");
                    };
                    let [result] = self.inserter.function.dfg.instruction_result(instruction);
                    let [previous_result] =
                        self.inserter.function.dfg.instruction_result(*last_load);
                    if *previous_address == address {
                        self.inserter.map_value(result, previous_result);
                        self.instructions_to_remove.insert(instruction);
                    }
                } else {
                    // Remember that this address has been loaded, so stores to it should not be removed.
                    self.last_loads.insert(address);
                    // Stores to any of its aliases should also be considered loaded.
                    self.last_loads.extend(references.get_aliases_for_value(address).iter());
                }

                // If the address is potentially aliased we must keep the stores to it
                if references.get_aliases_for_value(address).single_alias().is_none() {
                    references.mark_value_used(address, self.inserter.function);
                }

                // We want to set the load for every load even if the address has a known value
                // and the previous load instruction was removed.
                // We are safe to still remove a repeat load in this case as we are mapping from the current load's
                // result to the previous load, which if it was removed should already have a mapping to the known value.
                references.set_last_load(address, instruction);
            }
            Instruction::Store { address, value } => {
                let address = *address;
                let value = *value;

                let address_aliases = references.get_aliases_for_value(address);
                // If there was another store to this address without any (unremoved) loads or
                // function calls in-between, we can remove the previous store.
                // However, we must be conservative if there are loop carried aliases, as loads
                // through those aliases may occur in future loop iterations.
                let has_loop_aliases = self.has_loop_carried_aliases(address);
                if !self.aliased_references.contains_key(&address)
                    && !address_aliases.is_unknown()
                    && !has_loop_aliases
                {
                    if let Some(last_store) = references.last_stores.get(&address) {
                        self.instructions_to_remove.insert(*last_store);
                    }
                }

                // Remember that we used the value in this instruction. If this instruction
                // isn't removed at the end, we need to keep the stores to the value as well.
                for alias in references.get_aliases_for_value(value).iter() {
                    self.aliased_references.entry(alias).or_default().insert(instruction);
                }

                references.set_known_value(address, value);
                // If we see a store to an address, the last load to that address needs to remain.
                references.keep_last_load_for(address);
                references.last_stores.insert(address, instruction);
            }
            Instruction::Allocate => {
                // Register the new reference
                let [result] = self.inserter.function.dfg.instruction_result(instruction);
                references.expressions.insert(result, Expression::Other(result));
                references.aliases.insert(Expression::Other(result), AliasSet::known(result));
            }
            Instruction::ArrayGet { array, .. } => {
                let [result] = self.inserter.function.dfg.instruction_result(instruction);

                if self.inserter.function.dfg.type_of_value(result).contains_reference() {
                    let array = *array;
                    self.instruction_input_references
                        .extend(references.get_aliases_for_value(array).iter());
                    references.mark_value_used(array, self.inserter.function);

                    // An expression for the value might already exist, so try to fetch it first
                    let expression = references.expressions.get(&array).copied();
                    let expression = expression.unwrap_or(Expression::Other(array));
                    if let Some(aliases) = references.aliases.get_mut(&expression) {
                        aliases.insert(result);
                    }

                    // Any aliases of the array need to be updated to also include the result of the array get in their alias sets.
                    for alias in (*references.get_aliases_for_value(array)).clone().iter() {
                        // An expression for the alias might already exist, so try to fetch it first
                        let expression = references.expressions.get(&alias).copied();
                        let expression = expression.unwrap_or(Expression::Other(alias));
                        if let Some(aliases) = references.aliases.get_mut(&expression) {
                            aliases.insert(result);
                        }
                    }

                    // In this SSA:
                    //
                    // v2 = array_get v0, index v1 -> Field
                    //
                    // make v2 point to v0 so they share the same alias set
                    references.expressions.insert(result, expression);
                }
            }
            Instruction::ArraySet { array, value, .. } => {
                references.mark_value_used(*array, self.inserter.function);
                let element_type = self.inserter.function.dfg.type_of_value(*value);

                if element_type.contains_reference() {
                    let [result] = self.inserter.function.dfg.instruction_result(instruction);
                    let array = *array;

                    let expression = references.expressions.get(&array).copied();
                    let expression = expression.unwrap_or(Expression::Other(array));

                    let mut aliases = if let Some(aliases) = references.aliases.get_mut(&expression)
                    {
                        aliases.clone()
                    } else if let Some((elements, _)) =
                        self.inserter.function.dfg.get_array_constant(array)
                    {
                        references.collect_all_aliases(elements)
                    } else {
                        AliasSet::unknown()
                    };
                    aliases.unify(&references.get_aliases_for_value(*value));

                    references.expressions.insert(result, expression);
                    references.aliases.insert(expression, aliases.clone());

                    // Similar to how we remember that we used a value in a `Store` instruction,
                    // take note that it was used in the `ArraySet`. If this instruction is not
                    // going to be removed at the end, we shall keep the stores to this value as well.
                    //
                    // We want to make sure to mark aliased references before we update the value's alias list to match the array itself.
                    // This ordering is necessary because if the unified alias list becomes unknown we will not end up
                    // inserting the value as a possible aliased reference across blocks.
                    // This could also be done by checking whether the new unified aliases are unknown and marking the `value` explicitly as an alias.
                    for alias in references.get_aliases_for_value(*value).iter() {
                        self.aliased_references.entry(alias).or_default().insert(instruction);
                    }

                    // The value being stored in the array also needs its aliases updated to match the array itself
                    let value_expression = references.expressions.get(value).copied();
                    let value_expression = value_expression.unwrap_or(Expression::Other(*value));
                    references.aliases.insert(value_expression, aliases);
                }
            }
            Instruction::Call { arguments, .. } => {
                // We need to appropriately mark each alias of a reference as being used as a call argument.
                // This prevents us potentially removing a last store from a preceding block or is altered within another function.
                for arg in arguments {
                    self.instruction_input_references
                        .extend(references.get_aliases_for_value(*arg).iter());
                }
                self.mark_all_unknown(arguments, references);

                // Call results might be aliases of their arguments, if they are references
                let results = self.inserter.function.dfg.instruction_results(instruction);
                let results_contains_references = results.iter().any(|result| {
                    self.inserter.function.dfg.type_of_value(*result).contains_reference()
                });

                // Instead of aliasing results to arguments, because values might be nested references
                // we'll just consider all arguments and references as now having unknown aliases.
                if results_contains_references {
                    for value in arguments.iter().chain(results) {
                        self.clear_aliases(references, *value);
                    }
                }
            }
            Instruction::MakeArray { elements, typ } => {
                // If `array` is an array constant that contains reference types, then insert each element
                // as a potential alias to the array itself.
                if typ.contains_reference() {
                    let [array] = self.inserter.function.dfg.instruction_result(instruction);

                    let expr = Expression::ArrayElement(array);
                    references.expressions.insert(array, expr);

                    let new_aliases = self.collect_array_aliases(elements, references);
                    let aliases = references.aliases.entry(expr).or_insert(AliasSet::known_empty());
                    aliases.unify(&new_aliases);
                }
            }
            Instruction::IfElse { then_value, else_value, .. } => {
                let [result] = self.inserter.function.dfg.instruction_result(instruction);
                let result_type = self.inserter.function.dfg.type_of_value(result);

                if result_type.contains_reference() {
                    let expr = Expression::Other(result);
                    references.expressions.insert(result, expr);

                    // Collect all aliases from both branches.
                    // For the case: `if cond { array1 } else { array2 }` where
                    // both arrays contain references - we need to track them.
                    let then_aliases = references.get_aliases_for_value(*then_value).into_owned();
                    let else_aliases = references.get_aliases_for_value(*else_value).into_owned();
                    let mut all_aliases =
                        AliasSet::known_multiple(vec![*then_value, *else_value].into());
                    all_aliases.unify(&then_aliases);
                    all_aliases.unify(&else_aliases);

                    references.aliases.insert(expr, all_aliases.clone());

                    // Mark references in both branches as being used by this IfElse instruction.
                    // This ensures that stores to those references are kept even in loops where
                    // alias information may not propagate correctly through the back edge.
                    for alias in then_aliases.iter() {
                        self.aliased_references.entry(alias).or_default().insert(instruction);
                    }
                    for alias in else_aliases.iter() {
                        self.aliased_references.entry(alias).or_default().insert(instruction);
                    }

                    // `then_value` and `else_value` are now aliased by `result`
                    if let Some(then_expr) = references.expressions.get_mut(then_value) {
                        if let Some(then_aliases) = references.aliases.get_mut(then_expr) {
                            then_aliases.insert(result);
                        }
                    }

                    if let Some(else_expr) = references.expressions.get_mut(else_value) {
                        if let Some(else_aliases) = references.aliases.get_mut(else_expr) {
                            else_aliases.insert(result);
                        }
                    }
                }
            }
            _ => (),
        }
    }

    /// In order to handle nested arrays we need to recursively search for whether there
    /// are any aliases contained within an array's elements.
    fn collect_array_aliases(
        &self,
        elements: &im::Vector<ValueId>,
        references: &Block,
    ) -> AliasSet {
        let mut aliases = AliasSet::known_empty();
        for &element in elements {
            if let Some((elements, _)) = self.inserter.function.dfg.get_array_constant(element) {
                aliases.unify(&self.collect_array_aliases(&elements, references));
            } else if self.inserter.function.dfg.type_of_value(element).contains_reference() {
                // Handles both direct references and non-constant arrays (e.g., array_set results)
                aliases.unify(&references.get_aliases_for_value(element));
            }
        }
        aliases
    }

    fn set_aliases(&self, references: &mut Block, address: ValueId, new_aliases: AliasSet) {
        let expression =
            references.expressions.entry(address).or_insert(Expression::Other(address));
        let aliases = references.aliases.entry(*expression).or_default();
        *aliases = new_aliases;
    }

    fn clear_aliases(&self, references: &mut Block, value: ValueId) {
        if !self.inserter.function.dfg.type_of_value(value).contains_reference() {
            return;
        }

        if let Some(expression) = references.expressions.get(&value) {
            references.aliases.remove(expression);
        }

        if let Some((values, _)) = self.inserter.function.dfg.get_array_constant(value) {
            for value in values {
                self.clear_aliases(references, value);
            }
        }
    }

    fn mark_all_unknown(&self, values: &[ValueId], references: &mut Block) {
        for value in values {
            let typ = self.inserter.function.dfg.type_of_value(*value);
            if typ.contains_reference() {
                let value = *value;
                references.set_unknown(value);
                references.mark_value_used(value, self.inserter.function);

                // If a reference is an argument to a call, the last load to that address and its aliases needs to remain.
                references.keep_last_load_for(value);
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

        match self.inserter.function.dfg[block].unwrap_terminator() {
            TerminatorInstruction::JmpIf { .. } | TerminatorInstruction::Unreachable { .. } => (), // Nothing to do
            TerminatorInstruction::Jmp { destination, arguments, .. } => {
                let destination_parameters = self.inserter.function.dfg[*destination].parameters();
                assert_eq!(destination_parameters.len(), arguments.len());

                // If we have multiple parameters that alias that same argument value,
                // then those parameters also alias each other.
                // We save parameters with repeat arguments to later mark those
                // parameters as aliasing one another.
                let mut arg_set: HashMap<ValueId, VecSet<[ValueId; 1]>> = HashMap::default();

                // Add an alias for each reference parameter
                for (parameter, argument) in destination_parameters.iter().zip(arguments) {
                    match self.inserter.function.dfg.type_of_value(*parameter) {
                        // If the type indirectly contains a reference we have to assume all references
                        // are unknown since we don't have any ValueIds to use.
                        Type::Reference(element) if element.contains_reference() => {
                            self.mark_all_unknown(destination_parameters, references);
                            return;
                        }
                        Type::Reference(_) => {
                            if let Some(expression) = references.expressions.get(argument) {
                                if let Some(aliases) = references.aliases.get_mut(expression) {
                                    // If the argument has unknown aliases, we must be conservative
                                    // and mark all destination parameters as unknown. Otherwise,
                                    // inserting into an unknown alias set is a no-op and destination parameters
                                    // would incorrectly end up in separate alias sets.
                                    if aliases.is_unknown() {
                                        self.mark_all_unknown(destination_parameters, references);
                                        return;
                                    }

                                    let argument = *argument;

                                    // The argument reference is possibly aliased by this block parameter
                                    aliases.insert(*parameter);

                                    // Check if we have seen the same argument
                                    let seen_parameters = arg_set
                                        .entry(argument)
                                        .or_insert_with(|| VecSet::single(argument));
                                    // Add the current parameter to the parameters we have seen for this argument.
                                    // The previous parameters and the current one alias one another.
                                    seen_parameters.insert(*parameter);
                                    // Also add all of the argument aliases
                                    seen_parameters.extend(aliases.iter());
                                }
                            }
                        }
                        typ if typ.contains_reference() => {
                            self.mark_all_unknown(destination_parameters, references);
                            return;
                        }
                        _ => continue,
                    }
                }

                // Set the aliases of the parameters
                for (_, aliased_params) in arg_set {
                    for param in aliased_params.iter() {
                        self.set_aliases(
                            references,
                            *param,
                            AliasSet::known_multiple(aliased_params.clone()),
                        );
                    }
                }
            }
            TerminatorInstruction::Return { return_values, .. } => {
                // Removing all `last_stores` for each returned reference is more important here
                // than setting them all to unknown since no other block should
                // have a block with a Return terminator as a predecessor anyway.
                self.mark_all_unknown(return_values, references);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_ssa_snapshot,
        ssa::{Ssa, interpreter::value::Value, opt::assert_ssa_does_not_change},
    };

    #[test]
    fn test_simple() {
        let src = "
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v3 = make_array [Field 1, Field 1] : [Field; 2]
            store v3 at v0
            v4 = load v0 -> [Field; 2]
            v5 = array_get v4, index u32 1 -> Field
            return v5
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v2 = make_array [Field 1, Field 1] : [Field; 2]
            return Field 1
        }
        ");
    }

    #[test]
    fn test_simple_with_call() {
        let src = "
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v2 = load v0 -> Field
            call assert_constant(v0)
            return v2
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            call assert_constant(v0)
            return Field 1
        }
        ");
    }

    #[test]
    fn test_simple_with_return() {
        let src = "
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v1 = load v0 -> Field
            return v1
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn func f0 {
          b0():
            v0 = allocate -> &mut Field
            return Field 1
        }
        ");
    }

    #[test]
    fn multiple_blocks() {
        // Test that loads across multiple blocks are removed
        let src = "
         acir(inline) fn main f0 {
           b0():
             v0 = allocate -> &mut Field
             store Field 5 at v0
             v2 = load v0 -> Field
             jmp b1(v2)
           b1(v3: Field):
             v4 = load v0 -> Field
             store Field 6 at v0
             v6 = load v0 -> Field
             return v3, v4, v6
         }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            jmp b1(Field 5)
          b1(v0: Field):
            return v0, Field 5, Field 6
        }
        ");
    }

    #[test]
    fn load_aliases_in_predecessor_block() {
        // Test that a load in a predecessor block has been removed if the value
        // is later stored in a successor block
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = load v2 -> &mut Field
            v4 = load v2 -> &mut Field
            jmp b1()
          b1():
            store Field 1 at v3
            store Field 2 at v4
            v7 = load v3 -> Field
            v8 = eq v7, Field 2
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        // All loads should be removed
        // The first store is not removed as it is used as a nested reference in another store.
        // We would need to track whether the store where `v0` is the store value gets removed to know whether
        // to remove it.
        // The final store in b1 is removed as no loads are done within any blocks
        // to the stored values.
        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            jmp b1()
          b1():
            store Field 1 at v0
            return
        }
        ");
    }

    #[test]
    fn keep_store_to_alias_in_loop_block() {
        // This test makes sure the instruction `store Field 2 at v5` in b2 remains after mem2reg.
        // Although the only instruction on v5 is a lone store without any loads,
        // v5 is an alias of the reference v0 which is stored in v2.
        // This test makes sure that we are not inadvertently removing stores to aliases across blocks.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            jmp b1(Field 0)
          b1(v3: Field):
            v4 = eq v3, Field 0
            jmpif v4 then: b2, else: b3
          b2():
            v5 = load v2 -> &mut Field
            store Field 2 at v5
            v8 = add v3, Field 1
            jmp b1(v8)
          b3():
            v9 = load v0 -> Field
            v10 = eq v9, Field 2
            constrain v9 == Field 2
            v11 = load v2 -> &mut Field
            v12 = load v11 -> Field
            constrain v12 == Field 2
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn parameter_alias() {
        // Do not assume parameters are not aliased to each other.
        // The load below shouldn't be removed since `v0` could
        // be aliased to `v1`.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            store Field 0 at v0
            store Field 0 at v1
            v3 = load v0 -> Field
            constrain v3 == Field 0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn parameter_alias_nested_reference() {
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: &mut &mut Field):
            store Field 0 at v0
            store Field 0 at v1
            v3 = load v0 -> Field
            constrain v3 == Field 0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn remove_repeat_loads() {
        // This tests starts with two loads from the same unknown load.
        // Specifically you should look for `load v2` in `b3`.
        // We should be able to remove the second repeated load.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            jmp b1(Field 0)
          b1(v3: Field):
            v4 = eq v3, Field 0
            jmpif v4 then: b2, else: b3
          b2():
            v5 = load v2 -> &mut Field
            store Field 2 at v5
            v8 = add v3, Field 1
            jmp b1(v8)
          b3():
            v9 = load v0 -> Field
            v10 = eq v9, Field 2
            constrain v9 == Field 2
            v11 = load v2 -> &mut Field
            v12 = load v2 -> &mut Field
            v13 = load v12 -> Field
            v14 = eq v13, Field 2
            constrain v13 == Field 2
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();

        // The repeated load from v3 should be removed
        // b3 should only have three loads now rather than four previously
        //
        // All stores are expected to remain.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = allocate -> &mut &mut Field
            store v1 at v3
            jmp b1(Field 0)
          b1(v0: Field):
            v4 = eq v0, Field 0
            jmpif v4 then: b2, else: b3
          b2():
            v11 = load v3 -> &mut Field
            store Field 2 at v11
            v13 = add v0, Field 1
            jmp b1(v13)
          b3():
            v5 = load v1 -> Field
            v7 = eq v5, Field 2
            constrain v5 == Field 2
            v8 = load v3 -> &mut Field
            v9 = load v8 -> Field
            v10 = eq v9, Field 2
            constrain v9 == Field 2
            return
        }
        ");
    }

    #[test]
    fn keep_repeat_loads_passed_to_a_call() {
        // The test is the exact same as `remove_repeat_loads` above except with the call
        // to `f1` between the repeated loads.
        let src = "
        acir(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = allocate -> &mut &mut Field
            store v1 at v3
            jmp b1(Field 0)
          b1(v0: Field):
            v4 = eq v0, Field 0
            jmpif v4 then: b3, else: b2
          b2():
            v9 = load v1 -> Field
            v10 = eq v9, Field 2
            constrain v9 == Field 2
            v11 = load v3 -> &mut Field
            call f1(v3)
            v13 = load v3 -> &mut Field
            v14 = load v13 -> Field
            v15 = eq v14, Field 2
            constrain v14 == Field 2
            return
          b3():
            v5 = load v3 -> &mut Field
            store Field 2 at v5
            v8 = add v0, Field 1
            jmp b1(v8)
        }
        acir(inline) fn foo f1 {
          b0(v0: &mut &mut Field):
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_repeat_loads_with_alias_store() {
        // v1, v2, and v3 alias one another. We want to make sure that a repeat load to v1 with a store
        // to its aliases in between the repeat loads does not remove those loads.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b2, else: b1
          b1():
            v6 = allocate -> &mut Field
            store Field 1 at v6
            jmp b3(v6, v6, v6)
          b2():
            v4 = allocate -> &mut Field
            store Field 0 at v4
            jmp b3(v4, v4, v4)
          b3(v1: &mut Field, v2: &mut Field, v3: &mut Field):
            v8 = load v1 -> Field
            store Field 2 at v2
            v10 = load v1 -> Field
            store Field 1 at v3
            v11 = load v1 -> Field
            store Field 3 at v3
            v13 = load v1 -> Field
            constrain v8 == Field 0
            constrain v10 == Field 2
            constrain v11 == Field 1
            constrain v13 == Field 3
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_repeat_loads_with_alias_store_nested() {
        // v1, v2, and v3's inner reference alias one another. We want to make sure that a repeat load to v3 with a store
        // to its aliases in between the repeat loads does not remove those loads.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: u1):
            jmpif v0 then: b2, else: b1
          b1():
            v8 = allocate -> &mut Field
            store Field 1 at v8
            v10 = allocate -> &mut Field
            store Field 2 at v10
            v12 = allocate -> &mut &mut Field
            store v10 at v12
            jmp b3(v8, v8, v12)
          b2():
            v4 = allocate -> &mut Field
            store Field 0 at v4
            v6 = allocate -> &mut Field
            v7 = allocate -> &mut &mut Field
            store v4 at v7
            jmp b3(v4, v4, v7)
          b3(v1: &mut Field, v2: &mut Field, v3: &mut &mut Field):
            v13 = load v1 -> Field
            store Field 2 at v2
            v14 = load v1 -> Field
            store Field 1 at v2
            v15 = load v1 -> Field
            store Field 3 at v2
            v17 = load v1 -> Field
            constrain v13 == Field 0
            constrain v14 == Field 2
            constrain v15 == Field 1
            constrain v17 == Field 3
            v18 = load v3 -> &mut Field
            v19 = load v18 -> Field
            store Field 5 at v2
            v21 = load v3 -> &mut Field
            v22 = load v21 -> Field
            constrain v19 == Field 3
            constrain v22 == Field 5
            return
        }
        ";

        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store_in_make_array_used_in_call_single_block() {
        // This checks that when an array containing references is used in a call
        // that we do not remove the original stores to those internal references
        let src = r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 1 at v0
            v3 = make_array [u32 0, v0] : [(u32, &mut Field); 1]
            v5 = call f1(v3) -> Field
            v6 = load v0 -> Field   // make sure this isn't optimized to Field 1
            return v3
        }
        brillig(inline) fn foo f1 {
          b0(v0: [(u32, &mut Field); 1]):
            v2 = array_get v0, index u32 1 -> &mut Field
            v3 = load v2 -> Field
            store Field 77 at v2
            return v3
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store_used_in_make_array_used_as_reference() {
        let src = r"
        acir(inline) fn main f0 {
          b0():
            v2 = allocate -> &mut u1
            store u1 0 at v2
            v4 = make_array [v2] : [&mut u1; 1]
            v5 = allocate -> &mut [&mut u1; 1]
            store v4 at v5
            jmp b1(u32 0)
          b1(v0: u32):
            v7 = eq v0, u32 0
            jmpif v7 then: b2, else: b3
          b2():
            v14 = unchecked_add v0, u32 1
            jmp b1(v14)
          b3():
            v8 = load v5 -> [&mut u1; 1]
            v9 = array_get v8, index u32 0 -> &mut u1
            v10 = load v9 -> u1
            jmpif v10 then: b4, else: b5
          b4():
            jmp b6(Field 0)
          b5():
            jmp b6(Field 1)
          b6(v1: Field):
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store_in_make_array_returned_from_function() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = call f1() -> [&mut u1; 2]
            return
        }
        brillig(inline) fn foo f1 {
          b0():
            v0 = allocate -> &mut u1
            store u1 1 at v0
            v2 = allocate -> &mut u1
            store u1 0 at v2
            v4 = make_array [v0, v2] : [&mut u1; 2]
            return v4
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store_in_make_array_used_in_array_get_that_returns_result() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut u1
            store u1 0 at v1
            v3 = make_array [v1] : [&mut u1]
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(u32 0)
          b2():
            jmp b3(u32 0)
          b3(v0: u32):
            constrain v0 == u32 0
            v6 = array_get v3, index v0 -> &mut u1
            v7 = load v6 -> u1
            return v7
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store_in_diff_block_from_make_array_used_in_array_get_that_returns_result() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut u1
            store u1 0 at v1
            jmpif u1 1 then: b1, else: b2
          b1():
            v3 = make_array [v1] : [&mut u1]
            jmp b3(u32 0)
          b2():
            jmp b3(u32 0)
          b3(v0: u32):
            constrain v0 == u32 0
            v6 = array_get v3, index v0 -> &mut u1
            v7 = load v6 -> u1
            return v7
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn remove_last_store_in_make_array_that_is_never_used() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u1
            store u1 1 at v0
            jmp b1()
          b1():
            v2 = make_array [v0] : [&mut u1]
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut u1
            jmp b1()
          b1():
            v1 = make_array [v0] : [&mut u1]
            return
        }
        ");
    }

    #[test]
    fn keep_last_store_in_make_array_returned_from_function_separate_blocks() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = call f1(v0) -> [&mut u32; 1]
            v3 = array_get v1, index u32 0 -> &mut u32
            store u32 1 at v3
            v5 = load v3 -> u1
            return v5
        }
        brillig(inline_always) fn foo f1 {
          b0(v0: u32):
            v1 = allocate -> &mut u32
            store v0 at v1
            v2 = allocate -> &mut u32
            store u32 0 at v2
            jmp b1()
          b1():
            v4 = load v2 -> u32
            v6 = eq v4, u32 1
            jmpif v6 then: b2, else: b3
          b2():
            jmp b4()
          b3():
            v7 = load v2 -> u32
            v8 = add v7, u32 1
            store v8 at v2
            jmp b5()
          b4():
            v9 = make_array [v1] : [&mut u32; 1]
            return v9
          b5():
            jmp b1()
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v2 = call f1(v0) -> [&mut u32; 1]
            v4 = array_get v2, index u32 0 -> &mut u32
            store u32 1 at v4
            return u32 1
        }
        brillig(inline_always) fn foo f1 {
          b0(v0: u32):
            v1 = allocate -> &mut u32
            store v0 at v1
            v2 = allocate -> &mut u32
            store u32 0 at v2
            jmp b1()
          b1():
            v4 = load v2 -> u32
            v6 = eq v4, u32 1
            jmpif v6 then: b2, else: b3
          b2():
            jmp b4()
          b3():
            v7 = add v4, u32 1
            store v7 at v2
            jmp b5()
          b4():
            v8 = make_array [v1] : [&mut u32; 1]
            return v8
          b5():
            jmp b1()
        }
        ");
    }

    #[test]
    fn keep_last_store_in_make_array_where_aliases_are_none() {
        let src = "
        brillig(inline) fn foo f1 {
          b0(v0: &mut u1):
            v1 = call f2() -> &mut u1
            store u1 1 at v1
            v3 = make_array [v1] : [&mut u1; 1]
            return v3
        }
        brillig(inline) fn get_ref f2 {
          b0():
            v0 = allocate -> &mut u1
            store u1 1 at v0
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store_nested_array_used_in_array_get_in_separate_block() {
        let src = "
        acir(inline) fn main f0 {
          b0():
            v2 = allocate -> &mut u1
            store u1 1 at v2
            v4 = make_array [v2] : [&mut u1; 1]
            v5 = allocate -> &mut u1
            store u1 1 at v5
            v6 = make_array [v5] : [&mut u1; 1]
            v7 = make_array [v4, v6] : [[&mut u1; 1]; 2]
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(u32 0)
          b2():
            jmp b3(u32 1)
          b3(v0: u32):
            v11 = lt v0, u32 2
            constrain v11 == u1 1
            v12 = array_get v7, index v0 -> [&mut u1; 1]
            v13 = array_get v12, index u32 0 -> &mut u1
            v14 = load v13 -> u1
            jmpif v14 then: b4, else: b5
          b4():
            jmp b6(u1 1)
          b5():
            jmp b6(u1 0)
          b6(v1: u1):
            return v1
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_stores_with_aliased_references() {
        // Ensure `store v8 at v1` is not removed from the program
        // just because there is a subsequent `store v10 at v1`.
        // In this case `v1` is aliased to `*v3` so the store is significant.
        let src = "
            acir(inline) fn main f0 {
              b0():
                v1 = allocate -> &mut Field
                store Field 0 at v1
                v3 = allocate -> &mut &mut Field
                store v1 at v3
                jmp b1(u32 10)
              b1(v0: u32):
                v6 = lt v0, u32 11
                jmpif v6 then: b2, else: b3
              b2():
                v8 = cast v0 as Field
                store v8 at v1
                v9 = load v3 -> &mut Field
                v10 = load v9 -> Field
                store v10 at v1
                v12 = unchecked_add v0, u32 1
                jmp b1(v12)
              b3():
                v7 = load v1 -> Field
                return v7
            }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn unknown_aliases() {
        // This is just a test to ensure the case where `aliases.is_unknown()` is tested.
        // Here, `v8 = load v0 -> Field` cannot be removed and replaced with `Field 10`
        // because `v6` is passed to another function and we don't know what that reference
        // points to, and it could potentially change the value of `v0`.
        let src = "
        brillig(inline) fn foo f0 {
          b0(v0: &mut Field):
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            jmp b1(Field 0)
          b1(v1: Field):
            jmpif u1 0 then: b2, else: b3
          b2():
            store Field 10 at v0
            v6 = load v2 -> &mut Field
            call f0(v6)
            v8 = load v0 -> Field
            jmp b1(v8)
          b3():
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn does_not_remove_store_to_function_parameter() {
        // The last store can't be removed as it stores a value in a function parameter
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field):
            jmp b1()
          b1():
            store Field 4 at v0
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn does_not_remove_store_to_return_value() {
        // The last store can't be removed as it stores into a reference that is returned
        let src = "
        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 4 at v0
            jmp b1()
          b1():
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn does_not_remove_store_to_address_used_in_terminator() {
        // The store here shouldn't be removed as its address is eventually put inside
        // an array that's used in a terminator value, `b1(v7)`.
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v6 = allocate -> &mut u1
            store u1 1 at v6
            v7 = make_array [v6, Field 1] : [(&mut u1, Field); 1]
            jmp b1(v7)
          b1(v1: [(&mut u1, Field); 1]):
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn removes_last_store_in_single_block() {
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v1 = load v0 -> [Field; 2]
            store v1 at v0
            return
        }

        brillig(inline) impure fn append_note_hashes_with_logs f1 {
          b0(v0: &mut [Field; 2]):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            v1 = load v0 -> [Field; 2]
            return
        }
        brillig(inline) impure fn append_note_hashes_with_logs f1 {
          b0(v0: &mut [Field; 2]):
            return
        }
        ");
    }

    #[test]
    fn does_not_remove_last_store_in_single_block() {
        // Even though v0 is a reference passed to a function, the store that comes next
        // isn't removed because it's considered an "input reference" and we can't currently
        // tell if that value is going to be returned from the function.
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            call f1(v0)
            v1 = load v0 -> [Field; 2]
            store v1 at v0
            return
        }

        brillig(inline) impure fn append_note_hashes_with_logs f1 {
          b0(v0: &mut [Field; 2]):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut [Field; 2]
            call f1(v0)
            v2 = load v0 -> [Field; 2]
            store v2 at v0
            return
        }
        brillig(inline) impure fn append_note_hashes_with_logs f1 {
          b0(v0: &mut [Field; 2]):
            return
        }
        ");
    }

    #[test]
    fn if_aliases_each_branch() {
        let src = "
            brillig(inline) predicate_pure fn main f0 {
              b0(v0: u1):
                v1 = allocate -> &mut Field
                store Field 0 at v1
                v3 = allocate -> &mut Field
                store Field 1 at v3
                v5 = not v0
                v6 = if v0 then v1 else (if v5) v3
                store Field 9 at v6
                v8 = load v1 -> Field
                constrain v8 == Field 9
                v9 = load v3 -> Field
                constrain v9 == Field 1
                return
            }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn reuses_last_load_from_single_predecessor_block() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field):
            v2 = load v0 -> Field
            jmp b1()
          b1():
            v18 = load v0 -> Field
            return v18
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field):
            v1 = load v0 -> Field
            jmp b1()
          b1():
            return v1
        }
        ");
    }

    #[test]
    fn reuses_last_load_from_multiple_indirect_predecessor_block() {
        let src = r#"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: u1):
            v2 = load v0 -> Field
            jmpif v1 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            v18 = load v0 -> Field
            return v18
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: u1):
            v2 = load v0 -> Field
            jmpif v1 then: b1, else: b2
          b1():
            jmp b3()
          b2():
            jmp b3()
          b3():
            return v2
        }
        ");
    }

    #[test]
    fn store_load_from_array_get() {
        let src = "
        brillig(inline) fn main f0 {
          b0():
            v1 = allocate -> &mut u1
            store u1 0 at v1
            v3 = make_array [v1] : [&mut u1]
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(u32 0, u32 1)
          b2():
            jmp b3(u32 0, u32 1)
          b3(v0: u32, v8: u32):
            constrain v0 == u32 0
            v6 = array_get v3, index v0 -> &mut u1
            store u1 0 at v6
            v7 = load v6 -> u1
            return v7
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) fn main f0 {
          b0():
            v2 = allocate -> &mut u1
            store u1 0 at v2
            v4 = make_array [v2] : [&mut u1]
            jmpif u1 1 then: b1, else: b2
          b1():
            jmp b3(u32 0, u32 1)
          b2():
            jmp b3(u32 0, u32 1)
          b3(v0: u32, v1: u32):
            constrain v0 == u32 0
            v8 = array_get v4, index v0 -> &mut u1
            store u1 0 at v8
            return u1 0
        }
        ");
    }

    #[test]
    fn does_not_remove_store_to_potentially_aliased_address() {
        // This is a regression test for https://github.com/noir-lang/noir/pull/9613
        // In that PR all tests passed but the sync to Aztec-Packages failed.
        // That PR had `store v3 at v1` incorrectly removed.
        // Even though v3 is what was in v1 and it might seem that there's no
        // need to put v3 back in v1, v0 and v1 might be aliases so `store v2 at v0`
        // might change the value at v1, so the next store to v1 must be preserved.
        let src = r#"
        acir(inline) fn create_note f0 {
          b0(v0: &mut [Field; 1], v1: &mut [Field; 1]):
            v2 = load v0 -> [Field; 1]
            v3 = load v1 -> [Field; 1]
            store v2 at v0
            store v3 at v1
            v4 = load v1 -> [Field; 1]
            v6 = make_array [Field 0] : [Field; 1]
            store v6 at v1
            return v1, v4
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn create_note f0 {
          b0(v0: &mut [Field; 1], v1: &mut [Field; 1]):
            v2 = load v0 -> [Field; 1]
            v3 = load v1 -> [Field; 1]
            store v2 at v0
            store v3 at v1
            v5 = make_array [Field 0] : [Field; 1]
            store v5 at v1
            return v1, v3
        }
        ");
    }

    #[test]
    fn does_not_replace_load_with_value_when_one_of_its_predecessors_changes_it() {
        // This is another regression test for https://github.com/noir-lang/noir/pull/9613
        // There, in `v5 = load v0 -> Field` it was incorrectly assumed that v5 could
        // be replaced with `Field 0`, but another predecessor, through an alias
        // (v1) conditionally changes its value to `Field 1`.
        let src = r#"
        acir(inline) fn create_note f0 {
          b0(v0: &mut Field):
            store Field 0 at v0
            jmpif u1 0 then: b1, else: b2
          b1():
            v5 = load v0 -> Field
            return v5
          b2():
            jmp b3(v0)
          b3(v1: &mut Field):
            store Field 1 at v1
            jmp b1()
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn store_to_reference_from_array_get_is_not_lost() {
        // `store Field 9 at v7` was incorrectly removed because of a bug
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u1):
            v2 = allocate -> &mut Field
            store Field 0 at v2
            jmpif v0 then: b1, else: b2
          b1():
            v5 = make_array [v2] : [&mut Field; 1]
            jmp b3(v5)
          b2():
            v4 = make_array [v2] : [&mut Field; 1]
            jmp b3(v4)
          b3(v1: [&mut Field; 1]):
            v7 = array_get v1, index u32 0 -> &mut Field
            store Field 9 at v7
            v9 = array_get v1, index u32 0 -> &mut Field
            v10 = load v9 -> Field
            constrain v10 == Field 9
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn aliases_block_parameter_to_its_argument() {
        // Here:
        // - v0 and v1 are aliases of each other
        // - v2 must be an alias of v0 (there was a bug around this)
        // - v3 must be an alias of v1 (same as previous point)
        // - `v4 = load v2` cannot be replaced with `Field 2` because
        //   v2 and v3 are also potentially aliases of each other
        let src = r#"
        acir(inline) fn create_note f0 {
          b0(v0: &mut Field, v1: &mut Field):
            jmp b1(v0, v1)
          b1(v2: &mut Field, v3: &mut Field):
            store Field 2 at v2
            store Field 3 at v3
            v4 = load v2 -> Field
            return v4
        }
        "#;
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn aliases_unknown_block_arguments() {
        // Here:
        // - v0 and v1 both have unknown alias sets
        // - v0 and v1 are potentially aliases of each other
        // - v2 must be an alias of v0
        // - v3 must be an alias of v1
        // - `v4 = load v2` cannot be replaced with `Field 2` because
        //   v2 and v3 are also potentially aliases of each other
        let ssa = r#"
        acir(inline) fn create_note f0 {
          b0(v0: &mut Field, v1_arr: [&mut Field; 1]):
            v1 = array_get v1_arr, index u32 0 -> &mut Field
            jmp b1(v0, v1)
          b1(v2: &mut Field, v3: &mut Field):
            store Field 2 at v2
            store Field 3 at v3
            v4 = load v2 -> Field
            return v4
        }
        "#;
        assert_ssa_does_not_change(ssa, Ssa::mem2reg);
    }

    #[test]
    fn does_not_remove_store_used_in_if_then() {
        let src = "
        brillig(inline) fn func f0 {
          b0(v0: &mut u1, v1: u1):
            v2 = allocate -> &mut u1
            store v1 at v2
            jmp b1()
          b1():
            v3 = not v1
            v4 = if v1 then v0 else (if v3) v2
            v6 = call f0(v4, v1) -> Field
            return v6
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn block_argument_is_alias_of_block_parameter_1() {
        // Here the last load can't be replaced with `Field 0` as v0 and v1 are aliases of one another.
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            jmp b1(v0)
          b1(v1: &mut Field):
            store Field 1 at v1
            v2 = load v0 -> Field
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn block_argument_is_alias_of_block_parameter_2() {
        // Here the last load can't be replaced with `Field 1` as v0 and v1 are aliases of one another.
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            jmp b1(v0)
          b1(v1: &mut Field):
            store Field 1 at v1
            store Field 2 at v0
            v2 = load v1 -> Field
            return v2
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn nested_alias_in_array() {
        let src = "
        acir(inline) fn regression_2445_deeper_ref f2 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            store v0 at v2
            v3 = allocate -> &mut &mut &mut Field
            store v2 at v3
            v4 = make_array [v3, v3] : [&mut &mut &mut Field; 2]
            v5 = allocate -> &mut [&mut &mut &mut Field; 2]
            store v4 at v5
            v6 = load v5 -> [&mut &mut &mut Field; 2]
            v8 = array_get v6, index u32 0 -> &mut &mut &mut Field
            v9 = load v8 -> &mut &mut Field
            v10 = load v9 -> &mut Field
            store Field 1 at v10
            v12 = load v5 -> [&mut &mut &mut Field; 2]
            v14 = array_get v12, index u32 1 -> &mut &mut &mut Field
            v15 = load v14 -> &mut &mut Field
            v16 = load v15 -> &mut Field
            store Field 2 at v16
            v18 = load v0 -> Field
            v19 = eq v18, Field 2
            constrain v18 == Field 2
            v20 = load v3 -> &mut &mut Field
            v21 = load v20 -> &mut Field
            v22 = load v21 -> Field
            v23 = eq v22, Field 2
            constrain v22 == Field 2
            v24 = load v5 -> [&mut &mut &mut Field; 2]
            v25 = array_get v24, index u32 0 -> &mut &mut &mut Field
            v26 = load v25 -> &mut &mut Field
            v27 = load v26 -> &mut Field
            v28 = load v27 -> Field
            v29 = eq v28, Field 2
            constrain v28 == Field 2
            v30 = load v5 -> [&mut &mut &mut Field; 2]
            v31 = array_get v30, index u32 1 -> &mut &mut &mut Field
            v32 = load v31 -> &mut &mut Field
            v33 = load v32 -> &mut Field
            v34 = load v33 -> Field
            v35 = eq v34, Field 2
            constrain v34 == Field 2
            return
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        // We expect the final references to both be resolved to `Field 2` and thus the constrain instructions
        // will be trivially true and simplified out.
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) fn regression_2445_deeper_ref f0 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            v2 = allocate -> &mut &mut Field
            v3 = allocate -> &mut &mut &mut Field
            v4 = make_array [v3, v3] : [&mut &mut &mut Field; 2]
            v5 = allocate -> &mut [&mut &mut &mut Field; 2]
            store Field 1 at v0
            return
        }
        ");
    }

    #[test]
    fn does_not_reuse_load_from_aliased_array_element() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: u32):
            v3 = make_array [v0] : [&mut Field; 1]
            v4 = array_set v3, index v2, value v1
            v5 = load v0 -> Field
            v6 = array_get v4, index v2 -> &mut Field
            store Field 0 at v6
            v7 = load v0 -> Field
            return v7
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field, v2: u32):
            v3 = make_array [v0] : [&mut Field; 1]
            v4 = array_set v3, index v2, value v1
            v5 = load v0 -> Field
            constrain v2 == u32 0, "Index out of bounds"
            v7 = array_get v4, index u32 0 -> &mut Field
            store Field 0 at v7
            v9 = load v0 -> Field
            return v9
        }
        "#);
    }

    #[test]
    fn does_not_remove_store_from_aliased_array_element() {
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = make_array [v1] : [&mut Field; 1]
            v5 = array_set v3, index v0, value v1
            v6 = array_get v5, index v0 -> &mut Field
            store Field 100 at v6
            v8 = load v1 -> Field
            constrain v8 == Field 100
            return v8
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn main f0 {
          b0(v0: u32):
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = make_array [v1] : [&mut Field; 1]
            v4 = array_set v3, index v0, value v1
            constrain v0 == u32 0, "Index out of bounds"
            v6 = array_get v4, index u32 0 -> &mut Field
            store Field 100 at v6
            v8 = load v1 -> Field
            constrain v8 == Field 100
            return v8
        }
        "#);
    }

    #[test]
    fn return_references_1() {
        // Here the last load can't be replaced with `Field 1` as v0 and v1 are potentially
        // aliases of one another (in our logic the alias set of v0 and v1 will be unknown)
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0, v1 = call f1() -> (&mut Field, &mut Field)
            store Field 0 at v0
            store Field 1 at v1
            v2 = load v0 -> Field
            return v2
        }

        brillig(inline) impure fn f1 f1 {
          b0():
            v0 = allocate -> &mut Field
            store Field 0 at v0
            return v0, v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn return_references_2() {
        // Here the last load can't be replaced with `Field 1` as v0 and v1 are potentially
        // aliases of one another (in our logic the alias set of v0 and v1 will be unknown)
        let src = "
        brillig(inline) impure fn main f0 {
          b0():
            v0 = allocate -> &mut Field
            v1 = call f1(v0) -> &mut Field
            store Field 0 at v0
            store Field 1 at v1
            v2 = load v0 -> Field
            return v2
        }

        brillig(inline) impure fn f1 f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    // This test is currently asserting undesirable behavior!
    // See https://github.com/noir-lang/noir/issues/9745 for more info.
    #[test]
    fn prefers_preserving_load_in_same_block_as_store() {
        // Here we have the mutable references `v2`, `v3`, `v4` and `v5` loaded in `b2` and then again in
        // `b4`. The values loaded in `b2` are never used however `b2` dominates `b4` (so b4 can use the values loaded
        // in `b2`).
        //
        // The compiler can then choose to discard either the loads in `b2` or in `b4`. In some situations it may be
        // preferable for the compiler to keep the loads in `b2` (e.g. we want to keep the load of `v4` as it's used in
        // `b2` itself), but here it's preferable for us to load most of these values only once we enter `b4`.
        let src = r#"
        brillig(inline) fn perform_duplex f5 {
          b0(v2: &mut [Field; 3], v3: &mut [Field; 4], v4: &mut u32, v5: &mut u1):
            jmp b1(u32 0)
          b1(v6: u32):
            v8 = lt v6, u32 3
            jmpif v8 then: b2, else: b3
          b2():
            v20 = load v2 -> [Field; 3]
            v21 = load v3 -> [Field; 4]
            v22 = load v4 -> u32
            v23 = load v5 -> u1
            v24 = lt v6, v22
            jmpif v24 then: b4, else: b5
          b3():
            v9 = load v2 -> [Field; 3]
            v10 = load v3 -> [Field; 4]
            v11 = load v4 -> u32
            v12 = load v5 -> u1
            inc_rc v10
            v15 = call poseidon2_permutation(v10) -> [Field; 4]
            v16 = load v2 -> [Field; 3]
            v17 = load v3 -> [Field; 4]
            v18 = load v4 -> u32
            v19 = load v5 -> u1
            store v16 at v2
            store v15 at v3
            store v18 at v4
            store v19 at v5
            return
          b4():
            v25 = load v2 -> [Field; 3]
            v26 = load v3 -> [Field; 4]
            v27 = load v4 -> u32
            v28 = load v5 -> u1
            v29 = lt v6, u32 4
            constrain v29 == u1 1, "Index out of bounds"
            v31 = array_get v26, index v6 -> Field
            v32 = load v2 -> [Field; 3]
            v33 = load v3 -> [Field; 4]
            v34 = load v4 -> u32
            v35 = load v5 -> u1
            v36 = lt v6, u32 3
            constrain v36 == u1 1, "Index out of bounds"
            v37 = array_get v32, index v6 -> Field
            v38 = add v31, v37
            v39 = load v2 -> [Field; 3]
            v40 = load v3 -> [Field; 4]
            v41 = load v4 -> u32
            v42 = load v5 -> u1
            v43 = lt v6, u32 4
            constrain v43 == u1 1, "Index out of bounds"
            v44 = array_set v40, index v6, value v38
            v46 = unchecked_add v6, u32 1
            store v39 at v2
            store v44 at v3
            store v41 at v4
            store v42 at v5
            jmp b5()
          b5():
            v47 = unchecked_add v6, u32 1
            jmp b1(v47)
        }"#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r#"
        brillig(inline) fn perform_duplex f0 {
          b0(v0: &mut [Field; 3], v1: &mut [Field; 4], v2: &mut u32, v3: &mut u1):
            jmp b1(u32 0)
          b1(v4: u32):
            v7 = lt v4, u32 3
            jmpif v7 then: b2, else: b3
          b2():
            v14 = load v0 -> [Field; 3]
            v15 = load v1 -> [Field; 4]
            v16 = load v2 -> u32
            v17 = load v3 -> u1
            v18 = lt v4, v16
            jmpif v18 then: b4, else: b5
          b3():
            v8 = load v0 -> [Field; 3]
            v9 = load v1 -> [Field; 4]
            v10 = load v2 -> u32
            v11 = load v3 -> u1
            inc_rc v9
            v13 = call poseidon2_permutation(v9) -> [Field; 4]
            store v8 at v0
            store v13 at v1
            store v10 at v2
            store v11 at v3
            return
          b4():
            v20 = lt v4, u32 4
            constrain v20 == u1 1, "Index out of bounds"
            v22 = array_get v15, index v4 -> Field
            v23 = lt v4, u32 3
            constrain v23 == u1 1, "Index out of bounds"
            v24 = array_get v14, index v4 -> Field
            v25 = add v22, v24
            v26 = lt v4, u32 4
            constrain v26 == u1 1, "Index out of bounds"
            v27 = array_set v15, index v4, value v25
            v29 = unchecked_add v4, u32 1
            store v14 at v0
            store v27 at v1
            store v16 at v2
            store v17 at v3
            jmp b5()
          b5():
            v30 = unchecked_add v4, u32 1
            jmp b1(v30)
        }
        "#);
    }

    #[test]
    fn analyzes_instruction_simplified_to_multiple() {
        // This is a test to make sure that if an instruction is simplified to multiple instructions,
        // like in the case of `vector_push_back`, those are handled correctly.
        let src = r#"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v4 = allocate -> &mut u1
            store u1 0 at v4
            v7 = make_array [v4] : [&mut u1]
            v8 = allocate -> &mut u1
            store u1 0 at v8
            v11, v12 = call vector_push_back(u32 2, v7, v8) -> (u32, [&mut u1])
            v16 = array_get v12, index u32 1 -> &mut u1
            v17 = load v16 -> u1
            jmpif v17 then: b1, else: b2
          b1():
            jmp b3(v12)
          b2():
            jmp b3(v12)
          b3(v2: [&mut u1]):
            v23 = array_get v2, index u32 0 -> &mut u1
            v24 = load v23 -> u1
            return v24
        }
        "#;

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        brillig(inline) predicate_pure fn main f0 {
          b0():
            v1 = allocate -> &mut u1
            store u1 0 at v1
            v3 = make_array [v1] : [&mut u1]
            v4 = allocate -> &mut u1
            store u1 0 at v4
            v5 = make_array [v1, v4] : [&mut u1]
            v7 = array_set v5, index u32 2, value v4
            v8 = make_array [v1, v4] : [&mut u1]
            jmpif u1 0 then: b1, else: b2
          b1():
            jmp b3(v8)
          b2():
            jmp b3(v8)
          b3(v0: [&mut u1]):
            v10 = array_get v0, index u32 0 -> &mut u1
            v11 = load v10 -> u1
            return v11
        }
        "
        );
    }

    #[test]
    fn simplify_to_global_instruction() {
        // We want to have more global instructions than instructions in the function as we want
        // to test that we never attempt to access a function's instruction map using a global instruction index.
        // If we were to access the function's instruction map using the global instruction index in this case
        // we would expect to hit an OOB panic.
        // We also want to make sure that we simplify to a make array instructions as global constants will
        // resolve directly to a constant.
        let src = "
        g0 = Field 1
        g1 = Field 2
        g2 = Field 3
        g3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g5 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g6 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g7 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g8 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g9 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g10 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g11 = make_array [g10, g10, g10] : [[Field; 3]; 3]

        acir(inline) fn main f0 {
          b0():
            v0 = allocate -> &mut [[Field; 3]; 3]
            store g11 at v0
            v1 = load v0 -> [[Field; 3]; 3]
            v3 = array_get v1, index u32 0 -> [Field; 3]
            return v3
        }
        ";
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_ssa_snapshot!(ssa, @r"
        g0 = Field 1
        g1 = Field 2
        g2 = Field 3
        g3 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g4 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g5 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g6 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g7 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g8 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g9 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g10 = make_array [Field 1, Field 2, Field 3] : [Field; 3]
        g11 = make_array [g10, g10, g10] : [[Field; 3]; 3]

        acir(inline) fn main f0 {
          b0():
            v12 = allocate -> &mut [[Field; 3]; 3]
            return g10
        }
        ");
    }

    #[test]
    fn correctly_aliases_references_in_call_return_values_to_arguments_with_simple_values() {
        // Here v2 could be an alias of v1, and in fact it is, so the second store to v1
        // should invalidate the value at v2.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store v0 at v1
            v2 = call f1(v1) -> &mut Field
            store Field 1 at v2
            store Field 2 at v1
            v8 = load v2 -> Field
            constrain v8 == Field 2
            return
        }
        acir(inline) fn helper f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn correctly_aliases_references_in_call_return_values_to_arguments_with_arrays() {
        // Similar to the previous test except that arrays with references are passed and returned
        let src = "
        acir(inline) fn main f0 {
          b0(v0: Field):
            v1 = allocate -> &mut Field
            store v0 at v1
            v3 = make_array [v1] : [&mut Field; 1]
            v4 = call f1(v3) -> [&mut Field; 1]
            v5 = array_get v4, index u32 0 -> &mut Field
            store Field 1 at v5
            store Field 2 at v1
            v8 = load v5 -> Field
            constrain v8 == Field 2
            return
        }
        acir(inline) fn helper f1 {
          b0(v0: [&mut Field; 1]):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn correctly_aliases_references_in_call_return_values_to_arguments_with_existing_aliases() {
        // Here v0 and v1 are aliases of each other. When we pass v1 to the call v2 could be
        // an alias of v1. Then when storing to v2 we should invalidate the value at v1 but also
        // at v0.
        let src = "
        acir(inline) fn main f0 {
          b0(v0: &mut Field, v1: &mut Field):
            store Field 0 at v1
            v2 = call f1(v1) -> &mut Field
            store Field 1 at v2
            store Field 2 at v0
            v8 = load v2 -> Field
            constrain v8 == Field 2
            return
        }
        acir(inline) fn helper f1 {
          b0(v0: &mut Field):
            return v0
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_last_store() {
        // This test check that used Store instructions are not simplified:
        // - Allocate v1 and v3 and store into them
        // - Put them in array v4 = [v1, v3]
        // - Store v4 at v5
        // - Pass v5 a function as an argument, so that v5 is used
        // - This should keep the store to v5, and recursively to v1 and v3
        let src = "
    brillig(inline) fn main f0 {
      b0():
        v1 = allocate -> &mut Field
        store Field 99 at v1
        v3 = allocate -> &mut Field
        store Field 88 at v3
        v4 = make_array [v1, v3] : [&mut Field; 2]
        v5 = allocate -> &mut [&mut Field; 2]
        store v4 at v5
        v6 = call f1(v5) -> Field
        return v6
    }
    brillig(inline) fn helper f1 {
      b0(v0: &mut [&mut Field; 2]):
        return Field 0
    }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn regression_10070() {
        // Here v6 and v7 aliases v2 expression.
        // When storing to v3 we may modify value referenced by v2 depending on the taken branch
        // This must invalidate v8's value previously set.
        let src = "
        brillig(inline) fn main f0 {
          b0(v0: [&mut Field; 1], v1: u1):
            v3 = allocate -> &mut Field
            v4 = allocate -> &mut Field
            jmpif v1 then: b1, else: b2
          b1():
            v7 = array_set v0, index u32 0, value v3
            jmp b3(v7)
          b2():
            v6 = array_set v0, index u32 0, value v4
            jmp b3(v6)
          b3(v2: [&mut Field; 1]):
            v8 = array_get v2, index u32 0 -> &mut Field
            store Field 1 at v8
            store Field 2 at v3
            store Field 3 at v4
            v12 = load v8 -> Field
            return v12
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn regression_10020() {
        // v14 = add v12, v13 is NOT replaced by v13 = add v12, Field 1
        let src = "
        acir(inline) predicate_pure fn main f0 {
          b0():
            v1 = allocate -> &mut Field
            store Field 0 at v1
            v3 = allocate -> &mut Field
            store Field 0 at v3
            v4 = make_array [v1, v3] : [&mut Field; 2]
            v5 = allocate -> &mut Field
            store Field 0 at v5
            jmp b1(u32 0)
          b1(v0: u32):
            v7 = eq v0, u32 0
            jmpif v7 then: b2, else: b3
          b2():
            v9 = array_get v4, index v0 -> &mut Field
            store Field 1 at v9
            store Field 2 at v1
            v12 = load v5 -> Field
            v13 = load v9 -> Field
            v14 = add v12, v13
            store v14 at v5
            v16 = unchecked_add v0, u32 1
            jmp b1(v16)
          b3():
            v8 = load v5 -> Field
            return v8
        }";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn keep_store_to_reference_in_array_selected_by_if() {
        // Regression test: The store to v5 should NOT be removed because:
        // - v5 is placed inside array v6
        // - v6 can be selected via the `if` expression based on loading from another array
        // - The selected array is stored back to v4
        // - On the next loop iteration, loading from v4 and doing array_get can return v5
        // - Then loading from v5 would read uninitialized memory if the store was removed
        let src = r"
        brillig(inline) predicate_pure fn foo f1 {
          b0():
            v1 = allocate -> &mut u1
            store u1 0 at v1
            v3 = make_array [v1] : [&mut u1; 1]
            v4 = allocate -> &mut [&mut u1; 1]
            store v3 at v4
            v5 = allocate -> &mut u1
            store u1 0 at v5
            v6 = make_array [v5] : [&mut u1; 1]
            jmp b1(u32 0)
          b1(v0: u32):
            v9 = lt v0, u32 3
            jmpif v9 then: b2, else: b3
          b2():
            v10 = load v4 -> [&mut u1; 1]
            v11 = array_get v10, index u32 0 -> &mut u1
            v12 = load v11 -> u1
            inc_rc v10
            v13 = not v12
            inc_rc v6
            v14 = if v12 then v10 else (if v13) v6
            store v14 at v4
            v16 = unchecked_add v0, u32 1
            jmp b1(v16)
          b3():
            return
        }
        ";
        assert_ssa_does_not_change(src, Ssa::mem2reg);
    }

    #[test]
    fn aliases_in_a_loop() {
        let src = "
      acir(inline) impure fn main f0 {
        b0():
          v1 = call f1() -> Field
          return v1
      }
      brillig(inline) impure fn foo f1 {
        b0():
          v0 = allocate -> &mut Field
          store Field 3405691582 at v0
          v4 = call f2(v0, Field 3735928559) -> Field
          return v4
      }
      brillig(inline) impure fn bar f2 {
        b0(v0: &mut Field, v1: Field):
          v2 = allocate -> &mut &mut Field
          store v0 at v2
          v3 = allocate -> &mut Field
          store v1 at v3
          v4 = allocate -> &mut Field
          store Field 0 at v4
          jmp b1()
        b1():
          v6 = load v4 -> Field
          v8 = eq v6, Field 2
          jmpif v8 then: b2, else: b3
        b2():
          jmp b4()
        b3():
          v9 = load v4 -> Field
          v11 = add v9, Field 1
          store v11 at v4
          store Field 3735928559 at v3
          v13 = load v2 -> &mut Field
          v14 = load v13 -> Field
          v15 = load v3 -> Field
          store v14 at v3
          store v3 at v2
          jmp b1()
        b4():
          v16 = load v3 -> Field
          return v16
      }
      ";
        let ssa = Ssa::from_str(src).unwrap();
        let result = ssa.interpret(vec![]).unwrap();
        assert_eq!(result, vec![Value::field(3735928559u128.into())]);
        let ssa = ssa.mem2reg();
        let mem2reg_result = ssa.interpret(vec![]).unwrap();
        assert_eq!(result, mem2reg_result);

        // We expect `store Field 3735928559 at v3` to remain.
        // If alias analysis does not account for loops, when mem2reg sees `store v14 at v3`
        // it will view the first store as being overwritten.
        // However, the following instruction `store v3 at v2` makes v3 and the loaded result of v2 possibly alias one another.
        // In the next loop iteration, `v13 = load v2` were are returning v3 (through the alias) and then loading
        // from the alias with `v14 = load v13`. We should only remove a last store if there are no (unremoved) loads from that address
        // between the last store and the current one. Thus, `store Field 3735928559 at v3` should not be removed
        // as we load from v13 (an alias of v3) before we reach `store v14 at v3`.
        assert_ssa_snapshot!(ssa, @r"
      acir(inline) impure fn main f0 {
        b0():
          v1 = call f1() -> Field
          return v1
      }
      brillig(inline) impure fn foo f1 {
        b0():
          v0 = allocate -> &mut Field
          store Field 3405691582 at v0
          v4 = call f2(v0, Field 3735928559) -> Field
          return v4
      }
      brillig(inline) impure fn bar f2 {
        b0(v0: &mut Field, v1: Field):
          v2 = allocate -> &mut &mut Field
          store v0 at v2
          v3 = allocate -> &mut Field
          store v1 at v3
          v4 = allocate -> &mut Field
          store Field 0 at v4
          jmp b1()
        b1():
          v6 = load v4 -> Field
          v8 = eq v6, Field 2
          jmpif v8 then: b2, else: b3
        b2():
          jmp b4()
        b3():
          v10 = add v6, Field 1
          store v10 at v4
          store Field 3735928559 at v3
          v12 = load v2 -> &mut Field
          v13 = load v12 -> Field
          store v13 at v3
          store v3 at v2
          jmp b1()
        b4():
          v14 = load v3 -> Field
          return v14
      }
      ");
    }

    #[test]
    fn set_reference_in_array_from_separate_block() {
        let src = "
      brillig(inline) impure fn bar f2 {
        b0(v0: [&mut u1; 1]):
          v1 = allocate -> &mut [&mut u1; 1]
          store v0 at v1
          v2 = allocate -> &mut u1
          store u1 1 at v2
          v4 = load v1 -> [&mut u1; 1]
          v6 = array_get v4, index u32 0 -> &mut u1
          v7 = load v6 -> u1
          jmpif v7 then: b1, else: b2
        b1():
          v8 = load v1 -> [&mut u1; 1]
          v9 = array_set v8, index u32 0, value v2
          store v9 at v1
          jmp b2()
        b2():
          v10 = load v1 -> [&mut u1; 1]
          v11 = array_get v10, index u32 0 -> &mut u1
          return v11
      }
      ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        // We expect `store u1 1 at v2` to remain in place as it is used later as a value in an array set in b1
        assert_ssa_snapshot!(ssa, @r"
      brillig(inline) impure fn bar f0 {
        b0(v0: [&mut u1; 1]):
          v1 = allocate -> &mut [&mut u1; 1]
          store v0 at v1
          v2 = allocate -> &mut u1
          store u1 1 at v2
          v5 = array_get v0, index u32 0 -> &mut u1
          v6 = load v5 -> u1
          jmpif v6 then: b1, else: b2
        b1():
          v7 = array_set v0, index u32 0, value v2
          store v7 at v1
          jmp b2()
        b2():
          v8 = load v1 -> [&mut u1; 1]
          v9 = array_get v8, index u32 0 -> &mut u1
          return v9
      }
      ");
    }

    #[test]
    fn missing_make_array_alias_from_array_set_result() {
        let src = "
    acir(inline) predicate_pure fn main f0 {
      b0(v0: u32, v1: u8, v2: u8):
        v3 = allocate -> &mut u8
        store v1 at v3
        v4 = allocate -> &mut [&mut u8; 1]
        v5 = make_array [v3] : [&mut u8; 1]
        store v5 at v4
        constrain v0 == u32 0
        v6 = load v4 -> [&mut u8; 1]
        v7 = array_set v6, index v0, value v3
        v8 = allocate -> &mut u8
        store u8 0 at v8
        v9 = make_array [v8] : [&mut u8; 1]
        v10 = make_array [v7, v9] : [[&mut u8; 1]; 2]
        v11 = array_get v10, index v0 -> [&mut u8; 1]
        v12 = array_get v11, index u32 0 -> &mut u8
        store v2 at v3
        v13 = load v12 -> u8
        constrain v13 == v2
        return
    }
    ";

        let ssa = Ssa::from_str(src).unwrap();
        let result = ssa.interpret(vec![Value::u32(0), Value::u8(0), Value::u8(0)]);
        assert_eq!(result, Ok(vec![]));
        let result = ssa.interpret(vec![Value::u32(0), Value::u8(0), Value::u8(1)]);
        assert_eq!(result, Ok(vec![]));

        let ssa = ssa.mem2reg();
        // Alias tracking should prevent `store v2 at v3` from being removed
        let result = ssa.interpret(vec![Value::u32(0), Value::u8(0), Value::u8(0)]);
        assert_eq!(result, Ok(vec![]));
        let result = ssa.interpret(vec![Value::u32(0), Value::u8(0), Value::u8(1)]);
        assert_eq!(result, Ok(vec![]));

        // Only `store v5 at v4` is safe to remove
        assert_ssa_snapshot!(ssa, @r"
        acir(inline) predicate_pure fn main f0 {
          b0(v0: u32, v1: u8, v2: u8):
            v3 = allocate -> &mut u8
            store v1 at v3
            v4 = allocate -> &mut [&mut u8; 1]
            v5 = make_array [v3] : [&mut u8; 1]
            constrain v0 == u32 0
            v7 = array_set v5, index v0, value v3
            v8 = allocate -> &mut u8
            store u8 0 at v8
            v10 = make_array [v8] : [&mut u8; 1]
            v11 = make_array [v7, v10] : [[&mut u8; 1]; 2]
            v12 = array_get v11, index v0 -> [&mut u8; 1]
            v13 = array_get v12, index u32 0 -> &mut u8
            store v2 at v3
            v14 = load v13 -> u8
            constrain v14 == v2
            return
        }
        ");
    }
}
