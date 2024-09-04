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
mod alias_set;
mod block;

use std::collections::{BTreeMap, BTreeSet};

use fxhash::FxHashMap as HashMap;

use crate::ssa::{
    ir::{
        basic_block::BasicBlockId,
        cfg::ControlFlowGraph,
        function::Function,
        function_inserter::FunctionInserter,
        instruction::{Instruction, InstructionId, TerminatorInstruction},
        post_order::PostOrder,
        types::Type,
        value::{Value, ValueId},
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
    instructions_to_remove: BTreeSet<InstructionId>,

    /// Track a value's last load across all blocks.
    /// If a value is not used in anymore loads we can remove the last store to that value.
    last_loads: HashMap<ValueId, (InstructionId, BasicBlockId, u32)>,

    /// Track whether a load result was used across all blocks.
    load_results: HashMap<ValueId, PerFuncLoadResultContext>,

    /// Track whether a reference was passed into another entry point
    stores_used_in_calls: HashMap<ValueId, Vec<(InstructionId, BasicBlockId)>>,

    /// Flag for tracking whether we had to perform a re-load as part of the Brillig CoW optimization.
    /// Stores made as part of this optimization should not be removed.
    /// We want to catch stores of this nature:
    /// ```text
    /// v3 = load v1
    //  inc_rc v3
    //  v4 = load v1
    //  inc_rc v4
    //  store v4 at v1
    //  store v3 at v2
    /// ```
    ///
    /// We keep track of an optional boolean flag as we go through instructions.
    /// If the flag exists it means we have hit a load instruction.
    /// If the flag is false it means we have processed a single load, while if the flag is true
    /// it means we have performed a re-load.
    /// The field is reset to `None` on every instruction that is not a load, inc_rc, dec_rc, or function call.
    inside_rc_reload: Option<bool>,
}

#[derive(Debug, Clone)]
struct PerFuncLoadResultContext {
    load_counter: u32,
    load_instruction: InstructionId,
    instructions_using_result: Vec<(InstructionId, BasicBlockId)>,
}

impl PerFuncLoadResultContext {
    fn new(load_instruction: InstructionId) -> Self {
        Self { load_counter: 0, load_instruction, instructions_using_result: vec![] }
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
            instructions_to_remove: BTreeSet::new(),
            last_loads: HashMap::default(),
            load_results: HashMap::default(),
            inside_rc_reload: None,
            stores_used_in_calls: HashMap::default(),
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

        let mut loads_removed = HashMap::default();
        for (_, PerFuncLoadResultContext { load_counter, load_instruction, .. }) in
            self.load_results.iter()
        {
            let Instruction::Load { address } = self.inserter.function.dfg[*load_instruction]
            else {
                panic!("Should only have a load instruction here");
            };

            if *load_counter == 0 {
                if let Some(counter) = loads_removed.get_mut(&address) {
                    *counter += 1;
                } else {
                    loads_removed.insert(address, 1);
                }

                self.instructions_to_remove.insert(*load_instruction);
            }
        }

        let mut not_removed_stores: HashMap<ValueId, (InstructionId, u32)> = HashMap::default();
        // If we never load from an address within a function we can remove all stores to that address.
        // This rule does not apply to reference parameters, which we must also check for before removing these stores.
        for (block_id, block) in self.blocks.iter() {
            let block_params = self.inserter.function.dfg.block_parameters(*block_id);
            for (store_address, store_instruction) in block.last_stores.iter() {
                let is_reference_param = block_params.contains(store_address);
                let terminator = self.inserter.function.dfg[*block_id].unwrap_terminator();

                let is_return = matches!(terminator, TerminatorInstruction::Return { .. });
                let remove_load = if is_return {
                    // Determine whether the last store is used in the return value
                    let mut is_return_value = false;
                    terminator.for_each_value(|return_value| {
                        is_return_value = return_value == *store_address || is_return_value;
                    });

                    // If the last load of a store is not part of the block with a return terminator,
                    // we can safely remove this store.
                    let last_load_not_in_return = self
                        .last_loads
                        .get(store_address)
                        .map(|(_, last_load_block, _)| *last_load_block != *block_id)
                        .unwrap_or(true);
                    !is_return_value && last_load_not_in_return
                } else if let (Some((_, _, last_loads_counter)), Some(loads_removed_counter)) =
                    (self.last_loads.get(store_address), loads_removed.get(store_address))
                {
                    *last_loads_counter == *loads_removed_counter
                } else {
                    self.last_loads.get(store_address).is_none()
                };

                let is_not_used_in_reference_param =
                    self.stores_used_in_calls.get(store_address).is_none();
                if remove_load && !is_reference_param && is_not_used_in_reference_param {
                    self.instructions_to_remove.insert(*store_instruction);
                    if let Some((_, counter)) = not_removed_stores.get_mut(store_address) {
                        *counter -= 1;
                    }
                } else if let Some((_, counter)) = not_removed_stores.get_mut(store_address) {
                    *counter += 1;
                } else {
                    not_removed_stores.insert(*store_address, (*store_instruction, 1));
                }
            }
        }

        self.load_results.retain(|_, PerFuncLoadResultContext { load_instruction, .. }| {
            let Instruction::Load { address } = self.inserter.function.dfg[*load_instruction]
            else {
                panic!("Should only have a load instruction here");
            };
            not_removed_stores.contains_key(&address)
        });

        let mut new_instructions = HashMap::default();
        for (store_address, (store_instruction, store_counter)) in not_removed_stores {
            let Instruction::Store { value, .. } = self.inserter.function.dfg[store_instruction]
            else {
                panic!("Should only have a store instruction");
            };

            if store_counter != 0 {
                continue;
            }
            self.instructions_to_remove.insert(store_instruction);

            if let (Some((_, _, last_loads_counter)), Some(loads_removed_counter)) =
                (self.last_loads.get(&store_address), loads_removed.get(&store_address))
            {
                if *last_loads_counter < *loads_removed_counter {
                    panic!("The number of loads removed should not be more than all loads");
                }
            }

            for (
                result,
                PerFuncLoadResultContext {
                    load_counter,
                    load_instruction,
                    instructions_using_result,
                },
            ) in self.load_results.iter()
            {
                let Instruction::Load { address } = self.inserter.function.dfg[*load_instruction]
                else {
                    panic!("Should only have a load instruction here");
                };
                if address != store_address {
                    continue;
                }

                if *load_counter > 0 {
                    self.inserter.map_value(*result, value);
                    for (instruction, block_id) in instructions_using_result {
                        let new_instruction =
                            self.inserter.push_instruction(*instruction, *block_id);
                        if let Some(new_instruction) = new_instruction {
                            new_instructions
                                .insert((*instruction, block_id), Some(new_instruction));
                        } else {
                            new_instructions.insert((*instruction, block_id), None);
                        }
                    }

                    self.instructions_to_remove.insert(*load_instruction);
                }
            }
        }

        // Re-assign or delete any mapped instructions after the final loads were removed.
        for ((old_instruction, block_id), new_instruction) in new_instructions {
            let instructions = self.inserter.function.dfg[*block_id].instructions_mut();
            if let Some(index) = instructions.iter().position(|v| *v == old_instruction) {
                if let Some(new_instruction) = new_instruction {
                    instructions[index] = new_instruction;
                } else {
                    instructions.remove(index);
                }
            }
        }
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

        for (allocation, instruction) in &references.last_stores {
            if let Some(expression) = references.expressions.get(allocation) {
                if let Some(aliases) = references.aliases.get(expression) {
                    let allocation_aliases_parameter =
                        aliases.any(|alias| reference_parameters.contains(&alias));

                    // If `allocation_aliases_parameter` is known to be false
                    if allocation_aliases_parameter == Some(false) {
                        self.instructions_to_remove.insert(*instruction);
                    }
                }
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

        self.inserter.function.dfg[instruction].for_each_value(|value| {
            if let Some(PerFuncLoadResultContext {
                load_counter, instructions_using_result, ..
            }) = self.load_results.get_mut(&value)
            {
                *load_counter += 1;
                instructions_using_result.push((instruction, block_id));
            }
        });

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

                    references.expressions.insert(result, Expression::Other(result));
                    references.aliases.insert(Expression::Other(result), AliasSet::known(result));
                    references.set_known_value(result, address);

                    self.load_results.insert(result, PerFuncLoadResultContext::new(instruction));

                    let load_counter =
                        if let Some((_, _, load_counter)) = self.last_loads.get(&address) {
                            *load_counter + 1
                        } else {
                            1
                        };
                    self.last_loads.insert(address, (instruction, block_id, load_counter));
                }
            }
            Instruction::Store { address, value } => {
                let address = self.inserter.function.dfg.resolve(*address);
                let value = self.inserter.function.dfg.resolve(*value);

                self.check_array_aliasing(references, value);

                // If there was another store to this instruction without any (unremoved) loads or
                // function calls in-between, we can remove the previous store.
                if let Some(last_store) = references.last_stores.get(&address) {
                    self.instructions_to_remove.insert(*last_store);
                    if let Some(PerFuncLoadResultContext { load_counter, .. }) =
                        self.load_results.get_mut(&value)
                    {
                        *load_counter -= 1;
                    }
                }

                let known_value = references.get_known_value(value);
                if let Some(known_value) = known_value {
                    let known_value_is_address = known_value == address;
                    if known_value_is_address {
                        if let Some(from_rc) = self.inside_rc_reload {
                            if !from_rc {
                                self.instructions_to_remove.insert(instruction);
                            }
                        } else {
                            self.instructions_to_remove.insert(instruction);
                        }
                        if let Some(PerFuncLoadResultContext { load_counter, .. }) =
                            self.load_results.get_mut(&value)
                        {
                            *load_counter -= 1;
                        }
                    }
                }

                references.set_known_value(address, value);
                references.last_stores.insert(address, instruction);
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
                        self.stores_used_in_calls
                            .entry(*arg)
                            .or_default()
                            .push((instruction, block_id));
                    }
                }
                self.mark_all_unknown(arguments, references);
            }
            _ => (),
        }

        self.track_rc_reload_state(instruction);
    }

    /// Update the `inside_rc_reload` context variable.
    /// To maintain the same value ids, we must run this method inside `analyze_instruction` so that
    /// we operate on the newly pushed instruction id.
    /// This method should also always come after running analysis on the new instruction.
    fn track_rc_reload_state(&mut self, instruction: InstructionId) {
        match &self.inserter.function.dfg[instruction] {
            Instruction::Load { .. } => {
                if self.inside_rc_reload.is_some() {
                    self.inside_rc_reload = Some(true);
                } else {
                    self.inside_rc_reload = Some(false);
                }
            }
            Instruction::Call { arguments, .. } => {
                for arg in arguments {
                    if let Value::Instruction { instruction, .. } =
                        &self.inserter.function.dfg[*arg]
                    {
                        let instruction = &self.inserter.function.dfg[*instruction];
                        if let Instruction::Load { .. } = instruction {
                            if self.inside_rc_reload.is_some() {
                                self.inside_rc_reload = Some(true);
                            } else {
                                self.inside_rc_reload = Some(false);
                            }
                        }
                    }
                }
            }
            Instruction::IncrementRc { .. } | Instruction::DecrementRc { .. } => {
                // Do nothing. We want the reload state to remain the same.
            }
            _ => self.inside_rc_reload = None,
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
        let databus = self.inserter.function.dfg.data_bus.clone();
        self.inserter.function.dfg.data_bus = databus.map_values(|t| self.inserter.resolve(t));
    }

    fn handle_terminator(&mut self, block: BasicBlockId, references: &mut Block) {
        self.inserter.map_terminator_in_place(block);

        let terminator = self.inserter.function.dfg[block].unwrap_terminator();

        terminator.for_each_value(|value| {
            if let Some(PerFuncLoadResultContext { load_counter, .. }) =
                self.load_results.get_mut(&value)
            {
                *load_counter += 1;
            }
        });

        match terminator {
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
}
