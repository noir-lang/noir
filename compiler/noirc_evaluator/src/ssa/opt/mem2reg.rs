//! The goal of the mem2reg SSA optimization pass is to replace any `Load` instructions to known
//! addresses with the value stored at that address, if it is also known. This pass will also remove
//! any `Store` instructions within a block that are no longer needed because no more loads occur in
//! between the Store in question and the next Store.
//!
//! ## How the pass works:
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
//!       - Set the value of the address to `Known(value)`.
//!     - If the address has more than 1 alias:
//!       - Set the value of every possible alias to `Unknown`.
//!     - If the address has 0 aliases:
//!       - Conservatively mark every alias in the block to `Unknown`.
//!   - If the address of the store is not known:
//!     - Conservatively mark every alias in the block to `Unknown`.
//!   - Additionally, if there were no Loads to any alias of the address between this Store and
//!     the previous Store to the same address, the previous store can be removed.
//!   - Remove the instance of the last load instruction to the address and its aliases
//! - On `Instruction::Call { arguments }`:
//!   - If any argument of the call is a reference, set the value of each alias of that
//!     reference to `Unknown`
//!   - Any builtin functions that may return aliases if their input also contains a
//!     reference should be tracked. Examples: `slice_push_back`, `slice_insert`, `slice_remove`, etc.
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

use fxhash::{FxHashMap as HashMap, FxHashSet as HashSet};
use vec_collections::VecSet;

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
            function.mem2reg();
        }
        self
    }
}

impl Function {
    pub(crate) fn mem2reg(&mut self) {
        let mut context = PerFunctionContext::new(self);
        context.mem2reg();
        context.remove_instructions();
        context.update_data_bus();
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
    last_loads: HashMap<ValueId, (InstructionId, BasicBlockId)>,

    /// Track whether a reference was passed into another instruction (e.g. Call)
    /// This is needed to determine whether we can remove a store.
    instruction_input_references: HashSet<ValueId>,

    /// Track whether a reference has been aliased, and store the respective
    /// instruction that aliased that reference.
    /// If that store has been set for removal, we can also remove this instruction.
    aliased_references: HashMap<ValueId, HashSet<InstructionId>>,
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
            aliased_references: HashMap::default(),
            instruction_input_references: HashSet::default(),
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

        let mut all_terminator_values = HashSet::default();
        let mut per_func_block_params: HashSet<ValueId> = HashSet::default();
        for (block_id, _) in self.blocks.iter() {
            let block_params = self.inserter.function.dfg.block_parameters(*block_id);
            per_func_block_params.extend(block_params.iter());
            let terminator = self.inserter.function.dfg[*block_id].unwrap_terminator();
            terminator.for_each_value(|value| all_terminator_values.insert(value));
        }

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

                if !self.last_loads.contains_key(store_address)
                    && !store_alias_used
                    && !is_dereference
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

        // Check whether the store address has an alias that crosses an entry point boundary (e.g. a Call or Return)
        if let Some(expression) = block.expressions.get(store_address) {
            if let Some(aliases) = block.aliases.get(expression) {
                let allocation_aliases_parameter =
                    aliases.any(|alias| reference_parameters.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }

                let allocation_aliases_parameter =
                    aliases.any(|alias| per_func_block_params.contains(&alias));
                if allocation_aliases_parameter == Some(true) {
                    return true;
                }

                let allocation_aliases_instr_input =
                    aliases.any(|alias| self.instruction_input_references.contains(&alias));
                if allocation_aliases_instr_input == Some(true) {
                    return true;
                }

                let allocation_aliases_terminator_args =
                    aliases.any(|alias| all_terminator_values.contains(&alias));
                if allocation_aliases_terminator_args == Some(true) {
                    return true;
                }

                // Check whether there are any aliases whose instructions are not all marked for removal.
                // If there is any alias marked to survive, we should not remove its last store.
                let has_alias_not_marked_for_removal = aliases.any(|alias| {
                    if let Some(alias_instructions) = self.aliased_references.get(&alias) {
                        !alias_instructions.is_subset(&self.instructions_to_remove)
                    } else {
                        false
                    }
                });

                if has_alias_not_marked_for_removal == Some(true) {
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
            // Last loads are tracked per block. During unification we are creating a new block from the current one,
            // so we must clear the last loads of the current block before we return the new block.
            first.last_loads.clear();

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
            self.remove_stores_that_do_not_alias_parameters(&references);
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
                Type::Reference(element) if element.contains_reference() => return,
                Type::Reference(element) => {
                    let empty_aliases = AliasSet::known_empty();
                    let alias_set =
                        aliases.entry(element.as_ref().clone()).or_insert(empty_aliases);
                    alias_set.insert(*param);
                }
                typ if typ.contains_reference() => return,
                _ => continue,
            }
        }

        for aliases in aliases.into_values() {
            let first = aliases.first();
            let first = first.expect("All parameters alias at least themselves or we early return");

            let expression = Expression::Other(first);
            let previous = references.aliases.insert(expression.clone(), aliases.clone());
            assert!(previous.is_none());

            aliases.for_each(|alias| {
                let previous = references.expressions.insert(alias, expression.clone());
                assert!(previous.is_none());
            });
        }
    }

    /// Add all instructions in `last_stores` to `self.instructions_to_remove` which do not
    /// possibly alias any parameters of the given function.
    fn remove_stores_that_do_not_alias_parameters(&mut self, references: &Block) {
        let reference_parameters = self.reference_parameters();

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

        match &self.inserter.function.dfg[instruction] {
            Instruction::Load { address } => {
                let address = *address;

                let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                references.remember_dereference(self.inserter.function, address, result);

                // If the load is known, replace it with the known value and remove the load
                if let Some(value) = references.get_known_value(address) {
                    let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                    self.inserter.map_value(result, value);
                    self.instructions_to_remove.insert(instruction);
                } else {
                    references.mark_value_used(address, self.inserter.function);

                    self.last_loads.insert(address, (instruction, block_id));
                }

                // Check whether the block has a repeat load from the same address (w/ no calls or stores in between the loads).
                // If we do have a repeat load, we can remove the current load and map its result to the previous load's result.
                if let Some(last_load) = references.last_loads.get(&address) {
                    let Instruction::Load { address: previous_address } =
                        &self.inserter.function.dfg[*last_load]
                    else {
                        panic!("Expected a Load instruction here");
                    };
                    let result = self.inserter.function.dfg.instruction_results(instruction)[0];
                    let previous_result =
                        self.inserter.function.dfg.instruction_results(*last_load)[0];
                    if *previous_address == address {
                        self.inserter.map_value(result, previous_result);
                        self.instructions_to_remove.insert(instruction);
                    }
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

                // If there was another store to this instruction without any (unremoved) loads or
                // function calls in-between, we can remove the previous store.
                if !self.aliased_references.contains_key(&address) {
                    if let Some(last_store) = references.last_stores.get(&address) {
                        self.instructions_to_remove.insert(*last_store);
                    }
                }

                references.for_each_alias_of(value, |_, alias| {
                    self.aliased_references.entry(alias).or_default().insert(instruction);
                });

                references.set_known_value(address, value);
                // If we see a store to an address, the last load to that address needs to remain.
                references.keep_last_load_for(address);
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

                let array = *array;
                let array_typ = self.inserter.function.dfg.type_of_value(array);
                if Self::contains_references(&array_typ) {
                    references.for_each_alias_of(array, |_, alias| {
                        self.instruction_input_references.insert(alias);
                    });
                    references.mark_value_used(array, self.inserter.function);

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
                    let array = *array;

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
                // We need to appropriately mark each alias of a reference as being used as a call argument.
                // This prevents us potentially removing a last store from a preceding block or is altered within another function.
                for arg in arguments {
                    references.for_each_alias_of(*arg, |_, alias| {
                        self.instruction_input_references.insert(alias);
                    });
                }
                self.mark_all_unknown(arguments, references);
            }
            Instruction::MakeArray { elements, typ } => {
                // If `array` is an array constant that contains reference types, then insert each element
                // as a potential alias to the array itself.
                if Self::contains_references(typ) {
                    let array = self.inserter.function.dfg.instruction_results(instruction)[0];

                    let expr = Expression::ArrayElement(Box::new(Expression::Other(array)));
                    references.expressions.insert(array, expr.clone());
                    let aliases = references.aliases.entry(expr).or_insert(AliasSet::known_empty());

                    self.add_array_aliases(elements, aliases);
                }
            }
            _ => (),
        }
    }

    /// In order to handle nested arrays we need to recursively search whether there are any references
    /// contained within an array's elements.
    fn add_array_aliases(&self, elements: &im::Vector<ValueId>, aliases: &mut AliasSet) {
        for &element in elements {
            if let Some((elements, _)) = self.inserter.function.dfg.get_array_constant(element) {
                self.add_array_aliases(&elements, aliases);
            } else if self.inserter.function.dfg.value_is_reference(element) {
                aliases.insert(element);
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
            let typ = self.inserter.function.dfg.type_of_value(*value);
            if Self::contains_references(&typ) {
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
        let mut databus = self.inserter.function.dfg.data_bus.clone();
        databus.map_values_mut(|t| self.inserter.resolve(t));
        self.inserter.function.dfg.data_bus = databus;
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
                let mut arg_set = HashMap::default();

                // Add an alias for each reference parameter
                for (parameter, argument) in destination_parameters.iter().zip(arguments) {
                    if self.inserter.function.dfg.value_is_reference(*parameter) {
                        let argument = *argument;

                        if let Some(expression) = references.expressions.get(&argument) {
                            if let Some(aliases) = references.aliases.get_mut(expression) {
                                // The argument reference is possibly aliased by this block parameter
                                aliases.insert(*parameter);

                                // Check if we have seen the same argument
                                let seen_parameters =
                                    arg_set.entry(argument).or_insert_with(VecSet::empty);
                                // Add the current parameter to the parameters we have seen for this argument.
                                // The previous parameters and the current one alias one another.
                                seen_parameters.insert(*parameter);
                            }
                        }
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
                // We need to appropriately mark each alias of a reference as being used as a return terminator argument.
                // This prevents us potentially removing a last store from a preceding block or is altered within another function.
                for return_value in return_values {
                    references.for_each_alias_of(*return_value, |_, alias| {
                        self.instruction_input_references.insert(alias);
                    });
                }
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

    use acvm::{FieldElement, acir::AcirField};
    use im::vector;

    use crate::{
        assert_ssa_snapshot,
        ssa::{
            Ssa,
            function_builder::FunctionBuilder,
            ir::{
                basic_block::BasicBlockId,
                dfg::DataFlowGraph,
                instruction::{
                    ArrayOffset, BinaryOp, Instruction, Intrinsic, TerminatorInstruction,
                },
                map::Id,
                types::{NumericType, Type},
            },
            opt::assert_normalized_ssa_equals,
        },
    };

    #[test]
    fn test_simple() {
        // fn func() {
        //   b0():
        //     v0 = allocate
        //     v1 = make_array [Field 1, Field 2]
        //     store v1 in v0
        //     v2 = load v0
        //     v3 = array_get v2, index u32 1
        //     return v3
        // }

        let func_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("func".into(), func_id);
        let v0 = builder.insert_allocate(Type::Array(Arc::new(vec![Type::field()]), 2));
        let one = builder.numeric_constant(FieldElement::one(), NumericType::length_type());
        let two = builder.field_constant(FieldElement::one());

        let element_type = Arc::new(vec![Type::field()]);
        let array_type = Type::Array(element_type, 2);
        let v1 = builder.insert_make_array(vector![one, two], array_type.clone());

        builder.insert_store(v0, v1);
        let v2 = builder.insert_load(v0, array_type);
        let offset = ArrayOffset::None;
        let v3 = builder.insert_array_get(v2, one, offset, Type::field());
        builder.terminate_with_return(vec![v3]);

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
        let main = ssa.main();

        let instructions = main.dfg[main.entry_block()].instructions();
        assert_eq!(instructions.len(), 6); // The final return is not counted

        // All loads should be removed
        // The first store is not removed as it is used as a nested reference in another store.
        // We would need to track whether the store where `v0` is the store value gets removed to know whether
        // to remove it.
        // The final store in b1 is removed as no loads are done within any blocks
        // to the stored values.
        let expected = "
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
        ";

        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, expected);
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
        //       v12 = load v11
        //       v13 = eq v12, Field 2
        //       constrain v11 == Field 2
        //       return
        //   }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let v0 = builder.insert_allocate(Type::field());
        let zero = builder.field_constant(0u128);
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
        let two = builder.field_constant(2u128);
        builder.insert_store(v5, two);
        let one = builder.field_constant(1u128);
        let v3_plus_one = builder.insert_binary(v3, BinaryOp::Add { unchecked: false }, one);
        builder.terminate_with_jmp(b1, vec![v3_plus_one]);

        builder.switch_to_block(b3);
        let v9 = builder.insert_load(v0, Type::field());
        let _ = builder.insert_binary(v9, BinaryOp::Eq, two);

        builder.insert_constrain(v9, two, None);
        let v11 = builder.insert_load(v2, v2_type);
        let v12 = builder.insert_load(v11, Type::field());

        builder.insert_constrain(v12, two, None);
        builder.terminate_with_return(vec![]);

        let ssa = builder.finish();

        // We expect the same result as above.
        let ssa = ssa.mem2reg();

        let main = ssa.main();
        assert_eq!(main.reachable_blocks().len(), 4);

        // The stores from the original SSA should remain
        assert_eq!(count_stores(main.entry_block(), &main.dfg), 2);
        assert_eq!(count_stores(b2, &main.dfg), 1);

        assert_eq!(count_loads(b2, &main.dfg), 1);
        assert_eq!(count_loads(b3, &main.dfg), 3);
    }

    #[test]
    fn parameter_alias() {
        // Do not assume parameters are not aliased to each other.
        // The load below shouldn't be removed since `v0` could
        // be aliased to `v1`.
        //
        // fn main f0 {
        //   b0(v0: &mut Field, v1: &mut Field):
        //     store Field 0 at v0
        //     store Field 1 at v1
        //     v4 = load v0
        //     constrain v4 == Field 1
        //     return
        // }
        let main_id = Id::test_new(0);
        let mut builder = FunctionBuilder::new("main".into(), main_id);

        let field_ref = Type::Reference(Arc::new(Type::field()));
        let v0 = builder.add_parameter(field_ref.clone());
        let v1 = builder.add_parameter(field_ref.clone());

        let zero = builder.field_constant(0u128);
        let one = builder.field_constant(0u128);
        builder.insert_store(v0, zero);
        builder.insert_store(v1, one);

        let v4 = builder.insert_load(v0, Type::field());
        builder.insert_constrain(v4, one, None);
        builder.terminate_with_return(Vec::new());

        let ssa = builder.finish();
        let main = ssa.main();
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 1);

        // No change expected
        let ssa = ssa.mem2reg();
        let main = ssa.main();
        assert_eq!(count_loads(main.entry_block(), &main.dfg), 1);
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
          b0(v0: &mut Field):
            return
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();
        // We expect the program to be unchanged
        assert_normalized_ssa_equals(ssa, src);
    }

    #[test]
    fn keep_repeat_loads_with_alias_store() {
        // v7, v8, and v9 alias one another. We want to make sure that a repeat load to v7 with a store
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

        let ssa = Ssa::from_str(src).unwrap();

        let ssa = ssa.mem2reg();
        // We expect the program to be unchanged
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();

        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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
          b0(v0: u1):
            v1 = call f1(v0) -> [&mut u1; 1]
            v3 = array_get v1, index u32 0 -> &mut u1
            store u32 1 at v3
            v5 = load v3 -> u1
            return v5
        }
        brillig(inline_always) fn foo f1 {
          b0(v0: u1):
            v1 = allocate -> &mut u1
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
            v9 = make_array [v1] : [&mut u1; 1]
            return v9
          b5():
            jmp b1()
        }
        ";

        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
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
        let ssa = Ssa::from_str(src).unwrap();
        let ssa = ssa.mem2reg();
        assert_normalized_ssa_equals(ssa, src);
    }
}
